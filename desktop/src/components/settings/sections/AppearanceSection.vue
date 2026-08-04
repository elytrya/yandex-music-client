<template>
  <section id="settings-theme" class="settings-group">
    <div class="settings-group-head">
      <h2>Оформление</h2>
      <p>Готовая палитра или полностью свои цвета.</p>
    </div>

    <div class="setting-row setting-row-column">
      <div class="setting-copy">
        <b>Тема</b>
        <span>Цвета фона, панелей и текста.</span>
      </div>
      <div class="theme-grid">
        <button
          v-for="theme in themes"
          :key="theme.value"
          class="theme-option"
          :class="{ on: ui.settings.theme === theme.value }"
          @click="ui.set('theme', theme.value)"
        >
          <span class="theme-preview" :style="{ background: theme.background }">
            <i :style="{ background: theme.surface }" />
            <i :style="{ background: theme.surface2 }" />
          </span>
          <span>{{ theme.label }}</span>
        </button>
        <button
          class="theme-option"
          :class="{ on: ui.settings.theme === 'custom' }"
          @click="ui.useCustomTheme()"
        >
          <span
            class="theme-preview custom-theme-preview"
            :style="{ background: ui.settings.customBackground }"
          >
            <i :style="{ background: ui.settings.customSurface }" />
            <i :style="{ background: ui.settings.customSurface2 }" />
          </span>
          <span>Своя</span>
        </button>
      </div>
    </div>

    <div
      v-if="ui.settings.theme === 'custom'"
      class="setting-row setting-row-column"
    >
      <div class="setting-copy">
        <b>Своя палитра</b>
        <span>Настрой каждый основной цвет отдельно.</span>
      </div>
      <div class="custom-palette-grid">
        <ColorField
          label="Фон"
          :value="ui.settings.customBackground"
          @change="setColor('customBackground', $event)"
        />
        <ColorField
          label="Панели"
          :value="ui.settings.customSurface"
          @change="setColor('customSurface', $event)"
        />
        <ColorField
          label="Активные панели"
          :value="ui.settings.customSurface2"
          @change="setColor('customSurface2', $event)"
        />
        <ColorField
          label="Текст"
          :value="ui.settings.customText"
          @change="setColor('customText', $event)"
        />
      </div>
    </div>

    <div class="setting-row setting-row-column">
      <div class="setting-copy">
        <b>Акцент</b>
        <span>Кнопки, активные пункты и индикаторы.</span>
      </div>
      <div class="settings-accent-row">
        <button
          v-for="color in accents"
          :key="color.value"
          class="settings-accent"
          :class="{ on: ui.settings.accent === color.value }"
          :title="color.label"
          :style="{ background: color.value }"
          @click="ui.set('accent', color.value)"
        >
          <Icon
            v-if="ui.settings.accent === color.value"
            name="check"
            :size="14"
          />
        </button>
        <ColorField
          label="Свой"
          :value="ui.settings.accent"
          @change="setColor('accent', $event)"
        />
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import ColorField from "@/components/settings/ColorField.vue";
import Icon from "@/components/Icon.vue";
import {
  accentColors,
  themeNames,
  themePalettes,
  useUiStore,
} from "@/stores/ui/index";

const ui = useUiStore();
const accents = accentColors;
const themes = themeNames.map((theme) => ({
  ...theme,
  ...themePalettes[theme.value],
}));

type ColorKey =
  | "accent"
  | "customBackground"
  | "customSurface"
  | "customSurface2"
  | "customText";

function setColor(key: ColorKey, value: string) {
  ui.settings[key] = value;
  if (key !== "accent") ui.settings.theme = "custom";
  ui.apply();
}
</script>
