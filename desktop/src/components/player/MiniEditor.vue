<template>
  <div class="me-sheet" @contextmenu.stop.prevent @mousedown.stop>
    <div class="me-head">
      <span class="me-title">Вид мини-плеера</span>
      <div class="me-head-actions">
        <button type="button" class="me-act" @click="reset">Сброс</button>
        <button type="button" class="me-act" @click="cancel">Отмена</button>
        <button type="button" class="me-act accent" @click="ui.finishEdit()">
          Готово
        </button>
      </div>
    </div>

    <div class="me-body">
      <div class="me-group-head">Окно</div>
      <EditRange
        label="Ширина"
        suffix=" px"
        :min="300"
        :max="720"
        :model-value="s.miniWidth"
        @update:model-value="setWidth"
      />
      <EditRange
        label="Высота"
        suffix=" px"
        :min="120"
        :max="420"
        :model-value="s.miniHeight"
        @update:model-value="setHeight"
      />

      <div class="me-group-head">Размеры</div>
      <EditRange
        label="Обложка"
        suffix=" px"
        :min="24"
        :max="72"
        :model-value="s.miniCoverSize"
        @update:model-value="ui.set('miniCoverSize', $event)"
      />
      <EditRange
        label="Иконки"
        suffix=" px"
        :min="11"
        :max="24"
        :model-value="s.miniIconSize"
        @update:model-value="ui.set('miniIconSize', $event)"
      />
      <EditRange
        label="Промежутки"
        suffix=" px"
        :min="0"
        :max="18"
        :model-value="s.miniGap"
        @update:model-value="ui.set('miniGap', $event)"
      />
      <EditRange
        label="Отступы"
        suffix=" px"
        :min="2"
        :max="26"
        :model-value="s.miniPadding"
        @update:model-value="ui.set('miniPadding', $event)"
      />
      <EditRange
        label="Прозрачность"
        suffix="%"
        :min="40"
        :max="100"
        :model-value="s.miniOpacity"
        @update:model-value="ui.set('miniOpacity', $event)"
      />
      <EditRange
        v-if="s.miniVolumeSlider"
        label="Дорожка громкости"
        suffix=" px"
        :min="24"
        :max="96"
        :model-value="s.miniVolumeHeight"
        @update:model-value="ui.set('miniVolumeHeight', $event)"
      />

      <div class="me-group-head">Показывать</div>
      <EditSwitch
        label="Время"
        :model-value="s.miniShowTime"
        @update:model-value="ui.set('miniShowTime', $event)"
      />
      <EditSwitch
        label="Визуализатор"
        :model-value="s.miniVisualizer"
        @update:model-value="ui.set('miniVisualizer', $event)"
      />
      <EditSwitch
        label="Ползунок громкости"
        :model-value="s.miniVolumeSlider"
        @update:model-value="ui.set('miniVolumeSlider', $event)"
      />

      <div class="me-group-head">Кнопки — тяни за порядок</div>
      <div class="me-list">
        <div
          v-for="(id, index) in order"
          :key="id"
          class="me-item"
          :class="{ off: !visible(id), dragging: dragId === id }"
          draggable="true"
          @dragstart="onDragStart(id, $event)"
          @dragend="dragId = null"
          @dragover.prevent
          @drop.prevent="onDrop(index)"
        >
          <Icon name="drag" :size="11" class="faint" />
          <Icon :name="iconOf(id)" :size="13" />
          <span class="me-item-label ellipsis">{{ labelOf(id) }}</span>
          <button
            type="button"
            class="me-eye"
            :disabled="locked.has(id)"
            :title="locked.has(id) ? 'Всегда на месте' : 'Показать или скрыть'"
            @click.stop="toggle(id)"
          >
            <Icon :name="visible(id) ? 'check' : 'close'" :size="12" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import Icon from "@/components/Icon.vue";
