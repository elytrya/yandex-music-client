<template>
  <div class="settings-page">
    <aside class="settings-nav">
      <div class="settings-nav-head">
        <div class="settings-nav-title">Настройки</div>

        <label class="settings-search">
          <Icon name="search" :size="14" />

          <input
            v-model="query"
            type="text"
            placeholder="Найти настройку"
            spellcheck="false"
          />

          <button
            v-if="query"
            class="settings-search-clear"
            type="button"
            @click="query = ''"
          >
            <Icon name="close" :size="11" />
          </button>
        </label>
      </div>

      <div class="settings-nav-list">
        <div
          v-for="group in groups"
          :key="group.title"
          class="settings-nav-group"
        >
          <div class="settings-nav-group-title">{{ group.title }}</div>

          <button
            v-for="section in group.items"
            :key="section.id"
            class="settings-nav-item"
            :class="{ on: active === section.id, hits: section.hits.length }"
            type="button"
            @click="select(section.id)"
          >
            <Icon :name="section.icon" :size="15" />

            <span class="settings-nav-text">
              <span class="settings-nav-label">{{ section.label }}</span>

              <span v-if="section.hits.length" class="settings-nav-hits">
                {{ section.hits.join(" · ") }}
              </span>
            </span>
          </button>
        </div>

        <p v-if="!groups.length" class="settings-nav-empty">
          Ничего не нашлось
        </p>
      </div>

      <div class="settings-nav-divider" />

      <button class="settings-reset-button danger" type="button" @click="reset">
        Сбросить всё
      </button>
    </aside>

    <div ref="scrollRoot" class="settings-scroll">
      <div class="settings-content">
        <component :is="activeComponent" :key="active" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRoute } from "vue-router";
import { Notify } from "quasar";
import { askConfirm } from "@/lib/dialogs";
import Icon from "@/components/Icon.vue";
import AppearanceSection from "@/components/settings/sections/AppearanceSection.vue";
import AboutSection from "@/components/settings/sections/AboutSection.vue";
import BehaviorSection from "@/components/settings/sections/BehaviorSection.vue";
import CacheSection from "@/components/settings/sections/CacheSection.vue";
import DiscordSection from "@/components/settings/sections/DiscordSection.vue";
import DownloadsSection from "@/components/settings/sections/DownloadsSection.vue";
import EqualizerSection from "@/components/settings/sections/EqualizerSection.vue";
import GeniusSection from "@/components/settings/sections/GeniusSection.vue";
import HotkeysSection from "@/components/settings/sections/HotkeysSection.vue";
import LayoutSection from "@/components/settings/sections/LayoutSection.vue";
import LyricsSection from "@/components/settings/sections/LyricsSection.vue";
import MiniPlayerSection from "@/components/settings/sections/MiniPlayerSection.vue";
import PlayerSection from "@/components/settings/sections/PlayerSection.vue";
import PlaylistToolsSection from "@/components/settings/sections/PlaylistToolsSection.vue";
import TogetherSection from "@/components/settings/sections/TogetherSection.vue";
import { matchItems } from "@/lib/settings-catalog";
import { useEqualizerStore } from "@/stores/equalizer";
import { usePlayerStore } from "@/stores/player/index";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();
const equalizer = useEqualizerStore();
const player = usePlayerStore();

const scrollRoot = ref<HTMLElement | null>(null);
const query = ref("");

const GROUPS = ["Основное", "Плеер", "Библиотека", "Сеть", "Система"];

