<template>
  <div
    v-if="player.showLyrics"
    ref="root"
    class="lyrics-wrap"
    :class="{ 'lyrics-fs': player.lyricsFullscreen, glow: s.glow }"
    :style="visualStyle"
  >
    <div
      v-if="s.backdrop === 'cover' && coverUrl"
      class="lyrics-backdrop"
      :style="{ backgroundImage: `url(${coverUrl})` }"
    />
    <div v-else-if="s.backdrop === 'gradient'" class="lyrics-gradient" />
    <div class="lyrics-shade" />

    <div class="lyrics-toolbar">
      <div class="lyrics-track">
        <div class="cover lyrics-mini-cover">
          <img
            v-if="coverUrl"
            loading="lazy"
            decoding="async"
            :src="coverUrl"
          />
          <Icon v-else name="note" :size="16" class="faint" />
        </div>
        <div style="min-width: 0">
          <div class="t-13 w-600 ellipsis lyrics-link" @click="openAlbum">
            {{ player.current?.title || "Текст песни" }}
          </div>
          <div class="lyrics-subtitle ellipsis lyrics-link" @click="openArtist">
            {{ artistLabel }}
          </div>
        </div>
      </div>

      <div class="lyrics-src">
        <button
          v-for="opt in sources"
          :key="opt.id"
          type="button"
          class="lyrics-src-btn"
          :class="{ on: pick === opt.id }"
          :disabled="!player.current"
          @click="choose(opt.id)"
        >
          {{ opt.label }}
        </button>
      </div>

      <div class="lyrics-tools">
        <div
          class="icon-btn round"
          data-no-drag
          @click="player.loadLyrics(true)"
        >
          <Icon name="repeat" :size="16" />
          <q-tooltip>Загрузить текст заново</q-tooltip>
        </div>
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
          <Icon name="settings" :size="17" />
          <q-tooltip>Вид текста</q-tooltip>
        </div>
        <div
          class="icon-btn round"
          :class="{ on: player.lyricsFullscreen }"
          data-no-drag
          @click="player.toggleLyricsFullscreen()"
        >
          <Icon
            :name="player.lyricsFullscreen ? 'restore' : 'maximize'"
            :size="15"
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
          <Icon name="album" :size="17" />
          <q-tooltip>Большая обложка</q-tooltip>
        </div>
        <div class="icon-btn round" data-no-drag @click="player.toggleLyrics()">
          <Icon name="close" :size="18" />
        </div>
      </div>
    </div>

    <LyricsSettingsPanel v-if="showSettings" />

    <div
      v-if="player.lyricsLoading && !lines.length"
      class="lyrics-empty lyrics-layer"
    >
      <q-spinner size="24px" color="primary" />
      <div class="lyrics-empty-sub">Ищем текст в {{ searchingIn }}…</div>
    </div>

    <div v-else-if="!lines.length" class="lyrics-empty lyrics-layer">
      <Icon name="lyrics" :size="26" />
      <div class="lyrics-empty-title">
        {{ player.lyricsError || "Текста для этого трека нет" }}
      </div>
      <div class="lyrics-empty-sub">
        Можно поискать в другом источнике: LRCLIB даёт строки с таймингами,
        Genius - полный текст с авторами.
      </div>
      <div class="lyrics-empty-actions">
        <button
          type="button"
          class="lyrics-chip"
          @click="choose('lrclib', true)"
        >
          Искать в LRCLIB
        </button>
        <button
          type="button"
          class="lyrics-chip"
          @click="choose('genius', true)"
        >
          Искать в Genius
        </button>
        <button type="button" class="lyrics-chip" @click="choose('auto', true)">
          Автоматически
        </button>
      </div>
    </div>

    <template v-else>
      <div
        class="lyrics-stage lyrics-layer"
        :class="{ 'no-artwork': !artworkVisible }"
        @wheel="onStageWheel"
      >
        <div v-if="artworkVisible" class="lyrics-artwork-column">
          <div
            class="lyrics-artwork"
            :class="{ playing: player.isPlaying && s.motion }"
            @click="player.toggleLyricsFullscreen()"
          >
            <img loading="lazy" decoding="async" :src="coverUrl" />
            <div class="lyrics-artwork-side">
              <button
                type="button"
                class="la-btn"
                :class="{ on: isLiked }"
                @click.stop="toggleLike"
              >
                <Icon :name="isLiked ? 'heartFilled' : 'heart'" :size="17" />
                <q-tooltip>
                  {{ isLiked ? "Убрать из любимых" : "Мне нравится" }}
                </q-tooltip>
              </button>
              <button
                type="button"
                class="la-btn"
                @click.stop="player.openFullscreen()"
              >
                <Icon name="album" :size="17" />
                <q-tooltip>Большая обложка</q-tooltip>
              </button>
            </div>

            <div class="lyrics-artwork-controls">
              <button
                type="button"
                class="la-btn"
                :class="{ on: player.shuffle }"
                @click.stop="player.toggleShuffle()"
              >
                <Icon name="shuffle" :size="16" />
                <q-tooltip>Перемешать</q-tooltip>
              </button>
              <button
                type="button"
                class="la-btn"
                :disabled="!player.hasPrev"
                @click.stop="player.prev()"
              >
                <Icon name="prev" :size="18" />
              </button>
              <button
                type="button"
                class="la-btn la-btn-main"
                @click.stop="player.toggle()"
              >
                <Icon :name="player.isPlaying ? 'pause' : 'play'" :size="22" />
              </button>
              <button
                type="button"
                class="la-btn"
                :disabled="!player.hasNext"
                @click.stop="player.next()"
              >
                <Icon name="next" :size="18" />
              </button>
              <button
                type="button"
                class="la-btn"
                :class="{ on: player.repeat !== 'off' }"
                @click.stop="player.cycleRepeat()"
              >
                <Icon
                  :name="player.repeat === 'one' ? 'repeatOne' : 'repeat'"
                  :size="16"
                />
                <q-tooltip>Повтор</q-tooltip>
              </button>
            </div>
          </div>
          <div class="lyrics-artwork-meta">
            <div class="lyrics-artwork-title">{{ player.current?.title }}</div>
            <div class="lyrics-subtitle ellipsis">{{ artistLabel }}</div>
          </div>
        </div>

        <q-scroll-area ref="scroller" class="lyrics-scroll">
          <div
            class="lyrics-lines"
            :class="[
              `align-${s.align}`,
              `hl-${s.highlight}`,
              `ann-${markMode}`,
              { motion: s.motion && synced, plain: !synced },
            ]"
          >
            <template v-if="synced">
              <button
                v-for="(line, i) in lines"
                :key="i"
                :ref="(el) => setLineRef(el, i)"
                class="lyric-line"
                :class="[lineClass(i), { annotated: !!lineAnnotations[i] }]"
                :style="i === player.activeLine ? fillStyle : undefined"
                @click="seekTo(line.time_ms)"
                @contextmenu.stop.prevent="onLineMenu($event, i)"
              >
                <span class="lyric-text">{{ line.text || "…" }}</span>
                <span
                  v-if="lineAnnotations[i]"
                  class="lyric-note"
                  title="Разбор строки"
                  @click.stop="openAnnotation(lineAnnotations[i])"
                >
                  <Icon name="info" :size="13" />
                </span>
              </button>
            </template>
            <div v-else class="lyrics-plain-text">
              <p
                v-for="(line, i) in lines"
                :key="i"
                :class="{
                  'lyric-part': isPart(line.text),
                  annotated: !isPart(line.text) && !!lineAnnotations[i],
                }"
                @click="
                  !isPart(line.text) &&
                  lineAnnotations[i] &&
                  openAnnotation(lineAnnotations[i])
                "
                @contextmenu.stop.prevent="onLineMenu($event, i)"
              >
                <span class="lyric-text">{{ partLabel(line.text) }}</span>
                <span
                  v-if="!isPart(line.text) && lineAnnotations[i]"
                  class="lyric-note"
                  title="Разбор строки"
                  @click.stop="openAnnotation(lineAnnotations[i])"
                >
                  <Icon name="info" :size="12" />
                </span>
              </p>
            </div>
          </div>
        </q-scroll-area>
      </div>

      <div v-if="footVisible" class="lyrics-foot">
        <span v-if="originTag && ui.settings.lyricsShowOrigin"
          >Источник: <b>{{ originTag }}</b></span
        >

        <template v-if="creditsVisible">
          <span v-if="credits.length" class="lyrics-people">
            <button
              v-for="person in credits"
              :key="`${person.id}-${person.role}-${person.name}`"
              type="button"
              class="lyrics-person"
              @click="openPerson(person)"
            >
              <img v-if="person.image" :src="person.image" alt="" />
              <span v-else class="lyrics-person-dot">{{
                person.name.slice(0, 1)
              }}</span>
              <span>{{ person.name }}</span>
              <i>{{ person.role }}</i>
            </button>
          </span>
          <span v-else-if="writers"
            >Авторы: <b>{{ writers }}</b></span
          >
        </template>
      </div>
    </template>

    <button
      v-if="hint"
      ref="hintEl"
      type="button"
      class="lyric-hint"
      :style="{ left: `${hint.x}px`, top: `${hint.y}px` }"
      @click.stop="openHint"
    >
      <Icon name="info" :size="14" />
      <span>Разбор строки</span>
    </button>

    <q-dialog v-model="annotationOpen">
      <div class="ann-card">
        <div class="ann-head">
          <div class="ann-title ellipsis">{{ player.current?.title }}</div>
          <div class="icon-btn round" @click="annotationOpen = false">
            <Icon name="close" :size="16" />
          </div>
        </div>
        <div v-if="activeAnnotation" class="ann-frag">
          {{ activeAnnotation.fragment }}
        </div>
        <p v-if="activeAnnotation" class="ann-text">
          {{ activeAnnotation.text }}
        </p>
        <div class="ann-foot">
          <span class="ann-by ellipsis">
            {{
              activeAnnotation?.authors?.length
                ? activeAnnotation.authors.map((a) => a.name).join(", ")
                : "Genius"
            }}
          </span>
          <a
            v-if="activeAnnotation?.url"
            class="ann-link"
            @click.prevent="openUrl(activeAnnotation.url)"
            >Источник</a
          >
        </div>
      </div>
    </q-dialog>

    <TrackMenu
      v-if="player.current"
      :track="player.current"
      :context-menu="true"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import type { QScrollArea } from "quasar";
