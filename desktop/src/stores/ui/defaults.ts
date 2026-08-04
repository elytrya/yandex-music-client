import type { MiniButtonSlots, PlayerButtonSlots } from "./playerButtons";
import { defaultMiniButtons, defaultPlayerButtonSlots } from "./playerButtons";
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
  miniShowTime: boolean;
  miniVisualizer: boolean;
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
  miniShowTime: true,
  miniVisualizer: true,
  playerVisualizer: false,
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
