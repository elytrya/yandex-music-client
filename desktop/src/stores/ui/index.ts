import { defineStore } from "pinia";
import { applyInterfaceSettings } from "./apply";
import type { InterfaceSettings } from "./defaults";
import {
  DEFAULT_DISCORD_CLIENT_ID,
  defaultInterfaceSettings,
} from "./defaults";
import type {
  MiniButtonId,
  MiniButtonSlots,
  PlayerButtonId,
  PlayerButtonSlots,
  PlayerZone,
} from "./playerButtons";
import {
  defaultMiniButtons,
  defaultPlayerButtonSlots,
  miniButtonOrder,
  playerButtonOrder,
} from "./playerButtons";

export type {
  MiniButtonId,
  MiniButtonSlots,
  PlayerButtonId,
  PlayerButtonSlots,
  PlayerZone,
} from "./playerButtons";
export {
  defaultMiniButtons,
  defaultPlayerButtonSlots,
  miniButtonCatalog,
  miniButtonOrder,
  playerButtonCatalog,
  playerButtonOrder,
  playerZoneLabels,
} from "./playerButtons";

export type { CoverStyle, Density, InterfaceSettings } from "./defaults";
export {
  DEFAULT_DISCORD_CLIENT_ID,
  defaultInterfaceSettings,
} from "./defaults";
export type { Palette, ThemeName } from "./themes";
export { accentColors, themeNames, themePalettes } from "./themes";

const STORAGE_KEY = "mashiro.interface";

const lockedMiniButtons = new Set<MiniButtonId>(["prev", "play", "next"]);

function load(): InterfaceSettings {
  try {
    const saved = JSON.parse(
      localStorage.getItem(STORAGE_KEY) || "{}",
    ) as Partial<InterfaceSettings>;
    const merged: InterfaceSettings = { ...defaultInterfaceSettings, ...saved };
    merged.playerButtons = {
      ...defaultPlayerButtonSlots,
      ...(saved.playerButtons || {}),
    } as PlayerButtonSlots;
    merged.miniButtons = {
      ...defaultMiniButtons,
      ...(saved.miniButtons || {}),
    } as MiniButtonSlots;
    merged.playerButtons.play = "center";
    merged.miniButtons.play = true;
    merged.miniButtons.prev = true;
    merged.miniButtons.next = true;
    if (!merged.discordClientId.trim()) {
      merged.discordClientId = DEFAULT_DISCORD_CLIENT_ID;
    }
    return merged;
  } catch {
    return { ...defaultInterfaceSettings };
  }
}

export const useUiStore = defineStore("ui", {
  state: () => ({ settings: load() }),

  actions: {
    apply() {
      applyInterfaceSettings(this.settings);
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(this.settings));
      } catch {}
    },

    set<K extends keyof InterfaceSettings>(
      key: K,
      value: InterfaceSettings[K],
    ) {
      this.settings[key] = value;
      this.apply();
    },

    playerZone(id: PlayerButtonId): PlayerZone {
      if (id === "play") return "center";
      return this.settings.playerButtons[id] ?? "off";
    },

    playerZoneButtons(zone: PlayerZone): PlayerButtonId[] {
      return playerButtonOrder.filter((id) => this.playerZone(id) === zone);
    },

    setPlayerZone(id: PlayerButtonId, zone: PlayerZone) {
      if (id === "play") return;
      this.settings.playerButtons = {
        ...this.settings.playerButtons,
        [id]: zone,
      };
      this.apply();
    },

    activeMiniButtons(): MiniButtonId[] {
      return miniButtonOrder.filter(
        (id) => lockedMiniButtons.has(id) || this.settings.miniButtons[id],
      );
    },

    setMiniButton(id: MiniButtonId, on: boolean) {
      if (lockedMiniButtons.has(id)) return;
      this.settings.miniButtons = { ...this.settings.miniButtons, [id]: on };
      this.apply();
    },

    resetPlayerButtons() {
      this.settings.playerButtons = { ...defaultPlayerButtonSlots };
      this.settings.miniButtons = { ...defaultMiniButtons };
      this.settings.miniShowTime = defaultInterfaceSettings.miniShowTime;
      this.apply();
    },

    useCustomTheme() {
      this.settings.theme = "custom";
      this.apply();
    },

    reset() {
      this.settings = { ...defaultInterfaceSettings };
      this.apply();
    },

    resetLyrics() {
      const d = defaultInterfaceSettings;
      this.settings.lyricsFontSize = d.lyricsFontSize;
      this.settings.lyricsBackgroundBlur = d.lyricsBackgroundBlur;
      this.settings.lyricsBackgroundOpacity = d.lyricsBackgroundOpacity;
      this.settings.lyricsLineBlur = d.lyricsLineBlur;
      this.settings.lyricsAlign = d.lyricsAlign;
      this.settings.lyricsShowArtwork = d.lyricsShowArtwork;
      this.settings.lyricsMotion = d.lyricsMotion;
      this.apply();
    },
  },
});
