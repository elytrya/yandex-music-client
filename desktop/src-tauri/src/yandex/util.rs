use super::dto::{AlbumBriefDto, ArtistDto, TrackDto};
use super::raw::{YAlbum, YArtist, YTrack};

pub fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn val_to_id(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

pub fn img(uri: Option<&str>, size: &str) -> Option<String> {
    uri.filter(|u| !u.is_empty()).map(|u| {
        let replaced = u.replace("%%", size);
        if replaced.starts_with("http") {
            replaced
        } else {
            format!("https://{replaced}")
        }
    })
}

pub fn cover_url(uri: &Option<String>) -> Option<String> {
    img(uri.as_deref(), "200x200")
}

pub fn between<'a>(s: &'a str, a: &str, b: &str) -> Option<&'a str> {
    let start = s.find(a)? + a.len();
    let end = s[start..].find(b)? + start;
    Some(&s[start..end])
}

pub fn str_at(v: &serde_json::Value, path: &[&str]) -> Option<String> {
    let mut cur = v;
    for key in path {
        cur = cur.get(*key)?;
    }
    cur.as_str().map(|s| s.to_string())
}

pub fn map_artist(a: &YArtist) -> ArtistDto {
    let uri = a
        .cover
        .as_ref()
        .and_then(|c| c.uri.clone())
        .or_else(|| a.og_image.clone());
    ArtistDto {
        id: val_to_id(&a.id),
        name: a.name.clone().unwrap_or_default(),
        cover_url: img(uri.as_deref(), "200x200"),
    }
}

pub fn map_track(t: &YTrack) -> TrackDto {
    let album = t.albums.first();
    TrackDto {
        id: val_to_id(&t.id),
        title: t.title.clone().unwrap_or_default(),
        artists: t.artists.iter().map(map_artist).collect(),
        album_id: album.map(|a| val_to_id(&a.id)),
        album_title: album.and_then(|a| a.title.clone()),
        cover_url: cover_url(&t.cover_uri.clone().or_else(|| {
            album.and_then(|a| a.cover_uri.clone())
        })),
        duration_ms: t.duration_ms,
        available: t.available.unwrap_or(true),
    }
}

pub fn map_album_brief(a: &YAlbum) -> AlbumBriefDto {
    AlbumBriefDto {
        id: val_to_id(&a.id),
        title: a.title.clone().unwrap_or_default(),
        cover_url: cover_url(&a.cover_uri),
        year: a.year,
    }
}
