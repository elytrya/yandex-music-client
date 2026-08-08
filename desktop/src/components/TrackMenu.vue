<template>
  <q-menu
    class="menu"
    :context-menu="contextMenu"
    :touch-position="contextMenu"
    :anchor="contextMenu ? undefined : 'bottom right'"
    :self="contextMenu ? undefined : 'top right'"
  >
    <div class="menu-body">
      <div class="menu-item" v-close-popup @click="library.toggleLike(track)">
        <Icon :name="isLiked ? 'heartFilled' : 'heart'" :size="18" />
        <span>{{ isLiked ? "Убрать из «Мне нравится»" : "Нравится" }}</span>
      </div>

      <div class="menu-item" v-close-popup @click="player.waveByTrack(track)">
        <Icon name="wave" :size="18" />
        <span>Моя волна по треку</span>
      </div>

      <div class="menu-item" v-close-popup @click="playSimilar">
        <Icon name="paths" :size="18" />
        <span>Похожие треки</span>
      </div>

      <div class="menu-item" v-close-popup @click="download">
        <Icon name="download" :size="18" />
        <span>Скачать</span>
      </div>

      <div class="menu-item" v-close-popup @click="player.playNext(track)">
        <Icon name="playNext" :size="18" />
        <span>Играть следующим</span>
      </div>

      <div class="menu-item" v-close-popup @click="player.enqueue(track)">
        <Icon name="addQueue" :size="18" />
        <span>Добавить в конец очереди</span>
      </div>

      <div class="menu-item" v-close-popup @click="library.dislike(track)">
        <Icon name="heartOff" :size="18" />
        <span>Не нравится</span>
      </div>

      <div class="menu-item">
        <Icon name="playlistAdd" :size="18" />
        <span class="col">Добавить в плейлист</span>
        <Icon name="chevronRight" :size="15" />
        <q-menu
          anchor="top end"
          self="top start"
          class="panel menu"
          @show="library.loadMembership()"
        >
          <div class="menu-body" style="max-height: 320px; overflow-y: auto">
            <div
              v-for="pl in library.playlists"
              :key="pl.kind"
              class="menu-item"
              :class="{ picked: library.inPlaylist(pl.kind, track.id) }"
              v-close-popup
              @click="library.togglePlaylistTrack(pl.kind, track)"
            >
              <div class="cover menu-cover">
                <img
                  v-if="pl.cover_url"
                  :src="pl.cover_url"
                  loading="lazy"
                  decoding="async"
                />
                <Icon v-else name="queue" :size="14" class="faint" />
              </div>
              <span class="ellipsis col">{{ pl.title }}</span>
              <Icon
                v-if="library.inPlaylist(pl.kind, track.id)"
                name="check"
                :size="16"
                class="pl-check"
              />
            </div>
            <div
              v-if="library.membershipLoading"
              class="faint t-11 q-px-sm q-pb-xs"
            >
              сверяю плейлисты…
            </div>
            <div v-if="!library.playlists.length" class="faint t-12 q-pa-sm">
              Нет плейлистов
            </div>
          </div>
        </q-menu>
      </div>

      <div class="menu-item" v-close-popup @click="showLyrics">
        <Icon name="lyrics" :size="18" />
        <span>Показать текст песни</span>
      </div>

      <div class="menu-item" v-close-popup @click="share">
        <Icon name="share" :size="18" />
        <span>Поделиться</span>
      </div>

      <div
        v-if="track.album_id"
        class="menu-item"
        v-close-popup
        @click="goAlbum"
      >
        <Icon name="album" :size="18" />
        <span>Перейти к альбому</span>
      </div>

      <div
        v-if="track.artists.length === 1"
        class="menu-item"
        v-close-popup
        @click="goArtist(track.artists[0]?.id)"
      >
        <Icon name="artist" :size="18" />
        <span>Перейти к исполнителю</span>
      </div>

      <div v-else-if="track.artists.length > 1" class="menu-item">
        <Icon name="artist" :size="18" />
        <span class="col">Перейти к исполнителям</span>
        <Icon name="chevronRight" :size="15" />
        <q-menu anchor="top end" self="top start" class="panel menu">
          <div class="menu-body">
            <div
              v-for="a in track.artists"
              :key="a.id"
              class="menu-item"
              v-close-popup
              @click="goArtist(a.id)"
            >
              <span class="ellipsis">{{ a.name }}</span>
            </div>
          </div>
        </q-menu>
      </div>

      <template v-if="playlistKind !== null && playlistKind !== undefined">
        <div class="sep" style="margin: 6px 4px" />
        <div class="menu-item danger" v-close-popup @click="emit('remove')">
          <Icon name="trash" :size="18" />
          <span>Удалить из плейлиста</span>
        </div>
      </template>

      <div class="sep" style="margin: 6px 4px" />

      <div
        v-if="cleanAvailable"
        class="menu-item"
        v-close-popup
        @click="toggleCensorVersion"
      >
        <Icon :name="playsOriginal ? 'heartOff' : 'apple'" :size="18" />
        <span>{{
          playsOriginal
            ? "Играть версию без цензуры"
            : "Играть версию с Яндекс Музыки"
        }}</span>
      </div>

      <div class="menu-item" v-close-popup @click="openReplace">
        <Icon name="upload" :size="18" />
        <span>{{
          hasReplacement ? "Своя версия — настроить" : "Подменить версию"
        }}</span>
      </div>

      <div class="menu-item" v-close-popup @click="openInfo">
        <Icon name="info" :size="18" />
        <span>О треке</span>
      </div>
    </div>
  </q-menu>

  <q-dialog v-model="replaceOpen">
    <div class="info-card">
      <div class="row items-start no-wrap" style="gap: 12px">
        <div class="col" style="min-width: 0">
          <span class="info-title ellipsis">Своя версия трека</span>
          <div class="dim t-12 ellipsis">{{ track.title }}</div>
        </div>
        <div class="icon-btn round" v-close-popup>
          <Icon name="close" :size="16" />
        </div>
      </div>

      <p class="dim t-12 q-pt-md" style="line-height: 1.45">
        Вставь прямую ссылку на аудиофайл или путь на диске — именно он будет
        играть вместо этого трека. Подмена сохранится и переживёт перезапуск.
      </p>

      <div class="field q-mt-md">
        <input
          v-model="replaceSource"
          type="text"
          spellcheck="false"
          placeholder="https://…/track.mp3 или C:\Music\track.flac"
        />
      </div>

      <div class="field q-mt-sm">
        <input
          v-model="replaceLabel"
          type="text"
          spellcheck="false"
          placeholder="Подпись: например, «без цензуры»"
        />
      </div>

      <div class="row items-center q-mt-md" style="gap: 8px">
        <button class="btn-solid" type="button" @click="saveReplace">
          Сохранить
        </button>
        <button
          v-if="hasReplacement"
          class="btn"
          type="button"
          @click="dropReplace"
        >
          Убрать подмену
        </button>
      </div>
    </div>
  </q-dialog>

  <q-dialog v-model="info">
    <div class="info-card">
      <div class="row items-start no-wrap" style="gap: 12px">
        <div
          class="col row items-center no-wrap"
          style="gap: 7px; min-width: 0"
        >
          <span class="info-title ellipsis">{{
            details?.title || track.title
          }}</span>
          <Icon v-if="details?.explicit" name="info" :size="16" class="faint" />
        </div>
        <div class="icon-btn round" v-close-popup>
          <Icon name="close" :size="16" />
        </div>
      </div>

      <div v-if="loading" class="row justify-center q-py-xl">
        <q-spinner size="22px" color="grey-6" />
      </div>

      <div v-else-if="error" class="dim t-13 q-pt-lg">{{ error }}</div>

      <div v-else class="info-list">
        <div v-for="row in rows" :key="row.label" class="info-row">
          <div class="info-label">{{ row.label }}</div>
          <div class="info-value">{{ row.value }}</div>
        </div>
      </div>
    </div>
  </q-dialog>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { Notify } from "quasar";
