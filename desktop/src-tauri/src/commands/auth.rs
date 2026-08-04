use tauri::State;

use crate::state::{AppState, Session};
use crate::yandex::{ProfileDto, Yandex};

#[tauri::command]
pub async fn auth_login(token: String, state: State<'_, AppState>) -> Result<ProfileDto, String> {
    let client = Yandex::new(&token);
    let (profile, uid) = client.account_status().await?;
    state.set_session(Session { token, uid });
    Ok(profile)
}

#[tauri::command]
pub async fn auth_me(state: State<'_, AppState>) -> Result<ProfileDto, String> {
    let session = state.session()?;
    let (profile, _uid) = Yandex::new(&session.token).account_status().await?;
    Ok(profile)
}

#[tauri::command]
pub async fn auth_logout(state: State<'_, AppState>) -> Result<(), String> {
    state.clear();
    Ok(())
}
