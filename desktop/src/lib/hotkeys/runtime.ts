import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { createLogger } from "@/lib/log";
import { HOTKEYS } from "./defaults";
import type { HotkeyAction } from "./defaults";
import { eventSignature, isModifierEvent, isTypingTarget } from "./keys";

const log = createLogger("hotkeys");

export type HotkeyHandlers = Partial<Record<HotkeyAction, () => void>>;

let localHandler: ((event: KeyboardEvent) => void) | null = null;
const unlisteners: UnlistenFn[] = [];

export function bindLocalHotkeys(
  resolve: (signature: string) => HotkeyAction | null,
  handlers: HotkeyHandlers,
): void {
  unbindLocalHotkeys();
  localHandler = (event: KeyboardEvent) => {
    if (event.repeat || isModifierEvent(event)) return;
    if (isTypingTarget(event.target)) return;
    const action = resolve(eventSignature(event));
    if (!action) return;
    const run = handlers[action];
    if (!run) return;
    event.preventDefault();
    event.stopPropagation();
    run();
  };
  window.addEventListener("keydown", localHandler, true);
}

export function unbindLocalHotkeys(): void {
  if (!localHandler) return;
  window.removeEventListener("keydown", localHandler, true);
  localHandler = null;
}

export async function bindGlobalHotkeyEvents(
  handlers: HotkeyHandlers,
): Promise<void> {
  if (unlisteners.length) return;
  for (const meta of HOTKEYS) {
    const run = handlers[meta.action];
    if (!run) continue;
    try {
      unlisteners.push(await listen(`hotkey://${meta.action}`, () => run()));
    } catch (error) {
      log.warn("global hotkey listen failed", error);
    }
  }
}

export function unbindGlobalHotkeyEvents(): void {
  for (const off of unlisteners.splice(0)) {
    try {
      off();
    } catch {}
  }
}
