use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use tauri::http::{Request, Response};
use tauri::UriSchemeResponder;

pub const SCHEME: &str = "ymstream";

const CACHE_LIMIT: usize = 3;

struct Cached {
    url: String,
    body: Vec<u8>,
    mime: String,
}

fn cache() -> &'static Mutex<Vec<Cached>> {
    static CACHE: OnceLock<Mutex<Vec<Cached>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

fn inflight() -> &'static Mutex<HashSet<String>> {
    static INFLIGHT: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    INFLIGHT.get_or_init(|| Mutex::new(HashSet::new()))
}

fn cached(url: &str) -> Option<(Vec<u8>, String)> {
    let store = cache().lock().ok()?;
    store
        .iter()
        .find(|item| item.url == url)
        .map(|item| (item.body.clone(), item.mime.clone()))
}

fn remember(url: &str, body: &[u8], mime: &str) {
    if body.is_empty() {
        return;
    }
    if let Ok(mut store) = cache().lock() {
        store.retain(|item| item.url != url);
        store.push(Cached {
            url: url.to_string(),
            body: body.to_vec(),
            mime: mime.to_string(),
        });
        while store.len() > CACHE_LIMIT {
            store.remove(0);
        }
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push((hi * 16 + lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn mime_for(url: &str) -> &'static str {
    let clean = url.split('?').next().unwrap_or(url).to_lowercase();
    if clean.contains("flac-mp4") || clean.contains("alac-mp4") {
        return "audio/mp4";
    }
    if clean.ends_with(".mp3") || clean.ends_with("/mp3") {
        "audio/mpeg"
    } else if clean.ends_with(".flac") || clean.ends_with("/flac") {
        "audio/flac"
    } else if clean.ends_with(".m4a") || clean.ends_with(".aac") || clean.ends_with("/aac") {
        "audio/mp4"
    } else if clean.ends_with(".ogg") || clean.ends_with(".opus") {
        "audio/ogg"
    } else {
        "audio/mpeg"
    }
}

fn empty(status: u16) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .body(Vec::new())
        .expect("empty response")
}

fn parse_range(value: &str, total: usize) -> Option<(usize, usize)> {
    let raw = value.trim().strip_prefix("bytes=")?;
    let (from, to) = raw.split_once('-')?;
    if from.is_empty() {
        let suffix: usize = to.parse().ok()?;
        let start = total.saturating_sub(suffix);
        return Some((start, total.saturating_sub(1)));
    }
    let start: usize = from.parse().ok()?;
    if start >= total {
        return None;
    }
    let end = if to.is_empty() {
        total.saturating_sub(1)
    } else {
        to.parse().unwrap_or(total.saturating_sub(1))
    };
    Some((start, end.min(total.saturating_sub(1))))
}

fn from_cache(body: Vec<u8>, mime: String, range: Option<String>) -> Response<Vec<u8>> {
    let total = body.len();
    let mut builder = Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .header("Accept-Ranges", "bytes")
        .header("Content-Type", mime);
    if let Some((start, end)) = range.as_deref().and_then(|value| parse_range(value, total)) {
        let slice = body[start..=end].to_vec();
        builder = builder
            .status(206)
            .header("Content-Length", slice.len().to_string())
            .header("Content-Range", format!("bytes {}-{}/{}", start, end, total));
        return builder.body(slice).unwrap_or_else(|_| empty(500));
    }
    builder = builder
        .status(200)
        .header("Content-Length", total.to_string());
    builder.body(body).unwrap_or_else(|_| empty(500))
}

async fn download(target: &str) -> Option<(Vec<u8>, String)> {
    let client = reqwest::Client::builder()
        .user_agent("Yandex-Music-API")
        .build()
        .ok()?;
    let response = client.get(target).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let upstream = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let body = response.bytes().await.ok()?.to_vec();
    let mime = if upstream.starts_with("audio/") {
        upstream
    } else {
        mime_for(target).to_string()
    };
    Some((body, mime))
}

const CHUNKS: usize = 4;
const MIN_CHUNKED: usize = 2 * 1024 * 1024;

fn http() -> Option<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent("Yandex-Music-API")
        .build()
        .ok()
}

async fn probe(target: &str) -> Option<(usize, String)> {
    let response = http()?
        .get(target)
        .header("Range", "bytes=0-0")
        .send()
        .await
        .ok()?;
    let mime = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let total = response
        .headers()
        .get("content-range")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.rsplit('/').next().map(|part| part.to_string()))
        .and_then(|part| part.trim().parse::<usize>().ok())?;
    Some((total, mime))
}

