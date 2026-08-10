<template>
  <section id="settings-lyrics" class="settings-group">
    <div class="settings-group-head">
      <h2>Текст песни</h2>
      <p>
        Всё то же есть в самой панели текста - кнопка с шестерёнкой справа
        сверху.
      </p>
    </div>

    <div class="setting-row setting-row-column">
      <div class="setting-copy">
        <b>Источник текста</b>
        <span>
          «Авто» - сначала LRCLIB (там чаще есть синхронный текст), потом
          Genius. Яндекс не спрашивается, пока его не выберешь вручную. В самой
          панели источник переключается для одного трека.
        </span>
      </div>
      <div class="settings-choice">
        <button
          v-for="item in sources"
          :key="item.value"
          :class="{ on: ui.settings.lyricsSource === item.value }"
          @click="ui.set('lyricsSource', item.value)"
        >
          {{ item.label }}
        </button>
      </div>
    </div>

    <div class="settings-subgroup-head">Шрифт и строки</div>

    <div class="setting-row">
      <div class="setting-copy">
        <b>Гарнитура</b><span>Каким шрифтом набран текст песни.</span>
      </div>
      <div class="settings-choice">
        <button
          v-for="item in fonts"
          :key="item.value"
          :class="{ on: ui.settings.lyricsFont === item.value }"
          @click="ui.set('lyricsFont', item.value)"
        >
          {{ item.label }}
        </button>
      </div>
    </div>

    <div v-if="ui.settings.lyricsFont === 'custom'" class="setting-row">
      <div class="setting-copy">
        <b>Свой шрифт</b>
        <span>
          Имя шрифта, установленного в системе. Можно перечислить несколько
          через запятую - возьмётся первый доступный.
        </span>
      </div>
      <input
        class="settings-input"
        type="text"
        placeholder="Например: Inter"
        :value="ui.settings.lyricsFontCustom"
        @change="onCustomFont"
      />
    </div>

    <div class="setting-row">
      <div class="setting-copy">
        <b>Выравнивание</b><span>Положение строк в области текста.</span>
      </div>
      <div class="settings-choice">
        <button
          v-for="item in aligns"
          :key="item.value"
          :class="{ on: ui.settings.lyricsAlign === item.value }"
          @click="ui.set('lyricsAlign', item.value)"
        >
          {{ item.label }}
        </button>
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-copy">
        <b>Активная строка</b>
        <span>
          «Караоке» заливает строку по ходу песни - нужен синхронный текст.
        </span>
      </div>
      <div class="settings-choice">
        <button
          v-for="item in highlights"
          :key="item.value"
          :class="{ on: ui.settings.lyricsHighlight === item.value }"
          @click="ui.set('lyricsHighlight', item.value)"
        >
          {{ item.label }}
        </button>
      </div>
    </div>

    <SettingSlider
      label="Размер строк"
      description="Размер текста песни."
      :model-value="ui.settings.lyricsFontSize"
      :min="18"
      :max="72"
      :step="1"
      suffix=" px"
      @update:model-value="ui.set('lyricsFontSize', $event)"
    />

    <SettingSlider
      label="Интерлиньяж"
      description="Расстояние между строками."
      :model-value="ui.settings.lyricsLineHeight"
      :min="1"
      :max="1.8"
      :step="0.02"
      suffix=""
      @update:model-value="ui.set('lyricsLineHeight', $event)"
    />

    <SettingSlider
      label="Жирность"
      description="Насыщенность шрифта активной строки."
      :model-value="ui.settings.lyricsWeight"
      :min="400"
      :max="900"
      :step="50"
      suffix=""
      @update:model-value="ui.set('lyricsWeight', $event)"
    />

    <SettingSlider
      label="Яркость остальных строк"
      description="Насколько видны строки вокруг активной."
      :model-value="ui.settings.lyricsInactive"
      :min="8"
      :max="70"
      :step="1"
      suffix="%"
      @update:model-value="ui.set('lyricsInactive', $event)"
    />

    <SettingSlider
      label="Размытие строк"
      description="Размытие строк вокруг активной."
      :model-value="ui.settings.lyricsLineBlur"
      :min="0"
      :max="8"
      :step="0.5"
      suffix=" px"
      @update:model-value="ui.set('lyricsLineBlur', $event)"
    />

    <div class="settings-subgroup-head">Фон и эффекты</div>

    <div class="setting-row">
      <div class="setting-copy">
        <b>Фон панели</b><span>Что показывать за текстом.</span>
      </div>
      <div class="settings-choice">
        <button
          v-for="item in backdrops"
          :key="item.value"
          :class="{ on: ui.settings.lyricsBackdrop === item.value }"
          @click="ui.set('lyricsBackdrop', item.value)"
        >
          {{ item.label }}
        </button>
      </div>
    </div>

    <SettingSlider
      label="Размытие фона"
      description="Размытие обложки за текстом."
      :model-value="ui.settings.lyricsBackgroundBlur"
      :min="0"
      :max="90"
      :step="1"
      suffix=" px"
      @update:model-value="ui.set('lyricsBackgroundBlur', $event)"
    />

    <SettingSlider
      label="Яркость фона"
      description="Насколько виден фон."
      :model-value="ui.settings.lyricsBackgroundOpacity"
      :min="0"
      :max="100"
      :step="1"
      suffix="%"
      @update:model-value="ui.set('lyricsBackgroundOpacity', $event)"
    />

    <SettingToggle
      :model-value="ui.settings.lyricsShowArtwork"
      label="Большая обложка"
      description="Показывать обложку рядом с текстом."
      @update:model-value="ui.set('lyricsShowArtwork', $event)"
    />

    <SettingToggle
      :model-value="ui.settings.lyricsGlow"
      label="Свечение активной строки"
      description="Мягкий свет вокруг текущей строки."
      @update:model-value="ui.set('lyricsGlow', $event)"
    />

    <SettingToggle
      :model-value="ui.settings.lyricsMotion"
      label="Анимация строк"
      description="Плавно прокручивать и увеличивать активную строку."
      @update:model-value="ui.set('lyricsMotion', $event)"
    />

    <SettingToggle
      :model-value="ui.settings.lyricsShowCredits"
      label="Участники записи"
      description="Показывать продюсеров и авторов с Genius в подвале панели текста."
      @update:model-value="ui.set('lyricsShowCredits', $event)"
    />

    <SettingToggle
      :model-value="ui.settings.lyricsShowOrigin"
      label="Подпись «Источник»"
      description="Строка с названием сервиса, откуда загружен текст. Если выключить её и участников записи, подвал исчезнет."
      @update:model-value="ui.set('lyricsShowOrigin', $event)"
    />

    <div class="settings-subgroup-head">Разборы Genius</div>

    <SettingToggle
      :model-value="ui.settings.lyricsAnnotations"
      label="Разборы строк"
      description="Отмечать строки, у которых на Genius есть аннотация. Работает и с синхронным текстом из LRCLIB."
      @update:model-value="ui.set('lyricsAnnotations', $event)"
    />

    <div v-if="ui.settings.lyricsAnnotations" class="setting-row">
      <div class="setting-copy">
        <b>Как отмечать</b>
        <span>
          Левый клик по строке перематывает трек, правый - показывает значок
          разбора, по нему откроется аннотация.
        </span>
      </div>
      <div class="settings-choice">
        <button
          v-for="item in marks"
          :key="item.value"
          :class="{ on: ui.settings.lyricsAnnotationMark === item.value }"
          @click="ui.set('lyricsAnnotationMark', item.value)"
        >
          {{ item.label }}
        </button>
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-copy">
        <b>Сбросить вид текста</b>
        <span>Шрифт, фон, эффекты и источник вернутся к исходным.</span>
      </div>
      <button
        class="settings-reset-button"
        type="button"
        @click="ui.resetLyrics()"
      >
        Сбросить
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import SettingSlider from "@/components/settings/SettingSlider.vue";
import SettingToggle from "@/components/settings/SettingToggle.vue";
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

