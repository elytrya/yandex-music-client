<template>
  <div class="gp">
    <div v-if="!genius.ready" class="gp-note">
      <span>
        Genius выключен. Открой Настройки → Genius и вставь Client Access Token.
      </span>
      <div class="btn" @click="openUrl('https://genius.com/api-clients')">
        Получить токен
      </div>
    </div>

    <div v-else-if="personOpen" class="gp-person">
      <button class="gp-back" type="button" @click="genius.closePerson()">
        <Icon name="chevronLeft" :size="14" />
        <span>Назад к треку</span>
      </button>

      <div v-if="genius.personLoading" class="gp-note">
        <q-spinner size="16px" color="grey-6" />
        <span>Открываю страницу…</span>
      </div>

      <div v-else-if="genius.personError" class="gp-note">
        <span>{{ genius.personError }}</span>
      </div>

      <template v-else-if="genius.person">
        <div class="gp-head">
          <div class="cover gp-art round">
            <img v-if="genius.person.image" :src="genius.person.image" />
            <Icon v-else name="person" :size="18" class="faint" />
          </div>
          <div class="col" style="min-width: 0">
            <div class="gp-title ellipsis">{{ genius.person.name }}</div>
            <div class="gp-meta ellipsis">{{ personMeta }}</div>
          </div>
        </div>

        <p v-if="genius.person.description" class="gp-text">
          {{ genius.person.description }}
        </p>

        <div class="gp-links">
          <a @click.prevent="openUrl(genius.person.url)">Открыть на Genius</a>
          <a
            v-for="social in genius.person.socials"
            :key="social.url"
            @click.prevent="openUrl(social.url)"
          >
            {{ social.kind }}: {{ social.handle }}
          </a>
        </div>
      </template>
    </div>

    <div v-else-if="picking" class="gp-person">
      <button class="gp-back" type="button" @click="picking = false">
        <Icon name="chevronLeft" :size="14" />
        <span>Назад</span>
      </button>

      <form class="gp-search" @submit.prevent="runSearch">
        <input v-model="query" type="text" placeholder="артист и название" />
        <button class="btn" type="submit">Найти</button>
      </form>

      <div v-if="genius.hitsLoading" class="gp-note">
        <q-spinner size="16px" color="grey-6" />
        <span>Ищу…</span>
      </div>

      <button
        v-for="hit in genius.hits"
        :key="hit.id"
        class="gp-hit"
        type="button"
        @click="choose(hit)"
      >
        <div class="cover gp-hit-art">
          <img v-if="hit.art" :src="hit.art" />
        </div>
        <span class="ellipsis">{{ hit.full_title || hit.title }}</span>
      </button>
    </div>

    <div v-else-if="genius.loading" class="gp-note">
      <q-spinner size="16px" color="grey-6" />
      <span>Смотрю Genius…</span>
    </div>

    <div v-else-if="!song" class="gp-note">
      <span>{{ genius.error || "На Genius ничего не нашлось" }}</span>
      <div class="btn" @click="startPicking">Найти вручную</div>
    </div>

    <template v-else>
      <div class="gp-head">
        <div class="cover gp-art">
          <img v-if="song.art" :src="song.art" />
          <Icon v-else name="note" :size="18" class="faint" />
        </div>
        <div class="col" style="min-width: 0">
          <div class="gp-title ellipsis">
            {{ song.full_title || song.title }}
          </div>
          <div class="gp-meta ellipsis">{{ songMeta }}</div>
        </div>
        <div class="gp-head-actions">
          <button class="gp-plain" type="button" @click="openUrl(song.url)">
            На Genius
          </button>
          <button class="gp-plain" type="button" @click="refresh">
            Обновить
          </button>
          <button class="gp-plain" type="button" @click="startPicking">
            Не тот трек
          </button>
        </div>
      </div>

      <div class="gp-tabs">
        <button
          v-for="item in tabs"
          :key="item.value"
          type="button"
          :class="{ on: tab === item.value }"
          @click="tab = item.value"
        >
          {{ item.label }}
        </button>
      </div>

      <div v-if="tab === 'lyrics'" class="gp-body">
        <p v-if="!song.lyrics.length" class="gp-dim">
          {{ song.lyrics_error || "Genius не отдал текст для этой страницы." }}
        </p>
        <div v-for="(section, i) in lyricSections" :key="i" class="gp-section">
          <div v-if="section.heading" class="gp-section-head">
            {{ section.heading }}
          </div>
          <p v-if="section.lines.length" class="gp-block">
            <template v-for="(line, j) in section.lines" :key="j">
              {{ line }}<br v-if="j < section.lines.length - 1" />
            </template>
          </p>
        </div>
      </div>

      <div v-else-if="tab === 'quotes'" class="gp-body">
        <div v-if="!quotes.length" class="gp-dim">
          <p>
            {{ song.quotes_error || "К этому треку ещё не написали разборов." }}
          </p>
          <p v-if="song.annotation_count" class="gp-dim-inline">
            Genius насчитал разборов на этом треке:
            {{ song.annotation_count }}
          </p>
          <button class="gp-plain" type="button" @click="refresh">
            Загрузить заново
          </button>
        </div>
        <p v-else-if="song.quotes_source !== 'api'" class="gp-dim-inline">
          Разборы собраны в обход API-токена — источник:
          {{ song.quotes_source }}.
        </p>
        <div v-for="(quote, i) in quotes" :key="i" class="gp-quote">
          <div class="gp-quote-line">{{ quote.fragment }}</div>
          <p class="gp-quote-text">{{ quote.text }}</p>

          <div class="gp-quote-authors">
            <button
              v-for="author in quote.authors"
              :key="author.url || author.name"
              class="gp-author"
              type="button"
              @click="openUrl(author.url)"
            >
              <img v-if="author.image" :src="author.image" />
              <span class="ellipsis">{{ author.name }}</span>
              <b v-if="author.verified" title="Подтверждённый аккаунт">✓</b>
              <i v-if="author.iq">{{ shortNumber(author.iq) }} IQ</i>
            </button>
            <span v-if="!quote.authors.length" class="gp-dim-inline">
              Сообщество Genius
            </span>
          </div>

          <div class="gp-quote-foot">
            <span>{{ quoteMeta(quote) }}</span>
            <a v-if="quote.url" @click.prevent="openUrl(quote.url)">
              на Genius
            </a>
          </div>
        </div>
      </div>

      <div v-else class="gp-body">
        <p v-if="!groups.length" class="gp-dim">Участники записи не указаны.</p>
        <div v-for="group in groups" :key="group.role" class="gp-group">
          <div class="gp-group-role">{{ group.role }}</div>
          <div class="gp-people">
            <button
              v-for="person in group.people"
              :key="`${group.role}-${person.name}`"
              class="gp-person-chip"
              type="button"
              @click="openPerson(person)"
            >
              <img v-if="person.image" :src="person.image" />
              <span class="ellipsis">{{ person.name }}</span>
            </button>
          </div>
        </div>

        <div v-if="infoRows.length" class="gp-info">
          <div v-for="row in infoRows" :key="row.label" class="gp-info-row">
            <span class="gp-info-label">{{ row.label }}</span>
            <span class="gp-info-value">{{ row.value }}</span>
          </div>
        </div>

        <div v-if="tags.length" class="gp-tags">
          <span v-for="tag in tags" :key="tag" class="gp-tag">{{ tag }}</span>
        </div>

        <div v-if="media.length" class="gp-links">
          <a
            v-for="item in media"
            :key="item.url"
            @click.prevent="openUrl(item.url)"
          >
            {{ mediaLabel(item) }}
          </a>
        </div>

        <div
          v-for="relation in relations"
          :key="relation.kind"
          class="gp-group"
        >
          <div class="gp-group-role">{{ relation.kind }}</div>
          <div class="gp-people">
            <button
              v-for="item in relation.songs"
              :key="item.id"
              class="gp-person-chip"
              type="button"
              @click="openUrl(item.url)"
            >
              <img v-if="item.art" :src="item.art" />
              <span class="ellipsis">{{ item.full_title || item.title }}</span>
            </button>
          </div>
        </div>

        <p v-if="song.description" class="gp-text">{{ song.description }}</p>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Icon from "@/components/Icon.vue";
