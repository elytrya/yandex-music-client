<template>
  <div
    v-if="player.showLyrics"
    class="lyrics-wrap"
    :class="{ 'lyrics-fs': player.lyricsFullscreen }"
    :style="visualStyle"
  >
    <div
      v-if="coverUrl"
      class="lyrics-backdrop"
      :style="{ backgroundImage: `url(${coverUrl})` }"
    />
    <div class="lyrics-shade" />

    <div class="lyrics-toolbar">
      <div class="lyrics-track">
        <div class="cover lyrics-mini-cover">
          <img
            loading="lazy"
            decoding="async"
            v-if="coverUrl"
            :src="coverUrl"
          />
          <Icon v-else name="note" :size="18" class="faint" />
        </div>
        <div style="min-width: 0">
          <div class="t-14 w-600 ellipsis lyrics-link" @click="openAlbum">
            {{ player.current?.title || "Текст песни" }}
          </div>
          <div class="lyrics-subtitle ellipsis lyrics-link" @click="openArtist">
            {{ artistLabel }}
          </div>
        </div>
      </div>

      <div class="row items-center no-wrap" style="gap: 6px">
        <div v-if="player.current" class="icon-btn round" data-no-drag>
          <Icon name="more" :size="18" />
          <TrackMenu :track="player.current" />
        </div>
        <div
          class="icon-btn round"
          :class="{ on: showSettings }"
          data-no-drag
          @click="showSettings = !showSettings"
        >
          <Icon name="settings" :size="18" />
          <q-tooltip>Настройки текста</q-tooltip>
        </div>
        <div
          class="icon-btn round"
          :class="{ on: player.lyricsFullscreen }"
          data-no-drag
          @click="player.toggleLyricsFullscreen()"
        >
          <Icon
            :name="player.lyricsFullscreen ? 'restore' : 'maximize'"
            :size="16"
          />
          <q-tooltip>
            {{
              player.lyricsFullscreen
                ? "Выйти из полного экрана"
                : "На полный экран"
            }}
          </q-tooltip>
        </div>
        <div
          class="icon-btn round"
          data-no-drag
          @click="player.openFullscreen()"
        >
          <Icon name="album" :size="18" />
          <q-tooltip>Большая обложка</q-tooltip>
        </div>
        <div class="icon-btn round" data-no-drag @click="player.toggleLyrics()">
          <Icon name="close" :size="19" />
        </div>
      </div>
    </div>

    <LyricsSettingsPanel v-if="showSettings" />

    <div v-if="player.lyricsLoading" class="lyrics-empty lyrics-layer">
      <q-spinner size="26px" color="primary" />
    </div>

    <div v-else-if="player.lyricsError" class="lyrics-empty lyrics-layer">
      <Icon name="lyrics" :size="28" />
      <div>{{ player.lyricsError }}</div>
    </div>

    <div
      v-else-if="lines.length"
      class="lyrics-stage lyrics-layer"
      :class="{ 'no-artwork': !settings.showArtwork || !coverUrl }"
    >
      <div
        v-if="settings.showArtwork && coverUrl"
        class="lyrics-artwork-column"
      >
        <div
          class="lyrics-artwork"
          :class="{ playing: player.isPlaying && settings.motion }"
          @click="player.toggleLyricsFullscreen()"
        >
          <img loading="lazy" decoding="async" :src="coverUrl" />
          <div
            class="lyrics-artwork-glow"
            :style="{ backgroundImage: `url(${coverUrl})` }"
          />
        </div>
        <div class="lyrics-artwork-meta">
          <div class="lyrics-artwork-title">{{ player.current?.title }}</div>
          <div class="lyrics-subtitle">{{ artistLabel }}</div>
        </div>
      </div>

      <q-scroll-area ref="scroller" class="lyrics-scroll">
        <div
          class="lyrics-lines"
          :class="[
            `align-${settings.align}`,
            { motion: settings.motion && synced, plain: !synced },
          ]"
        >
          <template v-if="synced">
            <button
              v-for="(line, i) in lines"
              :key="i"
              :ref="(el) => setLineRef(el, i)"
              class="lyric-line"
              :class="lineClass(i)"
              @click="seekTo(line.time_ms)"
            >
              {{ line.text || "…" }}
            </button>
          </template>
          <div v-else class="lyrics-plain-text">
            <p v-for="(line, i) in lines" :key="i">{{ line.text }}</p>
          </div>
          <div v-if="player.lyrics?.writers.length" class="lyrics-writers">
            Авторы: {{ player.lyrics.writers.join(", ") }}
          </div>
        </div>
      </q-scroll-area>
    </div>

    <div v-else class="lyrics-empty lyrics-layer">
      <Icon name="lyrics" :size="28" />
      <div>Текста для этого трека нет</div>
    </div>
    <TrackMenu
      v-if="player.current"
      :track="player.current"
      :context-menu="true"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { QScrollArea } from "quasar";
