<template>
  <q-page class="playlist-page">
    <div class="playlist-page-header" :class="{ collapsed: headerCollapsed }">
      <div class="page-head">
        <div
          class="cover head-cover playlist-cover"
          :title="tracks.length ? 'Слушать плейлист' : ''"
          @click="playFrom(0)"
        >
          <img loading="lazy" decoding="async" v-if="cover" :src="cover" />
          <Icon v-else name="queue" :size="42" class="faint" />
          <div v-if="tracks.length" class="playlist-cover-play">
            <Icon name="play" :size="20" />
          </div>
        </div>

        <div class="head-info">
          <div class="head-kind">
            Плейлист
            <span v-if="isPinned" class="head-pin-mark">
              <Icon name="pin" :size="12" />
              закреплён
            </span>
          </div>

          <input
            v-if="editing"
            ref="titleInput"
            v-model="draftTitle"
            class="head-title head-title-input"
            type="text"
            spellcheck="false"
            maxlength="120"
            @keyup.enter="commitTitle"
            @keyup.esc="cancelTitle"
            @blur="commitTitle"
          />
          <button
            v-else
            class="head-title head-title-edit"
            type="button"
            title="Нажми, чтобы переименовать"
            @click="beginEdit"
          >
            <span>{{ title }}</span>
            <Icon class="head-title-pen" name="name" :size="20" />
          </button>

          <div class="head-meta">
            {{ plural(tracks.length, "трек", "трека", "треков") }}
            <span v-if="totalLabel" class="faint"> · {{ totalLabel }}</span>
          </div>

          <div class="head-actions">
            <button
              class="btn-solid"
              type="button"
              :disabled="!tracks.length"
              @click="playFrom(0)"
            >
              <Icon name="play" :size="15" />
              <span>Слушать</span>
            </button>

            <button
              class="btn"
              type="button"
              :disabled="!tracks.length"
              @click="shufflePlay"
            >
              <Icon name="shuffle" :size="15" />
              <span>Перемешать</span>
            </button>

            <button
              v-if="downloading"
              class="btn danger-btn"
              type="button"
              title="Отменить загрузку плейлиста"
              @click="cancelDownload"
            >
              <q-spinner size="15px" />
              <span>{{ downloadLabel }}</span>
              <Icon name="close" :size="14" />
            </button>

            <button
              v-else
              class="btn"
              type="button"
              :class="{ on: fullyDownloaded }"
              :disabled="!tracks.length || fullyDownloaded"
              :title="
                fullyDownloaded
                  ? 'Плейлист уже скачан целиком'
                  : 'Скачать все треки плейлиста в отдельную папку'
              "
              @click="downloadAll"
            >
              <Icon :name="fullyDownloaded ? 'check' : 'download'" :size="15" />
              <span>{{ downloadLabel }}</span>
            </button>

            <button
              class="btn"
              type="button"
              :class="{ on: isPinned }"
              :title="
                isPinned
                  ? 'Открепить от боковой панели'
                  : 'Закрепить в боковой панели'
              "
              @click="library.togglePin(kindNumber)"
            >
              <Icon :name="isPinned ? 'pinOff' : 'pin'" :size="15" />
              <span>{{ isPinned ? "Открепить" : "Закрепить" }}</span>
            </button>

            <button class="icon-btn" type="button" title="Ещё">
              <Icon name="more" :size="18" />
              <q-menu class="menu" anchor="bottom left" self="top left">
                <div class="menu-body" style="min-width: 232px">
                  <div class="menu-item" v-close-popup @click="beginEdit">
                    <Icon name="name" :size="17" />
                    <span>Переименовать</span>
                  </div>

                  <div
                    class="menu-item"
                    v-close-popup
                    @click="enqueueAll"
                    :class="{ disabled: !tracks.length }"
                  >
                    <Icon name="addQueue" :size="17" />
                    <span>Добавить всё в очередь</span>
                  </div>

                  <div
                    class="menu-item"
                    v-close-popup
                    :class="{ disabled: !tracks.length || fullyDownloaded }"
                    @click="downloadAll"
                  >
                    <Icon name="download" :size="17" />
                    <span>{{
                      fullyDownloaded ? "Уже скачано" : "Скачать всё"
                    }}</span>
                  </div>

                  <div
                    v-if="downloading"
                    class="menu-item danger"
                    v-close-popup
                    @click="cancelDownload"
                  >
                    <Icon name="close" :size="17" />
                    <span>Отменить загрузку</span>
                  </div>

                  <div class="menu-item" v-close-popup @click="playRecommended">
                    <Icon name="wave" :size="17" />
                    <span>Похожее на плейлист</span>
                  </div>

                  <div class="menu-sep" />

                  <div class="menu-item" v-close-popup @click="setPublic(true)">
                    <Icon name="share" :size="17" />
                    <span>Сделать публичным</span>
                  </div>

                  <div
                    class="menu-item"
                    v-close-popup
                    @click="setPublic(false)"
                  >
                    <Icon name="person" :size="17" />
                    <span>Сделать личным</span>
                  </div>

                  <div class="menu-sep" />

                  <div class="menu-item" v-close-popup @click="clearTracks">
                    <Icon name="filter" :size="17" />
                    <span>Очистить плейлист</span>
                  </div>

                  <div
                    class="menu-item danger"
                    v-close-popup
                    @click="removePlaylist"
                  >
                    <Icon name="trash" :size="17" />
                    <span>Удалить плейлист</span>
                  </div>
                </div>
              </q-menu>
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="loading" class="playlist-loading">
      <q-spinner size="24px" color="primary" />
    </div>

    <div v-else class="playlist-tracks-scroll">
      <div v-if="selectedIds.length" class="select-float">
        <button
          class="track-check-box"
          :class="{ on: allVisibleSelected }"
          type="button"
          title="Выбрать все"
          @click="toggleSelectAllRows"
        >
          <Icon v-if="allVisibleSelected" name="check" :size="13" />
        </button>

        <div class="select-bar-count t-12 w-600">
          {{ plural(selectedIds.length, "трек", "трека", "треков") }}
        </div>

        <div class="select-bar-actions">
          <button
            class="btn"
            type="button"
            :disabled="!selectedIds.length"
            @click="playRowSelection"
          >
            <Icon name="play" :size="14" />
            <span>Слушать</span>
          </button>

          <button
            class="btn"
            type="button"
            :disabled="!selectedIds.length"
            @click="enqueueRowSelection"
          >
            <Icon name="addQueue" :size="14" />
            <span>В очередь</span>
          </button>

          <button class="btn" type="button" :disabled="!selectedIds.length">
            <Icon name="playlistAdd" :size="14" />
            <span>В плейлист</span>
            <q-menu class="menu" anchor="bottom start" self="top start">
              <div class="menu-body">
                <div v-if="!otherPlaylists.length" class="menu-item disabled">
                  <span>Нет других плейлистов</span>
                </div>
                <div
                  v-for="playlist in otherPlaylists"
                  :key="playlist.kind"
                  class="menu-item"
                  v-close-popup
                  @click="addRowSelectionTo(playlist.kind)"
                >
                  <div class="cover menu-cover">
                    <img
                      v-if="playlist.cover_url"
                      :src="playlist.cover_url"
                      loading="lazy"
                      decoding="async"
                    />
                    <Icon v-else name="queue" :size="14" class="faint" />
                  </div>
                  <span class="ellipsis">{{ playlist.title }}</span>
                </div>
              </div>
            </q-menu>
          </button>

          <button
            class="btn"
            type="button"
            :disabled="!selectedIds.length || downloading"
            @click="downloadRowSelection"
          >
            <Icon name="download" :size="14" />
            <span>Скачать</span>
          </button>

          <button
            v-if="canReorder"
            class="btn danger-btn"
            type="button"
            :disabled="!selectedIds.length"
            @click="removeRowSelection"
          >
            <Icon name="trash" :size="14" />
            <span>Удалить</span>
          </button>

          <button
            class="icon-btn sm"
            type="button"
            title="Снять выделение"
            @click="clearRowSelection"
          >
            <Icon name="close" :size="15" />
          </button>
        </div>
      </div>

      <div v-if="tracks.length" class="playlist-toolbar">
        <div class="field playlist-filter">
          <Icon name="search" :size="15" class="faint" />
          <input
            v-model="filter"
            type="text"
            spellcheck="false"
            placeholder="Найти в плейлисте"
          />
          <button
            v-if="filter"
            class="icon-btn xs"
            type="button"
            title="Сбросить"
            @click="filter = ''"
          >
            <Icon name="close" :size="13" />
          </button>
        </div>

        <button class="btn" type="button" title="Сортировка">
          <Icon name="filter" :size="15" />
          <span>{{ sortLabel }}</span>
          <Icon name="chevronDown" :size="13" />
          <q-menu class="menu" anchor="bottom left" self="top left">
            <div class="menu-body" style="min-width: 220px">
              <div
                v-for="option in sortOptions"
                :key="option.id"
                class="menu-item"
                v-close-popup
                @click="sortMode = option.id"
              >
                <Icon
                  :name="sortMode === option.id ? 'check' : 'name'"
                  :size="16"
                />
                <span>{{ option.label }}</span>
              </div>
            </div>
          </q-menu>
        </button>

        <button
          class="btn"
          type="button"
          :title="
            sortDir === 'asc'
              ? 'Сейчас по возрастанию - нажми для убывания'
              : 'Сейчас по убыванию - нажми для возрастания'
          "
          @click="sortDir = sortDir === 'asc' ? 'desc' : 'asc'"
        >
          <span class="t-13 w-600">{{ sortDir === "asc" ? "↑" : "↓" }}</span>
          <span>{{
            sortDir === "asc" ? "По возрастанию" : "По убыванию"
          }}</span>
        </button>

        <div class="col" />

        <div v-if="filter" class="faint t-12">
          {{ plural(rows.length, "совпадение", "совпадения", "совпадений") }}
        </div>
        <div v-else-if="canReorder" class="faint t-12">
          Перетаскивай треки, чтобы менять порядок
        </div>
      </div>

      <div class="head-row">
        <div style="width: 26px" />
        <div style="width: 20px; text-align: center">#</div>
        <div style="width: 38px" />
        <div class="col">Название</div>
        <div class="gt-sm" style="width: 190px">Альбом</div>
        <div style="width: 84px" />
        <div style="width: 40px; text-align: right">Время</div>
      </div>

      <q-virtual-scroll
        v-if="rows.length"
        ref="virtualRef"
        v-slot="{ item }"
        class="track-virtual"
        :items="rows"
        :virtual-scroll-item-size="52"
        :virtual-scroll-slice-size="20"
        @scroll="onTracksScroll"
      >
        <TrackRow
          :key="`${item.track.id}-${item.index}`"
          :track="item.track"
          :index="item.index"
          :playlist-kind="props.kind"
          :dragging="dragFrom === item.index"
          :drop-target="dragOver === item.index && dragFrom !== item.index"
          :selectable="true"
          :selected="isRowSelected(item.track)"
          :downloaded="localIds.has(item.track.id)"
          @play="playFrom(item.index)"
          @remove="removeAt(item.index)"
          @reorder-start="startReorder"
          @toggle-select="toggleRow(item.track, $event)"
        />
      </q-virtual-scroll>

      <div v-else-if="filter" class="playlist-empty">
        <Icon name="search" :size="22" class="faint" />
        <div class="t-13">По запросу «{{ filter }}» ничего нет</div>
        <button class="btn" type="button" @click="filter = ''">
          Сбросить поиск
        </button>
      </div>

      <div v-else class="playlist-empty">
        <Icon name="queue" :size="22" class="faint" />
        <div class="t-13">Плейлист пуст</div>
        <div class="faint t-12">
          Добавляй треки через меню трека - «Добавить в плейлист»
        </div>
        <button class="btn" type="button" @click="playRecommended">
          <Icon name="wave" :size="15" />
          <span>Подобрать похожее</span>
        </button>
      </div>
    </div>

    <q-dialog v-model="pickerOpen">
      <div class="app-modal picker-modal">
        <div class="picker-head">
          <div>
            <div class="app-modal-title">Выбор треков</div>
            <div class="app-modal-text">
              Отметь треки и примени действие сразу ко всем.
            </div>
          </div>
          <button
            class="icon-btn sm"
            type="button"
            title="Закрыть"
            @click="pickerOpen = false"
          >
            <Icon name="close" :size="15" />
          </button>
        </div>

        <div class="picker-toolbar">
          <div class="field picker-search">
            <Icon name="search" :size="15" class="faint" />
            <input
              v-model="pickerFilter"
              type="text"
              spellcheck="false"
              placeholder="Найти трек"
            />
          </div>
          <button class="more-link" type="button" @click="selectAllVisible">
            Выбрать все
          </button>
          <button
            class="more-link"
            type="button"
            :disabled="!selected.length"
            @click="clearSelection"
          >
            Снять
          </button>
        </div>

        <div class="picker-list">
          <button
            v-for="row in pickerRows"
            :key="`${row.track.id}-${row.index}`"
            class="picker-row"
            :class="{ on: selected.includes(row.index) }"
            type="button"
            @click="toggleSelect(row.index, $event)"
          >
            <span
              class="track-check-box"
              :class="{ on: selected.includes(row.index) }"
            >
              <Icon
                v-if="selected.includes(row.index)"
                name="check"
                :size="12"
              />
            </span>

            <div class="cover picker-cover">
              <img
                v-if="row.track.cover_url"
                loading="lazy"
                decoding="async"
                :src="row.track.cover_url"
              />
              <Icon v-else name="note" :size="14" class="faint" />
            </div>

            <span class="picker-meta">
              <span class="ellipsis t-13">{{ row.track.title }}</span>
              <span class="faint t-11 ellipsis">
                {{ artistNames(row.track.artists) }}
              </span>
            </span>

            <span class="faint t-11">
              {{ formatDuration(row.track.duration_ms) }}
            </span>
          </button>

          <div v-if="!pickerRows.length" class="picker-empty faint t-12">
            Ничего не нашлось
          </div>
        </div>

        <div class="picker-actions">
          <span class="faint t-12">Выбрано: {{ selected.length }}</span>

          <div class="col" />

          <button
            class="btn"
            type="button"
            :disabled="!selected.length"
            @click="playSelected"
          >
            <Icon name="play" :size="15" />
            <span>Слушать</span>
          </button>

          <button
            class="btn"
            type="button"
            :disabled="!selected.length"
            @click="enqueueSelected"
          >
            <Icon name="addQueue" :size="15" />
            <span>В очередь</span>
          </button>

          <button class="btn" type="button" :disabled="!selected.length">
            <Icon name="playlistAdd" :size="15" />
            <span>В плейлист</span>
            <q-menu class="menu" anchor="top right" self="bottom right">
              <div class="menu-body" style="min-width: 220px">
                <div class="menu-head">Добавить в плейлист</div>
                <div
                  v-for="item in otherPlaylists"
                  :key="item.kind"
                  class="menu-item"
                  v-close-popup
                  @click="addSelectedTo(item.kind)"
                >
                  <Icon name="queue" :size="16" />
                  <span class="ellipsis">{{ item.title }}</span>
                </div>
                <div v-if="!otherPlaylists.length" class="menu-item disabled">
                  <span class="faint">Нет других плейлистов</span>
                </div>
              </div>
            </q-menu>
          </button>

          <button
            class="btn"
            type="button"
            :disabled="!selected.length || downloading"
            @click="downloadSelected"
          >
            <Icon name="download" :size="15" />
            <span>Скачать</span>
          </button>

          <button
            class="btn danger-btn"
            type="button"
            :disabled="!selected.length"
            @click="removeSelected"
          >
            <Icon name="trash" :size="15" />
            <span>Удалить</span>
          </button>
        </div>
      </div>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import {
  computed,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
  nextTick,
} from "vue";
import { Notify } from "quasar";
import { useRoute, useRouter } from "vue-router";
import Icon from "@/components/Icon.vue";
import TrackRow from "@/components/TrackRow.vue";
import { api } from "@/api/client";
import type { Playlist, Track } from "@/api/types";
import { artistNames, formatDuration, plural } from "@/lib/format";
import { readCache, swr, writeCache } from "@/lib/cache";
import { askConfirm } from "@/lib/dialogs";
import { useLibraryStore } from "@/stores/library";
import { usePlayerStore } from "@/stores/player/index";
import { useUiStore } from "@/stores/ui/index";

