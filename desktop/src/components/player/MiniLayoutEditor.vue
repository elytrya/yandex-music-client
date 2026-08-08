<template>
  <div class="mpe">
    <div class="mpe-preview">
      <div class="mpe-preview-head">
        <span class="mpe-preview-title">Предпросмотр</span>
        <span class="mpe-preview-size">
          {{ s.miniWidth }} × {{ s.miniHeight }} px
          <template v-if="scale < 0.999">
            · масштаб {{ Math.round(scale * 100) }}%
          </template>
        </span>
      </div>

      <div ref="stageWrap" class="mpe-stage-wrap">
        <div class="mpe-stage" :style="stageStyle">
          <div class="mpe-frame" :style="frameStyle">
            <MiniPlayer preview />
          </div>
        </div>
      </div>

      <p class="mpe-note">
        Это настоящий мини-плеер с текущим треком — кнопки в предпросмотре не
        нажимаются, чтобы ничего не задеть случайно.
      </p>

      <div class="mpe-preview-actions">
        <button class="btn accent" type="button" @click="openMini">
          <Icon name="mini" :size="15" />
          {{ panels.mini ? "Мини-плеер открыт" : "Открыть мини-плеер" }}
        </button>
        <button class="settings-reset-button" type="button" @click="reset">
          Сбросить вид
        </button>
      </div>
    </div>

    <div class="mpe-controls">
      <div class="mpe-group">
        <div class="mpe-group-head">Размер окна</div>
        <div class="mpe-grid">
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
        </div>
      </div>

      <div class="mpe-group">
        <div class="mpe-group-head">Размеры и отступы</div>
        <div class="mpe-grid">
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
            label="Поля"
            suffix=" px"
            :min="2"
            :max="26"
            :model-value="s.miniPadding"
            @update:model-value="ui.set('miniPadding', $event)"
          />
          <EditRange
            label="Непрозрачность"
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
        </div>
      </div>

      <div class="mpe-group">
        <div class="mpe-group-head">Показывать</div>
        <div class="mpe-switches">
          <EditSwitch
            label="Время трека"
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
        </div>
      </div>

      <div class="mpe-group">
        <div class="mpe-group-head">
          Кнопки
          <span class="mpe-group-hint">перетаскивай, чтобы менять порядок</span>
        </div>

        <div
          ref="listRoot"
          class="mpe-list"
          @dragover.prevent="onListOver"
          @dragleave="onListLeave"
          @drop.prevent="onListDrop"
        >
          <template v-for="(id, index) in order" :key="id">
            <i v-if="caretAt(index)" class="mpe-caret" />
            <div
              class="mpe-item"
              :class="{ off: !visible(id), dragging: dragId === id }"
              draggable="true"
              @dragstart="onDragStart(id, $event)"
              @dragend="endDrag"
            >
              <Icon name="drag" :size="13" class="faint mpe-item-grip" />
              <Icon :name="iconOf(id)" :size="15" />
              <span class="mpe-item-label ellipsis">{{ labelOf(id) }}</span>
              <span v-if="locked.has(id)" class="mpe-item-lock">всегда</span>
              <button
                v-else
                type="button"
                class="mpe-eye"
                :class="{ on: visible(id) }"
                :title="visible(id) ? 'Скрыть кнопку' : 'Показать кнопку'"
                @click.stop="toggle(id)"
              >
                <Icon :name="visible(id) ? 'check' : 'close'" :size="13" />
              </button>
            </div>
          </template>
          <i v-if="caretAt(order.length)" class="mpe-caret" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import Icon from "@/components/Icon.vue";
import MiniPlayer from "@/components/MiniPlayer.vue";
import EditRange from "@/components/player/EditRange.vue";
import EditSwitch from "@/components/player/EditSwitch.vue";
import { api } from "@/api/client";
import { usePanelsStore } from "@/stores/panels";
import { miniButtonCatalog } from "@/stores/ui/playerButtons";
import type { MiniButtonId } from "@/stores/ui/playerButtons";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();
const panels = usePanelsStore();
const s = computed(() => ui.settings);

const locked = new Set<MiniButtonId>(["prev", "play", "next"]);
const order = computed(() => ui.miniOrderList());

const stageWrap = ref<HTMLElement | null>(null);
const wrapWidth = ref(0);
let observer: ResizeObserver | null = null;

const scale = computed(() => {
  if (!wrapWidth.value) return 1;
  return Math.min(1, wrapWidth.value / s.value.miniWidth);
});

const stageStyle = computed(() => ({
  width: `${Math.round(s.value.miniWidth * scale.value)}px`,
  height: `${Math.round(s.value.miniHeight * scale.value)}px`,
}));

const frameStyle = computed(() => ({
  width: `${s.value.miniWidth}px`,
  height: `${s.value.miniHeight}px`,
  transform: `scale(${scale.value})`,
  transformOrigin: "top left",
}));

onMounted(() => {
  const el = stageWrap.value;
  if (!el) return;
  wrapWidth.value = el.clientWidth;
  observer = new ResizeObserver((entries) => {
    wrapWidth.value = entries[0]?.contentRect.width ?? el.clientWidth;
  });
  observer.observe(el);
});

onBeforeUnmount(() => {
  observer?.disconnect();
  observer = null;
});

