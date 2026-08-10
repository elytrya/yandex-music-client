<template>
  <q-page class="scroll-page">
    <q-scroll-area class="scroll-page-area" @scroll="onScroll">
      <div class="scroll-page-inner">
        <div v-if="loading" class="flex flex-center" style="height: 300px">
          <q-spinner size="26px" color="primary" />
        </div>

        <template v-else-if="artist">
          <div
            class="page-head artist-head"
            :class="{ collapsed: headerCollapsed }"
          >
            <div
              class="cover head-cover round"
              :class="{ clickable: coverImages.length }"
              @click="openViewer"
            >
              <img
                v-if="artist.cover_url"
                loading="lazy"
                decoding="async"
                :src="artist.cover_url"
              />
              <Icon v-else name="person" :size="56" class="faint" />
              <div v-if="coverImages.length" class="cover-zoom">
                <Icon name="maximize" :size="22" />
              </div>
              <div v-if="coverImages.length > 1" class="cover-badge">
                {{ coverImages.length }} фото
              </div>
            </div>

            <div class="head-info">
              <div class="head-kind">Артист</div>
              <div class="head-title">
                {{ artist.name }}<AiTag :show="isAiArtistPage" />
              </div>

              <div class="head-meta">
                <span v-if="artist.listeners">
                  {{ formatCount(artist.listeners) }}
                  {{
                    pluralWord(
                      artist.listeners,
                      "слушатель",
                      "слушателя",
                      "слушателей",
                    )
                  }}
                  в месяц
                </span>
                <span v-if="artist.likes" class="faint">
                  · {{ formatCount(artist.likes) }}
                  {{ pluralWord(artist.likes, "лайк", "лайка", "лайков") }}
                </span>
                <span v-if="artist.tracks_count" class="faint">
                  · {{ plural(artist.tracks_count, "трек", "трека", "треков") }}
                </span>
                <span v-if="artist.albums_count" class="faint">
                  ·
                  {{
                    plural(artist.albums_count, "альбом", "альбома", "альбомов")
                  }}
                </span>
              </div>

              <div class="head-actions">
                <button
                  class="btn-solid"
                  type="button"
                  :disabled="!popular.length"
                  @click="playPopular(0)"
                >
                  <Icon name="play" :size="15" />
                  <span>Слушать</span>
                </button>

                <button
                  class="btn"
                  type="button"
                  :disabled="!popular.length"
                  @click="shufflePlay"
                >
                  <Icon name="shuffle" :size="15" />
                  <span>Перемешать</span>
                </button>

                <button class="btn" type="button" @click="playArtistWave">
                  <Icon name="wave" :size="15" />
                  <span>Волна по артисту</span>
                </button>

                <button class="icon-btn" type="button" title="Ещё">
                  <Icon name="more" :size="18" />
                  <q-menu class="menu" anchor="bottom left" self="top left">
                    <div class="menu-body" style="min-width: 224px">
                      <div
                        class="menu-item"
                        :class="{ disabled: !popular.length }"
                        v-close-popup
                        @click="enqueuePopular"
                      >
                        <Icon name="addQueue" :size="17" />
                        <span>Популярное в очередь</span>
                      </div>
                      <div
                        class="menu-item"
                        v-close-popup
                        @click="openAllTracks"
                      >
                        <Icon name="library" :size="17" />
                        <span>Все треки артиста</span>
                      </div>
                      <div
                        v-if="coverImages.length"
                        class="menu-item"
                        v-close-popup
                        @click="openViewer"
                      >
                        <Icon name="maximize" :size="17" />
                        <span>{{
                          coverImages.length > 1
                            ? "Все фото артиста"
                            : "Открыть фото"
                        }}</span>
                      </div>
                    </div>
                  </q-menu>
                </button>
              </div>

              <div class="head-chips head-chips-links">
                <button
                  v-if="normalizedLinks.length"
                  class="chip chip-menu"
                  type="button"
                >
                  <span>Ссылки и соцсети</span>
                  <span class="chip-count">{{ normalizedLinks.length }}</span>
                  <Icon name="chevronDown" :size="12" />

                  <q-menu class="menu" anchor="bottom left" self="top left">
                    <div class="menu-body links-menu">
                      <template v-if="officialLinks.length">
                        <div class="menu-label">Официальные</div>
                        <div
                          v-for="link in officialLinks"
                          :key="link.href"
                          class="menu-item"
                          :title="link.href"
                          v-close-popup
                          @click="openLink(link.href)"
                        >
                          <Icon :name="iconForLink(link)" :size="16" />
                          <span class="col ellipsis">{{ link.title }}</span>
                        </div>
                      </template>

                      <template v-if="socialLinks.length">
                        <div class="menu-label">Соцсети</div>
                        <div
                          v-for="link in socialLinks"
                          :key="link.href"
                          class="menu-item"
                          :title="link.href"
                          v-close-popup
                          @click="openLink(link.href)"
                        >
                          <Icon :name="iconForLink(link)" :size="16" />
                          <span class="col ellipsis">{{ link.title }}</span>
                        </div>
                      </template>
                    </div>
                  </q-menu>
                </button>

                <button
                  class="chip chip-menu"
                  type="button"
                  @click="infoOpen = true"
                >
                  <span>Подробнее</span>
                </button>
              </div>
            </div>
          </div>

          <p v-if="artist.description" class="artist-about">
            {{ artist.description }}
          </p>

          <template v-if="popular.length">
            <div class="section-head">
              <div class="h2">Популярное</div>
              <button class="more-link" type="button" @click="openAllTracks">
                <span>Все треки</span>
                <Icon name="chevronRight" :size="15" />
              </button>
            </div>

            <div class="head-row">
              <div style="width: 20px; text-align: center">#</div>
              <div style="width: 38px" />
              <div class="col">Название</div>
              <div style="width: 40px; text-align: right">Время</div>
            </div>

            <LazyTracks v-slot="{ item, index }" :items="popular">
              <TrackRow
                :track="item"
                :index="index"
                @play="playPopular(index)"
              />
            </LazyTracks>
          </template>

          <template v-if="artist.albums.length">
            <div class="section-head">
              <div class="h2">Альбомы</div>
              <div class="album-tools">
                <button class="btn" type="button" title="Сортировка">
                  <Icon name="filter" :size="14" />
                  <span>{{ albumSortLabel }}</span>
                  <Icon name="chevronDown" :size="12" />
                  <q-menu class="menu" anchor="bottom right" self="top right">
                    <div class="menu-body" style="min-width: 190px">
                      <div
                        v-for="opt in albumSortOptions"
                        :key="opt.id"
                        class="menu-item"
                        v-close-popup
                        @click="albumSort = opt.id"
                      >
                        <Icon
                          :name="albumSort === opt.id ? 'check' : 'name'"
                          :size="16"
                        />
                        <span>{{ opt.label }}</span>
                      </div>
                    </div>
                  </q-menu>
                </button>
                <button
                  class="btn"
                  type="button"
                  :title="albumDir === 'asc' ? 'По возрастанию' : 'По убыванию'"
                  @click="albumDir = albumDir === 'asc' ? 'desc' : 'asc'"
                >
                  <span class="t-13 w-600">{{
                    albumDir === "asc" ? "↑" : "↓"
                  }}</span>
                </button>
              </div>
            </div>

            <div class="row q-col-gutter-md">
              <div
                v-for="album in sortedAlbums"
                :key="album.id"
                class="col-auto"
              >
                <div
                  class="card home-card"
                  style="width: 168px"
                  @click="router.push(`/album/${album.id}`)"
                >
                  <div class="cover home-card-art">
                    <img
                      v-if="album.cover_url"
                      loading="lazy"
                      decoding="async"
                      :src="album.cover_url"
                    />
                    <Icon v-else name="album" :size="26" class="faint" />

                    <div v-if="album.id === latestAlbumId" class="album-latest">
                      Последний релиз
                    </div>

                    <button
                      class="home-card-play"
                      type="button"
                      title="Слушать альбом"
                      @click.stop="playAlbum(album.id)"
                    >
                      <Icon name="play" :size="16" />
                    </button>
                  </div>

                  <div class="t-13 w-500 ellipsis q-mt-sm">
                    {{ album.title }}
                  </div>
                  <div class="faint t-11">{{ album.year || "" }}</div>

                  <q-menu context-menu touch-position class="menu">
                    <div class="menu-body" style="min-width: 200px">
                      <div
                        class="menu-item"
                        v-close-popup
                        @click="playAlbum(album.id)"
                      >
                        <Icon name="play" :size="17" />
                        <span>Слушать альбом</span>
                      </div>
                      <div
                        class="menu-item"
                        v-close-popup
                        @click="router.push(`/album/${album.id}`)"
                      >
                        <Icon name="album" :size="17" />
                        <span>Открыть альбом</span>
                      </div>
                    </div>
                  </q-menu>
                </div>
              </div>
            </div>
          </template>

          <ImageViewer
            v-model:open="viewerOpen"
            :images="coverImages"
            :title="artist.name"
          />

          <ArtistInfoDialog
            v-model:open="infoOpen"
            :artist="artist"
            :links="normalizedLinks"
          />
        </template>

        <div v-else class="dim t-13">Не удалось загрузить артиста</div>
      </div>
    </q-scroll-area>
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { Notify } from "quasar";
import { api } from "@/api/client";

