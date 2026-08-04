import { invoke } from "@tauri-apps/api/core";
import type {
  AiArtistResult,
  AiTrackResult,
  AlbumPage,
  ArtistPage,
  FeedbackPayload,
  Lyrics,
  Playlist,
  PresencePayload,
  PresenceStatus,
  Quality,
  Profile,
  SearchResult,
  Station,
  StationInfo,
  Stream,
  Track,
  TrackInfo,
  WaveResponse,
  WheelItem,
} from "./types";

const TOKEN_KEY = "mashiro.token";

const ENV_TOKEN =
  (import.meta.env.VITE_YM_TOKEN as string | undefined)?.trim() || null;

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY) || ENV_TOKEN;
}

export function setToken(token: string | null): void {
  if (token) localStorage.setItem(TOKEN_KEY, token);
  else localStorage.removeItem(TOKEN_KEY);
}

export class ApiError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ApiError";
  }
}

async function call<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw new ApiError(typeof e === "string" ? e : "Ошибка вызова ядра");
  }
}

export const api = {
  login: (token: string) => call<Profile>("auth_login", { token }),
  me: () => call<Profile>("auth_me"),
  logout: () => call<void>("auth_logout"),

  wave: (queue?: string, station?: string) => {
    const args: Record<string, unknown> = {};
    if (queue) args.queue = queue;
    if (station) args.station = station;
    return call<WaveResponse>("get_wave", args);
  },

  wheel: () => call<WheelItem[]>("get_wheel"),

  artist: (id: string) => call<ArtistPage>("get_artist", { id }),

  artistTracks: (id: string) => call<Track[]>("get_artist_tracks", { id }),

  aiCheckArtists: (ids: string[]) =>
    call<AiArtistResult[]>("ai_check_artists", { ids }),

  aiCheckTracks: (ids: string[]) =>
    call<AiTrackResult[]>("ai_check_tracks", { ids }),

  album: (id: string) => call<AlbumPage>("get_album", { id }),

  search: (text: string) => call<SearchResult>("search_tracks", { text }),

  likedTracks: () => call<Track[]>("get_liked_tracks"),

  likedIds: () => call<string[]>("get_liked_ids"),

  setLike: (id: string, remove: boolean) =>
    call<boolean>("set_like", { id, remove }),

  setDislike: (id: string, remove: boolean) =>
    call<boolean>("set_dislike", { id, remove }),

  playlistAdd: (kind: number | string, id: string, album: string, at = 0) =>
    call<boolean>("playlist_add_track", { kind: Number(kind), id, album, at }),

  playlistRemove: (
    kind: number | string,
    id: string,
    album: string,
    at: number,
  ) =>
    call<boolean>("playlist_remove_track", {
      kind: Number(kind),
      id,
      album,
      at,
    }),

  playlistMove: (
    kind: number | string,
    id: string,
    album: string,
    from: number,
    to: number,
  ) =>
    call<boolean>("playlist_move_track", {
      kind: Number(kind),
      id,
      album,
      from,
      to,
    }),

  download: (
    id: string,
    name: string,
    quality: Quality = "lossless",
    dir?: string | null,
  ) => call<string>("download_track", { id, name, quality, dir: dir || null }),

  defaultDownloadDir: () => call<string>("default_download_dir"),

  downloadImage: (url: string, name: string) =>
    call<string>("download_image", { url, name }),

  findLocalTrack: (id: string, dir?: string | null) =>
    call<string | null>("find_local_track", { id, dir: dir || null }),

  removeLocalTrack: (id: string, dir?: string | null) =>
    call<boolean>("remove_local_track", { id, dir: dir || null }),

  downloadsInfo: (dir?: string | null) =>
    call<[number, number]>("downloads_info", { dir: dir || null }),

  stations: () => call<Station[]>("get_stations"),

  stationInfo: (station?: string) =>
    call<StationInfo>("station_info", { station: station || null }),

  setStationSettings: (payload: {
    station: string;
    language?: string | null;
    moodEnergy?: string | null;
    diversity?: string | null;
  }) => call<{ ok: boolean }>("set_station_settings", { payload }),

  sendFeedback: (payload: FeedbackPayload) =>
    call<{ ok: boolean }>("wave_feedback", { payload }),

  searchSuggest: (text: string) => call<string[]>("search_suggest", { text }),

  similarTracks: (id: string) => call<Track[]>("get_similar_tracks", { id }),

  playlistRecommendations: (kind: number | string) =>
    call<Track[]>("playlist_recommendations", { kind: Number(kind) }),

  playlists: () => call<Playlist[]>("get_playlists"),

  playlistCreate: (title: string, isPublic = false) =>
    call<Playlist>("playlist_create", { title, public: isPublic }),

  playlistRename: (kind: number | string, title: string) =>
    call<boolean>("playlist_rename", { kind: Number(kind), title }),

  playlistSetVisibility: (kind: number | string, isPublic: boolean) =>
    call<boolean>("playlist_set_visibility", {
      kind: Number(kind),
      public: isPublic,
    }),

  playlistDelete: (kind: number | string) =>
    call<boolean>("playlist_delete", { kind: Number(kind) }),

  playlistClear: (kind: number | string) =>
    call<number>("playlist_clear", { kind: Number(kind) }),

  playlistTracks: (kind: number | string) =>
    call<Track[]>("get_playlist_tracks", { kind: Number(kind) }),

  track: (id: string) => call<Track>("get_track", { id }),

  trackInfo: (id: string) => call<TrackInfo>("get_track_info", { id }),

  stream: (id: string, quality: Quality = "lossless") =>
    call<Stream>("get_stream", { id, quality }),

  lyrics: (id: string) => call<Lyrics>("get_lyrics", { id }),

  updateDiscordPresence: (payload: PresencePayload) =>
    call<void>("update_discord_presence", { payload }),

  clearDiscordPresence: () => call<void>("clear_discord_presence"),

  discordPresenceStatus: () => call<PresenceStatus>("discord_presence_status"),

  reconnectDiscordPresence: (applicationId: string) =>
    call<PresenceStatus>("reconnect_discord_presence", { applicationId }),

  validateDiscordApp: (applicationId: string) =>
    call<string>("validate_discord_app", { applicationId }),

  setTrayTooltip: (text: string) => call<void>("set_tray_tooltip", { text }),

  setCloseToTray: (enabled: boolean) =>
    call<void>("set_close_to_tray", { enabled }),

  hideToTray: () => call<void>("hide_to_tray"),

  showFromTray: () => call<void>("show_from_tray"),

  quitApp: () => call<void>("quit_app"),

  setGlobalHotkeys: (
    bindings: Array<{ action: string; accelerator: string }>,
  ) => call<string[]>("set_global_hotkeys", { bindings }),

  clearGlobalHotkeys: () => call<void>("clear_global_hotkeys"),

  enterMiniPlayer: () => call<void>("enter_mini_player"),

  exitMiniPlayer: () => call<void>("exit_mini_player"),

  exportTextFile: (name: string, content: string, dir?: string | null) =>
    call<string>("export_text_file", { name, content, dir: dir || null }),

  readTextFile: (path: string) => call<string>("read_text_file", { path }),

  openExternal: (url: string) => call<void>("open_external", { url }),
};
