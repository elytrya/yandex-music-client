
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

const API: &str = "https://lrclib.net/api";
const UA: &str = "Mashiro/1.0 (Yandex Music desktop client)";
const SEARCH_LIMIT: usize = 12;

#[derive(Serialize, Clone, Debug)]
pub struct LrcLine {
    pub time_ms: i64,
    pub text: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct LrcLyrics {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration: Option<f64>,
    pub instrumental: bool,
    pub synced: bool,
    pub lines: Vec<LrcLine>,
    pub source: String,
}

fn http() -> Result<&'static reqwest::Client, String> {
    static CLIENT: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(12))
                .user_agent(UA)
                .build()
                .ok()
        })
        .as_ref()
        .ok_or_else(|| "не удалось создать http-клиент".to_string())
}

fn cache() -> &'static Mutex<HashMap<String, Option<LrcLyrics>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<LrcLyrics>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len() * 3);
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

fn text(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

async fn get_json(url: &str) -> Result<Option<Value>, String> {
    let resp = http()?
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("LRCLIB недоступен: {e}"))?;

    let status = resp.status();
    if status.as_u16() == 404 {
        return Ok(None);
    }
    if !status.is_success() {
        return Err(format!("LRCLIB ответил {}", status.as_u16()));
    }

    let body = resp
        .text()
        .await
        .map_err(|e| format!("LRCLIB: не удалось прочитать ответ ({e})"))?;

    serde_json::from_str::<Value>(&body)
        .map(Some)
        .map_err(|e| format!("LRCLIB: непонятный ответ ({e})"))
}

fn parse_stamp(stamp: &str) -> Option<i64> {
    let (mm, rest) = stamp.split_once(':')?;
    let minutes: i64 = mm.trim().parse().ok()?;
    let seconds: f64 = rest.replace(',', ".").trim().parse().ok()?;
    if !(0.0..3600.0).contains(&seconds) {
        return None;
    }
    Some(minutes * 60_000 + (seconds * 1000.0).round() as i64)
}

fn parse_lrc(raw: &str) -> Vec<LrcLine> {
    let mut out: Vec<LrcLine> = Vec::new();

    for line in raw.lines() {
        let mut rest = line.trim();
        let mut stamps: Vec<i64> = Vec::new();

        while rest.starts_with('[') {
            let Some(end) = rest.find(']') else { break };
            let stamp = &rest[1..end];
            let tail = rest[end + 1..].trim_start();
            match parse_stamp(stamp) {
                Some(ms) => {
                    stamps.push(ms);
                    rest = tail;
                }
                None => {
                    if stamps.is_empty() {
                        rest = "";
                    }
                    break;
                }
            }
        }

        if stamps.is_empty() {
            continue;
        }

        let body = rest.trim().to_string();
        for ms in stamps {
            out.push(LrcLine {
                time_ms: ms,
                text: body.clone(),
            });
        }
    }

    out.sort_by_key(|line| line.time_ms);
    out
}

fn plain_lines(raw: &str) -> Vec<LrcLine> {
    raw.trim()
        .lines()
        .map(|line| LrcLine {
            time_ms: 0,
            text: line.trim().to_string(),
        })
        .collect()
}

fn build(item: &Value) -> Option<LrcLyrics> {
    let title = text(item.get("trackName")).unwrap_or_default();
    let artist = text(item.get("artistName")).unwrap_or_default();
    let instrumental = item
        .get("instrumental")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let mut synced = false;
    let mut lines: Vec<LrcLine> = Vec::new();

    if let Some(raw) = text(item.get("syncedLyrics")) {
        let parsed = parse_lrc(&raw);
        if parsed.iter().any(|line| !line.text.is_empty()) {
            synced = true;
            lines = parsed;
        }
    }

    if lines.is_empty() {
        if let Some(raw) = text(item.get("plainLyrics")) {
            lines = plain_lines(&raw);
        }
    }

    if lines.is_empty() {
        if !instrumental {
            return None;
        }
        lines.push(LrcLine {
            time_ms: 0,
            text: "Инструментальная композиция".to_string(),
        });
    }

    Some(LrcLyrics {
        id: item.get("id").and_then(Value::as_i64).unwrap_or(0),
        title,
        artist,
        album: text(item.get("albumName")),
        duration: item.get("duration").and_then(Value::as_f64),
        instrumental,
        synced,
        lines,
        source: "lrclib".to_string(),
    })
}

