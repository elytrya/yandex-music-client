import { Dark } from "quasar";
import type { InterfaceSettings, LyricsFont } from "./defaults";
import { isLightColor, lightThemes, themePalettes } from "./themes";

const lyricsFonts: Record<LyricsFont, string> = {
  sans: "inherit",
  serif:
    '"Iowan Old Style", "Palatino Linotype", Georgia, "Times New Roman", serif',
  mono: 'ui-monospace, "JetBrains Mono", "Cascadia Code", Consolas, monospace',
  custom: "inherit",
};

function lyricsFontStack(settings: InterfaceSettings): string {
  if (settings.lyricsFont !== "custom") {
    return lyricsFonts[settings.lyricsFont] ?? "inherit";
  }
  const raw = (settings.lyricsFontCustom || "").trim();
  if (!raw) return "inherit";
  const stack = raw
    .split(",")
    .map((part) => part.trim().replace(/^["']|["']$/g, ""))
    .filter(Boolean)
    .map((part) => (/^[a-zA-Z0-9-]+$/.test(part) ? part : `"${part}"`));
  return stack.length ? `${stack.join(", ")}, sans-serif` : "inherit";
}

export function applyInterfaceSettings(settings: InterfaceSettings): void {
  const root = document.documentElement;
  const preset =
    settings.theme === "custom" ? null : themePalettes[settings.theme];

  const background = preset?.background ?? settings.customBackground;
  const surface = preset?.surface ?? settings.customSurface;
  const surface2 = preset?.surface2 ?? settings.customSurface2;
  const light = lightThemes.has(settings.theme) || isLightColor(background);
  const hover =
    preset?.hover ??
    (light
      ? `color-mix(in srgb, ${surface2} 88%, black)`
      : `color-mix(in srgb, ${surface2} 76%, white)`);
  const rawText = preset?.text ?? settings.customText;
  const text =
    !preset && isLightColor(rawText) === light
      ? light
        ? "#16161a"
        : "#f5f5f7"
      : rawText;
  const line = light ? "rgba(0, 0, 0, 0.105)" : "rgba(255, 255, 255, 0.085)";
  const fgDim = `color-mix(in srgb, ${text} 64%, ${background})`;
  const fgFaint = `color-mix(in srgb, ${text} 42%, ${background})`;
  const trackHeight =
    settings.density === "compact"
      ? 44
      : settings.density === "spacious"
        ? 58
        : 52;

  const vars: Record<string, string> = {
    "--bg": background,
    "--surface": surface,
    "--surface-2": surface2,
    "--hover": hover,
    "--fg": text,
    "--fg-dim": fgDim,
    "--fg-faint": fgFaint,
    "--line": line,
    "--shadow-strong": light ? "rgba(15, 18, 26, 0.22)" : "rgba(0, 0, 0, 0.75)",
    "--shadow-soft": light ? "rgba(15, 18, 26, 0.12)" : "rgba(0, 0, 0, 0.45)",
    "--accent": settings.accent,
    "--radius": `${settings.radius}px`,
    "--track-height": `${trackHeight}px`,
    "--sidebar-width": `${settings.sidebarWidth}px`,
    "--search-width": `${settings.searchWidth}px`,
    "--page-padding": `${settings.pagePadding}px`,
    "--card-size": `${settings.cardSize}px`,
    "--text-scale": `${settings.textScale / 100}`,
    "--glass-blur": `${settings.glassBlur}px`,
    "--lyrics-size": `${settings.lyricsFontSize}px`,
    "--lyrics-line-height": `${settings.lyricsLineHeight}`,
    "--lyrics-weight": `${settings.lyricsWeight}`,
    "--lyrics-font": lyricsFontStack(settings),
    "--lyrics-inactive": `${settings.lyricsInactive / 100}`,
    "--lyrics-bg-blur": `${settings.lyricsBackgroundBlur}px`,
    "--lyrics-bg-opacity": `${settings.lyricsBackgroundOpacity / 100}`,
    "--lyrics-line-blur": `${settings.lyricsLineBlur}px`,
    "--player-height": `${settings.playerHeight}px`,
    "--player-cover": `${settings.playerCoverSize}px`,
    "--player-icon": `${settings.playerIconSize}px`,
    "--player-gap": `${settings.playerGap}px`,
    "--player-pad": `${settings.playerSidePadding}px`,
    "--player-progress-width": `${settings.playerProgressWidth}%`,
    "--player-progress-thickness": `${settings.playerProgressThickness}px`,
    "--player-meta-width": `${settings.playerMetaWidth}px`,
    "--mini-opacity": `${settings.miniOpacity / 100}`,
    "--mini-cover": `${settings.miniCoverSize}px`,
    "--mini-icon": `${settings.miniIconSize}px`,
    "--mini-gap": `${settings.miniGap}px`,
    "--mini-pad": `${settings.miniPadding}px`,
    "--mini-volume-height": `${settings.miniVolumeHeight}px`,
  };

  for (const [name, value] of Object.entries(vars)) {
    root.style.setProperty(name, value);
  }

  root.dataset.mode = light ? "light" : "dark";
  root.classList.toggle("light-theme", light);
  Dark.set(!light);

  root.classList.toggle("no-glass", !settings.glass);
  root.classList.toggle("no-motion", !settings.animations);
  root.classList.toggle("thin-scrollbar", settings.thinScrollbar);
  root.classList.toggle("hide-playlist-covers", !settings.showPlaylistCovers);
  root.classList.toggle("hide-player-artwork", !settings.showPlayerArtwork);
  root.classList.toggle("hide-player-times", !settings.playerShowTimes);
  root.dataset.coverStyle = settings.coverStyle;
}
