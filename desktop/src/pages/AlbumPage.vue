<template>
  <q-page class="scroll-page">
    <q-scroll-area class="scroll-page-area">
      <div class="scroll-page-inner">
        <div v-if="loading" class="flex flex-center" style="height: 300px">
          <q-spinner size="26px" color="primary" />
        </div>

        <template v-else-if="album">
          <div class="page-head">
            <div class="cover head-cover">
              <img
                loading="lazy"
                decoding="async"
                v-if="album.cover_url"
                :src="album.cover_url"
              />
              <Icon v-else name="album" :size="42" class="faint" />
            </div>

            <div class="head-info">
              <div class="head-kind">Альбом</div>
              <div class="head-title">{{ album.title }}</div>
              <div class="head-meta">
                <ArtistsLine :artists="album.artists" :limit="3" />
                <span v-if="album.year" class="faint"> · {{ album.year }}</span>
                <span v-if="album.genre" class="faint">
                  · {{ album.genre }}</span
                >
                <span class="faint">
                  · {{ plural(album.tracks.length, "трек", "трека", "треков") }}
                </span>
              </div>

              <div class="head-actions">
                <button
                  class="btn-solid"
                  type="button"
                  :disabled="!album.tracks.length"
                  @click="playFrom(0)"
                >
                  <Icon name="play" :size="15" />
                  <span>Слушать</span>
                </button>

                <button
                  class="btn"
                  type="button"
                  :disabled="!album.tracks.length"
                  @click="shufflePlay"
                >
                  <Icon name="shuffle" :size="15" />
                  <span>Перемешать</span>
                </button>

                <button class="btn" type="button" @click="playAlbumWave">
                  <Icon name="wave" :size="15" />
                  <span>Волна по альбому</span>
                </button>

                <button
                  class="icon-btn"
                  :class="{ on: albumLiked }"
                  type="button"
                  :title="
                    albumLiked
                      ? 'Убрать альбом из коллекции'
                      : 'Сохранить альбом в коллекцию'
                  "
                  @click="toggleAlbumLike"
                >
                  <Icon
                    :name="albumLiked ? 'heartFilled' : 'heart'"
                    :size="18"
                  />
                </button>

                <button class="icon-btn" type="button" title="Ещё">
                  <Icon name="moreH" :size="18" />
                  <q-menu class="menu" anchor="bottom left" self="top left">
                    <div class="menu-body" style="min-width: 226px">
                      <div
                        class="menu-item"
                        :class="{ disabled: !album.tracks.length }"
                        v-close-popup
                        @click="enqueueAll"
                      >
                        <Icon name="addQueue" :size="17" />
                        <span>Добавить в конец очереди</span>
                      </div>
                      <div
                        class="menu-item"
                        :class="{ disabled: !album.tracks.length }"
                      >
                        <Icon name="playlistAdd" :size="17" />
                        <span>Перенести все треки в плейлист</span>
                        <Icon name="chevronRight" :size="15" class="faint" />
                        <q-menu class="menu" anchor="top end" self="top start">
                          <div class="menu-body" style="min-width: 200px">
                            <div
                              v-if="!library.playlists.length"
                              class="menu-item disabled"
                            >
                              <span>Нет плейлистов</span>
                            </div>
                            <div
                              v-for="playlist in library.playlists"
                              :key="playlist.kind"
                              class="menu-item"
                              v-close-popup
                              @click="moveAllTo(playlist.kind)"
                            >
                              <div class="cover menu-cover">
                                <img
                                  v-if="playlist.cover_url"
                                  :src="playlist.cover_url"
                                  loading="lazy"
                                  decoding="async"
                                />
                                <Icon
                                  v-else
                                  name="queue"
                                  :size="14"
                                  class="faint"
                                />
                              </div>
                              <span class="ellipsis">{{ playlist.title }}</span>
                            </div>
                          </div>
                        </q-menu>
                      </div>

                      <div class="menu-sep" />

                      <div
                        class="menu-item"
                        :class="{ disabled: !firstArtistId }"
                        v-close-popup
                        @click="openArtist"
                      >
                        <Icon name="artist" :size="17" />
                        <span>Перейти к исполнителю</span>
                      </div>
                    </div>
                  </q-menu>
                </button>
              </div>
            </div>
          </div>

          <div v-if="selectedIds.length" class="select-float">
            <button
              class="track-check-box"
              :class="{ on: allSelected }"
              type="button"
              title="Выбрать все"
              @click="toggleSelectAll"
            >
              <Icon v-if="allSelected" name="check" :size="13" />
            </button>

            <div class="select-bar-count t-12 w-600">
              {{ plural(selectedIds.length, "трек", "трека", "треков") }}
            </div>

            <div class="select-bar-actions">
              <button
                class="btn"
                type="button"
                :disabled="!selectedIds.length"
                @click="playSelected"
              >
                <Icon name="play" :size="14" />
                <span>Слушать</span>
              </button>

              <button
                class="btn"
                type="button"
                :disabled="!selectedIds.length"
                @click="enqueueSelected"
              >
                <Icon name="addQueue" :size="14" />
                <span>В очередь</span>
              </button>

              <button class="btn" type="button" :disabled="!selectedIds.length">
                <Icon name="playlistAdd" :size="14" />
                <span>В плейлист</span>
                <q-menu class="menu" anchor="bottom start" self="top start">
                  <div class="menu-body">
                    <div
                      v-if="!library.playlists.length"
                      class="menu-item disabled"
                    >
                      <span>Нет плейлистов</span>
                    </div>
                    <div
                      v-for="playlist in library.playlists"
                      :key="playlist.kind"
                      class="menu-item"
                      v-close-popup
                      @click="addSelectedTo(playlist.kind)"
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
                class="icon-btn sm"
                type="button"
                title="Снять выделение"
                @click="clearSelection"
              >
                <Icon name="close" :size="15" />
              </button>
            </div>
          </div>

          <div class="head-row q-mt-xl">
            <div style="width: 20px; text-align: center">#</div>
            <div class="col">Название</div>
            <div style="width: 40px; text-align: right">Время</div>
          </div>

          <LazyTracks v-slot="{ item, index }" :items="album.tracks">
            <TrackRow
              :track="item"
              :index="index"
              :show-cover="false"
              :show-album="false"
              :selectable="true"
              :selected="selectedIds.includes(item.id)"
              @play="playFrom(index)"
              @toggle-select="toggleRow(item.id, $event)"
            />
          </LazyTracks>
        </template>

        <div v-else class="dim t-13">Не удалось загрузить альбом</div>
      </div>
    </q-scroll-area>
  </q-page>