fn norm(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut skip = 0usize;
    for ch in value.chars() {
        match ch {
            '(' | '[' => skip += 1,
            ')' | ']' => skip = skip.saturating_sub(1),
            _ if skip > 0 => {}
            _ if ch.is_alphanumeric() => out.extend(ch.to_lowercase()),
            _ => {}
        }
    }
    if out.is_empty() {
        value.trim().to_lowercase()
    } else {
        out
    }
}

fn score(item: &LrcLyrics, title: &str, artist: &str, duration: Option<f64>) -> i32 {
    let want_title = norm(title);
    let want_artist = norm(artist);
    let got_title = norm(&item.title);
    let got_artist = norm(&item.artist);

    let mut points = 0;

    if got_title == want_title {
        points += 4;
    } else if got_title.contains(&want_title) || want_title.contains(&got_title) {
        points += 2;
    } else {
        points -= 3;
    }

    if !want_artist.is_empty() {
        if got_artist == want_artist {
            points += 3;
        } else if got_artist.contains(&want_artist) || want_artist.contains(&got_artist) {
            points += 2;
        } else {
            points -= 2;
        }
    }

    if let (Some(want), Some(got)) = (duration, item.duration) {
        let diff = (want - got).abs();
        if diff <= 3.0 {
            points += 3;
        } else if diff <= 8.0 {
            points += 1;
        } else if diff > 25.0 {
            points -= 3;
        }
    }

    if item.synced {
        points += 1;
    }

    points
}

async fn exact(title: &str, artist: &str, album: Option<&str>, duration: Option<f64>) -> Option<LrcLyrics> {
    let mut url = format!(
        "{API}/get?track_name={}&artist_name={}",
        encode(title),
        encode(artist)
    );
    if let Some(album) = album.filter(|a| !a.trim().is_empty()) {
        url.push_str(&format!("&album_name={}", encode(album)));
    }
    if let Some(duration) = duration {
        url.push_str(&format!("&duration={}", duration.round() as i64));
    }

    match get_json(&url).await {
        Ok(Some(payload)) => build(&payload),
        _ => None,
    }
}

async fn search(title: &str, artist: &str) -> Vec<LrcLyrics> {
    let mut url = format!("{API}/search?track_name={}", encode(title));
    if !artist.trim().is_empty() {
        url.push_str(&format!("&artist_name={}", encode(artist)));
    }

    let payload = match get_json(&url).await {
        Ok(Some(payload)) => payload,
        _ => return Vec::new(),
    };

    payload
        .as_array()
        .map(|items| {
            items
                .iter()
                .take(SEARCH_LIMIT)
                .filter_map(build)
                .collect::<Vec<LrcLyrics>>()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub async fn lrclib_lookup(
    title: String,
    artist: String,
    album: Option<String>,
    duration: Option<f64>,
    force: Option<bool>,
) -> Result<Option<LrcLyrics>, String> {
    let title = title.trim().to_string();
    let artist = artist.trim().to_string();
    if title.is_empty() {
        return Ok(None);
    }

    let key = format!(
        "{}|{}|{}",
        norm(&title),
        norm(&artist),
        duration.map(|d| d.round() as i64).unwrap_or(0)
    );

    if !force.unwrap_or(false) {
        if let Some(hit) = cache().lock().ok().and_then(|c| c.get(&key).cloned()) {
            return Ok(hit);
        }
    }

    let mut found = exact(&title, &artist, album.as_deref(), duration).await;

    if found.is_none() {
        found = exact(&title, &artist, None, None).await;
    }

    if found.is_none() {
        let mut best: Option<(i32, LrcLyrics)> = None;
        for item in search(&title, &artist).await {
            let points = score(&item, &title, &artist, duration);
            if best.as_ref().map(|(top, _)| points > *top).unwrap_or(true) {
                best = Some((points, item));
            }
        }
        found = best.filter(|(points, _)| *points >= 3).map(|(_, item)| item);
    }

    if let Ok(mut cache) = cache().lock() {
        cache.insert(key, found.clone());
    }

    Ok(found)
}

#[tauri::command]
pub fn lrclib_clear_cache() {
    if let Ok(mut cache) = cache().lock() {
        cache.clear();
    }
}
