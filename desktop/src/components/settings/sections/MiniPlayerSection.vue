<template>
  <section id="settings-mini" class="settings-group">
    <div class="settings-group-head">
      <h2>Мини-плеер</h2>
      <p>Кнопки и полоса времени в компактном окне поверх других приложений.</p>
    </div>

    <div class="setting-row setting-row-column preview-row">
      <div class="setting-copy">
        <b>Предпросмотр мини-плеера</b>
        <span>Управление всегда по центру.</span>
      </div>

      <div class="mini-preview">
        <div class="mini-preview-top">
          <div class="mini-preview-cover">
            <Icon name="note" :size="14" class="faint" />
          </div>
          <div class="player-preview-meta">
            <span class="player-preview-line w-70" />
            <span class="player-preview-line w-45 dimmed" />
          </div>
          <span class="player-preview-btn"
            ><Icon name="restore" :size="13"
          /></span>
        </div>

        <div class="mini-preview-controls">
          <span
            v-for="id in miniActive"
            :key="id"
            class="player-preview-btn"
            :class="{ accent: id === 'play' }"
            :title="miniLabelOf(id)"
          >
            <Icon :name="miniIconOf(id)" :size="14" />
          </span>
        </div>

        <div class="mini-preview-seek">
          <span v-if="ui.settings.miniShowTime" class="player-preview-tick">
            0:42
          </span>
          <span class="player-preview-track"><i /></span>
          <span v-if="ui.settings.miniShowTime" class="player-preview-tick">
            3:15
          </span>
        </div>
      </div>
    </div>

    <div class="setting-row setting-row-column">
      <div class="setting-copy">
        <b>Порядок кнопок</b>
        <span>
          Перетащи кнопку на нужное место. Видимость переключается тумблерами
          ниже.
        </span>
      </div>

      <div class="mini-dnd">
        <div
          v-for="(id, index) in miniOrder"
          :key="id"
          class="dnd-chip"
          :class="{
            dragging: dragId === id,
            hidden: !isVisible(id),
            locked: lockedMini.has(id),
          }"
          :draggable="true"
          :title="miniLabelOf(id)"
          @dragstart="dragId = id"
          @dragend="dragId = null"
          @dragover.prevent
          @drop.prevent="dropAt(index)"
        >
          <Icon name="drag" :size="12" class="faint" />
          <Icon :name="miniIconOf(id)" :size="14" />
          <span class="ellipsis">{{ miniLabelOf(id) }}</span>
          <Icon v-if="!isVisible(id)" name="close" :size="11" class="faint" />
        </div>
      </div>
    </div>

    <div v-for="item in miniCatalog" :key="item.id" class="setting-row">
      <div class="setting-copy setting-copy-icon">
        <span class="setting-glyph"><Icon :name="item.icon" :size="15" /></span>
        <span class="setting-copy-text">
          <b>{{ item.label }}</b>
          <span>Показывать в мини-плеере</span>
        </span>
      </div>
      <div
        class="setting-switch"
        :class="{ on: ui.settings.miniButtons[item.id] }"
        @click="ui.setMiniButton(item.id, !ui.settings.miniButtons[item.id])"
      >
        <i />
      </div>
    </div>

    <SettingToggle
      label="Время в мини-плеере"
      description="Текущая позиция и длительность по краям полосы прокрутки."
      :model-value="ui.settings.miniShowTime"
      @update:model-value="ui.set('miniShowTime', $event)"
    />

    <SettingToggle
      label="Визуализатор"
      description="Живой спектр звука на весь мини-плеер. Требует воспроизведения."
      :model-value="ui.settings.miniVisualizer"
      @update:model-value="ui.set('miniVisualizer', $event)"
    />

    <SettingToggle
      label="Вертикальная громкость"
      description="При наведении на кнопку громкости разворачивается ползунок вверх. Колёсико мыши тоже меняет громкость."
      :model-value="ui.settings.miniVolumeSlider"
      @update:model-value="ui.set('miniVolumeSlider', $event)"
    />

    <SettingSlider
      v-if="ui.settings.miniVolumeSlider"
      label="Длина ползунка громкости"
      description="Высота вертикальной шкалы."
      :model-value="ui.settings.miniVolumeHeight"
      :min="48"
      :max="200"
      :step="4"
      suffix=" px"
      @update:model-value="ui.set('miniVolumeHeight', $event)"
    />

    <SettingSlider
      label="Прозрачность окна"
      description="Насколько мини-плеер просвечивает."
      :model-value="ui.settings.miniOpacity"
      :min="40"
      :max="100"
      :step="1"
      suffix=" %"
      @update:model-value="ui.set('miniOpacity', $event)"
    />

    <SettingSlider
      label="Размер обложки"
      description="Квадрат с обложкой в шапке."
      :model-value="ui.settings.miniCoverSize"
      :min="24"
      :max="64"
      :step="1"
      suffix=" px"
      @update:model-value="ui.set('miniCoverSize', $event)"
    />

    <SettingSlider
      label="Размер иконок"
      description="Величина кнопок управления."
      :model-value="ui.settings.miniIconSize"
      :min="11"
      :max="24"
      :step="1"
      suffix=" px"
      @update:model-value="ui.set('miniIconSize', $event)"
    />

    <SettingSlider
      label="Расстояние между элементами"
      description="Промежутки между блоками и кнопками."
      :model-value="ui.settings.miniGap"
      :min="0"
      :max="20"
      :step="1"
      suffix=" px"
      @update:model-value="ui.set('miniGap', $event)"
    />

    <SettingSlider
      label="Внутренние отступы"
      description="Поля от края окна до содержимого."
      :model-value="ui.settings.miniPadding"
      :min="2"
      :max="28"
      :step="1"
      suffix=" px"
      @update:model-value="ui.set('miniPadding', $event)"
    />

    <div class="setting-row">
      <div class="setting-copy">
        <b>Сбросить настройки мини-плеера</b>
        <span>Размеры, прозрачность и громкость вернутся к исходным.</span>
      </div>
      <button
        class="settings-reset-button"
        type="button"
        @click="ui.resetMiniLayout()"
      >
        Сбросить
      </button>
    </div>

    <div class="setting-row">
      <div class="setting-copy">
        <b>Сбросить раскладку</b>
        <span
          >Вернуть кнопки плеера и мини-плеера к значениям по умолчанию.</span
        >
      </div>
      <button
        class="settings-reset-button"
        type="button"
        @click="ui.resetPlayerButtons()"
      >
        Сбросить
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import Icon from "@/components/Icon.vue";
import SettingSlider from "@/components/settings/SettingSlider.vue";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import type { MiniButtonId } from "@/stores/ui/index";
import { miniButtonCatalog, useUiStore } from "@/stores/ui/index";