import { useRoute, useRouter } from "vue-router";
import Icon from "@/components/Icon.vue";
import LyricsSettingsPanel from "@/components/lyrics/LyricsSettingsPanel.vue";
import TrackMenu from "@/components/TrackMenu.vue";
import { api } from "@/api/client";
import type { GeniusQuote } from "@/api/types";
import { audio } from "@/lib/audio";
import { artistNames } from "@/lib/format";
import { ORIGIN_LABEL } from "@/lib/lyricsSource";
import { useGeniusStore } from "@/stores/genius";
import { useLibraryStore } from "@/stores/library";
import { usePlayerStore } from "@/stores/player/index";
import { useUiStore } from "@/stores/ui/index";

type Credit = { id: number; name: string; role: string; image: string };

type SourcePick = "auto" | "lrclib" | "genius";

const player = usePlayerStore();
const router = useRouter();
const route = useRoute();
const ui = useUiStore();
const genius = useGeniusStore();
const library = useLibraryStore();

const s = computed(() => ({
  fontSize: ui.settings.lyricsFontSize,
  lineHeight: ui.settings.lyricsLineHeight,
  weight: ui.settings.lyricsWeight,
  backgroundBlur: ui.settings.lyricsBackgroundBlur,
  backgroundOpacity: ui.settings.lyricsBackgroundOpacity,
  lineBlur: ui.settings.lyricsLineBlur,
  inactive: ui.settings.lyricsInactive,
  align: ui.settings.lyricsAlign,
  backdrop: ui.settings.lyricsBackdrop,
  highlight: ui.settings.lyricsHighlight,
  glow: ui.settings.lyricsGlow,
  showArtwork: ui.settings.lyricsShowArtwork,
  motion: ui.settings.lyricsMotion,
  annotations: ui.settings.lyricsAnnotations,
  annotationMark: ui.settings.lyricsAnnotationMark,
}));

