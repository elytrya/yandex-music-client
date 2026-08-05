<template>
  <div class="player-bar" :class="{ 'has-viz': ui.settings.playerVisualizer }">
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
        <q-tooltip>На полный экран</q-tooltip>
        <div class="player-cover-hover">
          <Icon name="maximize" :size="15" />
        </div>
        <img
          v-if="player.current?.cover_url"
          loading="lazy"
          decoding="async"
          :src="player.current.cover_url"
        />
        <Icon v-else name="note" :size="18" class="faint" />
      </div>

      <div class="player-meta">
        <div class="player-title-row">
          <span class="ellipsis t-13 w-600 player-title-text">
            {{ displayTitle }}
          </span>
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
          <ArtistsLine
            v-if="player.current"
            :artists="player.current.artists"
            :limit="2"
          />
        </div>
      </div>

      <div v-if="leftButtons.length" class="player-zone player-zone-left">
        <PlayerButton v-for="id in leftButtons" :key="id" :id="id" />
      </div>
    </div>

    <div class="player-center">
      <div class="player-controls">
        <PlayerButton v-for="id in centerButtons" :key="id" :id="id" />
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

    <div class="player-right">
      <PlayerButton v-for="id in rightButtons" :key="id" :id="id" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
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
