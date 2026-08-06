<template>
  <q-layout
    view="hHh lpR fFf"
    class="app-shell"
    :class="{ 'queue-open': rightPanel }"
  >
    <q-header v-if="!panels.hidden && !panels.mini" class="topbar">
      <div
        class="titlebar"
        @mousedown="startWindowDrag"
        @dblclick="toggleWindowMaximize"
      >
        <div class="topbar-search-wrap" data-no-drag>
          <div class="searchbar">
            <Icon name="search" :size="15" class="faint" />
            <input
              v-model="query"
              placeholder="Найти трек, артиста или альбом"
              @keyup.enter="submitSearch"
            />
            <div v-if="query" class="icon-btn xs round" @click="query = ''">
              <Icon name="close" :size="13" />
            </div>
          </div>
        </div>
        <div class="topbar-drag-space" />

        <div
          class="icon-btn sm round"
          :class="{ on: rightPanel }"
          @click="rightPanel = !rightPanel"
        >
          <Icon name="queue" :size="17" />
          <q-tooltip>Очередь</q-tooltip>
        </div>

        <div class="account">
          <div class="account-avatar">
            <img
              v-if="avatarUrl"
              :src="avatarUrl"
              loading="lazy"
              decoding="async"
              referrerpolicy="no-referrer"
              @error="avatarError = true"
            />
            <span v-else class="avatar-monogram">{{ accountInitial }}</span>
          </div>
          <span class="account-name ellipsis">{{ accountName }}</span>
          <Icon name="chevronDown" :size="13" class="account-chevron" />
          <q-menu class="menu" anchor="bottom right" self="top right">
            <div class="menu-body profile-menu">
              <div class="profile-menu-head">
                <div class="profile-menu-avatar">
                  <img
                    v-if="avatarUrl"
                    :src="avatarUrl"
                    loading="lazy"
                    decoding="async"
                    referrerpolicy="no-referrer"
                    @error="avatarError = true"
                  />
                  <span v-else class="avatar-monogram">{{
                    accountInitial
                  }}</span>
                </div>
                <div class="profile-menu-id">
                  <div class="profile-menu-name ellipsis">
                    {{ accountName }}
                  </div>
                  <div class="profile-menu-sub ellipsis">{{ accountSub }}</div>
                </div>
              </div>

              <div class="profile-menu-actions">
                <div
                  class="menu-item"
                  v-close-popup
                  @click="router.push('/settings')"
                >
                  <Icon name="settings" :size="16" />
                  <span>Настройки</span>
                </div>
                <div class="menu-item danger" v-close-popup @click="doLogout">
                  <Icon name="logout" :size="16" />
                  <span>Выйти</span>
                </div>
              </div>
            </div>
          </q-menu>
        </div>

        <WindowControls />
      </div>
    </q-header>

    <q-drawer
      v-if="!panels.hidden && !panels.mini"
      :model-value="true"
      show-if-above
      side="left"
      :width="ui.settings.sidebarWidth"
      :breakpoint="0"
      class="sidebar"
    >
      <div class="column full-height">
        <div class="brand">Mashiro</div>

        <div class="nav-group">
          <div
            v-for="item in nav"
            :key="item.to"
            class="nav"
            :class="{ on: route.path === item.to }"
            @click="router.push(item.to)"
          >
            <Icon :name="item.icon" :size="18" />
            <span>{{ item.label }}</span>
          </div>
        </div>

        <div class="nav-sep" />

        <div class="row items-center no-wrap side-head">
          <span class="col">Плейлисты</span>
          <span class="faint t-11">{{ library.playlists.length || "" }}</span>
        </div>

        <q-scroll-area class="col">
          <div
            v-for="pl in library.sortedPlaylists"
            :key="pl.kind"
            class="side-item"
            :class="{ on: route.path === `/playlists/${pl.kind}` }"
            @click="router.push(`/playlists/${pl.kind}`)"
          >
            <div
              class="cover"
              style="width: 30px; height: 30px; border-radius: 7px"
            >
              <img
                loading="lazy"
                decoding="async"
                v-if="pl.cover_url"
                :src="pl.cover_url"
              />
              <Icon v-else name="queue" :size="14" class="faint" />
            </div>
            <div class="col" style="min-width: 0">
              <div class="ellipsis t-13">{{ pl.title }}</div>
            </div>
            <Icon
              v-if="library.isPinned(pl.kind)"
              name="pin"
              :size="13"
              class="faint"
            />
            <span v-else class="faint t-11">{{ pl.track_count }}</span>

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
              </div>
            </q-menu>
          </div>

          <div
            v-if="!library.playlists.length"
            class="faint t-12 q-px-lg q-py-sm"
          >
            Пусто
          </div>
        </q-scroll-area>
      </div>
    </q-drawer>

    <q-drawer
      v-if="!panels.hidden && !panels.mini"
      v-model="rightPanel"
      side="right"
      :width="300"
      :breakpoint="0"
      class="rightbar"
    >
      <div class="column full-height" style="overflow: hidden">
        <div class="side-head">Сейчас играет</div>

        <div v-if="player.current" class="now-block">
          <div
            class="cover"
            style="width: 100%; height: 268px; border-radius: 12px"
          >
            <img
              loading="lazy"
              decoding="async"
              v-if="player.current.cover_url"
              :src="player.current.cover_url"
            />
            <Icon v-else name="note" :size="30" class="faint" />
          </div>
          <div class="q-mt-md t-15 w-600 ellipsis">
            {{ player.current.title }}
          </div>
          <div class="dim t-13 clamp-2">
            <ArtistsLine :artists="player.current.artists" :limit="2" />
          </div>
        </div>
        <div v-else class="now-empty faint t-12">Очередь пуста</div>

        <div class="side-head queue-head">
          <span class="col">Очередь</span>
          <span v-if="queueItems.length" class="faint t-11 queue-pos">
            {{ player.index + 1 }} / {{ queueItems.length }}
          </span>
          <button
            v-if="queueItems.length"
            type="button"
            class="queue-locate"
            :class="{ off: followCurrent }"
            title="Пролистать к текущему треку"
            @click="locateCurrent"
          >
            <Icon name="queue" :size="13" />
          </button>
        </div>

        <q-scroll-area
          ref="queueScroll"
          class="col queue-scroll"
          @scroll="onQueueScroll"
        >
          <div
            v-for="item in queueItems"
            :key="`${item.track.id}-${item.index}`"
            class="side-item"
            :class="{
              'is-current': item.index === player.index,
              'is-played': item.index < player.index,
            }"
            :data-queue-current="item.index === player.index ? '1' : null"
            @click="jumpTo(item.index)"
          >
            <div
              class="cover"
              style="width: 30px; height: 30px; border-radius: 7px"
            >
              <img
                loading="lazy"
                decoding="async"
                v-if="item.track.cover_url"
                :src="item.track.cover_url"
              />
              <Icon v-else name="note" :size="14" class="faint" />
            </div>
            <div class="col" style="min-width: 0">
              <div class="ellipsis t-13">{{ item.track.title }}</div>
              <div class="faint t-11 ellipsis">
                <ArtistsLine :artists="item.track.artists" :limit="2" />
              </div>
            </div>
            <Icon
              v-if="item.index === player.index"
              :name="player.isPlaying ? 'pause' : 'play'"
              :size="13"
              class="queue-state"
            />
            <TrackMenu :context-menu="true" :track="item.track" />
          </div>

          <div v-if="!queueItems.length" class="faint t-12 q-px-md q-py-sm">
            Очередь пуста
          </div>
        </q-scroll-area>
      </div>
    </q-drawer>

    <q-page-container
      v-if="!panels.hidden && !panels.mini"
      style="position: relative"
    >
      <router-view v-slot="{ Component, route: current }">
        <Transition name="page" mode="out-in">
          <component :is="Component" :key="current.fullPath" />
        </Transition>
      </router-view>
      <LyricsPanel
        v-if="lyricsVisible"
        :style="player.lyricsFullscreen ? undefined : lyricsBox"
      />
    </q-page-container>

    <div v-if="panels.hidden" class="tray-idle">Играет в фоновом режиме</div>

    <q-footer
      v-if="!panels.hidden && !panels.mini"
      style="background: var(--bg); border-top: 1px solid var(--line)"
    >
      <PlayerBar />
    </q-footer>

    <MiniPlayer v-if="panels.mini && !panels.hidden" />

    <FullscreenPlayer />
    <TrackLyricsDialog />
    <AppDialog />
  </q-layout>
