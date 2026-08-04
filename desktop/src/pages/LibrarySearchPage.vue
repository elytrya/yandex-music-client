<template>
  <div class="libsearch-page">
    <div class="page-head libsearch-head">
      <div>
        <div class="h1">Поиск по библиотеке</div>
        <p class="faint t-13 libsearch-sub">
          {{ index.length }} уникальных треков из
          {{ sources.length }} источников
          <span v-if="loading"> - загружаю {{ done }}/{{ total }}…</span>
        </p>
      </div>
      <button class="btn" :disabled="loading" @click="reload(true)">
        Обновить
      </button>
    </div>

    <div class="libsearch-controls">
      <input
        v-model="query"
        class="field libsearch-input"
        placeholder="Название, исполнитель или альбом…"
      />

      <div class="libsearch-filters">
        <select v-model="source" class="field libsearch-select">
          <option value="">Все источники</option>
          <option v-for="s in sources" :key="s" :value="s">{{ s }}</option>
        </select>

        <select v-model="artist" class="field libsearch-select">
          <option value="">Все исполнители</option>
          <option v-for="a in artists" :key="a" :value="a">{{ a }}</option>
        </select>

        <select v-model="sort" class="field libsearch-select">
          <option value="title">По названию</option>
          <option value="artist">По исполнителю</option>
          <option value="shortest">Сначала короткие</option>
          <option value="longest">Сначала длинные</option>
        </select>

        <label class="libsearch-range">
          <span class="faint t-12">
            Длительность: {{ minMinutes }}-{{ maxMinutes }} мин
          </span>
          <q-range
            v-model="duration"
            :min="0"
            :max="20"
            :step="1"
            dense
            color="primary"
          />
        </label>

        <button
          class="btn"
          :class="{ on: onlyLiked }"
          @click="onlyLiked = !onlyLiked"
        >
          Только любимые
        </button>
        <button class="btn" @click="resetFilters">Сбросить</button>
      </div>
    </div>

    <div class="libsearch-actions">
      <span class="faint t-12">Найдено треков: {{ results.length }}</span>
      <button class="btn-solid" :disabled="!results.length" @click="playAll(0)">
        Слушать найденное
      </button>
    </div>

    <div class="libsearch-list">
      <div v-if="loading && !index.length" class="row justify-center q-py-xl">
        <q-spinner size="24px" color="grey-6" />
      </div>

      <div v-else-if="!results.length" class="faint t-13 q-pa-md">
        Ничего не нашлось. Попробуй другой запрос или сбрось фильтры.
      </div>

      <LazyTracks v-else :items="resultTracks" :initial="50" :step="50">
        <template #default="{ item, index: i }">
          <div
            class="libsearch-row"
            :class="{ on: player.current?.id === item.id }"
            @dblclick="playAll(i)"
          >
            <div class="cover libsearch-cover" @click="playAll(i)">
              <img v-if="item.cover_url" :src="item.cover_url" loading="lazy" />
              <Icon v-else name="note" :size="15" class="faint" />
            </div>
            <div class="col" style="min-width: 0" @click="playAll(i)">
              <div class="ellipsis t-13">{{ item.title }}</div>
              <div class="faint t-11 ellipsis">
                <ArtistsLine :artists="item.artists" :limit="2" />
              </div>
            </div>
            <span
              v-if="sourceMap.get(item.id)?.length"
              class="faint t-11 libsearch-source ellipsis link"
              :title="sourceTitle(item.id)"
              @click.stop
            >
              {{ sourceOf(item.id) }}
              <q-menu class="menu" anchor="bottom middle" self="top middle">
                <div class="menu-body" style="min-width: 220px">
                  <div
                    v-for="src in sourceMap.get(item.id)"
                    :key="src"
                    class="menu-item"
                    v-close-popup
                    @click="openSource(src, item.id)"
                  >
                    <Icon name="queue" :size="16" />
                    <span class="ellipsis">Открыть: {{ src }}</span>
                  </div>
                </div>
              </q-menu>
            </span>
            <span v-else class="faint t-11 libsearch-source ellipsis" />
            <span class="faint t-12">
              {{ formatDuration(item.duration_ms || 0) }}
            </span>
            <div class="icon-btn">
              <Icon name="more" :size="17" />
              <TrackMenu :track="item" />
            </div>
          </div>
        </template>
      </LazyTracks>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { Notify } from "quasar";
import { useRouter } from "vue-router";
import Icon from "@/components/Icon.vue";
import ArtistsLine from "@/components/ArtistsLine.vue";
import LazyTracks from "@/components/LazyTracks.vue";
import TrackMenu from "@/components/TrackMenu.vue";
import { api } from "@/api/client";
import type { Track } from "@/api/types";
import { readCache, writeCache } from "@/lib/cache";
import { artistNames, formatDuration } from "@/lib/format";
import { useLibraryStore } from "@/stores/library";
import { usePlayerStore } from "@/stores/player/index";

interface Entry {
  track: Track;
  sources: string[];
  search: string;
}

const library = useLibraryStore();
const player = usePlayerStore();
const router = useRouter();

const index = ref<Entry[]>([]);
const loading = ref(false);
const done = ref(0);
const total = ref(0);

const query = ref("");
const source = ref("");
const artist = ref("");
const sort = ref("title");
const onlyLiked = ref(false);
const duration = ref({ min: 0, max: 20 });

