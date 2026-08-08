use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

const LIST_URL: &str =
    "https://raw.githubusercontent.com/Hazzz895/FckCensorData/main/list.json";

#[derive(Debug, Clone, Serialize)]
pub struct CensorTrack {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
}

fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mashiro")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn list_cache() -> &'static Mutex<Option<HashMap<String, String>>> {
    static CACHE: OnceLock<Mutex<Option<HashMap<String, String>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

fn title_cache() -> &'static Mutex<HashMap<String, CensorTrack>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CensorTrack>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

async fn ensure_list() -> Result<HashMap<String, String>, String> {
    {
        let guard = list_cache().lock().map_err(|_| "lock".to_string())?;
        if let Some(map) = guard.as_ref() {
            return Ok(map.clone());
        }
    }

    let client = build_client();
    let resp = client
        .get(LIST_URL)
        .send()
        .await
        .map_err(|e| format!("Не удалось загрузить список FckCensorData: {e}"))?;
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Ошибка чтения списка FckCensorData: {e}"))?;
    let parsed: HashMap<String, String> =
        serde_json::from_str(&text).map_err(|e| format!("Некорректный список FckCensorData: {e}"))?;

    {
        let mut guard = list_cache().lock().map_err(|_| "lock".to_string())?;
        *guard = Some(parsed.clone());
    }
    Ok(parsed)
}

#[tauri::command]
pub async fn censor_ids() -> Result<Vec<String>, String> {
    let map = ensure_list().await?;
    Ok(map.into_keys().collect())
}

fn find_title_object(value: &Value) -> Option<&Value> {
    match value {
        Value::Object(map) => {
            if map.get("title").and_then(Value::as_str).is_some() {
                return Some(value);
            }
            for v in map.values() {
                if let Some(found) = find_title_object(v) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(items) => {
            for v in items {
                if let Some(found) = find_title_object(v) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

fn extract_track(id: &str, value: &Value) -> Option<CensorTrack> {
    let obj = find_title_object(value)?;
    let title = obj.get("title").and_then(Value::as_str)?.trim().to_string();
    if title.is_empty() {
        return None;
    }

    let mut artists = Vec::new();
    if let Some(arr) = obj.get("artists").and_then(Value::as_array) {
        for a in arr {
            if let Some(name) = a.get("name").and_then(Value::as_str) {
                let name = name.trim();
                if !name.is_empty() {
                    artists.push(name.to_string());
                }
            }
        }
    }

    Some(CensorTrack {
        id: id.to_string(),
        title,
        artists,
    })
}

async fn fetch_track(client: &reqwest::Client, id: &str, url: &str) -> Option<CensorTrack> {
    {
        let guard = title_cache().lock().ok()?;
        if let Some(found) = guard.get(id) {
            return Some(found.clone());
        }
    }

    let resp = client.get(url).send().await.ok()?;
    let bytes = resp.bytes().await.ok()?;
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    let track = extract_track(id, &value)?;

    if let Ok(mut guard) = title_cache().lock() {
        guard.insert(id.to_string(), track.clone());
    }
    Some(track)
}

#[tauri::command]
pub async fn censor_titles(ids: Vec<String>) -> Result<Vec<CensorTrack>, String> {
    let map = ensure_list().await?;
    let client = build_client();
    let mut out = Vec::new();
    for id in ids {
        let Some(url) = map.get(&id) else { continue };
        if let Some(track) = fetch_track(&client, &id, url).await {
            out.push(track);
        }
    }
    Ok(out)
}