</template>

<script setup lang="ts">
import ArtistsLine from "@/components/ArtistsLine.vue";
import Icon from "@/components/Icon.vue";
import { computed, onMounted, ref, watch } from "vue";
import { Notify } from "quasar";
import { useRouter } from "vue-router";
import LazyTracks from "@/components/LazyTracks.vue";
import TrackRow from "@/components/TrackRow.vue";
import { api } from "@/api/client";
import type { AlbumPage } from "@/api/types";
import { plural } from "@/lib/format";
import { readCache, swr } from "@/lib/cache";
import { useLibraryStore } from "@/stores/library";
import { usePlayerStore } from "@/stores/player/index";

const props = defineProps<{ id: string }>();

const router = useRouter();
const player = usePlayerStore();
const library = useLibraryStore();

const album = ref<AlbumPage | null>(null);
const loading = ref(false);
const selectedIds = ref<string[]>([]);
const lastPicked = ref<string | null>(null);

const albumLiked = computed(() => library.albumLiked(props.id));

const allSelected = computed(() => {
  const tracks = album.value?.tracks || [];
  return (
    tracks.length > 0 &&
    tracks.every((track) => selectedIds.value.includes(track.id))
  );
});

const selectedTracks = computed(() =>
  (album.value?.tracks || []).filter((track) =>
    selectedIds.value.includes(track.id),
  ),
);

function toggleAlbumLike() {
  if (album.value) library.toggleAlbumLike(album.value);
}

function clearSelection() {
  selectedIds.value = [];
  lastPicked.value = null;
}

function toggleRow(id: string, event?: MouseEvent) {
  const ids = (album.value?.tracks || []).map((track) => track.id);

  if (event?.shiftKey && lastPicked.value) {
    const from = ids.indexOf(lastPicked.value);
    const to = ids.indexOf(id);
    if (from >= 0 && to >= 0) {
      const range = ids.slice(Math.min(from, to), Math.max(from, to) + 1);
      selectedIds.value = [...new Set([...selectedIds.value, ...range])];
      return;
    }
  }

  selectedIds.value = selectedIds.value.includes(id)
    ? selectedIds.value.filter((item) => item !== id)
    : [...selectedIds.value, id];
  lastPicked.value = id;
}

function toggleSelectAll() {
  selectedIds.value = allSelected.value
    ? []
    : (album.value?.tracks || []).map((track) => track.id);
}

function playSelected() {
  const list = selectedTracks.value;
  if (list.length) void player.playQueue(list, 0);
}

function enqueueSelected() {
  const list = selectedTracks.value;
  if (!list.length) return;
  list.forEach((track) => player.enqueue(track));
  Notify.create({ message: `В очередь добавлено: ${list.length}` });
}

async function addSelectedTo(kind: number | string) {
  const list = selectedTracks.value;
  if (!list.length) return;
  await library.addTracksToPlaylist(Number(kind), list);
}

async function moveAllTo(kind: number | string) {
  const tracks = album.value?.tracks || [];
  if (!tracks.length) return;
  await library.addTracksToPlaylist(Number(kind), tracks);
}

function playFrom(i: number) {
  const tracks = album.value?.tracks || [];
  if (!tracks.length) return;
  void player.playQueue(tracks, i);
}

const firstArtistId = computed(() => album.value?.artists?.[0]?.id ?? "");

function enqueueAll() {
  const tracks = album.value?.tracks || [];
  if (!tracks.length) return;
  tracks.forEach((track) => player.enqueue(track));
  Notify.create({ message: `В очередь добавлено: ${tracks.length}` });
}

function playAlbumWave() {
  void player.startWave(`album:${props.id}`, album.value?.title);
  void router.push("/wave");
}

function openArtist() {
  if (firstArtistId.value) void router.push(`/artist/${firstArtistId.value}`);
}

function shufflePlay() {
  const tracks = album.value?.tracks || [];
  if (!tracks.length) return;
  void player.playQueue(tracks, 0).then(() => {
    if (!player.shuffle) player.toggleShuffle();
  });
}

async function load() {
  const key = `album.${props.id}`;
  const cached = readCache<AlbumPage>(key);
  album.value = cached ?? null;
  loading.value = !cached;

  await swr<AlbumPage>(key, () => api.album(props.id), {
    onData: (data) => {
      album.value = data;
    },
    onSettled: () => {
      loading.value = false;
    },
  });
}

watch(() => props.id, load);
onMounted(load);
</script>
