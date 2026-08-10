import type { Track } from "@/api/types";
import { QUEUE_LIMIT } from "./protocol";

export interface QueueSlice {
  queue: Track[];
  index: number;
}

/** Берём окно вокруг текущего трека, чтобы не гнать по сети тысячи треков. */
export function trimQueue(queue: Track[], index: number): QueueSlice {
  if (queue.length <= QUEUE_LIMIT) return { queue: [...queue], index };

  const half = Math.floor(QUEUE_LIMIT / 2);
  const start = Math.max(0, Math.min(index - half, queue.length - QUEUE_LIMIT));

  return {
    queue: queue.slice(start, start + QUEUE_LIMIT),
    index: index - start,
  };
}


/** Сжимаем окно очереди, пока пакет не влезет в лимит сервера. */
export function fitBudget(
  base: Record<string, unknown>,
  slice: QueueSlice,
  budget: number,
): QueueSlice {
  const measure = (queue: Track[]) =>
    new TextEncoder().encode(JSON.stringify({ ...base, queue })).length;

  if (measure(slice.queue) <= budget) return slice;

  let span = slice.queue.length;
  while (span > 1) {
    span = Math.floor(span / 2);
    const half = Math.floor(span / 2);
    const start = Math.max(
      0,
      Math.min(slice.index - half, slice.queue.length - span),
    );
    const window = slice.queue.slice(start, start + span);
    if (measure(window) <= budget) {
      return { queue: window, index: slice.index - start };
    }
  }

  const current = slice.queue[slice.index];
  if (current && measure([current]) <= budget) {
    return { queue: [current], index: 0 };
  }

  return { queue: [], index: 0 };
}

/** Ищем трек в присланной очереди: сначала по позиции, потом по id. */
export function findIndex(
  queue: Track[],
  index: number,
  trackId: string,
): number {
  if (queue[index]?.id === trackId) return index;
  return queue.findIndex((track) => track.id === trackId);
}
