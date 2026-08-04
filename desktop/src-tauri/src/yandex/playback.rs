use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use sha2::Sha256;
use super::*;

static STREAM_CACHE: std::sync::OnceLock<
    std::sync::Mutex<Vec<(String, StreamDto, std::time::Instant)>>,
> = std::sync::OnceLock::new();

const STREAM_TTL_SECS: u64 = 240;

static BEST_PROBE: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(usize::MAX);

fn lyrics_from_text(
    track_id: &str,
    text: &str,
    writers: Vec<String>,
) -> Result<LyricsDto, String> {
    let lines = parse_lrc(text);
    if lines.is_empty() {
        return Err("Для этого трека текста нет".to_string());
    }
    let synced = lines.iter().any(|line| line.time_ms > 0);
    Ok(LyricsDto {
        track_id: track_id.to_string(),
        synced,
        lines,
        writers,
    })
}

impl Yandex {
    fn file_info_sign(
        ts: u64,
        track_id: &str,
        quality: &str,
        codecs: &str,
        transports: &str,
    ) -> String {
        let message = format!(
            "{ts}{track_id}{quality}{}{}",
            codecs.replace(',', ""),
            transports.replace(',', "")
        );
        let mut mac = Hmac::<Sha256>::new_from_slice(FILE_INFO_SECRET.as_bytes())
            .expect("hmac key");
        mac.update(message.as_bytes());
        BASE64
            .encode(mac.finalize().into_bytes())
            .trim_end_matches('=')
            .to_string()
    }

    async fn stream_file_info(
        &self,
        track_id: &str,
        quality: &str,
        codecs: &str,
        client: &str,
    ) -> Result<StreamDto, String> {
        let transports = "raw";
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let sign = Self::file_info_sign(ts, track_id, quality, codecs, transports);
        let url = format!(
            "{BASE}/get-file-info?ts={ts}&trackId={track_id}&quality={quality}&codecs={}&transports={transports}&sign={}",
            urlencode(codecs),
            urlencode(&sign)
        );

        let resp = self
            .http
            .get(url)
            .timeout(std::time::Duration::from_secs(10))
            .header("Authorization", self.auth())
            .header("X-Yandex-Music-Client", client)
            .header("Accept", "application/json")
            .header("Accept-Language", "ru")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        if status.as_u16() == 401 {
            return Err("Невалидный токен Яндекс Музыки".to_string());
        }
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            let hint: String = text.chars().take(200).collect();
            return Err(format!("Поток недоступен: {} {hint}", status.as_u16()));
        }
        let body: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;

        let info = body
            .get("result")
            .and_then(|r| r.get("downloadInfo").or_else(|| r.get("download_info")))
            .cloned()
            .ok_or_else(|| "Нет данных о потоке".to_string())?;

        let raw_url = info
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                info.get("urls")
                    .and_then(|v| v.as_array())
                    .and_then(|list| list.first())
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_default();
        if raw_url.is_empty() {
            return Err("Нет ссылки на поток".to_string());
        }
        let url = if let Some(rest) = raw_url.strip_prefix("//") {
            format!("https://{rest}")
        } else {
            raw_url
        };

        let codec = info
            .get("codec")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let bitrate = info
            .get("bitrate")
            .and_then(|v| v.as_i64())
            .unwrap_or_default();

