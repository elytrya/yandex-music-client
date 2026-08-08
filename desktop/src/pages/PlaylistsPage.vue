<template>
  <q-page class="scroll-page">
    <q-scroll-area class="scroll-page-area">
      <div class="scroll-page-inner">
        <div class="row items-center no-wrap" style="gap: 12px">
          <div class="h1">Коллекция</div>
          <div class="col" />
          <button class="btn-solid" type="button" @click="createPlaylist">
            <Icon name="plus" :size="15" />
            <span>Новый плейлист</span>
          </button>
        </div>

        <div v-if="loading" class="q-mt-xl">
          <q-spinner size="24px" color="primary" />
        </div>

        <div v-else class="row q-col-gutter-md q-mt-md">
          <div v-for="pl in ordered" :key="pl.kind" class="col-auto">
            <div
              class="card"
              style="width: 168px"
              @click="router.push(`/playlists/${pl.kind}`)"
            >
              <div
                class="cover card-cover"
                style="width: 100%; aspect-ratio: 1"
              >
                <img
                  loading="lazy"
                  decoding="async"
                  v-if="pl.cover_url"
                  :src="pl.cover_url"
                />
                <Icon v-else name="queue" :size="26" class="faint" />
                <button
                  class="card-play"
                  type="button"
                  title="Слушать плейлист"
                  @click.stop="playPlaylist(pl)"
                >
                  <Icon name="play" :size="18" />
                </button>
              </div>
              <div class="row items-center no-wrap q-mt-sm" style="gap: 6px">
                <div class="col t-13 w-500 ellipsis">{{ pl.title }}</div>
                <div
                  class="icon-btn xs card-pin"
                  :class="{ on: library.isPinned(pl.kind) }"
                  @click.stop="library.togglePin(pl.kind)"
                >
                  <Icon
                    :name="library.isPinned(pl.kind) ? 'pinOff' : 'pin'"
                    :size="14"
                  />
                  <q-tooltip>{{
                    library.isPinned(pl.kind) ? "Открепить" : "Закрепить"
                  }}</q-tooltip>
                </div>
              </div>
              <div class="faint t-11">
                {{ plural(pl.track_count, "трек", "трека", "треков") }}
              </div>

              <q-menu context-menu touch-position class="menu">
                <div class="menu-body" style="min-width: 196px">
                  <div
                    class="menu-item"
                    v-close-popup
                    @click="library.togglePin(pl.kind)"
                  >
                    <Icon
                      :name="library.isPinned(pl.kind) ? 'pinOff' : 'pin'"
                      :size="17"
                    />
                    <span>{{
                      library.isPinned(pl.kind) ? "Открепить" : "Закрепить"
                    }}</span>
                  </div>
                  <div
                    class="menu-item"
                    v-close-popup
                    @click="router.push(`/playlists/${pl.kind}`)"
                  >
                    <Icon name="queue" :size="17" />
                    <span>Открыть плейлист</span>
                  </div>
                  <div class="menu-sep" />
                  <div class="menu-item" v-close-popup @click="rename(pl)">
                    <Icon name="name" :size="17" />
                    <span>Переименовать</span>
                  </div>
                  <div class="menu-item" v-close-popup @click="share(pl, true)">
                    <Icon name="share" :size="17" />
                    <span>Сделать публичным</span>
                  </div>
                  <div
                    class="menu-item"
                    v-close-popup
                    @click="share(pl, false)"
                  >
                    <Icon name="person" :size="17" />
                    <span>Сделать личным</span>
                  </div>
                  <div class="menu-sep" />
                  <div
                    class="menu-item"
                    v-close-popup
                    @click="library.toggleHidden(pl.kind)"
                  >
                    <Icon
                      :name="library.isHidden(pl.kind) ? 'eye' : 'eyeOff'"
                      :size="17"
                    />
                    <span>{{
                      library.isHidden(pl.kind)
                        ? "Вернуть в коллекцию"
                        : "Скрыть плейлист"
                    }}</span>
                  </div>
                  <div class="menu-item" v-close-popup @click="clearTracks(pl)">
                    <Icon name="filter" :size="17" />
                    <span>Очистить треки</span>
                  </div>
                  <div
                    class="menu-item danger"
                    v-close-popup
                    @click="removePlaylist(pl)"
                  >
                    <Icon name="trash" :size="17" />
                    <span>Удалить плейлист</span>
                  </div>
                </div>
              </q-menu>
            </div>
          </div>

          <div
            v-for="al in likedAlbums"
            :key="`album-${al.id}`"
            class="col-auto"
          >
            <div
              class="card"
              style="width: 168px"
              @click="router.push(`/album/${al.id}`)"
            >
              <div class="cover" style="width: 100%; aspect-ratio: 1">
                <img
                  loading="lazy"
                  decoding="async"
                  v-if="al.cover_url"
                  :src="al.cover_url"
                />
                <Icon v-else name="album" :size="26" class="faint" />
                <div class="card-badge t-11">Альбом</div>
              </div>
              <div class="row items-center no-wrap q-mt-sm" style="gap: 6px">
                <div class="col t-13 w-500 ellipsis">{{ al.title }}</div>
                <div
                  class="icon-btn xs card-pin on"
                  @click.stop="library.removeAlbumLike(al.id)"
                >
                  <Icon name="heartFilled" :size="14" />
                  <q-tooltip>Убрать из коллекции</q-tooltip>
                </div>
              </div>
              <div class="faint t-11 ellipsis">{{ al.artists }}</div>

              <q-menu context-menu touch-position class="menu">
                <div class="menu-body" style="min-width: 196px">
                  <div
                    class="menu-item"
                    v-close-popup
                    @click="router.push(`/album/${al.id}`)"
                  >
                    <Icon name="album" :size="17" />
                    <span>Открыть альбом</span>
                  </div>
                  <div
                    class="menu-item danger"
                    v-close-popup
                    @click="library.removeAlbumLike(al.id)"
                  >
                    <Icon name="heartOff" :size="17" />
                    <span>Убрать из коллекции</span>
                  </div>
                </div>
              </q-menu>
            </div>
          </div>

          <div
            v-if="!playlists.length && !likedAlbums.length"
            class="dim t-13 q-pa-md"
          >
            Плейлистов нет
          </div>
        </div>

        <template v-if="hiddenOrdered.length">
          <div class="row items-center no-wrap q-mt-xl">
            <div class="h2 col">Скрытые</div>
            <button type="button" class="btn" @click="showHidden = !showHidden">
              {{
                showHidden ? "Свернуть" : `Показать (${hiddenOrdered.length})`
              }}
            </button>
          </div>

          <div v-if="showHidden" class="row q-col-gutter-md q-mt-sm">
            <div
              v-for="pl in hiddenOrdered"
              :key="`hidden-${pl.kind}`"
              class="col-auto"
            >
              <div class="card playlist-hidden-card" style="width: 168px">
                <div
                  class="cover card-cover"
                  @click="router.push(`/playlists/${pl.kind}`)"
                >
                  <img
                    v-if="pl.cover_url"
                    :src="pl.cover_url"
                    loading="lazy"
                    decoding="async"
                  />
                  <Icon v-else name="queue" :size="26" class="faint" />
                </div>

                <div class="row items-center no-wrap q-mt-sm">
                  <div class="t-13 w-500 ellipsis col">{{ pl.title }}</div>
                  <button
                    type="button"
                    class="icon-btn xs"
                    title="Вернуть в коллекцию"
                    @click="library.toggleHidden(pl.kind)"
                  >
                    <Icon name="eye" :size="15" />
                  </button>
                </div>
                <div class="faint t-11">
                  {{ plural(pl.track_count, "трек", "трека", "треков") }}
                </div>

                <q-menu context-menu touch-position class="menu">
                  <div class="menu-body" style="min-width: 196px">
                    <div
                      class="menu-item"
                      v-close-popup
                      @click="library.toggleHidden(pl.kind)"
                    >
                      <Icon name="eye" :size="17" />
                      <span>Вернуть в коллекцию</span>
                    </div>
                    <div
                      class="menu-item"
                      v-close-popup
                      @click="router.push(`/playlists/${pl.kind}`)"
                    >
                      <Icon name="queue" :size="17" />
                      <span>Открыть плейлист</span>
                    </div>
                    <div class="menu-sep" />
                    <div
                      class="menu-item danger"
                      v-close-popup
                      @click="removePlaylist(pl)"
                    >
                      <Icon name="trash" :size="17" />
                      <span>Удалить плейлист</span>
                    </div>
                  </div>
                </q-menu>
              </div>
            </div>
          </div>
        </template>
      </div>
    </q-scroll-area>
  </q-page>
