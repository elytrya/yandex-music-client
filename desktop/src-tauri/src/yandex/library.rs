use super::*;

impl Yandex {
    pub async fn like(&self, uid: i64, track_id: &str, dislike: bool) -> Result<bool, String> {
        self.set_like(uid, track_id, dislike, false).await
    }

    pub async fn set_like(
        &self,
        uid: i64,
        track_id: &str,
        dislike: bool,
        remove: bool,
    ) -> Result<bool, String> {
        if uid <= 0 {
            return Err("Нет данных аккаунта, войди заново".to_string());
        }
        let bucket = if dislike { "dislikes" } else { "likes" };
        let action = if remove { "remove" } else { "add-multiple" };
        let url = format!("{BASE}/users/{uid}/{bucket}/tracks/{action}");
        let status = self.post_track_ids(&url, track_id).await?;
        if status < 300 {
            return Ok(true);
        }
        if !remove {
            let legacy = format!("{BASE}/users/{uid}/{bucket}/tracks/add");
            let retry = self.post_track_ids(&legacy, track_id).await?;
            if retry < 300 {
                return Ok(true);
            }
        }
        Err(format!("Не удалось обновить оценку: {status}"))
    }

    async fn post_track_ids(&self, url: &str, track_id: &str) -> Result<u16, String> {
        let resp = self
            .http
            .post(url)
            .header("Authorization", self.auth())
            .header("X-Yandex-Music-Client", MUSIC_CLIENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!("track-ids={}", urlencode(track_id)))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Ok(resp.status().as_u16())
    }

