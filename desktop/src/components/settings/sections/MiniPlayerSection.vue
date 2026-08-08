<template>
  <section id="settings-mini" class="settings-group">
    <div class="settings-group-head">
      <h2>Мини-плеер</h2>
      <p>
        Компактное окно поверх других приложений настраивается здесь, в большом
        окне. Изменения сохраняются сразу и видны в предпросмотре.
      </p>
    </div>

    <MiniLayoutEditor />

    <div class="setting-row">
      <div class="setting-copy">
        <b>Сбросить вид мини-плеера</b>
        <span>Размер окна, отступы и прозрачность вернутся к исходным.</span>
      </div>
      <button
        class="settings-reset-button"
        type="button"
        @click="ui.resetMiniLayout()"
      >
        Сбросить
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import MiniLayoutEditor from "@/components/player/MiniLayoutEditor.vue";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();
</script>
