<template>
  <section id="settings-cache" class="settings-group">
    <div class="settings-group-head">
      <h2>Кеш</h2>
      <p>
        Страницы открываются из кеша мгновенно, а свежие данные подгружаются
        фоном.
      </p>
    </div>

    <SettingToggle
      :model-value="ui.settings.cacheEnabled"
      label="Кешировать данные"
      description="Плейлисты, треки, волна, тексты песен."
      @update:model-value="ui.set('cacheEnabled', $event)"
    />

    <div class="setting-row">
      <div class="setting-copy">
        <b>Занято</b><span>{{ size }} КБ локального кеша.</span>
      </div>
      <button class="settings-reset-button" @click="clear">Очистить кеш</button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Notify } from "quasar";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import { cacheSizeKb, clearCache } from "@/lib/cache";
import { useUiStore } from "@/stores/ui/index";

const ui = useUiStore();
const size = ref(cacheSizeKb());

function clear() {
  clearCache();
  size.value = cacheSizeKb();
  Notify.create({ message: "Кеш очищен" });
}
</script>