const markMode = computed(() =>
  s.value.annotations ? s.value.annotationMark : "off",
);

const showSettings = ref(false);
const scroller = ref<QScrollArea | null>(null);
const lineRefs = ref<Array<HTMLElement | null>>([]);

const ARTWORK_MIN_WIDTH = 900;
const root = ref<HTMLElement | null>(null);
const panelWidth = ref(1400);
let sizeObserver: ResizeObserver | null = null;

watch(root, (el) => {
  sizeObserver?.disconnect();
  sizeObserver = null;
  if (!el || typeof ResizeObserver === "undefined") return;
  panelWidth.value = el.clientWidth || panelWidth.value;
  sizeObserver = new ResizeObserver((entries) => {
    const width = entries[0]?.contentRect.width;
    if (width) panelWidth.value = width;
  });
  sizeObserver.observe(el);
});

onBeforeUnmount(() => {
  sizeObserver?.disconnect();
  sizeObserver = null;
});

const lines = computed(() => player.lyrics?.lines || []);
const synced = computed(() => Boolean(player.lyrics?.synced));
const coverUrl = computed(() => player.current?.cover_url || "");
const artistLabel = computed(() =>
  player.current ? artistNames(player.current.artists) : "",
);
const writers = computed(() => player.lyrics?.writers?.join(", ") || "");

