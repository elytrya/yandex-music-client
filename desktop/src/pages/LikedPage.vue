<template>
  <q-page class="liked-page">
    <div class="liked-page-header">
      <div class="row items-end no-wrap" style="gap: 26px">
        <div
          class="cover"
          style="
            width: 132px;
            height: 132px;
            border-radius: 16px;
            background: var(--surface-2);
          "
        >
          <Icon name="heartFilled" :size="40" />
        </div>
        <div class="col">
          <div
            class="dim t-12"
            style="letter-spacing: 0.08em; text-transform: uppercase"
          >
            Плейлист
          </div>
          <div class="h1 q-mt-xs">Мне нравится</div>
          <div class="faint t-13 q-mt-sm">
            {{ plural(tracks.length, "трек", "трека", "треков") }}
          </div>

          <div class="row items-center q-mt-md" style="gap: 10px">
            <div class="btn-solid" @click="playFrom(0)">
              <Icon name="play" :size="15" />
              <span>Слушать</span>
            </div>
            <div class="btn" @click="shufflePlay">
              <Icon name="shuffle" :size="15" />
              <span>Перемешать</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="loading" class="playlist-loading">
      <q-spinner size="24px" color="primary" />
    </div>

    <div v-else class="playlist-tracks-scroll">
      <div class="head-row liked-head-row">
        <div style="width: 20px; text-align: center">#</div>
        <div style="width: 38px" />
        <div class="col">Название</div>
        <div class="gt-sm" style="width: 190px">Альбом</div>
        <div style="width: 40px; text-align: right">Время</div>
      </div>

      <q-virtual-scroll
        v-if="tracks.length"
        v-slot="{ item, index }"
        class="track-virtual"
        :items="tracks"
        :virtual-scroll-item-size="52"
        :virtual-scroll-slice-size="20"
      >
        <TrackRow
          :key="`${item.id}-${index}`"
          :track="item"
          :index="index"
          @play="playFrom(index)"
        />
      </q-virtual-scroll>

      <div v-else class="dim t-13 q-py-xl">Здесь пока ничего нет</div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import Icon from "@/components/Icon.vue";
import { onMounted, ref } from "vue";
import TrackRow from "@/components/TrackRow.vue";
import { api } from "@/api/client";
import type { Track } from "@/api/types";
import { plural } from "@/lib/format";
import { readCache, swr } from "@/lib/cache";
import { usePlayerStore } from "@/stores/player/index";

const player = usePlayerStore();
const tracks = ref<Track[]>([]);
const loading = ref(false);

function playFrom(i: number) {
  if (tracks.value.length) void player.playQueue(tracks.value, i);
}

function shufflePlay() {
  if (!tracks.value.length) return;
  const start = Math.floor(Math.random() * tracks.value.length);
  void player.playQueue(tracks.value, start).then(() => {
    if (!player.shuffle) player.toggleShuffle();
  });
}

onMounted(() => {
  tracks.value = readCache<Track[]>("liked.tracks") ?? [];
  loading.value = !tracks.value.length;

  void swr<Track[]>("liked.tracks", () => api.likedTracks(), {
    onData: (data) => {
      tracks.value = data;
    },
    onSettled: () => {
      loading.value = false;
    },
  });
});
</script>
