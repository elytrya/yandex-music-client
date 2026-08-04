use tauri::http::{Request, Response};
use tauri::UriSchemeResponder;

pub const SCHEME: &str = "ymstream";

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

async fn fetch(target: String, range: Option<String>) -> Response<Vec<u8>> {
    if !target.starts_with("https://") && !target.starts_with("http://") {
        return empty(400);
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
