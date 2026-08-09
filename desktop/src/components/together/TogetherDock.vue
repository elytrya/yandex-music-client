<template>
  <div
    v-if="together.active"
    ref="root"
    class="together-dock"
    :class="{ open, dragging }"
    :style="style"
  >
    <div
      class="together-dock-head"
      @pointerdown="onDown"
      @pointermove="onMove"
      @pointerup="onUp"
      @pointercancel="onCancel"
    >
      <Icon class="together-dock-grip" name="drag" :size="12" />

      <span class="together-dock-dot" :class="{ wait: waiting }" />

      <span class="together-dock-title">
        {{ together.isHost ? "Вы ведёте" : `Слушаете: ${together.hostNick}` }}
      </span>

      <span class="together-dock-count">{{ together.peers.length }}</span>
      <Icon :name="open ? 'minimize' : 'expand'" :size="12" />
    </div>

    <div v-if="open" class="together-dock-body">
      <div v-if="together.isHost" class="together-dock-row">
        <span class="together-dock-label">Адрес</span>

        <code
          class="together-dock-code"
          :title="
            visible ? 'Нажмите, чтобы скопировать' : 'Наведите, чтобы увидеть'
          "
          @mouseenter="hover = true"
          @mouseleave="hover = false"
          @click="copyInvite"
          >{{ visible ? invite : mask }}</code
        >

        <button
          class="together-dock-icon"
          type="button"
          :title="shown ? 'Скрыть адрес' : 'Показать адрес'"
          @click="shown = !shown"
        >
          <Icon :name="shown ? 'eyeOff' : 'eye'" :size="12" />
        </button>

        <button
          class="together-dock-icon"
          type="button"
          title="Скопировать адрес"
          @click="copyInvite"
        >
          <Icon name="copy" :size="12" />
        </button>
      </div>

      <div class="together-dock-row">
        <span class="together-dock-label">Сейчас</span>
        <span class="together-dock-value">{{ nowPlaying }}</span>
      </div>

      <TogetherPeers
        :peers="together.peers"
        :waiting="together.waiting"
        :controllers="together.controllers"
        :manage="together.isHost"
        @grant="together.grant"
      />

      <p v-if="together.waitingNicks.length" class="together-dock-wait">
        Ждём загрузку: {{ together.waitingNicks.join(", ") }}
      </p>

      <p v-else-if="!together.isHost" class="together-dock-hint">
        {{
          together.rights ? "У вас есть управление комнатой" : "Управляет хост"
        }}
      </p>

      <div class="together-dock-actions">
        <button type="button" @click="openSettings">Настройки</button>
        <button
          type="button"
          class="danger"
          :disabled="together.busy"
          @click="together.leave()"
        >
          {{ together.isHost ? "Закрыть" : "Выйти" }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import Icon from "@/components/Icon.vue";
import TogetherPeers from "@/components/together/TogetherPeers.vue";
import { copyText } from "@/lib/clipboard";
import { clampPlace, loadPlace, savePlace } from "@/lib/dock-place";
import { usePlayerStore } from "@/stores/player/index";
import { useTogetherStore } from "@/stores/together/index";
import { DOCK_KEY, DOCK_POS_KEY } from "@/stores/together/protocol";

const router = useRouter();
const together = useTogetherStore();
const player = usePlayerStore();

const root = ref<HTMLElement | null>(null);
const open = ref(false);
const shown = ref(false);
const hover = ref(false);
const dragging = ref(false);
const place = ref(loadPlace(DOCK_POS_KEY));

let from = { x: 0, y: 0, left: 0, top: 0 };
let moved = false;

const mask = "•".repeat(12);
const waiting = computed(() => together.waiting.length > 0);
const visible = computed(() => shown.value || hover.value);
const invite = computed(() => together.invite || `ваш-ip:${together.port}`);

const nowPlaying = computed(() =>
  player.current ? player.current.title : "ничего не играет",
);

const style = computed(() =>
  place.value
    ? {
        left: `${place.value.x}px`,
        top: `${place.value.y}px`,
        right: "auto",
        bottom: "auto",
      }
    : undefined,
);

onMounted(() => {
  void together.init();
  try {
    open.value = localStorage.getItem(DOCK_KEY) === "1";
  } catch {}
  window.addEventListener("resize", keepInside);
});

onBeforeUnmount(() => window.removeEventListener("resize", keepInside));

function toggle() {
  open.value = !open.value;
  try {
    localStorage.setItem(DOCK_KEY, open.value ? "1" : "0");
  } catch {}
}

function onDown(event: PointerEvent) {
  if (event.button !== 0) return;
  const box = root.value?.getBoundingClientRect();
  if (!box) return;

  dragging.value = true;
  moved = false;
  from = { x: event.clientX, y: event.clientY, left: box.left, top: box.top };
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
}

function onMove(event: PointerEvent) {
  if (!dragging.value) return;

  const dx = event.clientX - from.x;
  const dy = event.clientY - from.y;
  if (!moved && Math.abs(dx) + Math.abs(dy) < 4) return;

  moved = true;
  place.value = clampPlace({ x: from.left + dx, y: from.top + dy }, root.value);
}

function onUp(event: PointerEvent) {
  if (!dragging.value) return;

  dragging.value = false;
  (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);

  if (!moved) {
    toggle();
    return;
  }
  savePlace(DOCK_POS_KEY, place.value);
}

function onCancel() {
  dragging.value = false;
}

function keepInside() {
  if (!place.value) return;
  place.value = clampPlace(place.value, root.value);
  savePlace(DOCK_POS_KEY, place.value);
}

function copyInvite() {
  // в буфер уходит только настоящий адрес, а не заглушка и не точки
  void copyText(
    together.invite,
    "Адрес скопирован",
    "Адрес ещё не определился",
  );
}

function openSettings() {
  void router.push({ path: "/settings", query: { section: "together" } });
}
</script>

<style scoped>
.together-dock {
  position: fixed;
  z-index: 2400;
  bottom: 96px;
  left: 16px;
  width: 268px;
  overflow: hidden;
  border: 1px solid var(--line);
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: 0 18px 40px var(--shadow-soft);
  user-select: none;
}

.together-dock.open {
  width: 320px;
}

.together-dock.dragging {
  box-shadow: 0 24px 60px var(--shadow-strong);
  opacity: 0.96;
}

.together-dock-head {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  color: var(--fg);
  font-size: 13px;
  cursor: grab;
  touch-action: none;
}

.together-dock.dragging .together-dock-head {
  cursor: grabbing;
}

.together-dock-head:hover {
  background: var(--hover);
}

.together-dock-grip {
  flex: 0 0 auto;
  color: var(--fg-faint);
}

.together-dock-dot {
  width: 8px;
  height: 8px;
  flex: 0 0 auto;
  border-radius: 50%;
  background: #3ecf6b;
}

.together-dock-dot.wait {
  background: var(--accent);
}

.together-dock-title {
  flex: 1 1 auto;
  overflow: hidden;
  text-align: left;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.together-dock-count {
  padding: 1px 7px;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--fg-dim);
  font-size: 11px;
}

.together-dock-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 4px 12px 12px;
  border-top: 1px solid var(--line);
}

