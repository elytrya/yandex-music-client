use std::io::{BufRead, BufReader, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use super::state::{Peer, PeerInfo, TogetherState, HOST_ID};
use super::wire::{broadcast, clean_nick, emit_status, spawn_writer};

pub fn listen(app: AppHandle, listener: TcpListener, alive: Arc<AtomicBool>) {
    thread::spawn(move || {
        while alive.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => accept(&app, stream),
                Err(ref error) if error.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(150));
                }
                Err(_) => thread::sleep(Duration::from_millis(300)),
            }
        }
    });
}

fn accept(app: &AppHandle, stream: TcpStream) {
    let _ = stream.set_nodelay(true);

    let reader = match stream.try_clone() {
        Ok(value) => value,
        Err(_) => return,
    };
    let writer = match stream.try_clone() {
        Ok(value) => value,
        Err(_) => return,
    };

    let (tx, rx) = channel::<String>();

    let id = {
        let state = app.state::<TogetherState>();
        let mut room = state.inner.lock().unwrap();
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

    spawn_writer(writer, rx);

    let reader_app = app.clone();
    thread::spawn(move || {
        let lines = BufReader::new(reader).lines();
        for line in lines {
            match line {
                Ok(line) => handle(&reader_app, id, &line),
                Err(_) => break,
            }
        }
        drop_peer(&reader_app, id);
    });
}

fn handle(app: &AppHandle, from: u64, line: &str) {
    let message: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(_) => return,
    };

    match message.get("t").and_then(Value::as_str) {
        Some("hello") => {
            let nick = clean_nick(message.get("nick").and_then(Value::as_str).unwrap_or(""));
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
            let _ = app.emit(
                "together://message",
                json!({ "from": from, "nick": nick, "payload": payload }),
            );
        }
        _ => {}
    }
}

pub fn sync_roster(app: &AppHandle) {
    {
        let state = app.state::<TogetherState>();
        let mut room = state.inner.lock().unwrap();

        let mut roster = vec![PeerInfo {
            id: HOST_ID,
            nick: room.nick.clone(),
        }];
        let mut guests: Vec<PeerInfo> = room.peers.values().map(|peer| peer.info.clone()).collect();
        guests.sort_by_key(|peer| peer.id);
        roster.extend(guests);

        room.roster = roster.clone();
        broadcast(&room, &json!({ "t": "peers", "peers": roster }), None);
    }

    emit_status(app);
}

fn drop_peer(app: &AppHandle, id: u64) {
    {
        let state = app.state::<TogetherState>();
        let mut room = state.inner.lock().unwrap();
        if room.peers.remove(&id).is_none() {
            return;
        }
    }

    sync_roster(app);
}
