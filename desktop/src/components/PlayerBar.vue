<template>
  <div
    class="player-bar"
    :class="{
      'has-viz': ui.settings.playerVisualizer,
      'player-edit': editMode,
    }"
    @contextmenu.prevent="toggleEdit"
  >
    <template v-if="editMode">
      <div
        class="pe-grip pe-grip-top"
        title="Тяни вверх или вниз — высота плеера"
        @pointerdown="startResize('height', $event)"
      />
      <div
        class="pe-grip pe-grip-meta"
        title="Тяни влево или вправо — ширина блока с треком"
        @pointerdown="startResize('meta', $event)"
      />

      <div class="pe-hint">
        <Icon name="drag" :size="13" />
        <span>
          Режим редактирования: тащи кнопки между зонами, края — размеры
        </span>
        <button
          type="button"
          class="pe-btn"
          @click.stop="ui.resetPlayerLayout()"
        >
          Сбросить
        </button>
        <button type="button" class="pe-btn accent" @click.stop="toggleEdit">
          Готово
        </button>
      </div>
    </template>
    <canvas
      v-if="ui.settings.playerVisualizer"
      ref="vizCanvas"
      class="player-visualizer"
    />

    <div class="player-left">
      <div
        class="cover player-cover"
        @click="player.openFullscreen()"
        @contextmenu.prevent="openAlbum"
      >
        <div class="player-cover-hover">
          <Icon name="expand" :size="16" />
        </div>
        <Transition name="pb-cover">
          <img
            v-if="player.current?.cover_url"
            :key="player.current.cover_url"
            loading="lazy"
            decoding="async"
            :src="player.current.cover_url"
          />
          <Icon v-else name="note" :size="18" class="faint" />
        </Transition>
      </div>

      <div class="player-meta">
        <div class="player-title-row">
          <Transition name="pb-swap" mode="out-in">
            <span
              :key="displayTitle"
              class="ellipsis t-13 w-600 player-title-text"
            >
              {{ displayTitle }}
            </span>
          </Transition>
          <AiTag :show="isAiCurrent" />
          <span v-if="showCensorBadge" class="censor-tag">
            без цензуры
            <q-tooltip>
              Оригинальный трек заменён на версию без цензуры из базы
              FckCensorData
            </q-tooltip>
          </span>
          <button
            v-if="player.current"
            type="button"
            class="pb-dots"
            aria-label="Действия с треком"
          >
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="currentColor"
              aria-hidden="true"
            >
              <circle cx="5" cy="12" r="2" />
              <circle cx="12" cy="12" r="2" />
              <circle cx="19" cy="12" r="2" />
            </svg>
            <q-tooltip>Действия с треком</q-tooltip>
            <TrackMenu :track="player.current" />
          </button>
        </div>
        <div class="dim t-12 ellipsis">
          <Transition name="pb-swap" mode="out-in">
            <ArtistsLine
              v-if="player.current"
              :key="player.current.id"
              :artists="player.current.artists"
              :limit="2"
            />
          </Transition>
        </div>
      </div>

      <div
        v-if="leftButtons.length || editMode"
        class="player-zone player-zone-left"
        :data-zone-label="editMode ? 'Слева' : null"
        @dragover.prevent
        @drop.prevent="dropInZone('left', leftButtons.length)"
      >
        <span
          v-for="(id, index) in leftButtons"
          :key="id"
          class="pe-item"
          :class="{ dragging: dragId === id }"
          :draggable="editMode"
          @dragstart="dragId = id"
          @dragend="dragId = null"
          @dragover.prevent
          @drop.prevent.stop="dropInZone('left', index)"
        >
          <PlayerButton :id="id" />
        </span>
      </div>
    </div>

    <div class="player-center">
      <div
        class="player-controls"
        :data-zone-label="editMode ? 'Центр' : null"
        @dragover.prevent
        @drop.prevent="dropInZone('center', centerButtons.length)"
      >
        <span
          v-for="(id, index) in centerButtons"
          :key="id"
          class="pe-item"
          :class="{ dragging: dragId === id }"
          :draggable="editMode"
          @dragstart="dragId = id"
          @dragend="dragId = null"
          @dragover.prevent
          @drop.prevent.stop="dropInZone('center', index)"
        >
          <PlayerButton :id="id" />
        </span>
      </div>

      <div class="player-progress">
        <div class="faint t-11 player-time">
          {{ formatDuration(player.progress * 1000) }}
        </div>
        <q-slider
          class="col"
          :model-value="player.progress"
          :min="0"
          :max="Math.max(player.duration, 1)"
          :step="1"
          dense
          color="primary"
          @update:model-value="(v) => player.seek(Number(v ?? 0))"
        />
        <div class="faint t-11 player-time">
          {{ formatDuration(player.duration * 1000) }}
        </div>
      </div>
    </div>

    <div
      class="player-right"
      :data-zone-label="editMode ? 'Справа' : null"
      @dragover.prevent
      @drop.prevent="dropInZone('right', rightButtons.length)"
    >
      <span
        v-for="(id, index) in rightButtons"
        :key="id"
        class="pe-item"
        :class="{ dragging: dragId === id }"
        :draggable="editMode"
        @dragstart="dragId = id"
        @dragend="dragId = null"
        @dragover.prevent
        @drop.prevent.stop="dropInZone('right', index)"
      >
        <PlayerButton :id="id" />
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import ArtistsLine from "@/components/ArtistsLine.vue";
import AiTag from "@/components/AiTag.vue";
import Icon from "@/components/Icon.vue";
import PlayerButton from "@/components/player/PlayerButton.vue";
import TrackMenu from "@/components/TrackMenu.vue";
import { useVisualizer } from "@/composables/useVisualizer";
import { formatDuration } from "@/lib/format";
import { ensureAiArtists, isAiArtist } from "@/lib/aiTag";
import { useLibraryStore } from "@/stores/library";
import { usePlayerStore } from "@/stores/player/index";
import type { PlayerButtonId, PlayerZone } from "@/stores/ui/index";
import { useUiStore } from "@/stores/ui/index";