import EditRange from "@/components/player/EditRange.vue";
import EditSwitch from "@/components/player/EditSwitch.vue";
import { setWindowSize } from "@/lib/window";
import { miniButtonCatalog } from "@/stores/ui/playerButtons";
import type { MiniButtonId } from "@/stores/ui/playerButtons";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();
const s = computed(() => ui.settings);
const dragId = ref<MiniButtonId | null>(null);
const locked = new Set<MiniButtonId>(["prev", "play", "next"]);

const order = computed(() => ui.miniOrderList());

function labelOf(id: MiniButtonId): string {
  return miniButtonCatalog.find((item) => item.id === id)?.label ?? id;
}

function iconOf(id: MiniButtonId): string {
  return miniButtonCatalog.find((item) => item.id === id)?.icon ?? "drag";
}

function visible(id: MiniButtonId): boolean {
  return locked.has(id) || ui.settings.miniButtons[id];
}

function toggle(id: MiniButtonId) {
  if (locked.has(id)) return;
  ui.setMiniButton(id, !ui.settings.miniButtons[id]);
}

function onDragStart(id: MiniButtonId, event: DragEvent) {
  dragId.value = id;
  event.dataTransfer?.setData("text/plain", id);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
}

function onDrop(index: number) {
  const id = dragId.value;
  dragId.value = null;
  if (id) ui.moveMiniButton(id, index);
}

/* размер окна меняется сразу, чтобы результат был виден */
function applySize() {
  void setWindowSize(ui.settings.miniWidth, ui.settings.miniHeight);
}

function setWidth(value: number) {
  ui.set("miniWidth", value);
  applySize();
}

function setHeight(value: number) {
  ui.set("miniHeight", value);
  applySize();
}

function reset() {
  ui.resetMiniLayout();
  applySize();
}

function cancel() {
  ui.cancelEdit();
  applySize();
}
</script>

<style scoped>
.me-sheet {
  position: absolute;
  z-index: 40;
  inset: 0;
  display: flex;
  flex-direction: column;
  background: color-mix(in srgb, var(--surface) 94%, transparent);
  backdrop-filter: blur(20px);
}

.me-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex: 0 0 auto;
  gap: 8px;
  padding: 7px 9px;
  border-bottom: 1px solid var(--line);
}

.me-title {
  font-size: 11.5px;
  font-weight: 620;
}

.me-head-actions {
  display: flex;
  gap: 4px;
}

.me-act {
  padding: 3px 8px;
  border: 1px solid var(--line);
  border-radius: 7px;
  background: transparent;
  color: var(--fg);
  font: inherit;
  font-size: 10.5px;
  cursor: pointer;
}

.me-act:hover {
  background: var(--hover);
}

.me-act.accent {
  border-color: var(--accent);
  background: var(--accent);
  color: #fff;
}

.me-body {
  flex: 1 1 auto;
  overflow: hidden auto;
  padding: 8px 10px 12px;
}

.me-group-head {
  margin: 8px 0 6px;
  color: var(--fg-faint);
  font-size: 9.5px;
  font-weight: 620;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.me-group-head:first-child {
  margin-top: 0;
}

.me-body :deep(.ed-range) {
  margin-bottom: 8px;
}

.me-list {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.me-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  border: 1px solid transparent;
  border-radius: 8px;
  background: var(--surface-2);
  font-size: 11px;
  cursor: grab;
}

.me-item:hover {
  border-color: var(--line);
}

.me-item.off {
  opacity: 0.45;
}

.me-item.dragging {
  opacity: 0.35;
}

.me-item-label {
  flex: 1 1 auto;
  min-width: 0;
}

.me-eye {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  flex: 0 0 auto;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--fg-dim);
  cursor: pointer;
}

.me-eye:hover:not(:disabled) {
  background: var(--hover);
  color: var(--fg);
}

.me-eye:disabled {
  cursor: default;
  opacity: 0.35;
}
</style>
