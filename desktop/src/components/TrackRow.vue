<template>
  <div
    class="row-track"
    :class="{
      on: isCurrent,
      drop: dropTarget,
      dragging,
      picked: selected,
      disliked: isDisliked,
    }"
    :data-track-index="index"
    @dblclick="emit('play')"
    @click="onRowClick"
  >
    <div
      v-if="reorderable"
      class="handle"
      @pointerdown.stop.prevent="emit('reorderStart', index, $event)"
    >
      <Icon name="drag" :size="16" />
    </div>

    <button
      v-if="selectable"
      class="track-check-box"
      :class="{ on: selected }"
      type="button"
      :title="selected ? 'Снять выделение' : 'Выделить трек'"
      @click.stop="emit('toggleSelect', $event)"
    >
      <Icon v-if="selected" name="check" :size="13" />
    </button>

    <div style="width: 20px; text-align: center; flex: 0 0 auto">
      <div v-if="showIndex" class="num faint t-13">{{ index + 1 }}</div>
      <div class="num-play" @click.stop="onPlayClick">
        <Icon
          :name="isCurrent && player.isPlaying ? 'pause' : 'play'"
          :size="16"
        />
      </div>
    </div>

    <div v-if="showCover" class="cover" style="width: 38px; height: 38px">
      <img
        loading="lazy"
        decoding="async"
        v-if="track.cover_url"
        :src="track.cover_url"
      />
      <Icon v-else name="note" :size="15" class="faint" />
    </div>

    <div class="col" style="min-width: 0">
      <div class="row-track-title">
        <span class="ellipsis t-13 w-500">{{ displayTitle }}</span>
        <AiTag :show="isAi" />

        <Icon
          v-if="downloaded"
          name="download"
          :size="13"
          class="faint row-track-saved"
        />

        <div v-if="isDisliked" class="row-track-disliked">
          <Icon name="heartOff" :size="13" />
          <q-tooltip>Отмечен как «Не нравится»</q-tooltip>
        </div>

        <div class="icon-btn xs row-track-dots" @click.stop>
          <Icon name="moreH" :size="15" />
          <q-tooltip>Действия с треком</q-tooltip>
          <TrackMenu
            :track="track"
            :playlist-kind="playlistKind"
            @remove="emit('remove')"
          />
        </div>
      </div>
      <div class="dim t-12 ellipsis">
        <ArtistsLine :artists="track.artists" />
      </div>
    </div>

    <div v-if="showAlbum" class="dim t-12 ellipsis gt-sm" style="width: 190px">
      <span
        v-if="track.album_id"
        class="link"
        @click.stop="openAlbum(track.album_id)"
      >
        {{ track.album_title }}
      </span>
      <span v-else>{{ track.album_title }}</span>
    </div>

    <div class="actions">
      <div v-if="reorderable" class="icon-btn sm" @click.stop="emit('remove')">
        <Icon name="trash" :size="17" />
        <q-tooltip>Удалить из плейлиста</q-tooltip>
      </div>

      <div
        class="icon-btn sm"
        :class="{ on: isLiked }"
        @click.stop="library.toggleLike(track)"
      >
        <Icon :name="isLiked ? 'heartFilled' : 'heart'" :size="17" />
        <q-tooltip>{{
          isLiked ? "Убрать из «Мне нравится»" : "Нравится"
        }}</q-tooltip>
      </div>

      <div class="icon-btn sm" @click.stop>
        <Icon name="moreH" :size="17" />
        <TrackMenu
          :track="track"
          :playlist-kind="playlistKind"
          @remove="emit('remove')"
        />
      </div>
    </div>

    <div class="faint t-12 duration">
      {{ formatDuration(track.duration_ms) }}
    </div>

    <TrackMenu
      :context-menu="true"
      :track="track"
      :playlist-kind="playlistKind"
      @remove="emit('remove')"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from "vue";
import { useRouter } from "vue-router";
import ArtistsLine from "@/components/ArtistsLine.vue";
import AiTag from "@/components/AiTag.vue";
import Icon from "@/components/Icon.vue";
import TrackMenu from "@/components/TrackMenu.vue";
import type { Track } from "@/api/types";
import { formatDuration } from "@/lib/format";
import { ensureAiArtists, isAiArtist } from "@/lib/aiTag";
import { useLibraryStore } from "@/stores/library";
import { usePlayerStore } from "@/stores/player/index";

const props = withDefaults(
  defineProps<{
    track: Track;
    index: number;
    showCover?: boolean;
    showAlbum?: boolean;
    showIndex?: boolean;
    playlistKind?: number | string | null;
    dragging?: boolean;
    dropTarget?: boolean;
    selectable?: boolean;
    selected?: boolean;
    downloaded?: boolean;
  }>(),
  {
    showCover: true,
    showAlbum: true,
    showIndex: true,
    playlistKind: null,
    dragging: false,
    dropTarget: false,
    selectable: false,
    selected: false,
    downloaded: false,
  },
);

const emit = defineEmits<{
  play: [];
  remove: [];
  reorderStart: [number, PointerEvent];
  toggleSelect: [MouseEvent];
}>();

function onRowClick(event: MouseEvent) {
  if (!props.selectable) return;
  if (props.selected || event.ctrlKey || event.metaKey || event.shiftKey)
    emit("toggleSelect", event);
}

function onPlayClick() {
  if (isCurrent.value) {
    player.toggle();
    return;
  }
  emit("play");
}

const router = useRouter();
const player = usePlayerStore();
const library = useLibraryStore();

const primaryArtistId = computed(() => props.track.artists?.[0]?.id ?? null);
const isAi = computed(() => isAiArtist(primaryArtistId.value));
const displayTitle = computed(() => props.track.title);
watch(
  primaryArtistId,
  (id) => {
    if (id) ensureAiArtists([id]);
  },
  { immediate: true },
);

const isCurrent = computed(() => player.current?.id === props.track.id);
const isLiked = computed(() => library.liked(props.track.id));
const isDisliked = computed(() => library.disliked(props.track.id));
const reorderable = computed(
  () => props.playlistKind !== null && props.playlistKind !== undefined,
);

function openAlbum(id: string) {
  if (id) void router.push(`/album/${id}`);
}
</script>