type SortMode = "custom" | "title" | "artist" | "album" | "duration";

const props = defineProps<{ kind: string }>();
const router = useRouter();
const route = useRoute();
const player = usePlayerStore();
const library = useLibraryStore();
const ui = useUiStore();
const tracks = ref<Track[]>([]);
const title = ref("Плейлист");
const cover = ref<string | null>(null);
const loading = ref(false);
const filter = ref("");
const dragFrom = ref<number | null>(null);
const dragOver = ref<number | null>(null);
const editing = ref(false);
const draftTitle = ref("");
const titleInput = ref<HTMLInputElement | null>(null);
const sortMode = ref<SortMode>("custom");
const sortDir = ref<"asc" | "desc">("asc");
const pickerOpen = ref(false);
const pickerFilter = ref("");
const selected = ref<number[]>([]);
const lastPicked = ref<number | null>(null);
const downloading = ref(false);
const downloadDone = ref(0);
const downloadTotal = ref(0);
const cancelRequested = ref(false);
const localIds = ref<Set<string>>(new Set());
const selectedIds = ref<string[]>([]);
const headerCollapsed = ref(false);
const virtualRef = ref<{
  scrollTo: (index: number, edge?: string) => void;
} | null>(null);

const sortOptions: Array<{ id: SortMode; label: string }> = [
  { id: "custom", label: "Как в плейлисте" },
  { id: "title", label: "По названию" },
  { id: "artist", label: "По исполнителю" },
  { id: "album", label: "По альбому" },
  { id: "duration", label: "По длительности" },
];