import { api } from "@/api/client";
import type {
  GeniusHit,
  GeniusMedia,
  GeniusPerson,
  GeniusQuote,
  Track,
} from "@/api/types";
import { artistNames } from "@/lib/format";
import { useGeniusStore } from "@/stores/genius";
import { useRouter } from "vue-router";

const props = defineProps<{ track: Track | null; active?: boolean }>();

const genius = useGeniusStore();
const router = useRouter();

function refresh() {
  if (props.track) void genius.refresh(props.track);
}

type Tab = "lyrics" | "quotes" | "credits";

const tab = ref<Tab>("lyrics");
const picking = ref(false);
const query = ref("");

const song = computed(() => genius.song);
const personOpen = computed(
  () => !!genius.person || genius.personLoading || !!genius.personError,
);

const tabs = computed(() => {
  const list: Array<{ value: Tab; label: string }> = [
    { value: "lyrics", label: "Текст" },
  ];
  if (genius.showQuotes) {
    list.push({
      value: "quotes",
      label: `Цитаты и разборы (${quotes.value.length})`,
    });
  }
  list.push({ value: "credits", label: "Над треком работали" });
  return list;
});

const quotes = computed(() =>
  genius.showQuotes ? song.value?.quotes || [] : [],
);

type LyricSection = { heading: string | null; lines: string[] };