const router = useRouter();
const player = usePlayerStore();
const library = useLibraryStore();
const ui = useUiStore();

const vizCanvas = ref<HTMLCanvasElement | null>(null);
useVisualizer(vizCanvas, () => ui.settings.playerVisualizer, { bars: 72 });

const primaryArtistId = computed(
  () => player.current?.artists?.[0]?.id ?? null,
);
const isAiCurrent = computed(() => isAiArtist(primaryArtistId.value));
const displayTitle = computed(
  () => player.current?.title || "Ничего не играет",
);
const showCensorBadge = computed(
  () => ui.settings.censorBadge && player.censorReplaced,
);

const handledDislike = new Set<string>();

watch(
  () => player.current?.id,
  (id) => {
    if (!id) return;
    if (primaryArtistId.value) ensureAiArtists([primaryArtistId.value]);
  },
  { immediate: true },
);

watch(
  () => [player.current?.id ?? null, isAiCurrent.value] as const,
  ([id, ai]) => {
    if (!id || !ai) return;
    if (!ui.settings.autoDislikeAi) return;
    if (handledDislike.has(id)) return;
    handledDislike.add(id);
    void player.dislike();
  },
);

const leftButtons = computed(() => ui.playerZoneButtons("left"));
const centerButtons = computed(() => ui.playerZoneButtons("center"));
const rightButtons = computed(() => ui.playerZoneButtons("right"));

/* --- Свободное редактирование прямо на плеере --- */

const editMode = computed(() => ui.settings.playerEditMode);
const dragId = ref<PlayerButtonId | null>(null);

function toggleEdit() {
  ui.set("playerEditMode", !ui.settings.playerEditMode);
}

function dropInZone(zone: PlayerZone, index: number) {
  const id = dragId.value;
  dragId.value = null;
  if (!id) return;
  ui.movePlayerButton(id, zone, index);
}

type ResizeKind = "height" | "meta";

let resizeKind: ResizeKind | null = null;
let startPos = 0;
let startValue = 0;

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, Math.round(value)));
}