        Ok(StreamDto {
            track_id: track_id.to_string(),
            url,
            codec,
            bitrate,
            source: "get-file-info".to_string(),
        })
    }

    fn lossy(codec: &str) -> bool {
        let c = codec.to_ascii_lowercase();
        !(c.contains("flac")
            || c.contains("alac")
            || c.starts_with("wav")
            || c.starts_with("aiff"))
    }

    fn report(track_id: &str, dto: &StreamDto) {
        let bitrate = if dto.bitrate > 0 {
            format!("{} kbps", dto.bitrate)
        } else {
            "bitrate n/a".to_string()
        };
        println!(
            "[quality] track {track_id}: {} {} via {}",
            dto.codec, bitrate, dto.source
        );
    }

    pub async fn stream(&self, track_id: &str, quality: &str) -> Result<StreamDto, String> {
        let key = format!("{track_id}:{quality}");
        let store = STREAM_CACHE.get_or_init(|| std::sync::Mutex::new(Vec::new()));
        if let Ok(mut items) = store.lock() {
            items.retain(|(_, _, at)| at.elapsed().as_secs() < STREAM_TTL_SECS);
            if let Some((_, dto, _)) = items.iter().find(|(id, _, _)| id == &key) {
                println!("[quality] track {track_id}: ссылка из кэша ({})", dto.codec);
                return Ok(dto.clone());
            }
        }
        let dto = self.stream_fresh(track_id, quality).await?;
        if let Ok(mut items) = store.lock() {
            items.push((key, dto.clone(), std::time::Instant::now()));
            while items.len() > 24 {
                items.remove(0);
            }
        }
        Ok(dto)
    }

    async fn stream_fresh(
        &self,
        track_id: &str,
        quality: &str,
    ) -> Result<StreamDto, String> {
        if quality == "lossless" {
            self.report_entitlements().await;
            let mut fallback: Option<StreamDto> = None;
            for (attempt, (client, codecs)) in Self::probe_order().into_iter().enumerate() {
                match self
                    .stream_file_info(track_id, "lossless", codecs, client)
                    .await
                {
                    Ok(dto) => {
                        Self::report(track_id, &dto);
                        if !Self::lossy(&dto.codec) {
                            Self::remember_probe(client, codecs);
                            println!(
                                "[quality] track {track_id}: lossless получен с попытки {} - {} ({client}, codecs={codecs})",
                                attempt + 1,
                                dto.codec
                            );
                            return Ok(dto);
                        }
                        if fallback.is_none() {
                            fallback = Some(dto);
                        }
                    }
                    Err(err) => println!(
                        "[quality] track {track_id}: get-file-info ({client}, codecs={codecs}) failed: {err}"
                    ),
                }
            }
            if let Some(dto) = fallback {
                println!(
                    "[quality] track {track_id}: lossless для этого трека недоступен, играет {} {} kbps",
                    dto.codec, dto.bitrate
                );
                return Ok(dto);
            }
            match self.stream_legacy(track_id, quality, true).await {
                Ok(dto) => {
                    Self::report(track_id, &dto);
                    return Ok(dto);
                }
                Err(err) => {
                    println!("[quality] track {track_id}: streaming download-info failed: {err}")
                }
            }
        }
        let dto = self.stream_legacy(track_id, quality, false).await?;
        Self::report(track_id, &dto);
        Ok(dto)
    }

    fn probe_order() -> Vec<(&'static str, &'static str)> {
        use std::sync::atomic::Ordering;
        let mut all: Vec<(&'static str, &'static str)> = Vec::new();
        for client in MUSIC_CLIENTS {
            for codecs in PROBE_CODECS {
                all.push((client, codecs));
            }
        }
        let best = BEST_PROBE.load(Ordering::Relaxed);
        if best < all.len() {
            let winner = all.remove(best);
            all.insert(0, winner);
        }
        all
    }

    fn remember_probe(client: &str, codecs: &str) {
        use std::sync::atomic::Ordering;
        let mut index = 0usize;
        for c in MUSIC_CLIENTS {
            for k in PROBE_CODECS {
                if c == client && k == codecs {
                    BEST_PROBE.store(index, Ordering::Relaxed);
                    return;
                }
                index += 1;
            }
        }
    }

    async fn report_entitlements(&self) {
        use std::sync::atomic::{AtomicBool, Ordering};
        static DONE: AtomicBool = AtomicBool::new(false);
        if DONE.swap(true, Ordering::Relaxed) {
            return;
        }
        let Ok(result) = self.get_result("/account/status").await else {
            return;
        };
        for key in ["plus", "subscription", "permissions"] {
            if let Some(value) = result.get(key) {
                let dump: String = value.to_string().chars().take(400).collect();
                println!("[quality] account {key}: {dump}");
            }
        }
    }

    async fn stream_legacy(
        &self,
        track_id: &str,
        quality: &str,
        streaming: bool,
    ) -> Result<StreamDto, String> {
        let suffix = if streaming { "?can_use_streaming=true" } else { "" };
        let result = self
            .get_result(&format!("/tracks/{track_id}/download-info{suffix}"))
            .await?;
        let infos: Vec<YDownloadInfo> =
            serde_json::from_value(result).map_err(|e| e.to_string())?;
        if infos.is_empty() {
            return Err("Нет доступных потоков".to_string());
        }

        let rank = |i: &YDownloadInfo| -> i64 {
            let br = i.bitrate_in_kbps.unwrap_or(0);
            let bonus = if !Self::lossy(&i.codec) {
                3000
            } else if i.codec.contains("aac") {
                1000
            } else {
                0
            };
            br + bonus
        };

        let best = match quality {
            "low" => infos
                .iter()
                .filter(|i| i.bitrate_in_kbps.unwrap_or(0) <= 128)
                .max_by_key(|i| i.bitrate_in_kbps.unwrap_or(0))
                .or_else(|| infos.iter().min_by_key(|i| rank(i))),
            "normal" => infos
                .iter()
                .filter(|i| i.codec != "flac" && i.bitrate_in_kbps.unwrap_or(0) <= 256)
                .max_by_key(|i| rank(i))
                .or_else(|| infos.iter().min_by_key(|i| rank(i))),
            "high" => infos
                .iter()
                .filter(|i| i.codec != "flac")
                .max_by_key(|i| rank(i))
                .or_else(|| infos.iter().max_by_key(|i| rank(i))),
            _ => infos.iter().max_by_key(|i| rank(i)),
        }

        .ok_or_else(|| "Нет доступных потоков".to_string())?;

        let xml = self
            .http
            .get(&best.download_info_url)
            .header("Authorization", self.auth())
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let host = between(&xml, "<host>", "</host>").ok_or("Некорректный ответ CDN")?;
        let path = between(&xml, "<path>", "</path>").ok_or("Некорректный ответ CDN")?;
        let ts = between(&xml, "<ts>", "</ts>").ok_or("Некорректный ответ CDN")?;
        let s = between(&xml, "<s>", "</s>").ok_or("Некорректный ответ CDN")?;

        let raw = format!("{SIGN_SALT}{}{}", &path[1..], s);
        let mut hasher = Md5::new();
        hasher.update(raw.as_bytes());
        let sign = hex::encode(hasher.finalize());
        let url = format!("{}://{host}/get-mp3/{sign}/{ts}{path}", "https");

        Ok(StreamDto {
            track_id: track_id.to_string(),
            url,
            codec: best.codec.clone(),
            bitrate: best.bitrate_in_kbps.unwrap_or(0),
            source: if streaming {
                "download-info+streaming".to_string()
            } else {
                "download-info".to_string()
            },
        })
    }

    pub async fn download_bytes(&self, url: &str) -> Result<Vec<u8>, String> {
        let res = self
            .http
            .get(url)
            .header("X-Yandex-Music-Client", MUSIC_CLIENT)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("CDN ответил {}", res.status().as_u16()));
        }
        let bytes = res.bytes().await.map_err(|e| e.to_string())?;
        Ok(bytes.to_vec())
    }

    async fn lyrics_meta(
        &self,
        track_id: &str,
        ts: u64,
        sign: &str,
        format: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!(
            "{BASE}/tracks/{track_id}/lyrics?format={format}&timeStamp={ts}&sign={}",
            urlencode(sign)
        );
        let resp = self
            .http
            .get(url)
            .timeout(std::time::Duration::from_secs(7))
            .header("Authorization", self.auth())
            .header("X-Yandex-Music-Client", MUSIC_CLIENT)
            .header("Accept", "application/json")
            .header("Accept-Language", "ru")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
        if status.as_u16() == 403 {
            return Err("Текст песни доступен только с подпиской Плюс".to_string());
        }
        if !status.is_success() {
            return Err(format!("Текст недоступен: {}", status.as_u16()));
        }
        Ok(body
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }

    async fn lyrics_supplement(&self, track_id: &str) -> Result<(String, Vec<String>), String> {
        let url = format!("{BASE}/tracks/{track_id}/supplement");
        let resp = self
            .http
            .get(url)
            .timeout(std::time::Duration::from_secs(7))
            .header("Authorization", self.auth())
            .header("X-Yandex-Music-Client", MUSIC_CLIENT)
            .header("Accept", "application/json")
            .header("Accept-Language", "ru")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("Текст недоступен: {}", resp.status().as_u16()));
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let node = body
            .get("result")
            .and_then(|r| r.get("lyrics"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let text = node
            .get("fullLyrics")
            .or_else(|| node.get("lyrics"))
            .or_else(|| node.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if text.trim().is_empty() {
            return Err("Для этого трека текста нет".to_string());
        }
        let writers = node
            .get("textAuthor")
            .and_then(|v| v.as_str())
            .map(|s| {
                s.split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        Ok((text, writers))
    }

    async fn lyrics_external(&self, track_id: &str) -> Result<(String, Vec<String>), String> {
        let track = self.track(track_id).await?;
        let artist = track
            .artists
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_default();
        if artist.is_empty() || track.title.is_empty() {
            return Err("Нет данных о треке для поиска текста".to_string());
        }
        let duration = (track.duration_ms.unwrap_or(0) / 1000).max(0);

        let get_url = format!(
            "{}://lrclib.net/api/get?artist_name={}&track_name={}&duration={}",
            "https",
            urlencode(&artist),
            urlencode(&track.title),
            duration
        );
        let mut body = self.lrclib_json(&get_url).await;

        if body.is_none() {
            let search_url = format!(
                "{}://lrclib.net/api/search?artist_name={}&track_name={}",
                "https",
                urlencode(&artist),
                urlencode(&track.title)
            );
            if let Some(list) = self.lrclib_json(&search_url).await {
                if let Some(items) = list.as_array() {
                    body = items
                        .iter()
                        .find(|i| {
                            i.get("syncedLyrics")
                                .and_then(|v| v.as_str())
                                .map(|s| !s.trim().is_empty())
                                .unwrap_or(false)
                        })
                        .or_else(|| items.first())
                        .cloned();
                }
            }
        }

        let body = body.ok_or_else(|| "Текста для этого трека нет".to_string())?;
        let text = body
            .get("syncedLyrics")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .or_else(|| {
                body.get("plainLyrics")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.trim().is_empty())
            })
            .ok_or_else(|| "Для этого трека текста нет".to_string())?
            .to_string();

        Ok((text, Vec::new()))
    }

    async fn lrclib_json(&self, url: &str) -> Option<serde_json::Value> {
        let resp = self
            .http
            .get(url)
            .timeout(std::time::Duration::from_secs(6))
            .header("Accept", "application/json")
            .header("User-Agent", "Mashiro (https://github.com/mashiro/mashiro)")
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        resp.json::<serde_json::Value>().await.ok()
    }

    async fn lyrics_lrclib(&self, track_id: &str) -> Result<(String, Vec<String>), String> {
        let track = self.track(track_id).await?;
        let artist = track
            .artists
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_default();
        let album = track.album_title.clone().unwrap_or_default();
        let duration = (track.duration_ms.unwrap_or(0) / 1000).max(0);

        let pick = |body: &serde_json::Value| -> Option<String> {
            let synced = body
                .get("syncedLyrics")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            if synced.is_some() {
                return synced;
            }
            body.get("plainLyrics")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        };

        let exact = format!(
            "{}://lrclib.net/api/get?artist_name={}&track_name={}&album_name={}&duration={}",
            "https",
            urlencode(&artist),
            urlencode(&track.title),
            urlencode(&album),
            duration
        );
        if let Ok(response) = self
            .http
            .get(exact)
            .header("Accept", "application/json")
            .header("User-Agent", "Mashiro (https://github.com/mashiro)")
            .send()
            .await
        {
            if response.status().is_success() {
                if let Ok(body) = response.json::<serde_json::Value>().await {
                    if let Some(text) = pick(&body) {
                        return Ok((text, Vec::new()));
                    }
                }
            }
        }

        let search = format!(
            "{}://lrclib.net/api/search?artist_name={}&track_name={}",
            "https",
            urlencode(&artist),
            urlencode(&track.title)
        );
        let response = self
            .http
            .get(search)
            .header("Accept", "application/json")
            .header("User-Agent", "Mashiro (https://github.com/mashiro)")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            return Err("LRCLIB не нашёл текст".to_string());
        }
        let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let items = body
            .as_array()
            .ok_or_else(|| "LRCLIB не нашёл текст".to_string())?;

        let mut best: Option<(i64, bool, String)> = None;
        for item in items {
            let Some(text) = pick(item) else { continue };
            let synced = item
                .get("syncedLyrics")
                .and_then(|v| v.as_str())
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
            let item_duration = item
                .get("duration")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
                .round() as i64;
            let delta = (item_duration - duration as i64).abs();
            let better = match &best {
                None => true,
                Some((best_delta, best_synced, _)) => {
                    (synced && !*best_synced) || (synced == *best_synced && delta < *best_delta)
                }
            };
            if better {
                best = Some((delta, synced, text));
            }
        }

        match best {
            Some((_, _, text)) => Ok((text, Vec::new())),
            None => Err("LRCLIB не нашёл текст".to_string()),
        }
    }

    async fn lyrics_textyl(&self, track_id: &str) -> Result<(String, Vec<String>), String> {
        let track = self.track(track_id).await?;
        let artist = track.artists.first().map(|a| a.name.clone()).unwrap_or_default();
        let query = urlencode(&format!("{} {}", artist, track.title));
        let url = format!("{}://api.textyl.co/api/lyrics?q={query}", "https");
        let response = self.http.get(url).timeout(std::time::Duration::from_secs(6)).header("Accept", "application/json").send().await.map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            return Err("Textyl не нашёл текст".to_string());
        }
        let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let items = body.as_array().ok_or_else(|| "Textyl не нашёл текст".to_string())?;
        let mut text = String::new();
        for item in items {
            let line = item.get("lyrics").and_then(|v| v.as_str()).unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let seconds = item.get("seconds").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0);
            let minutes = (seconds / 60.0).floor() as i64;
            let rest = seconds - minutes as f64 * 60.0;
            text.push_str(&format!("[{minutes:02}:{rest:05.2}]{line}\n"));
        }
        if text.trim().is_empty() {
            return Err("Textyl не нашёл текст".to_string());
        }
        Ok((text, Vec::new()))
    }

    async fn lyrics_ovh(&self, track_id: &str) -> Result<(String, Vec<String>), String> {
        let track = self.track(track_id).await?;
        let artist = track.artists.first().map(|a| a.name.clone()).unwrap_or_default();
        let url = format!(
            "{}://api.lyrics.ovh/v1/{}/{}",
            "https",
            urlencode(&artist),
            urlencode(&track.title)
        );
        let response = self.http.get(url).timeout(std::time::Duration::from_secs(6)).header("Accept", "application/json").send().await.map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            return Err("Lyrics.ovh не нашёл текст".to_string());
        }
        let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let text = body.get("lyrics").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        if text.is_empty() {
            return Err("Lyrics.ovh не нашёл текст".to_string());
        }
        Ok((text, Vec::new()))
    }

    pub async fn lyrics(&self, track_id: &str) -> Result<LyricsDto, String> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        let mut mac = Hmac::<Sha256>::new_from_slice(LYRICS_SECRET.as_bytes())
            .map_err(|e| e.to_string())?;
        mac.update(format!("{track_id}{ts}").as_bytes());
        let sign = BASE64
            .encode(mac.finalize().into_bytes())
            .trim_end_matches('=')
            .to_string();

        let result = match self.lyrics_meta(track_id, ts, &sign, "LRC").await {
            Ok(value) => value,
            Err(_) => {
                let mut plain_fallback: Option<LyricsDto> = None;

                if let Ok((text, writers)) = self.lyrics_lrclib(track_id).await {
                    if let Ok(lyrics) = lyrics_from_text(track_id, &text, writers) {
                        if lyrics.synced {
                            return Ok(lyrics);
                        }
                        plain_fallback = Some(lyrics);
                    }
                }

                if let Ok((text, writers)) = self.lyrics_external(track_id).await {
                    let lyrics = lyrics_from_text(track_id, &text, writers)?;
                    if lyrics.synced {
                        return Ok(lyrics);
                    }
                    plain_fallback = Some(lyrics);
                }

                if let Ok((text, writers)) = self.lyrics_textyl(track_id).await {
                    let lyrics = lyrics_from_text(track_id, &text, writers)?;
                    if lyrics.synced {
                        return Ok(lyrics);
                    }
                    if plain_fallback.is_none() {
                        plain_fallback = Some(lyrics);
                    }
                }

                match self.lyrics_meta(track_id, ts, &sign, "TEXT").await {
                    Ok(value) => value,
                    Err(_) => {
                        if let Ok((text, writers)) = self.lyrics_supplement(track_id).await {
                            return lyrics_from_text(track_id, &text, writers);
                        }
                        if let Some(lyrics) = plain_fallback {
                            return Ok(lyrics);
                        }
                        let (text, writers) = self.lyrics_ovh(track_id).await?;
                        return lyrics_from_text(track_id, &text, writers);
                    }
                }
            }
        };

        let inline = result
            .get("lyrics")
            .and_then(|l| {
                l.get("fullLyrics")
                    .or_else(|| l.get("lyrics"))
                    .or_else(|| l.get("text"))
            })
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let raw = match result.get("downloadUrl").and_then(|v| v.as_str()) {
            Some(download_url) => self
                .http
                .get(download_url)
                .send()
                .await
                .map_err(|e| e.to_string())?
                .text()
                .await
                .map_err(|e| e.to_string())?,
            None => inline.ok_or_else(|| "Для этого трека текста нет".to_string())?,
        };

        let writers = result
            .get("writers")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|w| w.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let lines = parse_lrc(&raw);
        if lines.is_empty() {
            return Err("Для этого трека текста нет".to_string());
        }
        let synced = lines.iter().any(|l| l.time_ms > 0);

        Ok(LyricsDto {
            track_id: track_id.to_string(),
            synced,
            lines,
            writers,
        })
    }
}