import { useRouter } from "vue-router";
import Icon from "@/components/Icon.vue";
import { api } from "@/api/client";
import type { Track, TrackInfo } from "@/api/types";
import { openTrackLyrics } from "@/lib/dialogs";
import { artistNames, formatDuration } from "@/lib/format";
import { useLibraryStore } from "@/stores/library";
import { usePlayerStore } from "@/stores/player/index";
import { clearOverride, getOverride, setOverride } from "@/lib/censorOverrides";
import { ensureCensorList, isCensored, prefersOriginal } from "@/lib/censor";
import { useUiStore } from "@/stores/ui/index";

const props = withDefaults(
  defineProps<{
    track: Track;
    playlistKind?: number | string | null;
    contextMenu?: boolean;
  }>(),
  { playlistKind: null, contextMenu: false },
);
const emit = defineEmits<{ remove: [] }>();

defineOptions({ inheritAttrs: false });

const contextMenu = computed(() => props.contextMenu);

const router = useRouter();
const library = useLibraryStore();
const player = usePlayerStore();
const info = ref(false);
const loading = ref(false);
const error = ref("");
const details = ref<TrackInfo | null>(null);

function formatReleaseDate(value: string | null | undefined): string {
  if (!value) return "";
  const raw = value.trim();
  if (!raw) return "";
  const parsed = new Date(raw);
  if (Number.isNaN(parsed.getTime())) {
    const match = /^(\d{4})-(\d{2})-(\d{2})/.exec(raw);
    return match ? `${match[3]}.${match[2]}.${match[1]}` : raw;
  }
  const hasTime = /\d{2}:\d{2}/.test(raw);
  const date = parsed.toLocaleDateString("ru-RU", {
    day: "numeric",
    month: "long",
    year: "numeric",
  });
  if (!hasTime) return date;
  const time = parsed.toLocaleTimeString("ru-RU", {
    hour: "2-digit",
    minute: "2-digit",
  });
  return `${date}, ${time}`;
}

