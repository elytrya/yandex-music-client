<template>
  <div class="lyrics-settings" data-no-drag @click.stop>
    <div class="lyrics-settings-head">
      <b>Настройки текста</b>
      <button class="lyrics-settings-reset" @click="ui.resetLyrics()">
        Сбросить
      </button>
    </div>

    <div v-for="item in sliders" :key="item.key" class="lyrics-settings-row">
      <span>{{ item.label }}</span>
      <q-slider
        :model-value="ui.settings[item.key]"
        :min="item.min"
        :max="item.max"
        :step="item.step"
        dense
        @update:model-value="update(item.key, Number($event ?? 0))"
      />
      <code>{{ ui.settings[item.key] }}{{ item.suffix }}</code>
    </div>

    <div class="lyrics-settings-row">
      <span>Выравнивание</span>
      <div class="lyrics-settings-choice">
        <button
          :class="{ on: ui.settings.lyricsAlign === 'left' }"
          @click="ui.set('lyricsAlign', 'left')"
        >
          Слева
        </button>
        <button
          :class="{ on: ui.settings.lyricsAlign === 'center' }"
          @click="ui.set('lyricsAlign', 'center')"
        >
          По центру
        </button>
      </div>
    </div>

    <button
      class="lyrics-settings-toggle"
      :class="{ on: ui.settings.lyricsShowArtwork }"
      @click="ui.set('lyricsShowArtwork', !ui.settings.lyricsShowArtwork)"
    >
      <span>Большая обложка</span><i />
    </button>
    <button
      class="lyrics-settings-toggle"
      :class="{ on: ui.settings.lyricsMotion }"
      @click="ui.set('lyricsMotion', !ui.settings.lyricsMotion)"
    >
      <span>Анимация строк</span><i />
    </button>
  </div>
</template>

<script setup lang="ts">
import type { InterfaceSettings } from "@/stores/ui/index";
import { useUiStore } from "@/stores/ui/index";

type NumberKey =
  | "lyricsFontSize"
  | "lyricsBackgroundBlur"
  | "lyricsBackgroundOpacity"
  | "lyricsLineBlur";

const ui = useUiStore();

const sliders: Array<{
  key: NumberKey;
  label: string;
  min: number;
  max: number;
  step: number;
  suffix: string;
}> = [
  {
    key: "lyricsFontSize",
    label: "Размер строк",
    min: 20,
    max: 64,
    step: 1,
    suffix: "px",
  },
  {
    key: "lyricsBackgroundBlur",
    label: "Размытие фона",
    min: 0,
    max: 60,
    step: 1,
    suffix: "px",
  },
  {
    key: "lyricsBackgroundOpacity",
    label: "Яркость фона",
    min: 0,
    max: 85,
    step: 1,
    suffix: "%",
  },
  {
    key: "lyricsLineBlur",
    label: "Размытие строк",
    min: 0,
    max: 8,
    step: 0.5,
    suffix: "px",
  },
];

function update(key: NumberKey, value: number) {
  ui.set(key, value as InterfaceSettings[NumberKey]);
}
</script>