function onResizeMove(event: PointerEvent) {
  if (resizeKind === "height") {
    ui.set(
      "playerHeight",
      clamp(startValue + (startPos - event.clientY), 60, 140),
    );
  } else if (resizeKind === "meta") {
    ui.set(
      "playerMetaWidth",
      clamp(startValue + (event.clientX - startPos), 150, 460),
    );
  }
}

function onResizeUp() {
  resizeKind = null;
  window.removeEventListener("pointermove", onResizeMove);
  window.removeEventListener("pointerup", onResizeUp);
}

function startResize(kind: ResizeKind, event: PointerEvent) {
  event.preventDefault();
  resizeKind = kind;
  startPos = kind === "height" ? event.clientY : event.clientX;
  startValue =
    kind === "height" ? ui.settings.playerHeight : ui.settings.playerMetaWidth;
  window.addEventListener("pointermove", onResizeMove);
  window.addEventListener("pointerup", onResizeUp);
}

onBeforeUnmount(onResizeUp);

function openAlbum() {
  const id = player.current?.album_id;
  if (id) void router.push(`/album/${id}`);
}

onMounted(() => {
  player.bind();
  void library.init();
});
</script>

<style scoped>
.pe-item {
  display: inline-flex;
  align-items: center;
}

.player-edit {
  outline: 1px dashed color-mix(in srgb, var(--accent) 60%, transparent);
  outline-offset: -2px;
}

.player-edit .pe-item {
  border-radius: 10px;
  cursor: grab;
  transition:
    background 0.14s ease,
    opacity 0.14s ease;
}

.player-edit .pe-item > * {
  pointer-events: none;
}

.player-edit .pe-item:hover {
  background: var(--hover);
}

.player-edit .pe-item.dragging {
  opacity: 0.4;
}

.player-edit .player-zone,
.player-edit .player-controls,
.player-edit .player-right {
  position: relative;
  min-width: 42px;
  min-height: 34px;
  border: 1px dashed color-mix(in srgb, var(--accent) 45%, transparent);
  border-radius: 12px;
}

.player-edit [data-zone-label]::before {
  position: absolute;
  top: -9px;
  left: 8px;
  padding: 0 4px;
  border-radius: 4px;
  background: var(--surface);
  color: var(--fg-faint);
  content: attr(data-zone-label);
  font-size: 9px;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.pe-grip {
  position: absolute;
  z-index: 14;
  background: color-mix(in srgb, var(--accent) 55%, transparent);
}

.pe-grip-top {
  top: 0;
  left: 0;
  width: 100%;
  height: 5px;
  cursor: ns-resize;
}

.pe-grip-meta {
  top: 20%;
  left: var(--player-meta-width, 260px);
  width: 5px;
  height: 60%;
  border-radius: 3px;
  cursor: ew-resize;
}

.pe-hint {
  position: absolute;
  bottom: calc(100% + 8px);
  left: 50%;
  z-index: 20;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border: 1px solid var(--line);
  border-radius: 10px;
  background: var(--surface-2);
  box-shadow: var(--shadow-strong, 0 10px 28px rgba(0, 0, 0, 0.45));
  color: var(--fg-dim);
  font-size: 11px;
  transform: translateX(-50%);
  white-space: nowrap;
}

.pe-btn {
  padding: 3px 8px;
  border: 1px solid var(--line);
  border-radius: 7px;
  background: transparent;
  color: var(--fg);
  font: inherit;
  cursor: pointer;
}

.pe-btn.accent {
  border-color: var(--accent);
  background: var(--accent);
  color: #fff;
}

.censor-tag {
  display: inline-flex;
  align-items: center;
  margin-left: 6px;
  padding: 0 5px;
  border: 1px solid var(--accent, #fa2d48);
  border-radius: 4px;
  background: rgba(250, 45, 72, 0.12);
  color: var(--accent, #fa2d48);
  font-size: 10px;
  font-weight: 600;
  line-height: 1.5;
  letter-spacing: 0.02em;
  white-space: nowrap;
  vertical-align: middle;
  flex: 0 0 auto;
  cursor: default;
}
</style>