</template>

<script setup lang="ts">
import Icon from "@/components/Icon.vue";
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { api } from "@/api/client";
import type { Playlist, Track } from "@/api/types";
import { plural } from "@/lib/format";
import { readCache, swr, writeCache } from "@/lib/cache";
import { askConfirm, askText } from "@/lib/dialogs";
import { useLibraryStore } from "@/stores/library";
import { usePlayerStore } from "@/stores/player/index";

const router = useRouter();
const library = useLibraryStore();
const player = usePlayerStore();
const playlists = ref<Playlist[]>([]);
const loading = ref(false);

const likedAlbums = computed(() => library.sortedLikedAlbums);

const showHidden = ref(false);

const sorted = computed(() => {
  const rank = (kind: number) => {
    const at = library.pinned.indexOf(Number(kind));
    return at === -1 ? library.pinned.length + 1 : at;
  };
  return [...playlists.value].sort((a, b) => rank(a.kind) - rank(b.kind));
});

const ordered = computed(() =>
  sorted.value.filter((pl) => !library.isHidden(pl.kind)),
);

const hiddenOrdered = computed(() =>
  sorted.value.filter((pl) => library.isHidden(pl.kind)),
);

function syncFromStore() {
  playlists.value = [...library.playlists];
  writeCache("home.playlists", playlists.value);
}

