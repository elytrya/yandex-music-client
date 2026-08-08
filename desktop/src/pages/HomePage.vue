<template>
  <q-page class="home-page">
    <q-scroll-area class="home-scroll">
      <div class="home-content">
        <div class="h1">{{ greeting }}</div>

        <div class="home-actions">
          <button class="btn-solid" type="button" @click="openWave">
            <Icon name="wave" :size="15" />
            <span>Моя волна</span>
          </button>

          <button class="btn" type="button" @click="router.push('/liked')">
            <Icon name="heartFilled" :size="15" />
            <span>Мне нравится</span>
          </button>

          <button class="btn" type="button" @click="router.push('/library')">
            <Icon name="search" :size="15" />
            <span>Поиск в библиотеке</span>
          </button>
        </div>

        <template v-if="visiblePlaylists.length">
          <div class="home-section-head">
            <div class="h2">Твои плейлисты</div>
            <button
              class="more-link"
              type="button"
              @click="router.push('/playlists')"
            >
              <span>Вся коллекция</span>
              <Icon name="chevronRight" :size="14" />
            </button>
          </div>

          <div class="row q-col-gutter-md">
            <div v-for="pl in visiblePlaylists" :key="pl.kind" class="col-auto">
              <div
                class="card home-card"
                style="width: 150px"
                @click="router.push(`/playlists/${pl.kind}`)"
              >
                <div class="cover home-card-art">
                  <img
                    v-if="pl.cover_url"
                    loading="lazy"
                    decoding="async"
                    :src="pl.cover_url"
                  />
                  <Icon v-else name="queue" :size="24" class="faint" />

                  <button
                    class="home-card-play"
                    type="button"
                    title="Слушать"
                    @click.stop="playPlaylist(pl)"
                  >
                    <Icon name="play" :size="16" />
                  </button>

                  <span
                    v-if="library.isPinned(pl.kind)"
                    class="home-card-pin"
                    title="Закреплён в боковой панели"
                  >
                    <Icon name="pin" :size="11" />
                  </span>
                </div>

                <div class="t-13 w-500 ellipsis q-mt-sm">{{ pl.title }}</div>
                <div class="faint t-11">
                  {{ plural(pl.track_count, "трек", "трека", "треков") }}
                </div>

                <q-menu context-menu touch-position class="menu">
                  <div class="menu-body" style="min-width: 216px">
                    <div
                      class="menu-item"
                      v-close-popup
                      @click="playPlaylist(pl)"
                    >
                      <Icon name="play" :size="17" />
                      <span>Слушать</span>
                    </div>
                    <div
                      class="menu-item"
                      v-close-popup
                      @click="router.push(`/playlists/${pl.kind}`)"
                    >
                      <Icon name="queue" :size="17" />
                      <span>Открыть плейлист</span>
                    </div>
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

                    <div class="menu-sep" />

                    <div class="menu-item" v-close-popup @click="rename(pl)">
                      <Icon name="name" :size="17" />
                      <span>Переименовать</span>
                    </div>
                    <div
                      class="menu-item"
                      v-close-popup
                      @click="library.setPlaylistPublic(pl.kind, true)"
                    >
                      <Icon name="share" :size="17" />
                      <span>Сделать публичным</span>
                    </div>
                    <div
                      class="menu-item"
                      v-close-popup
                      @click="library.setPlaylistPublic(pl.kind, false)"
                    >
                      <Icon name="person" :size="17" />
                      <span>Сделать личным</span>
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

        <template v-if="picks.length">
          <div class="home-section-head">
            <div class="h2">Новое для тебя</div>
          </div>

          <div class="row q-col-gutter-md">
            <div v-for="p in picks" :key="`${p.kind}-${p.id}`" class="col-auto">
              <div
                class="card home-card"
                style="width: 150px"
                @click="openPick(p)"
              >
                <div
                  class="cover home-card-art"
                  :class="{ round: p.kind === 'artist' }"
                  :style="p.color ? { background: p.color } : {}"
                >
                  <img
                    v-if="p.cover_url"
                    loading="lazy"
                    decoding="async"
                    :src="p.cover_url"
                  />
                  <Icon
                    v-else
                    :name="p.kind === 'artist' ? 'person' : 'album'"
                    :size="24"
                    class="faint"
                  />

                  <button
                    class="home-card-play"
                    type="button"
                    title="Слушать"
                    @click.stop="playPick(p)"
                  >
                    <Icon name="play" :size="16" />
                  </button>
                </div>

                <div class="t-13 w-500 ellipsis q-mt-sm">{{ p.name }}</div>
                <div class="faint t-11 ellipsis">
                  {{
                    p.artists.length ? artistNames(p.artists) : p.description
                  }}
                </div>

                <q-menu context-menu touch-position class="menu">
                  <div class="menu-body" style="min-width: 212px">
                    <div class="menu-item" v-close-popup @click="playPick(p)">
                      <Icon name="play" :size="17" />
                      <span>Слушать</span>
                    </div>
                    <div class="menu-item" v-close-popup @click="openPick(p)">
                      <Icon
                        :name="p.kind === 'artist' ? 'artist' : 'album'"
                        :size="17"
                      />
                      <span>{{
                        p.kind === "artist"
                          ? "Открыть артиста"
                          : "Открыть альбом"
                      }}</span>
                    </div>
                    <div class="menu-item" v-close-popup @click="waveFrom(p)">
                      <Icon name="wave" :size="17" />
                      <span>Волна по этому</span>
                    </div>
                  </div>
                </q-menu>
              </div>
            </div>
          </div>
        </template>

        <template v-if="waves.length">
          <div class="home-section-head">
            <div class="h2">Волны по настроению</div>
            <button
              class="more-link"
              type="button"
              @click="router.push('/wave')"
            >
              <span>Все станции</span>
              <Icon name="chevronRight" :size="14" />
            </button>
          </div>

          <div class="row q-col-gutter-md">
            <div v-for="w in waves" :key="w.id" class="col-auto">
              <div
                class="card home-card"
                style="width: 150px"
                @click="playWave(w)"
              >
                <div
                  class="cover round home-card-art"
                  :style="w.color ? { background: w.color } : {}"
                >
                  <img
                    v-if="w.cover_url"
                    loading="lazy"
                    decoding="async"
                    :src="w.cover_url"
                  />
                  <Icon v-else name="wave" :size="22" class="faint" />

                  <span class="home-card-play" title="Запустить">
                    <Icon name="play" :size="16" />
                  </span>
                </div>

                <div class="t-13 w-500 clamp-2 q-mt-sm">{{ w.name }}</div>
                <div class="faint t-11 ellipsis">{{ w.description }}</div>

                <q-menu context-menu touch-position class="menu">
                  <div class="menu-body" style="min-width: 200px">
                    <div class="menu-item" v-close-popup @click="playWave(w)">
                      <Icon name="play" :size="17" />
                      <span>Запустить волну</span>
                    </div>
                    <div
                      class="menu-item"
                      v-close-popup
                      @click="router.push('/wave')"
                    >
                      <Icon name="wave" :size="17" />
                      <span>Открыть страницу волны</span>
                    </div>
                  </div>
                </q-menu>
              </div>
            </div>
          </div>
        </template>

        <div v-if="loading" class="q-mt-xl">
          <q-spinner size="24px" color="primary" />
        </div>
      </div>
    </q-scroll-area>
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Notify } from "quasar";
import { api } from "@/api/client";
import type { Playlist, Track, WheelItem } from "@/api/types";
import Icon from "@/components/Icon.vue";
import { artistNames, plural } from "@/lib/format";
import { askConfirm, askText } from "@/lib/dialogs";
import { readCache, swr, writeCache } from "@/lib/cache";
import { useLibraryStore } from "@/stores/library";
import { usePlayerStore } from "@/stores/player/index";

