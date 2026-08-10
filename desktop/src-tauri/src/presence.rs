use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::State;

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(windows)]
use std::fs::File;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

macro_rules! dev_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            eprintln!("[presence] {}", format!($($arg)*));
        }
    };
}

const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;
const OP_CLOSE: u32 = 2;
const OP_PING: u32 = 3;
const OP_PONG: u32 = 4;

const RETRY_COOLDOWN: Duration = Duration::from_secs(5);

#[cfg(windows)]
struct Pipe {
    file: File,
}

#[cfg(unix)]
struct Pipe {
    stream: UnixStream,
}

impl Pipe {
    #[cfg(windows)]
    fn open(index: u8) -> Result<Self, String> {
        let path = format!(r"\\.\pipe\discord-ipc-{index}");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| format!("{path}: {e}"))?;
        Ok(Self { file })
    }

    #[cfg(unix)]
    fn open(index: u8) -> Result<Self, String> {
        let base = std::env::var("XDG_RUNTIME_DIR")
            .or_else(|_| std::env::var("TMPDIR"))
            .or_else(|_| std::env::var("TMP"))
            .or_else(|_| std::env::var("TEMP"))
            .unwrap_or_else(|_| "/tmp".to_string());

        let candidates = [
            format!("{base}/discord-ipc-{index}"),
            format!("{base}/app/com.discordapp.Discord/discord-ipc-{index}"),
            format!("{base}/snap.discord/discord-ipc-{index}"),
            format!("{base}/.flatpak/dev.vencord.Vesktop/xdg-run/discord-ipc-{index}"),
        ];

        let mut last = String::new();
        for path in candidates {
            match UnixStream::connect(&path) {
                Ok(stream) => {
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
                    return Ok(Self { stream });
                }
                Err(e) => last = format!("{path}: {e}"),
            }
        }
        Err(last)
    }

    #[cfg(windows)]
    fn raw(&mut self) -> &mut File {
        &mut self.file
    }

    #[cfg(unix)]
    fn raw(&mut self) -> &mut UnixStream {
        &mut self.stream
    }

    fn send(&mut self, opcode: u32, payload: &str) -> Result<(), String> {
        let bytes = payload.as_bytes();
        let mut frame = Vec::with_capacity(8 + bytes.len());
        frame.extend_from_slice(&opcode.to_le_bytes());
        frame.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        frame.extend_from_slice(bytes);

        let io = self.raw();
        io.write_all(&frame).map_err(|e| e.to_string())?;
        io.flush().map_err(|e| e.to_string())
    }

    fn receive(&mut self) -> Result<(u32, String), String> {
        let io = self.raw();

        let mut header = [0u8; 8];
        io.read_exact(&mut header).map_err(|e| e.to_string())?;

        let opcode = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let length = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;

        if length > 1024 * 256 {
            return Err("Discord прислал слишком большой ответ".to_string());
        }

        let mut body = vec![0u8; length];
        if length > 0 {
            io.read_exact(&mut body).map_err(|e| e.to_string())?;
        }

        Ok((opcode, String::from_utf8_lossy(&body).to_string()))
    }
}

struct Connection {
    application_id: String,
    user: Option<String>,
    pipe: Pipe,
}