import type { ArtistLink, ArtistPage } from "@/api/types";
import Icon from "@/components/Icon.vue";
import LazyTracks from "@/components/LazyTracks.vue";
import TrackRow from "@/components/TrackRow.vue";
import ImageViewer from "@/components/ImageViewer.vue";
import ArtistInfoDialog from "@/components/ArtistInfoDialog.vue";
import AiTag from "@/components/AiTag.vue";
import { plural, pluralWord } from "@/lib/format";
import { readCache, swr } from "@/lib/cache";
import { recordArtistStats } from "@/lib/artistStats";
import { ensureAiArtists, isAiArtist } from "@/lib/aiTag";
import { usePlayerStore } from "@/stores/player/index";

const props = defineProps<{ id: string }>();
const router = useRouter();
const player = usePlayerStore();
const artist = ref<ArtistPage | null>(null);
const loading = ref(false);
const popular = computed(() => artist.value?.tracks ?? []);
const viewerOpen = ref(false);
const infoOpen = ref(false);
const headerCollapsed = ref(false);

type AlbumSort = "year" | "title";
const albumSort = ref<AlbumSort>("year");
const albumDir = ref<"asc" | "desc">("desc");
const albumSortOptions: Array<{ id: AlbumSort; label: string }> = [
  { id: "year", label: "По году" },
  { id: "title", label: "По названию" },
];
const albumSortLabel = computed(
  () => albumSortOptions.find((o) => o.id === albumSort.value)?.label ?? "",
);
const sortedAlbums = computed(() => {
  const list = [...(artist.value?.albums ?? [])];
  list.sort((a, b) => {
    if (albumSort.value === "year") return (a.year || 0) - (b.year || 0);
    return (a.title || "").localeCompare(b.title || "", "ru");
  });
  if (albumDir.value === "desc") list.reverse();
  return list;
});
const latestAlbumId = computed(() => {
  const list = artist.value?.albums ?? [];
  if (!list.length) return null;
  let best = list[0];
  for (const al of list) {
    if ((al.year || 0) > (best.year || 0)) best = al;
  }
  return best.id;
});

