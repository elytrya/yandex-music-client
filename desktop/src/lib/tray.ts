import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { api } from "@/api/client";
import { createLogger } from "@/lib/log";

const log = createLogger("tray");

export interface TrayHandlers {
  onToggle: () => void;
  onNext: () => void;
  onPrev: () => void;
  onLike: () => void;
  onHidden: () => void;
  onShown: () => void;
}

let bound = false;
const unlisteners: UnlistenFn[] = [];

export async function bindTrayEvents(handlers: TrayHandlers): Promise<void> {
  if (bound) return;
  bound = true;
  try {
    unlisteners.push(await listen("tray://toggle", () => handlers.onToggle()));
    unlisteners.push(await listen("tray://next", () => handlers.onNext()));
    unlisteners.push(await listen("tray://prev", () => handlers.onPrev()));
    unlisteners.push(await listen("tray://like", () => handlers.onLike()));
    unlisteners.push(await listen("app://hidden", () => handlers.onHidden()));
    unlisteners.push(await listen("app://shown", () => handlers.onShown()));
    log.info("tray events bound");
  } catch (error) {
    log.warn("tray events failed", error);
  }
}

export function unbindTrayEvents(): void {
  for (const off of unlisteners.splice(0)) {
    try {
      off();
    } catch {}
  }
  bound = false;
}

export async function setTrayTooltip(text: string): Promise<void> {
  try {
    await api.setTrayTooltip(text);
  } catch (error) {
    log.warn("tray tooltip failed", error);
  }
}

export async function setCloseToTray(enabled: boolean): Promise<void> {
  try {
    await api.setCloseToTray(enabled);
  } catch (error) {
    log.warn("close to tray flag failed", error);
  }
}

export async function hideToTray(): Promise<void> {
  try {
    await api.hideToTray();
  } catch (error) {
    log.warn("hide to tray failed", error);
  }
}

export async function quitApp(): Promise<void> {
  try {
    await api.quitApp();
  } catch (error) {
    log.warn("quit failed", error);
  }
}