async fn fetch_range(target: String, start: usize, end: usize) -> Option<Vec<u8>> {
    let response = http()?
        .get(&target)
        .header("Range", format!("bytes={}-{}", start, end))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    Some(response.bytes().await.ok()?.to_vec())
}

async fn download_chunked(target: &str) -> Option<(Vec<u8>, String)> {
    let Some((total, upstream)) = probe(target).await else {
        return download(target).await;
    };
    if total < MIN_CHUNKED {
        return download(target).await;
    }
    let span = (total + CHUNKS - 1) / CHUNKS;
    let mut tasks = Vec::new();
    for index in 0..CHUNKS {
        let start = index * span;
        if start >= total {
            break;
        }
        let end = (start + span).min(total) - 1;
        let url = target.to_string();
        tasks.push(tauri::async_runtime::spawn(async move {
            fetch_range(url, start, end).await
        }));
    }
    let mut body: Vec<u8> = Vec::with_capacity(total);
    for task in tasks {
        let part = task.await.ok()??;
        body.extend_from_slice(&part);
    }
    if body.len() != total {
        return download(target).await;
    }
    let mime = if upstream.starts_with("audio/") {
        upstream
    } else {
        mime_for(target).to_string()
    };
    Some((body, mime))
}
pub fn clear() {
    if let Ok(mut store) = cache().lock() {
        store.clear();
    }
}

pub async fn warm(target: String) {
    if !target.starts_with("https://") && !target.starts_with("http://") {
        return;
    }
    if cached(&target).is_some() {
        return;
    }
    if let Ok(mut set) = inflight().lock() {
        if !set.insert(target.clone()) {
            return;
        }
    }
    let started = std::time::Instant::now();
    if let Some((body, mime)) = download_chunked(&target).await {
        let size = body.len();
        remember(&target, &body, &mime);
        println!(
            "[cache] предзагружено {} КБ за {} мс, {} КБ/с ({mime})",
            size / 1024,
            started.elapsed().as_millis(),
            size as u128 / 1024 / started.elapsed().as_millis().max(1) * 1000
        );
    }
    if let Ok(mut set) = inflight().lock() {
        set.remove(&target);
    }
}
async fn fetch(target: String, range: Option<String>) -> Response<Vec<u8>> {
    if !target.starts_with("https://") && !target.starts_with("http://") {
        return empty(400);
    }

    if let Some((body, mime)) = cached(&target) {
        println!("[cache] из памяти: {} КБ", body.len() / 1024);
        return from_cache(body, mime, range);
    }

    {
        let warming = target.clone();
        tauri::async_runtime::spawn(async move { warm(warming).await });
    }

    let client = match reqwest::Client::builder()
        .user_agent("Yandex-Music-API")
        .build()
    {
        Ok(client) => client,
        Err(_) => return empty(500),
    };

    let mut request = client.get(&target);
    if let Some(value) = range.as_deref() {
        request = request.header("Range", value);
    }

    let response = match request.send().await {
        Ok(response) => response,
        Err(_) => return empty(502),
    };

    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body = match response.bytes().await {
        Ok(bytes) => bytes.to_vec(),
        Err(_) => return empty(502),
    };

    let mut builder = Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .header("Accept-Ranges", "bytes");

    let upstream_type = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let content_type = if upstream_type.starts_with("audio/") {
        upstream_type.to_string()
    } else {
        mime_for(&target).to_string()
    };
    builder = builder.header("Content-Type", content_type);

    for name in ["content-length", "content-range"] {
        if let Some(value) = headers.get(name).and_then(|value| value.to_str().ok()) {
            builder = builder.header(name, value);
        }
    }

    builder.body(body).unwrap_or_else(|_| empty(500))
}

pub fn handle(request: Request<Vec<u8>>, responder: UriSchemeResponder) {
    let path = request.uri().path().trim_start_matches('/').to_string();
    let target = percent_decode(&path);
    let range = request
        .headers()
        .get("range")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());

    tauri::async_runtime::spawn(async move {
        responder.respond(fetch(target, range).await);
    });
}
