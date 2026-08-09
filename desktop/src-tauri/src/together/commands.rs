use std::net::{Shutdown, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use super::guest;
use super::host;
use super::log::{path as log_path, write as tlog};
use super::state::{Mode, TogetherState, TogetherStatus, DEFAULT_PORT, HOST_ID};
use super::wire::{broadcast, clean_nick, emit_status, local_ip, normalize_address, snapshot};

fn shutdown(app: &AppHandle) {
    let state = app.state::<TogetherState>();
    let mut room = state.inner.lock().unwrap();

    if let Some(alive) = room.alive.take() {
        alive.store(false, Ordering::Relaxed);
    }

    for (_, peer) in room.peers.drain() {
        let _ = peer.stream.shutdown(Shutdown::Both);
    }

    if let Some(stream) = room.stream.take() {
        let _ = stream.shutdown(Shutdown::Both);
    }

    room.mode = Mode::Off;
    room.address = None;
    room.roster = Vec::new();
    room.out = None;
    room.self_id = 0;
    room.next_id = 0;
    room.session += 1;
}

fn bind(app: &AppHandle, port: u16) -> Result<TcpListener, String> {
    let mut last = String::new();

    for attempt in 0..6 {
        match TcpListener::bind(("0.0.0.0", port)) {
            Ok(listener) => return Ok(listener),
            Err(error) => {
                last = error.to_string();
                tlog(
                    app,
                    "host",
                    &format!("попытка {} занять порт {port}: {last}", attempt + 1),
                );
                if attempt < 5 {
                    thread::sleep(Duration::from_millis(200));
                }
            }
        }
    }

    Err(format!("Не удалось занять порт {port}: {last}"))
}

#[tauri::command]
pub fn together_host(
    app: AppHandle,
    nick: String,
    port: Option<u16>,
) -> Result<TogetherStatus, String> {
    shutdown(&app);

    let port = port.unwrap_or(DEFAULT_PORT);
    tlog(&app, "host", &format!("создаём комнату на порту {port}"));

    let listener = bind(&app, port)?;
    listener
        .set_nonblocking(true)
        .map_err(|_| "Не удалось настроить сокет".to_string())?;

    let alive = Arc::new(AtomicBool::new(true));

    let session = {
        let state = app.state::<TogetherState>();
        let mut room = state.inner.lock().unwrap();
        room.mode = Mode::Host;
        room.nick = clean_nick(&nick);
        room.port = port;
        room.address = local_ip();
        room.self_id = HOST_ID;
        room.next_id = 0;
        room.alive = Some(alive.clone());
        room.session
    };

    let status = snapshot(&app);
    tlog(
        &app,
        "host",
        &format!(
            "комната готова: {}:{port}, имя {}",
            status.address.clone().unwrap_or_else(|| "?".to_string()),
            status.nick
        ),
    );

    host::listen(app.clone(), listener, alive, session);
    host::sync_roster(&app);

    Ok(snapshot(&app))
}

#[tauri::command]
pub fn together_join(
    app: AppHandle,
    address: String,
    nick: String,
) -> Result<TogetherStatus, String> {
    tlog(&app, "guest", &format!("запрошено подключение к {address}"));

    let target = normalize_address(&address)?;

    shutdown(&app);
    guest::connect(&app, target, clean_nick(&nick))?;

    Ok(snapshot(&app))
}

#[tauri::command]
pub fn together_leave(app: AppHandle) -> TogetherStatus {
    tlog(&app, "room", "выходим из комнаты");
    shutdown(&app);
    emit_status(&app);
    snapshot(&app)
}

#[tauri::command]
pub fn together_send(app: AppHandle, payload: Value) -> Result<(), String> {
    let kind = payload
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("?")
        .to_string();

    let sent = {
        let state = app.state::<TogetherState>();
        let room = state.inner.lock().unwrap();
        let mode = room.mode;

        match mode {
            Mode::Host => {
                let nick = room.nick.clone();
                let count = room.peers.len();
                broadcast(
                    &room,
                    &json!({ "t": "msg", "from": HOST_ID, "nick": nick, "payload": payload }),
                    None,
                );
                Some(format!("отправлено участникам ({count}): {kind}"))
            }
            Mode::Guest => {
                let out = room
                    .out
                    .as_ref()
                    .ok_or_else(|| "Нет соединения с комнатой".to_string())?;
                out.send(json!({ "t": "msg", "payload": payload }).to_string())
                    .map_err(|_| "Соединение потеряно".to_string())?;
                Some(format!("отправлено хосту: {kind}"))
            }
            Mode::Off => None,
        }
    };

    if let Some(text) = sent {
        tlog(&app, "room", &text);
    }

    Ok(())
}

#[tauri::command]
pub fn together_status(app: AppHandle) -> TogetherStatus {
    snapshot(&app)
}

#[tauri::command]
pub fn together_log_path(app: AppHandle) -> String {
    log_path(&app)
        .map(|value| value.display().to_string())
        .unwrap_or_default()
}
