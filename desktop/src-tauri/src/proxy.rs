use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use tauri::http::{Request, Response};
use tauri::UriSchemeResponder;

pub const SCHEME: &str = "ymstream";

const FIRST_CHUNK: usize = 256 * 1024;
const CHUNK: usize = 1024 * 1024;
const CHUNK_LIMIT: usize = 48;
const WARM_BYTES: usize = 1536 * 1024;
const META_TTL_SECS: u64 = 600;

struct Chunk {
    url: String,
    start: usize,
    body: Vec<u8>,
}

struct Meta {
    total: usize,
    mime: String,
    at: Instant,
}

fn chunks() -> &'static Mutex<Vec<Chunk>> {
    static CHUNKS: OnceLock<Mutex<Vec<Chunk>>> = OnceLock::new();
    CHUNKS.get_or_init(|| Mutex::new(Vec::new()))
}

fn metas() -> &'static Mutex<HashMap<String, Meta>> {
    static METAS: OnceLock<Mutex<HashMap<String, Meta>>> = OnceLock::new();
    METAS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn inflight() -> &'static Mutex<HashSet<String>> {
    static INFLIGHT: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    INFLIGHT.get_or_init(|| Mutex::new(HashSet::new()))
}

fn http() -> Option<reqwest::Client> {
    static CLIENT: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .user_agent("Yandex-Music-API")
                .build()
                .ok()
        })
        .clone()
}

fn cached_chunk(url: &str, start: usize) -> Option<Vec<u8>> {
    let mut store = chunks().lock().ok()?;
    let at = store
        .iter()
        .position(|item| item.start == start && item.url == url)?;
    let item = store.remove(at);
    let body = item.body.clone();
    store.push(item);
    Some(body)
}

fn remember_chunk(url: &str, start: usize, body: &[u8]) {
    if body.is_empty() {
        return;
    }
    if let Ok(mut store) = chunks().lock() {
        store.retain(|item| !(item.start == start && item.url == url));
        store.push(Chunk {
            url: url.to_string(),
            start,
            body: body.to_vec(),
        });
        while store.len() > CHUNK_LIMIT {
            store.remove(0);
        }
    }
}

fn cached_meta(url: &str) -> Option<(usize, String)> {
    let mut store = metas().lock().ok()?;
    let fresh = match store.get(url) {
        Some(meta) => meta.at.elapsed().as_secs() < META_TTL_SECS,
        None => return None,
    };
    if !fresh {
        store.remove(url);
        return None;
    }
    store.get(url).map(|meta| (meta.total, meta.mime.clone()))
}

