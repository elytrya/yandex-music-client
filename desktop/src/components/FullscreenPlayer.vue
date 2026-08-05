<template>
  <Transition name="fs" :duration="{ enter: 280, leave: 320 }">
    <div
      v-if="player.fullscreen && player.current"
      class="fs-player"
      :class="{ 'with-queue': showQueue }"
    >
      <div
        v-if="coverUrl"
        class="fs-backdrop"
        :style="{ backgroundImage: `url(${coverUrl})` }"
      />
      <div class="fs-shade" />

      <div class="fs-top">
        <div class="fs-drag" @mousedown="onDrag" @dblclick="toggleNative" />
        <div class="fs-top-actions">
          <button class="icon-btn round" @click="openLyrics">
            <Icon name="lyrics" :size="18" />
            <q-tooltip>Текст трека</q-tooltip>
          </button>
          <button
            class="icon-btn round"
            :class="{ on: showQueue }"
            @click="showQueue = !showQueue"
          >
            <Icon name="queue" :size="18" />
            <q-tooltip>Очередь</q-tooltip>
          </button>
          <button class="icon-btn round" @click="toggleNative">
            <Icon :name="native ? 'restore' : 'maximize'" :size="16" />
            <q-tooltip>
              {{ native ? "Оконный режим" : "Весь экран" }}
            </q-tooltip>
          </button>
          <button class="icon-btn round" @click="close">
            <Icon name="close" :size="19" />
            <q-tooltip>Закрыть (Esc)</q-tooltip>
          </button>
        </div>
      </div>

      <div class="fs-body">
        <div class="fs-stage">
          <div class="fs-art">
            <Transition name="fs-swap">
              <img
                v-if="coverUrl"
                :key="coverUrl"
                :src="coverUrl"
                decoding="async"
                alt=""
                @click="openLyrics"
              />
              <Icon v-else name="note" :size="72" class="faint" />
            </Transition>

            <div class="fs-art-hover">
              <div class="fs-art-row">
                <button
                  class="fs-ctl"
                  :disabled="!player.hasPrev"
                  @click.stop="player.prev()"
                >
                  <Icon name="prev" :size="22" />
                  <q-tooltip>Предыдущий</q-tooltip>
                </button>
                <button
                  class="fs-ctl fs-ctl-main"
                  @click.stop="player.toggle()"
                >
                  <Icon
                    :name="player.isPlaying ? 'pause' : 'play'"
                    :size="28"
                  />
                  <q-tooltip>
                    {{ player.isPlaying ? "Пауза" : "Играть" }}
                  </q-tooltip>
                </button>
                <button
                  class="fs-ctl"
                  :disabled="!player.hasNext"
                  @click.stop="player.next()"
                >
                  <Icon name="next" :size="22" />
                  <q-tooltip>Следующий</q-tooltip>
                </button>
              </div>
              <div class="fs-art-row fs-art-row-small">
                <button
                  class="fs-ctl"
                  :class="{ on: liked }"
                  @click.stop="like"
                >
                  <Icon :name="liked ? 'heartFilled' : 'heart'" :size="19" />
                  <q-tooltip>
                    {{ liked ? "Убрать лайк" : "Мне нравится" }}
                  </q-tooltip>
                </button>
                <button
                  class="fs-ctl"
                  :class="{ on: showQueue }"
                  @click.stop="showQueue = !showQueue"
                >
                  <Icon name="queue" :size="19" />
                  <q-tooltip>Очередь</q-tooltip>
                </button>
                <button ref="dotsButton" class="fs-ctl" @click.stop>
                  <Icon name="more" :size="19" />
                  <TrackMenu :track="player.current" />
                </button>
              </div>
            </div>
          </div>

          <Transition name="fs-swap" mode="out-in">
            <div :key="player.current.id" class="fs-meta">
              <div class="fs-title ellipsis">{{ player.current.title }}</div>
              <div class="fs-artists ellipsis">
                <ArtistsLine :artists="player.current.artists" :limit="3" />
              </div>
            </div>
          </Transition>

          <div class="fs-progress">
            <span class="t-11 faint">{{ formatDuration(player.progress * 1000) }}</span>
            <q-slider
              dense
              color="red"
              :model-value="player.progress"
              :min="0"
              :max="Math.max(player.duration, 1)"
              :step="1"
              @update:model-value="(v) => player.seek(Number(v ?? 0))"
            />
            <span class="t-11 faint">{{ formatDuration(player.duration * 1000) }}</span>
          </div>
        </div>

        <Transition name="fs-q">
          <div v-if="showQueue" class="fs-queue">
            <div class="fs-queue-head">
              <span>Очередь</span>
              <button class="icon-btn round" @click="showQueue = false">
                <Icon name="close" :size="16" />
              </button>
            </div>
            <q-scroll-area class="fs-queue-list">
              <div
                v-for="item in queueWindow"
                :key="`${item.index}-${item.track.id}`"
                class="fs-queue-item"
                :class="{ on: item.index === player.index }"
                @click="jumpTo(item.index)"
              >
                <div class="cover fs-queue-cover">
                  <img
                    v-if="item.track.cover_url"
                    :src="item.track.cover_url"
                    loading="lazy"
                    decoding="async"
                    alt=""
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
                />
              </div>
              <div v-if="!queueWindow.length" class="faint t-12 q-pa-md">
                Очередь пуста
              </div>
            </q-scroll-area>
          </div>
        </Transition>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRoute } from "vue-router";
