<template>
  <section id="settings-playlists" class="settings-group">
    <div class="settings-group-head">
      <h2>Плейлисты</h2>
      <p>Экспорт в текст или CSV и импорт обратно по названиям.</p>
    </div>

    <div class="setting-row column" style="gap: 10px; align-items: stretch">
      <div class="row items-center wrap" style="gap: 8px">
        <select v-model="exportKind" class="field" style="min-width: 220px">
          <option value="all">Все плейлисты</option>
          <option value="liked">Мне нравится</option>
          <option
            v-for="pl in library.playlists"
            :key="pl.kind"
            :value="String(pl.kind)"
          >
            {{ pl.title }}
          </option>
        </select>

        <select v-model="format" class="field" style="min-width: 140px">
          <option value="txt">Текст (.txt)</option>
          <option value="csv">Таблица (.csv)</option>
        </select>

        <button class="btn-solid" :disabled="busy" @click="runExport">
          {{ busy ? "Работаю…" : "Экспортировать" }}
        </button>
      </div>

      <div v-if="exportPath" class="faint t-12">Файл: {{ exportPath }}</div>
    </div>

    <div class="settings-group-head" style="margin-top: 18px">
      <h2 class="t-14">Импорт</h2>
      <p>
        Одна строка - один трек в виде «Исполнитель - Название». Можно вставить
        текст или выбрать файл.
      </p>
    </div>

    <div class="setting-row column" style="gap: 10px; align-items: stretch">
      <div class="row items-center wrap" style="gap: 8px">
        <button class="btn" :disabled="busy" @click="pickFile">
          <Icon name="upload" :size="15" />
          <span>Выбрать файл…</span>
        </button>
        <span v-if="fileName" class="faint t-12 ellipsis">{{ fileName }}</span>
        <input
          ref="fileInput"
          type="file"
          accept=".txt,.csv,text/plain"
          hidden
          @change="onFilePicked"
        />
      </div>

      <textarea
        v-model="importText"
        class="field import-area"
        rows="6"
        placeholder="Макан - Бомба&#10;ЕГОР КРИД - Malo"
      />

      <div class="row items-center wrap" style="gap: 8px">
        <select v-model="targetKind" class="field" style="min-width: 220px">
          <option value="">Куда добавить…</option>
          <option value="__new__">➕ Новый плейлист…</option>
          <option
            v-for="pl in library.playlists"
            :key="pl.kind"
            :value="String(pl.kind)"
          >
            {{ pl.title }}
          </option>
        </select>
        <button
          class="btn-solid"
          :disabled="busy || !targetKind || !importText.trim()"
          @click="runImport"
        >
          {{ busy ? `Импорт ${progress}` : "Импортировать" }}
        </button>
      </div>

      <div v-if="missed.length" class="faint t-12">
        Не нашлось ({{ missed.length }}): {{ missed.slice(0, 8).join("; ") }}
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Notify } from "quasar";
import Icon from "@/components/Icon.vue";
import { api } from "@/api/client";
import type { Track } from "@/api/types";
import { askText } from "@/lib/dialogs";
import { artistNames, formatDuration } from "@/lib/format";
import { useLibraryStore } from "@/stores/library";
import { useUiStore } from "@/stores/ui/index";

const library = useLibraryStore();
const ui = useUiStore();

const exportKind = ref("all");
const format = ref<"txt" | "csv">("txt");
const exportPath = ref("");
const importText = ref("");
const fileInput = ref<HTMLInputElement | null>(null);
const fileName = ref("");
const targetKind = ref("");
const missed = ref<string[]>([]);
const busy = ref(false);
const progress = ref("");

function escapeCsv(value: string): string {
  return `"${value.replace(/"/g, '""')}"`;
}

function render(title: string, tracks: Track[]): string {
  if (format.value === "csv") {
    const head = "playlist,title,artists,album,duration,track_id";
    const rows = tracks.map((t) =>
      [
        title,
        t.title,
        artistNames(t.artists),
        t.album_title ?? "",
        formatDuration(t.duration_ms || 0),
        t.id,
      ]
        .map(escapeCsv)
        .join(","),
    );
    return [head, ...rows].join("\n");
  }
  const lines = tracks.map((t) => `${artistNames(t.artists)} - ${t.title}`);
  return [`# ${title} (${tracks.length})`, ...lines, ""].join("\n");
}

