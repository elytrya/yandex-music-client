<template>
  <div
    class="mini-player"
    :class="{ 'has-viz': ui.settings.miniVisualizer }"
    @mousedown="startWindowDrag"
  >
    <canvas
      v-if="ui.settings.miniVisualizer"
      ref="vizCanvas"
      class="mini-visualizer"
    />

    <div class="mini-top" @dblclick="panels.exitMini()">
      <div class="cover mini-cover">
        <img
          v-if="player.current?.cover_url"
          :src="player.current.cover_url"
          decoding="async"
        />
        <Icon v-else name="note" :size="18" class="faint" />
      </div>

      <div class="mini-meta">
        <div class="mini-title ellipsis">
          {{ displayTitle }}<AiTag :show="isAiCurrent" />
        </div>
        <div class="mini-artist ellipsis" data-no-drag>
          <ArtistsLine
            v-if="player.current"
            :artists="player.current.artists"
            :limit="2"
          />
          <template v-else>Mashiro</template>
        </div>
        <TrackMenu
          v-if="player.current"
          :context-menu="true"
          :track="player.current"
        />
      </div>

      <button
        type="button"
        class="mini-btn mini-restore"
        data-no-drag
        aria-label="Вернуться в обычный вид"
        @click.stop="panels.exitMini()"
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
          <q-spinner v-if="player.loading" size="15px" color="white" />
          <Icon v-else :name="player.isPlaying ? 'pause' : 'play'" :size="16" />
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

        <button
          v-else-if="id === 'volume'"
          type="button"
          class="mini-btn"
          data-no-drag
          aria-label="Громкость"
          @click.stop="player.toggleMute()"
        >
          <Icon :name="player.muted ? 'volumeOff' : 'volume'" :size="15" />
          <q-tooltip>Громкость</q-tooltip>
        </button>
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
import Icon from "@/components/Icon.vue";
import AiTag from "@/components/AiTag.vue";
import ArtistsLine from "@/components/ArtistsLine.vue";
import TrackMenu from "@/components/TrackMenu.vue";
import { useVisualizer } from "@/composables/useVisualizer";
import { formatDuration } from "@/lib/format";
import { ensureAiArtists, isAiArtist } from "@/lib/aiTag";
import { startWindowDrag } from "@/lib/window";
import { useLibraryStore } from "@/stores/library";
import { usePanelsStore } from "@/stores/panels";
import { usePlayerStore } from "@/stores/player/index";
import { useUiStore } from "@/stores/ui/index";

const player = usePlayerStore();
const panels = usePanelsStore();
const library = useLibraryStore();
const ui = useUiStore();

const miniButtons = computed(() => ui.activeMiniButtons());

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

const vizCanvas = ref<HTMLCanvasElement | null>(null);
useVisualizer(vizCanvas, () => ui.settings.miniVisualizer, { bars: 56 });

function onKey(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    void panels.exitMini();
  }
}

onMounted(() => {
  window.addEventListener("keydown", onKey);
  document.documentElement.dataset.mini = "1";
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKey);
  delete document.documentElement.dataset.mini;
});
</script>
