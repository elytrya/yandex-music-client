<template>
  <label class="ed-range">
    <span class="ed-range-top">
      <span class="ed-range-label">{{ label }}</span>
      <span class="ed-range-value">{{ modelValue }}{{ suffix }}</span>
    </span>
    <input
      class="ed-range-input"
      type="range"
      :min="min"
      :max="max"
      :step="step"
      :value="modelValue"
      @input="onInput"
    />
  </label>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string;
    modelValue: number;
    min: number;
    max: number;
    step?: number;
    suffix?: string;
  }>(),
  { step: 1, suffix: "" },
);

const emit = defineEmits<{
  (e: "update:modelValue", value: number): void;
}>();

function onInput(event: Event) {
  const raw = Number((event.target as HTMLInputElement).value);
  if (Number.isFinite(raw)) emit("update:modelValue", raw);
}
</script>

<style scoped>
.ed-range {
  display: block;
  user-select: none;
}

.ed-range-top {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 4px;
}

.ed-range-label {
  font-size: 11.5px;
  color: var(--fg-dim);
}

.ed-range-value {
  font-size: 11.5px;
  font-variant-numeric: tabular-nums;
  color: var(--fg);
  opacity: 0.85;
}

.ed-range-input {
  display: block;
  width: 100%;
  height: 16px;
  margin: 0;
  padding: 0;
  cursor: pointer;
  background: transparent;
  appearance: none;
  -webkit-appearance: none;
}

.ed-range-input::-webkit-slider-runnable-track {
  height: 3px;
  border-radius: 3px;
  background: color-mix(in srgb, var(--fg) 16%, transparent);
}

.ed-range-input::-moz-range-track {
  height: 3px;
  border-radius: 3px;
  background: color-mix(in srgb, var(--fg) 16%, transparent);
}

.ed-range-input::-webkit-slider-thumb {
  width: 12px;
  height: 12px;
  margin-top: -4.5px;
  border: none;
  border-radius: 50%;
  background: var(--accent);
  appearance: none;
  -webkit-appearance: none;
  transition: transform 0.12s ease;
}

.ed-range-input::-moz-range-thumb {
  width: 12px;
  height: 12px;
  border: none;
  border-radius: 50%;
  background: var(--accent);
}

.ed-range-input:hover::-webkit-slider-thumb {
  transform: scale(1.18);
}
</style>