import { useRouter } from "vue-router";
import Icon from "@/components/Icon.vue";
import LyricsSettingsPanel from "@/components/lyrics/LyricsSettingsPanel.vue";
import TrackMenu from "@/components/TrackMenu.vue";
import { artistNames } from "@/lib/format";
import { usePlayerStore } from "@/stores/player/index";
import { useUiStore } from "@/stores/ui/index";

const player = usePlayerStore();
const router = useRouter();
const ui = useUiStore();
const settings = computed(() => ({
  fontSize: ui.settings.lyricsFontSize,
  backgroundBlur: ui.settings.lyricsBackgroundBlur,
  backgroundOpacity: ui.settings.lyricsBackgroundOpacity,
  lineBlur: ui.settings.lyricsLineBlur,
  align: ui.settings.lyricsAlign,
  showArtwork: ui.settings.lyricsShowArtwork,
  motion: ui.settings.lyricsMotion,
}));
const showSettings = ref(false);
const scroller = ref<QScrollArea | null>(null);
const lineRefs = ref<Array<HTMLElement | null>>([]);

const lines = computed(() => player.lyrics?.lines || []);
const synced = computed(() => Boolean(player.lyrics?.synced));
const coverUrl = computed(() => player.current?.cover_url || "");
const artistLabel = computed(() =>
  player.current ? artistNames(player.current.artists) : "",
);
const visualStyle = computed(() => ({
  "--lyrics-size": `${settings.value.fontSize}px`,
  "--lyrics-bg-blur": `${settings.value.backgroundBlur}px`,
  "--lyrics-bg-opacity": `${settings.value.backgroundOpacity / 100}`,
  "--lyrics-line-blur": `${settings.value.lineBlur}px`,
}));

function openAlbum() {
  const id = player.current?.album_id;
  if (id) void router.push(`/album/${id}`);
}

function openArtist() {
  const id = player.current?.artists[0]?.id;
  if (id) void router.push(`/artist/${id}`);
}

function setLineRef(el: unknown, i: number) {
  lineRefs.value[i] = (el as HTMLElement | null) ?? null;
}

function seekTo(ms: number) {
  if (synced.value && ms >= 0) player.seek(ms / 1000);
}

function lineClass(i: number) {
  const active = player.activeLine;
  return {
    on: i === active,
    past: synced.value && i < active,
    near: synced.value && Math.abs(i - active) === 1,
    far: synced.value && active >= 0 && Math.abs(i - active) > 1,
  };
}

watch(
  () => player.activeLine,
  (i) => {
    if (!synced.value || i < 0) return;
    const el = lineRefs.value[i];
    const area = scroller.value;
    if (!el || !area) return;
    const target = Math.max(0, el.offsetTop - area.$el.clientHeight * 0.42);
    area.setScrollPosition("vertical", target, settings.value.motion ? 520 : 0);
  },
);
</script>