import ArtistsLine from "@/components/ArtistsLine.vue";
import Icon from "@/components/Icon.vue";
import TrackMenu from "@/components/TrackMenu.vue";
import { formatDuration } from "@/lib/format";
import {
  isNativeFullscreen,
  setNativeFullscreen,
  startWindowDrag,
} from "@/lib/window";
import { useLibraryStore } from "@/stores/library";
import { usePlayerStore } from "@/stores/player/index";

const player = usePlayerStore();
const library = useLibraryStore();
const route = useRoute();

const showQueue = ref(false);
const native = ref(false);
const dotsButton = ref<HTMLElement | null>(null);

const coverUrl = computed(() => player.current?.cover_url || "");
const liked = computed(() =>
  player.current ? library.liked(player.current.id) : false,
);

const HISTORY = 5;
const VISIBLE = 60;

const queueWindow = computed(() => {
  const queue = player.queue ?? [];
  const start = Math.max(0, player.index - HISTORY);
  const end = Math.min(queue.length, player.index + 1 + VISIBLE);
  const out: Array<{ track: (typeof queue)[number]; index: number }> = [];
  for (let i = start; i < end; i++) {
    const track = queue[i];
    if (track) out.push({ track, index: i });
  }
  return out;
});

function jumpTo(index: number) {
  player.index = index;
  void player.loadCurrent();
}

function onDrag(event: MouseEvent) {
  if (event.button !== 0 || native.value) return;
  void startWindowDrag(event);
}

function like() {
  if (player.current) void library.toggleLike(player.current);
}

function openLyrics() {
  void player.openLyricsFullscreen();
}

async function toggleNative() {
  const value = !native.value;
  const ok = await setNativeFullscreen(value);
  native.value = ok ? value : await isNativeFullscreen();
}

function close() {
  const wasNative = native.value;
  showQueue.value = false;
  player.closeFullscreen();
  if (!wasNative) return;
  native.value = false;
  window.setTimeout(() => {
    void setNativeFullscreen(false);
  }, 340);
}

function onKey(event: KeyboardEvent) {
  if (!player.fullscreen) return;
  if (event.key === "Escape") close();
}

watch(
  () => player.fullscreen,
  (open) => {
    if (open) {
      void isNativeFullscreen().then((value) => {
        native.value = value;
      });
      return;
    }
    showQueue.value = false;
    if (native.value) {
      native.value = false;
      void setNativeFullscreen(false);
    }
  },
);

/* любой переход по ссылке закрывает полноэкранный режим */
watch(
  () => route.fullPath,
  () => {
    if (player.fullscreen) close();
  },
);

onMounted(() => {
  window.addEventListener("keydown", onKey);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKey);
});
</script>
