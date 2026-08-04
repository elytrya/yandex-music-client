use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ArtistDto {
    pub id: String,
    pub name: String,
    pub cover_url: Option<String>,
}

#[derive(Serialize)]
pub struct TrackDto {
    pub id: String,
    pub title: String,
    pub artists: Vec<ArtistDto>,
    pub album_id: Option<String>,
    pub album_title: Option<String>,
    pub cover_url: Option<String>,
    pub duration_ms: Option<i64>,
    pub available: bool,
}

#[derive(Serialize, Clone, Default)]
pub struct TrackInfoDto {
    pub track_id: String,
    pub title: String,
    pub version: Option<String>,
    pub label: Option<String>,
    pub source: Option<String>,
    pub artists: Vec<String>,
    pub composers: Vec<String>,
    pub lyricists: Vec<String>,
    pub album: Option<String>,
    pub year: Option<i64>,
    pub release_date: Option<String>,
    pub genre: Option<String>,
    pub duration_ms: Option<i64>,
    pub explicit: bool,
}

#[derive(Serialize)]
pub struct ProfileDto {
    pub uid: Option<i64>,
    pub login: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub has_plus: bool,
}

#[derive(Serialize)]
pub struct PlaylistDto {
    pub kind: i64,
    pub title: String,
    pub track_count: i64,
    pub cover_url: Option<String>,
    pub owner_uid: Option<i64>,
}

#[derive(Serialize, Clone)]
pub struct LyricsLineDto {
    pub time_ms: i64,
    pub text: String,
}

#[derive(Serialize, Clone)]
pub struct LyricsDto {
    pub track_id: String,
    pub synced: bool,
    pub lines: Vec<LyricsLineDto>,
    pub writers: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct StreamDto {
    pub track_id: String,
    pub url: String,
    pub codec: String,
    pub bitrate: i64,
}

#[derive(Serialize)]
pub struct StationDto {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Serialize)]
pub struct WheelItemDto {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub color: Option<String>,
    pub station: Option<String>,
    pub artists: Vec<ArtistDto>,
}

#[derive(Serialize)]
pub struct AlbumBriefDto {
    pub id: String,
    pub title: String,
    pub cover_url: Option<String>,
    pub year: Option<i64>,
}

#[derive(Serialize)]
pub struct AlbumPageDto {
    pub id: String,
    pub title: String,
    pub cover_url: Option<String>,
    pub color: Option<String>,
    pub year: Option<i64>,
    pub genre: Option<String>,
    pub artists: Vec<ArtistDto>,
    pub tracks: Vec<TrackDto>,
}

#[derive(Serialize)]
pub struct ArtistLinkDto {
    pub title: String,
    pub href: String,
    pub kind: String,
    pub network: Option<String>,
}

#[derive(Serialize)]
pub struct ArtistPageDto {
    pub id: String,
    pub name: String,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub listeners: Option<i64>,
    pub likes: Option<i64>,
    pub tracks_count: Option<i64>,
    pub albums_count: Option<i64>,
    pub links: Vec<ArtistLinkDto>,
    pub genres: Vec<String>,
    pub tracks: Vec<TrackDto>,
    pub albums: Vec<AlbumBriefDto>,
    pub covers: Vec<String>,
}

#[derive(Serialize)]
pub struct SearchDto {
    pub tracks: Vec<TrackDto>,
    pub artists: Vec<ArtistDto>,
    pub albums: Vec<AlbumBriefDto>,
}

#[derive(Serialize)]
pub struct WaveResponse {
    pub station_id: String,
    pub batch_id: Option<String>,
    pub tracks: Vec<TrackDto>,
}

#[derive(Serialize)]
pub struct StationOptionDto {
    pub value: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct StationInfoDto {
    pub id: String,
    pub name: String,
    pub language: Option<String>,
    pub mood_energy: Option<String>,
    pub diversity: Option<String>,
    pub languages: Vec<StationOptionDto>,
    pub moods: Vec<StationOptionDto>,
    pub diversities: Vec<StationOptionDto>,
}
