use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tauri::{AppHandle, Emitter};
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::{interval, timeout, MissedTickBehavior};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use together_crypto::encoding::{from_base64, to_base64};
use together_crypto::identity;
use together_crypto::room_key::RoomSecrets;

use super::wire::{describe, is_fatal, ClientMessage, Peer, ServerMessage, PROTOCOL_VERSION};
use super::{crypto, keys, Outgoing, PeerInfo, RelayState, RelayStatus};

const REPLY: Duration = Duration::from_secs(10);

const PING: Duration = Duration::from_secs(15);

const EVENT_STATUS: &str = "together://relay-status";
const EVENT_MESSAGE: &str = "together://relay-message";
const EVENT_JOINED: &str = "together://relay-joined";
const EVENT_CLOSED: &str = "together://relay-closed";
const EVENT_LOG: &str = "together://log";

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type Writer = SplitSink<Socket, Message>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intent {
    Create,
    Join,
}

#[derive(Clone, Debug)]
pub struct Options {

    pub address: String,

    pub invite: String,
    pub nick: String,

    pub phrase: String,
    pub intent: Intent,
}

struct Entered {
    id: u64,
    room: String,
    host: u64,
}

pub async fn run(
    app: AppHandle,
    relay: RelayState,
    options: Options,
    orders: UnboundedReceiver<Outgoing>,
    epoch: u64,
) {
    let outcome = serve(&app, &relay, &options, orders, epoch).await;

    if !relay.owns(epoch) {

        return;
    }

    let reason = match outcome {
        Ok(()) => String::from("соединение с комнатой закрыто"),
        Err(text) => text,
    };

    relay.finish(epoch);
    note(&app, reason.clone());
    let _ = app.emit(EVENT_CLOSED, json!({ "reason": reason }));
    announce(&app, &relay.status());
}

async fn serve(
    app: &AppHandle,
    relay: &RelayState,
    options: &Options,
    mut orders: UnboundedReceiver<Outgoing>,
    epoch: u64,
) -> Result<(), String> {
    let secrets = crypto::secrets(&options.invite)?;

    note(app, format!("подключаемся к {}", options.address));
    let (mut socket, _) = connect_async(options.address.clone())
        .await
        .map_err(|error| format!("не удалось подключиться к серверу: {error}"))?;

    handshake(app, &mut socket, &options.phrase).await?;
    let entered = enter(&mut socket, options, &secrets).await?;

    let status = RelayStatus {
        connected: true,
        address: options.address.clone(),
        room: entered.room,
        invite: options.invite.clone(),
        nick: options.nick.clone(),
        self_id: entered.id,
        host: entered.host,
        peers: Vec::new(),
    };

    let status = relay.update(|current| *current = status);
    announce(app, &status);
    note(app, format!("вошли в комнату, наш номер {}", entered.id));

    pump(app, relay, socket, &mut orders, &secrets, epoch).await
}

async fn handshake(app: &AppHandle, socket: &mut Socket, phrase: &str) -> Result<(), String> {
    let key = keys::signing_key(phrase)?;
    let public = to_base64(&key.verifying_key().to_bytes());

    send(
        socket,
        &ClientMessage::Auth {
            public,
            v: PROTOCOL_VERSION,
        },
    )
    .await?;

    let nonce = match receive(socket).await? {
        ServerMessage::Challenge { nonce } => nonce,
        ServerMessage::Error { code, text } => return Err(describe(&code, &text)),
        _ => return Err(String::from("сервер не прислал nonce")),
    };

    let raw = from_base64(&nonce).ok_or_else(|| String::from("nonce пришёл испорченным"))?;
    let signature = identity::sign_nonce(&key, &raw);

    send(
        socket,
        &ClientMessage::Proof {
            sig: to_base64(&signature.to_bytes()),
        },
    )
    .await?;

    match receive(socket).await? {
        ServerMessage::Session { ttl } => {
            note(app, format!("сервер принял подпись, сессия на {ttl} секунд"));
            Ok(())
        }
        ServerMessage::Error { code, text } => Err(describe(&code, &text)),
        _ => Err(String::from("сервер не выдал сессию")),
    }
}

async fn enter(
    socket: &mut Socket,
    options: &Options,
    secrets: &RoomSecrets,
) -> Result<Entered, String> {
    let request = match options.intent {
        Intent::Create => ClientMessage::RoomCreate {
            ticket: secrets.ticket.clone(),
            nick: options.nick.clone(),
        },
        Intent::Join => ClientMessage::RoomJoin {
            invite: secrets.ticket.clone(),
            nick: options.nick.clone(),
        },
    };

    send(socket, &request).await?;

    let mut room = String::new();
    loop {
        match receive(socket).await? {
            ServerMessage::Room { id, invite } => {
                if invite.is_some() {

                    return Err(String::from(
                        "сервер выдал свой код приглашения: такую комнату клиент не откроет",
                    ));
                }
                room = id;
            }
            ServerMessage::Welcome {
                id,
                room: given,
                host,
            } => {
                return Ok(Entered {
                    id,
                    room: if room.is_empty() { given } else { room },
                    host,
                });
            }
            ServerMessage::Error { code, text } => return Err(describe(&code, &text)),
            _ => {}
        }
    }
}