function applySize() {
  if (!panels.mini) return;
  void api.resizeMiniPlayer(ui.settings.miniWidth, ui.settings.miniHeight);
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

async function openMini() {
  if (panels.mini) return;
  await panels.enterMini();
}

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

const listRoot = ref<HTMLElement | null>(null);
const dragId = ref<MiniButtonId | null>(null);
const overIndex = ref(-1);

function caretAt(index: number): boolean {
  return dragId.value !== null && overIndex.value === index;
}

function slotAt(clientY: number): number {
  const root = listRoot.value;
  if (!root) return order.value.length;
  const items = Array.from(root.querySelectorAll<HTMLElement>(".mpe-item"));
  let slot = 0;
  for (const el of items) {
    const box = el.getBoundingClientRect();
    if (clientY >= box.top + box.height / 2) slot += 1;
  }
  return slot;
}

function onDragStart(id: MiniButtonId, event: DragEvent) {
  dragId.value = id;
  event.dataTransfer?.setData("text/plain", id);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
}

function endDrag() {
  dragId.value = null;
  overIndex.value = -1;
}

function onListOver(event: DragEvent) {
  if (!dragId.value) return;
  if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
  overIndex.value = slotAt(event.clientY);
}

function onListLeave(event: DragEvent) {
  const root = listRoot.value;
  const to = event.relatedTarget as Node | null;
  if (root && to && root.contains(to)) return;
  overIndex.value = -1;
}

function onListDrop(event: DragEvent) {
  const dropped = event.dataTransfer?.getData("text/plain") as
    MiniButtonId | undefined;
  const id = dragId.value ?? (dropped || null);
  const slot = slotAt(event.clientY);
  endDrag();
  if (!id) return;

  const from = order.value.indexOf(id);
  ui.moveMiniButton(id, from >= 0 && from < slot ? slot - 1 : slot);
}
</script>

<style scoped>
.mpe {
  display: grid;
  width: 100%;
  grid-template-columns: minmax(280px, 1fr) minmax(260px, 1fr);
  gap: 18px;
  align-items: start;
}

@media (max-width: 980px) {
  .mpe {
    grid-template-columns: 1fr;
  }
}

.mpe-preview {
  position: sticky;
  top: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  border: 1px solid var(--line);
  border-radius: calc(var(--radius) * 0.9);
  background: var(--surface);
}

.mpe-preview-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
}

.mpe-preview-title {
  font-size: 12.5px;
  font-weight: 640;
}

.mpe-preview-size {
  color: var(--fg-faint);
  font-size: 11.5px;
  font-variant-numeric: tabular-nums;
}

.mpe-stage-wrap {
  display: flex;
  justify-content: center;
  width: 100%;
  padding: 14px;
  border-radius: calc(var(--radius) * 0.8);
  background: repeating-conic-gradient(
      color-mix(in srgb, var(--fg) 5%, transparent) 0% 25%,
      transparent 0% 50%
    )
    50% / 18px 18px;
}

.mpe-stage {
  position: relative;
  overflow: hidden;
  transition:
    width 0.12s ease,
    height 0.12s ease;
}

.mpe-frame {
  position: absolute;
  top: 0;
  left: 0;
  border-radius: 14px;
  box-shadow: 0 12px 34px rgb(0 0 0 / 32%);
}

.mpe-note {
  margin: 0;
  color: var(--fg-faint);
  font-size: 11.5px;
  line-height: 1.45;
}

.mpe-preview-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.mpe-preview-actions .btn {
  display: inline-flex;
  align-items: center;
  gap: 7px;
}

.mpe-preview-actions .btn.accent {
  border-color: var(--accent);
  background: var(--accent);
  color: #fff;
}

.mpe-controls {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-width: 0;
}

.mpe-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  border: 1px solid var(--line);
  border-radius: calc(var(--radius) * 0.9);
  background: var(--surface);
}

.mpe-group-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
  color: var(--fg-faint);
  font-size: 10px;
  font-weight: 650;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.mpe-group-hint {
  font-weight: 500;
  letter-spacing: 0;
  text-transform: none;
}

.mpe-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px 18px;
}

.mpe-switches {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.mpe-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-height: 40px;
}

.mpe-item {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 8px 10px;
  border: 1px solid transparent;
  border-radius: 10px;
  background: var(--surface-2);
  font-size: 12.5px;
  cursor: grab;
  transition:
    border-color 0.14s ease,
    opacity 0.14s ease;
}

.mpe-item:hover {
  border-color: var(--line);
}

.mpe-item.off {
  opacity: 0.45;
}

.mpe-item.dragging {
  opacity: 0.3;
}

.mpe-item-grip {
  cursor: grab;
}

.mpe-item-label {
  flex: 1 1 auto;
  min-width: 0;
}

.mpe-item-lock {
  color: var(--fg-faint);
  font-size: 10.5px;
}

.mpe-eye {
  display: inline-flex;
  width: 24px;
  height: 24px;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: var(--fg-dim);
  cursor: pointer;
  transition:
    background 0.14s ease,
    color 0.14s ease;
}

.mpe-eye:hover {
  background: var(--hover);
  color: var(--fg);
}

.mpe-eye.on {
  color: var(--accent);
}

.mpe-caret {
  height: 2px;
  flex: 0 0 2px;
  border-radius: 2px;
  background: var(--accent);
  box-shadow: 0 0 6px color-mix(in srgb, var(--accent) 55%, transparent);
  pointer-events: none;
}
</style>
