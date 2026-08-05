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