</template>

<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import { useRoute, useRouter } from "vue-router";
import ArtistsLine from "@/components/ArtistsLine.vue";
import TrackMenu from "@/components/TrackMenu.vue";
import Icon from "@/components/Icon.vue";
import FullscreenPlayer from "@/components/FullscreenPlayer.vue";
import LyricsPanel from "@/components/LyricsPanel.vue";
import MiniPlayer from "@/components/MiniPlayer.vue";
import PlayerBar from "@/components/PlayerBar.vue";
import TrackLyricsDialog from "@/components/TrackLyricsDialog.vue";
import AppDialog from "@/components/AppDialog.vue";
import WindowControls from "@/components/WindowControls.vue";
import { trackLyricsDialog } from "@/lib/dialogs";
import {
  bindGlobalHotkeyEvents,
  bindLocalHotkeys,
  unbindGlobalHotkeyEvents,
  unbindLocalHotkeys,
} from "@/lib/hotkeys/runtime";
import { bindTrayEvents, setCloseToTray } from "@/lib/tray";
import {
  releaseStuckPointer,
  startWindowDrag,
  toggleWindowMaximize,
} from "@/lib/window";
import { useAuthStore } from "@/stores/auth";
import { useLibraryStore } from "@/stores/library";
import { useHotkeysStore } from "@/stores/hotkeys";
import { usePanelsStore } from "@/stores/panels";
import { usePlayerStore } from "@/stores/player/index";
import { useUiStore } from "@/stores/ui/index";

