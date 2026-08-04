use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde::Serialize;

const SLOPLESS_API: &str = "https://slopless.art";
const AI_ARTIST_THRESHOLD: f64 = 0.05;
const AI_TRACK_THRESHOLD: f64 = 0.5;

#[derive(Serialize, Clone)]
pub struct AiArtistResult {
    pub id: String,
    pub ai: bool,
    pub ai_tracks: u64,
    pub total_tracks: u64,
}

#[derive(Serialize, Clone)]
pub struct AiTrackResult {
    pub id: String,
    pub ai: bool,
    pub score: Option<f64>,
}

fn artist_cache() -> &'static Mutex<HashMap<String, AiArtistResult>> {
    static CACHE: OnceLock<Mutex<HashMap<String, AiArtistResult>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn track_cache() -> &'static Mutex<HashMap<String, AiTrackResult>> {
    static CACHE: OnceLock<Mutex<HashMap<String, AiTrackResult>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())
}

async fn fetch_artist(client: &reqwest::Client, id: &str) -> AiArtistResult {
    let url = format!("{SLOPLESS_API}/api/artist/{id}");
    let stats = async {
        let resp = client.get(&url).send().await.ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let v: serde_json::Value = resp.json().await.ok()?;
        let ai_tracks = v.get("aiTracks").and_then(|x| x.as_u64()).unwrap_or(0);
        let total = v.get("totalTracks").and_then(|x| x.as_u64()).unwrap_or(0);
        Some((ai_tracks, total))
    }
    .await;

    match stats {
        Some((ai_tracks, total_tracks)) => AiArtistResult {
            id: id.to_string(),
            ai: total_tracks > 0
                && (ai_tracks as f64 / total_tracks as f64) >= AI_ARTIST_THRESHOLD,
            ai_tracks,
            total_tracks,
        },
        None => AiArtistResult {
            id: id.to_string(),
            ai: false,
            ai_tracks: 0,
            total_tracks: 0,
        },
    }
}

async fn fetch_track(client: &reqwest::Client, id: &str) -> AiTrackResult {
    let url = format!("{SLOPLESS_API}/api/track/{id}");
    let score = async {
        let resp = client.get(&url).send().await.ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let v: serde_json::Value = resp.json().await.ok()?;
        Some(v.get("score").and_then(|x| x.as_f64()))
    }
    .await
    .flatten();

    AiTrackResult {
        id: id.to_string(),
        ai: score.map(|s| s > AI_TRACK_THRESHOLD).unwrap_or(false),
        score,
    }
}

#[tauri::command]
pub async fn ai_check_artists(ids: Vec<String>) -> Result<Vec<AiArtistResult>, String> {
    let client = build_client()?;
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        if id.is_empty() {
            continue;
        }
        let cached = artist_cache()
            .lock()
            .ok()
            .and_then(|c| c.get(&id).cloned());
        if let Some(hit) = cached {
            out.push(hit);
            continue;
        }
        let res = fetch_artist(&client, &id).await;
        if let Ok(mut c) = artist_cache().lock() {
            c.insert(id.clone(), res.clone());
        }
        out.push(res);
    }
    Ok(out)
}

#[tauri::command]
pub async fn ai_check_tracks(ids: Vec<String>) -> Result<Vec<AiTrackResult>, String> {
    let client = build_client()?;
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        if id.is_empty() {
            continue;
        }
        let cached = track_cache().lock().ok().and_then(|c| c.get(&id).cloned());
        if let Some(hit) = cached {
            out.push(hit);
            continue;
        }
        let res = fetch_track(&client, &id).await;
        if let Ok(mut c) = track_cache().lock() {
            c.insert(id.clone(), res.clone());
        }
        out.push(res);
    }
    Ok(out)
}
