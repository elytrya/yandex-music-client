import { api } from "@/api/client";
import type { usePlayerStore } from "@/stores/player/index";
import type { StatePayload } from "./protocol";
import { DRIFT_LIMIT } from "./protocol";

type Player = ReturnType<typeof usePlayerStore>;

export function buildState(player: Player): StatePayload {
  return {
    kind: "state",
    trackId: player.current?.id ?? null,
    positionMs: Math.round(player.progress * 1000),
    paused: !player.isPlaying,
    updatedAt: Date.now(),
    title: player.current?.title ?? null,
  };
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
    const track = await api.track(payload.trackId);
    await player.playQueue([track], 0);
    player.seek(target);
  } else if (Math.abs(player.progress - target) > DRIFT_LIMIT) {
    player.seek(target);
  }

  if (payload.paused === player.isPlaying) player.toggle();
}
