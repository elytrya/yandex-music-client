<template>
  <Teleport to="body">
    <Transition name="ai-fade">
      <div v-if="open" class="ai-overlay" @click.self="close">
        <div class="ai-card" role="dialog" aria-modal="true">
          <div class="ai-head">
            <div class="cover ai-avatar round">
              <img v-if="artist?.cover_url" :src="artist.cover_url" />
              <Icon v-else name="person" :size="26" class="faint" />
            </div>
            <div class="ai-head-text">
              <div class="ai-kind">Подробная информация</div>
              <div class="ai-name ellipsis">{{ artist?.name }}</div>
            </div>
            <button
              class="icon-btn"
              type="button"
              title="Закрыть"
              @click="close"
            >
              <Icon name="close" :size="18" />
            </button>
          </div>

          <div class="ai-body">
            <div class="ai-stats">
              <div v-for="card in cards" :key="card.key" class="ai-stat">
                <div class="ai-stat-label">{{ card.label }}</div>
                <div class="ai-stat-value">
                  {{ card.value === null ? "—" : formatCount(card.value) }}
                </div>
                <div
                  v-if="card.diff !== null && card.diff !== 0"
                  class="ai-delta"
                  :class="card.diff > 0 ? 'up' : 'down'"
                >
                  {{ card.diff > 0 ? "+" : "−"
                  }}{{ formatCount(Math.abs(card.diff)) }}
                  <span class="ai-delta-since">{{ card.sinceLabel }}</span>
                </div>
                <div v-else class="ai-delta muted">
                  {{ card.diff === 0 ? "без изменений" : "нет данных" }}
                </div>
              </div>
            </div>

            <div v-if="listenerSeries.length > 1" class="ai-block">
              <div class="ai-block-head">
                <span>Слушатели за всё время наблюдений</span>
                <span class="faint t-11">
                  {{ listenerSeries.length }} замеров
                </span>
              </div>
              <svg
                class="ai-spark"
                viewBox="0 0 320 64"
                preserveAspectRatio="none"
              >
                <polyline
                  class="ai-spark-line"
                  :points="sparkPoints"
                  fill="none"
                />
                <polygon class="ai-spark-area" :points="sparkArea" />
              </svg>
              <div class="ai-spark-axis">
                <span>{{ formatDate(listenerSeries[0].t) }}</span>
                <span>
                  {{ formatDate(listenerSeries[listenerSeries.length - 1].t) }}
                </span>
              </div>
            </div>

            <div class="ai-block">
              <div class="ai-block-head"><span>Динамика слушателей</span></div>
              <div class="ai-rows">
                <div v-for="row in trendRows" :key="row.label" class="ai-row">
                  <span class="ai-row-label">{{ row.label }}</span>
                  <span
                    class="ai-row-value"
                    :class="
                      row.diff === null || row.diff === 0
                        ? 'muted'
                        : row.diff > 0
                          ? 'up'
                          : 'down'
                    "
                  >
                    <template v-if="row.diff === null">нет данных</template>
                    <template v-else-if="row.diff === 0"
                      >без изменений</template
                    >
                    <template v-else>
                      {{ row.diff > 0 ? "+" : "−"
                      }}{{ formatCount(Math.abs(row.diff)) }}
                      <span v-if="row.percent !== null" class="faint">
                        ({{ row.percent > 0 ? "+" : "−"
                        }}{{ Math.abs(row.percent).toFixed(1) }}%)
                      </span>
                    </template>
                  </span>
                </div>
              </div>
              <p class="ai-note faint t-11">
                Яндекс Музыка не отдаёт историю показателей, поэтому прирост
                считается по замерам, сохранённым локально при каждом открытии
                страницы артиста.
              </p>
            </div>

            <div v-if="artist?.genres?.length" class="ai-block">
              <div class="ai-block-head"><span>Жанры</span></div>
              <div class="ai-chips">
                <span v-for="g in artist.genres" :key="g" class="chip">{{
                  g
                }}</span>
              </div>
            </div>

            <div v-if="artist?.description" class="ai-block">
              <div class="ai-block-head"><span>Об артисте</span></div>
              <p class="ai-desc">{{ artist.description }}</p>
            </div>

            <div v-if="links.length" class="ai-block">
              <div class="ai-block-head"><span>Ссылки</span></div>
              <div class="ai-links">
                <a
                  v-for="link in links"
                  :key="link.href"
                  class="ai-link"
                  :href="link.href"
                  :title="link.href"
                  @click.prevent="openLink(link.href)"
                >
                  <Icon name="share" :size="13" />
                  <span class="ellipsis">{{ link.title }}</span>
                </a>
              </div>
            </div>

            <div class="ai-block">
              <div class="ai-block-head"><span>Служебное</span></div>
              <div class="ai-rows">
                <div class="ai-row">
                  <span class="ai-row-label">ID артиста</span>
                  <span class="ai-row-value mono">{{ artist?.id }}</span>
                </div>
                <div class="ai-row">
                  <span class="ai-row-label">Первый замер</span>
                  <span class="ai-row-value">
                    {{ history.length ? formatDate(history[0].t) : "—" }}
                  </span>
                </div>
                <div class="ai-row">
                  <span class="ai-row-label">Обновлено</span>
                  <span class="ai-row-value">
                    {{
                      history.length
                        ? formatDate(history[history.length - 1].t)
                        : "—"
                    }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <div class="ai-foot">
            <button class="btn" type="button" @click="resetHistory">
              <Icon name="trash" :size="14" />
              <span>Сбросить историю замеров</span>
            </button>
            <button class="btn-solid" type="button" @click="close">
              Готово
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
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

const history = ref<ArtistSnapshot[]>([]);

watch(
  () => [props.open, props.artist?.id] as const,
  ([open, id]) => {
    if (open && id) history.value = artistHistory(id);
  },
  { immediate: true },
);

function close() {
  emit("update:open", false);
}

function openLink(url: string) {
  void api.openExternal(url);
}

function formatCount(value: number): string {
  return value.toLocaleString("ru-RU");
}

function formatDate(ts: number): string {
  return new Date(ts).toLocaleDateString("ru-RU", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
}

function sinceLabel(since: number | null): string {
  if (!since) return "";
  const days = Math.round((Date.now() - since) / (24 * 60 * 60 * 1000));
  if (days <= 0) return "сегодня";
  if (days === 1) return "за сутки";
  return `за ${days} дн.`;
}

const cards = computed(() => {
  const h = history.value;
  const defs = [
    { key: "listeners", label: "Слушателей в месяц" },
    { key: "likes", label: "Лайков" },
    { key: "tracks", label: "Треков" },
    { key: "albums", label: "Альбомов" },
  ] as const;

  return defs.map((def) => {
    const d = statDelta(h, def.key, null);
    return {
      key: def.key,
      label: def.label,
      value: d.value,
      diff: d.diff,
      sinceLabel: sinceLabel(d.since),
    };
  });
});

const trendRows = computed(() => {
  const h = history.value;
  const spans: Array<{ label: string; days: number | null }> = [
    { label: "С прошлого захода", days: null },
    { label: "За 7 дней", days: 7 },
    { label: "За 30 дней", days: 30 },
    { label: "За 90 дней", days: 90 },
  ];

  return spans.map((span) => {
    const d = statDelta(h, "listeners", span.days);
    const base = d.diff !== null && d.value !== null ? d.value - d.diff : null;
    const percent =
      base && base > 0 && d.diff !== null ? (d.diff / base) * 100 : null;
    return { label: span.label, diff: d.diff, percent };
  });
});

const listenerSeries = computed(() => seriesOf(history.value, "listeners"));

const sparkPoints = computed(() => {
  const pts = listenerSeries.value;
  if (pts.length < 2) return "";
  const values = pts.map((p) => p.v);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;
  return pts
    .map((p, i) => {
      const x = (i / (pts.length - 1)) * 320;
      const y = 60 - ((p.v - min) / range) * 52;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
});

const sparkArea = computed(() => {
  const line = sparkPoints.value;
  if (!line) return "";
  return `0,64 ${line} 320,64`;
});

function resetHistory() {
  if (!props.artist?.id) return;
  clearArtistStats(props.artist.id);
  history.value = [];
}
</script>

<style scoped>
.ai-overlay {
  position: fixed;
  inset: 0;
  z-index: 7000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px;
  background: rgba(0, 0, 0, 0.62);
  backdrop-filter: blur(6px);
}
.ai-card {
  display: flex;
  flex-direction: column;
  width: min(680px, 100%);
  max-height: 100%;
  overflow: hidden;
  border: 1px solid var(--line);
  border-radius: var(--radius, 12px);
  background: var(--surface);
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
}
.ai-head {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 18px 20px;
  border-bottom: 1px solid var(--line);
}
.ai-avatar {
  flex: 0 0 auto;
  width: 52px;
  height: 52px;
}
.ai-head-text {
  flex: 1;
  min-width: 0;
}
.ai-kind {
  font-size: 11px;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--fg-dim);
}
.ai-name {
  font-size: 19px;
  font-weight: 600;
}
.ai-body {
  flex: 1;
  overflow-y: auto;
  padding: 18px 20px;
}
.ai-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 10px;
}
.ai-stat {
  padding: 12px 14px;
  border: 1px solid var(--line);
  border-radius: 10px;
  background: var(--surface-2);
}
.ai-stat-label {
  font-size: 11px;
  color: var(--fg-dim);
}
.ai-stat-value {
  margin-top: 2px;
  font-size: 20px;
  font-weight: 650;
  line-height: 1.25;
}
.ai-delta {
  margin-top: 3px;
  font-size: 11px;
  font-weight: 600;
}
.ai-delta.up {
  color: #35c26a;
}
.ai-delta.down {
  color: var(--accent, #fa2d48);
}
.ai-delta.muted,
.ai-delta-since {
  color: var(--fg-dim);
  font-weight: 400;
}
.ai-delta-since {
  margin-left: 4px;
}
.ai-block {
  margin-top: 20px;
}
.ai-block-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  color: var(--fg-dim);
}
.ai-spark {
  display: block;
  width: 100%;
  height: 64px;
}
.ai-spark-line {
  stroke: var(--accent, #fa2d48);
  stroke-width: 2;
  vector-effect: non-scaling-stroke;
}
.ai-spark-area {
  fill: var(--accent, #fa2d48);
  opacity: 0.14;
}
.ai-spark-axis {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  color: var(--fg-dim);
}
.ai-rows {
  display: flex;
  flex-direction: column;
  gap: 1px;
  overflow: hidden;
  border: 1px solid var(--line);
  border-radius: 10px;
}
.ai-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 9px 13px;
  background: var(--surface-2);
  font-size: 13px;
}
.ai-row-label {
  color: var(--fg-dim);
}
.ai-row-value {
  font-weight: 600;
  text-align: right;
}
.ai-row-value.up {
  color: #35c26a;
}
.ai-row-value.down {
  color: var(--accent, #fa2d48);
}
.ai-row-value.muted {
  color: var(--fg-dim);
  font-weight: 400;
}
.ai-row-value.mono {
  font-family: ui-monospace, monospace;
  font-size: 12px;
}
.ai-note {
  margin: 8px 2px 0;
  line-height: 1.5;
}
.ai-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.ai-desc {
  margin: 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--fg-dim);
  white-space: pre-line;
}
.ai-links {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 6px;
}
.ai-link {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 8px 11px;
  border: 1px solid var(--line);
  border-radius: 9px;
  color: var(--fg);
  font-size: 12px;
  text-decoration: none;
  cursor: pointer;
  transition: background 0.14s ease;
}
.ai-link:hover {
  background: var(--hover);
}
.ai-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 14px 20px;
  border-top: 1px solid var(--line);
}
.ai-fade-enter-active,
.ai-fade-leave-active {
  transition: opacity 0.18s ease;
}
.ai-fade-enter-from,
.ai-fade-leave-to {
  opacity: 0;
}
</style>
