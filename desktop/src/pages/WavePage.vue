<template>
  <q-page class="wave-page">
    <div class="wave-scroll">
      <div class="wave-content">
        <header class="wave-top">
          <div class="wave-top-info">
            <span class="wave-top-eyebrow">{{ stationTitle }}</span>
            <h1>{{ hasCurrent ? currentTitle : "Моя волна" }}</h1>
            <p v-if="hasCurrent" class="wave-top-artists">
              <ArtistsLine
                :artists="player.current?.artists ?? []"
                :limit="3"
              />
            </p>
            <p v-else>{{ heroSubtitle }}</p>
          </div>
          <TrackMenu
            v-if="player.current"
            :context-menu="true"
            :track="player.current"
          />
          <div class="wave-top-controls">
            <button
              v-if="hasCurrent"
              type="button"
              class="wave-ic"
              title="Не нравится"
              @click="player.dislike()"
            >
              <Icon name="heartOff" :size="17" />
            </button>
            <button
              type="button"
              class="wave-ic wave-ic-play"
              :title="primaryLabel"
              @click="onPrimary"
            >
              <q-spinner v-if="player.loading" size="16px" />
              <Icon v-else :name="primaryIcon" :size="18" />
            </button>
            <button
              v-if="hasCurrent"
              type="button"
              class="wave-ic"
              title="Нравится"
              @click="player.like()"
            >
              <Icon name="heart" :size="17" />
            </button>
            <button
              v-if="hasCurrent"
              type="button"
              class="wave-ic"
              title="Следующий"
              @click="player.next(false)"
            >
              <Icon name="next" :size="17" />
            </button>
          </div>
        </header>

        <p v-if="errorText" class="wave-notice">
          {{ errorText }}
          <button type="button" @click="refresh">Обновить</button>
        </p>

        <nav class="wave-chips">
          <button
            v-for="item in stations"
            :key="item.id"
            type="button"
            class="wave-chip"
            :class="{ on: isActiveStation(item) }"
            @click="playStation(item)"
          >
            {{ item.name }}
          </button>
        </nav>

        <section v-if="settingsGroups.length" class="wave-settings">
          <div
            v-for="group in settingsGroups"
            :key="group.key"
            class="wave-settings-row"
          >
            <span class="wave-settings-label">{{ group.label }}</span>
            <div class="settings-choice">
              <button
                v-for="option in group.options"
                :key="option.value"
                type="button"
                :class="{ on: group.current === option.value }"
                :disabled="applying"
                @click="applySetting(group.key, option.value)"
              >
                {{ option.name }}
              </button>
            </div>
          </div>
        </section>

        <section v-if="queueWindow.length" class="wave-list">
          <h3>Очередь</h3>
          <div
            v-for="item in queueWindow"
            :key="`${item.track.id}-${item.index}`"
            class="wave-item"
            :class="{
              'is-current': item.index === player.index,
              'is-played': item.index < player.index,
            }"
            @click="jumpTo(item.index)"
          >
            <span class="wave-item-art">
              <img
                v-if="item.track.cover_url"
                :src="item.track.cover_url"
                alt=""
                loading="lazy"
                decoding="async"
              />
              <Icon v-else name="note" :size="14" class="faint" />
            </span>
            <span class="wave-item-copy">
              <span class="wave-item-title">{{ item.track.title }}</span>
              <span class="wave-item-sub">
                <ArtistsLine :artists="item.track.artists" :limit="2" />
              </span>
            </span>
            <Icon
              v-if="item.index === player.index"
              :name="player.isPlaying ? 'pause' : 'play'"
              :size="13"
              class="wave-item-state"
            />
            <TrackMenu :context-menu="true" :track="item.track" />
          </div>
        </section>

        <p v-else-if="!hasCurrent" class="wave-empty">
          Запусти волну - подберём музыку под твоё настроение.
        </p>
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api } from "@/api/client";
import { Notify } from "quasar";
import type { StationInfo, Track, WheelItem } from "@/api/types";
import Icon from "@/components/Icon.vue";
import ArtistsLine from "@/components/ArtistsLine.vue";
import TrackMenu from "@/components/TrackMenu.vue";
import { readCache, swr } from "@/lib/cache";
import { usePlayerStore } from "@/stores/player/index";

const DEFAULT_STATION = "user:onyourwave";

function station(id: string, name: string): WheelItem {
  return {
    id,
    kind: "wave",
    name,
    description: null,
    cover_url: null,
    color: null,
    station: id,
    artists: [],
  };
}