const rows = computed(() => {
  const d = details.value;
  const list: Array<{ label: string; value: string }> = [];
  if (!d) return list;
  if (d.version) list.push({ label: "Версия", value: d.version });
  if (d.label) list.push({ label: "Лейбл", value: d.label });
  if (d.artists.length)
    list.push({ label: "Исполнитель", value: d.artists.join(", ") });
  if (d.composers.length)
    list.push({ label: "Автор музыки", value: d.composers.join(", ") });
  if (d.lyricists.length)
    list.push({ label: "Автор текста", value: d.lyricists.join(", ") });
  if (d.source) list.push({ label: "Источник фонограммы", value: d.source });
  if (d.album) list.push({ label: "Альбом", value: d.album });
  const release = formatReleaseDate(d.release_date);
  if (release) list.push({ label: "Дата выхода", value: release });
  else if (d.year) list.push({ label: "Год", value: String(d.year) });
  if (d.duration_ms)
    list.push({ label: "Длительность", value: formatDuration(d.duration_ms) });
  return list;
});

async function playSimilar() {
  try {
    const list = await api.similarTracks(props.track.id);
    if (!list.length) {
      Notify.create({ message: "Похожих треков не нашлось" });
      return;
    }
    await player.playQueue(list, 0);
    Notify.create({ message: `Похожих треков: ${list.length}` });
  } catch {
    Notify.create({ message: "Не удалось загрузить похожие треки" });
  }
}

async function openInfo() {
  info.value = true;
  error.value = "";
  loading.value = true;
  try {
    details.value = await api.trackInfo(props.track.id);
    if (!rows.value.length)
      error.value = "Яндекс не отдал информацию о этом треке";
  } catch (e) {
    error.value =
      e instanceof Error ? e.message : "Не удалось загрузить информацию";
  } finally {
    loading.value = false;
  }
}

const track = computed(() => props.track);
const playlistKind = computed(() => props.playlistKind);
const isLiked = computed(() => library.liked(props.track.id));

const replaceOpen = ref(false);
const replaceSource = ref("");
const replaceLabel = ref("");
const replaceSaved = ref(0);

const ui = useUiStore();
const censorTick = ref(0);

const cleanAvailable = computed(() => {
  void censorTick.value;
  return ui.settings.censorBypass && isCensored(props.track.id);
});

const playsOriginal = computed(() => {
  void censorTick.value;
  return prefersOriginal(props.track.id);
});

async function toggleCensorVersion() {
  const next = !playsOriginal.value;
  await player.useOriginalVersion(props.track.id, next);
  censorTick.value += 1;
  Notify.create({
    message: next
      ? "Играет версия с Яндекс Музыки"
      : "Играет версия без цензуры",
  });
}

const hasReplacement = computed(() => {
  void replaceSaved.value;
  return Boolean(getOverride(props.track.id));
});

function openReplace() {
  const saved = getOverride(props.track.id);
  replaceSource.value = saved?.source ?? "";
  replaceLabel.value = saved?.label ?? "без цензуры";
  replaceOpen.value = true;
}

function saveReplace() {
  const source = replaceSource.value.trim();
  if (!source) {
    Notify.create({
      type: "negative",
      message: "Нужна ссылка или путь к файлу",
    });
    return;
  }
  setOverride(props.track.id, source, replaceLabel.value || "своя версия");
  replaceSaved.value += 1;
  replaceOpen.value = false;
  Notify.create({ message: "Подмена сохранена" });
  if (player.current?.id === props.track.id) void player.loadCurrent();
}

function dropReplace() {
  clearOverride(props.track.id);
  replaceSaved.value += 1;
  replaceSource.value = "";
  replaceOpen.value = false;
  Notify.create({ message: "Подмена убрана" });
  if (player.current?.id === props.track.id) void player.loadCurrent();
}

async function download() {
  Notify.create({ message: "Скачивание началось" });
  try {
    const path = await api.download(
      props.track.id,
      `${artistNames(props.track.artists)} - ${props.track.title}`,
    );
    Notify.create({ type: "positive", message: `Сохранил: ${path}` });
  } catch (e) {
    Notify.create({
      type: "negative",
      message: e instanceof Error ? e.message : "Не удалось скачать трек",
    });
  }
}

async function share() {
  const link = props.track.album_id
    ? `https://music.yandex.ru/album/${props.track.album_id}/track/${props.track.id}`
    : `https://music.yandex.ru/track/${props.track.id}`;
  try {
    await navigator.clipboard.writeText(link);
    Notify.create({ message: "Ссылка скопирована" });
  } catch {
    Notify.create({ message: link });
  }
}

function showLyrics() {
  openTrackLyrics(props.track);
}

function goAlbum() {
  if (props.track.album_id) void router.push(`/album/${props.track.album_id}`);
}

function goArtist(id?: string) {
  if (id) void router.push(`/artist/${id}`);
}

onMounted(() => {
  if (!library.playlists.length) void library.init();
  if (ui.settings.censorBypass) {
    void ensureCensorList().then(() => {
      censorTick.value += 1;
    });
  }
});
</script>
