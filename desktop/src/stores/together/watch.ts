import { watch } from "vue";
import type { WatchStopHandle } from "vue";
import type { usePlayerStore } from "@/stores/player/index";
import { JUMP_LIMIT, PUSH_DEBOUNCE } from "./protocol";

type Player = ReturnType<typeof usePlayerStore>;

export interface PlayerHandlers {
  change: (why: string) => void;
  loading: (busy: boolean) => void;
}

/**
 * Следит за плеером реактивно, а не опросом раз в секунду: пауза,
 * смена трека и перемотка улетают в комнату сразу.
 */
export function watchPlayer(
  player: Player,
  handlers: PlayerHandlers,
): WatchStopHandle[] {
  let timer: number | null = null;
  let mark = player.progress;
  let stamp = Date.now();

  function fire(why: string) {
    if (timer !== null) window.clearTimeout(timer);
    timer = window.setTimeout(() => {
      timer = null;
      handlers.change(why);
    }, PUSH_DEBOUNCE);
  }

  function reset() {
    mark = player.progress;
    stamp = Date.now();
  }

  return [
    watch(
      () => player.current?.id,
      () => {
        reset();
        fire("трек");
      },
    ),

    watch(
      () => player.isPlaying,
      (value) => {
        reset();
        fire(value ? "плей" : "пауза");
      },
    ),

    watch(
      () => player.progress,
      (value) => {
        const now = Date.now();
        const moved = player.isPlaying ? (now - stamp) / 1000 : 0;
        const drift = Math.abs(value - (mark + moved));

        mark = value;
        stamp = now;

        if (drift > JUMP_LIMIT) fire("перемотка");
      },
    ),

    watch(
      () => player.loading,
      (busy) => handlers.loading(busy),
    ),
  ];
}
