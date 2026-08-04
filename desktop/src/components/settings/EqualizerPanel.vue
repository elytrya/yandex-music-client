<template>
  <div class="eq-panel">
    <div class="eq-head">
      <button class="eq-power" :class="{ on: eq.enabled }" @click="eq.toggle()">
        {{ eq.enabled ? "Включён" : "Выключен" }}
      </button>
      <div class="eq-presets">
        <button
          v-for="preset in presets"
          :key="preset.id"
          :class="{ on: eq.activePreset === preset.id }"
          @click="eq.usePreset(preset.id)"
        >
          {{ preset.label }}
        </button>
      </div>
      <button class="eq-reset" @click="eq.reset()">Сбросить</button>
    </div>

    <div class="eq-bands" :class="{ off: !eq.enabled }">
      <div v-for="(band, index) in eq.bands" :key="band" class="eq-band">
        <span class="eq-gain">{{ formatGain(eq.gains[index] ?? 0) }}</span>
        <q-slider
          :model-value="eq.gains[index] ?? 0"
          :min="-12"
          :max="12"
          :step="0.5"
          vertical
          reverse
          dense
          @update:model-value="eq.setBand(index, Number($event ?? 0))"
        />
        <span class="eq-freq">{{ formatFreq(band) }}</span>
      </div>
    </div>

    <div class="eq-preamp">
      <span>Предусиление</span>
      <q-slider
        :model-value="eq.preamp"
        :min="-12"
        :max="12"
        :step="0.5"
        dense
        @update:model-value="eq.setPreamp(Number($event ?? 0))"
      />
      <code>{{ formatGain(eq.preamp) }}</code>
    </div>

    <div v-if="eq.unsupported" class="eq-warning">
      Эквалайзер недоступен для текущего потока. Попробуй перезапустить трек.
    </div>
  </div>
</template>

<script setup lang="ts">
import { eqPresets, useEqualizerStore } from "@/stores/equalizer";

const eq = useEqualizerStore();
const presets = eqPresets;

function formatFreq(value: number): string {
  return value >= 1000 ? `${value / 1000}k` : `${value}`;
}

function formatGain(value: number): string {
  const rounded = Math.round(value * 10) / 10;
  return `${rounded > 0 ? "+" : ""}${rounded}`;
}
</script>
