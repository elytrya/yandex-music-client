use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use super::log::write as tlog;
use super::state::{Room, TogetherState, TogetherStatus, DEFAULT_PORT};

pub const NICK_LIMIT: usize = 32;

pub fn clean_nick(raw: &str) -> String {
    let trimmed = raw.trim();
    let name = if trimmed.is_empty() {
        "слушатель"
    } else {
        trimmed
    };
    name.chars().take(NICK_LIMIT).collect()
}

pub fn local_ip() -> Option<String> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect("9.9.9.9:53").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}

pub fn normalize_address(raw: &str) -> Result<SocketAddr, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Введите адрес комнаты".to_string());
    }

    let target = if trimmed.contains(':') {
        trimmed.to_string()
    } else {
        format!("{trimmed}:{DEFAULT_PORT}")
    };

    target
        .to_socket_addrs()
        .map_err(|_| "Не удалось разобрать адрес".to_string())?
        .next()
        .ok_or_else(|| "Не удалось разобрать адрес".to_string())
}

fn retryable(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
    )
}

pub fn spawn_writer(app: AppHandle, scope: &'static str, mut stream: TcpStream, rx: Receiver<String>) {
    thread::spawn(move || {
        for line in rx {
            let sent = (|| -> std::io::Result<()> {
                stream.write_all(line.as_bytes())?;
                stream.write_all(b"\n")?;
                stream.flush()
            })();

            if let Err(error) = sent {
                tlog(&app, scope, &format!("отправка оборвалась: {error}"));
                break;
            }
        }
    });
}

pub fn read_lines<F>(app: &AppHandle, scope: &'static str, stream: TcpStream, mut on_line: F)
where
    F: FnMut(&str),
{
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();

    loop {
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                tlog(app, scope, "соединение закрыто другой стороной");
                break;
            }
            Ok(_) => {
                let line = buffer.trim().to_string();
                buffer.clear();
                if !line.is_empty() {
                    on_line(&line);
                }
            }
            Err(ref error) if retryable(error) => thread::sleep(Duration::from_millis(50)),
            Err(error) => {
                tlog(app, scope, &format!("чтение оборвалось: {error}"));
                break;
            }
        }
    }
}

pub fn broadcast(room: &Room, message: &Value, except: Option<u64>) {
    let line = message.to_string();

    for (id, peer) in room.peers.iter() {
        if Some(*id) == except {
            continue;
        }
        let _ = peer.tx.send(line.clone());
    }
}

pub fn snapshot(app: &AppHandle) -> TogetherStatus {
    let state = app.state::<TogetherState>();
    let room = state.inner.lock().unwrap();
    room.snapshot()
}

pub fn emit_status(app: &AppHandle) {
    let _ = app.emit("together://status", snapshot(app));
}
