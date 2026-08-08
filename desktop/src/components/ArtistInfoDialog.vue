<template>
  <q-dialog v-model="model">
    <div class="ainfo">
      <div class="ainfo-head">
        <div class="cover ainfo-avatar round">
          <img v-if="artist?.cover_url" :src="artist.cover_url" />
          <Icon v-else name="person" :size="22" class="faint" />
        </div>
        <div class="col" style="min-width: 0">
          <div class="ainfo-name ellipsis">{{ artist?.name }}</div>
          <div class="ainfo-sub ellipsis">{{ subtitle }}</div>
        </div>
        <div class="icon-btn round" @click="model = false">
          <Icon name="close" :size="16" />
        </div>
      </div>

      <div class="ainfo-body">
        <div class="ainfo-facts">
          <div v-for="fact in facts" :key="fact.key" class="ainfo-fact">
            <span class="ainfo-fact-label">{{ fact.label }}</span>
            <span class="ainfo-fact-value">{{ fact.value }}</span>
            <span v-if="fact.delta" class="ainfo-fact-delta" :class="fact.dir">
              {{ fact.delta }}
            </span>
          </div>
        </div>

        <div v-if="chart" class="ainfo-chart">
          <svg viewBox="0 0 300 56" preserveAspectRatio="none">
            <polyline :points="chart.line" fill="none" />
          </svg>
          <div class="ainfo-chart-axis">
            <span>{{ chart.from }}</span>
            <span>слушатели по локальным замерам</span>
            <span>{{ chart.to }}</span>
          </div>
        </div>

        <p v-if="artist?.description" class="ainfo-text">
          {{ artist.description }}
        </p>

        <div v-if="artist?.genres?.length" class="ainfo-chips">
          <span v-for="genre in artist.genres" :key="genre" class="chip">
            {{ genre }}
          </span>
        </div>

        <div v-if="links.length" class="ainfo-links">
          <a
            v-for="link in links"
            :key="link.href"
            :title="link.href"
            @click.prevent="openLink(link.href)"
          >
            {{ link.title }}
          </a>
        </div>
      </div>

      <div class="ainfo-foot">
        <span class="ainfo-meta">
          id {{ artist?.id
          }}<template v-if="updatedAt"> · обновлено {{ updatedAt }}</template>
        </span>
        <button
          v-if="history.length"
          class="ainfo-plain"
          type="button"
          @click="resetHistory"
        >
          Очистить замеры
        </button>
      </div>
    </div>
  </q-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Icon from "@/components/Icon.vue";
import { api } from "@/api/client";
import type { ArtistPage } from "@/api/types";
import type { ArtistSnapshot } from "@/lib/artistStats";
import {
  artistHistory,
  clearArtistStats,
  seriesOf,
  statDelta,
} from "@/lib/artistStats";

const props = defineProps<{
  open: boolean;
  artist: ArtistPage | null;
  links: Array<{ href: string; title: string }>;
}>();

const emit = defineEmits<{ "update:open": [boolean] }>();

const model = computed({
  get: () => props.open,
  set: (value: boolean) => emit("update:open", value),
});

const history = ref<ArtistSnapshot[]>([]);

watch(
  () => [props.open, props.artist?.id] as const,
  ([open, id]) => {
    if (open && id) history.value = artistHistory(id);
  },
  { immediate: true },
);

const subtitle = computed(() => {
  const genres = props.artist?.genres ?? [];
  return genres.length ? genres.slice(0, 3).join(" · ") : "Артист";
});

const count = (value: number) => value.toLocaleString("ru-RU");

const day = (ts: number) =>
  new Date(ts).toLocaleDateString("ru-RU", { day: "2-digit", month: "short" });

