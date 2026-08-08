
use serde_json::Value;

use super::cache;
use super::http;
use super::parse::{flag, hit_from, num, rich_text, text};
use super::types::{GeniusArtist, GeniusHit, GeniusSocial};

const SONGS_LIMIT: usize = 50;
const SONGS_PAGES: usize = 6;

fn socials_of(artist: &Value) -> Vec<GeniusSocial> {
    let mut out = Vec::new();
    for (key, kind, base) in [
        ("instagram_name", "Instagram", "https://instagram.com/"),
        ("twitter_name", "X", "https://x.com/"),
        ("facebook_name", "Facebook", "https://facebook.com/"),
    ] {
        if let Some(handle) = text(artist, key) {
            out.push(GeniusSocial {
                kind: kind.to_string(),
                url: format!("{base}{handle}"),
                handle,
            });
        }
    }
    out
}

async fn songs_of(token: &str, id: u64) -> Vec<GeniusHit> {
    let mut out: Vec<GeniusHit> = Vec::new();

    for page in 1..=SONGS_PAGES {
        let path = format!(
            "/artists/{id}/songs?sort=popularity&per_page={SONGS_LIMIT}&page={page}"
        );
        let Ok(payload) = http::get(token, &path).await else {
            break;
        };
        let Some(list) = payload.get("songs").and_then(Value::as_array) else {
            break;
        };
        if list.is_empty() {
            break;
        }

        let received = list.len();
        for item in list {
            if let Some(hit) = hit_from(item) {
                if !out.iter().any(|existing| existing.id == hit.id) {
                    out.push(hit);
                }
            }
        }

        if received < SONGS_LIMIT {
            break;
        }
    }

    out
}

pub async fn build(token: &str, id: u64, refresh: bool) -> Result<GeniusArtist, String> {
    if !refresh {
        if let Some(found) = cache::artists().get(&id) {
            return Ok(found);
        }
    }

    let payload = http::get(token, &format!("/artists/{id}?text_format=plain,html")).await?;
    let artist = payload
        .get("artist")
        .cloned()
        .ok_or_else(|| "Genius: пустой ответ по артисту".to_string())?;

    let built = GeniusArtist {
        id,
        name: text(&artist, "name").unwrap_or_default(),
        url: text(&artist, "url").unwrap_or_default(),
        image: text(&artist, "image_url"),
        header: text(&artist, "header_image_url"),
        description: rich_text(&artist, "description").filter(|value| value != "?"),
        followers: num(&artist, "followers_count"),
        alternate_names: artist
            .get("alternate_names")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        socials: socials_of(&artist),
        songs: songs_of(token, id).await,
        iq: artist.get("iq").and_then(Value::as_i64),
        verified: flag(&artist, "is_verified") || flag(&artist, "is_meme_verified"),
        instagram: text(&artist, "instagram_name"),
    };

    cache::artists().put(id, built.clone());
    Ok(built)
}
