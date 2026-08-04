import { Dark } from "quasar";
import type { InterfaceSettings } from "./defaults";
import { isLightColor, lightThemes, themePalettes } from "./themes";

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
    "--lyrics-bg-blur": `${settings.lyricsBackgroundBlur}px`,
    "--lyrics-bg-opacity": `${settings.lyricsBackgroundOpacity / 100}`,
    "--lyrics-line-blur": `${settings.lyricsLineBlur}px`,
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
  root.dataset.coverStyle = settings.coverStyle;
}
