use super::*;

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct YRotorSession {
    radio_session_id: Option<String>,
    batch_id: Option<String>,
    sequence: Vec<YTrackWrap>,
}

impl Yandex {
    pub async fn wave_session(
        &self,
        station: &str,
        session_id: Option<&str>,
        queue: &[String],
    ) -> Result<WaveResponse, String> {
        let (path, body) = match session_id {
            Some(sid) => (
                format!("/rotor/session/{sid}/tracks"),
                serde_json::json!({ "queue": queue }),
            ),
            None => (
                "/rotor/session/new".to_string(),
                serde_json::json!({ "seeds": [station], "includeTracksInResponse": true }),
            ),
        };
        let result = self.post_value(&path, body).await?;
        let parsed: YRotorSession =
            serde_json::from_value(result).map_err(|e| e.to_string())?;
        let tracks = parsed
            .sequence
            .iter()
            .filter_map(|w| w.track.as_ref())
            .map(map_track)
            .collect();
        Ok(WaveResponse {
            station_id: station.to_string(),
            radio_session_id: parsed
                .radio_session_id
                .or_else(|| session_id.map(|s| s.to_string())),
            batch_id: parsed.batch_id,
            tracks,
        })
    }

    pub async fn wheel(&self) -> Result<Vec<WheelItemDto>, String> {
        let value = self.post_json("/wheel/new").await?;
        let mut out = Vec::new();
        let items = match value.get("items").and_then(|i| i.as_array()) {
            Some(arr) => arr,
            None => return Ok(out),
        };

        for item in items {
            let kind_raw = item
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_uppercase();
            let id = item
                .get("id")
                .map(val_to_id)
                .unwrap_or_default();
            let data = item.get("data");
            let item_desc = item.get("description").and_then(|v| v.as_str());

            match kind_raw.as_str() {
                "WAVE" => {
                    let wave = data.and_then(|d| d.get("wave"));
                    let agent = data.and_then(|d| d.get("agent"));
                    let seed = wave
                        .and_then(|w| w.get("seeds"))
                        .and_then(|s| s.as_array())
                        .and_then(|a| a.first())
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| id.clone());
                    out.push(WheelItemDto {
                        id: id.clone(),
                        kind: "wave".to_string(),
                        name: wave
                            .and_then(|w| w.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("Волна")
                            .to_string(),
                        description: wave
                            .and_then(|w| w.get("description"))
                            .and_then(|v| v.as_str())
                            .or(item_desc)
                            .map(|s| s.to_string()),
                        cover_url: img(
                            agent
                                .and_then(|a| str_at(a, &["cover", "uri"]))
                                .as_deref(),
                            "400x400",
                        ),
                        color: agent.and_then(|a| str_at(a, &["cover", "color"])),
                        station: Some(seed),
                        artists: Vec::new(),
                    });
                }
                "ALBUM" => {
                    let album = data.and_then(|d| d.get("album"));
                    let artists = data
                        .and_then(|d| d.get("artists"))
                        .and_then(|a| a.as_array())
                        .map(|arr| {
                            arr.iter()
                                .map(|a| ArtistDto {
                                    id: a.get("id").map(val_to_id).unwrap_or_default(),
                                    name: a
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    cover_url: img(
                                        str_at(a, &["cover", "uri"])
                                            .or_else(|| str_at(a, &["ogImage"]))
                                            .as_deref(),
                                        "200x200",
                                    ),
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let album_id = album
                        .and_then(|a| a.get("id"))
                        .map(val_to_id)
                        .unwrap_or_else(|| id.clone());
                    out.push(WheelItemDto {
                        id: album_id,
                        kind: "album".to_string(),
                        name: album
                            .and_then(|a| a.get("title"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("Альбом")
                            .to_string(),
                        description: item_desc.map(|s| s.to_string()),
                        cover_url: img(
                            album.and_then(|a| str_at(a, &["cover", "uri"])).as_deref(),
                            "400x400",
                        ),
                        color: album.and_then(|a| str_at(a, &["cover", "color"])),
                        station: None,
                        artists,
                    });
                }
                "ARTIST" => {
                    let artist = data.and_then(|d| d.get("artist"));
                    let artist_id = artist
                        .and_then(|a| a.get("id"))
                        .map(val_to_id)
                        .unwrap_or_else(|| id.clone());
                    out.push(WheelItemDto {
                        id: artist_id,
                        kind: "artist".to_string(),
                        name: artist
                            .and_then(|a| a.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("Артист")
                            .to_string(),
                        description: item_desc.map(|s| s.to_string()),
                        cover_url: img(
                            artist.and_then(|a| str_at(a, &["cover", "uri"])).as_deref(),
                            "400x400",
                        ),
                        color: artist.and_then(|a| str_at(a, &["cover", "color"])),
                        station: None,
                        artists: Vec::new(),
                    });
                }
                _ => {}
            }
        }

        Ok(out)
    }

    pub async fn station_info(&self, station: &str) -> Result<StationInfoDto, String> {
        let result = self
            .get_result(&format!("/rotor/station/{station}/info"))
            .await?;
        let value = match result {
            serde_json::Value::Array(items) => items
                .into_iter()
                .next()
                .ok_or_else(|| "empty station info".to_string())?,
            other => other,
        };

        let station_obj = value.get("station").unwrap_or(&value);
        let settings = value
            .get("settings2")
            .or_else(|| value.get("settings"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let restrictions = station_obj
            .get("restrictions2")
            .or_else(|| station_obj.get("restrictions"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        Ok(StationInfoDto {
            id: station.to_string(),
            name: station_obj
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(station)
                .to_string(),
            language: setting_value(&settings, "language"),
            mood_energy: setting_value(&settings, "moodEnergy"),
            diversity: setting_value(&settings, "diversity"),
            languages: enum_options(&restrictions, "language"),
            moods: enum_options(&restrictions, "moodEnergy"),
            diversities: enum_options(&restrictions, "diversity"),
        })
    }

    pub async fn set_station_settings(
        &self,
        station: &str,
        language: Option<&str>,
        mood_energy: Option<&str>,
        diversity: Option<&str>,
    ) -> Result<bool, String> {
        let mut body = serde_json::Map::new();
        if let Some(v) = language {
            body.insert("language".into(), serde_json::Value::String(v.to_string()));
        }
        if let Some(v) = mood_energy {
            body.insert("moodEnergy".into(), serde_json::Value::String(v.to_string()));
        }
        if let Some(v) = diversity {
            body.insert("diversity".into(), serde_json::Value::String(v.to_string()));
        }
        if body.is_empty() {
            return Ok(false);
        }

        let response = self
            .http
            .post(format!("{BASE}/rotor/station/{station}/settings3"))
            .header("Authorization", self.auth())
            .header("X-Yandex-Music-Client", MUSIC_CLIENT)
            .json(&serde_json::Value::Object(body))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("station settings failed: {}", response.status()));
        }
        Ok(true)
    }

    pub async fn feedback(
        &self,
        station: &str,
        kind: &str,
        track_id: Option<&str>,
        batch_id: Option<&str>,
        total: Option<f64>,
    ) -> Result<bool, String> {
        let mut url = format!("{BASE}/rotor/station/{station}/feedback");
        if let Some(b) = batch_id {
            url.push_str(&format!("?batch-id={b}"));
        }
        let mut body = serde_json::json!({ "type": kind, "from": "mashiro" });
        if let Some(t) = track_id {
            body["trackId"] = serde_json::json!(t);
        }
        if let Some(s) = total {
            body["totalPlayedSeconds"] = serde_json::json!(s);
        }
        let resp = self
            .http
            .post(url)
            .header("Authorization", self.auth())
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Ok(resp.status().is_success())
    }

    pub async fn stations(&self) -> Result<Vec<StationDto>, String> {
        let result = self.get_result("/rotor/stations/dashboard").await?;
        let mut out = Vec::new();
        if let Some(arr) = result.get("stations").and_then(|s| s.as_array()) {
            for item in arr {
                let st = item.get("station").unwrap_or(item);
                let id_obj = st.get("id");
                let kind = id_obj
                    .and_then(|i| i.get("type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let tag = id_obj
                    .and_then(|i| i.get("tag"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if kind.is_empty() || tag.is_empty() {
                    continue;
                }
                let icon = st.get("icon");
                out.push(StationDto {
                    id: format!("{kind}:{tag}"),
                    name: st
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Станция")
                        .to_string(),
                    color: icon
                        .and_then(|i| i.get("backgroundColor"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    icon_url: img(
                        icon.and_then(|i| i.get("imageUrl")).and_then(|v| v.as_str()),
                        "200x200",
                    ),
                });
            }
        }
        Ok(out)
    }
}

fn setting_value(settings: &serde_json::Value, key: &str) -> Option<String> {
    settings
        .get(key)
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
}

fn enum_options(restrictions: &serde_json::Value, key: &str) -> Vec<StationOptionDto> {
    restrictions
        .get(key)
        .and_then(|v| v.get("possibleValues"))
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let value = item.get("value").and_then(|v| v.as_str())?;
                    let name = item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(value)
                        .to_string();
                    Some(StationOptionDto {
                        value: value.to_string(),
                        name,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}
