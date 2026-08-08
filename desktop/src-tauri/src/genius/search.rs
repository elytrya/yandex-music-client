
use serde_json::Value;

use super::cache;
use super::http;
use super::parse::{hit_from, person_hit};
use super::types::{GeniusHit, GeniusPersonHit};

const HIT_LIMIT: usize = 8;
const PEOPLE_LIMIT: usize = 10;

pub fn norm(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut gap = false;

    for ch in value.to_lowercase().chars() {
        let ch = if ch == 'ё' { 'е' } else { ch };
        if ch.is_alphanumeric() {
            if gap && !out.is_empty() {
                out.push(' ');
            }
            gap = false;
            out.push(ch);
        } else {
            gap = true;
        }
    }

    out
}

pub fn core(value: &str) -> String {
    let mut plain = String::with_capacity(value.len());
    let mut depth = 0usize;

    for ch in value.chars() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            other if depth == 0 => plain.push(other),
            _ => {}
        }
    }

    let mut lower = plain.to_lowercase();
    for marker in [
        " feat.", " feat ", " ft.", " ft ", " prod.", " prod ", " with ", " совместно ",
    ] {
        if let Some(at) = lower.find(marker) {
            lower.truncate(at);
        }
    }

    norm(&lower)
}

pub fn score(hit: &GeniusHit, title: &str, artist: &str) -> i32 {
    let want_title = core(title);
    let want_artist = norm(artist);
    let got_title = core(&hit.title);
    let got_artist = norm(&hit.artist);

    let mut points = 0;

    if !want_title.is_empty() && !got_title.is_empty() {
        if got_title == want_title {
            points += 3;
        } else if got_title.contains(&want_title) || want_title.contains(&got_title) {
            points += 2;
        }
    }

    if !want_artist.is_empty() && !got_artist.is_empty() {
        if got_artist == want_artist {
            points += 2;
        } else if got_artist.contains(&want_artist) || want_artist.contains(&got_artist) {
            points += 1;
        }
    }

    points
}

pub async fn hits(token: &str, query: &str) -> Result<Vec<GeniusHit>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let key = norm(query);
    if let Some(found) = cache::song_search().get(&key) {
        return Ok(found);
    }

    let body = http::get(token, &format!("/search?q={}", http::encode(query))).await?;
    let out: Vec<GeniusHit> = body
        .get("hits")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter(|item| {
                    item.get("type").and_then(Value::as_str).unwrap_or("song") == "song"
                })
                .filter_map(|item| item.get("result"))
                .filter_map(hit_from)
                .take(HIT_LIMIT)
                .collect()
        })
        .unwrap_or_default();

    cache::song_search().put(key, out.clone());
    Ok(out)
}

pub async fn people(token: &str, query: &str) -> Result<Vec<GeniusPersonHit>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let key = norm(query);
    if let Some(found) = cache::people_search().get(&key) {
        return Ok(found);
    }

    let mut out: Vec<GeniusPersonHit> = Vec::new();

    let multi = http::get_site(&format!(
        "/search/multi?per_page=10&q={}",
        http::encode(query)
    ))
    .await;

    if let Ok(body) = multi {
        if let Some(sections) = body.get("sections").and_then(Value::as_array) {
            for section in sections {
                let kind = section.get("type").and_then(Value::as_str).unwrap_or("");
                if kind != "artist" {
                    continue;
                }
                let Some(list) = section.get("hits").and_then(Value::as_array) else {
                    continue;
                };
                for item in list {
                    let node = item.get("result").unwrap_or(item);
                    if let Some(found) = person_hit(node) {
                        if !out.iter().any(|old| old.id == found.id) {
                            out.push(found);
                        }
                    }
                }
            }
        }
    }

    if out.is_empty() && !token.trim().is_empty() {
        if let Ok(body) = http::get(token, &format!("/search?q={}", http::encode(query))).await {
            if let Some(list) = body.get("hits").and_then(Value::as_array) {
                for item in list {
                    let Some(song) = item.get("result") else {
                        continue;
                    };
                    let mut people: Vec<&Value> = Vec::new();
                    if let Some(primary) = song.get("primary_artist") {
                        people.push(primary);
                    }
                    if let Some(extra) = song.get("featured_artists").and_then(Value::as_array) {
                        people.extend(extra.iter());
                    }
                    for node in people {
                        if let Some(found) = person_hit(node) {
                            if !out.iter().any(|old| old.id == found.id) {
                                out.push(found);
                            }
                        }
                    }
                }
            }
        }
    }

    out.truncate(PEOPLE_LIMIT);
    cache::people_search().put(key, out.clone());
    Ok(out)
}

pub async fn match_song(token: &str, title: &str, artist: &str) -> Result<Option<u64>, String> {
    let query = format!("{title} {artist}");
    let found = hits(token, query.trim()).await?;

    let mut best: Option<(i32, u64)> = None;
    for hit in &found {
        let points = score(hit, title, artist);
        if points < 3 {
            continue;
        }
        if best.map(|(old, _)| points > old).unwrap_or(true) {
            best = Some((points, hit.id));
        }
    }

    Ok(best.map(|(_, id)| id))
}