const route = useRoute();
const router = useRouter();
const auth = useAuthStore();
const player = usePlayerStore();
const library = useLibraryStore();
const ui = useUiStore();
const panels = usePanelsStore();
const hotkeys = useHotkeysStore();

const avatarError = ref(false);
watch(
  () => auth.profile?.avatar_url,
  () => {
    avatarError.value = false;
  },
);
const avatarUrl = computed(() =>
  avatarError.value ? null : auth.profile?.avatar_url || null,
);
const accountName = computed(
  () => auth.profile?.display_name || auth.profile?.login || "Профиль",
);
const accountInitial = computed(() => {
  const src = (auth.profile?.display_name || auth.profile?.login || "").trim();
  return src ? src.charAt(0).toUpperCase() : "☺";
});
const accountLogin = computed(() => {
  const login = auth.profile?.login;
  if (!login || login === auth.profile?.display_name) return "";
  return "@" + login;
});
const accountSub = computed(() =>
  auth.profile?.has_plus
    ? "Яндекс Плюс"
    : accountLogin.value || "Яндекс Музыка",
);

const query = ref("");
const rightPanel = computed({
  get: () => panels.queueOpen,
  set: (value: boolean) => {
    panels.queueOpen = value;
  },
});

const lyricsVisible = computed(
  () =>
    player.showLyrics &&
    !!player.current &&
    !route.path.startsWith("/settings"),
);

const lyricsBox = computed(() => ({
  top: "40px",
  bottom: "77px",
  left: `${ui.settings.sidebarWidth}px`,
  right: rightPanel.value ? "300px" : "0px",
}));

const nav = [
  { to: "/", label: "Главное", icon: "home" },
  { to: "/wave", label: "Моя во��на", icon: "wave" },
  { to: "/search", label: "Поиск", icon: "search" },
  { to: "/liked", label: "Мне нравится", icon: "heart" },
  { to: "/playlists", label: "Коллекция", icon: "library" },
  { to: "/library", label: "Поиск в библиотеке", icon: "filter" },
  { to: "/stats", label: "Статистика", icon: "stats" },
];

type ScrollAreaRef = {
  getScrollTarget: () => HTMLElement;
  setScrollPosition: (
    axis: "vertical" | "horizontal",
    offset: number,
    duration?: number,
  ) => void;
} | null;

const queueScroll = ref<ScrollAreaRef>(null);
const followCurrent = ref(true);
let autoScrolling = false;

/** Очередь целиком: и сыгранные треки, и всё, что впереди. */
const queueItems = computed(() =>
  (player.queue ?? []).map((track, index) => ({ track, index })),
);

function currentEl(): { target: HTMLElement; el: HTMLElement } | null {
  const area = queueScroll.value;
  if (!area?.getScrollTarget) return null;
  const target = area.getScrollTarget();
  const el = target?.querySelector<HTMLElement>('[data-queue-current="1"]');
  return target && el ? { target, el } : null;
}

/** Центрирует играющий трек в видимой области очереди. */
async function scrollToCurrent(duration = 0) {
  await nextTick();
  const found = currentEl();
  const area = queueScroll.value;
  if (!found || !area) return;

  const { target, el } = found;
  const offset = el.offsetTop - target.clientHeight / 2 + el.clientHeight / 2;

  autoScrolling = true;
  area.setScrollPosition("vertical", Math.max(0, offset), duration);
  window.setTimeout(() => {
    autoScrolling = false;
  }, duration + 60);
}