const facts = computed(() => {
  const defs = [
    { key: "listeners", label: "Слушателей в месяц" },
    { key: "likes", label: "Лайков" },
    { key: "tracks", label: "Треков" },
    { key: "albums", label: "Альбомов" },
  ] as const;

  return defs.map((def) => {
    const stat = statDelta(history.value, def.key, null);
    const diff = stat.diff ?? 0;
    return {
      key: def.key,
      label: def.label,
      value: stat.value === null ? "—" : count(stat.value),
      delta:
        diff === 0 ? "" : `${diff > 0 ? "+" : "−"}${count(Math.abs(diff))}`,
      dir: diff > 0 ? "up" : "down",
    };
  });
});

const chart = computed(() => {
  const points = seriesOf(history.value, "listeners");
  if (points.length < 3) return null;

  const values = points.map((p) => p.v);
  const min = Math.min(...values);
  const range = Math.max(...values) - min || 1;
  const line = points
    .map((p, i) => {
      const x = (i / (points.length - 1)) * 300;
      const y = 52 - ((p.v - min) / range) * 46;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");

  return {
    line,
    from: day(points[0]!.t),
    to: day(points[points.length - 1]!.t),
  };
});

const updatedAt = computed(() => {
  const last = history.value[history.value.length - 1];
  return last ? day(last.t) : "";
});

function openLink(url: string) {
  void api.openExternal(url);
}

function resetHistory() {
  if (!props.artist?.id) return;
  clearArtistStats(props.artist.id);
  history.value = [];
}
</script>

<style scoped>
.ainfo {
  display: flex;
  flex-direction: column;
  width: min(520px, 92vw);
  max-height: 82vh;
  overflow: hidden;
  border: 1px solid var(--line);
  border-radius: var(--radius, 12px);
  background: var(--surface);
}
.ainfo-head {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 14px 12px;
}
.ainfo-avatar {
  flex: 0 0 auto;
  width: 42px;
  height: 42px;
}
.ainfo-name {
  font-size: 15px;
  font-weight: 600;
}
.ainfo-sub {
  margin-top: 1px;
  font-size: 12px;
  color: var(--fg-dim);
}
.ainfo-body {
  overflow-y: auto;
  padding: 0 16px 16px;
}
.ainfo-facts {
  display: flex;
  flex-direction: column;
}
.ainfo-fact {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 7px 0;
  border-top: 1px solid var(--line);
  font-size: 13px;
}
.ainfo-fact-label {
  flex: 1;
  color: var(--fg-dim);
}
.ainfo-fact-value {
  font-variant-numeric: tabular-nums;
}
.ainfo-fact-delta {
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}
.ainfo-fact-delta.up {
  color: #35c26a;
}
.ainfo-fact-delta.down {
  color: var(--accent, #fa2d48);
}
.ainfo-chart {
  margin-top: 14px;
}
.ainfo-chart svg {
  display: block;
  width: 100%;
  height: 56px;
}
.ainfo-chart polyline {
  stroke: var(--fg-dim);
  stroke-width: 1.5;
  vector-effect: non-scaling-stroke;
}
.ainfo-chart-axis {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  font-size: 10px;
  color: var(--fg-dim);
}
.ainfo-text {
  margin: 16px 0 0;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-line;
}
.ainfo-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 14px;
}
.ainfo-links {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 14px;
  margin-top: 14px;
}
.ainfo-links a {
  color: var(--fg-dim);
  font-size: 12px;
  text-decoration: none;
  cursor: pointer;
  border-bottom: 1px solid transparent;
}
.ainfo-links a:hover {
  color: var(--fg);
  border-bottom-color: var(--line);
}
.ainfo-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 16px 12px;
  border-top: 1px solid var(--line);
}
.ainfo-meta {
  font-size: 11px;
  color: var(--fg-dim);
}
.ainfo-plain {
  padding: 0;
  border: 0;
  background: none;
  color: var(--fg-dim);
  font-size: 11px;
  cursor: pointer;
}
.ainfo-plain:hover {
  color: var(--fg);
}
</style>
