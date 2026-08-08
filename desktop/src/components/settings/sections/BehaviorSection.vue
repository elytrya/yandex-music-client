<template>
  <section id="settings-behavior" class="settings-group">
    <div class="settings-group-head">
      <h2>Поведение</h2>
      <p>
        Как приложение ведёт себя во время работы: внешние эффекты, окно,
        воспроизведение и фильтры контента.
      </p>
    </div>

    <div class="settings-subgroup">
      <div class="settings-subgroup-head">
        <h3>Визуальные эффекты</h3>
        <p>Влияют только на внешний вид. На звук не действуют.</p>
      </div>

      <SettingToggle
        :model-value="ui.settings.glass"
        label="Размытие панелей"
        description="Меню, заголовки и всплывающие панели становятся полупрозрачными, сквозь них видно фон. Выключи, если интерфейс подтормаживает."
        @update:model-value="ui.set('glass', $event)"
      />
      <SettingSlider
        v-if="ui.settings.glass"
        label="Сила размытия"
        description="0 px - панели почти прозрачные без размытия, 40 px - фон за панелью размывается полностью."
        :model-value="ui.settings.glassBlur"
        :min="0"
        :max="40"
        :step="2"
        suffix=" px"
        @update:model-value="ui.set('glassBlur', $event)"
      />
      <SettingToggle
        :model-value="ui.settings.animations"
        label="Анимации интерфейса"
        description="Плавные переходы между страницами, появление меню и движение элементов. Без них интерфейс переключается мгновенно."
        @update:model-value="ui.set('animations', $event)"
      />
      <SettingToggle
        :model-value="ui.settings.thinScrollbar"
        label="Тонкие полосы прокрутки"
        description="Узкие скроллбары во всех списках вместо системных широких."
        @update:model-value="ui.set('thinScrollbar', $event)"
      />
    </div>

    <div class="settings-subgroup">
      <div class="settings-subgroup-head">
        <h3>Обложки и миниатюры</h3>
        <p>Показывать ли картинки в боковой панели и в плеере.</p>
      </div>

      <SettingToggle
        :model-value="ui.settings.showPlaylistCovers"
        label="Обложки плейлистов в боковой панели"
        description="Рядом с названием каждого плейлиста показывается его миниатюра."
        @update:model-value="ui.set('showPlaylistCovers', $event)"
      />
      <SettingToggle
        :model-value="ui.settings.showPlayerArtwork"
        label="Обложка трека в нижней панели"
        description="Квадрат с обложкой слева от названия трека. По клику открывается альбом."
        @update:model-value="ui.set('showPlayerArtwork', $event)"
      />
    </div>

    <div class="settings-subgroup">
      <div class="settings-subgroup-head">
        <h3>Окно и запуск</h3>
        <p>Что происходит при закрытии окна и при следующем запуске.</p>
      </div>

      <SettingToggle
        :model-value="ui.settings.minimizeToTray"
        label="Закрытие сворачивает в трей"
        description="Крестик прячет окно в системный трей, музыка продолжает играть. Полностью выйти можно через меню трея."
        @update:model-value="ui.set('minimizeToTray', $event)"
      />
      <SettingToggle
        :model-value="ui.settings.resumeLastSession"
        label="Восстанавливать очередь после запуска"
        description="При старте приложение возвращает трек, очередь и позицию воспроизведения с прошлого раза."
        @update:model-value="ui.set('resumeLastSession', $event)"
      />
      <SettingToggle
        v-if="ui.settings.resumeLastSession"
        :model-value="ui.settings.resumeAutoplay"
        label="Сразу продолжать воспроизведение"
        description="Восстановленный трек заиграет автоматически с той же секунды. Если выключено - трек будет готов, но останется на паузе."
        @update:model-value="ui.set('resumeAutoplay', $event)"
      />
    </div>

    <div class="settings-subgroup">
      <div class="settings-subgroup-head">
        <h3>Воспроизведение</h3>
        <p>Как приложение обрабатывает сам звук и переходы между треками.</p>
      </div>

      <SettingToggle
        :model-value="ui.settings.autoSkipDisliked"
        label="Автоскип дизлайкнутых треков"
        description="Трек, отмеченный как «Не нравится», пропускается автоматически, когда доходит до него очередь. Сам трек остаётся в списке, но выглядит приглушённым."
        @update:model-value="ui.set('autoSkipDisliked', $event)"
      />
      <SettingToggle
        :model-value="ui.settings.trimSilence"
        label="Подрезать тишину в конце трека"
        description="Если последние секунды трека почти беззвучные, приложение не ждёт их и сразу включает следующий трек."
        @update:model-value="ui.set('trimSilence', $event)"
      />
      <div class="setting-row setting-row-column">
        <div class="setting-copy">
          <b>Когда очередь закончилась</b
          ><span>
            Сыграл последний трек списка - дальше или плейлист идёт по кругу,
            или включается волна, подобранная по этому же плейлисту.
          </span>
        </div>
        <div class="settings-choice">
          <button
            :class="{ on: queueEnd === 'stop' }"
            @click="setQueueEnd('stop')"
          >
            Остановиться
          </button>
          <button
            :class="{ on: queueEnd === 'repeat' }"
            @click="setQueueEnd('repeat')"
          >
            Повтор плейлиста
          </button>
          <button
            :class="{ on: queueEnd === 'wave' }"
            @click="setQueueEnd('wave')"
          >
            Моя волна по плейлисту
          </button>
        </div>
      </div>
    </div>

    <div class="settings-subgroup">
      <div class="settings-subgroup-head">
        <h3>Фильтры контента</h3>
        <p>Подмена зацензуренных версий и отношение к AI-музыке.</p>
      </div>

      <SettingToggle
        :model-value="ui.settings.censorBypass"
        label="Обход цензуры"
        description="Если у трека есть незацензуренная версия в открытой базе FckCensorData, приложение играет её вместо запиканной. Работает только для треков из этой базы, остальные играют как обычно."
        @update:model-value="ui.set('censorBypass', $event)"
      />
      <SettingToggle
        v-if="ui.settings.censorBypass"
        :model-value="ui.settings.censorBadge"
        label="Показывать пометку о подмене"
        description="Рядом с названием трека в плеере появляется метка «без цензуры», когда играет подменённая версия."
        @update:model-value="ui.set('censorBadge', $event)"
      />
      <SettingToggle
        :model-value="ui.settings.autoDislikeAi"
        label="Автодизлайк AI-треков"
        description="Трек, помеченный базой Slopless как сгенерированный нейросетью, автоматически получает дизлайк и пропускается."
        @update:model-value="ui.set('autoDislikeAi', $event)"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue";
import SettingSlider from "@/components/settings/SettingSlider.vue";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();

type QueueEnd = "stop" | "repeat" | "wave";

const queueEnd = computed<QueueEnd>(() => {
  if (ui.settings.repeatPlaylistAlways) return "repeat";
  return ui.settings.autoWaveOnQueueEnd ? "wave" : "stop";
});

function setQueueEnd(mode: QueueEnd) {
  ui.set("repeatPlaylistAlways", mode === "repeat");
  ui.set("autoWaveOnQueueEnd", mode === "wave");
  if (mode === "wave") ui.set("autoWaveSource", "playlist");
}
</script>

<style scoped>
.settings-subgroup {
  margin-top: 26px;
  padding-top: 18px;
  border-top: 1px solid var(--border, rgba(255, 255, 255, 0.08));
}
.settings-subgroup:first-of-type {
  margin-top: 18px;
}
.settings-subgroup-head {
  margin-bottom: 12px;
}
.settings-subgroup-head h3 {
  margin: 0 0 4px;
  font-size: 14px;
  font-weight: 600;
  line-height: 1.3;
}
.settings-subgroup-head p {
  margin: 0;
  font-size: 12px;
  line-height: 1.45;
  opacity: 0.6;
}
</style>
