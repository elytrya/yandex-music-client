import { defineStore } from "pinia";
import { Notify } from "quasar";
import { audio, cancelFade, fadeOut } from "@/lib/audio";

let ticker: number | null = null;

export const useSleepStore = defineStore("sleep", {
  state: () => ({
    endsAt: 0,
    untilTrackEnd: false,
    now: Date.now(),
    firing: false,
  }),

  getters: {
    active: (s) => s.endsAt > 0 || s.untilTrackEnd,
    remainingMs: (s) => (s.endsAt > 0 ? Math.max(0, s.endsAt - s.now) : 0),
    label(): string {
      if (this.untilTrackEnd) return "До конца трека";
      if (!this.endsAt) return "Выключен";
      const total = Math.ceil(this.remainingMs / 1000);
      const m = Math.floor(total / 60);
      const s = total % 60;
      return `${m}:${String(s).padStart(2, "0")}`;
    },
  },

  actions: {
    startTicker() {
      if (ticker !== null) return;
      ticker = window.setInterval(() => {
        this.now = Date.now();
        if (this.endsAt > 0 && this.now >= this.endsAt) void this.fire();
      }, 1000);
    },

    stopTicker() {
      if (ticker !== null) {
        window.clearInterval(ticker);
        ticker = null;
      }
    },

    start(minutes: number) {
      this.untilTrackEnd = false;
      this.endsAt = Date.now() + minutes * 60000;
      this.now = Date.now();
      this.startTicker();
      Notify.create({ message: `Таймер сна: через ${minutes} мин` });
    },

    startUntilTrackEnd() {
      this.endsAt = 0;
      this.untilTrackEnd = true;
      this.startTicker();
      Notify.create({ message: "Таймер сна: до конца трека" });
    },

    cancel(silent = false) {
      const had = this.active;
      this.endsAt = 0;
      this.untilTrackEnd = false;
      this.stopTicker();
      if (had && !silent) Notify.create({ message: "Таймер сна выключен" });
    },

    onTrackEnded(): boolean {
      if (!this.untilTrackEnd) return false;
      void this.fire();
      return true;
    },

    async fire() {
      if (this.firing) return;
      this.firing = true;
      const { usePlayerStore } = await import("@/stores/player/index");
      const player = usePlayerStore();
      this.cancel(true);
      try {
        await fadeOut(6000);
        audio.pause();
        player.isPlaying = false;
      } finally {
        cancelFade(player.muted ? 0 : player.volume);
        this.firing = false;
        Notify.create({ message: "Спокойной ночи - музыка выключена" });
      }
    },
  },
});