const kindNumber = computed(() => Number(props.kind));
const isPinned = computed(() => library.isPinned(kindNumber.value));
const canReorder = computed(
  () => sortMode.value === "custom" && sortDir.value === "asc" && !filter.value,
);

const sortLabel = computed(
  () => sortOptions.find((option) => option.id === sortMode.value)?.label ?? "",
);

const otherPlaylists = computed(() =>
  library.sortedPlaylists.filter(
    (playlist) => String(playlist.kind) !== String(props.kind),
  ),
);

const downloadLabel = computed(() => {
  if (downloading.value)
    return cancelRequested.value
      ? "Отменяю…"
      : `${downloadDone.value} / ${downloadTotal.value}`;
  if (fullyDownloaded.value) return "Уже скачано";
  const left = pendingTracks.value.length;
  if (left && left !== tracks.value.length) return `Дозакачать (${left})`;
  return "Скачать всё";
});

const pendingTracks = computed(() =>
  tracks.value.filter((track) => !localIds.value.has(track.id)),
);

const fullyDownloaded = computed(
  () => tracks.value.length > 0 && pendingTracks.value.length === 0,
);

const folderName = computed(() => safeFolder(title.value));

const rows = computed(() => {
  const all = tracks.value.map((track, index) => ({ track, index }));
  const query = filter.value.trim().toLowerCase();
  const found = query
    ? all.filter(({ track }) => {
        const artists = track.artists.map((artist) => artist.name).join(" ");
        const haystack = `${track.title} ${artists} ${track.album_title || ""}`;
        return haystack.toLowerCase().includes(query);
      })
    : all;

  let result: Array<{ track: Track; index: number }>;
  if (sortMode.value === "custom") {
    result = found;
  } else {
    result = [...found];
    result.sort((a, b) => {
      if (sortMode.value === "duration")
        return (a.track.duration_ms || 0) - (b.track.duration_ms || 0);
      const left = sortKey(a.track);
      const right = sortKey(b.track);
      return left.localeCompare(right, "ru");
    });
  }

  if (sortDir.value === "desc") result = [...result].reverse();
  return result;
});

