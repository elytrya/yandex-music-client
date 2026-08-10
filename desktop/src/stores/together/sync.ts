import { api } from "@/api/client";
import type { usePlayerStore } from "@/stores/player/index";
import type { StatePayload } from "./protocol";
import { DRIFT_LIMIT, SYNC_BUDGET } from "./protocol";
import { fitBudget, findIndex, trimQueue } from "./queue";

type Player = ReturnType<typeof usePlayerStore>;

/**
 * @param withQueue полное состояние с очередью или лёгкий пульс без неё
 */
export function buildState(player: Player, withQueue = true): StatePayload {
  const base: StatePayload = {
    kind: "state",
    trackId: player.current?.id ?? null,
    positionMs: Math.round(player.progress * 1000),
    paused: !player.isPlaying,
    updatedAt: Date.now(),
    title: player.current?.title ?? null,
    track: player.current ?? null,
    queue: [],
    index: 0,
  };

  if (!withQueue) return base;

  const slice = fitBudget(
    base,
    trimQueue(player.queue, player.index),
    SYNC_BUDGET,
  );

  return { ...base, queue: slice.queue, index: slice.index };
}

export function expectedPosition(payload: StatePayload): number {
  const lag = payload.paused
    ? 0
    : Math.max(0, Date.now() - payload.updatedAt) / 1000;

  return payload.positionMs / 1000 + lag;
}

export async function applyState(
  player: Player,
  payload: StatePayload,
): Promise<void> {
  if (!payload.trackId) return;

  const target = expectedPosition(payload);

  if (player.current?.id !== payload.trackId) {
    await startTrack(player, payload);
    player.seek(target);
  } else if (Math.abs(player.progress - target) > DRIFT_LIMIT) {
    player.seek(target);
  }

  if (payload.paused === player.isPlaying) player.toggle();
}

async function startTrack(
  player: Player,
  payload: StatePayload,
): Promise<void> {
  const trackId = payload.trackId!;

  // прислали очередь - повторяем её целиком, тогда работает и дальше/назад
  if (payload.queue.length) {
    const index = findIndex(payload.queue, payload.index, trackId);
    if (index >= 0) {
      await player.playQueue(payload.queue, index);
      return;
    }
  }

  // пульс без очереди: берём трек из самого сообщения
  if (payload.track && payload.track.id === trackId) {
    await player.playQueue([payload.track], 0);
    return;
  }

  // старый клиент или пустое сообщение: тянем трек своим аккаунтом
  const track = await api.track(trackId);
  await player.playQueue([track], 0);
}
