import type {
  MiniButtonId,
  MiniButtonSlots,
  PlayerButtonId,
  PlayerButtonSlots,
} from "./playerButtons";
import {
  defaultMiniButtons,
  defaultPlayerButtonSlots,
  miniButtonOrder,
  playerButtonOrder,
} from "./playerButtons";
import type { ThemeName } from "./themes";

export type Density = "compact" | "comfortable" | "spacious";
export type CoverStyle = "square" | "soft" | "rounded";

export const DEFAULT_DISCORD_CLIENT_ID = "1532382113821753354";

export interface InterfaceSettings {
  theme: ThemeName;
  accent: string;
  customBackground: string;
  customSurface: string;
  customSurface2: string;
  customText: string;
  radius: number;
  density: Density;
  sidebarWidth: number;
  searchWidth: number;
  pagePadding: number;
  cardSize: number;
  textScale: number;
  coverStyle: CoverStyle;
  glass: boolean;
  glassBlur: number;
  animations: boolean;
  thinScrollbar: boolean;
  showPlaylistCovers: boolean;
  showPlayerArtwork: boolean;
  playerButtons: PlayerButtonSlots;
  miniButtons: MiniButtonSlots;
  playerOrder: PlayerButtonId[];
  miniOrder: MiniButtonId[];
  playerHeight: number;
  playerCoverSize: number;
  playerIconSize: number;
  playerGap: number;
  playerSidePadding: number;
  playerProgressWidth: number;
  playerProgressThickness: number;
  playerShowTimes: boolean;
  playerMetaWidth: number;
  miniShowTime: boolean;
  miniVisualizer: boolean;
  miniOpacity: number;
  miniCoverSize: number;
  miniIconSize: number;
  miniGap: number;
  miniPadding: number;
  miniVolumeSlider: boolean;
  miniVolumeHeight: number;
  playerVisualizer: boolean;
  lyricsFontSize: number;
  lyricsBackgroundBlur: number;
  lyricsBackgroundOpacity: number;
  lyricsLineBlur: number;
  lyricsAlign: "left" | "center";
  lyricsShowArtwork: boolean;
  lyricsMotion: boolean;
  discordEnabled: boolean;
  discordClientId: string;
  discordDetails: string;
  discordState: string;
  discordButtonLabel: string;
  discordShowArtwork: boolean;
  discordShowTime: boolean;
  cacheEnabled: boolean;
  downloadDir: string;
  autoSkipDisliked: boolean;
  preferLocalFiles: boolean;
  crossfadeEnabled: boolean;
  crossfadeSeconds: number;
  trimSilence: boolean;
  censorBypass: boolean;
  censorBadge: boolean;
  minimizeToTray: boolean;
  autoDislikeAi: boolean;
  resumeLastSession: boolean;
  resumeAutoplay: boolean;
}

export const defaultInterfaceSettings: InterfaceSettings = {
  theme: "black",
  accent: "#fa2d48",
  customBackground: "#09090b",
  customSurface: "#151518",
  customSurface2: "#1c1c20",
  customText: "#f5f5f7",
  radius: 12,
  density: "comfortable",
  sidebarWidth: 236,
  searchWidth: 360,
  pagePadding: 40,
  cardSize: 150,
  textScale: 100,
  coverStyle: "soft",
  glass: true,
  glassBlur: 24,
  animations: true,
  thinScrollbar: false,
  showPlaylistCovers: true,
  showPlayerArtwork: true,
  playerButtons: { ...defaultPlayerButtonSlots },
  miniButtons: { ...defaultMiniButtons },
  playerOrder: [...playerButtonOrder],
  miniOrder: [...miniButtonOrder],
  playerHeight: 78,
  playerCoverSize: 48,
  playerIconSize: 18,
  playerGap: 6,
  playerSidePadding: 16,
  playerProgressWidth: 100,
  playerProgressThickness: 4,
  playerShowTimes: true,
  playerMetaWidth: 260,
  miniShowTime: true,
  miniVisualizer: true,
  miniOpacity: 100,
  miniCoverSize: 38,
  miniIconSize: 15,
  miniGap: 6,
  miniPadding: 10,
  miniVolumeSlider: true,
  miniVolumeHeight: 96,
  playerVisualizer: true,
  lyricsFontSize: 36,
  lyricsBackgroundBlur: 38,
  lyricsBackgroundOpacity: 48,
  lyricsLineBlur: 2.5,
  lyricsAlign: "left",
  lyricsShowArtwork: true,
  lyricsMotion: true,
  discordEnabled: true,
  discordClientId: DEFAULT_DISCORD_CLIENT_ID,
  discordDetails: "{title}",
  discordState: "{artist}",
  discordButtonLabel: "Слушать в Яндекс Музыке",
  discordShowArtwork: true,
  discordShowTime: true,
  cacheEnabled: true,
  downloadDir: "",
  autoSkipDisliked: true,
  preferLocalFiles: true,
  crossfadeEnabled: true,
  crossfadeSeconds: 4,
  trimSilence: false,
  censorBypass: true,
  censorBadge: true,
  minimizeToTray: true,
  autoDislikeAi: false,
  resumeLastSession: true,
  resumeAutoplay: true,
};
