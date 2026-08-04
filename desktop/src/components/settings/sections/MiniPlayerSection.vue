<template>
  <section id="settings-mini" class="settings-group">
    <div class="settings-group-head">
      <h2>Мини-плеер</h2>
      <p>Кнопки и полоса времени в компактном окне поверх других приложений.</p>
    </div>

    <div class="setting-row setting-row-column preview-row">
      <div class="setting-copy">
        <b>Предпросмотр мини-плеера</b>
        <span>Управление всегда по центру.</span>
      </div>

      <div class="mini-preview">
        <div class="mini-preview-top">
          <div class="mini-preview-cover">
            <Icon name="note" :size="14" class="faint" />
          </div>
          <div class="player-preview-meta">
            <span class="player-preview-line w-70" />
            <span class="player-preview-line w-45 dimmed" />
          </div>
          <span class="player-preview-btn"
            ><Icon name="restore" :size="13"
          /></span>
        </div>

        <div class="mini-preview-controls">
          <span
            v-for="id in miniActive"
            :key="id"
            class="player-preview-btn"
            :class="{ accent: id === 'play' }"
            :title="miniLabelOf(id)"
          >
            <Icon :name="miniIconOf(id)" :size="14" />
          </span>
        </div>

        <div class="mini-preview-seek">
          <span v-if="ui.settings.miniShowTime" class="player-preview-tick">
            0:42
          </span>
          <span class="player-preview-track"><i /></span>
          <span v-if="ui.settings.miniShowTime" class="player-preview-tick">
            3:15
          </span>
        </div>
      </div>
    </div>

    <div v-for="item in miniCatalog" :key="item.id" class="setting-row">
      <div class="setting-copy setting-copy-icon">
        <span class="setting-glyph"><Icon :name="item.icon" :size="15" /></span>
        <span class="setting-copy-text">
          <b>{{ item.label }}</b>
          <span>Показывать в мини-плеере</span>
        </span>
      </div>
      <div
        class="setting-switch"
        :class="{ on: ui.settings.miniButtons[item.id] }"
        @click="ui.setMiniButton(item.id, !ui.settings.miniButtons[item.id])"
      >
        <i />
      </div>
    </div>

    <SettingToggle
      label="Время в мини-плеере"
      description="Текущая позиция и длительность по краям полосы прокрутки."
      :model-value="ui.settings.miniShowTime"
      @update:model-value="ui.set('miniShowTime', $event)"
    />

    <SettingToggle
      label="Визуализатор"
      description="Живой спектр звука на весь мини-плеер. Требует воспроизведения."
      :model-value="ui.settings.miniVisualizer"
      @update:model-value="ui.set('miniVisualizer', $event)"
    />

    <div class="setting-row">
      <div class="setting-copy">
        <b>Сбросить раскладку</b>
        <span
          >Вернуть кнопки плеера и мини-плеера к значениям по умолчанию.</span
        >
      </div>
      <button
        class="settings-reset-button"
        type="button"
        @click="ui.resetPlayerButtons()"
      >
        Сбросить
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue";
import Icon from "@/components/Icon.vue";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import type { MiniButtonId } from "@/stores/ui/index";
import {
  miniButtonCatalog,
  miniButtonOrder,
  useUiStore,
} from "@/stores/ui/index";

const lockedMini = new Set<MiniButtonId>(["prev", "play", "next"]);

const ui = useUiStore();

const miniCatalog = miniButtonCatalog.filter((item) => !item.locked);
const miniActive = computed(() =>
  miniButtonOrder.filter(
    (id) => lockedMini.has(id) || ui.settings.miniButtons[id],
  ),
);

const miniMetaOf = new Map(miniCatalog.map((item) => [item.id, item]));

function miniIconOf(id: MiniButtonId): string {
  return miniMetaOf.get(id)?.icon ?? "note";
}

function miniLabelOf(id: MiniButtonId): string {
  return miniMetaOf.get(id)?.label ?? id;
}
</script>
