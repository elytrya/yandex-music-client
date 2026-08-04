<template>
  <section id="settings-layout" class="settings-group">
    <div class="settings-group-head">
      <h2>Интерфейс</h2>
      <p>Размеры применяются ко всему приложению, включая меню и панели.</p>
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
        <b>Плотность списков</b><span>Высота строк с треками.</span>
      </div>
      <div class="settings-choice">
        <button
          v-for="option in densities"
          :key="option.value"
          :class="{ on: ui.settings.density === option.value }"
          @click="ui.set('density', option.value)"
        >
          {{ option.label }}
        </button>
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-copy">
        <b>Форма обложек</b
        ><span>Отдельно от общего скругления интерфейса.</span>
      </div>
      <div class="settings-choice">
        <button
          v-for="option in coverStyles"
          :key="option.value"
          :class="{ on: ui.settings.coverStyle === option.value }"
          @click="ui.set('coverStyle', option.value)"
        >
          {{ option.label }}
        </button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import SettingSlider from "@/components/settings/SettingSlider.vue";
import type { CoverStyle, Density } from "@/stores/ui/index";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();

type SliderKey =
  | "textScale"
  | "radius"
  | "pagePadding"
  | "cardSize"
  | "sidebarWidth"
  | "searchWidth";

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
    key: "textScale",
    label: "Масштаб текста",
    description: "Текст в навигации, меню, плеере и на страницах.",
    min: 85,
    max: 125,
    step: 1,
    suffix: "%",
  },
  {
    key: "radius",
    label: "Скругление",
    description: "Карточки, меню, кнопки, поля и переключатели.",
    min: 0,
    max: 24,
    step: 1,
    suffix: " px",
  },
  {
    key: "pagePadding",
    label: "Отступы страниц",
    description: "Расстояние между содержимым и краями окна.",
    min: 16,
    max: 64,
    step: 2,
    suffix: " px",
  },
  {
    key: "cardSize",
    label: "Размер карточек",
    description: "Обложки на главной странице и в подборках.",
    min: 120,
    max: 210,
    step: 2,
    suffix: " px",
  },
  {
    key: "sidebarWidth",
    label: "Ширина боковой панели",
    description: "Навигация и список плейлистов.",
    min: 200,
    max: 320,
    step: 2,
    suffix: " px",
  },
  {
    key: "searchWidth",
    label: "Ширина поиска",
    description: "Поисковая строка в верхней панели.",
    min: 240,
    max: 540,
    step: 10,
    suffix: " px",
  },
];

const densities: Array<{ value: Density; label: string }> = [
  { value: "compact", label: "Плотно" },
  { value: "comfortable", label: "Обычно" },
  { value: "spacious", label: "Свободно" },
];

const coverStyles: Array<{ value: CoverStyle; label: string }> = [
  { value: "square", label: "Строго" },
  { value: "soft", label: "Мягко" },
  { value: "rounded", label: "Круглее" },
];
</script>
