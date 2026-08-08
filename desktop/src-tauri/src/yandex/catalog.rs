use super::*;

fn host_of(href: &str) -> String {
    let without_scheme = href
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(href);
    let host = without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .trim_start_matches("www.");
    host.to_string()
}

fn network_from_href(href: &str) -> Option<String> {
    let host = host_of(href).to_lowercase();
    let known = [
        ("vk.com", "vk"),
        ("vkontakte.ru", "vk"),
        ("t.me", "telegram"),
        ("telegram.me", "telegram"),
        ("instagram.com", "instagram"),
        ("youtube.com", "youtube"),
        ("youtu.be", "youtube"),
        ("twitter.com", "twitter"),
        ("x.com", "twitter"),
        ("tiktok.com", "tiktok"),
        ("facebook.com", "facebook"),
        ("soundcloud.com", "soundcloud"),
        ("spotify.com", "spotify"),
        ("apple.com", "apple"),
        ("bandcamp.com", "bandcamp"),
        ("twitch.tv", "twitch"),
        ("dzen.ru", "dzen"),
        ("ok.ru", "ok"),
    ];
    known
        .iter()
        .find(|(domain, _)| host == *domain || host.ends_with(&format!(".{domain}")))
        .map(|(_, name)| (*name).to_string())
}

fn network_title(network: &str) -> String {
    match network {
        "vk" => "ВКонтакте",
        "telegram" => "Telegram",
        "instagram" => "Instagram",
        "youtube" => "YouTube",
        "twitter" => "X",
        "tiktok" => "TikTok",
        "facebook" => "Facebook",
        "soundcloud" => "SoundCloud",
        "spotify" => "Spotify",
        "apple" => "Apple Music",
        "bandcamp" => "Bandcamp",
        "twitch" => "Twitch",
        "dzen" => "Дзен",
        "ok" => "Одноклассники",
        other => other,
    }
    .to_string()
}

impl Yandex {
    pub async fn artist_tracks(&self, artist_id: &str) -> Result<Vec<TrackDto>, String> {
        let mut tracks = Vec::new();
        for page in 0..10 {
            let result = self
                .get_result(&format!(
                    "/artists/{artist_id}/tracks?page={page}&page-size=100"
                ))
                .await?;
            let items = result
                .get("tracks")
                .and_then(|value| value.as_array())
                .or_else(|| result.as_array())
                .cloned()
                .unwrap_or_default();
            let count = items.len();
            for raw in items {
                if let Ok(track) = serde_json::from_value::<YTrack>(raw) {
                    tracks.push(map_track(&track));
                }
            }
            if count < 100 {
                break;
            }
        }
        Ok(tracks)
    }