function isCurrentVisible(): boolean {
  const found = currentEl();
  if (!found) return false;
  const { target, el } = found;
  const top = el.offsetTop - target.scrollTop;
  return top > -el.clientHeight && top < target.clientHeight;
}

function onQueueScroll() {
  if (autoScrolling) return;
  followCurrent.value = isCurrentVisible();
}

function locateCurrent() {
  followCurrent.value = true;
  void scrollToCurrent(220);
}

// При открытии панели сразу показываем текущий трек.
watch(
  () => panels.queueOpen,
  (open) => {
    if (!open) return;
    followCurrent.value = true;
    void scrollToCurrent(0);
  },
);

// При смене трека догоняем его, если пользователь не ушёл листать список.
watch(
  () => player.index,
  () => {
    if (!panels.queueOpen || !followCurrent.value) return;
    void scrollToCurrent(260);
  },
);

function jumpTo(index: number) {
  player.index = index;
  followCurrent.value = true;
  void player.loadCurrent();
}

function submitSearch() {
  if (!query.value.trim()) return;
  void router.push({ name: "search", query: { q: query.value.trim() } });
}

async function doLogout() {
  await auth.logout();
  void router.replace("/login");
}

const hotkeyHandlers = {
  toggle: () => player.toggle(),
  mute: () => player.toggleMute(),
  seekForward: () =>
    player.seek(Math.min(player.duration || 0, player.progress + 5)),
  seekBackward: () => player.seek(Math.max(0, player.progress - 5)),
  volumeUp: () =>
    player.setVolume(Math.round(Math.min(1, player.volume + 0.05) * 100) / 100),
  volumeDown: () =>
    player.setVolume(Math.round(Math.max(0, player.volume - 0.05) * 100) / 100),
  like: () => {
    void player.like();
  },
  dislike: () => {
    void player.dislike();
  },
  repeat: () => player.cycleRepeat(),
  shuffle: () => player.toggleShuffle(),
  next: () => {
    void player.next(false);
  },
  prev: () => {
    void player.prev();
  },
  lyrics: () => player.toggleLyrics(),
  queue: () => panels.toggleQueue(),
};

onMounted(() => {
  hotkeys.load();
  bindLocalHotkeys((signature) => hotkeys.resolve(signature), hotkeyHandlers);
  void bindGlobalHotkeyEvents(hotkeyHandlers);
  void hotkeys.applyGlobal();
  void library.init();
  void setCloseToTray(ui.settings.minimizeToTray);
  if (ui.settings.resumeLastSession) {
    void player.restoreSession(ui.settings.resumeAutoplay);
  }
  void bindTrayEvents({
    onToggle: () => player.toggle(),
    onNext: () => {
      void player.next(false);
    },
    onPrev: () => {
      void player.prev();
    },
    onLike: () => {
      void player.like();
    },
    onHidden: () => {
      panels.queueOpen = false;
      panels.hidden = true;
      player.releaseMemory();
    },
    onShown: () => {
      panels.hidden = false;
    },
  });
});

watch(
  () => ui.settings,
  () => {
    ui.apply();
    void player.syncPresence();
    void setCloseToTray(ui.settings.minimizeToTray);
  },
  { deep: true, immediate: true },
);

function onWindowFocus() {
  releaseStuckPointer();
}

watch(
  () => route.fullPath,
  () => {
    player.closeLyrics();
    trackLyricsDialog.open = false;
    document.documentElement.dataset.route = route.path.startsWith("/settings")
      ? "settings"
      : route.path.replace(/^\//, "").split("/")[0] || "home";
  },
  { immediate: true },
);

window.addEventListener("focus", onWindowFocus);

onBeforeUnmount(() => {
  window.removeEventListener("focus", onWindowFocus);
  unbindLocalHotkeys();
  unbindGlobalHotkeyEvents();
});
</script>

<style scoped>
.queue-head {
  display: flex;
  align-items: center;
  gap: 8px;
}
.queue-pos {
  flex: 0 0 auto;
  font-variant-numeric: tabular-nums;
}
.queue-locate {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  width: 22px;
  height: 22px;
  padding: 0;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--accent, #fa2d48);
  cursor: pointer;
  transition:
    background 0.14s ease,
    color 0.14s ease,
    opacity 0.14s ease;
}
.queue-locate:hover {
  background: var(--hover);
}
.queue-locate.off {
  color: var(--fg-dim);
  opacity: 0.5;
}
</style>
