use std::net::{Shutdown, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use super::guest;
use super::host;
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
}

fn bind(port: u16) -> Result<TcpListener, String> {
    let mut last = String::new();

    for attempt in 0..6 {
        match TcpListener::bind(("0.0.0.0", port)) {
            Ok(listener) => return Ok(listener),
            Err(error) => {
                last = error.to_string();
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
    let listener = bind(port)?;
    listener
        .set_nonblocking(true)
        .map_err(|_| "Не удалось настроить сокет".to_string())?;

    let alive = Arc::new(AtomicBool::new(true));

    {
        let state = app.state::<TogetherState>();
        let mut room = state.inner.lock().unwrap();
        room.mode = Mode::Host;
        room.nick = clean_nick(&nick);
        room.port = port;
        room.address = local_ip();
        room.self_id = HOST_ID;
        room.next_id = 0;
        room.alive = Some(alive.clone());
    }

    host::listen(app.clone(), listener, alive);
    host::sync_roster(&app);

    Ok(snapshot(&app))
}

#[tauri::command]
pub fn together_join(
    app: AppHandle,
    address: String,
    nick: String,
) -> Result<TogetherStatus, String> {
    let target = normalize_address(&address)?;

    shutdown(&app);
    guest::connect(&app, target, clean_nick(&nick))?;

    Ok(snapshot(&app))
}

#[tauri::command]
pub fn together_leave(app: AppHandle) -> TogetherStatus {
    shutdown(&app);
    emit_status(&app);
    snapshot(&app)
}

#[tauri::command]
pub fn together_send(app: AppHandle, payload: Value) -> Result<(), String> {
    let state = app.state::<TogetherState>();
    let room = state.inner.lock().unwrap();

    match room.mode {
        Mode::Host => {
            let nick = room.nick.clone();
            broadcast(
                &room,
                &json!({ "t": "msg", "from": HOST_ID, "nick": nick, "payload": payload }),
                None,
            );
            Ok(())
        }
        Mode::Guest => {
            let out = room
                .out
                .as_ref()
                .ok_or_else(|| "Нет соединения с комнатой".to_string())?;
            out.send(json!({ "t": "msg", "payload": payload }).to_string())
                .map_err(|_| "Соединение потеряно".to_string())
        }
        Mode::Off => Ok(()),
    }
}

#[tauri::command]
pub fn together_status(app: AppHandle) -> TogetherStatus {
    snapshot(&app)
}
