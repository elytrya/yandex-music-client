export interface Artist {
  id: string;
  name: string;
  cover_url?: string | null;
}

export interface AiArtistResult {
  id: string;
  ai: boolean;
  ai_tracks: number;
  total_tracks: number;
}

export interface AiTrackResult {
  id: string;
  ai: boolean;
  score: number | null;
}

export interface Track {
  id: string;
  title: string;
  artists: Artist[];
  album_id: string | null;
  album_title: string | null;
  cover_url: string | null;
  duration_ms: number | null;
  available: boolean;
}

export interface Profile {
  uid: number | null;
  login: string | null;
  display_name: string | null;
  avatar_url: string | null;
  has_plus: boolean;
}

export interface Playlist {
  kind: number;
  title: string;
  track_count: number;
  cover_url: string | null;
  owner_uid: number | null;
}

export interface PlaylistMembership {
  kind: number;
  track_ids: string[];
}

export interface Stream {
  track_id: string;
  url: string;
  codec: string;
  bitrate: number;
}

export interface LyricsLine {
  time_ms: number;
  text: string;
}

export interface Lyrics {
  track_id: string;
  synced: boolean;
  lines: LyricsLine[];
  writers: string[];
}

export interface LrcLyrics {
  id: number;
  title: string;
  artist: string;
  album: string | null;
  duration: number | null;
  instrumental: boolean;
  synced: boolean;
  lines: LyricsLine[];
  source: string;
}

export type Quality = "low" | "normal" | "high" | "lossless";

export type RepeatMode = "off" | "all" | "one";

export interface WaveResponse {
  station_id: string;
  batch_id: string | null;
  tracks: Track[];
}

export interface Station {
  id: string;
  name: string;
  color: string | null;
  icon_url: string | null;
}

export interface StationOption {
  value: string;
  name: string;
}

export interface StationInfo {
  id: string;
  name: string;
  language: string | null;
  mood_energy: string | null;
  diversity: string | null;
  languages: StationOption[];
  moods: StationOption[];
  diversities: StationOption[];
}

export interface WheelItem {
  id: string;
  kind: "wave" | "album" | "artist";
  name: string;
  description: string | null;
  cover_url: string | null;
  color: string | null;
  station: string | null;
  artists: Artist[];
}

export interface AlbumBrief {
  id: string;
  title: string;
  cover_url: string | null;
  year: number | null;
}

export interface AlbumPage {
  id: string;
  title: string;
  cover_url: string | null;
  color: string | null;
  year: number | null;
  genre: string | null;
  artists: Artist[];
  tracks: Track[];
}

export interface ArtistLink {
  title: string;
  href: string;
  kind: string;
  network: string | null;
}

export interface ArtistPage {
  id: string;
  name: string;
  cover_url: string | null;
  description: string | null;
  listeners: number | null;
  likes: number | null;
  tracks_count: number | null;
  albums_count: number | null;
  links: ArtistLink[];
  genres: string[];
  tracks: Track[];
  albums: AlbumBrief[];
  covers: string[];
}

export interface TrackInfo {
  track_id: string;
  title: string;
  version: string | null;
  label: string | null;
  source: string | null;
  artists: string[];
  composers: string[];
  lyricists: string[];
  album: string | null;
  year: number | null;
  release_date: string | null;
  genre: string | null;
  duration_ms: number | null;
  explicit: boolean;
}

export interface SearchResult {
  tracks: Track[];
  artists: Artist[];
  albums: AlbumBrief[];
}

export type FeedbackType =
  | "radioStarted"
  | "trackStarted"
  | "trackFinished"
  | "skip"
  | "like"
  | "dislike";

export interface FeedbackPayload {
  station_id?: string;
  type: FeedbackType;
  track_id?: string | null;
  batch_id?: string | null;
  total_played_seconds?: number | null;
}

export interface PresenceStatus {
  connected: boolean;
  applicationId: string | null;
  user?: string | null;
  lastError?: string | null;
}

export interface GeniusPerson {
  id: number;
  name: string;
  url: string;
  image: string | null;
  role: string;
}

export interface GeniusAuthor {
  name: string;
  url: string;
  image: string | null;
  iq: number | null;
  verified: boolean;
}

export interface GeniusQuote {
  id: number;
  fragment: string;
  text: string;
  url: string;
  votes: number;
  verified: boolean;
  pinned: boolean;
  state: string;
  comments: number;
  authors: GeniusAuthor[];
}

export interface GeniusMedia {
  provider: string;
  kind: string;
  url: string;
}

export interface GeniusAlbum {
  id: number;
  name: string;
  url: string;
  art: string | null;
  artist: string | null;
  release_date: string | null;
}

export interface GeniusHit {
  id: number;
  title: string;
  full_title: string;
  artist: string;
  url: string;
  art: string | null;
}

export interface GeniusSong {
  id: number;
  title: string;
  full_title: string;
  url: string;
  artist: string;
  artist_id: number;
  artist_url: string;
  art: string | null;
  album: string | null;
  album_url: string | null;
  release_date: string | null;
  pageviews: number | null;
  description: string | null;
  credits: GeniusPerson[];
  lyrics: string[];
  quotes: GeniusQuote[];
  album_info: GeniusAlbum | null;
  contributors: number | null;
  concurrents: number | null;
  annotation_count: number | null;
  hot: boolean;
  recording_location: string | null;
  language: string | null;
  apple_music_id: string | null;
  media: GeniusMedia[];
  relations: GeniusRelation[];
  tags: string[];
  verified_by: GeniusPerson[];
  quotes_error: string | null;
  lyrics_error: string | null;
  quotes_source: string;
}

export interface GeniusRelation {
  kind: string;
  songs: GeniusHit[];
}

export interface GeniusSocial {
  kind: string;
  handle: string;
  url: string;
}

export interface GeniusArtist {
  id: number;
  name: string;
  url: string;
  image: string | null;
  header: string | null;
  description: string | null;
  followers: number | null;
  alternate_names: string[];
  socials: GeniusSocial[];
  songs: GeniusHit[];
  iq: number | null;
  verified: boolean;
  instagram: string | null;
}

export interface GeniusPersonHit {
  id: number;
  name: string;
  url: string;
  image: string | null;
  verified: boolean;
}

export interface PresencePayload {
  enabled: boolean;
  applicationId: string;
  details: string;
  state: string;
  album: string;
  coverUrl: string | null;
  trackUrl: string;
  buttonLabel: string;
  startedAt: number | null;
  endsAt: number | null;
}
