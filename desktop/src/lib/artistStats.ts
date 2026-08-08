import type { ArtistPage } from "@/api/types";

const STORAGE_KEY = "mashiro.artistStats";
const MAX_SNAPSHOTS = 90;
const MIN_GAP_MS = 6 * 60 * 60 * 1000;

export interface ArtistSnapshot {
  t: number;
  listeners: number | null;
  likes: number | null;
  tracks: number | null;
  albums: number | null;
}

export interface StatDelta {
  value: number | null;
  diff: number | null;
  since: number | null;
  span: number | null;
}

type StatsStore = Record<string, ArtistSnapshot[]>;

function readStore(): StatsStore {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as StatsStore;
    return parsed && typeof parsed === "object" ? parsed : {};
  } catch {
    return {};
  }
}

function writeStore(store: StatsStore) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
  } catch {}
}

function sameNumbers(a: ArtistSnapshot, b: ArtistSnapshot): boolean {
  return (
    a.listeners === b.listeners &&
    a.likes === b.likes &&
    a.tracks === b.tracks &&
    a.albums === b.albums
  );
}

export function artistHistory(id: string): ArtistSnapshot[] {
  const list = readStore()[id];
  return Array.isArray(list) ? [...list].sort((a, b) => a.t - b.t) : [];
}

export function recordArtistStats(artist: ArtistPage): ArtistSnapshot[] {
  if (!artist?.id) return [];

  const store = readStore();
  const list = Array.isArray(store[artist.id]) ? store[artist.id] : [];
  const snapshot: ArtistSnapshot = {
    t: Date.now(),
    listeners: artist.listeners ?? null,
    likes: artist.likes ?? null,
    tracks: artist.tracks_count ?? null,
    albums: artist.albums_count ?? null,
  };

  const last = list[list.length - 1];
  if (last && sameNumbers(last, snapshot) && snapshot.t - last.t < MIN_GAP_MS) {
    return [...list];
  }

  const next =
    last && sameNumbers(last, snapshot)
      ? [...list.slice(0, -1), snapshot]
      : [...list, snapshot];

  const trimmed = next.slice(-MAX_SNAPSHOTS);
  store[artist.id] = trimmed;
  writeStore(store);
  return trimmed;
}

function pickBaseline(
  history: ArtistSnapshot[],
  days: number | null,
): ArtistSnapshot | null {
  if (history.length < 2) return null;
  if (days === null) return history[history.length - 2] ?? null;

  const cutoff = Date.now() - days * 24 * 60 * 60 * 1000;
  let baseline: ArtistSnapshot | null = null;
  for (const point of history) {
    if (point.t <= cutoff) baseline = point;
    else break;
  }
  return baseline ?? history[0] ?? null;
}

export function statDelta(
  history: ArtistSnapshot[],
  field: keyof Omit<ArtistSnapshot, "t">,
  days: number | null = null,
): StatDelta {
  const latest = history[history.length - 1] ?? null;
  const value = latest ? (latest[field] ?? null) : null;
  const baseline = pickBaseline(history, days);

  if (!latest || !baseline || baseline === latest) {
    return { value, diff: null, since: null, span: null };
  }

  const before = baseline[field];
  if (before === null || value === null) {
    return { value, diff: null, since: baseline.t, span: null };
  }

  return {
    value,
    diff: value - before,
    since: baseline.t,
    span: latest.t - baseline.t,
  };
}

export function seriesOf(
  history: ArtistSnapshot[],
  field: keyof Omit<ArtistSnapshot, "t">,
): Array<{ t: number; v: number }> {
  return history
    .filter((p) => typeof p[field] === "number")
    .map((p) => ({ t: p.t, v: p[field] as number }));
}

export function clearArtistStats(id?: string) {
  if (!id) {
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch {}
    return;
  }
  const store = readStore();
  delete store[id];
  writeStore(store);
}