async fn pump(
    app: &AppHandle,
    relay: &RelayState,
    socket: Socket,
    orders: &mut UnboundedReceiver<Outgoing>,
    secrets: &RoomSecrets,
    epoch: u64,
) -> Result<(), String> {
    let (mut writer, mut reader) = socket.split();

    let mut beat = interval(PING);
    beat.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        if !relay.owns(epoch) {
            let _ = writer.send(Message::Text(ClientMessage::RoomLeave.to_frame())).await;
            let _ = writer.close().await;
            return Ok(());
        }

        tokio::select! {
            frame = reader.next() => {
                let Some(frame) = frame else {
                    return Ok(());
                };

                let frame = frame.map_err(|error| format!("связь оборвалась: {error}"))?;
                match frame {
                    Message::Text(text) => {
                        if let Ok(message) = serde_json::from_str::<ServerMessage>(&text) {
                            if !accept(app, relay, secrets, message) {
                                return Ok(());
                            }
                        }
                    }
                    Message::Close(_) => return Ok(()),
                    _ => {}
                }
            }
            order = orders.recv() => {
                let Some(order) = order else {
                    let _ = writer.send(Message::Text(ClientMessage::RoomLeave.to_frame())).await;
                    let _ = writer.close().await;
                    return Ok(());
                };

                match order {
                    Outgoing::Payload(value) => {
                        let sealed = crypto::seal(&secrets.key, &value)?;
                        write(&mut writer, &ClientMessage::Msg { payload: sealed }).await?;
                    }
                    Outgoing::Handoff(to) => {
                        write(&mut writer, &ClientMessage::RoomHandoff { to }).await?;
                    }
                    Outgoing::Leave => {
                        let _ = writer.send(Message::Text(ClientMessage::RoomLeave.to_frame())).await;
                        let _ = writer.close().await;
                        return Ok(());
                    }
                }
            }
            _ = beat.tick() => {
                write(&mut writer, &ClientMessage::Ping).await?;
            }
        }
    }
}

fn accept(
    app: &AppHandle,
    relay: &RelayState,
    secrets: &RoomSecrets,
    message: ServerMessage,
) -> bool {
    match message {
        ServerMessage::Msg { from, nick, payload } => {
            match crypto::open(&secrets.key, &payload) {
                Ok(value) => {
                    let _ = app.emit(
                        EVENT_MESSAGE,
                        json!({ "from": from, "nick": nick, "payload": value }),
                    );
                }
                Err(reason) => note(app, format!("сообщение от {from} не открылось: {reason}")),
            }
            true
        }
        ServerMessage::Peers { peers, host } => {
            roster(app, relay, peers, host);
            true
        }
        ServerMessage::Host { id } => {
            let status = relay.update(|status| status.host = id);
            announce(app, &status);
            note(app, format!("комнату ведёт участник {id}"));
            true
        }
        ServerMessage::Session { ttl } => {
            note(app, format!("сессия продлена на {ttl} секунд"));
            true
        }
        ServerMessage::Error { code, text } => {
            note(app, describe(&code, &text));
            !is_fatal(&code)
        }
        _ => true,
    }
}

fn roster(app: &AppHandle, relay: &RelayState, peers: Vec<Peer>, host: u64) {
    let before = relay.status();

    let fresh: Vec<PeerInfo> = peers
        .into_iter()
        .map(|peer| PeerInfo {
            id: peer.id,
            nick: peer.nick,
        })
        .collect();

    for peer in &fresh {
        let known = before.peers.iter().any(|old| old.id == peer.id);
        if !known && peer.id != before.self_id {
            let _ = app.emit(EVENT_JOINED, json!({ "id": peer.id, "nick": peer.nick }));
        }
    }

    let status = relay.update(|status| {
        status.peers = fresh;
        status.host = host;
    });

    announce(app, &status);
}

async fn send(socket: &mut Socket, message: &ClientMessage) -> Result<(), String> {
    socket
        .send(Message::Text(message.to_frame()))
        .await
        .map_err(|error| format!("не удалось отправить сообщение: {error}"))
}

async fn write(writer: &mut Writer, message: &ClientMessage) -> Result<(), String> {
    writer
        .send(Message::Text(message.to_frame()))
        .await
        .map_err(|error| format!("не удалось отправить сообщение: {error}"))
}

async fn receive(socket: &mut Socket) -> Result<ServerMessage, String> {
    loop {
        let frame = timeout(REPLY, socket.next())
            .await
            .map_err(|_| String::from("сервер не ответил вовремя"))?
            .ok_or_else(|| String::from("сервер закрыл соединение"))?
            .map_err(|error| format!("связь оборвалась: {error}"))?;

        match frame {
            Message::Text(text) => {
                if let Ok(message) = serde_json::from_str::<ServerMessage>(&text) {
                    return Ok(message);
                }
            }
            Message::Close(_) => return Err(String::from("сервер закрыл соединение")),
            _ => {}
        }
    }
}

fn announce(app: &AppHandle, status: &RelayStatus) {
    let _ = app.emit(EVENT_STATUS, status.clone());
}

fn note(app: &AppHandle, text: impl Into<String>) {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or_default();

    let _ = app.emit(
        EVENT_LOG,
        json!({ "time": time, "scope": "релей", "text": text.into() }),
    );
}
