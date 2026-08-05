use tauri::State;

use crate::state::AppState;
use crate::yandex::{PlaylistDto, PlaylistTracksDto, TrackDto, Yandex};

#[tauri::command]
pub async fn get_liked_tracks(state: State<'_, AppState>) -> Result<Vec<TrackDto>, String> {
    let session = state.session()?;
    Yandex::new(&session.token).liked_tracks(session.uid).await
}

#[tauri::command]
pub async fn get_liked_ids(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let session = state.session()?;
    Yandex::new(&session.token).liked_ids(session.uid).await
}

#[tauri::command]
pub async fn get_disliked_ids(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let session = state.session()?;
    Yandex::new(&session.token).disliked_ids(session.uid).await
}

#[tauri::command]
pub async fn set_like(
    id: String,
    remove: bool,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let session = state.session()?;
    Yandex::new(&session.token)
        .set_like(session.uid, &id, false, remove)
        .await
}

#[tauri::command]
pub async fn set_dislike(
    id: String,
    remove: bool,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let session = state.session()?;
    Yandex::new(&session.token)
        .set_like(session.uid, &id, true, remove)
        .await
}

#[tauri::command]
pub async fn get_playlists(state: State<'_, AppState>) -> Result<Vec<PlaylistDto>, String> {
    let session = state.session()?;
    Yandex::new(&session.token).playlists(session.uid).await
}

#[tauri::command]
pub async fn playlist_memberships(
    kinds: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<PlaylistTracksDto>, String> {
    if kinds.is_empty() {
        return Ok(Vec::new());
    }
    let session = state.session()?;
    let list = kinds
        .iter()
        .map(|kind| kind.to_string())
        .collect::<Vec<_>>()
        .join(",");
    Yandex::new(&session.token)
        .playlists_track_ids(session.uid, &list)
        .await
}

#[tauri::command]
pub async fn get_playlist_tracks(
    kind: i64,
    state: State<'_, AppState>,
) -> Result<Vec<TrackDto>, String> {
    let session = state.session()?;
    Yandex::new(&session.token)
        .playlist_tracks(session.uid, kind)
        .await
}

#[tauri::command]
pub async fn playlist_add_track(
    kind: i64,
    id: String,
    album: String,
    at: Option<i64>,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let session = state.session()?;
    Yandex::new(&session.token)
        .playlist_insert(session.uid, kind, at.unwrap_or(0), &id, &album)
        .await
}

#[tauri::command]
pub async fn playlist_remove_track(
    kind: i64,
    id: String,
    album: String,
    at: i64,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let session = state.session()?;
    Yandex::new(&session.token)
        .playlist_delete(session.uid, kind, at, &id, &album)
        .await
}

#[tauri::command]
pub async fn playlist_move_track(
    kind: i64,
    id: String,
    album: String,
    from: i64,
    to: i64,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let session = state.session()?;
    Yandex::new(&session.token)
        .playlist_move(session.uid, kind, from, to, &id, &album)
        .await
}

#[tauri::command]
pub async fn playlist_create(
    title: String,
    public: Option<bool>,
    state: State<'_, AppState>,
) -> Result<PlaylistDto, String> {
    let session = state.session()?;
    Yandex::new(&session.token)
        .playlist_create(session.uid, &title, public.unwrap_or(false))
        .await
}

#[tauri::command]
pub async fn playlist_rename(
    kind: i64,
    title: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let session = state.session()?;
    Yandex::new(&session.token)
        .playlist_rename(session.uid, kind, &title)
        .await
}

#[tauri::command]
pub async fn playlist_set_visibility(
    kind: i64,
    public: bool,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let session = state.session()?;
    Yandex::new(&session.token)
        .playlist_set_visibility(session.uid, kind, public)
        .await
}

#[tauri::command]
pub async fn playlist_delete(kind: i64, state: State<'_, AppState>) -> Result<bool, String> {
    let session = state.session()?;
    Yandex::new(&session.token)
        .playlist_remove(session.uid, kind)
        .await
}

#[tauri::command]
pub async fn playlist_clear(kind: i64, state: State<'_, AppState>) -> Result<i64, String> {
    let session = state.session()?;
    Yandex::new(&session.token)
        .playlist_clear(session.uid, kind)
        .await
}

#[tauri::command]
pub async fn playlist_recommendations(
    kind: i64,
    state: State<'_, AppState>,
) -> Result<Vec<TrackDto>, String> {
    let session = state.session()?;
    Yandex::new(&session.token)
        .playlist_recommendations(session.uid, kind)
        .await
}