const lyricSections = computed<LyricSection[]>(() => {
  const sections: LyricSection[] = [];
  let heading: string | null = null;
  let lines: string[] = [];

  const flush = () => {
    if (heading !== null || lines.length) sections.push({ heading, lines });
    heading = null;
    lines = [];
  };

  for (const raw of song.value?.lyrics || []) {
    const line = raw.trim();

    if (!line) {
      if (heading !== null || lines.length) flush();
      continue;
    }

    if (line.startsWith("[") && line.endsWith("]")) {
      if (heading !== null || lines.length) flush();
      heading = line.slice(1, -1).trim();
      continue;
    }

    lines.push(raw);
  }

  flush();
  return sections;
});

const groups = computed(() => {
  const map = new Map<string, GeniusPerson[]>();
  const people = [
    ...(song.value?.credits || []),
    ...(song.value?.verified_by || []),
  ];
  for (const person of people) {
    const list = map.get(person.role) || [];
    if (list.some((old) => old.name === person.name)) continue;
    list.push(person);
    map.set(person.role, list);
  }
  return [...map.entries()].map(([role, people]) => ({ role, people }));
});

const tags = computed(() => song.value?.tags || []);
const media = computed(() => song.value?.media || []);
const relations = computed(() => song.value?.relations || []);

const infoRows = computed(() => {
  const value = song.value;
  const rows: Array<{ label: string; value: string }> = [];
  if (!value) return rows;

  const album = value.album_info;
  if (album?.name) {
    rows.push({
      label: "Альбом",
      value: album.artist ? `${album.name} — ${album.artist}` : album.name,
    });
    if (album.release_date) {
      rows.push({ label: "Релиз альбома", value: album.release_date });
    }
  } else if (value.album) {
    rows.push({ label: "Альбом", value: value.album });
  }

  if (value.release_date)
    rows.push({ label: "Релиз", value: value.release_date });
  if (value.recording_location) {
    rows.push({ label: "Записано", value: value.recording_location });
  }
  if (value.language) {
    rows.push({ label: "Язык", value: value.language.toUpperCase() });
  }
  if (value.pageviews) {
    rows.push({ label: "Просмотры", value: ruNumber(value.pageviews) });
  }
  if (value.contributors) {
    rows.push({ label: "Участников", value: ruNumber(value.contributors) });
  }
  if (value.annotation_count) {
    rows.push({ label: "Аннотаций", value: ruNumber(value.annotation_count) });
  }
  if (value.concurrents) {
    rows.push({ label: "Читают сейчас", value: ruNumber(value.concurrents) });
  }
  if (value.apple_music_id) {
    rows.push({ label: "Apple Music ID", value: value.apple_music_id });
  }
  if (value.hot) rows.push({ label: "Статус", value: "В тренде на Genius" });

  return rows;
});