fn remember_meta(url: &str, total: usize, mime: &str) {
    if let Ok(mut store) = metas().lock() {
        store.insert(
            url.to_string(),
            Meta {
                total,
                mime: mime.to_string(),
                at: Instant::now(),
            },
        );
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

fn pick_mime(upstream: &str, target: &str) -> String {
    if upstream.starts_with("audio/") {
        return upstream.to_string();
    }
    mime_for(target).to_string()
}

fn empty(status: u16) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .body(Vec::new())
        .expect("empty response")
}

fn unsatisfiable(total: usize) -> Response<Vec<u8>> {
    Response::builder()
        .status(416)
        .header("Access-Control-Allow-Origin", "*")
        .header("Content-Range", format!("bytes */{total}"))
        .body(Vec::new())
        .unwrap_or_else(|_| empty(416))
}

fn parse_range(value: &str, total: usize) -> Option<(usize, Option<usize>)> {
    let raw = value.trim().strip_prefix("bytes=")?;
    let (from, to) = raw.split_once('-')?;
    let from = from.trim();
    let to = to.trim();
    if total == 0 {
        return None;
    }
    if from.is_empty() {
        let suffix: usize = to.parse().ok()?;
        if suffix == 0 {
            return None;
        }
        let start = total.saturating_sub(suffix);
        return Some((start, Some(total - 1)));
    }
    let start: usize = from.parse().ok()?;
    if start >= total {
        return None;
    }
    let end = if to.is_empty() {
        None
    } else {
        to.parse::<usize>().ok().map(|end| end.min(total - 1))
    };
    Some((start, end))
}

fn chunk_span(start: usize) -> usize {
    if start == 0 {
        FIRST_CHUNK
    } else {
        CHUNK
    }
}

async fn fetch_range(target: &str, start: usize, end: usize) -> Option<Vec<u8>> {
    let response = http()?
        .get(target)
        .header("Range", format!("bytes={start}-{end}"))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    Some(response.bytes().await.ok()?.to_vec())
}

async fn ensure_chunk(target: &str, start: usize, end: usize) -> Option<Vec<u8>> {
    if let Some(body) = cached_chunk(target, start) {
        return Some(body);
    }
    let body = fetch_range(target, start, end).await?;
    remember_chunk(target, start, &body);
    Some(body)
}

async fn meta(target: &str) -> Option<(usize, String)> {
    if let Some(found) = cached_meta(target) {
        return Some(found);
    }
    let response = http()?
        .get(target)
        .header("Range", "bytes=0-0")
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let upstream = response
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
    if total == 0 {
        return None;
    }
    let mime = pick_mime(&upstream, target);
    remember_meta(target, total, &mime);
    Some((total, mime))
}

async fn readahead(target: String, start: usize) {
    let key = format!("{target}#{start}");
    if cached_chunk(&target, start).is_some() {
        return;
    }
    if let Ok(mut set) = inflight().lock() {
        if !set.insert(key.clone()) {
            return;
        }
    }
    if let Some((total, _)) = meta(&target).await {
        if start < total {
            let end = (start + chunk_span(start)).min(total) - 1;
            let _ = ensure_chunk(&target, start, end).await;
        }
    }
    if let Ok(mut set) = inflight().lock() {
        set.remove(&key);
    }
}

async fn download_all(target: &str) -> Option<(Vec<u8>, String)> {
    let response = http()?.get(target).send().await.ok()?;
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
    let mime = pick_mime(&upstream, target);
    Some((body, mime))
}

fn whole(body: Vec<u8>, mime: String, range: Option<String>) -> Response<Vec<u8>> {
    let total = body.len();
    let mut builder = Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .header("Accept-Ranges", "bytes")
        .header("Content-Type", mime);
    if let Some((start, end)) = range.as_deref().and_then(|value| parse_range(value, total)) {
        let end = end.unwrap_or(total.saturating_sub(1));
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

pub fn clear() {
    if let Ok(mut store) = chunks().lock() {
        store.clear();
    }
    if let Ok(mut store) = metas().lock() {
        store.clear();
    }
    if let Ok(mut set) = inflight().lock() {
        set.clear();
    }
}

pub async fn warm(target: String) {
    if !target.starts_with("https://") && !target.starts_with("http://") {
        return;
    }
    let Some((total, _)) = meta(&target).await else {
        return;
    };
    let started = Instant::now();
    let mut start = 0usize;
    let mut loaded = 0usize;
    while start < total && loaded < WARM_BYTES {
        let end = (start + chunk_span(start)).min(total) - 1;
        if ensure_chunk(&target, start, end).await.is_none() {
            break;
        }
        loaded += end - start + 1;
        start = end + 1;
    }
    println!(
        "[cache] прогрето {} КБ из {} КБ за {} мс",
        loaded / 1024,
        total / 1024,
        started.elapsed().as_millis()
    );
}

async fn fetch(target: String, range: Option<String>) -> Response<Vec<u8>> {
    if !target.starts_with("https://") && !target.starts_with("http://") {
        return empty(400);
    }

    let Some((total, mime)) = meta(&target).await else {
        return match download_all(&target).await {
            Some((body, mime)) => whole(body, mime, range),
            None => empty(502),
        };
    };

    let (start, limit) = match range.as_deref() {
        Some(value) => match parse_range(value, total) {
            Some(parsed) => parsed,
            None => return unsatisfiable(total),
        },
        None => (0usize, None),
    };

    let mut end = (start + chunk_span(start)).min(total) - 1;
    if let Some(explicit) = limit {
        end = end.min(explicit);
    }
    if end < start {
        end = start;
    }

    let Some(body) = ensure_chunk(&target, start, end).await else {
        return empty(502);
    };
    if body.is_empty() {
        return empty(502);
    }
    let served_end = start + body.len() - 1;

    if served_end + 1 < total {
        let ahead = target.clone();
        let next = served_end + 1;
        tauri::async_runtime::spawn(async move {
            readahead(ahead, next).await;
        });
    }

    Response::builder()
        .status(206)
        .header("Access-Control-Allow-Origin", "*")
        .header("Accept-Ranges", "bytes")
        .header("Content-Type", mime)
        .header("Content-Length", body.len().to_string())
        .header(
            "Content-Range",
            format!("bytes {}-{}/{}", start, served_end, total),
        )
        .body(body)
        .unwrap_or_else(|_| empty(500))
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
