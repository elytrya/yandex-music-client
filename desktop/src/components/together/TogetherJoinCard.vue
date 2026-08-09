<template>
  <div class="setting-row">
    <div class="setting-copy">
      <b>Подключиться к комнате</b>
      <span>
        Адрес хоста в виде ip или ip:порт. Ваш плеер начнёт повторять трек и
        позицию хоста.
      </span>
    </div>

    <div class="together-controls">
      <input
        v-model="address"
        class="together-input"
        type="text"
        placeholder="26.13.4.7:7331"
        :disabled="together.active"
        @keyup.enter="connect"
      />

      <button
        v-if="together.mode !== 'guest'"
        class="settings-reset-button"
        type="button"
        :disabled="together.busy || together.active || !address.trim()"
        @click="connect"
      >
        Подключиться
      </button>

      <button
        v-else
        class="settings-reset-button danger"
        type="button"
        :disabled="together.busy"
        @click="together.leave()"
      >
        Отключиться
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useTogetherStore } from "@/stores/together/index";

const together = useTogetherStore();
const address = ref("");

function connect() {
  const value = address.value.trim();
  if (!value || together.busy || together.active) return;
  void together.join(value);
}
</script>

<style scoped>
.together-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.together-input {
  width: 190px;
  padding: 7px 10px;
  border-radius: 8px;
  border: 1px solid var(--border, rgba(255, 255, 255, 0.12));
  background: transparent;
  color: inherit;
  font: inherit;
}
</style>
