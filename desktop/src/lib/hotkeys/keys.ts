const DISPLAY: Record<string, string> = {
  ArrowRight: "→",
  ArrowLeft: "←",
  ArrowUp: "↑",
  ArrowDown: "↓",
  Space: "Пробел",
  Escape: "Esc",
  Ctrl: "Ctrl",
  Alt: "Alt",
  Shift: "Shift",
  Meta: "Win",
};

const ACCELERATOR_DISPLAY: Record<string, string> = {
  CommandOrControl: "Ctrl",
  Control: "Ctrl",
  Super: "Win",
  Right: "→",
  Left: "←",
  Up: "↑",
  Down: "↓",
  MediaPlayPause: "Play / Pause",
  MediaTrackNext: "Next Track",
  MediaTrackPrevious: "Prev Track",
  MediaStop: "Stop",
};

const ACCELERATOR_KEYS: Record<string, string> = {
  ArrowRight: "Right",
  ArrowLeft: "Left",
  ArrowUp: "Up",
  ArrowDown: "Down",
  Ctrl: "CommandOrControl",
  Meta: "Super",
};

const MODIFIER_CODES = new Set([
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "ShiftLeft",
  "ShiftRight",
  "MetaLeft",
  "MetaRight",
]);

export function isModifierEvent(event: KeyboardEvent): boolean {
  return MODIFIER_CODES.has(event.code);
}

function baseKey(event: KeyboardEvent): string {
  if (event.code.startsWith("Key")) return event.code.slice(3);
  if (event.code.startsWith("Digit")) return event.code.slice(5);
  if (event.code.startsWith("Numpad")) return event.code;
  if (event.code === "Space") return "Space";
  if (event.key.length === 1) return event.key.toUpperCase();
  return event.key;
}

export function eventSignature(event: KeyboardEvent): string {
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  if (event.metaKey) parts.push("Meta");
  parts.push(baseKey(event));
  return parts.join("+");
}

export function displayParts(signature: string): string[] {
  if (!signature) return [];
  return signature.split("+").map((part) => DISPLAY[part] || part);
}

export function displayAcceleratorParts(accelerator: string): string[] {
  if (!accelerator) return [];
  return accelerator
    .split("+")
    .map((part) => ACCELERATOR_DISPLAY[part] || DISPLAY[part] || part);
}

export function toAccelerator(signature: string): string {
  if (!signature) return "";
  return signature
    .split("+")
    .map((part) => ACCELERATOR_KEYS[part] || part)
    .join("+");
}

export function isTypingTarget(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  if (!el || !el.tagName) return false;
  const tag = el.tagName.toLowerCase();
  if (tag === "input" || tag === "textarea" || tag === "select") return true;
  return el.isContentEditable === true;
}
