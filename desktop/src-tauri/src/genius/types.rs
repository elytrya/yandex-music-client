
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct GeniusPerson {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub image: Option<String>,
    pub role: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct GeniusAuthor {
    pub name: String,
    pub url: String,
    pub image: Option<String>,
    pub iq: Option<i64>,
    pub verified: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct GeniusQuote {
    pub id: u64,
    pub fragment: String,
    pub text: String,
    pub url: String,
    pub votes: i64,
    pub verified: bool,
    pub pinned: bool,
    pub state: String,
    pub comments: u64,
    pub authors: Vec<GeniusAuthor>,
}

#[derive(Serialize, Clone, Debug)]
pub struct GeniusMedia {
    pub provider: String,
    pub kind: String,
    pub url: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct GeniusRelation {
    pub kind: String,
    pub songs: Vec<GeniusHit>,
}

#[derive(Serialize, Clone, Debug)]
pub struct GeniusAlbum {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub art: Option<String>,
    pub artist: Option<String>,
    pub release_date: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct GeniusHit {
    pub id: u64,
    pub title: String,
    pub full_title: String,
    pub artist: String,
    pub url: String,
    pub art: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct GeniusSong {
    pub id: u64,
    pub title: String,
    pub full_title: String,
    pub url: String,
    pub artist: String,
    pub artist_id: u64,
    pub artist_url: String,
    pub art: Option<String>,
    pub album: Option<String>,
    pub album_url: Option<String>,
    pub release_date: Option<String>,
    pub pageviews: Option<u64>,
    pub description: Option<String>,
    pub credits: Vec<GeniusPerson>,
    pub lyrics: Vec<String>,
    pub quotes: Vec<GeniusQuote>,
    pub album_info: Option<GeniusAlbum>,
    pub contributors: Option<u64>,
    pub concurrents: Option<u64>,
    pub annotation_count: Option<u64>,
    pub hot: bool,
    pub recording_location: Option<String>,
    pub language: Option<String>,
    pub apple_music_id: Option<String>,
    pub media: Vec<GeniusMedia>,
    pub relations: Vec<GeniusRelation>,
    pub tags: Vec<String>,
    pub verified_by: Vec<GeniusPerson>,
    pub quotes_error: Option<String>,
    pub lyrics_error: Option<String>,
    pub quotes_source: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct GeniusSocial {
    pub kind: String,
    pub handle: String,
    pub url: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct GeniusArtist {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub image: Option<String>,
    pub header: Option<String>,
    pub description: Option<String>,
    pub followers: Option<u64>,
    pub alternate_names: Vec<String>,
    pub socials: Vec<GeniusSocial>,
    pub songs: Vec<GeniusHit>,
    pub iq: Option<i64>,
    pub verified: bool,
    pub instagram: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct GeniusPersonHit {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub image: Option<String>,
    pub verified: bool,
}
