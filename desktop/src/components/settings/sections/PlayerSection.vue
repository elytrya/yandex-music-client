<template>
  <section id="settings-player" class="settings-group">
    <div class="settings-group-head">
      <h2>Плеер</h2>
      <p>
        Вид плеера настраивается прямо на самом плеере — без списков и табличек.
      </p>
    </div>

    <div class="setting-row setting-row-column editor-cta">
      <div class="setting-copy">
        <b>Режим редактирования</b>
        <span>
          Включи его — и меняй плеер живьём: всё обновляется сразу на экране.
        </span>
      </div>

      <ul class="editor-steps">
        <li>
          <Icon name="drag" :size="14" />
          Перетаскивай любую кнопку между левой, центральной и правой частями
        </li>
        <li>
          <Icon name="close" :size="14" />
          Лишнее тяни в корзину «Скрытые кнопки», оттуда же возвращай обратно
        </li>
        <li>
          <Icon name="expand" :size="14" />
          Верхний край тянет высоту, боковой — ширину блока с треком
        </li>
        <li>
          <Icon name="settings" :size="14" />
          Ползунки рядом — обложка, иконки, отступы, полоса прогресса
        </li>
        <li>
          <Icon name="check" :size="14" />
          «Готово» сохраняет, «Отмена» возвращает всё как было
        </li>
      </ul>

      <div class="editor-actions">
        <button
          class="btn accent"
          type="button"
          @click="ui.togglePlayerEdit(true)"
        >
          <Icon name="layout" :size="15" />
          Редактировать на плеере
        </button>
        <span class="editor-hint">
          Или просто кликни правой кнопкой мыши по пустому месту плеера.
        </span>
      </div>
    </div>

    <SettingToggle
      label="Визуализатор"
      description="Живой спектр звука фоном в нижней панели. Требует воспроизведения."
      :model-value="ui.settings.playerVisualizer"
      @update:model-value="ui.set('playerVisualizer', $event)"
    />

    <SettingToggle
      label="Показывать тайминги"
      description="Текущее время и длительность по краям полосы прогресса."
      :model-value="ui.settings.playerShowTimes"
      @update:model-value="ui.set('playerShowTimes', $event)"
    />

    <div class="setting-row">
      <div class="setting-copy">
        <b>Сбросить размеры плеера</b>
        <span>Высота, отступы и пропорции вернутся к исходным.</span>
      </div>
      <button
        class="settings-reset-button"
        type="button"
        @click="ui.resetPlayerLayout()"
      >
        Сбросить
      </button>
    </div>

    <div class="setting-row">
      <div class="setting-copy">
        <b>Сбросить раскладку кнопок</b>
        <span>Кнопки плеера и мини-плеера вернутся на свои места.</span>
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
import Icon from "@/components/Icon.vue";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();
</script>

<style scoped>
.editor-steps {
  display: flex;
  flex-direction: column;
  gap: 7px;
  width: 100%;
  margin: 4px 0 0;
  padding: 0;
  list-style: none;
}

.editor-steps li {
  display: flex;
  align-items: center;
  gap: 9px;
  color: var(--fg-dim);
  font-size: 12.5px;
  line-height: 1.4;
}

.editor-steps :deep(svg) {
  flex: 0 0 auto;
  color: var(--accent);
}

.editor-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px 14px;
  width: 100%;
  margin-top: 12px;
}

.editor-actions .btn {
  display: inline-flex;
  align-items: center;
  gap: 7px;
}

.editor-actions .btn.accent {
  border-color: var(--accent);
  background: var(--accent);
  color: #fff;
}

.editor-hint {
  color: var(--fg-faint);
  font-size: 12px;
}
</style>
