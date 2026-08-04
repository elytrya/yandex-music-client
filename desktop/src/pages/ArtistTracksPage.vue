<template>
  <q-page class="scroll-page">
    <q-scroll-area class="scroll-page-area">
      <div class="scroll-page-inner">
        <div class="section-head">
          <div>
            <button class="more-link" type="button" @click="back">
              <Icon name="chevronLeft" :size="14" />
              <span>{{ name || "Артист" }}</span>
            </button>
            <div class="h1 q-mt-sm">Все треки</div>
            <div class="faint t-12 q-mt-xs">
              {{ plural(tracks.length, "трек", "трека", "треков") }}
            </div>
          </div>

          <div class="head-actions">
            <button
              class="btn-solid"
              type="button"
              :disabled="!tracks.length"
              @click="playFrom(0)"
            >
              <Icon name="play" :size="15" />
              <span>Слушать</span>
            </button>

            <button
              class="btn"
              type="button"
              :disabled="!tracks.length"
              @click="shufflePlay"
            >
              <Icon name="shuffle" :size="15" />
              <span>Перемешать</span>
            </button>

            <button
              class="btn"
              type="button"
              :disabled="!tracks.length"
              @click="enqueueAll"
            >
              <Icon name="addQueue" :size="15" />
              <span>В очередь</span>
            </button>
          </div>
        </div>

        <div v-if="loading" class="flex flex-center" style="height: 260px">
          <q-spinner size="26px" color="primary" />
        </div>

        <div v-else-if="error" class="dim t-13">{{ error }}</div>

        <template v-else>
          <div class="head-row">
            <div style="width: 20px; text-align: center">#</div>
            <div style="width: 38px" />
            <div class="col">Название</div>
            <div class="gt-sm" style="width: 190px">Альбом</div>
            <div style="width: 84px" />
            <div style="width: 40px; text-align: right">Время</div>
          </div>

          <LazyTracks v-slot="{ item, index }" :items="tracks">
            <TrackRow
              :track="item"
              :index="index"
              :show-album="true"
              @play="playFrom(index)"
            />
          </LazyTracks>
        </template>
      </div>
    </q-scroll-area>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { Notify } from "quasar";
import { api } from "@/api/client";
import type { ArtistPage, Track } from "@/api/types";
import Icon from "@/components/Icon.vue";
import LazyTracks from "@/components/LazyTracks.vue";
import TrackRow from "@/components/TrackRow.vue";
import { plural } from "@/lib/format";
import { readCache, swr } from "@/lib/cache";
import { usePlayerStore } from "@/stores/player/index";

const props = defineProps<{ id: string }>();
const router = useRouter();
const player = usePlayerStore();
const tracks = ref<Track[]>([]);
const name = ref("");
const loading = ref(false);
const error = ref("");

function back() {
  void router.push(`/artist/${props.id}`);
}

function playFrom(index: number) {
  if (tracks.value.length) void player.playQueue(tracks.value, index);
}

function shufflePlay() {
  if (!tracks.value.length) return;
  void player.playQueue(tracks.value, 0).then(() => {
    if (!player.shuffle) player.toggleShuffle();
  });
}

function enqueueAll() {
  if (!tracks.value.length) return;
  tracks.value.forEach((track) => player.enqueue(track));
  Notify.create({ message: `В очередь добавлено: ${tracks.value.length}` });
}

async function load() {
  error.value = "";
  name.value = readCache<ArtistPage>(`artist.${props.id}`)?.name ?? "";
  const key = `artist.tracks.${props.id}`;
  const cached = readCache<Track[]>(key) ?? [];
  tracks.value = cached;
  loading.value = !cached.length;

  await swr<Track[]>(key, () => api.artistTracks(props.id), {
    onData: (list) => {
      tracks.value = list;
      if (!list.length) error.value = "У артиста не нашлось доступных треков";
    },
    onSettled: () => {
      loading.value = false;
    },
  });
}

watch(() => props.id, load);
onMounted(load);
</script>
