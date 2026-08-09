use serde_json::Value;
use tauri::{AppHandle, State};
use tokio::sync::mpsc::unbounded_channel;

use super::client::{self, Intent, Options};
use super::{crypto, keys, Outgoing, RelayState, RelayStatus, DEFAULT_NICK, NICK_LIMIT};

fn clean_nick(nick: &str) -> String {
    let trimmed = nick.trim();
    if trimmed.is_empty() {
        return DEFAULT_NICK.to_string();
    }

    trimmed.chars().take(NICK_LIMIT).collect()
}

fn clean_address(address: &str) -> Result<String, String> {
    let trimmed = address.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(String::from("адрес сервера не задан"));
    }

    let full = if trimmed.starts_with("ws://") || trimmed.starts_with("wss://") {
        trimmed.to_string()
    } else if let Some(rest) = trimmed.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = trimmed.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        format!("wss://{trimmed}")
    };

    Ok(if full.ends_with("/ws") {
        full
    } else {
        format!("{full}/ws")
    })
}

fn start(
    app: &AppHandle,
    relay: &RelayState,
    address: &str,
    nick: &str,
    invite: String,
    intent: Intent,
) -> Result<RelayStatus, String> {
    let phrase = keys::load()?
        .ok_or_else(|| String::from("сначала создайте или введите сид-фразу"))?;

    let address = clean_address(address)?;
    let nick = clean_nick(nick);

    crypto::secrets(&invite)?;

    let (sender, orders) = unbounded_channel();
    let waiting = RelayStatus {
        address: address.clone(),
        invite: invite.clone(),
        nick: nick.clone(),
        ..RelayStatus::default()
    };

    let epoch = relay.begin(sender, waiting.clone());

    let options = Options {
        address,
        invite,
        nick,
        phrase,
        intent,
    };

    let task_app = app.clone();
    let task_relay = relay.clone();
    tauri::async_runtime::spawn(async move {
        client::run(task_app, task_relay, options, orders, epoch).await;
    });

    Ok(waiting)
}

#[tauri::command]
pub fn together_relay_create(
    app: AppHandle,
    state: State<'_, RelayState>,
    address: String,
    nick: String,
    invite: Option<String>,
) -> Result<RelayStatus, String> {
    let invite = match invite {
        Some(code) if !code.trim().is_empty() => code.trim().to_string(),
        _ => crypto::fresh_invite(),
    };

    start(&app, state.inner(), &address, &nick, invite, Intent::Create)
}

#[tauri::command]
pub fn together_relay_join(
    app: AppHandle,
    state: State<'_, RelayState>,
    address: String,
    nick: String,
    invite: String,
) -> Result<RelayStatus, String> {
    let invite = invite.trim().to_string();
    if invite.is_empty() {
        return Err(String::from("код приглашения не задан"));
    }

    start(&app, state.inner(), &address, &nick, invite, Intent::Join)
}

#[tauri::command]
pub fn together_relay_leave(state: State<'_, RelayState>) {
    state.stop();
}

#[tauri::command]
pub fn together_relay_send(state: State<'_, RelayState>, payload: Value) -> Result<(), String> {
    state.send(Outgoing::Payload(Box::new(payload)))
}

#[tauri::command]
pub fn together_relay_handoff(state: State<'_, RelayState>, to: u64) -> Result<(), String> {
    state.send(Outgoing::Handoff(to))
}

#[tauri::command]
pub fn together_relay_status(state: State<'_, RelayState>) -> RelayStatus {
    state.status()
}

#[tauri::command]
pub fn together_relay_seed_exists() -> Result<bool, String> {
    Ok(keys::load()?.is_some())
}

#[tauri::command]
pub fn together_relay_seed_show() -> Result<Option<String>, String> {
    keys::load()
}

#[tauri::command]
pub fn together_relay_seed_set(phrase: String) -> Result<String, String> {
    keys::save(&phrase)
}

#[tauri::command]
pub fn together_relay_seed_new(words: Option<usize>) -> Result<String, String> {
    keys::create(words.unwrap_or(12))
}

#[tauri::command]
pub fn together_relay_seed_forget(state: State<'_, RelayState>) -> Result<(), String> {
    state.stop();
    keys::forget()
}

#[tauri::command]
pub fn together_relay_identity() -> Result<Option<String>, String> {
    match keys::load()? {
        Some(phrase) => keys::public(&phrase).map(Some),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn together_relay_invite(short: Option<bool>) -> String {
    if short.unwrap_or(false) {
        crypto::short_invite()
    } else {
        crypto::fresh_invite()
    }
}
