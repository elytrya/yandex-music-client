
use std::sync::OnceLock;
use std::time::Duration;

use serde_json::Value;

pub const API: &str = "https://api.genius.com";
pub const SITE: &str = "https://genius.com/api";
pub const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

pub fn http() -> Result<&'static reqwest::Client, String> {
    static CLIENT: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .user_agent(UA)
                .build()
                .ok()
        })
        .as_ref()
        .ok_or_else(|| "не удалось создать http-клиент".to_string())
}

pub fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len() * 3);
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push('+'),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

fn unwrap_response(body: Value) -> Value {
    body.get("response").cloned().unwrap_or(body)
}

fn status_error(status: u16) -> Option<String> {
    match status {
        401 | 403 => Some("Genius не принял токен".to_string()),
        404 => Some("Genius: ничего не найдено".to_string()),
        429 => Some("Genius просит подождать: слишком много запросов".to_string()),
        code if !(200..300).contains(&code) => Some(format!("Genius ответил {code}")),
        _ => None,
    }
}

pub async fn get(token: &str, path: &str) -> Result<Value, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("Не задан токен Genius".to_string());
    }

    let resp = http()?
        .get(format!("{API}{path}"))
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Genius недоступен: {e}"))?;

    let status = resp.status().as_u16();
    if let Some(error) = status_error(status) {
        return Err(error);
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| format!("Genius прислал не JSON: {e}"))?;

    Ok(unwrap_response(body))
}

pub async fn get_site(path: &str) -> Result<Value, String> {
    let resp = http()?
        .get(format!("{SITE}{path}"))
        .header("Accept", "application/json")
        .header("X-Requested-With", "XMLHttpRequest")
        .send()
        .await
        .map_err(|e| format!("Genius недоступен: {e}"))?;

    let status = resp.status().as_u16();
    if let Some(error) = status_error(status) {
        return Err(error);
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| format!("Genius прислал не JSON: {e}"))?;

    Ok(unwrap_response(body))
}

pub async fn page(url: &str) -> Result<String, String> {
    if url.is_empty() {
        return Err("у страницы нет адреса".to_string());
    }

    let resp = http()?
        .get(url)
        .header("Accept", "text/html,application/xhtml+xml")
        .header("Accept-Language", "en-US,en;q=0.9,ru;q=0.8")
        .send()
        .await
        .map_err(|e| format!("страница Genius не открылась: {e}"))?;

    let status = resp.status().as_u16();
    if let Some(error) = status_error(status) {
        return Err(error);
    }

    resp.text()
        .await
        .map_err(|e| format!("страница Genius пришла битой: {e}"))
}