    pub async fn liked_ids(&self, uid: i64) -> Result<Vec<String>, String> {
        let result = self
            .get_result(&format!("/users/{uid}/likes/tracks"))
            .await?;
        Ok(result
            .get("library")
            .and_then(|l| l.get("tracks"))
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.get("id").map(val_to_id))
                    .collect()
            })
            .unwrap_or_default())
    }

    pub async fn liked_tracks(&self, uid: i64) -> Result<Vec<TrackDto>, String> {
        let ids = self.liked_ids(uid).await?;
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let joined = ids.iter().take(200).cloned().collect::<Vec<_>>().join(",");
        let result = self
            .get_result(&format!("/tracks?track-ids={joined}"))
            .await?;
        let list: Vec<YTrack> = serde_json::from_value(result).map_err(|e| e.to_string())?;
        Ok(list.iter().map(map_track).collect())
    }

    pub async fn playlists(&self, uid: i64) -> Result<Vec<PlaylistDto>, String> {
        let result = self
            .get_result(&format!("/users/{uid}/playlists/list"))
            .await?;
        let raw: Vec<YPlaylist> = serde_json::from_value(result).map_err(|e| e.to_string())?;
        Ok(raw
            .into_iter()
            .map(|p| PlaylistDto {
                kind: p.kind,
                title: p.title.unwrap_or_else(|| "Без названия".to_string()),
                track_count: p.track_count.unwrap_or(0),
                cover_url: cover_url(&p.cover.and_then(|c| c.uri).or(p.og_image)),
                owner_uid: p.owner.and_then(|o| o.uid),
            })
            .collect())
    }

    pub async fn playlist_tracks(&self, uid: i64, kind: i64) -> Result<Vec<TrackDto>, String> {
        let result = self
            .get_result(&format!("/users/{uid}/playlists/{kind}"))
            .await?;
        let full: YPlaylistFull = serde_json::from_value(result).map_err(|e| e.to_string())?;
        Ok(full
            .tracks
            .iter()
            .filter_map(|w| w.track.as_ref())
            .map(map_track)
            .collect())
    }

    async fn revision(&self, uid: i64, kind: i64) -> Result<i64, String> {
        let result = self
            .get_result(&format!("/users/{uid}/playlists/{kind}"))
            .await?;
        result
            .get("revision")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| "Не удалось прочитать версию плейлиста".to_string())
    }

    async fn change_relative(&self, uid: i64, kind: i64, diff: &str) -> Result<bool, String> {
        let mut last_status = 0u16;
        for attempt in 0..4 {
            let revision = self.revision(uid, kind).await?;
            let url = format!("{BASE}/users/{uid}/playlists/{kind}/change-relative");
            let resp = self
                .http
                .post(url)
                .header("Authorization", self.auth())
                .form(&[
                    ("diff", diff.to_string()),
                    ("revision", revision.to_string()),
                ])
                .send()
                .await
                .map_err(|e| e.to_string())?;

            let status = resp.status().as_u16();
            if resp.status().is_success() {
                return Ok(true);
            }
            last_status = status;

            let retryable = status == 412 || status == 409 || status == 429 || status >= 500;
            if !retryable || attempt == 3 {
                break;
            }

        }

        if last_status == 412 {
            return Err(
                "Плейлист изменился на сервере, обнови его и попробуй снова".to_string(),
            );
        }
        Err(format!("Не удалось изменить плейлист: {last_status}"))
    }

    pub async fn playlist_insert(
        &self,
        uid: i64,
        kind: i64,
        at: i64,
        track_id: &str,
        album_id: &str,
    ) -> Result<bool, String> {
        let diff = serde_json::json!([{
            "op": "insert",
            "at": at,
            "tracks": [{ "id": track_id, "albumId": album_id }],
        }]);
        self.change_relative(uid, kind, &diff.to_string()).await
    }

    pub async fn playlist_delete(
        &self,
        uid: i64,
        kind: i64,
        from: i64,
        track_id: &str,
        album_id: &str,
    ) -> Result<bool, String> {
        let diff = serde_json::json!([{
            "op": "delete",
            "from": from,
            "to": from + 1,
            "tracks": [{ "id": track_id, "albumId": album_id }],
        }]);
        self.change_relative(uid, kind, &diff.to_string()).await
    }

    pub async fn playlist_recommendations(
        &self,
        uid: i64,
        kind: i64,
    ) -> Result<Vec<TrackDto>, String> {
        let result = self
            .get_result(&format!("/users/{uid}/playlists/{kind}/recommendations"))
            .await?;
        let items = result
            .get("tracks")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(items
            .into_iter()
            .filter_map(|raw| serde_json::from_value::<YTrack>(raw).ok())
            .map(|track| map_track(&track))
            .collect())
    }

    pub async fn playlist_create(
        &self,
        uid: i64,
        title: &str,
        public: bool,
    ) -> Result<PlaylistDto, String> {
        let title = title.trim();
        if title.is_empty() {
            return Err("Укажи название плейлиста".to_string());
        }
        let visibility = if public { "public" } else { "private" };
        let result = self
            .post_form(
                &format!("/users/{uid}/playlists/create"),
                &[
                    ("title", title.to_string()),
                    ("visibility", visibility.to_string()),
                ],
            )
            .await?;
        let created: YPlaylist = serde_json::from_value(result).map_err(|e| e.to_string())?;
        Ok(PlaylistDto {
            kind: created.kind,
            title: created.title.unwrap_or_else(|| title.to_string()),
            track_count: created.track_count.unwrap_or(0),
            cover_url: cover_url(&created.cover.and_then(|c| c.uri).or(created.og_image)),
            owner_uid: created.owner.and_then(|o| o.uid),
        })
    }

    pub async fn playlist_rename(&self, uid: i64, kind: i64, title: &str) -> Result<bool, String> {
        let title = title.trim();
        if title.is_empty() {
            return Err("Название не может быть пустым".to_string());
        }
        self.post_form(
            &format!("/users/{uid}/playlists/{kind}/name"),
            &[("value", title.to_string())],
        )
        .await
        .map(|_| true)
    }

    pub async fn playlist_set_visibility(
        &self,
        uid: i64,
        kind: i64,
        public: bool,
    ) -> Result<bool, String> {
        let visibility = if public { "public" } else { "private" };
        self.post_form(
            &format!("/users/{uid}/playlists/{kind}/visibility"),
            &[("value", visibility.to_string())],
        )
        .await
        .map(|_| true)
    }

    pub async fn playlist_remove(&self, uid: i64, kind: i64) -> Result<bool, String> {
        self.post_form(&format!("/users/{uid}/playlists/{kind}/delete"), &[])
            .await
            .map(|_| true)
    }

    pub async fn playlist_clear(&self, uid: i64, kind: i64) -> Result<i64, String> {
        let tracks = self.playlist_tracks(uid, kind).await?;
        let count = tracks.len() as i64;
        if count == 0 {
            return Ok(0);
        }
        let diff = serde_json::json!([{ "op": "delete", "from": 0, "to": count }]);
        self.change_relative(uid, kind, &diff.to_string()).await?;
        Ok(count)
    }

    pub async fn playlist_move(
        &self,
        uid: i64,
        kind: i64,
        from: i64,
        to: i64,
        track_id: &str,
        album_id: &str,
    ) -> Result<bool, String> {
        if from == to {
            return Ok(true);
        }
        self.playlist_delete(uid, kind, from, track_id, album_id)
            .await?;
        let target = if to > from { to } else { to };
        self.playlist_insert(uid, kind, target, track_id, album_id)
            .await
    }
}
