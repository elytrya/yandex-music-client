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
            <div class="cover head-cover round">
              <img
                v-if="person.image"
                loading="lazy"
                decoding="async"
                :src="person.image"
              />
              <Icon v-else name="person" :size="56" class="faint" />
            </div>

            <div class="head-info">
              <div class="head-kind">{{ roleLabel }} · все треки</div>
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
                <span>
                  {{ plural(person.songs.length, "трек", "трека", "треков") }}
                </span>
                <span v-if="person.followers" class="faint">
                  · {{ person.followers.toLocaleString("ru-RU") }}
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

          <div class="section-head">
            <div class="h2">Все треки</div>
            <div class="gsongs-tools">
              <input
                v-model="query"
                class="gsongs-search"
                type="text"
                placeholder="Найти трек"
              />
              <button
                class="chip chip-menu"
                type="button"
                @click="byTitle = !byTitle"
              >
                <Icon name="filter" :size="13" />
                <span>{{ byTitle ? "По названию" : "По популярности" }}</span>
              </button>
            </div>
          </div>

          <p v-if="!person.songs.length" class="dim t-13">
            Genius не отдал список треков — у продюсеров он часто пустой.
          </p>

          <p v-else-if="!shown.length" class="dim t-13">
            Ничего не нашлось по запросу «{{ query }}».
          </p>

          <div v-else class="row q-col-gutter-md">
            <div v-for="song in shown" :key="song.id" class="col-auto">
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

                <div class="t-13 w-500 ellipsis q-mt-sm">{{ song.title }}</div>
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
import Icon from "@/components/Icon.vue";
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
const query = ref("");
const byTitle = ref(false);

const id = computed(() => Number(route.params.id) || 0);
const roleLabel = computed(() => String(route.query.role || "Страница Genius"));

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

const shown = computed(() => {
  const list = person.value?.songs || [];
  const needle = query.value.trim().toLowerCase();
  const filtered = needle
    ? list.filter((song) =>
        `${song.title} ${song.artist}`.toLowerCase().includes(needle),
      )
    : list;

  if (!byTitle.value) return filtered;
  return [...filtered].sort((a, b) => a.title.localeCompare(b.title, "ru"));
});

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

function searchInApp(song: GeniusHit) {
  const text = [song.title, song.artist].filter(Boolean).join(" ");
  void router.push({ path: "/search", query: { q: text } });
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

.gsongs-tools {
  display: flex;
  align-items: center;
  gap: 10px;
}

.gsongs-search {
  width: 210px;
  padding: 6px 11px;
  border: 1px solid var(--line);
  border-radius: 9px;
  background: var(--surface);
  color: var(--fg);
  font: inherit;
  font-size: 12.5px;
  outline: none;
  transition: border-color 0.14s ease;
}

.gsongs-search:focus {
  border-color: var(--fg-dim);
}

.gsongs-search::placeholder {
  color: var(--fg-faint);
}
</style>
