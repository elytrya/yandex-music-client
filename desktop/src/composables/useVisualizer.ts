import { onBeforeUnmount, onMounted, watch, type Ref } from "vue";
import { frequencyData } from "@/lib/audio";
import { usePlayerStore } from "@/stores/player/index";

interface VisualizerOptions {
  bars?: number;
}

export function useVisualizer(
  canvas: Ref<HTMLCanvasElement | null>,
  enabled: () => boolean,
  options: VisualizerOptions = {},
) {
  const player = usePlayerStore();
  const barCount = options.bars ?? 56;
  const levels = new Array<number>(barCount).fill(0);
  let rafId = 0;

  function frame() {
    const el = canvas.value;
    if (!el) {
      rafId = 0;
      return;
    }
    const ctx = el.getContext("2d");
    if (!ctx) {
      rafId = 0;
      return;
    }

    const dpr = window.devicePixelRatio || 1;
    const w = el.clientWidth;
    const h = el.clientHeight;
    const pw = Math.round(w * dpr);
    const ph = Math.round(h * dpr);
    if (el.width !== pw || el.height !== ph) {
      el.width = pw;
      el.height = ph;
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);

    const playing = player.isPlaying;
    const data = playing ? frequencyData() : null;
    const usable = data ? Math.floor(data.length * 0.66) : 0;
    const step = data ? Math.max(1, Math.floor(usable / barCount)) : 1;

    const accent =
      getComputedStyle(document.documentElement)
        .getPropertyValue("--accent")
        .trim() || "#fa2d48";
    const gap = 2;
    const bw = Math.max(1, (w - gap * (barCount - 1)) / barCount);
    ctx.fillStyle = accent;

    let maxLevel = 0;
    for (let i = 0; i < barCount; i++) {
      let target = 0;
      if (data && data.length) {
        let sum = 0;
        for (let j = 0; j < step; j++) sum += data[i * step + j] ?? 0;
        const raw = sum / step / 255;
        target = raw * raw;
      }
      const cur = levels[i] ?? 0;
      const ease = target > cur ? 0.42 : 0.08;
      const next = cur + (target - cur) * ease;
      levels[i] = next;
      if (next > maxLevel) maxLevel = next;

      if (next <= 0.002) continue;
      const bh = Math.max(2, next * h);
      const x = i * (bw + gap);
      ctx.globalAlpha = 0.2 + next * 0.6;
      ctx.fillRect(x, h - bh, bw, bh);
    }
    ctx.globalAlpha = 1;

    if (!playing && maxLevel < 0.004) {
      rafId = 0;
      return;
    }
    rafId = requestAnimationFrame(frame);
  }

  function start() {
    stop();
    if (enabled()) rafId = requestAnimationFrame(frame);
  }

  function stop() {
    if (rafId) cancelAnimationFrame(rafId);
    rafId = 0;
  }

  watch(enabled, start, { flush: "post" });
  watch(
    () => player.isPlaying,
    (playing) => {
      if (playing) start();
    },
  );

  onMounted(start);
  onBeforeUnmount(stop);

  return { start, stop };
}