function sortKey(track: Track): string {
  if (sortMode.value === "artist") return artistNames(track.artists) || "";
  if (sortMode.value === "album") return track.album_title || "";
  return track.title || "";
}

const selectedTracks = computed(() =>
  selected.value
    .slice()
    .sort((a, b) => a - b)
    .map((index) => tracks.value[index])
    .filter((track): track is Track => Boolean(track)),
);

const totalLabel = computed(() => {
  const ms = tracks.value.reduce(
    (sum, track) => sum + (track.duration_ms || 0),
    0,
  );
  if (!ms) return "";
  const minutes = Math.round(ms / 60000);
  if (minutes < 60) return `${minutes} мин`;
  return `${Math.floor(minutes / 60)} ч ${minutes % 60} мин`;
});

function playFrom(index: number) {
  if (tracks.value.length) void player.playQueue(tracks.value, index);
}

function shufflePlay() {
  if (!tracks.value.length) return;
  void player.playQueue(tracks.value, 0).then(() => {
    if (!player.shuffle) player.toggleShuffle();
  });
}

function enqueueAll() {
  if (!tracks.value.length) return;
  tracks.value.forEach((track) => player.enqueue(track));
  Notify.create({
    message: `В очередь добавлено: ${tracks.value.length}`,
  });
}

