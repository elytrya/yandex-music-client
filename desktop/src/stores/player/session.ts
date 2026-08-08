import type { Track } from "@/api/types";

const SESSION_KEY = "mashiro.session";
const KEEP_BEFORE = 20;
const KEEP_TOTAL = 150;

export interface PlayerSession {
  queue: Track[];
  sourceQueue: Track[];
  shuffle: boolean;
  index: number;
  progress: number;
  isWave: boolean;
  stationId: string | null;
  stationName: string | null;
  waveBatchId: string | null;
  savedAt: number;
}

export function windowAround(
  queue: Track[],
  index: number,
): { queue: Track[]; index: number } {
  if (queue.length <= KEEP_TOTAL) return { queue, index };
  const from = Math.max(
    0,
    Math.min(index - KEEP_BEFORE, queue.length - KEEP_TOTAL),
  );
  return { queue: queue.slice(from, from + KEEP_TOTAL), index: index - from };
}

export function loadSession(): PlayerSession | null {
  try {
    const raw = localStorage.getItem(SESSION_KEY);
    if (!raw) return null;
    const data = JSON.parse(raw) as Partial<PlayerSession>;
    if (!Array.isArray(data.queue) || data.queue.length === 0) return null;
    const index =
      typeof data.index === "number" && data.index >= 0 ? data.index : 0;
    return {
      queue: data.queue,
      sourceQueue: Array.isArray(data.sourceQueue)
        ? data.sourceQueue
        : data.queue,
      shuffle: Boolean(data.shuffle),
      index: Math.min(index, data.queue.length - 1),
      progress: typeof data.progress === "number" ? data.progress : 0,
      isWave: Boolean(data.isWave),
      stationId: data.stationId ?? null,
      stationName: data.stationName ?? null,
      waveBatchId: data.waveBatchId ?? null,
      savedAt: typeof data.savedAt === "number" ? data.savedAt : 0,
    };
  } catch {
    return null;
  }
}

export function saveSession(session: PlayerSession): void {
  try {
    const trimmed = windowAround(session.queue, session.index);
    const source = session.shuffle
      ? session.sourceQueue.slice(0, KEEP_TOTAL)
      : trimmed.queue;
    const payload: PlayerSession = {
      ...session,
      queue: trimmed.queue,
      sourceQueue: source,
      index: trimmed.index,
    };
    localStorage.setItem(SESSION_KEY, JSON.stringify(payload));
  } catch {}
}

export function clearSession(): void {
  try {
    localStorage.removeItem(SESSION_KEY);
  } catch {}
}
