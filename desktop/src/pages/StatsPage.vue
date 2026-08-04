<template>
  <div class="stats-page">
    <div class="stats-scroll">
      <div class="stats-content">
        <div class="stats-topbar">
          <div class="stats-topbar-text">
            <div class="h1">Статистика</div>
            <p class="faint t-13 stats-subtitle">
              {{ periodLabel.toLowerCase() }} · история хранится только на этом
              компьютере
            </p>
          </div>

          <div class="stats-topbar-actions">
            <div class="settings-choice">
              <button
                v-for="p in periods"
                :key="p.days"
                :class="{ on: days === p.days }"
                @click="days = p.days"
              >
                {{ p.label }}
              </button>
            </div>
            <button class="btn stats-clear" type="button" @click="clear">
              <Icon name="trash" :size="14" />
              <span>Очистить</span>
            </button>
          </div>
        </div>

        <div class="stats-summary">
          <div
            v-for="card in cards"
            :key="card.label"
            class="stats-summary-item"
          >
            <span class="stats-label">{{ card.label }}</span>
            <div class="stats-value">
              {{ card.value
              }}<small v-if="card.suffix">{{ card.suffix }}</small>
            </div>
            <span class="faint t-11">{{ card.hint }}</span>
          </div>
        </div>

        <section class="stats-block">
          <div class="stats-block-head">
            <h2>Минуты по дням</h2>
            <span class="faint t-12">
              пик {{ peakMinutes }} мин · в среднем {{ avgPerDay }} мин
            </span>
          </div>

          <div class="card stats-panel">
            <div v-if="hasChartData" class="stats-graph">
              <div class="stats-graph-body">
                <div class="stats-graph-scale">
                  <span>{{ peakMinutes }}</span>
                  <span>{{ Math.round(peakMinutes / 2) }}</span>
                  <span>0</span>
                </div>

                <div class="stats-graph-plot">
                  <svg
                    class="stats-graph-svg"
                    :viewBox="`0 0 ${WIDTH} ${HEIGHT}`"
                    preserveAspectRatio="none"
                    aria-hidden="true"
                  >
                    <defs>
                      <linearGradient
                        id="statsAreaFill"
                        x1="0"
                        y1="0"
                        x2="0"
                        y2="1"
                      >
                        <stop
                          offset="0%"
                          stop-color="var(--accent)"
                          stop-opacity="0.36"
                        />
                        <stop
                          offset="100%"
                          stop-color="var(--accent)"
                          stop-opacity="0"
                        />
                      </linearGradient>
                    </defs>

                    <line
                      v-for="line in gridLines"
                      :key="line"
                      class="stats-grid-line"
                      x1="0"
                      :y1="line"
                      :x2="WIDTH"
                      :y2="line"
                      vector-effect="non-scaling-stroke"
                    />

                    <path
                      class="stats-area"
                      :d="areaPath"
                      fill="url(#statsAreaFill)"
                    />
                    <path
                      class="stats-line"
                      :d="linePath"
                      vector-effect="non-scaling-stroke"
                    />
                  </svg>

                  <div class="stats-graph-hover">
                    <div
                      v-for="point in points"
                      :key="point.day"
                      class="stats-graph-slot"
                      :style="{ left: `${point.x}%` }"
                    >
                      <span
                        class="stats-graph-dot"
                        :class="{ zero: !point.seconds }"
                        :style="{ top: `${point.top}%` }"
                      />
                      <q-tooltip anchor="top middle" self="bottom middle">
                        {{ point.label }} - {{ point.minutes }} мин
                      </q-tooltip>
                    </div>
                  </div>
                </div>
              </div>

              <div class="stats-graph-axis">
                <span v-for="tick in axisTicks" :key="tick.day">
                  {{ tick.label }}
                </span>
              </div>
            </div>

            <div v-else class="stats-empty">
              <Icon name="stats" :size="22" class="faint" />
              <span class="faint t-13"
                >Пока нечего показать за этот период.</span
              >
            </div>
          </div>
        </section>

        <div class="stats-columns">
          <section class="stats-block">
            <div class="stats-block-head">
              <h2>Топ треков</h2>
              <div class="stats-sort">
                <button
                  v-for="option in rankSorts"
                  :key="option.id"
                  class="stats-sort-btn"
                  :class="{ on: rankSort === option.id }"
                  type="button"
                  @click="rankSort = option.id"
                >
                  {{ option.label }}
                </button>
              </div>
            </div>

            <div class="card stats-panel stats-list">
              <div v-if="!topTracks.length" class="stats-empty">
                <Icon name="note" :size="20" class="faint" />
                <span class="faint t-13"
                  >Пока пусто - послушай что-нибудь.</span
                >
              </div>

              <div
                v-for="(row, i) in topTracks"
                :key="row.key"
                class="stats-row"
              >
                <span class="stats-rank" :class="{ top: i < 3 }">
                  {{ i + 1 }}
                </span>
                <div class="cover stats-cover">
                  <img v-if="row.cover" :src="row.cover" decoding="async" />
                  <Icon v-else name="note" :size="14" class="faint" />
                </div>
                <div class="stats-row-meta">
                  <div class="ellipsis t-13">{{ row.label }}</div>
                  <div class="faint t-11 ellipsis">{{ row.sub }}</div>
                </div>
                <span class="stats-badge">{{ rankValue(row) }}</span>
              </div>
            </div>
          </section>

          <section class="stats-block">
            <div class="stats-block-head">
              <h2>Топ исполнителей</h2>
              <div class="stats-sort">
                <button
                  v-for="option in rankSorts"
                  :key="option.id"
                  class="stats-sort-btn"
                  :class="{ on: rankSort === option.id }"
                  type="button"
                  @click="rankSort = option.id"
                >
                  {{ option.label }}
                </button>
              </div>
            </div>

            <div class="card stats-panel stats-list">
              <div v-if="!topArtists.length" class="stats-empty">
                <Icon name="artist" :size="20" class="faint" />
                <span class="faint t-13">Пока пусто.</span>
              </div>

              <div
                v-for="(row, i) in topArtists"
                :key="row.key"
                class="stats-row"
              >
                <span class="stats-rank" :class="{ top: i < 3 }">
                  {{ i + 1 }}
                </span>
                <div class="cover stats-cover round">
                  <img v-if="row.cover" :src="row.cover" decoding="async" />
                  <Icon v-else name="artist" :size="14" class="faint" />
                </div>
                <div class="stats-row-meta">
                  <div class="ellipsis t-13">{{ row.label }}</div>
                  <div class="stats-meter">
                    <span :style="{ width: `${artistShare(row.seconds)}%` }" />
                  </div>
                </div>
                <span class="stats-badge">{{ rankValue(row) }}</span>
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { askConfirm } from "@/lib/dialogs";
import Icon from "@/components/Icon.vue";
import type { RankRow } from "@/stores/stats";
import { plural } from "@/lib/format";
import { useStatsStore } from "@/stores/stats";

