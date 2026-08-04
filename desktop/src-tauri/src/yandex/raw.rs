use serde::Deserialize;

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YArtist {
    pub id: serde_json::Value,
    pub name: Option<String>,
    pub cover: Option<YCover>,
    pub og_image: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YAlbum {
    pub id: serde_json::Value,
    pub title: Option<String>,
    pub cover_uri: Option<String>,
    pub year: Option<i64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YTrack {
    pub id: serde_json::Value,
    pub title: Option<String>,
    pub artists: Vec<YArtist>,
    pub albums: Vec<YAlbum>,
    pub cover_uri: Option<String>,
    pub duration_ms: Option<i64>,
    pub available: Option<bool>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YTrackWrap {
    pub track: Option<YTrack>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YRotor {
    pub batch_id: Option<String>,
    pub sequence: Vec<YTrackWrap>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YAccount {
    pub uid: Option<i64>,
    pub login: Option<String>,
    pub display_name: Option<String>,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_id: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YPlus {
    pub has_plus: Option<bool>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YAccountStatus {
    pub account: YAccount,
    pub plus: Option<YPlus>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YCover {
    pub uri: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YOwner {
    pub uid: Option<i64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YPlaylist {
    pub kind: i64,
    pub title: Option<String>,
    pub track_count: Option<i64>,
    pub cover: Option<YCover>,
    pub og_image: Option<String>,
    pub owner: Option<YOwner>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YPlaylistFull {
    pub tracks: Vec<YTrackWrap>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct YDownloadInfo {
    pub codec: String,
    pub bitrate_in_kbps: Option<i64>,
    pub download_info_url: String,
}
