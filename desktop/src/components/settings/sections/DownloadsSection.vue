<template>
  <section id="settings-downloads" class="settings-group">
    <div class="settings-group-head">
      <h2>Загрузки и качество</h2>
      <p>Скачанные треки играются с диска и не тратят трафик.</p>
    </div>

    <label class="setting-row setting-field-row">
      <span class="setting-copy">
        <b>Папка для загрузок</b>
        <span
          >Пусто - папка по умолчанию:
          {{ defaultDir || "Музыка/Mashiro" }}</span
        >
      </span>
      <input
        :value="ui.settings.downloadDir"
        type="text"
        spellcheck="false"
        :placeholder="defaultDir"
        @change="setDir"
      />
    </label>

    <div class="setting-row">
      <div class="setting-copy">
        <b>Скачано</b>
        <span>{{ info }}</span>
      </div>
      <div class="discord-actions">
        <button class="settings-reset-button" type="button" @click="refresh">
          Обновить
        </button>
        <button class="settings-reset-button" type="button" @click="useDefault">
          Папка по умолчанию
        </button>
      </div>
    </div>

    <SettingToggle
      :model-value="ui.settings.preferLocalFiles"
      label="Сначала искать скачанный файл"
      description="Если трек уже есть на диске - играем его, иначе берём из сети."
      @update:model-value="ui.set('preferLocalFiles', $event)"
    />

    <div class="setting-row">
      <div class="setting-copy">
        <b>Качество воспроизведения</b>
        <span>Максимум, что отдаёт Яндекс для твоей подписки.</span>
      </div>
      <div class="settings-choice">
        <button
          v-for="option in qualities"
          :key="option.value"
          type="button"
          :class="{ on: player.quality === option.value }"
          @click="player.setQuality(option.value)"
        >
          {{ option.label }}
        </button>
      </div>
    </div>

    <SettingToggle
      :model-value="ui.settings.crossfadeEnabled"
      label="Плавные переходы"
      description="Старый трек угасает, новый плавно разгорается."
      @update:model-value="ui.set('crossfadeEnabled', $event)"
    />

    <SettingSlider
      v-if="ui.settings.crossfadeEnabled"
      :model-value="ui.settings.crossfadeSeconds"
      label="Длительность перехода"
      description="Сколько длится угасание и разгорание."
      :min="0.5"
      :max="8"
      :step="0.5"
      suffix=" с"
      @update:model-value="ui.set('crossfadeSeconds', $event)"
    />
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "@/api/client";
import type { Quality } from "@/api/types";
import SettingSlider from "@/components/settings/SettingSlider.vue";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import { usePlayerStore } from "@/stores/player/index";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();
const player = usePlayerStore();

const defaultDir = ref("");
const info = ref("Считаем…");

const qualities: Array<{ value: Quality; label: string }> = [
  { value: "low", label: "Экономное" },
  { value: "normal", label: "Стандартное" },
  { value: "high", label: "Высокое" },
  { value: "lossless", label: "Максимальное" },
];

function setDir(event: Event) {
  const value = (event.target as HTMLInputElement).value.trim();
  ui.set("downloadDir", value);
  void refresh();
}

function useDefault() {
  ui.set("downloadDir", "");
  void refresh();
}

async function refresh() {
  try {
    const [count, mb] = await api.downloadsInfo(
      ui.settings.downloadDir || null,
    );
    info.value = count ? `${count} файлов, ${mb} МБ` : "Пока ничего не скачано";
  } catch {
    info.value = "Папка недоступна";
  }
}

onMounted(async () => {
  defaultDir.value = await api.defaultDownloadDir().catch(() => "");
  void refresh();
});
</script>
