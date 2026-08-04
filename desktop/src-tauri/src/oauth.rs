use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const CLIENT_ID: &str = "23cabbbdc6cd418abb4b39c32c41195d";
const CLIENT_SECRET: &str = "53bc75238f0c4d08a118e51fe9203300";
const DEVICE_CODE_URL: &str = "https://oauth.yandex.ru/device/code";
const TOKEN_URL: &str = "https://oauth.yandex.ru/token";
const DEVICE_ID: &str = "mashiro-desktop-client";
const DEVICE_NAME: &str = "Mashiro Desktop";

static GENERATION: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
pub struct DeviceInfo {
    pub user_code: String,
    pub verification_url: String,
}

#[derive(Deserialize)]
struct DeviceCodeResp {
    device_code: Option<String>,
    user_code: Option<String>,
    verification_url: Option<String>,
    interval: Option<u64>,
    expires_in: Option<u64>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Deserialize)]
struct TokenResp {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

fn open_in_browser(url: &str) {
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

#[tauri::command]
pub async fn oauth_device_start(app: AppHandle) -> Result<DeviceInfo, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(DEVICE_CODE_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            ("device_id", DEVICE_ID),
            ("device_name", DEVICE_NAME),
        ])
        .send()
        .await
        .map_err(|e| format!("Сеть недоступна: {e}"))?;
    let data: DeviceCodeResp = resp
        .json()
        .await
        .map_err(|e| format!("Не удалось разобрать ответ Яндекса: {e}"))?;
    if let Some(err) = data.error {
        let desc = data.error_description.unwrap_or(err);
        return Err(format!("Яндекс отклонил запрос: {desc}"));
    }
    let device_code = data.device_code.ok_or("Яндекс не вернул device_code")?;
    let user_code = data
        .user_code
        .ok_or("Яндекс не вернул код подтверждения")?;
    let verification_url = data
        .verification_url
        .unwrap_or_else(|| "https://oauth.yandex.ru/device".to_string());
    let poll_interval = data.interval.unwrap_or(5).max(1);
    let expires_in = data.expires_in.unwrap_or(300);

    open_in_browser(&verification_url);

    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let info = DeviceInfo {
        user_code: user_code.clone(),
        verification_url: verification_url.clone(),
    };

    tauri::async_runtime::spawn(async move {
        let client = reqwest::Client::new();
        let mut interval = poll_interval;
        let mut waited: u64 = 0;
        loop {
            tokio::time::sleep(Duration::from_secs(interval)).await;
            waited += interval;
            if GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }
            if waited >= expires_in {
                let _ = app.emit(
                    "oauth-error",
                    "Время ожидания истекло. Попробуй снова.".to_string(),
                );
                return;
            }
            let resp = client
                .post(TOKEN_URL)
                .form(&[
                    ("grant_type", "device_code"),
                    ("code", device_code.as_str()),
                    ("client_id", CLIENT_ID),
                    ("client_secret", CLIENT_SECRET),
                ])
                .send()
                .await;
            let resp = match resp {
                Ok(r) => r,
                Err(_) => continue, // сетевой сбой - продолжаем опрос
            };
            let data: TokenResp = match resp.json().await {
                Ok(d) => d,
                Err(_) => continue,
            };
            if let Some(token) = data.access_token {
                let _ = app.emit("oauth-token", token);
                return;
            }
            match data.error.as_deref() {
                Some("authorization_pending") => continue,
                Some("slow_down") => {
                    interval += 5;
                    continue;
                }
                Some(other) => {
                    let desc = data
                        .error_description
                        .unwrap_or_else(|| other.to_string());
                    let _ = app.emit("oauth-error", desc);
                    return;
                }
                None => continue,
            }
        }
    });

    Ok(info)
}

#[tauri::command]
pub fn oauth_cancel() {
    GENERATION.fetch_add(1, Ordering::SeqCst);
}
