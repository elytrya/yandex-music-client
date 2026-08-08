<template>
  <div
    class="mini-player"
    :class="{ 'has-viz': ui.settings.miniVisualizer, 'mini-preview': preview }"
    :style="rootStyle"
    @mousedown="onRootMouseDown"
    @contextmenu="onContext"
  >
    <canvas
      v-if="ui.settings.miniVisualizer"
      ref="vizCanvas"
      class="mini-visualizer"
    />

    <div class="mini-top" @dblclick="leaveMini()">
      <div class="cover mini-cover">
        <Transition name="pb-cover">
          <img
            v-if="player.current?.cover_url"
            :key="player.current.cover_url"
            :src="player.current.cover_url"
            decoding="async"
          />
          <Icon v-else name="note" :size="18" class="faint" />
        </Transition>
      </div>

      <div class="mini-meta">
        <Transition name="pb-swap" mode="out-in">
          <div :key="displayTitle" class="mini-title ellipsis">
            {{ displayTitle }}<AiTag :show="isAiCurrent" />
          </div>
        </Transition>
        <div class="mini-artist ellipsis" data-no-drag>
          <Transition name="pb-swap" mode="out-in">
            <ArtistsLine
              v-if="player.current"
              :key="player.current.id"
              :artists="player.current.artists"
              :limit="2"
            />
            <span v-else>Mashiro</span>
          </Transition>
        </div>
        <TrackMenu
          v-if="player.current"
          :context-menu="true"
          :track="player.current"
        />
      </div>

      <button
        v-if="!preview"
        type="button"
        class="mini-btn mini-restore"
        data-no-drag
        aria-label="Вернуться в обычный вид"
        @click.stop="leaveMini()"
      >
        <Icon name="restore" :size="15" />
        <q-tooltip>Обычное окно (Esc)</q-tooltip>
      </button>
    </div>

    <div class="mini-controls">
      <template v-for="id in miniButtons" :key="id">
        <button
          v-if="id === 'play'"
          type="button"
          class="mini-btn mini-play"
          data-no-drag
          :aria-label="player.isPlaying ? 'Пауза' : 'Играть'"
          @click.stop="player.toggle()"
        >
          <Transition name="ic-pop" mode="out-in">
            <q-spinner
              v-if="player.loading"
              key="load"
              size="15px"
              color="white"
            />
            <Icon
              v-else
              :key="player.isPlaying ? 'pause' : 'play'"
              :name="player.isPlaying ? 'pause' : 'play'"
              :size="16"
            />
          </Transition>
        </button>

        <button
          v-else-if="id === 'prev'"
          type="button"
          class="mini-btn"
          :class="{ dim: !player.hasPrev }"
          data-no-drag
          aria-label="Предыдущий трек"
          @click.stop="player.prev()"
        >
          <Icon name="prev" :size="16" />
          <q-tooltip>Предыдущий трек</q-tooltip>
        </button>

        <button
          v-else-if="id === 'next'"
          type="button"
          class="mini-btn"
          data-no-drag
          aria-label="Следующий трек"
          @click.stop="player.next(false)"
        >
          <Icon name="next" :size="16" />
          <q-tooltip>Следующий трек</q-tooltip>
        </button>

        <button
          v-else-if="id === 'shuffle'"
          type="button"
          class="mini-btn"
          :class="{ on: player.shuffle }"
          data-no-drag
          aria-label="Перемешать"
          @click.stop="player.toggleShuffle()"
        >
          <Icon name="shuffle" :size="15" />
          <q-tooltip>Перемешать</q-tooltip>
        </button>

        <button
          v-else-if="id === 'repeat'"
          type="button"
          class="mini-btn"
          :class="{ on: player.repeat !== 'off' }"
          data-no-drag
          aria-label="Повтор"
          @click.stop="player.cycleRepeat()"
        >
          <Icon
            :name="player.repeat === 'one' ? 'repeatOne' : 'repeat'"
            :size="15"
          />
          <q-tooltip>Повтор</q-tooltip>
        </button>

        <button
          v-else-if="id === 'like'"
          type="button"
          class="mini-btn"
          :class="{ on: isLiked, dim: !player.current }"
          data-no-drag
          aria-label="Нравится"
          @click.stop="player.current && player.like()"
        >
          <Icon :name="isLiked ? 'heartFilled' : 'heart'" :size="15" />
          <q-tooltip>Нравится</q-tooltip>
        </button>

        <button
          v-else-if="id === 'dislike'"
          type="button"
          class="mini-btn"
          :class="{ dim: !player.current }"
          data-no-drag
          aria-label="Не нравится"
          @click.stop="player.current && player.dislike()"
        >
          <Icon name="heartOff" :size="15" />
          <q-tooltip>Не нравится</q-tooltip>
        </button>

        <button
          v-else-if="id === 'lyrics'"
          type="button"
          class="mini-btn"
          :class="{ on: player.showLyrics, dim: !player.current }"
          data-no-drag
          aria-label="Текст песни"
          @click.stop="player.current && player.toggleLyrics()"
        >
          <Icon name="lyrics" :size="15" />
          <q-tooltip>Текст песни</q-tooltip>
        </button>

        <div
          v-else-if="id === 'volume'"
          class="mini-vol"
          data-no-drag
          @mouseenter="onVolEnter"
          @mouseleave="onVolLeave"
        >
          <Transition name="mini-vol-pop">
            <div
              v-if="volumeOpen"
              class="mini-vol-pop"
              data-no-drag
              @wheel.prevent.stop="onVolWheel"
            >
              <div
                ref="volTrack"
                class="mini-vol-track"
                @pointerdown.stop="onVolPointerDown"
              >
                <div
                  class="mini-vol-fill"
                  :style="{ height: `${volumePercent}%` }"
                >
                  <span class="mini-vol-knob" />
                </div>
              </div>
            </div>
          </Transition>

          <button
            type="button"
            class="mini-btn"
            :class="{ on: volumeOpen }"
            data-no-drag
            aria-label="Громкость"
            @click.stop="onVolClick"
            @wheel.prevent.stop="onVolWheel"
          >
            <Icon :name="volumeIcon" :size="15" />
            <q-tooltip v-if="!volumeOpen">
              Громкость: {{ volumePercent }}%
            </q-tooltip>
          </button>
        </div>
      </template>
    </div>

    <div class="mini-seek">
      <span v-if="ui.settings.miniShowTime" class="mini-time">
        {{ formatDuration(player.progress * 1000) }}
      </span>

      <div class="mini-bar" data-no-drag @click.stop="seekAt">
        <div class="mini-bar-fill" :style="{ width: `${percent}%` }">
          <span class="mini-bar-knob" />
        </div>
      </div>

      <span v-if="ui.settings.miniShowTime" class="mini-time">
        {{ formatDuration(player.duration * 1000) }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import Icon from "@/components/Icon.vue";
import AiTag from "@/components/AiTag.vue";
import ArtistsLine from "@/components/ArtistsLine.vue";
import TrackMenu from "@/components/TrackMenu.vue";
import { useVisualizer } from "@/composables/useVisualizer";
import { formatDuration } from "@/lib/format";
import { ensureAiArtists, isAiArtist } from "@/lib/aiTag";
import { startWindowDrag } from "@/lib/window";
import { api } from "@/api/client";
import { useLibraryStore } from "@/stores/library";
import { usePanelsStore } from "@/stores/panels";
import { usePlayerStore } from "@/stores/player/index";
import { useUiStore } from "@/stores/ui/index";

const props = withDefaults(defineProps<{ preview?: boolean }>(), {
  preview: false,
});

const player = usePlayerStore();
const panels = usePanelsStore();
const library = useLibraryStore();
const ui = useUiStore();
const router = useRouter();

const miniButtons = computed(() => ui.activeMiniButtons());

function onRootMouseDown(event: MouseEvent) {
  if (props.preview) return;
  startWindowDrag(event);
}

async function leaveMini() {
  if (props.preview) return;
  await panels.exitMini();
}

async function onContext(event: MouseEvent) {
  if (props.preview) return;
  const target = event.target as HTMLElement | null;
  if (target?.closest(".mini-meta")) return;
  event.preventDefault();
  await panels.exitMini();
  await router.push({ path: "/settings", query: { section: "mini" } });
}

const primaryArtistId = computed(
  () => player.current?.artists?.[0]?.id ?? null,
);
const isAiCurrent = computed(() => isAiArtist(primaryArtistId.value));
const displayTitle = computed(
  () => player.current?.title || "Ничего не играет",
);

watch(
  () => player.current?.id,
  (id) => {
    if (!id) return;
    if (primaryArtistId.value) ensureAiArtists([primaryArtistId.value]);
  },
  { immediate: true },
);

const percent = computed(() =>
  player.duration > 0
    ? Math.min(100, (player.progress / player.duration) * 100)
    : 0,
);

const isLiked = computed(() =>
  player.current ? library.liked(player.current.id) : false,
);

function seekAt(event: MouseEvent) {
  const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
  if (!box.width || !player.duration) return;
  const ratio = Math.min(
    1,
    Math.max(0, (event.clientX - box.left) / box.width),
  );
  player.seek(ratio * player.duration);
}

const volumeOpen = ref(false);
const volTrack = ref<HTMLElement | null>(null);
let volCloseTimer = 0;
let volDragging = false;

const volumePercent = computed(() =>
  Math.round((player.muted ? 0 : player.volume) * 100),
);

const volumeIcon = computed(() =>
  player.muted || player.volume <= 0.001 ? "volumeOff" : "volume",
);

function openVolume() {
  if (!ui.settings.miniVolumeSlider) return;
  window.clearTimeout(volCloseTimer);
  volumeOpen.value = true;
}

function scheduleVolumeClose(delay = 320) {
  window.clearTimeout(volCloseTimer);
  volCloseTimer = window.setTimeout(() => {
    if (!volDragging) volumeOpen.value = false;
  }, delay);
}

function onVolEnter() {
  openVolume();
}

function onVolLeave() {
  scheduleVolumeClose();
}

function onVolClick() {
  player.toggleMute();
}

function applyVolume(value: number) {
  const next = Math.min(1, Math.max(0, value));
  player.setVolume(next);
  if (next > 0 && player.muted) player.toggleMute();
}

function onVolWheel(event: WheelEvent) {
  openVolume();
  applyVolume((player.muted ? 0 : player.volume) - event.deltaY / 2000);
  scheduleVolumeClose(900);
}

function volumeFromEvent(event: PointerEvent) {
  const box = volTrack.value?.getBoundingClientRect();
  if (!box?.height) return;
  applyVolume((box.bottom - event.clientY) / box.height);
}

function onVolPointerMove(event: PointerEvent) {
  if (!volDragging) return;
  volumeFromEvent(event);
}

function onVolPointerUp() {
  volDragging = false;
  window.removeEventListener("pointermove", onVolPointerMove);
  window.removeEventListener("pointerup", onVolPointerUp);
  window.removeEventListener("pointercancel", onVolPointerUp);
  scheduleVolumeClose();
}

function onVolPointerDown(event: PointerEvent) {
  event.preventDefault();
  volDragging = true;
  openVolume();
  try {
    (event.currentTarget as HTMLElement | null)?.setPointerCapture(
      event.pointerId,
    );
  } catch {}
  volumeFromEvent(event);
  window.addEventListener("pointermove", onVolPointerMove);
  window.addEventListener("pointerup", onVolPointerUp);
  window.addEventListener("pointercancel", onVolPointerUp);
}

const vizCanvas = ref<HTMLCanvasElement | null>(null);
useVisualizer(vizCanvas, () => ui.settings.miniVisualizer, { bars: 56 });

const rootStyle = computed(() => ({
  "--mini-h": props.preview ? `${ui.settings.miniHeight}px` : "100vh",
}));

watch(
  () => [props.preview, ui.settings.miniVolumeSlider] as const,
  ([isPreview, sliderOn]) => {
    if (isPreview) volumeOpen.value = !!sliderOn;
  },
  { immediate: true },
);

function onKey(event: KeyboardEvent) {
  if (event.key !== "Escape") return;
  event.preventDefault();
  void panels.exitMini();
}

onMounted(() => {
  if (props.preview) return;
  window.addEventListener("keydown", onKey);
  document.documentElement.dataset.mini = "1";
  void api.resizeMiniPlayer(ui.settings.miniWidth, ui.settings.miniHeight);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKey);
  window.removeEventListener("pointermove", onVolPointerMove);
  window.removeEventListener("pointerup", onVolPointerUp);
  window.clearTimeout(volCloseTimer);
  if (!props.preview) delete document.documentElement.dataset.mini;
});
</script>

<style scoped>
.mini-player.mini-preview {
  position: absolute;
  z-index: 1;
  border-radius: 14px;
  pointer-events: none;
}
</style>