function onScroll(info: { verticalPosition: number }) {
  const y = info.verticalPosition;
  if (!headerCollapsed.value && y > 220) headerCollapsed.value = true;
  else if (headerCollapsed.value && y < 80) headerCollapsed.value = false;
}
const isAiArtistPage = computed(() => isAiArtist(artist.value?.id ?? null));
watch(
  () => artist.value?.id,
  (id) => {
    if (id) ensureAiArtists([id]);
  },
);

function openLink(url: string) {
  void api.openExternal(url);
}

const socialTitles: Record<string, string> = {
  vk: "ВКонтакте",
  telegram: "Telegram",
  instagram: "Instagram",
  youtube: "YouTube",
  twitter: "X",
  tiktok: "TikTok",
  facebook: "Facebook",
  soundcloud: "SoundCloud",
  spotify: "Spotify",
  apple: "Apple Music",
  bandcamp: "Bandcamp",
  twitch: "Twitch",
  dzen: "Дзен",
  ok: "Одноклассники",
};

function hostOf(href: string): string {
  try {
    return new URL(href).hostname.replace(/^www\./, "");
  } catch {
    return href;
  }
}

function linkLabel(link: ArtistLink): string {
  const network = (link.network || "").toLowerCase();
  if (network && socialTitles[network]) return socialTitles[network];
  const title = (link.title || "").trim();
  if (title && !/^https?:\/\//i.test(title)) return title;
  return hostOf(link.href);
}

const socialIcons: Record<string, string> = {
  vk: "vk",
  telegram: "telegram",
  instagram: "instagram",
  youtube: "youtube",
  twitter: "twitter",
  x: "twitter",
  tiktok: "tiktok",
  facebook: "facebook",
  soundcloud: "soundcloud",
  spotify: "spotify",
  apple: "globe",
  itunes: "globe",
  bandcamp: "bandcamp",
  twitch: "twitch",
  dzen: "dzen",
  zen: "dzen",
  ok: "ok",
  odnoklassniki: "ok",
  discord: "discord",
};

const hostIcons: Array<[string, string]> = [
  ["vk.com", "vk"],
  ["vk.ru", "vk"],
  ["t.me", "telegram"],
  ["telegram", "telegram"],
  ["instagram", "instagram"],
  ["youtube", "youtube"],
  ["youtu.be", "youtube"],
  ["twitter", "twitter"],
  ["x.com", "twitter"],
  ["tiktok", "tiktok"],
  ["facebook", "facebook"],
  ["fb.com", "facebook"],
  ["soundcloud", "soundcloud"],
  ["spotify", "spotify"],
  ["apple", "globe"],
  ["bandcamp", "bandcamp"],
  ["twitch", "twitch"],
  ["dzen.ru", "dzen"],
  ["zen.yandex", "dzen"],
  ["ok.ru", "ok"],
  ["discord", "discord"],
];

function iconForLink(link: { network?: string; href: string }): string {
  const network = (link.network || "").toLowerCase();
  if (network && socialIcons[network]) return socialIcons[network]!;
  const host = hostOf(link.href).toLowerCase();
  const hit = hostIcons.find(([needle]) => host.includes(needle));
  return hit ? hit[1] : "globe";
}

const normalizedLinks = computed(() => {
  const list = artist.value?.links ?? [];
  const seen = new Set<string>();
  return list
    .filter((link) => {
      if (!link.href || seen.has(link.href)) return false;
      seen.add(link.href);
      return true;
    })
    .map((link) => ({
      href: link.href,
      title: linkLabel(link),
      network: (link.network || "").toLowerCase(),
      social:
        (link.kind || "").toLowerCase() === "social" || Boolean(link.network),
    }));
});

const officialLinks = computed(() =>
  normalizedLinks.value.filter((link) => !link.social),
);
const socialLinks = computed(() =>
  normalizedLinks.value.filter((link) => link.social),
);

const coverImages = computed<string[]>(() => {
  const list = artist.value?.covers ?? [];
  if (list.length) return list;
  return artist.value?.cover_url ? [artist.value.cover_url] : [];
});

function openViewer() {
  if (coverImages.value.length) viewerOpen.value = true;
}

function playPopular(index: number) {
  if (popular.value.length) void player.playQueue(popular.value, index);
}

function openAllTracks() {
  void router.push(`/artist/${props.id}/tracks`);
}

function formatCount(value: number): string {
  return value.toLocaleString("ru-RU");
}

function shufflePlay() {
  if (!popular.value.length) return;
  void player.playQueue(popular.value, 0).then(() => {
    if (!player.shuffle) player.toggleShuffle();
  });
}

function enqueuePopular() {
  if (!popular.value.length) return;
  popular.value.forEach((track) => player.enqueue(track));
  Notify.create({ message: `В очередь добавлено: ${popular.value.length}` });
}

function playArtistWave() {
  void player.startWave(`artist:${props.id}`, artist.value?.name);
  void router.push("/wave");
}

async function playAlbum(id: string) {
  try {
    const album = await api.album(id);
    if (!album.tracks.length) {
      Notify.create({ message: "В альбоме нет доступных треков" });
      return;
    }
    await player.playQueue(album.tracks, 0);
  } catch {
    Notify.create({ message: "Не удалось загрузить альбом" });
  }
}

async function load() {
  const key = `artist.${props.id}`;
  headerCollapsed.value = false;
  const cached = readCache<ArtistPage>(key);
  artist.value = cached ?? null;
  loading.value = !cached;

  if (cached) recordArtistStats(cached);

  await swr<ArtistPage>(key, () => api.artist(props.id), {
    onData: (data) => {
      artist.value = data;
      recordArtistStats(data);
    },
    onSettled: () => {
      loading.value = false;
    },
  });
}

watch(() => props.id, load);
onMounted(load);
</script>

<style scoped>
.chip.link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.chip.link.social {
  text-transform: none;
}
.chip-social {
  display: inline-flex;
  width: 28px;
  height: 28px;
  align-items: center;
  justify-content: center;
  padding: 0;
  cursor: pointer;
  transition:
    color 0.14s ease,
    background 0.14s ease,
    transform 0.14s ease;
}

.chip-social:hover {
  background: var(--hover);
  color: var(--accent);
  transform: translateY(-1px);
}

.head-chips-links {
  margin-top: 10px;
}

.chip-menu {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1px solid var(--line);
  background: transparent;
  color: var(--fg);
  font: inherit;
  font-size: 12px;
  text-transform: none;
  cursor: pointer;
  transition:
    background 0.14s ease,
    border-color 0.14s ease;
}
.chip-menu:hover {
  background: var(--hover);
  border-color: var(--fg-dim);
}
.chip-count {
  padding: 0 5px;
  border-radius: 7px;
  background: var(--surface-2);
  color: var(--fg-dim);
  font-size: 10px;
  font-weight: 600;
}
.links-menu {
  min-width: 236px;
  max-height: 340px;
  overflow-y: auto;
}
.menu-label {
  padding: 8px 12px 4px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--fg-dim);
}
.head-cover.clickable {
  cursor: pointer;
  position: relative;
}
.cover-zoom {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.32);
  color: #fff;
  opacity: 0;
  transition: opacity 0.15s ease;
  border-radius: inherit;
}
.head-cover.clickable:hover .cover-zoom {
  opacity: 1;
}
.cover-badge {
  position: absolute;
  right: 6px;
  bottom: 6px;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  pointer-events: none;
}
</style>
