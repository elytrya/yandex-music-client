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

    <div class="setting-row setting-row-column">
      <div class="setting-copy">
        <b>Раскладка кнопок</b>
        <span>
          Перетаскивай кнопки мышкой между зонами и меняй их порядок внутри
          зоны. Кнопками ниже — тоже работает.
        </span>
      </div>

      <div class="dnd-zones">
        <div
          v-for="zone in dndZones"
          :key="zone.value"
          class="dnd-zone"
          :class="{ over: dragOverZone === zone.value }"
          @dragover.prevent="dragOverZone = zone.value"
          @dragleave="onZoneLeave(zone.value)"
          @drop.prevent="dropInZone(zone.value, zoneItems(zone.value).length)"
        >
          <div class="dnd-zone-head">
            <span>{{ zone.label }}</span>
            <span class="faint">{{ zoneItems(zone.value).length }}</span>
          </div>

          <div class="dnd-list">
            <div
              v-for="(id, index) in zoneItems(zone.value)"
              :key="id"
              class="dnd-chip"
              :class="{ locked: id === 'play', dragging: dragId === id }"
              :draggable="id !== 'play'"
              :title="labelOf(id)"
              @dragstart="onDragStart(id, $event)"
              @dragend="onDragEnd"
              @dragover.prevent.stop="dragOverZone = zone.value"
              @drop.prevent.stop="dropInZone(zone.value, index)"
            >
              <Icon name="drag" :size="12" class="faint" />
              <Icon :name="iconOf(id)" :size="14" />
              <span class="ellipsis">{{ labelOf(id) }}</span>
            </div>

            <div v-if="!zoneItems(zone.value).length" class="dnd-empty faint">
              Перетащи сюда
            </div>
          </div>
        </div>
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

    <SettingToggle
      label="Показывать тайминги"
      description="Текущее время и длительность по краям полосы прогресса."
      :model-value="ui.settings.playerShowTimes"
      @update:model-value="ui.set('playerShowTimes', $event)"
    />

    <SettingSlider
      label="Высота панели"
      description="Общая высота нижнего плеера."
      :model-value="ui.settings.playerHeight"
      :min="56"
      :max="120"
      :step="1"
      suffix=" px"
      @update:model-value="ui.set('playerHeight', $event)"
    />

    <SettingSlider
      label="Размер обложки"
      description="Квадрат с обложкой слева."
      :model-value="ui.settings.playerCoverSize"
      :min="28"
      :max="84"
      :step="1"
      suffix=" px"
      @update:model-value="ui.set('playerCoverSize', $event)"
    />

    <SettingSlider
      label="Размер иконок"
      description="Величина значков на кнопках управления."
      :model-value="ui.settings.playerIconSize"
      :min="12"
      :max="28"
      :step="1"
      suffix=" px"
      @update:model-value="ui.set('playerIconSize', $event)"
    />

    <SettingSlider
      label="Расстояние между кнопками"
      description="Промежутки в зонах управления."
      :model-value="ui.settings.playerGap"
      :min="0"
      :max="20"
      :step="1"
      suffix=" px"
      @update:model-value="ui.set('playerGap', $event)"
    />

    <SettingSlider
      label="Отступы по краям"
      description="Поля слева и справа от содержимого плеера."
      :model-value="ui.settings.playerSidePadding"
      :min="0"
      :max="48"
      :step="1"
      suffix=" px"
      @update:model-value="ui.set('playerSidePadding', $event)"
    />

    <SettingSlider
      label="Ширина боковых зон"
      description="Сколько места отдаётся левому и правому блокам."
      :model-value="ui.settings.playerMetaWidth"
      :min="160"
      :max="420"
      :step="4"
      suffix=" px"
      @update:model-value="ui.set('playerMetaWidth', $event)"
    />

    <SettingSlider
      label="Ширина полосы прогресса"
      description="Доля центральной зоны, занятая перемоткой."
      :model-value="ui.settings.playerProgressWidth"
      :min="40"
      :max="100"
      :step="1"
      suffix=" %"
      @update:model-value="ui.set('playerProgressWidth', $event)"
    />

    <SettingSlider
      label="Толщина полосы прогресса"
      description="Высота линии перемотки."
      :model-value="ui.settings.playerProgressThickness"
      :min="2"
      :max="12"
      :step="1"
      suffix=" px"
      @update:model-value="ui.set('playerProgressThickness', $event)"
    />

    <div class="setting-row">
      <div class="setting-copy">
        <b>Сбросить размеры плеера</b>
        <span>Вернёт высоту, отступы и пропорции к исходным.</span>
      </div>
      <button class="btn" type="button" @click="ui.resetPlayerLayout()">
        Сбросить
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import Icon from "@/components/Icon.vue";
import SettingSlider from "@/components/settings/SettingSlider.vue";
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

/* --- Drag & drop раскладка --- */

const dndZones = zones
  .filter((z) => z.value !== "off")
  .concat(zones.filter((z) => z.value === "off"));

const dragId = ref<PlayerButtonId | null>(null);
const dragOverZone = ref<PlayerZone | null>(null);

function zoneItems(zone: PlayerZone): PlayerButtonId[] {
  return ui.playerOrderList().filter((id) => zoneOf.value[id] === zone);
}

function onDragStart(id: PlayerButtonId, event: DragEvent) {
  if (id === "play") {
    event.preventDefault();
    return;
  }
  dragId.value = id;
  // WebView2 не начинает перетаскивание без данных в dataTransfer.
  event.dataTransfer?.setData("text/plain", id);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
}

function onDragEnd() {
  dragId.value = null;
  dragOverZone.value = null;
}

function onZoneLeave(zone: PlayerZone) {
  if (dragOverZone.value === zone) dragOverZone.value = null;
}

function dropInZone(zone: PlayerZone, index: number) {
  const id = dragId.value;
  onDragEnd();
  if (!id) return;
  ui.movePlayerButton(id, zone, index);
}

const metaOf = new Map(playerButtonCatalog.map((item) => [item.id, item]));

function iconOf(id: PlayerButtonId): string {
  return metaOf.get(id)?.icon ?? "note";
}

function labelOf(id: PlayerButtonId): string {
  return metaOf.get(id)?.label ?? id;
}
</script>

<style scoped>
.dnd-zones {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 10px;
  width: 100%;
}
.dnd-zone {
  padding: 10px;
  border: 1px dashed var(--line);
  border-radius: 10px;
  background: var(--surface-2);
  transition:
    border-color 0.14s ease,
    background 0.14s ease;
}
.dnd-zone.over {
  border-color: var(--accent);
  background: var(--hover);
}
.dnd-zone-head {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.03em;
  text-transform: uppercase;
  color: var(--fg-dim);
}
.dnd-list {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-height: 44px;
}
.dnd-chip {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 6px 9px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--surface);
  font-size: 12px;
  cursor: grab;
}
.dnd-chip.dragging {
  opacity: 0.45;
}
.dnd-chip.locked {
  cursor: default;
  opacity: 0.75;
}
.dnd-empty {
  padding: 10px 4px;
  font-size: 11px;
  text-align: center;
}
</style>
