<template>
  <div class="settings-page">
    <aside class="settings-nav">
      <div class="settings-nav-title">Настройки</div>

      <button
        v-for="section in sections"
        :key="section.id"
        class="settings-nav-item"
        :class="{ on: active === section.id }"
        type="button"
        @click="select(section.id)"
      >
        <Icon :name="section.icon" :size="15" />
        {{ section.label }}
      </button>

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
import { useEqualizerStore } from "@/stores/equalizer";
import { usePlayerStore } from "@/stores/player/index";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();
const equalizer = useEqualizerStore();
const player = usePlayerStore();

const scrollRoot = ref<HTMLElement | null>(null);

const sections = [
  {
    id: "theme",
    label: "Оформление",
    icon: "display",
    component: AppearanceSection,
  },
  {
    id: "layout",
    label: "Интерфейс",
    icon: "layout",
    component: LayoutSection,
  },
  { id: "player", label: "Плеер", icon: "play", component: PlayerSection },
  {
    id: "mini",
    label: "Мини-плеер",
    icon: "mini",
    component: MiniPlayerSection,
  },
  {
    id: "behavior",
    label: "Поведение",
    icon: "settings",
    component: BehaviorSection,
  },
  {
    id: "playlists",
    label: "Плейлисты",
    icon: "library",
    component: PlaylistToolsSection,
  },
  {
    id: "hotkeys",
    label: "Горячие клавиши",
    icon: "key",
    component: HotkeysSection,
  },
  {
    id: "lyrics",
    label: "Текст песни",
    icon: "lyrics",
    component: LyricsSection,
  },
  {
    id: "genius",
    label: "Genius",
    icon: "note",
    component: GeniusSection,
  },
  {
    id: "equalizer",
    label: "Эквалайзер",
    icon: "wave",
    component: EqualizerSection,
  },
  {
    id: "downloads",
    label: "Загрузки",
    icon: "download",
    component: DownloadsSection,
  },
  {
    id: "discord",
    label: "Discord",
    icon: "discord",
    component: DiscordSection,
  },
  { id: "cache", label: "Кеш", icon: "trash", component: CacheSection },
  { id: "about", label: "О проекте", icon: "info", component: AboutSection },
];

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