    pub async fn artist(&self, artist_id: &str) -> Result<ArtistPageDto, String> {
        let result = self
            .get_result(&format!("/artists/{artist_id}/brief-info"))
            .await?;

        let artist = result.get("artist").cloned().unwrap_or_default();
        let name = artist
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Артист")
            .to_string();
        let cover = str_at(&artist, &["cover", "uri"]);
        let description = str_at(&artist, &["description", "text"]);
        let counts = |key: &str| {
            artist
                .get("counts")
                .and_then(|c| c.get(key))
                .and_then(|v| v.as_i64())
        };
        let listeners = result
            .get("stats")
            .and_then(|s| s.get("lastMonthListeners"))
            .and_then(|v| v.as_i64())
            .or_else(|| {
                artist
                    .get("ratings")
                    .and_then(|r| r.get("month"))
                    .and_then(|v| v.as_i64())
            });
        let likes = artist
            .get("likesCount")
            .and_then(|v| v.as_i64())
            .or_else(|| {
                result
                    .get("stats")
                    .and_then(|s| s.get("likesCount"))
                    .and_then(|v| v.as_i64())
            });
        let tracks_count = counts("tracks");
        let albums_count = counts("directAlbums");
        let links = artist
            .get("links")
            .and_then(|l| l.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|raw| {
                        let href = raw.get("href").and_then(|v| v.as_str())?;
                        let network = raw
                            .get("socialNetwork")
                            .and_then(|v| v.as_str())
                            .map(|s| s.trim().to_lowercase())
                            .filter(|s| !s.is_empty())
                            .or_else(|| network_from_href(href));
                        let kind = raw
                            .get("type")
                            .and_then(|v| v.as_str())
                            .map(|s| s.trim().to_lowercase())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| {
                                if network.is_some() {
                                    "social".to_string()
                                } else {
                                    "official".to_string()
                                }
                            });
                        let raw_title = raw
                            .get("title")
                            .and_then(|v| v.as_str())
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty());
                        let title = raw_title
                            .map(|s| s.to_string())
                            .or_else(|| network.as_deref().map(network_title))
                            .unwrap_or_else(|| host_of(href));
                        Some(ArtistLinkDto {
                            title,
                            href: href.to_string(),
                            kind,
                            network,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let genres = artist
            .get("genres")
            .and_then(|g| g.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut covers: Vec<String> = result
            .get("allCovers")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.get("uri").and_then(|u| u.as_str()))
                    .filter_map(|u| img(Some(u), "1000x1000"))
                    .collect()
            })
            .unwrap_or_default();
        if let Some(main) = img(cover.as_deref(), "1000x1000") {
            if !covers.iter().any(|c| c == &main) {
                covers.insert(0, main);
            }
        }

        let mut tracks = Vec::new();
        if let Some(arr) = result.get("popularTracks").and_then(|t| t.as_array()) {
            for raw in arr.iter().take(30) {
                if let Ok(t) = serde_json::from_value::<YTrack>(raw.clone()) {
                    tracks.push(map_track(&t));
                }
            }
        }

        let mut albums = Vec::new();
        for key in ["albums", "alsoAlbums"] {
            if let Some(arr) = result.get(key).and_then(|a| a.as_array()) {
                for raw in arr {
                    if let Ok(a) = serde_json::from_value::<YAlbum>(raw.clone()) {
                        albums.push(map_album_brief(&a));
                    }
                }
            }
        }

        Ok(ArtistPageDto {
            id: artist_id.to_string(),
            name,
            cover_url: img(cover.as_deref(), "400x400"),
            description,
            listeners,
            likes,
            tracks_count,
            albums_count,
            links,
            genres,
            tracks,
            albums,
            covers,
        })
    }

    pub async fn album(&self, album_id: &str) -> Result<AlbumPageDto, String> {
        let result = self
            .get_result(&format!("/albums/{album_id}/with-tracks"))
            .await?;

        let title = result
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Альбом")
            .to_string();
        let cover = result
            .get("coverUri")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| str_at(&result, &["cover", "uri"]));
        let color = str_at(&result, &["cover", "color"]);

