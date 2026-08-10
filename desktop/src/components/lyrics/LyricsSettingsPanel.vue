<template>
  <div class="lyrics-settings" @click.stop>
    <div class="lset-head">
      <div class="lset-title">Вид текста</div>
      <button type="button" class="lset-reset" @click="ui.resetLyrics()">
        Сбросить
      </button>
    </div>

    <div class="lset-group">
      <div class="lset-group-head">Текст</div>

      <label class="lset-range">
        <span class="lset-range-top">
          <span>Размер</span><b>{{ s.lyricsFontSize }} px</b>
        </span>
        <input
          type="range"
          min="18"
          max="72"
          step="1"
          :value="s.lyricsFontSize"
          @input="onNum('lyricsFontSize', $event)"
        />
      </label>

      <label class="lset-range">
        <span class="lset-range-top">
          <span>Интерлиньяж</span><b>{{ s.lyricsLineHeight.toFixed(2) }}</b>
        </span>
        <input
          type="range"
          min="1"
          max="1.8"
          step="0.02"
          :value="s.lyricsLineHeight"
          @input="onNum('lyricsLineHeight', $event)"
        />
      </label>

      <label class="lset-range">
        <span class="lset-range-top">
          <span>Жирность</span><b>{{ s.lyricsWeight }}</b>
        </span>
        <input
          type="range"
          min="400"
          max="900"
          step="50"
          :value="s.lyricsWeight"
          @input="onNum('lyricsWeight', $event)"
        />
      </label>

      <div class="lset-label">Шрифт</div>
      <div class="lset-seg">
        <button
          v-for="font in fonts"
          :key="font.id"
          type="button"
          :class="{ on: s.lyricsFont === font.id }"
          @click="ui.set('lyricsFont', font.id)"
        >
          {{ font.label }}
        </button>
      </div>

      <div v-if="s.lyricsFont === 'custom'" class="lset-fonts">
        <input
          class="lset-input"
          type="text"
          :placeholder="
            fontsLoading
              ? 'Читаю шрифты системы…'
              : 'Найти шрифт или вписать своё название'
          "
          :value="fontQuery"
          @input="onFontQuery"
          @change="onFont"
        />

        <div v-if="s.lyricsFontCustom" class="lset-font-current">
          <span>Сейчас:</span>
          <b :style="{ fontFamily: `'${s.lyricsFontCustom}'` }">{{
            s.lyricsFontCustom
          }}</b>
        </div>

        <div v-if="fontsNotice" class="lset-hint">{{ fontsNotice }}</div>

        <div v-if="visibleFonts.length" class="lset-font-list">
          <button
            v-for="name in visibleFonts"
            :key="name"
            type="button"
            class="lset-font"
            :class="{ on: s.lyricsFontCustom === name }"
            @click="pickFont(name)"
          >
            <span class="lset-font-name ellipsis">{{ name }}</span>
            <span
              class="lset-font-sample"
              :style="{ fontFamily: `'${name}', sans-serif` }"
              >Аа Bb 12</span
            >
          </button>
        </div>

        <button
          v-if="!fontsLoading && needsPermission"
          type="button"
          class="lset-font-load"
          @click="loadFonts(true)"
        >
          Показать шрифты системы
        </button>
      </div>

      <div class="lset-label">Выравнивание</div>
      <div class="lset-seg">
        <button
          v-for="align in aligns"
          :key="align.id"
          type="button"
          :class="{ on: s.lyricsAlign === align.id }"
          @click="ui.set('lyricsAlign', align.id)"
        >
          {{ align.label }}
        </button>
      </div>

      <div class="lset-label">Активная строка</div>
      <div class="lset-seg">
        <button
          v-for="item in highlights"
          :key="item.id"
          type="button"
          :class="{ on: s.lyricsHighlight === item.id }"
          @click="ui.set('lyricsHighlight', item.id)"
        >
          {{ item.label }}
        </button>
      </div>
      <div v-if="s.lyricsHighlight === 'karaoke'" class="lset-hint">
        Строка заливается по ходу песни — работает с синхронным текстом.
      </div>

      <label class="lset-range" style="margin-top: 10px">
        <span class="lset-range-top">
          <span>Яркость остальных</span><b>{{ s.lyricsInactive }}%</b>
        </span>
        <input
          type="range"
          min="8"
          max="70"
          step="1"
          :value="s.lyricsInactive"
          @input="onNum('lyricsInactive', $event)"
        />
      </label>

      <label class="lset-range">
        <span class="lset-range-top">
          <span>Размытие строк</span><b>{{ s.lyricsLineBlur }} px</b>
        </span>
        <input
          type="range"
          min="0"
          max="8"
          step="0.5"
          :value="s.lyricsLineBlur"
          @input="onNum('lyricsLineBlur', $event)"
        />
      </label>
    </div>

    <div class="lset-group">
      <div class="lset-group-head">Фон</div>

      <div class="lset-seg">
        <button
          v-for="item in backdrops"
          :key="item.id"
          type="button"
          :class="{ on: s.lyricsBackdrop === item.id }"
          @click="ui.set('lyricsBackdrop', item.id)"
        >
          {{ item.label }}
        </button>
      </div>

      <label class="lset-range">
        <span class="lset-range-top">
          <span>Размытие</span><b>{{ s.lyricsBackgroundBlur }} px</b>
        </span>
        <input
          type="range"
          min="0"
          max="90"
          step="1"
          :value="s.lyricsBackgroundBlur"
          @input="onNum('lyricsBackgroundBlur', $event)"
        />
      </label>

      <label class="lset-range">
        <span class="lset-range-top">
          <span>Насыщенность</span><b>{{ s.lyricsBackgroundOpacity }}%</b>
        </span>
        <input
          type="range"
          min="0"
          max="100"
          step="1"
          :value="s.lyricsBackgroundOpacity"
          @input="onNum('lyricsBackgroundOpacity', $event)"
        />
      </label>

      <button
        type="button"
        class="lset-switch"
        :class="{ on: s.lyricsShowArtwork }"
        @click="ui.set('lyricsShowArtwork', !s.lyricsShowArtwork)"
      >
        <span>Обложка рядом с текстом</span>
        <span class="lset-track"><i /></span>
      </button>

      <button
        type="button"
        class="lset-switch"
        :class="{ on: s.lyricsGlow }"
        @click="ui.set('lyricsGlow', !s.lyricsGlow)"
      >
        <span>Свечение активной строки</span>
        <span class="lset-track"><i /></span>
      </button>

      <button
        type="button"
        class="lset-switch"
        :class="{ on: s.lyricsMotion }"
        @click="ui.set('lyricsMotion', !s.lyricsMotion)"
      >
        <span>Плавные переходы</span>
        <span class="lset-track"><i /></span>
      </button>
    </div>

    <div class="lset-group">
      <div class="lset-group-head">Разборы Genius</div>

      <button
        type="button"
        class="lset-switch"
        :class="{ on: s.lyricsAnnotations }"
        @click="ui.set('lyricsAnnotations', !s.lyricsAnnotations)"
      >
        <span>Отмечать строки с разбором</span>
        <span class="lset-track"><i /></span>
      </button>

      <template v-if="s.lyricsAnnotations">
        <div class="lset-label">Как отмечать</div>
        <div class="lset-seg">
          <button
            v-for="item in marks"
            :key="item.id"
            type="button"
            :class="{ on: s.lyricsAnnotationMark === item.id }"
            @click="ui.set('lyricsAnnotationMark', item.id)"
          >
            {{ item.label }}
          </button>
        </div>
        <div class="lset-hint">
          Левый клик по строке перематывает трек, правый - показывает значок
          разбора. Нужен включённый Genius с токеном.
        </div>
      </template>
    </div>

    <div class="lset-group">
      <div class="lset-group-head">Подвал панели</div>

      <button
        type="button"
        class="lset-switch"
        :class="{ on: s.lyricsShowCredits }"
        @click="ui.set('lyricsShowCredits', !s.lyricsShowCredits)"
      >
        <span>Продюсеры и авторы</span>
        <span class="lset-track"><i /></span>
      </button>

      <button
        type="button"
        class="lset-switch"
        :class="{ on: s.lyricsShowOrigin }"
        @click="ui.set('lyricsShowOrigin', !s.lyricsShowOrigin)"
      >
        <span>Подпись «Источник»</span>
        <span class="lset-track"><i /></span>
      </button>

      <div class="lset-hint">
        Строка внизу с участниками записи с Genius. Если выключить оба пункта,
        подвал пропадёт полностью.
      </div>
    </div>

    <div class="lset-group">
      <div class="lset-group-head">Источник по умолчанию</div>
      <div class="lset-seg">
        <button
          v-for="item in lyricsSources"
          :key="item.id"
          type="button"
          :class="{ on: s.lyricsSource === item.id }"
          @click="ui.set('lyricsSource', item.id)"
        >
          {{ item.label }}
        </button>
      </div>
      <div class="lset-hint">
        Авто: сначала LRCLIB (синхрон), потом Genius. Разборы с Genius
        подтягиваются к синхронному тексту LRCLIB. Кнопки вверху меняют источник
        только для текущего трека.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type {
  LyricsAlign,
  LyricsAnnotationMark,
  LyricsBackdrop,
  LyricsFont,
  LyricsHighlight,
  LyricsSource,
} from "@/stores/ui/defaults";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();
const s = computed(() => ui.settings);

