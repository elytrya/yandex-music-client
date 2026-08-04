import { defineStore } from "pinia";
import { applyEqualizer, EQ_BANDS } from "@/lib/audio";

const STORAGE_KEY = "mashiro.equalizer";

export interface EqualizerPreset {
  id: string;
  label: string;
  gains: number[];
}

export const eqPresets: EqualizerPreset[] = [
  { id: "flat", label: "Ровно", gains: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
  {
    id: "bass",
    label: "Басы",
    gains: [6, 5, 4, 2, 0, 0, 0, 0, 0, 0],
  },
  {
    id: "vocal",
    label: "Голос",
    gains: [-2, -1, 0, 2, 4, 4, 3, 1, 0, -1],
  },
  {
    id: "pop",
    label: "Поп",
    gains: [-1, 1, 3, 4, 3, 1, 0, -1, -1, -2],
  },
  {
    id: "rock",
    label: "Рок",
    gains: [5, 3, 1, -1, -1, 1, 3, 4, 4, 3],
  },
  {
    id: "electronic",
    label: "Электроника",
    gains: [5, 4, 2, 0, -1, 1, 2, 3, 4, 4],
  },
  {
    id: "night",
    label: "Ночь",
    gains: [-3, -2, -1, 0, 1, 2, 1, 0, -1, -2],
  },
];

interface EqualizerState {
  enabled: boolean;
  preamp: number;
  gains: number[];
  preset: string;
  unsupported: boolean;
}

function load(): EqualizerState {
  const base: EqualizerState = {
    enabled: false,
    preamp: 0,
    gains: EQ_BANDS.map(() => 0),
    preset: "flat",
    unsupported: false,
  };
  try {
    const saved = JSON.parse(
      localStorage.getItem(STORAGE_KEY) || "{}",
    ) as Partial<EqualizerState>;
    const gains = Array.isArray(saved.gains)
      ? EQ_BANDS.map((_, i) => Number(saved.gains?.[i] ?? 0))
      : base.gains;
    return { ...base, ...saved, gains, unsupported: false };
  } catch {
    return base;
  }
}

export const useEqualizerStore = defineStore("equalizer", {
  state: (): EqualizerState => load(),

  getters: {
    bands: () => EQ_BANDS,
    activePreset: (s) =>
      eqPresets.find((preset) => preset.gains.join(",") === s.gains.join(","))
        ?.id || "custom",
  },

  actions: {
    apply() {
      const ok = applyEqualizer(this.enabled, this.gains, this.preamp);
      this.unsupported = !ok;
      try {
        localStorage.setItem(
          STORAGE_KEY,
          JSON.stringify({
            enabled: this.enabled,
            preamp: this.preamp,
            gains: this.gains,
            preset: this.preset,
          }),
        );
      } catch {}
    },

    toggle(value?: boolean) {
      this.enabled = value ?? !this.enabled;
      this.apply();
    },

    setBand(index: number, value: number) {
      const gains = [...this.gains];
      gains[index] = Math.max(-12, Math.min(12, value));
      this.gains = gains;
      this.preset = "custom";
      this.apply();
    },

    setPreamp(value: number) {
      this.preamp = Math.max(-12, Math.min(12, value));
      this.apply();
    },

    usePreset(id: string) {
      const preset = eqPresets.find((item) => item.id === id);
      if (!preset) return;
      this.gains = [...preset.gains];
      this.preset = preset.id;
      if (!this.enabled) this.enabled = true;
      this.apply();
    },

    reset() {
      this.gains = EQ_BANDS.map(() => 0);
      this.preamp = 0;
      this.preset = "flat";
      this.apply();
    },
  },
});
