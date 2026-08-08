<template>
  <div class="pe-panel" @contextmenu.stop.prevent @pointerdown.stop>
    <div class="pe-panel-head">
      <div class="pe-panel-copy">
        <div class="pe-panel-title">Редактор плеера</div>
        <div class="pe-panel-sub">
          Тяни кнопки прямо на плеере, края меняют размер. Всё видно сразу.
        </div>
      </div>
      <div class="pe-panel-actions">
        <button type="button" class="pe-act" @click="ui.resetPlayerLayout()">
          Сбросить
        </button>
        <button type="button" class="pe-act" @click="ui.cancelEdit()">
          Отмена
        </button>
        <button type="button" class="pe-act accent" @click="ui.finishEdit()">
          Готово
        </button>
      </div>
    </div>

    <div class="pe-panel-body">
      <div class="pe-col">
        <div class="pe-col-head">Размеры</div>
        <EditRange
          label="Высота плеера"
          suffix=" px"
          :min="60"
          :max="140"
          :model-value="s.playerHeight"
          @update:model-value="ui.set('playerHeight', $event)"
        />
        <EditRange
          label="Обложка"
          suffix=" px"
          :min="28"
          :max="80"
          :model-value="s.playerCoverSize"
          @update:model-value="ui.set('playerCoverSize', $event)"
        />
        <EditRange
          label="Иконки"
          suffix=" px"
          :min="12"
          :max="28"
          :model-value="s.playerIconSize"
          @update:model-value="ui.set('playerIconSize', $event)"
        />
        <EditRange
          label="Промежутки"
          suffix=" px"
          :min="0"
          :max="20"
          :model-value="s.playerGap"
          @update:model-value="ui.set('playerGap', $event)"
        />
      </div>

      <div class="pe-col">
        <div class="pe-col-head">Геометрия</div>
        <EditRange
          label="Поля по краям"
          suffix=" px"
          :min="0"
          :max="48"
          :model-value="s.playerSidePadding"
          @update:model-value="ui.set('playerSidePadding', $event)"
        />
        <EditRange
          label="Блок с треком"
          suffix=" px"
          :min="150"
          :max="460"
          :model-value="s.playerMetaWidth"
          @update:model-value="ui.set('playerMetaWidth', $event)"
        />
        <EditRange
          label="Ширина прогресса"
          suffix="%"
          :min="40"
          :max="100"
          :model-value="s.playerProgressWidth"
          @update:model-value="ui.set('playerProgressWidth', $event)"
        />
        <EditRange
          label="Толщина прогресса"
          suffix=" px"
          :min="2"
          :max="10"
          :model-value="s.playerProgressThickness"
          @update:model-value="ui.set('playerProgressThickness', $event)"
        />
      </div>

      <div class="pe-col">
        <div class="pe-col-head">На плеере</div>
        <EditSwitch
          label="Время трека"
          :model-value="s.playerShowTimes"
          @update:model-value="ui.set('playerShowTimes', $event)"
        />
        <EditSwitch
          label="Обложка трека"
          :model-value="s.showPlayerArtwork"
          @update:model-value="ui.set('showPlayerArtwork', $event)"
        />
        <EditSwitch
          label="Визуализатор"
          :model-value="s.playerVisualizer"
          @update:model-value="ui.set('playerVisualizer', $event)"
        />
      </div>

      <div class="pe-col pe-col-wide">
        <div class="pe-col-head">Скрытые кнопки</div>
        <div
          class="pe-tray"
          :class="{ over: trayOver }"
          @dragover.prevent="trayOver = true"
          @dragleave="trayOver = false"
          @drop.prevent="onDropHide"
        >
          <span
            v-for="id in hidden"
            :key="id"
            class="pe-chip"
            draggable="true"
            @dragstart="onDragStart(id, $event)"
          >
            <Icon :name="iconOf(id)" :size="13" />
            {{ labelOf(id) }}
          </span>
          <span v-if="!hidden.length" class="pe-tray-empty">
            Перетащи сюда кнопку с плеера, чтобы убрать её
          </span>
        </div>
        <div class="pe-tray-hint">
          Обратно — просто перетащи чип в нужную часть плеера.
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import Icon from "@/components/Icon.vue";
import EditRange from "@/components/player/EditRange.vue";
import EditSwitch from "@/components/player/EditSwitch.vue";
import { playerButtonCatalog } from "@/stores/ui/playerButtons";
import type { PlayerButtonId } from "@/stores/ui/playerButtons";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();
const s = computed(() => ui.settings);
const trayOver = ref(false);