const sources: Array<{ value: LyricsSource; label: string }> = [
  { value: "auto", label: "Авто" },
  { value: "lrclib", label: "LRCLIB" },
  { value: "genius", label: "Genius" },
  { value: "yandex", label: "Яндекс" },
];

const fonts: Array<{ value: LyricsFont; label: string }> = [
  { value: "sans", label: "Гротеск" },
  { value: "serif", label: "Антиква" },
  { value: "mono", label: "Моно" },
  { value: "custom", label: "Свой" },
];

function onCustomFont(event: Event) {
  ui.set("lyricsFontCustom", (event.target as HTMLInputElement).value.trim());
}

const aligns: Array<{ value: LyricsAlign; label: string }> = [
  { value: "left", label: "Слева" },
  { value: "center", label: "По центру" },
  { value: "right", label: "Справа" },
];

const highlights: Array<{ value: LyricsHighlight; label: string }> = [
  { value: "white", label: "Белая" },
  { value: "accent", label: "Акцент" },
  { value: "karaoke", label: "Караоке" },
];

const backdrops: Array<{ value: LyricsBackdrop; label: string }> = [
  { value: "cover", label: "Обложка" },
  { value: "gradient", label: "Градиент" },
  { value: "solid", label: "Без фона" },
];

const marks: Array<{ value: LyricsAnnotationMark; label: string }> = [
  { value: "underline", label: "Подчёркивание" },
  { value: "tint", label: "Заливка" },
  { value: "dot", label: "Значок" },
  { value: "off", label: "Без метки" },
];
</script>