const producerNames = computed(() =>
  (song.value?.credits || [])
    .filter((person) => person.role === "Продюсер")
    .map((person) => person.name)
    .slice(0, 3)
    .join(", "),
);

const songMeta = computed(() => {
  const value = song.value;
  if (!value) return "";
  const parts: string[] = [];
  if (value.album) parts.push(value.album);
  if (value.release_date) parts.push(value.release_date);
  if (value.pageviews) parts.push(`${ruNumber(value.pageviews)} просмотров`);
  if (producerNames.value) parts.push(`прод: ${producerNames.value}`);
  return parts.join(" · ");
});

const personMeta = computed(() => {
  const person = genius.person;
  if (!person) return "";
  const parts: string[] = [];
  if (person.followers) {
    parts.push(
      `${person.followers.toLocaleString("ru-RU")} подписчиков на Genius`,
    );
  }
  if (person.alternate_names.length) {
    parts.push(person.alternate_names.slice(0, 3).join(", "));
  }
  return parts.join(" · ");
});

function ruNumber(value: number): string {
  return value.toLocaleString("ru-RU");
}

function shortNumber(value: number): string {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1000) return `${(value / 1000).toFixed(1)}K`;
  return String(value);
}

function quoteMeta(quote: GeniusQuote): string {
  const parts: string[] = [];
  if (quote.votes) parts.push(`${quote.votes} голосов`);
  if (quote.comments) parts.push(`${quote.comments} комментариев`);
  if (quote.verified) parts.push("подтверждено");
  if (quote.pinned) parts.push("закреплено");
  return parts.join(" · ");
}

function mediaLabel(item: GeniusMedia): string {
  const provider = item.provider
    ? item.provider.charAt(0).toUpperCase() + item.provider.slice(1)
    : "Ссылка";
  if (item.kind === "video") return `${provider} · видео`;
  if (item.kind === "audio") return `${provider} · аудио`;
  return provider;
}

function openUrl(url: string) {
  if (url) void api.openExternal(url);
}

function openPerson(person: GeniusPerson) {
  if (person.id) {
    void router.push({
      name: "genius-artist",
      params: { id: String(person.id) },
      query: person.role ? { role: person.role } : {},
    });
    return;
  }
  openUrl(person.url);
}

function startPicking() {
  picking.value = true;
  query.value = props.track
    ? `${props.track.title} ${artistNames(props.track.artists)}`.trim()
    : "";
  void genius.searchHits(query.value);
}

function runSearch() {
  void genius.searchHits(query.value);
}

async function choose(hit: GeniusHit) {
  picking.value = false;
  tab.value = "lyrics";
  await genius.pickHit(hit);
}

watch(
  () => [props.active !== false, props.track?.id] as const,
  ([visible, id]) => {
    if (!visible || !id) return;
    tab.value = "lyrics";
    picking.value = false;
    genius.closePerson();
    void genius.fetchSong(props.track);
  },
  { immediate: true },
);
</script>

<style scoped>
.gp {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 120px;
  padding: 14px 16px 16px;
  overflow-y: auto;
}
.gp-note {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 18px 0;
  color: var(--fg-dim);
  font-size: 13px;
}
.gp-head {
  display: flex;
  align-items: center;
  gap: 10px;
}
.gp-art {
  flex: 0 0 auto;
  width: 44px;
  height: 44px;
}
.gp-title {
  font-size: 14px;
  font-weight: 600;
}
.gp-meta {
  margin-top: 2px;
  font-size: 11px;
  color: var(--fg-dim);
}
.gp-head-actions {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-left: auto;
}
.gp-plain,
.gp-back {
  padding: 0;
  border: 0;
  background: none;
  color: var(--fg-dim);
  font-size: 11px;
  text-align: right;
  cursor: pointer;
}
.gp-plain:hover,
.gp-back:hover {
  color: var(--fg);
}
.gp-back {
  display: flex;
  align-items: center;
  gap: 4px;
  text-align: left;
}
.gp-tabs {
  display: flex;
  gap: 14px;
  border-bottom: 1px solid var(--line);
}
.gp-tabs button {
  padding: 0 0 7px;
  border: 0;
  border-bottom: 2px solid transparent;
  background: none;
  color: var(--fg-dim);
  font-size: 12px;
  cursor: pointer;
}
.gp-tabs button.on {
  color: var(--fg);
  border-bottom-color: currentColor;
}
.gp-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.gp-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.gp-section-head {
  align-self: flex-start;
  padding: 2px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 16%, transparent);
  color: var(--accent);
  font-size: 10.5px;
  font-weight: 650;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}
