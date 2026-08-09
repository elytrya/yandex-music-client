use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use super::log::write as tlog;
use super::state::{Peer, PeerInfo, TogetherState, HOST_ID};
use super::wire::{broadcast, clean_nick, emit_status, read_lines, spawn_writer};

pub fn listen(app: AppHandle, listener: TcpListener, alive: Arc<AtomicBool>, session: u64) {
    thread::spawn(move || {
        tlog(&app, "host", "ждём входящие подключения");

        while alive.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, from)) => {
                    tlog(&app, "host", &format!("входящее подключение с {from}"));
                    accept(&app, stream, session);
                }
                Err(ref error) if error.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(150));
                }
                Err(error) => {
                    tlog(&app, "host", &format!("ошибка приёма подключения: {error}"));
                    thread::sleep(Duration::from_millis(300));
                }
            }
        }

        tlog(&app, "host", "комната закрыта, слушатель остановлен");
    });
}

fn accept(app: &AppHandle, stream: TcpStream, session: u64) {
    // на windows принятый сокет наследует неблокирующий режим слушателя,
    // и первое же чтение падает с WouldBlock — возвращаем блокирующий режим
    if let Err(error) = stream.set_nonblocking(false) {
        tlog(app, "host", &format!("не удалось настроить сокет: {error}"));
        return;
    }
    let _ = stream.set_nodelay(true);

    let reader = match stream.try_clone() {
        Ok(value) => value,
        Err(error) => {
            tlog(app, "host", &format!("не удалось клонировать сокет: {error}"));
            return;
        }
    };
    let writer = match stream.try_clone() {
        Ok(value) => value,
        Err(error) => {
            tlog(app, "host", &format!("не удалось клонировать сокет: {error}"));
            return;
        }
    };

    let (tx, rx) = channel::<String>();

    let id = {
        let state = app.state::<TogetherState>();
        let mut room = state.inner.lock().unwrap();

        if room.session != session {
            tlog(app, "host", "подключение отброшено: комната уже пересоздана");
            return;
        }

        room.next_id += 1;
        let id = room.next_id;
        room.peers.insert(
            id,
            Peer {
                info: PeerInfo {
                    id,
                    nick: format!("гость {id}"),
                },
                tx,
                stream,
            },
        );
        id
    };

    tlog(app, "host", &format!("участник #{id} в комнате, ждём hello"));
    spawn_writer(app.clone(), "host", writer, rx);

    let reader_app = app.clone();
    thread::spawn(move || {
        read_lines(&reader_app, "host", reader, |line| {
            handle(&reader_app, id, line);
        });
        drop_peer(&reader_app, id, session);
    });
}

fn handle(app: &AppHandle, from: u64, line: &str) {
    let message: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(error) => {
            tlog(app, "host", &format!("строка от #{from} не разобралась: {error}"));
            return;
        }
    };

    match message.get("t").and_then(Value::as_str) {
        Some("hello") => {
            let nick = clean_nick(message.get("nick").and_then(Value::as_str).unwrap_or(""));
            tlog(app, "host", &format!("#{from} представился как {nick}"));

            {
                let state = app.state::<TogetherState>();
                let mut room = state.inner.lock().unwrap();
                if let Some(peer) = room.peers.get_mut(&from) {
                    peer.info.nick = nick.clone();
                    let _ = peer
                        .tx
                        .send(json!({ "t": "welcome", "id": from }).to_string());
                }
            }

            sync_roster(app);
            let _ = app.emit("together://joined", json!({ "id": from, "nick": nick }));
        }
        Some("msg") => {
            let payload = message.get("payload").cloned().unwrap_or(Value::Null);
            let nick = {
                let state = app.state::<TogetherState>();
                let room = state.inner.lock().unwrap();
                let nick = room
                    .peers
                    .get(&from)
                    .map(|peer| peer.info.nick.clone())
                    .unwrap_or_default();
                broadcast(
                    &room,
                    &json!({ "t": "msg", "from": from, "nick": nick, "payload": payload }),
                    Some(from),
                );
                nick
            };

            tlog(app, "host", &format!("сообщение от {nick} (#{from})"));
            let _ = app.emit(
                "together://message",
                json!({ "from": from, "nick": nick, "payload": payload }),
            );
        }
        other => tlog(
            app,
            "host",
            &format!("непонятный тип сообщения от #{from}: {other:?}"),
        ),
    }
}

pub fn sync_roster(app: &AppHandle) {
    let count = {
        let state = app.state::<TogetherState>();
        let mut room = state.inner.lock().unwrap();

        let mut roster = vec![PeerInfo {
            id: HOST_ID,
            nick: room.nick.clone(),
        }];
        let mut guests: Vec<PeerInfo> = room.peers.values().map(|peer| peer.info.clone()).collect();
        guests.sort_by_key(|peer| peer.id);
        roster.extend(guests);

        let count = roster.len();
        room.roster = roster.clone();
        broadcast(&room, &json!({ "t": "peers", "peers": roster }), None);
        count
    };

    tlog(app, "host", &format!("в комнате участников: {count}"));
    emit_status(app);
}

fn drop_peer(app: &AppHandle, id: u64, session: u64) {
    {
        let state = app.state::<TogetherState>();
        let mut room = state.inner.lock().unwrap();

        if room.session != session || room.peers.remove(&id).is_none() {
            return;
        }
    }

    tlog(app, "host", &format!("участник #{id} отключился"));
    sync_roster(app);
}