const sections = [
  {
    id: "theme",
    label: "Оформление",
    icon: "display",
    group: "Основное",
    hint: "тема цвет акцент шрифт фон",
    component: AppearanceSection,
  },
  {
    id: "layout",
    label: "Интерфейс",
    icon: "layout",
    group: "Основное",
    hint: "макет сайдбар панели очередь",
    component: LayoutSection,
  },
  {
    id: "behavior",
    label: "Поведение",
    icon: "settings",
    group: "Основное",
    hint: "трей запуск закрытие окно",
    component: BehaviorSection,
  },
  {
    id: "player",
    label: "Плеер",
    icon: "play",
    group: "Плеер",
    hint: "качество громкость воспроизведение",
    component: PlayerSection,
  },
  {
    id: "mini",
    label: "Мини-плеер",
    icon: "mini",
    group: "Плеер",
    hint: "маленькое окно поверх всех",
    component: MiniPlayerSection,
  },
  {
    id: "equalizer",
    label: "Эквалайзер",
    icon: "wave",
    group: "Плеер",
    hint: "звук басы частоты",
    component: EqualizerSection,
  },
  {
    id: "lyrics",
    label: "Текст песни",
    icon: "lyrics",
    group: "Плеер",
    hint: "слова караоке строки",
    component: LyricsSection,
  },
  {
    id: "playlists",
    label: "Плейлисты",
    icon: "library",
    group: "Библиотека",
    hint: "скрытые сортировка главное",
    component: PlaylistToolsSection,
  },
  {
    id: "downloads",
    label: "Загрузки",
    icon: "download",
    group: "Библиотека",
    hint: "скачивание папка файлы",
    component: DownloadsSection,
  },
  {
    id: "cache",
    label: "Кеш",
    icon: "trash",
    group: "Библиотека",
    hint: "очистка место обложки",
    component: CacheSection,
  },
  {
    id: "together",
    label: "Слушать вместе",
    icon: "person",
    group: "Сеть",
    hint: "комната радмин локалка синхрон",
    component: TogetherSection,
  },
  {
    id: "genius",
    label: "Genius",
    icon: "note",
    group: "Сеть",
    hint: "genius тексты токен",
    component: GeniusSection,
  },
  {
    id: "discord",
    label: "Discord",
    icon: "discord",
    group: "Сеть",
    hint: "discord дискорд статус rpc",
    component: DiscordSection,
  },
  {
    id: "hotkeys",
    label: "Горячие клавиши",
    icon: "key",
    group: "Система",
    hint: "кнопки сочетания глобальные",
    component: HotkeysSection,
  },
  {
    id: "about",
    label: "О проекте",
    icon: "info",
    group: "Система",
    hint: "версия github автор",
    component: AboutSection,
  },
];

const groups = computed(() => {
  const needle = query.value.trim().toLowerCase();

  return GROUPS.map((title) => ({
    title,
    items: sections
      .filter((section) => section.group === title)
      .map((section) => ({
        ...section,
        // подпункты раздела, попавшие в запрос
        hits: matchItems(section.id, needle).slice(0, 2),
      }))
      .filter(
        (section) =>
          !needle ||
          section.label.toLowerCase().includes(needle) ||
          section.hint.includes(needle) ||
          section.hits.length > 0,
      ),
  })).filter((group) => group.items.length > 0);
});

const route = useRoute();

function sectionFromQuery(): string | null {
  const raw = route.query.section;
  const id = Array.isArray(raw) ? raw[0] : raw;
  if (typeof id !== "string") return null;
  return sections.some((s) => s.id === id) ? id : null;
}

const active = ref(sectionFromQuery() ?? sections[0]!.id);

watch(
  () => route.query.section,
  () => {
    const id = sectionFromQuery();
    if (id && id !== active.value) select(id);
  },
);

const activeComponent = computed(
  () => sections.find((s) => s.id === active.value)?.component ?? null,
);

function select(id: string) {
  active.value = id;
  scrollRoot.value?.scrollTo({ top: 0, behavior: "auto" });
}

async function reset() {
  const ok = await askConfirm({
    title: "Сбросить все настройки?",
    message:
      "Оформление, интерфейс, текст песни, эквалайзер и Discord вернутся к значениям по умолчанию. Авторизация и кеш останутся на месте.",
    okLabel: "Сбросить",
    danger: true,
  });
  if (!ok) return;
  ui.reset();
  equalizer.reset();
  equalizer.toggle(false);
  void player.syncPresence(true);
  select(sections[0]!.id);
  Notify.create({ message: "Настройки сброшены" });
}
</script>
