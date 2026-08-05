<template>
  <div v-if="id === 'play'" class="play play-sm" @click="player.toggle()">
    <Transition name="ic-pop" mode="out-in">
      <q-spinner v-if="player.loading" key="load" size="14px" color="dark" />
      <Icon
        v-else
        :key="player.isPlaying ? 'pause' : 'play'"
        :name="player.isPlaying ? 'pause' : 'play'"
        :size="18"
      />
    </Transition>
  </div>

  <div
    v-else-if="id === 'shuffle'"
    class="icon-btn"
    :class="{ on: player.shuffle }"
    @click="player.toggleShuffle()"
  >
    <Icon name="shuffle" :size="18" />
    <q-tooltip>Перемешать</q-tooltip>
  </div>

  <div
    v-else-if="id === 'prev'"
    class="icon-btn"
    :class="{ off: !player.hasPrev }"
    @click="player.prev()"
  >
    <Icon name="prev" :size="19" />
    <q-tooltip>Предыдущий</q-tooltip>
  </div>

  <div v-else-if="id === 'next'" class="icon-btn" @click="player.next(false)">
    <Icon name="next" :size="19" />
    <q-tooltip>Следующий</q-tooltip>
  </div>

  <div
    v-else-if="id === 'repeat'"
    class="icon-btn"
    :class="{ on: player.repeat !== 'off' }"
    @click="player.cycleRepeat()"
  >
    <Icon :name="player.repeat === 'one' ? 'repeatOne' : 'repeat'" :size="18" />
    <q-tooltip>{{ repeatLabel }}</q-tooltip>
  </div>

  <div
    v-else-if="id === 'like'"
    class="icon-btn"
    :class="{ on: isLiked, off: !player.current }"
    @click="player.current && player.like()"
  >
    <Icon :name="isLiked ? 'heartFilled' : 'heart'" :size="18" />
    <q-tooltip>
      {{ isLiked ? "Убрать из «Мне нравится»" : "Нравится" }}
    </q-tooltip>
  </div>

  <div
    v-else-if="id === 'dislike'"
    class="icon-btn"
    :class="{ off: !player.current }"
    @click="player.current && player.dislike()"
  >
    <Icon name="heartOff" :size="18" />
    <q-tooltip>Не нравится</q-tooltip>
  </div>

  <div
    v-else-if="id === 'lyrics'"
    class="icon-btn"
    :class="{ on: player.showLyrics, off: !player.current }"
    @click="player.current && player.toggleLyrics()"
  >
    <Icon name="lyrics" :size="18" />
    <q-tooltip>Текст песни</q-tooltip>
  </div>

  <div
    v-else-if="id === 'queue'"
    class="icon-btn"
    :class="{ on: panels.queueOpen }"
    @click="panels.toggleQueue()"
  >
    <Icon name="queue" :size="18" />
    <q-tooltip>Очередь треков</q-tooltip>
  </div>

  <div v-else-if="id === 'mini'" class="icon-btn" @click="panels.enterMini()">
    <Icon name="mini" :size="18" />
    <q-tooltip>Мини-плеер поверх окон</q-tooltip>
  </div>

  <div
    v-else-if="id === 'sleep'"
    class="icon-btn"
    :class="{ on: sleep.active }"
  >
    <Icon name="clock" :size="18" />
    <q-tooltip>Таймер сна: {{ sleep.label }}</q-tooltip>
    <q-menu class="panel menu" anchor="top middle" self="bottom middle">
      <div class="menu-body" style="min-width: 178px">
        <div class="sleep-head">Таймер сна</div>
        <div
          v-for="m in sleepPresets"
          :key="m"
          class="opt"
          v-close-popup
          @click="sleep.start(m)"
        >
          <span>Через {{ m }} мин</span>
        </div>
        <div class="opt" v-close-popup @click="sleep.startUntilTrackEnd()">
          <span>До конца трека</span>
        </div>
        <div
          v-if="sleep.active"
          class="opt danger"
          v-close-popup
          @click="sleep.cancel()"
        >
          <span>Выключить ({{ sleep.label }})</span>
        </div>
      </div>
    </q-menu>
  </div>

  <div
    v-else-if="id === 'speed'"
    class="icon-btn"
    :class="{ on: player.playbackRate !== 1 }"
  >
    <Icon name="speed" :size="18" />
    <q-tooltip>Скорость</q-tooltip>
    <q-menu class="panel menu" anchor="top middle" self="bottom middle">
      <div class="menu-body" style="min-width: 132px">
        <div
          v-for="r in rates"
          :key="r"
          class="opt"
          :class="{ on: player.playbackRate === r }"
          v-close-popup
          @click="player.setPlaybackRate(r)"
        >
          <span>{{ r }}x</span>
          <Icon v-if="player.playbackRate === r" name="check" :size="15" />
        </div>
      </div>
    </q-menu>
  </div>

  <div v-else-if="id === 'quality'" class="icon-btn">
    <Icon name="quality" :size="18" />
    <q-tooltip>{{ player.qualityLabel || "Качество" }}</q-tooltip>
    <q-menu class="panel menu" anchor="top middle" self="bottom middle">
      <div class="menu-body" style="min-width: 172px">
        <div
          v-for="q in qualities"
          :key="q.value"
          class="opt"
          :class="{ on: player.quality === q.value }"
          v-close-popup
          @click="player.setQuality(q.value)"
        >
          <span>{{ q.label }}</span>
          <Icon v-if="player.quality === q.value" name="check" :size="15" />
        </div>
      </div>
    </q-menu>
  </div>

  <div v-else-if="id === 'volume'" class="player-volume">
    <div class="icon-btn" @click="player.toggleMute()">
      <Icon :name="player.muted ? 'volumeOff' : 'volume'" :size="18" />
      <q-tooltip>{{
        player.muted ? "Включить звук" : "Выключить звук"
      }}</q-tooltip>
    </div>
    <q-slider
      class="player-volume-slider"
      :model-value="player.muted ? 0 : player.volume"
      :min="0"
      :max="1"
      :step="0.01"
      dense
      color="primary"
      @update:model-value="(v) => player.setVolume(Number(v ?? 0))"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import Icon from "@/components/Icon.vue";
import type { Quality } from "@/api/types";
import { useLibraryStore } from "@/stores/library";
import { usePanelsStore } from "@/stores/panels";
import { usePlayerStore } from "@/stores/player/index";
import { useSleepStore } from "@/stores/sleep";
import type { PlayerButtonId } from "@/stores/ui/index";

defineProps<{ id: PlayerButtonId }>();

const player = usePlayerStore();
const library = useLibraryStore();
const panels = usePanelsStore();
const sleep = useSleepStore();

const sleepPresets = [15, 30, 45, 60];
const rates = [0.75, 1, 1.25, 1.5, 1.75, 2];

const qualities: Array<{ value: Quality; label: string }> = [
  { value: "low", label: "Экономное" },
  { value: "normal", label: "Стандартное" },
  { value: "high", label: "Высокое" },
  { value: "lossless", label: "Максимальное" },
];

const isLiked = computed(() =>
  player.current ? library.liked(player.current.id) : false,
);

const repeatLabel = computed(() => {
  if (player.repeat === "one") return "Повтор трека";
  if (player.repeat === "all") return "Повтор плейлиста";
  return "Повтор выключен";
});
</script>
