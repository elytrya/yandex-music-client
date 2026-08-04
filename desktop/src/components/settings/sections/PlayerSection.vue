<template>
  <section id="settings-player" class="settings-group">
    <div class="settings-group-head">
      <h2>Плеер</h2>
      <p>
        Раскидай кнопки по трём зонам или скрой лишние - предпросмотр
        обновляется сразу.
      </p>
    </div>

    <div class="setting-row setting-row-column preview-row">
      <div class="setting-copy">
        <b>Предпросмотр плеера</b>
        <span>Так нижняя панель выглядит сейчас.</span>
      </div>

      <div class="player-preview">
        <div class="player-preview-left">
          <div class="player-preview-cover">
            <Icon name="note" :size="14" class="faint" />
          </div>
          <div class="player-preview-meta">
            <span class="player-preview-line w-60" />
            <span class="player-preview-line w-40 dimmed" />
          </div>
          <div class="player-preview-zone">
            <span
              v-for="id in leftButtons"
              :key="id"
              class="player-preview-btn"
              :title="labelOf(id)"
            >
              <Icon :name="iconOf(id)" :size="14" />
            </span>
          </div>
        </div>

        <div class="player-preview-center">
          <div class="player-preview-zone">
            <span
              v-for="id in centerButtons"
              :key="id"
              class="player-preview-btn"
              :class="{ accent: id === 'play' }"
              :title="labelOf(id)"
            >
              <Icon :name="iconOf(id)" :size="14" />
            </span>
          </div>
          <div class="player-preview-progress">
            <span class="player-preview-tick">0:42</span>
            <span class="player-preview-track"><i /></span>
            <span class="player-preview-tick">3:15</span>
          </div>
        </div>

        <div class="player-preview-right">
          <div class="player-preview-zone">
            <span
              v-for="id in rightButtons"
              :key="id"
              class="player-preview-btn"
              :title="labelOf(id)"
            >
              <Icon :name="iconOf(id)" :size="14" />
            </span>
            <span v-if="showVolumeSlider" class="player-preview-volume" />
          </div>
        </div>
      </div>

      <div class="player-preview-counts">
        <span>Слева: {{ leftButtons.length }}</span>
        <span>Центр: {{ centerButtons.length }}</span>
        <span>Справа: {{ rightButtons.length }}</span>
        <span v-if="crowded" class="warn">
          В одной зоне много кнопок - часть можно перенести.
        </span>
      </div>
    </div>

    <div v-for="item in catalog" :key="item.id" class="setting-row">
      <div class="setting-copy setting-copy-icon">
        <span class="setting-glyph"><Icon :name="item.icon" :size="15" /></span>
        <span class="setting-copy-text">
          <b>{{ item.label }}</b
          ><span>{{ item.hint }}</span>
        </span>
      </div>
      <div class="settings-choice">
        <button
          v-for="zone in zones"
          :key="zone.value"
          :class="{ on: zoneOf[item.id] === zone.value }"
          @click="ui.setPlayerZone(item.id, zone.value)"
        >
          {{ zone.label }}
        </button>
      </div>
    </div>

    <SettingToggle
      label="Визуализатор"
      description="Живой спектр звука фоном в нижней панели плеера. Требует воспроизведения."
      :model-value="ui.settings.playerVisualizer"
      @update:model-value="ui.set('playerVisualizer', $event)"
    />
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue";
import Icon from "@/components/Icon.vue";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import type { PlayerButtonId, PlayerZone } from "@/stores/ui/index";
import {
  playerButtonCatalog,
  playerButtonOrder,
  playerZoneLabels,
  useUiStore,
} from "@/stores/ui/index";

const ui = useUiStore();

const catalog = playerButtonCatalog.filter((item) => !item.locked);
const zones = playerZoneLabels;

const zoneOf = computed<Record<PlayerButtonId, PlayerZone>>(() => {
  const slots = ui.settings.playerButtons;
  const map = {} as Record<PlayerButtonId, PlayerZone>;
  for (const item of playerButtonCatalog) {
    map[item.id] = item.id === "play" ? "center" : (slots[item.id] ?? "off");
  }
  return map;
});

const leftButtons = computed(() =>
  playerButtonOrder.filter((id) => zoneOf.value[id] === "left"),
);
const centerButtons = computed(() =>
  playerButtonOrder.filter((id) => zoneOf.value[id] === "center"),
);
const rightButtons = computed(() =>
  playerButtonOrder.filter((id) => zoneOf.value[id] === "right"),
);

const showVolumeSlider = computed(() => rightButtons.value.includes("volume"));

const crowded = computed(
  () =>
    Math.max(
      leftButtons.value.length,
      centerButtons.value.length,
      rightButtons.value.length,
    ) > 6,
);

const metaOf = new Map(catalog.map((item) => [item.id, item]));

function iconOf(id: PlayerButtonId): string {
  return metaOf.get(id)?.icon ?? "note";
}

function labelOf(id: PlayerButtonId): string {
  return metaOf.get(id)?.label ?? id;
}
</script>