#[derive(Default)]
pub struct DiscordState {
    inner: Mutex<Option<Connection>>,
    last_attempt: Mutex<Option<Instant>>,
    last_error: Mutex<Option<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresencePayload {
    pub enabled: bool,
    pub application_id: String,
    pub details: String,
    pub state: String,
    pub album: String,
    pub cover_url: Option<String>,
    pub track_url: String,
    pub button_label: String,
    pub started_at: Option<i64>,
    pub ends_at: Option<i64>,
    #[serde(default)]
    pub party_id: Option<String>,
    #[serde(default)]
    pub party_size: Option<u32>,
    #[serde(default)]
    pub party_max: Option<u32>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PresenceStatus {
    pub connected: bool,
    pub application_id: Option<String>,
    pub user: Option<String>,
    pub last_error: Option<String>,
}

fn nonce() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("mashiro-{now}")
}

fn seconds(value: i64) -> i64 {
    if value > 100_000_000_000 {
        value / 1000
    } else {
        value
    }
}

fn clamp(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= 128 {
        return trimmed.to_string();
    }
    trimmed.chars().take(128).collect()
}

fn handshake(application_id: &str) -> Result<Connection, String> {
    let mut last = String::new();

    for index in 0u8..10 {
        let mut pipe = match Pipe::open(index) {
            Ok(pipe) => pipe,
            Err(e) => {
                last = e;
                continue;
            }
        };

        dev_log!("pipe {index} opened");

        let hello = serde_json::json!({ "v": 1, "client_id": application_id }).to_string();
        if let Err(e) = pipe.send(OP_HANDSHAKE, &hello) {
            last = e;
            continue;
        }

        let (opcode, body) = match pipe.receive() {
            Ok(frame) => frame,
            Err(e) => {
                last = e;
                continue;
            }
        };

        dev_log!("handshake reply op={opcode} body={body}");

        if opcode == OP_CLOSE {
            let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
            let message = parsed
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Discord закрыл соединение");
            let code = parsed.get("code").and_then(|v| v.as_i64()).unwrap_or(0);

            if code == 4000 || message.to_lowercase().contains("client id") {
                return Err(format!(
                    "Discord отверг Application ID {application_id}: {message}. Создай приложение на discord.com/developers/applications и вставь его ID."
                ));
            }
            return Err(format!("Discord закрыл соединение: {message} (код {code})"));
        }

        if opcode != OP_FRAME {
            last = format!("неожиданный опкод {opcode}");
            continue;
        }

        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
        let event = parsed.get("evt").and_then(|v| v.as_str()).unwrap_or("");

        if event != "READY" {
            last = format!("Discord ответил событием {event}");
            continue;
        }

        let user = parsed
            .get("data")
            .and_then(|d| d.get("user"))
            .and_then(|u| u.get("username"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        dev_log!("READY as {user:?}");

        return Ok(Connection {
            application_id: application_id.to_string(),
            user,
            pipe,
        });
    }

    if last.is_empty() {
        Err("Discord не найден. Запусти Discord и попробуй снова.".to_string())
    } else {
        Err(format!(
            "Discord не найден (проверено 10 каналов). Последняя ошибка: {last}"
        ))
    }
}

fn activity_json(payload: &PresencePayload, level: u8) -> serde_json::Value {
    let mut activity = serde_json::json!({
        "type": 2,
        "details": clamp(&payload.details),
        "state": clamp(&payload.state),
    });

    if level == 3 {
        return serde_json::json!({
            "details": clamp(&payload.details),
            "state": clamp(&payload.state),
        });
    }

    let map = activity.as_object_mut().expect("activity object");

    if let (Some(start), true) = (payload.started_at, level <= 2) {
        let mut timestamps = serde_json::Map::new();
        timestamps.insert("start".to_string(), seconds(start).into());
        if let Some(end) = payload.ends_at {
            timestamps.insert("end".to_string(), seconds(end).into());
        }
        map.insert("timestamps".to_string(), serde_json::Value::Object(timestamps));
    }

    if level <= 1 {
        if let Some(cover) = payload
            .cover_url
            .as_deref()
            .filter(|url| url.starts_with("https://"))
        {
            let mut assets = serde_json::Map::new();
            assets.insert("large_image".to_string(), cover.into());
            if !payload.album.trim().is_empty() {
                assets.insert("large_text".to_string(), clamp(&payload.album).into());
            }
            map.insert("assets".to_string(), serde_json::Value::Object(assets));
        }
    }

    if level == 0 {
        if let Some(size) = payload.party_size.filter(|n| *n > 0) {
            let max = payload.party_max.unwrap_or(size).max(size);
            let id = payload
                .party_id
                .clone()
                .unwrap_or_else(|| "mashiro-together".to_string());
            map.insert(
                "party".to_string(),
                serde_json::json!({ "id": id, "size": [size, max] }),
            );
        }

        let label = payload.button_label.trim();
        if !label.is_empty() && payload.track_url.starts_with("https://") {
            let short: String = label.chars().take(31).collect();
            map.insert(
                "buttons".to_string(),
                serde_json::json!([{ "label": short, "url": payload.track_url }]),
            );
        }
    }

    activity
}

fn send_activity(
    connection: &mut Connection,
    activity: Option<serde_json::Value>,
) -> Result<(), String> {
    let args = match activity {
        Some(value) => serde_json::json!({ "pid": std::process::id(), "activity": value }),
        None => serde_json::json!({ "pid": std::process::id(), "activity": serde_json::Value::Null }),
    };

    let frame = serde_json::json!({
        "cmd": "SET_ACTIVITY",
        "nonce": nonce(),
        "args": args,
    })
    .to_string();

    dev_log!("-> {frame}");
    connection.pipe.send(OP_FRAME, &frame)?;

    loop {
        let (opcode, body) = connection.pipe.receive()?;
        dev_log!("<- op={opcode} {body}");

        match opcode {
            OP_PING => {
                connection.pipe.send(OP_PONG, &body)?;
                continue;
            }
            OP_PONG => continue,
            OP_CLOSE => {
                let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                let message = parsed
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("соединение закрыто");
                return Err(format!("Discord закрыл соединение: {message}"));
            }
            _ => {}
        }

        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();

        if parsed.get("evt").and_then(|v| v.as_str()) == Some("ERROR") {
            let message = parsed
                .get("data")
                .and_then(|d| d.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("Discord отклонил активность");
            return Err(message.to_string());
        }

        if parsed.get("cmd").and_then(|v| v.as_str()) == Some("SET_ACTIVITY") {
            return Ok(());
        }

        return Ok(());
    }
}

fn push(connection: &mut Connection, payload: &PresencePayload) -> Result<(), String> {
    let mut last = String::new();

    for level in 0u8..4 {
        match send_activity(connection, Some(activity_json(payload, level))) {
            Ok(()) => {
                dev_log!("activity accepted at level {level}");
                return Ok(());
            }
            Err(e) => {
                dev_log!("activity rejected at level {level}: {e}");
                last = e;
            }
        }
    }

    Err(last)
}

fn remember_error(state: &State<'_, DiscordState>, message: Option<String>) {
    if let Ok(mut slot) = state.last_error.lock() {
        *slot = message;
    }
}

fn status_of(slot: &Option<Connection>, last_error: Option<String>) -> PresenceStatus {
    PresenceStatus {
        connected: slot.is_some(),
        application_id: slot.as_ref().map(|c| c.application_id.clone()),
        user: slot.as_ref().and_then(|c| c.user.clone()),
        last_error,
    }
}

#[tauri::command]
pub fn update_discord_presence(
    payload: PresencePayload,
    state: State<'_, DiscordState>,
) -> Result<(), String> {
    let mut slot = state.inner.lock().map_err(|e| e.to_string())?;

    if !payload.enabled || payload.application_id.trim().is_empty() {
        if let Some(mut connection) = slot.take() {
            let _ = send_activity(&mut connection, None);
        }
        drop(slot);
        remember_error(&state, None);
        return Ok(());
    }

    if slot
        .as_ref()
        .map(|c| c.application_id != payload.application_id)
        .unwrap_or(false)
    {
        dev_log!("application id changed, dropping connection");
        slot.take();
    }

    if slot.is_none() {
        let mut last = state.last_attempt.lock().map_err(|e| e.to_string())?;
        if let Some(at) = *last {
            if at.elapsed() < RETRY_COOLDOWN {
                return Ok(());
            }
        }
        *last = Some(Instant::now());
        drop(last);

        match handshake(payload.application_id.trim()) {
            Ok(connection) => *slot = Some(connection),
            Err(message) => {
                drop(slot);
                remember_error(&state, Some(message.clone()));
                return Err(message);
            }
        }
    }

    let connection = slot
        .as_mut()
        .ok_or_else(|| "Нет соединения с Discord".to_string())?;

    match push(connection, &payload) {
        Ok(()) => {
            drop(slot);
            remember_error(&state, None);
            Ok(())
        }
        Err(first) => {
            let _ = &first;
            dev_log!("retrying with a fresh connection after: {first}");
            slot.take();

            let mut fresh = match handshake(payload.application_id.trim()) {
                Ok(connection) => connection,
                Err(message) => {
                    drop(slot);
                    remember_error(&state, Some(message.clone()));
                    return Err(message);
                }
            };

            match push(&mut fresh, &payload) {
                Ok(()) => {
                    *slot = Some(fresh);
                    drop(slot);
                    remember_error(&state, None);
                    Ok(())
                }
                Err(message) => {
                    drop(slot);
                    remember_error(&state, Some(message.clone()));
                    Err(message)
                }
            }
        }
    }
}

#[tauri::command]
pub fn clear_discord_presence(state: State<'_, DiscordState>) -> Result<(), String> {
    let mut slot = state.inner.lock().map_err(|e| e.to_string())?;
    if let Some(connection) = slot.as_mut() {
        let _ = send_activity(connection, None);
    }
    Ok(())
}

#[tauri::command]
pub fn discord_presence_status(state: State<'_, DiscordState>) -> Result<PresenceStatus, String> {
    let slot = state.inner.lock().map_err(|e| e.to_string())?;
    let last_error = state.last_error.lock().ok().and_then(|e| e.clone());
    Ok(status_of(&slot, last_error))
}

#[tauri::command]
pub fn reconnect_discord_presence(
    application_id: String,
    state: State<'_, DiscordState>,
) -> Result<PresenceStatus, String> {
    let id = application_id.trim().to_string();
    if id.is_empty() {
        return Err("Не задан Application ID".to_string());
    }

    let mut slot = state.inner.lock().map_err(|e| e.to_string())?;
    slot.take();

    if let Ok(mut last) = state.last_attempt.lock() {
        *last = None;
    }

    match handshake(&id) {
        Ok(connection) => {
            *slot = Some(connection);
            let status = status_of(&slot, None);
            drop(slot);
            remember_error(&state, None);
            Ok(status)
        }
        Err(message) => {
            drop(slot);
            remember_error(&state, Some(message.clone()));
            Err(message)
        }
    }
}

#[tauri::command]
pub async fn validate_discord_app(application_id: String) -> Result<String, String> {
    let id = application_id.trim().to_string();
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
        return Err("Application ID должен состоять только из цифр".to_string());
    }

    let url = format!(
        "{}://discord.com/api/v10/oauth2/applications/{id}/rpc",
        "https"
    );

    let res = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Не удалось проверить ID: {e}"))?;

    if res.status().as_u16() == 404 {
        return Err(
            "Приложения с таким Application ID не существует. Создай своё на discord.com/developers/applications и вставь его ID."
                .to_string(),
        );
    }

    if !res.status().is_success() {
        return Err(format!("Discord ответил {}", res.status().as_u16()));
    }

    let body: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(body
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Discord app")
        .to_string())
}