function openPicker() {
  pickerFilter.value = "";
  pickerOpen.value = true;
}

function closePicker() {
  pickerOpen.value = false;
  clearSelection();
}

const selectedRowTracks = computed(() =>
  tracks.value.filter((track) => selectedIds.value.includes(track.id)),
);

const allVisibleSelected = computed(
  () =>
    rows.value.length > 0 &&
    rows.value.every((row) => selectedIds.value.includes(row.track.id)),
);

function isRowSelected(track: Track): boolean {
  return selectedIds.value.includes(track.id);
}

function clearRowSelection() {
  selectedIds.value = [];
}

function toggleRow(track: Track, event?: MouseEvent) {
  const visible = rows.value.map((row) => row.track.id);
  const lastId = lastPicked.value !== null ? String(lastPicked.value) : null;

  if (event?.shiftKey && lastId) {
    const from = visible.indexOf(lastId);
    const to = visible.indexOf(track.id);
    if (from >= 0 && to >= 0) {
      const range = visible.slice(Math.min(from, to), Math.max(from, to) + 1);
      selectedIds.value = [...new Set([...selectedIds.value, ...range])];
      return;
    }
  }

  selectedIds.value = selectedIds.value.includes(track.id)
    ? selectedIds.value.filter((id) => id !== track.id)
    : [...selectedIds.value, track.id];
  lastPicked.value = track.id as unknown as number;
}

function toggleSelectAllRows() {
  selectedIds.value = allVisibleSelected.value
    ? []
    : rows.value.map((row) => row.track.id);
}

