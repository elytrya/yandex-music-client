<template>
  <q-page class="scroll-page">
    <q-scroll-area class="scroll-page-area">
      <div class="scroll-page-inner">
        <div v-if="!genius.ready" class="dim t-13">
          Включи Genius и добавь токен в настройках, чтобы открывать страницы
          людей.
        </div>

        <div v-else-if="loading" class="flex flex-center" style="height: 300px">
          <q-spinner size="26px" color="primary" />
        </div>

        <template v-else-if="person">
          <div class="page-head artist-head">
            <div
              class="cover head-cover round"
              :class="{ clickable: photos.length }"
              @click="openViewer"
            >
              <img
                v-if="person.image"
                loading="lazy"
                decoding="async"
                :src="person.image"
              />
              <Icon v-else name="person" :size="56" class="faint" />
              <div v-if="photos.length" class="cover-zoom">
                <Icon name="maximize" :size="22" />
              </div>
              <div v-if="photos.length > 1" class="cover-badge">
                {{ photos.length }} фото
              </div>
            </div>

            <div class="head-info">
              <div class="head-kind">{{ roleLabel }}</div>
              <div class="head-title">
                {{ person.name
                }}<span
                  v-if="person.verified"
                  class="genius-check"
                  title="Подтверждённый аккаунт"
                >
                  <Icon name="check" :size="15" />
                </span>
              </div>

              <div class="head-meta">
                <span v-if="person.followers">
                  {{ person.followers.toLocaleString("ru-RU") }}
                  {{
                    pluralWord(
                      person.followers,
                      "подписчик",
                      "подписчика",
                      "подписчиков",
                    )
                  }}
                </span>
                <span v-if="person.iq" class="faint">
                  · {{ person.iq.toLocaleString("ru-RU") }} IQ
                </span>
                <span v-if="person.songs.length" class="faint">
                  ·
                  {{ plural(person.songs.length, "трек", "трека", "треков") }}
                </span>
              </div>

              <div class="head-actions">
                <button
                  class="btn-solid"
                  type="button"
                  @click="open(person.url)"
                >
                  <Icon name="globe" :size="15" />
                  <span>Открыть на Genius</span>
                </button>

                <button class="btn" type="button" @click="load(true)">
                  <Icon name="repeat" :size="15" />
                  <span>Обновить</span>
                </button>

                <button class="btn" type="button" @click="goBack">
                  <Icon name="chevronLeft" :size="15" />
                  <span>Назад</span>
                </button>

                <button class="icon-btn" type="button" title="Ещё">
                  <Icon name="more" :size="18" />
                  <q-menu class="menu" anchor="bottom left" self="top left">
                    <div class="menu-body" style="min-width: 224px">
                      <div
                        class="menu-item"
                        v-close-popup
                        @click="open(person.url)"
                      >
                        <Icon name="globe" :size="17" />
                        <span>Открыть на Genius</span>
                      </div>
                      <div class="menu-item" v-close-popup @click="copyLink">
                        <Icon name="copy" :size="17" />
                        <span>Скопировать ссылку</span>
                      </div>
                      <div
                        v-if="photos.length"
                        class="menu-item"
                        v-close-popup
                        @click="openViewer"
                      >
                        <Icon name="maximize" :size="17" />
                        <span>{{
                          photos.length > 1 ? "Все фото" : "Открыть фото"
                        }}</span>
                      </div>
                    </div>
                  </q-menu>
                </button>
              </div>

              <div class="head-chips head-chips-links">
                <button
                  v-if="person.socials.length"
                  class="chip chip-menu"
                  type="button"
                >
                  <span>Ссылки и соцсети</span>
                  <span class="chip-count">{{ person.socials.length }}</span>
                  <Icon name="chevronDown" :size="12" />

                  <q-menu class="menu" anchor="bottom left" self="top left">
                    <div class="menu-body links-menu">
                      <div class="menu-label">Соцсети</div>
                      <div
                        v-for="social in person.socials"
                        :key="social.url"
                        class="menu-item"
                        :title="social.url"
                        v-close-popup
                        @click="open(social.url)"
                      >
                        <Icon :name="iconFor(social.kind)" :size="16" />
                        <span class="col ellipsis">{{
                          social.handle || social.kind
                        }}</span>
                      </div>
                    </div>
                  </q-menu>
                </button>

                <button
                  v-if="person.alternate_names.length"
                  class="chip chip-menu"
                  type="button"
                >
                  <span>Также известен как</span>
                  <span class="chip-count">{{
                    person.alternate_names.length
                  }}</span>
                  <Icon name="chevronDown" :size="12" />

                  <q-menu class="menu" anchor="bottom left" self="top left">
                    <div class="menu-body links-menu">
                      <div class="menu-label">Также известен как</div>
                      <div
                        v-for="name in person.alternate_names"
                        :key="name"
                        class="menu-item"
                      >
                        <span class="col ellipsis">{{ name }}</span>
                      </div>
                    </div>
                  </q-menu>
                </button>
              </div>
            </div>
          </div>

          <p v-if="person.description" class="artist-about">
            {{ person.description }}
          </p>

          <template v-if="person.songs.length">
            <div class="section-head">
              <div class="h2">Треки</div>
              <div class="section-head-actions">
                <button
                  v-if="person.songs.length > SONGS_PREVIEW"
                  class="more-link"
                  type="button"
                  @click="openAllSongs"
                >
                  <span>Показать все ({{ person.songs.length }})</span>
                  <Icon name="chevronRight" :size="15" />
                </button>
                <button
                  class="more-link"
                  type="button"
                  @click="open(person.url)"
                >
                  <span>Всё на Genius</span>
                  <Icon name="chevronRight" :size="15" />
                </button>
              </div>
            </div>

            <div class="row q-col-gutter-md">
              <div v-for="song in visibleSongs" :key="song.id" class="col-auto">
                <div
                  class="card home-card"
                  style="width: 168px"
                  @click="open(song.url)"
                >
                  <div class="cover home-card-art">
                    <img
                      v-if="song.art"
                      loading="lazy"
                      decoding="async"
                      :src="song.art"
                    />
                    <Icon v-else name="note" :size="26" class="faint" />

                    <button
                      class="home-card-play"
                      type="button"
                      title="Открыть на Genius"
                      @click.stop="open(song.url)"
                    >
                      <Icon name="globe" :size="15" />
                    </button>
                  </div>

                  <div class="t-13 w-500 ellipsis q-mt-sm">
                    {{ song.title }}
                  </div>
                  <div class="faint t-11 ellipsis">{{ song.artist }}</div>

                  <q-menu context-menu touch-position class="menu">
                    <div class="menu-body" style="min-width: 200px">
                      <div
                        class="menu-item"
                        v-close-popup
                        @click="open(song.url)"
                      >
                        <Icon name="globe" :size="17" />
                        <span>Открыть разбор на Genius</span>
                      </div>
                      <div
                        class="menu-item"
                        v-close-popup
                        @click="searchInApp(song)"
                      >
                        <Icon name="search" :size="17" />
                        <span>Найти в Яндекс Музыке</span>
                      </div>
                    </div>
                  </q-menu>
                </div>
              </div>
            </div>
          </template>

          <p v-else class="dim t-13 q-mt-lg">
            Genius не отдал список треков — у продюсеров он часто пустой.
          </p>

          <ImageViewer
            v-model:open="viewerOpen"
            :images="photos"
            :title="person.name"
          />
        </template>

        <div v-else class="dim t-13">
          {{ error || "Не удалось загрузить страницу" }}
        </div>
      </div>
    </q-scroll-area>
  </q-page>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { Notify } from "quasar";
