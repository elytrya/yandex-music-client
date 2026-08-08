import { convertFileSrc } from "@tauri-apps/api/core";

const STORAGE_KEY = "mashiro.censor.custom";

export type CensorOverride = {
  source: string;
  label: string;
  at: number;
};

type Store = Record<string, CensorOverride>;

function key(id: string | null | undefined): string {
  return String(id ?? "").split(":")[0];
}

let cache: Store | null = null;

function read(): Store {
  if (cache) return cache;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const parsed = raw ? (JSON.parse(raw) as Store) : {};
    cache = parsed && typeof parsed === "object" ? parsed : {};
  } catch {
    cache = {};
  }
  return cache;
}

function write(next: Store): void {
  cache = next;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
  } catch {}
}

export function hasOverride(id: string | null | undefined): boolean {
  if (!id) return false;
  return Boolean(read()[key(id)]);
}

export function getOverride(
  id: string | null | undefined,
): CensorOverride | null {
  if (!id) return null;
  return read()[key(id)] ?? null;
}

export function overrideUrl(id: string | null | undefined): string | null {
  const item = getOverride(id);
  if (!item) return null;
  const source = item.source.trim();
  if (!source) return null;
  if (/^(https?|blob|data|asset):/i.test(source)) return source;
  try {
    return convertFileSrc(source);
  } catch {
    return source;
  }
}

export function setOverride(id: string, source: string, label: string): void {
  const trimmed = source.trim();
  if (!trimmed) return clearOverride(id);
  const next = { ...read() };
  next[key(id)] = { source: trimmed, label: label.trim(), at: Date.now() };
  write(next);
}

export function clearOverride(id: string): void {
  const next = { ...read() };
  delete next[key(id)];
  write(next);
}

export function listOverrides(): Array<CensorOverride & { id: string }> {
  return Object.entries(read())
    .map(([id, value]) => ({ id, ...value }))
    .sort((a, b) => b.at - a.at);
}

export function clearAllOverrides(): void {
  write({});
}