const lockedMini = new Set<MiniButtonId>(["prev", "play", "next"]);

const ui = useUiStore();

const miniCatalog = miniButtonCatalog.filter((item) => !item.locked);
const miniActive = computed(() => ui.activeMiniButtons());

/* --- Drag & drop порядка --- */

const miniOrder = computed(() => ui.miniOrderList());
const dragId = ref<MiniButtonId | null>(null);

function isVisible(id: MiniButtonId): boolean {
  return lockedMini.has(id) || !!ui.settings.miniButtons[id];
}

function dropAt(index: number) {
  const id = dragId.value;
  dragId.value = null;
  if (!id) return;
  ui.moveMiniButton(id, index);
}

const miniMetaOf = new Map(miniButtonCatalog.map((item) => [item.id, item]));

function miniIconOf(id: MiniButtonId): string {
  return miniMetaOf.get(id)?.icon ?? "note";
}

function miniLabelOf(id: MiniButtonId): string {
  return miniMetaOf.get(id)?.label ?? id;
}
</script>

<style scoped>
.mini-dnd {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  width: 100%;
}
.dnd-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--surface-2);
  font-size: 12px;
  cursor: grab;
}
.dnd-chip.dragging {
  opacity: 0.45;
}
.dnd-chip.hidden {
  opacity: 0.5;
  border-style: dashed;
}
.dnd-chip.locked {
  border-color: var(--accent);
}
</style>
