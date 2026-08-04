<template>
  <q-page class="scroll-page">
    <q-scroll-area class="scroll-page-area">
      <div class="scroll-page-inner">
        <div class="h1">Поиск</div>

        <div class="suggest-wrap q-mt-lg">
          <div class="field">
            <Icon name="search" :size="16" class="faint" />
            <input
              v-model="text"
              type="text"
              spellcheck="false"
              placeholder="Треки, артисты, альбомы"
              @keyup.enter="submit"
              @keydown.esc="closeSuggest"
              @focus="suggestOpen = true"
              @blur="closeSuggestSoon"
            />
            <button
              v-if="text"
              class="icon-btn xs"
              type="button"
              title="Очистить"
              @click="clearQuery"
            >
              <Icon name="close" :size="13" />
            </button>
          </div>

          <div v-if="suggestOpen && suggestions.length" class="suggest-list">
            <button
              v-for="hint in suggestions"
              :key="hint"
              class="suggest-item"
              type="button"
              @mousedown.prevent="pick(hint)"
            >
              <Icon name="search" :size="14" class="faint" />
              <span class="ellipsis">{{ hint }}</span>
            </button>
          </div>
        </div>

        <div v-if="loading" class="q-mt-xl">
          <q-spinner size="24px" color="primary" />
        </div>

        <template v-else-if="result">
          <template v-if="result.artists.length">
            <div class="h2 q-mt-xl q-mb-md">Артисты</div>
            <div class="row q-col-gutter-md">
              <div v-for="a in result.artists" :key="a.id" class="col-auto">
                <div
                  class="card text-center"
                  style="width: 132px"
                  @click="router.push(`/artist/${a.id}`)"
                >
                  <div class="cover round" style="width: 100%; aspect-ratio: 1">
                    <img
                      v-if="avatarOf(a)"
                      loading="lazy"
                      decoding="async"
                      :src="avatarOf(a) ?? undefined"
                    />
                    <Icon v-else name="person" :size="24" class="faint" />
                  </div>
                  <div class="t-13 w-500 ellipsis q-mt-sm">{{ a.name }}</div>
                </div>
              </div>
            </div>
          </template>

          <template v-if="result.albums.length">
            <div class="h2 q-mt-xl q-mb-md">Альбомы</div>
            <div class="row q-col-gutter-md">
              <div v-for="al in result.albums" :key="al.id" class="col-auto">
                <div
                  class="card"
                  style="width: 150px"
                  @click="router.push(`/album/${al.id}`)"
                >
                  <div class="cover" style="width: 100%; aspect-ratio: 1">
                    <img
                      loading="lazy"
                      decoding="async"
                      v-if="al.cover_url"
                      :src="al.cover_url"
                    />
                    <Icon v-else name="album" :size="24" class="faint" />
                  </div>
                  <div class="t-13 w-500 ellipsis q-mt-sm">{{ al.title }}</div>
                  <div class="faint t-11">{{ al.year || "" }}</div>
                </div>
              </div>
            </div>
          </template>

          <template v-if="result.tracks.length">
            <div class="h2 q-mt-xl q-mb-sm">Треки</div>
            <LazyTracks v-slot="{ item, index }" :items="result.tracks">
              <TrackRow :track="item" :index="index" @play="playFrom(index)" />
            </LazyTracks>
          </template>

          <div
            v-if="
              !result.tracks.length &&
              !result.artists.length &&
              !result.albums.length
            "
            class="dim t-13 q-mt-xl"
          >
            Ничего не нашлось
          </div>
        </template>
      </div>
    </q-scroll-area>
  </q-page>
</template>

<script setup lang="ts">
import Icon from "@/components/Icon.vue";
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import LazyTracks from "@/components/LazyTracks.vue";
import TrackRow from "@/components/TrackRow.vue";
import { api } from "@/api/client";
import type { Artist, SearchResult } from "@/api/types";
import { cachedAvatar, fetchAvatars } from "@/lib/artistAvatar";
import { usePlayerStore } from "@/stores/player/index";

const route = useRoute();
const router = useRouter();
const player = usePlayerStore();

const text = ref("");
const result = ref<SearchResult | null>(null);
const loading = ref(false);
const avatars = ref<Record<string, string | null>>({});

function avatarOf(artist: Artist): string | null {
  return artist.cover_url || avatars.value[artist.id] || null;
}

async function loadAvatars(artists: Artist[]) {
  const missing = artists.filter((a) => !a.cover_url);
  if (!missing.length) return;
  const next: Record<string, string | null> = { ...avatars.value };
  for (const artist of missing) {
    const cached = cachedAvatar(artist.id);
    if (cached) next[artist.id] = cached;
  }
  avatars.value = next;
  const loaded = await fetchAvatars(missing.map((a) => a.id));
  avatars.value = { ...avatars.value, ...loaded };
}

function playFrom(i: number) {
  const tracks = result.value?.tracks || [];
  if (tracks.length) void player.playQueue(tracks, i);
}

const suggestions = ref<string[]>([]);
const suggestOpen = ref(false);
let suggestTimer: ReturnType<typeof setTimeout> | undefined;
let blurTimer: ReturnType<typeof setTimeout> | undefined;

function closeSuggest() {
  suggestOpen.value = false;
}

function closeSuggestSoon() {
  blurTimer = setTimeout(closeSuggest, 120);
}

function clearQuery() {
  text.value = "";
  suggestions.value = [];
  result.value = null;
}

function pick(hint: string) {
  clearTimeout(blurTimer);
  text.value = hint;
  suggestions.value = [];
  suggestOpen.value = false;
  void run();
}

function submit() {
  suggestions.value = [];
  suggestOpen.value = false;
  void run();
}

watch(text, (value) => {
  clearTimeout(suggestTimer);
  const q = value.trim();
  if (q.length < 2) {
    suggestions.value = [];
    return;
  }
  suggestTimer = setTimeout(() => {
    void api
      .searchSuggest(q)
      .then((list) => {
        suggestions.value = list.filter(
          (hint) => hint.toLowerCase() !== q.toLowerCase(),
        );
      })
      .catch(() => {
        suggestions.value = [];
      });
  }, 220);
});

async function run() {
  const q = text.value.trim();
  if (!q) return;
  loading.value = true;
  result.value = await api
    .search(q)
    .catch(() => ({ tracks: [], artists: [], albums: [] }));
  loading.value = false;
  void loadAvatars(result.value?.artists ?? []);
}

watch(
  () => route.query.q,
  (q) => {
    if (typeof q === "string" && q) {
      text.value = q;
      void run();
    }
  },
);

onMounted(() => {
  const q = route.query.q;
  if (typeof q === "string" && q) {
    text.value = q;
    void run();
  }
});
onBeforeUnmount(() => {
  clearTimeout(suggestTimer);
  clearTimeout(blurTimer);
});
</script>