type ChartPoint = {
  day: string;
  label: string;
  seconds: number;
  minutes: number;
  x: number;
  y: number;
  top: number;
};

const WIDTH = 100;
const HEIGHT = 40;
const PADDING = 3;

const stats = useStatsStore();
type RankSort = "time" | "plays";

const days = ref(7);

const periods = [
  { days: 7, label: "Неделя" },
  { days: 30, label: "Месяц" },
  { days: 365, label: "Год" },
];

const periodLabel = computed(
  () => periods.find((p) => p.days === days.value)?.label ?? "Неделя",
);

const hours = computed(
  () => Math.round((stats.totalSeconds(days.value) / 3600) * 10) / 10,
);

const rankSort = ref<RankSort>("time");

const rankSorts: Array<{ id: RankSort; label: string }> = [
  { id: "time", label: "По времени" },
  { id: "plays", label: "По прослушиваниям" },
];

function rankBy(rows: RankRow[]): RankRow[] {
  const sorted = [...rows];
  sorted.sort((a, b) =>
    rankSort.value === "plays" ? b.plays - a.plays : b.seconds - a.seconds,
  );
  return sorted;
}

function rankValue(row: RankRow): string {
  if (rankSort.value === "plays")
    return plural(row.plays, "раз", "раза", "раз");
  const minutes = Math.max(1, Math.round(row.seconds / 60));
  return `${minutes} мин`;
}

const topTracks = computed(() => rankBy(stats.topTracks(days.value, 10)));
const topArtists = computed(() => rankBy(stats.topArtists(days.value, 10)));
const chart = computed(() => stats.daily(Math.min(days.value, 30)));
const hasChartData = computed(() => chart.value.some((row) => row.seconds > 0));