const artworkVisible = computed(
  () =>
    s.value.showArtwork &&
    Boolean(coverUrl.value) &&
    panelWidth.value >= ARTWORK_MIN_WIDTH,
);

const isLiked = computed(() =>
  player.current ? library.liked(player.current.id) : false,
);

function toggleLike() {
  if (player.current) void library.toggleLike(player.current);
}

const credits = computed<Credit[]>(() => {
  const current = player.current;
  if (!current || genius.songKey !== String(current.id)) return [];
  return (genius.people as Credit[])
    .filter((person) => person.id > 0)
    .slice(0, 8);
});

function openPerson(person: Credit) {
  void router.push({
    name: "genius-artist",
    params: { id: String(person.id) },
    query: { role: person.role },
  });
}

const annotations = computed<GeniusQuote[]>(() => {
  const current = player.current;
  if (!current || genius.songKey !== String(current.id)) return [];
  return genius.song?.quotes || [];
});

function normText(value: string): string {
  return (value || "")
    .toLowerCase()
    .replace(/[^\p{L}\p{N} ]+/gu, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function annotationFor(text: string): GeniusQuote | null {
  const line = normText(text);
  if (line.length < 6) return null;

  for (const quote of annotations.value) {
    const frag = normText(quote.fragment);
    if (!frag) continue;
    if (frag.includes(line) || line.includes(frag)) return quote;

    // Фрагмент с Genius часто из нескольких строк, а в LRCLIB они разделены.
    const parts = (quote.fragment || "")
      .split(/\r?\n/)
      .map(normText)
      .filter((part) => part.length >= 6);
    if (parts.some((part) => part === line || part.includes(line))) {
      return quote;
    }
  }
  return null;
}

const lineAnnotations = computed<Array<GeniusQuote | null>>(() => {
  if (!s.value.annotations) return lines.value.map(() => null);
  return lines.value.map((line) => annotationFor(line.text));
});

const activeAnnotation = ref<GeniusQuote | null>(null);
const annotationOpen = computed({
  get: () => !!activeAnnotation.value,
  set: (value: boolean) => {
    if (!value) activeAnnotation.value = null;
  },
});

function openAnnotation(quote: GeniusQuote | null) {
  if (quote) activeAnnotation.value = quote;
  hint.value = null;
}

const hintEl = ref<HTMLElement | null>(null);
const hint = ref<{ x: number; y: number; quote: GeniusQuote } | null>(null);

function onLineMenu(event: MouseEvent, index: number) {
  const quote = lineAnnotations.value[index];
  if (!quote) {
    hint.value = null;
    return;
  }
  hint.value = {
    x: Math.min(event.clientX, window.innerWidth - 190),
    y: Math.min(event.clientY, window.innerHeight - 60),
    quote,
  };
}

function openHint() {
  openAnnotation(hint.value?.quote ?? null);
}

function closeHint(event?: Event) {
  if (!hint.value) return;
  const el = hintEl.value;
  if (event && el && event.composedPath().includes(el)) return;
  hint.value = null;
}

function onHintKey(event: KeyboardEvent) {
  if (event.key === "Escape") hint.value = null;
}

window.addEventListener("pointerdown", closeHint);
window.addEventListener("wheel", closeHint, { passive: true });
window.addEventListener("keydown", onHintKey);

onBeforeUnmount(() => {
  window.removeEventListener("pointerdown", closeHint);
  window.removeEventListener("wheel", closeHint);
  window.removeEventListener("keydown", onHintKey);
});

watch(
  () => player.current?.id,
  () => {
    hint.value = null;
    activeAnnotation.value = null;
  },
);

function openUrl(url: string) {
  if (url) void api.openExternal(url);
}

watch(
  () => [player.showLyrics, player.current?.id] as const,
  ([shown, id]) => {
    if (shown && id && genius.ready) void genius.fetchSong(player.current);
  },
  { immediate: true },
);

function isPart(text: string): boolean {
  const value = (text || "").trim();
  return value.length > 2 && value.startsWith("[") && value.endsWith("]");
}

function partLabel(text: string): string {
  const value = (text || "").trim();
  return isPart(value) ? value.slice(1, -1).trim() : text;
}

const sources: Array<{ id: SourcePick; label: string }> = [
  { id: "auto", label: "Авто" },
  { id: "lrclib", label: "LRCLIB" },
  { id: "genius", label: "Genius" },
];

const pick = computed<SourcePick>(
  () => (player.lyricsPick as SourcePick | null) ?? "auto",
);

const searchingIn = computed(() => {
  if (pick.value === "lrclib") return "LRCLIB";
  if (pick.value === "genius") return "Genius";
  return "LRCLIB и Genius";
});

const originTag = computed(() => {
  const origin = player.lyricsOrigin;
  if (!origin || !lines.value.length) return "";
  return ORIGIN_LABEL[origin];
});

const creditsVisible = computed(
  () =>
    ui.settings.lyricsShowCredits &&
    (credits.value.length > 0 || Boolean(writers.value)),
);

const footVisible = computed(
  () =>
    creditsVisible.value ||
    (ui.settings.lyricsShowOrigin && Boolean(originTag.value)),
);

async function choose(id: SourcePick, force = false) {
  await player.setLyricsSource(id === "auto" ? null : id, force);
}

const visualStyle = computed(() => ({
  "--lyrics-size": `${s.value.fontSize}px`,
  "--lyrics-line-height": `${s.value.lineHeight}`,
  "--lyrics-weight": `${s.value.weight}`,
  "--lyrics-inactive": `${s.value.inactive / 100}`,
  "--lyrics-bg-blur": `${s.value.backgroundBlur}px`,
  "--lyrics-bg-opacity": `${s.value.backgroundOpacity / 100}`,
  "--lyrics-line-blur": `${s.value.lineBlur}px`,
}));

const fill = ref(0);
const fillStyle = computed(() => ({ "--lp": `${fill.value}%` }));
const karaoke = computed(
  () => s.value.highlight === "karaoke" && synced.value && player.showLyrics,
);

let raf = 0;

function tick() {
  const index = player.activeLine;
  const list = lines.value;
  const line = index >= 0 ? list[index] : null;
  if (line) {
    const start = line.time_ms;
    const end = list[index + 1]?.time_ms ?? start + 4200;
    const span = Math.max(320, end - start);
    const now = audio.currentTime * 1000;
    fill.value = Math.min(100, Math.max(0, ((now - start) / span) * 100));
  } else {
    fill.value = 0;
  }
  raf = requestAnimationFrame(tick);
}

function stopTick() {
  if (!raf) return;
  cancelAnimationFrame(raf);
  raf = 0;
  fill.value = 0;
}

watch(
  karaoke,
  (on) => {
    if (on && !raf) raf = requestAnimationFrame(tick);
    if (!on) stopTick();
  },
  { immediate: true },
);

onBeforeUnmount(stopTick);

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

function onStageWheel(event: WheelEvent) {
  const area = scroller.value;
  if (!area) return;
  const target = area.getScrollTarget() as HTMLElement | null;
  if (!target) return;
  const max = target.scrollHeight - target.clientHeight;
  if (max <= 0) return;
  event.preventDefault();
  event.stopPropagation();
  let step = event.deltaY;
  if (event.deltaMode === 1) step *= 18;
  else if (event.deltaMode === 2) step *= target.clientHeight;
  target.scrollTop = Math.min(max, Math.max(0, target.scrollTop + step));
}

watch(
  () => player.activeLine,
  (i) => {
    if (!synced.value || i < 0) return;
    const el = lineRefs.value[i];
    const area = scroller.value;
    if (!el || !area) return;
    const target = Math.max(0, el.offsetTop - area.$el.clientHeight * 0.42);
    area.setScrollPosition("vertical", target, s.value.motion ? 520 : 0);
  },
);

watch(
  () => route.fullPath,
  () => {
    if (player.lyricsFullscreen) player.closeLyrics();
  },
);
</script>