const FALLBACK_STATIONS: WheelItem[] = [
  station(DEFAULT_STATION, "Моя волна"),
  station("personal:collection", "Коллекция"),
  station("mood:all", "По настроению"),
  station("genre:all", "По жанру"),
  station("activity:all", "Под занятие"),
];

const player = usePlayerStore();

const items = ref<WheelItem[]>([]);
const loadError = ref<string | null>(null);
const info = ref<StationInfo | null>(null);
const applying = ref(false);

type SettingKey = "moodEnergy" | "diversity" | "language";

const settingsGroups = computed(() => {
  const data = info.value;
  if (!data) return [];
  const groups: Array<{
    key: SettingKey;
    label: string;
    current: string | null;
    options: Array<{ value: string; name: string }>;
  }> = [
    {
      key: "moodEnergy",
      label: "Настроение",
      current: data.mood_energy,
      options: data.moods,
    },
    {
      key: "diversity",
      label: "Разнообразие",
      current: data.diversity,
      options: data.diversities,
    },
    {
      key: "language",
      label: "Язык",
      current: data.language,
      options: data.languages,
    },
  ];
  return groups.filter((group) => group.options.length > 1);
});

async function loadInfo() {
  try {
    info.value = await api.stationInfo(player.stationId ?? DEFAULT_STATION);
  } catch {
    info.value = null;
  }
}

async function applySetting(key: SettingKey, value: string) {
  if (applying.value) return;
  applying.value = true;
  try {
    await api.setStationSettings({
      station: player.stationId ?? DEFAULT_STATION,
      [key]: value,
    });
    await loadInfo();
    await player.startWave(
      player.stationId ?? DEFAULT_STATION,
      player.stationName || undefined,
    );
  } catch {
    Notify.create({ message: "Не удалось обновить настройки волны" });
  } finally {
    applying.value = false;
  }
}

const stations = computed(() => {
  const wheel = items.value.filter((item) => item?.kind === "wave");
  const list = wheel.length ? wheel : FALLBACK_STATIONS;
  return list.some((item) => stationIdOf(item) === DEFAULT_STATION)
    ? list
    : [FALLBACK_STATIONS[0] as WheelItem, ...list];
});

const hasCurrent = computed(() => Boolean(player.current));
const stationTitle = computed(() => player.stationName || "Моя волна");
const errorText = computed(() => player.waveError || loadError.value);

const currentTitle = computed(() => player.current?.title ?? "");

const heroSubtitle = computed(() =>
  hasCurrent.value
    ? "Бесконечный поток, который подстраивается под тебя"
    : "Выбери станцию или просто нажми «Запустить»",
);

const primaryIcon = computed(() =>
  hasCurrent.value && player.isPlaying ? "pause" : "play",
);

const primaryLabel = computed(() => {
  if (!hasCurrent.value) return "Запустить";
  return player.isPlaying ? "Пауза" : "Продолжить";
});

const queueWindow = computed(() => {
  const queue = player.queue ?? [];
  const idx = player.index;
  const start = Math.max(0, idx - 10);
  const end = Math.min(queue.length, idx + 13);
  const out: Array<{ track: Track; index: number }> = [];
  for (let i = start; i < end; i++) {
    const track = queue[i];
    if (track) out.push({ track, index: i });
  }
  return out;
});

function stationIdOf(item: WheelItem | undefined): string {
  if (!item) return DEFAULT_STATION;
  return item.station || item.id || DEFAULT_STATION;
}

function isActiveStation(item: WheelItem): boolean {
  return stationIdOf(item) === (player.stationId ?? DEFAULT_STATION);
}

function onPrimary() {
  if (!hasCurrent.value) void player.startWave();
  else player.toggle();
}

function playStation(item: WheelItem) {
  void player.startWave(stationIdOf(item), item.name);
}

function jumpTo(index: number) {
  const queue = player.queue ?? [];
  if (index < 0 || index >= queue.length) return;
  player.index = index;
  void player.loadCurrent();
}

async function load() {
  const cached = readCache<WheelItem[]>("wave.wheel");
  if (cached?.length) items.value = cached;
  await swr<WheelItem[]>("wave.wheel", () => api.wheel(), {
    onData: (data) => {
      if (Array.isArray(data)) items.value = data;
      loadError.value = null;
    },
    onError: () => {
      if (!items.value.length) {
        loadError.value = "Не удалось загрузить список станций";
      }
    },
  });
}

function refresh() {
  loadError.value = null;
  void load();
}

onMounted(() => {
  void loadInfo();
  void load();
});
</script>