const avgPerDay = computed(() =>
  Math.round(stats.totalSeconds(days.value) / 60 / days.value),
);

const uniqueTracks = computed(() => stats.uniqueTracks(days.value));
const activeDays = computed(() => stats.activeDays(days.value));

const peak = computed(() =>
  Math.max(1, ...chart.value.map((row) => row.seconds)),
);
const peakMinutes = computed(() => Math.max(1, Math.round(peak.value / 60)));

const artistPeak = computed(() =>
  Math.max(1, ...topArtists.value.map((row) => row.seconds)),
);

const points = computed<ChartPoint[]>(() => {
  const rows = chart.value;
  const max = peak.value;
  const usable = HEIGHT - PADDING * 2;
  const step = rows.length > 1 ? WIDTH / (rows.length - 1) : 0;

  return rows.map((row, i) => {
    const ratio = Math.min(1, row.seconds / max);
    const y = HEIGHT - PADDING - ratio * usable;
    return {
      day: row.day,
      label: row.label,
      seconds: row.seconds,
      minutes: Math.round(row.seconds / 60),
      x: rows.length > 1 ? i * step : WIDTH / 2,
      y,
      top: (y / HEIGHT) * 100,
    };
  });
});

function curve(list: ChartPoint[]): string {
  if (!list.length) return "";
  const first = list[0] as ChartPoint;
  if (list.length === 1) return `M ${first.x} ${first.y}`;

  let path = `M ${first.x.toFixed(2)} ${first.y.toFixed(2)}`;
  for (let i = 0; i < list.length - 1; i += 1) {
    const p0 = list[Math.max(0, i - 1)] as ChartPoint;
    const p1 = list[i] as ChartPoint;
    const p2 = list[i + 1] as ChartPoint;
    const p3 = list[Math.min(list.length - 1, i + 2)] as ChartPoint;
    const c1x = p1.x + (p2.x - p0.x) / 6;
    const c2x = p2.x - (p3.x - p1.x) / 6;
    const lo = Math.min(p1.y, p2.y);
    const hi = Math.max(p1.y, p2.y);
    const clampY = (v: number) => Math.min(hi, Math.max(lo, v));
    const c1y = clampY(p1.y + (p2.y - p0.y) / 6);
    const c2y = clampY(p2.y - (p3.y - p1.y) / 6);
    path += ` C ${c1x.toFixed(2)} ${c1y.toFixed(2)}, ${c2x.toFixed(2)} ${c2y.toFixed(2)}, ${p2.x.toFixed(2)} ${p2.y.toFixed(2)}`;
  }
  return path;
}

const linePath = computed(() => curve(points.value));

const areaPath = computed(() => {
  const list = points.value;
  if (!list.length) return "";
  const first = list[0] as ChartPoint;
  const last = list[list.length - 1] as ChartPoint;
  return `${curve(list)} L ${last.x.toFixed(2)} ${HEIGHT} L ${first.x.toFixed(2)} ${HEIGHT} Z`;
});

const gridLines = [PADDING, HEIGHT / 2, HEIGHT - PADDING];

const axisTicks = computed(() => {
  const list = points.value;
  if (list.length <= 6) return list;
  const step = Math.ceil(list.length / 6);
  return list.filter((_, i) => i % step === 0 || i === list.length - 1);
});

const cards = computed(() => [
  {
    label: "Часов прослушано",
    value: hours.value,
    suffix: " ч",
    hint: periodLabel.value.toLowerCase(),
  },
  {
    label: "Прослушиваний",
    value: stats.totalPlays(days.value),
    suffix: "",
    hint: "засчитанных воспроизведений",
  },
  {
    label: "Уникальных треков",
    value: uniqueTracks.value,
    suffix: "",
    hint: `в ${activeDays.value} активных днях`,
  },
  {
    label: "Среднее за день",
    value: avgPerDay.value,
    suffix: " мин",
    hint: "по дням периода",
  },
]);

function artistShare(seconds: number): number {
  return Math.max(4, Math.round((seconds / artistPeak.value) * 100));
}

async function clear() {
  const ok = await askConfirm({
    title: "Очистить статистику?",
    message: "Вся локальная история прослушиваний будет удалена.",
    okLabel: "Очистить",
    danger: true,
  });
  if (ok) stats.clear();
}
</script>