type NumericKey =
  | "lyricsFontSize"
  | "lyricsLineHeight"
  | "lyricsWeight"
  | "lyricsInactive"
  | "lyricsLineBlur"
  | "lyricsBackgroundBlur"
  | "lyricsBackgroundOpacity";

function onNum(key: NumericKey, event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  if (Number.isFinite(value)) ui.set(key, value);
}

const fonts: Array<{ id: LyricsFont; label: string }> = [
  { id: "sans", label: "Гротеск" },
  { id: "serif", label: "Антиква" },
  { id: "mono", label: "Моно" },
  { id: "custom", label: "Свой" },
];

function onFont(event: Event) {
  const value = (event.target as HTMLInputElement).value.trim();
  if (value) ui.set("lyricsFontCustom", value);
}

type LocalFont = { family: string };
type FontHost = Window & {
  queryLocalFonts?: () => Promise<LocalFont[]>;
};

const PROBE_FAMILIES = [
  "Arial",
  "Arial Black",
  "Bahnschrift",
  "Calibri",
  "Cambria",
  "Candara",
  "Cascadia Code",
  "Cascadia Mono",
  "Century Gothic",
  "Comic Sans MS",
  "Consolas",
  "Constantia",
  "Corbel",
  "Courier New",
  "Ebrima",
  "Franklin Gothic",
  "Gabriola",
  "Gadugi",
  "Garamond",
  "Georgia",
  "Helvetica",
  "Impact",
  "Inter",
  "JetBrains Mono",
  "Lato",
  "Leelawadee UI",
  "Lucida Console",
  "Lucida Sans Unicode",
  "Malgun Gothic",
  "Manrope",
  "Menlo",
  "Meiryo",
  "Microsoft Sans Serif",
  "Montserrat",
  "MS Gothic",
  "Nirmala UI",
  "Nunito",
  "Open Sans",
  "Palatino Linotype",
  "PT Sans",
  "PT Serif",
  "Roboto",
  "Roboto Mono",
  "Rubik",
  "Segoe Print",
  "Segoe Script",
  "Segoe UI",
  "Segoe UI Emoji",
  "Segoe UI Semibold",
  "SF Pro Display",
  "Sitka",
  "Source Sans Pro",
  "Sylfaen",
  "Tahoma",
  "Times New Roman",
  "Trebuchet MS",
  "Verdana",
  "Yu Gothic",
];

