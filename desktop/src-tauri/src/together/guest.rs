use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use super::log::write as tlog;
use super::state::{Mode, PeerInfo, TogetherState};
use super::wire::{emit_status, read_lines, spawn_writer};

pub fn connect(app: &AppHandle, target: SocketAddr, nick: String) -> Result<(), String> {
    tlog(app, "guest", &format!("подключаемся к {target}"));

    let stream = TcpStream::connect_timeout(&target, Duration::from_secs(8)).map_err(|error| {
        tlog(app, "guest", &format!("не дозвонились: {error}"));
        "Комната не отвечает. Проверьте адрес и сеть".to_string()
    })?;

    // после connect_timeout сокет может остаться неблокирующим, а чтение ждёт строки
    if let Err(error) = stream.set_nonblocking(false) {
        tlog(app, "guest", &format!("не удалось настроить сокет: {error}"));
        return Err("Не удалось открыть соединение".to_string());
    }
    let _ = stream.set_nodelay(true);

    let reader = stream
        .try_clone()
        .map_err(|_| "Не удалось открыть соединение".to_string())?;
    let writer = stream
        .try_clone()
        .map_err(|_| "Не удалось открыть соединение".to_string())?;

    let (tx, rx) = channel::<String>();

    let session = {
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
        room.session
    };

    spawn_writer(app.clone(), "guest", writer, rx);
    let _ = tx.send(json!({ "t": "hello", "nick": nick }).to_string());
    tlog(app, "guest", &format!("соединение открыто, представились как {nick}"));
    emit_status(app);

    let reader_app = app.clone();
    thread::spawn(move || {
        read_lines(&reader_app, "guest", reader, |line| {
            handle(&reader_app, line);
        });
        closed(&reader_app, session);
    });

    Ok(())
}

fn handle(app: &AppHandle, line: &str) {
    let message: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(error) => {
            tlog(app, "guest", &format!("строка от хоста не разобралась: {error}"));
            return;
        }
    };

    match message.get("t").and_then(Value::as_str) {
        Some("welcome") => {
            let id = message.get("id").and_then(Value::as_u64).unwrap_or(0);
            {
                let state = app.state::<TogetherState>();
                let mut room = state.inner.lock().unwrap();
                room.self_id = id;
            }

            tlog(app, "guest", &format!("хост принял, наш номер #{id}"));
            emit_status(app);
        }
        Some("peers") => {
            let peers: Vec<PeerInfo> = message
                .get("peers")
                .and_then(|value| serde_json::from_value(value.clone()).ok())
                .unwrap_or_default();

            tlog(app, "guest", &format!("список участников: {}", peers.len()));

            {
                let state = app.state::<TogetherState>();
                let mut room = state.inner.lock().unwrap();
                room.roster = peers;
            }

            emit_status(app);
        }
        Some("msg") => {
            let nick = message.get("nick").and_then(Value::as_str).unwrap_or("");
            tlog(app, "guest", &format!("команда от {nick}"));

            let _ = app.emit(
                "together://message",
                json!({
                    "from": message.get("from").and_then(Value::as_u64).unwrap_or(0),
                    "nick": nick,
                    "payload": message.get("payload").cloned().unwrap_or(Value::Null),
                }),
            );
        }
        other => tlog(
            app,
            "guest",
            &format!("непонятный тип сообщения от хоста: {other:?}"),
        ),
    }
}

fn closed(app: &AppHandle, session: u64) {
    let changed = {
        let state = app.state::<TogetherState>();
        let mut room = state.inner.lock().unwrap();

        // старый поток не должен гасить свежее подключение
        if room.session != session || room.mode != Mode::Guest {
            false
        } else {
            room.mode = Mode::Off;
            room.address = None;
            room.roster = Vec::new();
            room.out = None;
            room.stream = None;
            room.self_id = 0;
            true
        }
    };

    if changed {
        tlog(app, "guest", "соединение с комнатой закрыто");
        let _ = app.emit(
            "together://closed",
            json!({ "reason": "Соединение с комнатой закрыто" }),
        );
        emit_status(app);
    } else {
        tlog(app, "guest", "старое соединение завершилось, текущее не трогаем");
    }
}