async function playPlaylist(pl: Playlist) {
  const key = `playlist.${pl.kind}`;
  let list = readCache<Track[]>(key) ?? [];
  if (!list.length) {
    list = await api.playlistTracks(pl.kind).catch(() => [] as Track[]);
    if (list.length) writeCache(key, list);
  }
  if (!list.length) return;
  void player.playQueue(list, 0);
}

async function createPlaylist() {
  const title = await askText({
    title: "Новый плейлист",
    placeholder: "Название плейлиста",
    okLabel: "Создать",
  });
  if (!title) return;
  const created = await library.createPlaylist(title);
  if (created) syncFromStore();
}

async function rename(pl: Playlist) {
  const title = await askText({
    title: "Переименовать",
    value: pl.title,
    placeholder: "Название плейлиста",
  });
  if (!title || title === pl.title) return;
  if (await library.renamePlaylist(pl.kind, title)) {
    pl.title = title;
    syncFromStore();
  }
}

function share(pl: Playlist, isPublic: boolean) {
  void library.setPlaylistPublic(pl.kind, isPublic);
}

async function clearTracks(pl: Playlist) {
  const ok = await askConfirm({
    title: "Очистить плейлист?",
    message: `Из «${pl.title}» будут убраны все треки.`,
    okLabel: "Очистить",
    danger: true,
  });
  if (!ok) return;
  if (await library.clearPlaylist(pl.kind)) {
    pl.track_count = 0;
    syncFromStore();
  }
}

async function removePlaylist(pl: Playlist) {
  const ok = await askConfirm({
    title: "Удалить плейлист?",
    message: `«${pl.title}» будет удалён безвозвратно.`,
    okLabel: "Удалить",
    danger: true,
  });
  if (!ok) return;
  if (await library.deletePlaylist(pl.kind)) {
    playlists.value = playlists.value.filter((item) => item.kind !== pl.kind);
    writeCache("home.playlists", playlists.value);
  }
}

onMounted(() => {
  playlists.value = readCache<Playlist[]>("home.playlists") ?? [];
  loading.value = !playlists.value.length;

  void swr<Playlist[]>("home.playlists", () => api.playlists(), {
    onData: (data) => {
      playlists.value = data;
      library.playlists = data;
    },
    onSettled: () => {
      loading.value = false;
    },
  });
});
</script>

<style scoped>
.playlist-hidden-card {
  opacity: 0.66;
  transition: opacity 0.15s ease;
}

.playlist-hidden-card:hover {
  opacity: 1;
}
</style>