        let artists = result
            .get("artists")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|raw| serde_json::from_value::<YArtist>(raw.clone()).ok())
                    .map(|a| map_artist(&a))
                    .collect()
            })
            .unwrap_or_default();

        let mut tracks = Vec::new();
        if let Some(volumes) = result.get("volumes").and_then(|v| v.as_array()) {
            for volume in volumes {
                if let Some(arr) = volume.as_array() {
                    for raw in arr {
                        if let Ok(t) = serde_json::from_value::<YTrack>(raw.clone()) {
                            tracks.push(map_track(&t));
                        }
                    }
                }
            }
        }

        Ok(AlbumPageDto {
            id: album_id.to_string(),
            title,
            cover_url: img(cover.as_deref(), "400x400"),
            color,
            year: result.get("year").and_then(|v| v.as_i64()),
            genre: result
                .get("genre")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            artists,
            tracks,
        })
    }

    pub async fn search(&self, text: &str) -> Result<SearchDto, String> {
        let encoded = urlencode(text);
        let result = self
            .get_result(&format!(
                "/search?text={encoded}&type=all&page=0&nocorrect=false"
            ))
            .await?;

        let mut tracks = Vec::new();
        if let Some(arr) = result
            .get("tracks")
            .and_then(|t| t.get("results"))
            .and_then(|r| r.as_array())
        {
            for item in arr {
                if let Ok(t) = serde_json::from_value::<YTrack>(item.clone()) {
                    tracks.push(map_track(&t));
                }
            }
        }

        let mut artists = Vec::new();
        if let Some(arr) = result
            .get("artists")
            .and_then(|t| t.get("results"))
            .and_then(|r| r.as_array())
        {
            for item in arr.iter().take(12) {
                if let Ok(a) = serde_json::from_value::<YArtist>(item.clone()) {
                    artists.push(map_artist(&a));
                }
            }
        }

        let mut albums = Vec::new();
        if let Some(arr) = result
            .get("albums")
            .and_then(|t| t.get("results"))
            .and_then(|r| r.as_array())
        {
            for item in arr.iter().take(12) {
                if let Ok(a) = serde_json::from_value::<YAlbum>(item.clone()) {
                    albums.push(map_album_brief(&a));
                }
            }
        }

        Ok(SearchDto {
            tracks,
            artists,
            albums,
        })
    }

    pub async fn search_suggest(&self, part: &str) -> Result<Vec<String>, String> {
        let part = part.trim();
        if part.is_empty() {
            return Ok(Vec::new());
        }
        let result = self
            .get_result(&format!("/search/suggest?part={}", urlencode(part)))
            .await?;

        let mut out: Vec<String> = Vec::new();
        if let Some(best) = result
            .get("best")
            .and_then(|b| b.get("text"))
            .and_then(|v| v.as_str())
        {
            out.push(best.to_string());
        }
        if let Some(arr) = result.get("suggestions").and_then(|v| v.as_array()) {
            for item in arr {
                let text = match item {
                    serde_json::Value::String(value) => Some(value.clone()),
                    other => other
                        .get("text")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                };
                if let Some(text) = text {
                    out.push(text);
                }
            }
        }

        let mut seen = std::collections::HashSet::new();
        out.retain(|item| !item.trim().is_empty() && seen.insert(item.to_lowercase()));
        out.truncate(10);
        Ok(out)
    }

    pub async fn similar_tracks(&self, track_id: &str) -> Result<Vec<TrackDto>, String> {
        let result = self
            .get_result(&format!("/tracks/{track_id}/similar"))
            .await?;
        let items = result
            .get("similarTracks")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(items
            .into_iter()
            .filter_map(|raw| serde_json::from_value::<YTrack>(raw).ok())
            .map(|track| map_track(&track))
            .collect())
    }

    pub async fn track_info(&self, track_id: &str) -> Result<TrackInfoDto, String> {
        let result = self
            .get_result(&format!("/tracks?track-ids={track_id}"))
            .await?;
        let item = result
            .get(0)
            .cloned()
            .ok_or_else(|| "Трек не найден".to_string())?;

        let text = |value: Option<&serde_json::Value>| -> Option<String> {
            value
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        };

        let names = |value: Option<&serde_json::Value>| -> Vec<String> {
            value
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| match x {
                            serde_json::Value::String(s) => Some(s.trim().to_string()),
                            other => other
                                .get("name")
                                .and_then(|n| n.as_str())
                                .map(|s| s.trim().to_string()),
                        })
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default()
        };

        let split = |value: Option<String>| -> Vec<String> {
            value
                .map(|s| {
                    s.split(|c| c == ',' || c == ';')
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default()
        };

        let meta = item.get("metaData");
        let album = item.get("albums").and_then(|a| a.get(0)).cloned();

        let mut release_date = text(item.get("releaseDate")).or_else(|| {
            album
                .as_ref()
                .and_then(|a| text(a.get("releaseDate")))
                .or_else(|| album.as_ref().and_then(|a| text(a.get("originalReleaseDate"))))
        });

        let mut source = major_name(item.get("major"))
            .or_else(|| major_name(album.as_ref().and_then(|a| a.get("major"))));

        let mut labels = names(item.get("labels"));
        if labels.is_empty() {
            labels = names(album.as_ref().and_then(|a| a.get("labels")));
        }
        if labels.is_empty() || release_date.is_none() || source.is_none() {
            let album_id = album
                .as_ref()
                .and_then(|a| a.get("id"))
                .and_then(|id| match id {
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    serde_json::Value::String(s) => Some(s.clone()),
                    _ => None,
                });
            if let Some(album_id) = album_id {
                if let Ok(full) = self.get_result(&format!("/albums/{album_id}")).await {
                    if labels.is_empty() {
                        labels = names(full.get("labels"));
                    }
                    if release_date.is_none() {
                        release_date = text(full.get("releaseDate"))
                            .or_else(|| text(full.get("originalReleaseDate")));
                    }
                    if source.is_none() {
                        source = major_name(full.get("major"))
                            .or_else(|| distributor(full.get("labels")));
                    }
                }
            }
        }

        if source.is_none() {
            source = distributor(item.get("labels"))
                .or_else(|| distributor(album.as_ref().and_then(|a| a.get("labels"))));
        }

        Ok(TrackInfoDto {
            track_id: track_id.to_string(),
            title: text(item.get("title")).unwrap_or_default(),
            version: text(item.get("version")),
            label: if labels.is_empty() {
                None
            } else {
                Some(labels.join(", "))
            },
            source,
            artists: names(item.get("artists")),
            composers: split(text(meta.and_then(|m| m.get("composer")))),
            lyricists: split(text(meta.and_then(|m| m.get("lyricist")))),
            album: album
                .as_ref()
                .and_then(|a| text(a.get("title")))
                .or_else(|| text(meta.and_then(|m| m.get("album")))),
            year: album
                .as_ref()
                .and_then(|a| a.get("year"))
                .and_then(|y| y.as_i64()),
            release_date,
            genre: album.as_ref().and_then(|a| text(a.get("genre"))),
            duration_ms: item.get("durationMs").and_then(|d| d.as_i64()),
            explicit: text(item.get("contentWarning")).is_some(),
        })
    }

    pub async fn track(&self, track_id: &str) -> Result<TrackDto, String> {
        let result = self
            .get_result(&format!("/tracks?track-ids={track_id}"))
            .await?;
        let list: Vec<YTrack> = serde_json::from_value(result).map_err(|e| e.to_string())?;
        list.first()
            .map(map_track)
            .ok_or_else(|| "Трек не найден".to_string())
    }
}

