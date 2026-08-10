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

function sanitizeOrder<T extends string>(
  saved: unknown,
  canonical: readonly T[],
): T[] {
  const known = new Set<string>(canonical);
  const seen = new Set<string>();
  const out: T[] = [];

  if (Array.isArray(saved)) {
    for (const id of saved) {
      if (typeof id === "string" && known.has(id) && !seen.has(id)) {
        seen.add(id);
        out.push(id as T);
      }
    }
  }
  for (const id of canonical) {
    if (!seen.has(id)) out.push(id);
  }
  return out;
}

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
    merged.playerOrder = sanitizeOrder(saved.playerOrder, playerButtonOrder);
    merged.miniOrder = sanitizeOrder(saved.miniOrder, miniButtonOrder);
    merged.playerEditMode = false;
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
  state: () => ({
    settings: load(),
    snapshot: null as InterfaceSettings | null,
  }),

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

    beginEdit() {
      this.snapshot = JSON.parse(
        JSON.stringify(this.settings),
      ) as InterfaceSettings;
    },

    finishEdit() {
      this.snapshot = null;
      this.settings.playerEditMode = false;
      this.apply();
    },

    cancelEdit() {
      if (this.snapshot) this.settings = { ...this.snapshot };
      this.snapshot = null;
      this.settings.playerEditMode = false;
      this.apply();
    },

    togglePlayerEdit(on?: boolean) {
      const next = on ?? !this.settings.playerEditMode;
      if (!next) {
        this.finishEdit();
        return;
      }
      this.beginEdit();
      this.settings.playerEditMode = true;
      this.apply();
    },

    playerZone(id: PlayerButtonId): PlayerZone {
      if (id === "play") return "center";
      return this.settings.playerButtons[id] ?? "off";
    },

    playerZoneButtons(zone: PlayerZone): PlayerButtonId[] {
      return this.playerOrderList().filter(
        (id) => this.playerZone(id) === zone,
      );
    },

    playerOrderList(): PlayerButtonId[] {
      return sanitizeOrder(this.settings.playerOrder, playerButtonOrder);
    },

    miniOrderList(): MiniButtonId[] {
      return sanitizeOrder(this.settings.miniOrder, miniButtonOrder);
    },

    movePlayerButton(id: PlayerButtonId, zone: PlayerZone, index: number) {
      if (id === "play" && zone !== "center") return;

      const wasIn = this.playerZone(id);
      const order = this.playerOrderList().filter((x) => x !== id);

      const inZone = order.filter((x) => this.playerZone(x) === zone);
      const raw = Number.isFinite(index) ? Math.round(index) : inZone.length;
      const clamped = Math.max(0, Math.min(raw, inZone.length));

      let at: number;
      if (!inZone.length) {
        at = order.length;
      } else if (clamped < inZone.length) {
        at = order.indexOf(inZone[clamped]);
      } else {
        at = order.indexOf(inZone[inZone.length - 1]) + 1;
      }

      order.splice(at, 0, id);

      if (wasIn !== zone) {
        this.settings.playerButtons = {
          ...this.settings.playerButtons,
          [id]: zone,
        };
      }

      this.settings.playerOrder = order;
      this.apply();
    },

    moveMiniButton(id: MiniButtonId, index: number) {
      const order = this.miniOrderList().filter((x) => x !== id);
      const raw = Number.isFinite(index) ? Math.round(index) : order.length;
      const clamped = Math.max(0, Math.min(raw, order.length));
      order.splice(clamped, 0, id);
      this.settings.miniOrder = order;
      this.apply();
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
      return this.miniOrderList().filter(
        (id) => lockedMiniButtons.has(id) || this.settings.miniButtons[id],
      );
    },

    setMiniButton(id: MiniButtonId, on: boolean) {
      if (lockedMiniButtons.has(id)) return;
      this.settings.miniButtons = { ...this.settings.miniButtons, [id]: on };
      this.apply();
    },

    resetPlayerButtons() {
      const d = defaultInterfaceSettings;
      this.settings.playerButtons = { ...defaultPlayerButtonSlots };
      this.settings.miniButtons = { ...defaultMiniButtons };
      this.settings.playerOrder = [...playerButtonOrder];
      this.settings.miniOrder = [...miniButtonOrder];
      this.settings.miniShowTime = d.miniShowTime;
      this.apply();
    },

    resetPlayerLayout() {
      const d = defaultInterfaceSettings;
      this.settings.playerHeight = d.playerHeight;
      this.settings.playerCoverSize = d.playerCoverSize;
      this.settings.playerIconSize = d.playerIconSize;
      this.settings.playerGap = d.playerGap;
      this.settings.playerSidePadding = d.playerSidePadding;
      this.settings.playerProgressWidth = d.playerProgressWidth;
      this.settings.playerProgressThickness = d.playerProgressThickness;
      this.settings.playerShowTimes = d.playerShowTimes;
      this.settings.playerMetaWidth = d.playerMetaWidth;
      this.apply();
    },

    resetMiniLayout() {
      const d = defaultInterfaceSettings;
      this.settings.miniOpacity = d.miniOpacity;
      this.settings.miniCoverSize = d.miniCoverSize;
      this.settings.miniIconSize = d.miniIconSize;
      this.settings.miniGap = d.miniGap;
      this.settings.miniPadding = d.miniPadding;
      this.settings.miniVolumeSlider = d.miniVolumeSlider;
      this.settings.miniVolumeHeight = d.miniVolumeHeight;
      this.settings.miniShowTime = d.miniShowTime;
      this.settings.miniVisualizer = d.miniVisualizer;
      this.settings.miniWidth = d.miniWidth;
      this.settings.miniHeight = d.miniHeight;
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
      this.settings.lyricsLineHeight = d.lyricsLineHeight;
      this.settings.lyricsWeight = d.lyricsWeight;
      this.settings.lyricsFont = d.lyricsFont;
      this.settings.lyricsBackgroundBlur = d.lyricsBackgroundBlur;
      this.settings.lyricsBackgroundOpacity = d.lyricsBackgroundOpacity;
      this.settings.lyricsLineBlur = d.lyricsLineBlur;
      this.settings.lyricsInactive = d.lyricsInactive;
      this.settings.lyricsAlign = d.lyricsAlign;
      this.settings.lyricsBackdrop = d.lyricsBackdrop;
      this.settings.lyricsHighlight = d.lyricsHighlight;
      this.settings.lyricsGlow = d.lyricsGlow;
      this.settings.lyricsShowArtwork = d.lyricsShowArtwork;
      this.settings.lyricsMotion = d.lyricsMotion;
      this.settings.lyricsShowCredits = d.lyricsShowCredits;
      this.settings.lyricsShowOrigin = d.lyricsShowOrigin;
      this.settings.lyricsAnnotations = d.lyricsAnnotations;
      this.settings.lyricsAnnotationMark = d.lyricsAnnotationMark;
      this.settings.lyricsSource = d.lyricsSource;
      this.apply();
    },
  },
});