.gp-block {
  margin: 0;
  font-size: 13px;
  line-height: 1.7;
}
.gp-quote-authors {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  margin-bottom: 5px;
}
.gp-author {
  display: inline-flex;
  max-width: 220px;
  align-items: center;
  gap: 5px;
  padding: 2px 8px 2px 3px;
  border: 1px solid var(--line);
  border-radius: 999px;
  background: none;
  color: inherit;
  font-size: 11px;
  cursor: pointer;
}
.gp-author:hover {
  border-color: var(--fg-dim);
}
.gp-author img {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  object-fit: cover;
}
.gp-author b {
  color: var(--accent);
  font-size: 10px;
}
.gp-author i {
  color: var(--fg-dim);
  font-size: 10px;
  font-style: normal;
  font-variant-numeric: tabular-nums;
}
.gp-dim-inline {
  color: var(--fg-dim);
  font-size: 11px;
}
.gp-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border: 1px solid var(--line);
  border-radius: 10px;
}
.gp-info-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  font-size: 12px;
}
.gp-info-label {
  flex: 0 0 auto;
  color: var(--fg-dim);
}
.gp-info-value {
  overflow: hidden;
  text-align: right;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.gp-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}
.gp-tag {
  padding: 2px 8px;
  border: 1px solid var(--line);
  border-radius: 999px;
  color: var(--fg-dim);
  font-size: 11px;
}
.gp-text {
  margin: 4px 0 0;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-line;
}
.gp-dim {
  margin: 4px 0 0;
  color: var(--fg-dim);
  font-size: 13px;
}
.gp-quote {
  padding-left: 10px;
  border-left: 2px solid var(--line);
}
.gp-quote-line {
  font-size: 13px;
  font-style: italic;
  white-space: pre-line;
}
.gp-quote-text {
  margin: 5px 0 4px;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-line;
}
.gp-quote-foot {
  display: flex;
  gap: 10px;
  font-size: 11px;
  color: var(--fg-dim);
}
.gp-quote-foot a,
.gp-links a {
  color: var(--fg-dim);
  cursor: pointer;
  text-decoration: none;
  border-bottom: 1px solid var(--line);
}
.gp-quote-foot a:hover,
.gp-links a:hover {
  color: var(--fg);
}
.gp-group-role {
  margin-bottom: 6px;
  font-size: 11px;
  color: var(--fg-dim);
}
.gp-people {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.gp-person-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  max-width: 220px;
  padding: 4px 9px 4px 4px;
  border: 1px solid var(--line);
  border-radius: 999px;
  background: none;
  color: inherit;
  font-size: 12px;
  cursor: pointer;
}
.gp-person-chip:hover {
  border-color: var(--fg-dim);
}
.gp-person-chip img {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  object-fit: cover;
}
.gp-person {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.gp-links {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 14px;
  font-size: 12px;
}
.gp-search {
  display: flex;
  gap: 8px;
}
.gp-search input {
  flex: 1;
  min-width: 0;
  padding: 6px 10px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: none;
  color: inherit;
  font-size: 13px;
}
.gp-hit {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px;
  border: 0;
  border-radius: 8px;
  background: none;
  color: inherit;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}
.gp-hit:hover {
  background: var(--line);
}
.gp-hit-art {
  flex: 0 0 auto;
  width: 32px;
  height: 32px;
}
</style>