async function runExport() {
  busy.value = true;
  exportPath.value = "";
  try {
    if (!library.playlists.length) await library.init();

    const jobs: Array<{ title: string; tracks: Track[] }> = [];

    if (exportKind.value === "liked" || exportKind.value === "all") {
      const liked = await api.likedTracks().catch(() => [] as Track[]);
      jobs.push({ title: "Мне нравится", tracks: liked });
    }

    if (exportKind.value === "all") {
      for (const pl of library.playlists) {
        const tracks = await api
          .playlistTracks(pl.kind)
          .catch(() => [] as Track[]);
        jobs.push({ title: pl.title, tracks });
      }
    } else if (exportKind.value !== "liked") {
      const kind = Number(exportKind.value);
      const pl = library.playlists.find((p) => p.kind === kind);
      const tracks = await api.playlistTracks(kind).catch(() => [] as Track[]);
      jobs.push({ title: pl?.title ?? `Плейлист ${kind}`, tracks });
    }

    if (exportKind.value === "all") {
      const seen = new Set<string>();
      for (const job of jobs) {
        job.tracks = job.tracks.filter((track) => {
          if (seen.has(track.id)) return false;
          seen.add(track.id);
          return true;
        });
      }
    }

    const body = jobs.map((job) => render(job.title, job.tracks)).join("\n");
    const stamp = new Date().toISOString().slice(0, 10);
    const name = `mashiro-${
      exportKind.value === "all" ? "library" : "playlist"
    }-${stamp}.${format.value}`;

    const path = await api.exportTextFile(
      name,
      body,
      ui.settings.downloadDir || null,
    );
    exportPath.value = path;
    Notify.create({ type: "positive", message: `Сохранил: ${path}` });
  } catch (e) {
    Notify.create({
      type: "negative",
      message: e instanceof Error ? e.message : "Не удалось экспортировать",
    });
  } finally {
    busy.value = false;
  }
}

function pickFile() {
  fileInput.value?.click();
}

async function onFilePicked(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  busy.value = true;
  try {
    importText.value = await file.text();
    fileName.value = file.name;
    Notify.create({ message: `Файл загружен: ${file.name}` });
  } catch (e) {
    Notify.create({
      type: "negative",
      message: e instanceof Error ? e.message : "Файл не прочитался",
    });
  } finally {
    busy.value = false;
    input.value = "";
  }
}

function parseLines(): string[] {
  return importText.value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("#"))
    .map((line) => line.replace(/^"|"$/g, ""))
    .slice(0, 400);
}

async function runImport() {
  let kind = Number(targetKind.value);

  if (targetKind.value === "__new__") {
    const title = await askText({
      title: "Новый плейлист",
      placeholder: "Название плейлиста",
      okLabel: "Создать",
    });
    if (!title) return;
    const created = await library.createPlaylist(title);
    if (!created) {
      Notify.create({
        type: "negative",
        message: "Не удалось создать плейлист",
      });
      return;
    }
    kind = created.kind;
    targetKind.value = String(created.kind);
  }

  if (!kind) return;
  const lines = parseLines();
  if (!lines.length) return;

  busy.value = true;
  missed.value = [];
  let added = 0;

  try {
    for (let i = 0; i < lines.length; i++) {
      progress.value = `${i + 1}/${lines.length}`;
      const line = lines[i] ?? "";
      const clean = line.replace(/[--]/g, " ").replace(/\s+/g, " ").trim();
      const found = await api.search(clean).catch(() => null);
      const track = found?.tracks?.[0];
      if (!track) {
        missed.value.push(line);
        continue;
      }
      await api
        .playlistAdd(kind, track.id, track.album_id || track.id, 0)
        .then(() => {
          added += 1;
        })
        .catch(() => {
          missed.value.push(line);
        });
    }
    await library.refreshPlaylists();
    Notify.create({
      type: added ? "positive" : "negative",
      message: `Добавил ${added} из ${lines.length}`,
    });
  } finally {
    busy.value = false;
    progress.value = "";
  }
}
</script>
