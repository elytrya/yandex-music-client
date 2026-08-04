use tauri::State;

use crate::state::AppState;
use crate::yandex::{AlbumPageDto, ArtistPageDto, SearchDto, TrackDto, TrackInfoDto, Yandex};

#[tauri::command]
pub async fn get_artist(id: String, state: State<'_, AppState>) -> Result<ArtistPageDto, String> {
    let session = state.session()?;
    Yandex::new(&session.token).artist(&id).await
}

#[tauri::command]
pub async fn get_artist_tracks(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<TrackDto>, String> {
    let session = state.session()?;
    Yandex::new(&session.token).artist_tracks(&id).await
}

#[tauri::command]
pub async fn get_album(id: String, state: State<'_, AppState>) -> Result<AlbumPageDto, String> {
    let session = state.session()?;
    Yandex::new(&session.token).album(&id).await
}

#[tauri::command]
pub async fn search_tracks(text: String, state: State<'_, AppState>) -> Result<SearchDto, String> {
    let session = state.session()?;
    Yandex::new(&session.token).search(&text).await
}

#[tauri::command]
pub async fn get_track(id: String, state: State<'_, AppState>) -> Result<TrackDto, String> {
    let session = state.session()?;
    Yandex::new(&session.token).track(&id).await
}

#[tauri::command]
pub async fn get_track_info(
    id: String,
    state: State<'_, AppState>,
) -> Result<TrackInfoDto, String> {
    let session = state.session()?;
    Yandex::new(&session.token).track_info(&id).await
}

#[tauri::command]
pub async fn search_suggest(
    text: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let session = state.session()?;
    Yandex::new(&session.token).search_suggest(&text).await
}

#[tauri::command]
pub async fn get_similar_tracks(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<TrackDto>, String> {
    let session = state.session()?;
    Yandex::new(&session.token).similar_tracks(&id).await
}