function playRowSelection() {
  const list = selectedRowTracks.value;
  if (list.length) void player.playQueue(list, 0);
}

function enqueueRowSelection() {
  const list = selectedRowTracks.value;
  if (!list.length) return;
  list.forEach((track) => player.enqueue(track));
  Notify.create({ message: `В очередь добавлено: ${list.length}` });
}

async function addRowSelectionTo(kind: number | string) {
  const list = selectedRowTracks.value;
  if (!list.length) return;
  await library.addTracksToPlaylist(Number(kind), list);
}

async function downloadRowSelection() {
  await downloadList(selectedRowTracks.value);
}

async function removeRowSelection() {
  const list = selectedRowTracks.value;
  if (!list.length) return;
  const ok = await askConfirm({
    title: "Удалить выбранные треки?",
    message: `Из «${title.value}» будет убрано ${list.length}.`,
    okLabel: "Удалить",
    danger: true,
  });
  if (!ok) return;

  const ids = new Set(list.map((track) => track.id));
  let removed = 0;
  for (let index = tracks.value.length - 1; index >= 0; index -= 1) {
    const track = tracks.value[index];
    if (!track || !ids.has(track.id)) continue;
    if (await library.removeFromPlaylist(kindNumber.value, track, index)) {
      tracks.value.splice(index, 1);
      removed += 1;
    }
  }
  selectedIds.value = [];
  writeCache(`playlist.${props.kind}`, tracks.value);
  Notify.create({ message: `Удалено треков: ${removed}` });
}

function clearSelection() {
  selected.value = [];
  lastPicked.value = null;
}

const pickerRows = computed(() => {
  const query = pickerFilter.value.trim().toLowerCase();
  const all = tracks.value.map((track, index) => ({ track, index }));
  if (!query) return all;
  return all.filter(({ track }) => {
    const artists = artistNames(track.artists);
    return `${track.title} ${artists}`.toLowerCase().includes(query);
  });
});

function selectAllVisible() {
  selected.value = pickerRows.value.map((row) => row.index);
}

function toggleSelect(index: number, event?: MouseEvent) {
  const visible = pickerRows.value.map((row) => row.index);
  if (event?.shiftKey && lastPicked.value !== null) {
    const from = visible.indexOf(lastPicked.value);
    const to = visible.indexOf(index);
    if (from >= 0 && to >= 0) {
      const range = visible.slice(Math.min(from, to), Math.max(from, to) + 1);
      const merged = new Set([...selected.value, ...range]);
      selected.value = [...merged];
      lastPicked.value = index;
      return;
    }
  }
  selected.value = selected.value.includes(index)
    ? selected.value.filter((value) => value !== index)
    : [...selected.value, index];
  lastPicked.value = index;
}

function playSelected() {
  const list = selectedTracks.value;
  if (!list.length) return;
  pickerOpen.value = false;
  void player.playQueue(list, 0);
}

function enqueueSelected() {
  const list = selectedTracks.value;
  if (!list.length) return;
  pickerOpen.value = false;
  list.forEach((track) => player.enqueue(track));
  Notify.create({ message: `В очередь добавлено: ${list.length}` });
}

async function addSelectedTo(kind: number | string) {
  const list = selectedTracks.value;
  if (!list.length) return;
  pickerOpen.value = false;
  let added = 0;
  for (const track of list) {
    if (await library.addToPlaylist(Number(kind), track)) added += 1;
  }
  Notify.create({ message: `Добавлено треков: ${added}` });
}

async function removeSelected() {
  const list = selected.value.slice().sort((a, b) => b - a);
  if (!list.length) return;
  const ok = await askConfirm({
    title: "Удалить выбранные треки?",
    message: `Из «${title.value}» будет убрано ${list.length}.`,
    okLabel: "Удалить",
    danger: true,
  });
  if (!ok) return;

  pickerOpen.value = false;
  let removed = 0;
  for (const index of list) {
    const track = tracks.value[index];
    if (!track) continue;
    if (await library.removeFromPlaylist(kindNumber.value, track, index)) {
      tracks.value.splice(index, 1);
      removed += 1;
    }
  }
  clearSelection();
  writeCache(`playlist.${props.kind}`, tracks.value);
  Notify.create({ message: `Удалено треков: ${removed}` });
}

