import type { Quality, RepeatMode } from "@/api/types";

const SETTINGS_KEY = "mashiro.settings";
const QUALITY_UPGRADE_KEY = "mashiro.settings.qualityUpgraded";

export interface PlaybackSettings {
  volume: number;
  quality: Quality;
  playbackRate: number;
  repeat: RepeatMode;
  muted: boolean;
}

export const defaultPlaybackSettings: PlaybackSettings = {
  volume: 0.8,
  quality: "lossless",
  playbackRate: 1,
  repeat: "off",
  muted: false,
};

export function loadPlaybackSettings(): PlaybackSettings {
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (!raw) return { ...defaultPlaybackSettings };
    const stored = {
      ...defaultPlaybackSettings,
      ...(JSON.parse(raw) as Partial<PlaybackSettings>),
    };
    if (
      stored.quality === "high" &&
      !localStorage.getItem(QUALITY_UPGRADE_KEY)
    ) {
      stored.quality = "lossless";
      localStorage.setItem(QUALITY_UPGRADE_KEY, "1");
      savePlaybackSettings(stored);
    }
    return stored;
  } catch {
    return { ...defaultPlaybackSettings };
  }
}

export function savePlaybackSettings(settings: PlaybackSettings): void {
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
  } catch {}
}
