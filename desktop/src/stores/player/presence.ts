import { api } from "@/api/client";
import type { Track } from "@/api/types";
import { DEFAULT_DISCORD_CLIENT_ID, useUiStore } from "@/stores/ui/index";
import { useTogetherStore } from "@/stores/together/index";

export interface PresenceContext {
  track: Track | null;
  isPlaying: boolean;
  progress: number;
  duration: number;
  playbackRate: number;
}

function template(text: string, track: Track): string {
  const artist = track.artists.map((item) => item.name).join(", ");
  return text
    .replaceAll("{title}", track.title)
    .replaceAll("{artist}", artist)
    .replaceAll("{album}", track.album_title || "Без альбома")
    .trim();
}

function normalizeCover(url: string | null): string | null {
  if (!url) return null;
  if (url.startsWith("//")) return `https:${url}`;
  if (url.startsWith("http://")) return url.replace("http://", "https://");
  return url;
}

function clampLine(value: string, fallback: string): string {
  const text = (value || "").trim() || fallback.trim();
  const padded = text.length < 2 ? `${text} ` : text;
  return padded.slice(0, 128);
}

const TRACK_BASE = "https" + "://music.yandex.ru/track/";

let lastSignature = "";

export function resetPresenceSignature(): void {
  lastSignature = "";
}

export async function syncDiscordPresence(
  context: PresenceContext,
  options: { force?: boolean } = {},
): Promise<string | null> {
  const settings = useUiStore().settings;
  const track = context.track;
  const clientId = settings.discordClientId.trim() || DEFAULT_DISCORD_CLIENT_ID;

  if (!settings.discordEnabled || !track) {
    if (lastSignature) {
      lastSignature = "";
      await api.clearDiscordPresence().catch(() => undefined);
    }
    return null;
  }

  const duration = context.duration || (track.duration_ms ?? 0) / 1000;
  const rate = Math.max(0.25, context.playbackRate);
  const timed = settings.discordShowTime && context.isPlaying && duration > 0;
  const now = Math.floor(Date.now() / 1000);
  const artists = track.artists.map((item) => item.name).join(", ");

  const together = useTogetherStore();
  let stateText = clampLine(
    template(settings.discordState, track),
    artists || "Яндекс Музыка",
  );
  let party: { id: string; size: number; max: number } | null = null;
  if (together.active) {
    const size = Math.max(1, together.peers.length);
    const note = together.isHost
      ? size > 1
        ? `в группе · ${size}`
        : "в группе"
      : `в группе с ${together.hostNick}`;
    stateText = clampLine(`${stateText} · 🎧 ${note}`, `🎧 ${note}`);
    const room = together.invite || "together";
    party = {
      id: `mashiro:${room}`.slice(0, 128),
      size,
      max: Math.max(size, 2),
    };
  }

  const payload = {
    enabled: true,
    applicationId: clientId,
    details: clampLine(template(settings.discordDetails, track), track.title),
    state: stateText,
    album: clampLine(track.album_title || track.title, track.title),
    coverUrl: settings.discordShowArtwork
      ? normalizeCover(track.cover_url)
      : null,
    trackUrl: TRACK_BASE + track.id,
    buttonLabel: clampLine(
      settings.discordButtonLabel,
      "Слушать в Яндекс Музыке",
    ).slice(0, 31),
    startedAt: timed ? now - Math.floor(context.progress / rate) : null,
    endsAt: timed
      ? now + Math.max(1, Math.floor((duration - context.progress) / rate))
      : null,
    partyId: party?.id ?? null,
    partySize: party?.size ?? null,
    partyMax: party?.max ?? null,
  };

  const signature = JSON.stringify({
    ...payload,
    startedAt: payload.startedAt ? Math.round(payload.startedAt / 4) : null,
    endsAt: payload.endsAt ? Math.round(payload.endsAt / 4) : null,
  });
  if (!options.force && signature === lastSignature) return null;
  lastSignature = signature;

  try {
    await api.updateDiscordPresence(payload);
    return null;
  } catch (error) {
    lastSignature = "";
    return error instanceof Error ? error.message : "Discord недоступен";
  }
}
