mod account;
mod catalog;
mod dto;
mod library;
mod lrc;
mod playback;
mod radio;
mod raw;
mod util;

pub use dto::*;

pub(crate) use lrc::*;
pub(crate) use raw::*;
pub(crate) use util::*;

pub(crate) const BASE: &str = "https://api.music.yandex.net";
pub(crate) const SIGN_SALT: &str = "XGRlBW9FXlekgbPrRHuSiA";
pub(crate) const LYRICS_SECRET: &str = "p93jhgh689SBReK6ghtw62";
pub(crate) const FILE_INFO_SECRET: &str = "kzqU4XhfCaY6B6JTHODeq5";
pub(crate) const MUSIC_CLIENT: &str = "YandexMusicAndroid/24023621";
pub(crate) const MUSIC_CLIENT_DESKTOP: &str = "YandexMusicDesktopAppWindows/5.13.2";

pub struct Yandex {
    http: reqwest::Client,
    token: String,
}

impl Yandex {
    pub fn new(token: &str) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("Yandex-Music-API")
            .build()
            .expect("reqwest client");
        Self {
            http,
            token: token.to_string(),
        }
    }

    fn auth(&self) -> String {
        format!("OAuth {}", self.token)
    }

    async fn get_result(&self, path: &str) -> Result<serde_json::Value, String> {
        let resp = self
            .http
            .get(format!("{BASE}{path}"))
            .header("Authorization", self.auth())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        if status.as_u16() == 401 {
            return Err("Невалидный токен Яндекс Музыки".to_string());
        }
        let val: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(format!("Ошибка API Яндекса: {status}"));
        }
        Ok(val
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }

    async fn post_form(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<serde_json::Value, String> {
        let resp = self
            .http
            .post(format!("{BASE}{path}"))
            .header("Authorization", self.auth())
            .header("X-Yandex-Music-Client", MUSIC_CLIENT)
            .form(params)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        if status.as_u16() == 401 {
            return Err("Невалидный токен Яндекс Музыки".to_string());
        }
        let body = resp.text().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(format!("Ошибка API Яндекса: {status}"));
        }
        let val: serde_json::Value =
            serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
        Ok(val
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }

    async fn post_json(&self, path: &str) -> Result<serde_json::Value, String> {
        let resp = self
            .http
            .post(format!("{BASE}{path}"))
            .header("Authorization", self.auth())
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        if status.as_u16() == 401 {
            return Err("Невалидный токен Яндекс Музыки".to_string());
        }
        let val: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(format!("Ошибка API Яндекса: {status}"));
        }
        match val.get("result") {
            Some(inner) if !inner.is_null() => Ok(inner.clone()),
            _ => Ok(val),
        }
    }
}
