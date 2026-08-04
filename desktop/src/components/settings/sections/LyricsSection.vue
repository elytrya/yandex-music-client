<template>
  <section id="settings-lyrics" class="settings-group">
    <div class="settings-group-head">
      <h2>Текст песни</h2>
      <p>
        Те же настройки доступны прямо в полноэкранном тексте - кнопка
        шестерёнки.
      </p>
    </div>

    <SettingSlider
      v-for="item in sliders"
      :key="item.key"
      :label="item.label"
      :description="item.description"
      :model-value="ui.settings[item.key]"
      :min="item.min"
      :max="item.max"
      :step="item.step"
      :suffix="item.suffix"
      @update:model-value="ui.set(item.key, $event)"
    />

    <div class="setting-row">
      <div class="setting-copy">
        <b>Выравнивание</b><span>Положение текста в области лирикса.</span>
      </div>
      <div class="settings-choice">
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

    <SettingToggle
      :model-value="ui.settings.lyricsShowArtwork"
      label="Большая обложка"
      description="Показывать обложку рядом с текстом."
      @update:model-value="ui.set('lyricsShowArtwork', $event)"
    />
    <SettingToggle
      :model-value="ui.settings.lyricsMotion"
      label="Анимация строк"
      description="Плавно перемещать активную строку."
      @update:model-value="ui.set('lyricsMotion', $event)"
    />
  </section>
</template>

<script setup lang="ts">
import SettingSlider from "@/components/settings/SettingSlider.vue";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();

type SliderKey =
  | "lyricsFontSize"
  | "lyricsBackgroundBlur"
  | "lyricsBackgroundOpacity"
  | "lyricsLineBlur";

const sliders: Array<{
  key: SliderKey;
  label: string;
  description: string;
  min: number;
  max: number;
  step: number;
  suffix: string;
}> = [
  {
    key: "lyricsFontSize",
    label: "Размер строк",
    description: "Размер синхронизированного текста.",
    min: 20,
    max: 64,
    step: 1,
    suffix: " px",
  },
  {
    key: "lyricsBackgroundBlur",
    label: "Размытие фона",
    description: "Размытие обложки за текстом.",
    min: 0,
    max: 60,
    step: 1,
    suffix: " px",
  },
  {
    key: "lyricsBackgroundOpacity",
    label: "Яркость фона",
    description: "Видимость фоновой обложки.",
    min: 0,
    max: 85,
    step: 1,
    suffix: "%",
  },
  {
    key: "lyricsLineBlur",
    label: "Размытие строк",
    description: "Размытие строк вокруг активной.",
    min: 0,
    max: 8,
    step: 0.5,
    suffix: " px",
  },
];
</script>
