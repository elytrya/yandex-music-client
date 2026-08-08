
mod artist;
mod cache;
mod http;
mod lyrics;
mod parse;
mod search;
mod song;
mod state;
mod types;

use serde_json::Value;

pub use types::{
    GeniusAlbum, GeniusArtist, GeniusAuthor, GeniusHit, GeniusMedia, GeniusPerson, GeniusPersonHit,
    GeniusQuote, GeniusRelation, GeniusSocial, GeniusSong,
};

#[tauri::command]
pub async fn genius_check(token: String) -> Result<String, String> {
    let payload = http::get(&token, "/search?q=test").await?;
    let count = payload
        .get("hits")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    Ok(format!(
        "Токен работает, Genius отвечает ({count} результатов на пробный запрос)"
    ))
}

#[tauri::command]
pub async fn genius_search(token: String, query: String) -> Result<Vec<GeniusHit>, String> {
    search::hits(&token, query.trim()).await
}

#[tauri::command]
pub async fn genius_search_people(
    token: String,
    query: String,
) -> Result<Vec<GeniusPersonHit>, String> {
    search::people(&token, query.trim()).await
}

#[tauri::command]
pub async fn genius_song(token: String, id: u64, force: Option<bool>) -> Result<GeniusSong, String> {
    song::build(&token, id, force.unwrap_or(false)).await
}

#[tauri::command]
pub async fn genius_lookup(
    token: String,
    title: String,
    artist: String,
    force: Option<bool>,
) -> Result<Option<GeniusSong>, String> {
    let title = title.trim().to_string();
    let artist = artist.trim().to_string();
    if title.is_empty() {
        return Ok(None);
    }

    let refresh = force.unwrap_or(false);
    let key = format!("{}|{}", search::norm(&title), search::norm(&artist));

    if !refresh {
        if let Some(id) = cache::matches().get(&key) {
            return song::build(&token, id, false).await.map(Some);
        }
    }

    let mut found = search::match_song(&token, &title, &artist).await?;
    if found.is_none() && !artist.is_empty() {
        found = search::match_song(&token, &title, "").await?;
    }

    let Some(id) = found else {
        return Ok(None);
    };

    cache::matches().put(key, id);
    song::build(&token, id, refresh).await.map(Some)
}

#[tauri::command]
pub async fn genius_artist(
    token: String,
    id: u64,
    force: Option<bool>,
) -> Result<GeniusArtist, String> {
    artist::build(&token, id, force.unwrap_or(false)).await
}

#[tauri::command]
pub fn genius_clear_cache() {
    cache::clear_all();
}