.together-dock-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-top: 8px;
  font-size: 12px;
}

.together-dock-label {
  flex: 0 0 auto;
  color: var(--fg-faint);
}

.together-dock-code {
  flex: 1 1 auto;
  overflow: hidden;
  color: var(--fg);
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
}

.together-dock-code:hover {
  color: var(--accent);
}

.together-dock-value {
  flex: 1 1 auto;
  overflow: hidden;
  color: var(--fg-dim);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.together-dock-icon {
  display: inline-flex;
  padding: 3px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--fg-dim);
  cursor: pointer;
}

.together-dock-icon:hover {
  background: var(--hover);
  color: var(--fg);
}

.together-dock-wait {
  margin: 0;
  color: var(--accent);
  font-size: 12px;
}

.together-dock-hint {
  margin: 0;
  color: var(--fg-faint);
  font-size: 12px;
}

.together-dock-actions {
  display: flex;
  gap: 8px;
}

.together-dock-actions button {
  flex: 1 1 0;
  padding: 7px 8px;
  border: 1px solid var(--line);
  border-radius: calc(var(--radius) * 0.7);
  background: var(--surface-2);
  color: var(--fg);
  font: inherit;
  font-size: 12px;
  cursor: pointer;
}

.together-dock-actions button:hover {
  background: var(--hover);
}

.together-dock-actions button.danger {
  color: var(--accent);
}
</style>