const minMinutes = computed(() => duration.value.min);
const maxMinutes = computed(() =>
  duration.value.max >= 20 ? "∞" : duration.value.max,
);

const sources = computed(() =>
  [...new Set(index.value.flatMap((entry) => entry.sources))].sort((a, b) =>
    a.localeCompare(b, "ru"),
  ),
);

const artists = computed(() =>
  [
    ...new Set(
      index.value.flatMap((entry) =>
        entry.track.artists.map((a) => a.name).filter(Boolean),
      ),
    ),
  ]
    .sort((a, b) => a.localeCompare(b, "ru"))
    .slice(0, 400),
);

const results = computed(() => {
  const text = query.value.trim().toLowerCase();
  const words = text.split(/\s+/).filter(Boolean);
  const min = duration.value.min * 60000;
  const max = duration.value.max >= 20 ? Infinity : duration.value.max * 60000;

  const rows = index.value.filter((entry) => {
    if (source.value && !entry.sources.includes(source.value)) return false;
    if (
      artist.value &&
      !entry.track.artists.some((a) => a.name === artist.value)
    )
      return false;
    if (onlyLiked.value && !library.liked(entry.track.id)) return false;
    const ms = entry.track.duration_ms ?? 0;
    if (ms && (ms < min || ms > max)) return false;
    return words.every((w) => entry.search.includes(w));
  });

  const sorted = [...rows];
  if (sort.value === "title")
    sorted.sort((a, b) => a.track.title.localeCompare(b.track.title, "ru"));
  else if (sort.value === "artist")
    sorted.sort((a, b) =>
      artistNames(a.track.artists).localeCompare(
        artistNames(b.track.artists),
        "ru",
      ),
    );
  else if (sort.value === "shortest")
    sorted.sort(
      (a, b) => (a.track.duration_ms ?? 0) - (b.track.duration_ms ?? 0),
    );
  else
    sorted.sort(
      (a, b) => (b.track.duration_ms ?? 0) - (a.track.duration_ms ?? 0),
    );

  return sorted;
});

const resultTracks = computed(() => results.value.map((entry) => entry.track));

const sourceMap = computed(() => {
  const map = new Map<string, string[]>();
  for (const entry of results.value) map.set(entry.track.id, entry.sources);
  return map;
});

function sourceOf(id: string): string {
  const list = sourceMap.value.get(id);
  if (!list?.length) return "";
  if (list.length === 1) return list[0]!;
  return `${list[0]!} +${list.length - 1}`;
}

function sourceTitle(id: string): string {
  return sourceMap.value.get(id)?.join(", ") ?? "";
}

const sourceKinds = computed(() => {
  const map = new Map<string, number>();
  for (const pl of library.playlists) map.set(pl.title, pl.kind);
  return map;
});

function openSource(src: string, trackId: string) {
  if (src === "Мне нравится") {
    void router.push("/liked");
    return;
  }
  const kind = sourceKinds.value.get(src);
  if (kind == null) return;
  void router.push({ path: `/playlists/${kind}`, query: { track: trackId } });
}

function resetFilters() {
  query.value = "";
  source.value = "";
  artist.value = "";
  onlyLiked.value = false;
  duration.value = { min: 0, max: 20 };
}

function playAll(startIndex: number) {
  if (!resultTracks.value.length) return;
  void player.playQueue(resultTracks.value, startIndex);
}

function toEntry(track: Track, source: string): Entry {
  return {
    track,
    sources: [source],
    search: `${track.title} ${artistNames(track.artists)} ${
      track.album_title ?? ""
    }`.toLowerCase(),
  };
}

function mergeEntries(target: Map<string, Entry>, rows: Entry[]): void {
  for (const row of rows) {
    const found = target.get(row.track.id);
    if (!found) {
      target.set(row.track.id, row);
      continue;
    }
    for (const name of row.sources) {
      if (!found.sources.includes(name)) found.sources.push(name);
    }
  }
}

async function loadSource(
  kind: number | "liked",
  title: string,
  force: boolean,
): Promise<Entry[]> {
  const key = kind === "liked" ? "liked.tracks" : `playlist.${kind}`;
  if (!force) {
    const cached = readCache<Track[]>(key);
    if (cached && Array.isArray(cached) && cached.length)
      return cached.map((track) => toEntry(track, title));
  }
  try {
    const tracks =
      kind === "liked"
        ? await api.likedTracks()
        : await api.playlistTracks(kind);
    writeCache(key, tracks);
    return tracks.map((track) => toEntry(track, title));
  } catch {
    return [];
  }
}

async function reload(force = false) {
  if (loading.value) return;
  loading.value = true;
  done.value = 0;

  if (!library.playlists.length) await library.init();

  const jobs: Array<{ kind: number | "liked"; title: string }> = [
    { kind: "liked", title: "Мне нравится" },
    ...library.playlists.map((pl) => ({ kind: pl.kind, title: pl.title })),
  ];
  total.value = jobs.length;

  const collected = new Map<string, Entry>();
  const queue = [...jobs];

  async function worker() {
    for (;;) {
      const job = queue.shift();
      if (!job) return;
      const rows = await loadSource(job.kind, job.title, force);
      mergeEntries(collected, rows);
      done.value += 1;
      index.value = [...collected.values()];
    }
  }

  await Promise.all([worker(), worker(), worker()]);

  loading.value = false;
  if (force) Notify.create({ message: "Библиотека обновлена" });
}

onMounted(() => {
  void reload(false);
});
</script>