const fontQuery = ref("");
const localFonts = ref<string[]>([]);
const fontsLoading = ref(false);
const fontsNotice = ref("");
const needsPermission = ref(false);

function onFontQuery(event: Event) {
  fontQuery.value = (event.target as HTMLInputElement).value;
}

function probeFamilies(): string[] {
  if (typeof document === "undefined" || !document.fonts?.check) return [];
  return PROBE_FAMILIES.filter((family) => {
    try {
      return document.fonts.check(`16px '${family}'`);
    } catch {
      return false;
    }
  });
}

function applyFallback(reason: string) {
  const probed = probeFamilies();
  localFonts.value = probed;
  needsPermission.value = true;
  fontsNotice.value = probed.length
    ? reason
    : `${reason} Впиши название шрифта вручную.`;
}

async function loadFonts(force = false) {
  if (fontsLoading.value) return;
  if (!force && localFonts.value.length) return;
  fontsLoading.value = true;
  fontsNotice.value = "";
  try {
    const query = (window as FontHost).queryLocalFonts;
    if (typeof query !== "function") {
      applyFallback(
        "Системный список шрифтов недоступен — показываю то, что удалось найти.",
      );
      return;
    }
    const found = await query.call(window);
    const families = [...new Set(found.map((item) => item.family))].sort(
      (a, b) => a.localeCompare(b, "ru"),
    );
    if (!families.length) {
      applyFallback("Система вернула пустой список шрифтов.");
      return;
    }
    localFonts.value = families;
    needsPermission.value = false;
    fontsNotice.value = "";
  } catch {
    applyFallback(
      "Доступ к шрифтам системы не выдан — показываю то, что удалось найти.",
    );
  } finally {
    fontsLoading.value = false;
  }
}

const visibleFonts = computed(() => {
  const needle = fontQuery.value.trim().toLowerCase();
  const list = needle
    ? localFonts.value.filter((name) => name.toLowerCase().includes(needle))
    : localFonts.value;
  return list.slice(0, 120);
});

function pickFont(name: string) {
  ui.set("lyricsFontCustom", name);
  fontQuery.value = "";
}

watch(
  () => s.value.lyricsFont,
  (font) => {
    if (font === "custom") void loadFonts();
  },
  { immediate: true },
);

const aligns: Array<{ id: LyricsAlign; label: string }> = [
  { id: "left", label: "Слева" },
  { id: "center", label: "По центру" },
  { id: "right", label: "Справа" },
];

const highlights: Array<{ id: LyricsHighlight; label: string }> = [
  { id: "white", label: "Белая" },
  { id: "accent", label: "Акцент" },
  { id: "karaoke", label: "Караоке" },
];

const backdrops: Array<{ id: LyricsBackdrop; label: string }> = [
  { id: "cover", label: "Обложка" },
  { id: "gradient", label: "Градиент" },
  { id: "solid", label: "Без фона" },
];

const marks: Array<{ id: LyricsAnnotationMark; label: string }> = [
  { id: "underline", label: "Подчёркивание" },
  { id: "tint", label: "Заливка" },
  { id: "dot", label: "Значок" },
  { id: "off", label: "Без метки" },
];

const lyricsSources: Array<{ id: LyricsSource; label: string }> = [
  { id: "auto", label: "Авто" },
  { id: "lrclib", label: "LRCLIB" },
  { id: "genius", label: "Genius" },
];
</script>