fn major_name(value: Option<&serde_json::Value>) -> Option<String> {
    let raw = value
        .and_then(|m| m.get("name"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())?;

    let upper = raw.to_uppercase();
    if matches!(
        upper.as_str(),
        "UNKNOWN" | "NO_MAJOR" | "NOMAJOR" | "NONE" | "OTHER" | "-"
    ) {
        return None;
    }

    let pretty = match upper.as_str() {
        "WARNER" => "Warner Music",
        "UNIVERSAL" => "Universal Music",
        "SONY" => "Sony Music",
        "BELIEVE" => "Believe",
        "MERLIN" => "Merlin",
        "ORCHARD" => "The Orchard",
        "FRESHTUNES" => "FreshTunes",
        "DISTROKID" => "DistroKid",
        "NATIONAL" => "Национальное музыкальное издательство",
        _ => raw,
    };
    Some(pretty.to_string())
}

fn distributor(value: Option<&serde_json::Value>) -> Option<String> {
    let items = value.and_then(|v| v.as_array())?;
    let names: Vec<String> = items
        .iter()
        .filter(|item| {
            item.get("type")
                .and_then(|t| t.as_str())
                .map(|t| t.eq_ignore_ascii_case("distributor"))
                .unwrap_or(false)
        })
        .filter_map(|item| item.get("name").and_then(|n| n.as_str()))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if names.is_empty() {
        None
    } else {
        Some(names.join(", "))
    }
}
