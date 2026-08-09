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

/** Ищем трек в присланной очереди: сначала по позиции, потом по id. */
export function findIndex(
  queue: Track[],
  index: number,
  trackId: string,
): number {
  if (queue[index]?.id === trackId) return index;
  return queue.findIndex((track) => track.id === trackId);
}
