use serde::Deserialize;
use tauri::State;

use super::{Ok_, DEFAULT_STATION};
use crate::state::AppState;
use crate::yandex::{StationDto, StationInfoDto, WaveResponse, WheelItemDto, Yandex};

#[derive(Deserialize)]
pub struct FeedbackPayload {
    #[serde(default = "default_station")]
    station_id: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    track_id: Option<String>,
    #[serde(default)]
    batch_id: Option<String>,
    #[serde(default)]
    total_played_seconds: Option<f64>,
}

fn default_station() -> String {
    DEFAULT_STATION.to_string()
}

#[tauri::command]
pub async fn get_wave(
    queue: Option<String>,
    station: Option<String>,
    state: State<'_, AppState>,
) -> Result<WaveResponse, String> {
    let session = state.session()?;
    let station = station.unwrap_or_else(|| DEFAULT_STATION.to_string());
    Yandex::new(&session.token)
        .wave(&station, queue.as_deref())
        .await
}

#[tauri::command]
pub async fn get_wheel(state: State<'_, AppState>) -> Result<Vec<WheelItemDto>, String> {
    let session = state.session()?;
    Yandex::new(&session.token).wheel().await
}

#[tauri::command]
pub async fn get_stations(state: State<'_, AppState>) -> Result<Vec<StationDto>, String> {
    let session = state.session()?;
    Yandex::new(&session.token).stations().await
}

#[tauri::command]
pub async fn wave_feedback(
    payload: FeedbackPayload,
    state: State<'_, AppState>,
) -> Result<Ok_, String> {
    let session = state.session()?;
    let client = Yandex::new(&session.token);
    let ok = match payload.kind.as_str() {
        "like" => match payload.track_id.as_deref() {
            Some(id) => client.like(session.uid, id, false).await?,
            None => false,
        },
        "dislike" => match payload.track_id.as_deref() {
            Some(id) => client.like(session.uid, id, true).await?,
            None => false,
        },
        other => {
            client
                .feedback(
                    &payload.station_id,
                    other,
                    payload.track_id.as_deref(),
                    payload.batch_id.as_deref(),
                    payload.total_played_seconds,
                )
                .await?
        }
    };
    Ok(Ok_ { ok })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationSettingsPayload {
    #[serde(default = "default_station")]
    station: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    mood_energy: Option<String>,
    #[serde(default)]
    diversity: Option<String>,
}

#[tauri::command]
pub async fn station_info(
    station: Option<String>,
    state: State<'_, AppState>,
) -> Result<StationInfoDto, String> {
    let session = state.session()?;
    let station = station.unwrap_or_else(|| DEFAULT_STATION.to_string());
    Yandex::new(&session.token).station_info(&station).await
}

#[tauri::command]
pub async fn set_station_settings(
    payload: StationSettingsPayload,
    state: State<'_, AppState>,
) -> Result<Ok_, String> {
    let session = state.session()?;
    let client = Yandex::new(&session.token);
    let ok = client
        .set_station_settings(
            &payload.station,
            payload.language.as_deref(),
            payload.mood_energy.as_deref(),
            payload.diversity.as_deref(),
        )
        .await?;
    Ok(Ok_ { ok })
}