const hidden = computed(() =>
  ui.playerOrderList().filter((id) => ui.playerZone(id) === "off"),
);

function labelOf(id: PlayerButtonId): string {
  return playerButtonCatalog.find((item) => item.id === id)?.label ?? id;
}

function iconOf(id: PlayerButtonId): string {
  return playerButtonCatalog.find((item) => item.id === id)?.icon ?? "drag";
}

function onDragStart(id: PlayerButtonId, event: DragEvent) {
  event.dataTransfer?.setData("text/plain", id);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
}

function onDropHide(event: DragEvent) {
  trayOver.value = false;
  const id = event.dataTransfer?.getData("text/plain") as PlayerButtonId | "";
  if (!id) return;
  ui.movePlayerButton(id, "off", 0);
}

function onKeydown(event: KeyboardEvent) {
  if (event.key !== "Escape" || !ui.settings.playerEditMode) return;
  event.preventDefault();
  event.stopPropagation();
  ui.cancelEdit();
}

onMounted(() => window.addEventListener("keydown", onKeydown, true));
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown, true));
</script>

<style scoped>
.pe-panel {
  position: absolute;
  bottom: calc(100% + 10px);
  left: 50%;
  z-index: 30;
  width: min(940px, calc(100vw - 32px));
  padding: 12px 14px 14px;
  border: 1px solid var(--line);
  border-radius: 14px;
  background: var(--surface-2);
  box-shadow: var(--shadow-strong, 0 22px 60px rgba(0, 0, 0, 0.5));
  transform: translateX(-50%);
  animation: pe-panel-in 0.16s ease;
}

@keyframes pe-panel-in {
  from {
    opacity: 0;
    transform: translate(-50%, 8px);
  }
  to {
    opacity: 1;
    transform: translate(-50%, 0);
  }
}

.pe-panel-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 14px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--line);
}

.pe-panel-title {
  font-size: 13px;
  font-weight: 620;
}

.pe-panel-sub {
  margin-top: 2px;
  color: var(--fg-dim);
  font-size: 11.5px;
}

.pe-panel-actions {
  display: flex;
  flex: 0 0 auto;
  gap: 6px;
}

.pe-act {
  padding: 5px 11px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: transparent;
  color: var(--fg);
  font: inherit;
  font-size: 11.5px;
  cursor: pointer;
  transition: background 0.14s ease;
}

.pe-act:hover {
  background: var(--hover);
}

.pe-act.accent {
  border-color: var(--accent);
  background: var(--accent);
  color: #fff;
}

.pe-panel-body {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px 20px;
  padding-top: 12px;
}

.pe-col-head {
  margin-bottom: 8px;
  color: var(--fg-faint);
  font-size: 10px;
  font-weight: 620;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.pe-col :deep(.ed-range) {
  margin-bottom: 9px;
}

.pe-tray {
  display: flex;
  align-items: flex-start;
  flex-wrap: wrap;
  gap: 5px;
  min-height: 74px;
  padding: 8px;
  border: 1px dashed color-mix(in srgb, var(--accent) 40%, transparent);
  border-radius: 10px;
  transition: background 0.14s ease;
}

.pe-tray.over {
  background: color-mix(in srgb, var(--accent) 12%, transparent);
}

.pe-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 9px;
  border: 1px solid var(--line);
  border-radius: 999px;
  background: var(--surface);
  color: var(--fg-dim);
  font-size: 11px;
  cursor: grab;
}

.pe-chip:hover {
  color: var(--fg);
}

.pe-tray-empty {
  color: var(--fg-faint);
  font-size: 11px;
  line-height: 1.4;
}

.pe-tray-hint {
  margin-top: 6px;
  color: var(--fg-faint);
  font-size: 10.5px;
  line-height: 1.4;
}

@media (max-width: 1100px) {
  .pe-panel-body {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
