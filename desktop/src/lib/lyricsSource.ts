import { api } from "@/api/client";
import type { Lyrics, Track } from "@/api/types";
import { artistNames } from "@/lib/format";
import { useGeniusStore } from "@/stores/genius";
import type { LyricsSource } from "@/stores/ui/defaults";
import { useUiStore } from "@/stores/ui/index";

export type LyricsOrigin = "lrclib" | "genius" | "yandex";

export interface LyricsResult {
  lyrics: Lyrics;
  origin: LyricsOrigin;
}

export const ORIGIN_LABEL: Record<LyricsOrigin, string> = {
  lrclib: "lrclib",
  genius: "genius",
  yandex: "яндекс",
};

export function lyricsMode(): LyricsSource {
  return useUiStore().settings.lyricsSource;
}

export function sourceOrder(mode: LyricsSource): LyricsOrigin[] {
  switch (mode) {
    case "lrclib":
      return ["lrclib"];
    case "genius":
      return ["genius"];
    case "yandex":
      return ["yandex"];
    default:
      return ["lrclib", "genius"];
  }
}

export function hasText(value: Lyrics | null | undefined): value is Lyrics {
  return Boolean(value?.lines?.some((line) => (line.text || "").trim()));
}

function mainArtist(track: Track): string {
  return track.artists?.[0]?.name || artistNames(track.artists || []);
}

async function fromLrclib(
  track: Track,
  force: boolean,
): Promise<Lyrics | null> {
  const found = await api.lrclibLookup({
    title: track.title,
    artist: mainArtist(track),
    album: track.album_title,
    duration: track.duration_ms ? Math.round(track.duration_ms / 1000) : null,
    force,
  });
  if (!found) return null;

  const lyrics: Lyrics = {
    track_id: track.id,
    synced: found.synced,
    lines: found.lines,
    writers: [],
  };
  return hasText(lyrics) ? lyrics : null;
}

async function fromGenius(
  track: Track,
  force: boolean,
): Promise<Lyrics | null> {
  const genius = useGeniusStore();
  if (!genius.ready) return null;

  const lines = await genius.lyricsFor(track, force);
  if (!lines.length) return null;

  const lyrics: Lyrics = {
    track_id: track.id,
    synced: false,
    lines: lines.map((text) => ({ time_ms: 0, text })),
    writers: (genius.song?.credits || [])
      .filter((person) => person.role === "Автор")
      .map((person) => person.name),
  };
  return hasText(lyrics) ? lyrics : null;
}

async function fromYandex(track: Track): Promise<Lyrics | null> {
  const found = await api.lyrics(track.id);
  return hasText(found) ? found : null;
}

export async function loadTrackLyrics(
  track: Track,
  mode: LyricsSource = lyricsMode(),
  force = false,
): Promise<LyricsResult | null> {
  let lastError: unknown = null;

  for (const origin of sourceOrder(mode)) {
    try {
      const lyrics =
        origin === "lrclib"
          ? await fromLrclib(track, force)
          : origin === "genius"
            ? await fromGenius(track, force)
            : await fromYandex(track);
      if (lyrics) return { lyrics, origin };
    } catch (error) {
      lastError = error;
    }
  }

  if (lastError) throw lastError;
  return null;
}
