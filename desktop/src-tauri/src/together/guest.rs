use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use super::state::{Mode, PeerInfo, TogetherState};
use super::wire::{emit_status, spawn_writer};

pub fn connect(app: &AppHandle, target: SocketAddr, nick: String) -> Result<(), String> {
    let stream = TcpStream::connect_timeout(&target, Duration::from_secs(8))
        .map_err(|_| "Комната не отвечает. Проверьте адрес и сеть".to_string())?;
    let _ = stream.set_nodelay(true);

    let reader = stream
        .try_clone()
        .map_err(|_| "Не удалось открыть соединение".to_string())?;
    let writer = stream
        .try_clone()
        .map_err(|_| "Не удалось открыть соединение".to_string())?;

    let (tx, rx) = channel::<String>();

    {
        let state = app.state::<TogetherState>();
        let mut room = state.inner.lock().unwrap();
        room.mode = Mode::Guest;
        room.nick = nick.clone();
        room.port = target.port();
        room.address = Some(target.ip().to_string());
        room.self_id = 0;
        room.roster = Vec::new();
        room.out = Some(tx.clone());
        room.stream = Some(stream);
    }

    spawn_writer(writer, rx);
    let _ = tx.send(json!({ "t": "hello", "nick": nick }).to_string());
    emit_status(app);

    let reader_app = app.clone();
    thread::spawn(move || {
        let lines = BufReader::new(reader).lines();
        for line in lines {
            match line {
                Ok(line) => handle(&reader_app, &line),
                Err(_) => break,
            }
        }
        closed(&reader_app);
    });

    Ok(())
}

fn handle(app: &AppHandle, line: &str) {
    let message: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(_) => return,
    };

    match message.get("t").and_then(Value::as_str) {
        Some("welcome") => {
            {
                let state = app.state::<TogetherState>();
                let mut room = state.inner.lock().unwrap();
                room.self_id = message.get("id").and_then(Value::as_u64).unwrap_or(0);
            }
            emit_status(app);
        }
        Some("peers") => {
            let peers: Vec<PeerInfo> = message
                .get("peers")
                .and_then(|value| serde_json::from_value(value.clone()).ok())
                .unwrap_or_default();
            {
                let state = app.state::<TogetherState>();
                let mut room = state.inner.lock().unwrap();
                room.roster = peers.clone();
            }
            emit_status(app);
        }
        Some("msg") => {
            let _ = app.emit(
                "together://message",
                json!({
                    "from": message.get("from").and_then(Value::as_u64).unwrap_or(0),
                    "nick": message.get("nick").and_then(Value::as_str).unwrap_or(""),
                    "payload": message.get("payload").cloned().unwrap_or(Value::Null),
                }),
            );
        }
        _ => {}
    }
}

fn closed(app: &AppHandle) {
    let changed = {
        let state = app.state::<TogetherState>();
        let mut room = state.inner.lock().unwrap();
        if room.mode == Mode::Guest {
            room.mode = Mode::Off;
            room.address = None;
            room.roster = Vec::new();
            room.out = None;
            room.stream = None;
            room.self_id = 0;
            true
        } else {
            false
        }
    };

    if changed {
        let _ = app.emit(
            "together://closed",
            json!({ "reason": "Соединение с комнатой закрыто" }),
        );
        emit_status(app);
    }
}