const router = useRouter();
const player = usePlayerStore();
const library = useLibraryStore();
const playlists = ref<Playlist[]>([]);
const wheel = ref<WheelItem[]>([]);
const loading = ref(false);

const visiblePlaylists = computed(() =>
  playlists.value.filter((pl) => !library.isHidden(pl.kind)),
);

const waves = computed(() =>
  wheel.value.filter((item) => item.kind === "wave"),
);
const picks = computed(() =>
  wheel.value.filter((item) => item.kind !== "wave"),
);

const greeting = computed(() => {
  const hour = new Date().getHours();
  if (hour < 6) return "Доброй ночи";
  if (hour < 12) return "Доброе утро";
  if (hour < 18) return "Добрый день";
  return "Добрый вечер";
});

function openWave() {
  void router.push("/wave");
}

async function playPlaylist(pl: Playlist) {
  try {
    const tracks =
      readCache<Track[]>(`playlist.${pl.kind}`) ??
      (await api.playlistTracks(pl.kind));
    if (!tracks.length) {
      Notify.create({ message: "Плейлист пуст" });
      return;
    }
    await player.playQueue(tracks, 0);
  } catch {
    Notify.create({ message: "Не удалось загрузить плейлист" });
  }
}

async function playPick(item: WheelItem) {
  if (item.kind === "album") {
    try {
      const album = await api.album(item.id);
      if (!album.tracks.length) {
        Notify.create({ message: "В альбоме нет доступных треков" });
        return;
      }
      await player.playQueue(album.tracks, 0);
    } catch {
      Notify.create({ message: "Не удалось загрузить альбом" });
    }
    return;
  }
  waveFrom(item);
}

function waveFrom(item: WheelItem) {
  const station =
    item.kind === "artist" ? `artist:${item.id}` : `album:${item.id}`;
  void player.startWave(station, item.name);
  void router.push("/wave");
}

function playWave(item: WheelItem) {
  void player.startWave(item.station || item.id, item.name);
  void router.push("/wave");
}

function openPick(item: WheelItem) {
  if (item.kind === "album") void router.push(`/album/${item.id}`);
  else if (item.kind === "artist") void router.push(`/artist/${item.id}`);
}

function syncCache() {
  writeCache("home.playlists", playlists.value);
}

async function rename(pl: Playlist) {
  const next = await askText({
    title: "Переименовать плейлист",
    value: pl.title,
    placeholder: "Название плейлиста",
  });
  if (!next || next === pl.title) return;
  if (await library.renamePlaylist(pl.kind, next)) {
    pl.title = next;
    syncCache();
  }
}

async function removePlaylist(pl: Playlist) {
  const ok = await askConfirm({
    title: "Удалить плейлист?",
    message: `«${pl.title}» исчезнет из Яндекс Музыки.`,
    okLabel: "Удалить",
    danger: true,
  });
  if (!ok) return;
  if (await library.deletePlaylist(pl.kind)) {
    playlists.value = playlists.value.filter((item) => item.kind !== pl.kind);
    syncCache();
  }
}

onMounted(() => {
  playlists.value = readCache<Playlist[]>("home.playlists") ?? [];
  wheel.value = readCache<WheelItem[]>("home.wheel") ?? [];
  loading.value = !playlists.value.length && !wheel.value.length;

  void swr<Playlist[]>("home.playlists", () => api.playlists(), {
    onData: (data) => {
      playlists.value = data;
    },
    onSettled: () => {
      loading.value = false;
    },
  });

  void swr<WheelItem[]>("home.wheel", () => api.wheel(), {
    onData: (data) => {
      wheel.value = data;
    },
    onSettled: () => {
      loading.value = false;
    },
  });
});
</script>