import Icon from "@/components/Icon.vue";
import ImageViewer from "@/components/ImageViewer.vue";
import { api } from "@/api/client";
import type { GeniusArtist, GeniusHit } from "@/api/types";
import { plural, pluralWord } from "@/lib/format";
import { useGeniusStore } from "@/stores/genius";

const route = useRoute();
const router = useRouter();
const genius = useGeniusStore();

const person = ref<GeniusArtist | null>(null);
const loading = ref(false);
const error = ref("");
const viewerOpen = ref(false);

const SONGS_PREVIEW = 12;
const visibleSongs = computed(() =>
  (person.value?.songs || []).slice(0, SONGS_PREVIEW),
);

const id = computed(() => Number(route.params.id) || 0);
const roleLabel = computed(() => String(route.query.role || "Страница Genius"));

const photos = computed(() =>
  [person.value?.image, person.value?.header].filter((src): src is string =>
    Boolean(src),
  ),
);

const SOCIAL_ICONS: Record<string, string> = {
  instagram: "instagram",
  twitter: "twitter",
  x: "twitter",
  facebook: "facebook",
  youtube: "youtube",
  soundcloud: "soundcloud",
  spotify: "spotify",
  vk: "vk",
  telegram: "telegram",
  tiktok: "tiktok",
};

function iconFor(kind: string): string {
  return SOCIAL_ICONS[kind.toLowerCase()] ?? "globe";
}

async function load(force = false) {
  if (!genius.ready || !id.value) return;
  loading.value = true;
  error.value = "";
  try {
    person.value = await api.geniusArtist(genius.token.trim(), id.value, force);
  } catch (cause) {
    person.value = null;
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    loading.value = false;
  }
}

function open(url: string) {
  if (url) void api.openExternal(url);
}

function openViewer() {
  if (photos.value.length) viewerOpen.value = true;
}

async function copyLink() {
  const url = person.value?.url;
  if (!url) return;
  try {
    await navigator.clipboard.writeText(url);
    Notify.create({ message: "Ссылка скопирована" });
  } catch {
    Notify.create({ type: "negative", message: "Не вышло скопировать" });
  }
}

function searchInApp(song: GeniusHit) {
  const query = [song.title, song.artist].filter(Boolean).join(" ");
  void router.push({ path: "/search", query: { q: query } });
}

function openAllSongs() {
  void router.push({
    name: "genius-songs",
    params: { id: String(id.value) },
    query: route.query.role ? { role: String(route.query.role) } : {},
  });
}

function goBack() {
  if (window.history.length > 1) router.back();
  else void router.push("/");
}

watch(id, () => void load(), { immediate: true });
</script>

<style scoped>
.genius-check {
  display: inline-flex;
  align-items: center;
  margin-left: 7px;
  color: #ffd400;
  vertical-align: middle;
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

.section-head-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.head-cover.clickable {
  position: relative;
  cursor: pointer;
}

.cover-zoom {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: inherit;
  background: rgba(0, 0, 0, 0.32);
  color: #fff;
  inset: 0;
  opacity: 0;
  transition: opacity 0.16s ease;
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
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  font-size: 11px;
  font-weight: 500;
  pointer-events: none;
}
</style>