function safeFolder(name: string): string {
  const cleaned = name
    .replace(/[\\/:*?"<>|]/g, "_")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/\.+$/, "");
  return cleaned || "Playlist";
}

function joinPath(base: string, folder: string): string {
  const sep = base.includes("\\") ? "\\" : "/";
  const trimmed = base.replace(/[\\/]+$/, "");
  return `${trimmed}${sep}${folder}`;
}

async function playlistDir(): Promise<string> {
  const base =
    ui.settings.downloadDir.trim() ||
    (await api.defaultDownloadDir().catch(() => ""));
  if (!base) return "";
  return joinPath(base, folderName.value);
}

async function refreshLocal() {
  const dir = await playlistDir();
  if (!dir) {
    localIds.value = new Set();
    return;
  }
  const found = new Set<string>();
  const list = tracks.value.slice();
  for (const track of list) {
    try {
      const path = await api.findLocalTrack(track.id, dir);
      if (path) found.add(track.id);
    } catch {}
  }
  localIds.value = found;
}

function cancelDownload() {
  if (!downloading.value) return;
  cancelRequested.value = true;
  Notify.create({ message: "Отменяю загрузку плейлиста…" });
}

async function downloadList(list: Track[], skipExisting = true) {
  if (!list.length || downloading.value) return;
  const dir = await playlistDir();
  const queue = skipExisting
    ? list.filter((track) => !localIds.value.has(track.id))
    : list;

  if (!queue.length) {
    Notify.create({ message: "Всё это уже скачано" });
    return;
  }

  downloading.value = true;
  cancelRequested.value = false;
  downloadTotal.value = queue.length;
  downloadDone.value = 0;
  let failed = 0;
  let done = 0;

  for (const track of queue) {
    if (cancelRequested.value) break;
    try {
      await api.download(
        track.id,
        `${artistNames(track.artists)} - ${track.title}`,
        "lossless",
        dir || null,
      );
      localIds.value = new Set([...localIds.value, track.id]);
      done += 1;
    } catch {
      failed += 1;
    }
    downloadDone.value += 1;
  }

  const wasCancelled = cancelRequested.value;
  downloading.value = false;
  cancelRequested.value = false;

  if (wasCancelled) {
    Notify.create({
      type: "warning",
      message: `Загрузка отменена. Успел скачать: ${done}`,
    });
    return;
  }

  Notify.create({
    type: failed ? "warning" : "positive",
    message: failed
      ? `Скачано ${done} из ${queue.length}`
      : `Скачано треков: ${done}${dir ? ` → ${folderName.value}` : ""}`,
  });
}

async function downloadAll() {
  if (downloading.value) return;
  if (fullyDownloaded.value) {
    Notify.create({ message: "Плейлист уже скачан целиком" });
    return;
  }
  const left = pendingTracks.value.length;
  const already = tracks.value.length - left;
  const ok = await askConfirm({
    title: "Скачать плейлист?",
    message: already
      ? `Будет скачано ${left} треков в папку «${folderName.value}». Уже на диске: ${already}.`
      : `Будет скачано ${left} треков в папку «${folderName.value}».`,
    okLabel: "Скачать",
  });
  if (ok) await downloadList(tracks.value);
}

async function downloadSelected() {
  const list = selectedTracks.value;
  pickerOpen.value = false;
  await downloadList(list);
}

async function playRecommended() {
  try {
    const list = await api.playlistRecommendations(props.kind);
    if (!list.length) {
      Notify.create({ message: "Яндекс не предложил похожих треков" });
      return;
    }
    await player.playQueue(list, 0);
    Notify.create({ message: `Похожих треков: ${list.length}` });
  } catch {
    Notify.create({ message: "Не удалось получить рекомендации" });
  }
}

function syncMeta(nextTitle?: string) {
  const cached = readCache<Playlist[]>("home.playlists") ?? [];
  const updated = cached.map((playlist) =>
    String(playlist.kind) === String(props.kind) && nextTitle
      ? { ...playlist, title: nextTitle }
      : playlist,
  );
  writeCache("home.playlists", updated);
  const inStore = library.playlists.find(
    (playlist) => String(playlist.kind) === String(props.kind),
  );
  if (inStore && nextTitle) inStore.title = nextTitle;
}

function beginEdit() {
  draftTitle.value = title.value;
  editing.value = true;
  void nextTick(() => {
    const input = titleInput.value;
    if (!input) return;
    input.focus();
    input.select();
  });
}

function cancelTitle() {
  editing.value = false;
  draftTitle.value = title.value;
}

async function commitTitle() {
  if (!editing.value) return;
  editing.value = false;
  const next = draftTitle.value.trim();
  if (!next || next === title.value) return;
  if (await library.renamePlaylist(kindNumber.value, next)) {
    title.value = next;
    syncMeta(next);
  }
}

function setPublic(isPublic: boolean) {
  void library.setPlaylistPublic(kindNumber.value, isPublic);
}

async function clearTracks() {
  const ok = await askConfirm({
    title: "Очистить плейлист?",
    message: `Из «${title.value}» будут убраны все треки.`,
    okLabel: "Очистить",
    danger: true,
  });
  if (!ok) return;
  if (await library.clearPlaylist(kindNumber.value)) {
    tracks.value = [];
    clearSelection();
    writeCache(`playlist.${props.kind}`, []);
  }
}

async function removePlaylist() {
  const ok = await askConfirm({
    title: "Удалить плейлист?",
    message: `«${title.value}» будет удалён безвозвратно.`,
    okLabel: "Удалить",
    danger: true,
  });
  if (!ok) return;
  if (await library.deletePlaylist(kindNumber.value)) {
    void router.push("/playlists");
  }
}

async function removeAt(index: number) {
  const track = tracks.value[index];
  if (!track) return;
  const ok = await library.removeFromPlaylist(kindNumber.value, track, index);
  if (ok) {
    tracks.value.splice(index, 1);
    clearSelection();
  }
}

function startReorder(index: number, event: PointerEvent) {
  if (event.button !== 0) return;
  if (!canReorder.value) {
    Notify.create({
      message: "Сбрось поиск и сортировку, чтобы менять порядок",
    });
    return;
  }
  dragFrom.value = index;
  dragOver.value = index;
  document.body.classList.add("track-reordering");
  window.addEventListener("pointermove", trackPointer, { passive: false });
  window.addEventListener("pointerup", finishReorder, { once: true });
  window.addEventListener("pointercancel", cancelReorder, { once: true });
}

function trackPointer(event: PointerEvent) {
  if (dragFrom.value === null) return;
  event.preventDefault();
  const element = document.elementFromPoint(event.clientX, event.clientY);
  const row = element?.closest<HTMLElement>(".row-track[data-track-index]");
  const index = Number(row?.dataset.trackIndex);
  if (Number.isInteger(index) && index >= 0 && index < tracks.value.length) {
    dragOver.value = index;
  }
  const area = document.querySelector<HTMLElement>(
    ".playlist-tracks-scroll .q-scrollarea__container",
  );
  if (!area) return;
  const bounds = area.getBoundingClientRect();
  if (event.clientY < bounds.top + 48) area.scrollTop -= 18;
  if (event.clientY > bounds.bottom - 48) area.scrollTop += 18;
}

async function finishReorder() {
  const from = dragFrom.value;
  const to = dragOver.value;
  clearReorder();
  if (from === null || to === null || from === to) return;
  const track = tracks.value[from];
  if (!track) return;
  const next = [...tracks.value];
  next.splice(from, 1);
  next.splice(to, 0, track);
  tracks.value = next;
  const ok = await library.moveInPlaylist(kindNumber.value, track, from, to);
  if (!ok) await load();
}

function cancelReorder() {
  clearReorder();
}

function clearReorder() {
  dragFrom.value = null;
  dragOver.value = null;
  document.body.classList.remove("track-reordering");
  window.removeEventListener("pointermove", trackPointer);
  window.removeEventListener("pointerup", finishReorder);
  window.removeEventListener("pointercancel", cancelReorder);
}

function onTracksScroll(event: Event) {
  const target = event.target as HTMLElement | null;
  if (!target) return;
  headerCollapsed.value = target.scrollTop > 24;
}

function jumpToTrack(id: string) {
  const at = rows.value.findIndex((row) => row.track.id === id);
  if (at < 0) return;
  void nextTick(() => {
    virtualRef.value?.scrollTo(at, "start");
    headerCollapsed.value = true;
  });
}

function applyMeta(list: Track[]) {
  const all = readCache<Playlist[]>("home.playlists") ?? [];
  const meta = all.find(
    (playlist) => String(playlist.kind) === String(props.kind),
  );
  title.value = meta?.title || title.value || "Плейлист";
  cover.value = meta?.cover_url || list[0]?.cover_url || null;
}

async function load() {
  const key = `playlist.${props.kind}`;
  const cached = readCache<Track[]>(key) ?? [];
  tracks.value = cached;
  filter.value = "";
  headerCollapsed.value = false;
  editing.value = false;
  closePicker();
  applyMeta(cached);
  loading.value = !cached.length;

  void swr<Playlist[]>("home.playlists", () => api.playlists(), {
    onData: () => applyMeta(tracks.value),
  });

  await swr<Track[]>(key, () => api.playlistTracks(props.kind), {
    onData: (list) => {
      tracks.value = list;
      applyMeta(list);
    },
    onSettled: () => {
      loading.value = false;
      void refreshLocal();
      const target = route.query.track;
      if (target) void nextTick(() => jumpToTrack(String(target)));
    },
  });
}

watch(() => props.kind, load);
onMounted(load);
onBeforeUnmount(clearReorder);
</script>
