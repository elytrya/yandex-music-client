<template>
  <q-dialog v-model="open">
    <div class="text-card">
      <div class="text-card-head">
        <div class="col" style="min-width: 0">
          <div class="text-card-title ellipsis">{{ title }}</div>
          <div class="text-card-sub ellipsis">{{ subtitle }}</div>
        </div>
        <div v-if="genius.ready" class="text-card-source">
          <button
            type="button"
            :class="{ on: source === 'yandex' }"
            @click="source = 'yandex'"
          >
            Яндекс
          </button>
          <button
            type="button"
            :class="{ on: source === 'genius' }"
            @click="source = 'genius'"
          >
            Genius
          </button>
        </div>
        <div class="icon-btn round" @click="open = false">
          <Icon name="close" :size="16" />
        </div>
      </div>

      <GeniusPanel
        v-if="source === 'genius'"
        :track="track"
        :active="open && source === 'genius'"
      />

      <template v-else>
        <div v-if="blocks.length" class="text-card-body">
          <div class="text-card-inner">
            <p v-for="(block, i) in blocks" :key="i" class="text-card-block">
              <template v-for="(line, j) in block" :key="j">
                {{ line }}<br v-if="j < block.length - 1" />
              </template>
            </p>
          </div>
          <div v-if="loading" class="text-card-note">
            <q-spinner size="12px" color="grey-6" />
            <span>Обновляю текст…</span>
          </div>
        </div>

        <div v-else-if="loading" class="text-card-empty">
          <q-spinner size="20px" color="grey-6" />
          <span>Ищу текст…</span>
        </div>

        <div v-else class="text-card-empty">
          <span>{{ error || "Текста для этого трека нет" }}</span>
          <div class="btn" @click="retry">Попробовать снова</div>
          <div v-if="genius.ready" class="btn" @click="source = 'genius'">
            Посмотреть на Genius
          </div>
        </div>

        <div v-if="writers" class="text-card-foot ellipsis">
          Авторы: {{ writers }}
        </div>
      </template>
    </div>
  </q-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Icon from "@/components/Icon.vue";
import GeniusPanel from "@/components/GeniusPanel.vue";
import { api } from "@/api/client";
import type { Lyrics } from "@/api/types";
import { readCache, writeCache } from "@/lib/cache";
import { trackLyricsDialog } from "@/lib/dialogs";
import { artistNames } from "@/lib/format";
import { useGeniusStore } from "@/stores/genius";

const loading = ref(false);
const error = ref("");
const lyrics = ref<Lyrics | null>(null);
const source = ref<"yandex" | "genius">("yandex");

const genius = useGeniusStore();

const open = computed({
  get: () => trackLyricsDialog.open,
  set: (value: boolean) => {
    trackLyricsDialog.open = value;
  },
});

const track = computed(() => trackLyricsDialog.track);
const title = computed(() => track.value?.title || "Текст песни");
const subtitle = computed(() =>
  track.value ? artistNames(track.value.artists) : "",
);
const writers = computed(() => (lyrics.value?.writers || []).join(", "));

function toBlocks(value: Lyrics | null): string[][] {
  const lines = (value?.lines || []).map((line) => (line.text || "").trim());
  const groups: string[][] = [];
  let current: string[] = [];
  for (const line of lines) {
    if (!line) {
      if (current.length) groups.push(current);
      current = [];
      continue;
    }
    current.push(line);
  }
  if (current.length) groups.push(current);
  return groups;
}

const blocks = computed(() => toBlocks(lyrics.value));

async function load(id: string, force = false) {
  const key = `lyrics.${id}`;
  const cached = force ? null : readCache<Lyrics>(key);
  const cachedOk = !!cached && toBlocks(cached).length > 0;

  lyrics.value = cachedOk ? cached : null;
  loading.value = true;
  error.value = "";

  try {
    const fresh = await api.lyrics(id);
    if (trackLyricsDialog.track?.id !== id) return;
    if (toBlocks(fresh).length) {
      lyrics.value = fresh;
      writeCache(key, fresh);
    } else if (!cachedOk) {
      error.value = "Текста для этого трека нет";
    }
  } catch (e) {
    if (trackLyricsDialog.track?.id !== id) return;
    if (!cachedOk) {
      const message = e instanceof Error ? e.message : String(e);
      error.value = message || "Не удалось загрузить текст";
    }
  } finally {
    if (trackLyricsDialog.track?.id === id) {
      loading.value = false;
      if (
        genius.ready &&
        genius.lyricsMode !== "off" &&
        !toBlocks(lyrics.value).length
      ) {
        source.value = "genius";
      }
    }
  }
}

function retry() {
  const id = track.value?.id;
  if (id) void load(String(id), true);
}

watch(
  () => [trackLyricsDialog.open, trackLyricsDialog.track?.id] as const,
  ([isOpen, id]) => {
    if (!isOpen || !id) return;
    source.value =
      genius.ready && genius.lyricsMode === "always" ? "genius" : "yandex";
    void load(String(id));
  },
  { immediate: true },
);
</script>

<style scoped>
.text-card-source {
  display: flex;
  gap: 2px;
  margin-right: 4px;
  padding: 2px;
  border: 1px solid var(--line);
  border-radius: 999px;
}
.text-card-source button {
  padding: 3px 10px;
  border: 0;
  border-radius: 999px;
  background: none;
  color: var(--fg-dim);
  font-size: 11px;
  cursor: pointer;
}
.text-card-source button.on {
  background: var(--line);
  color: var(--fg);
}
</style>
