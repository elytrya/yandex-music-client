<template>
  <div class="together-log">
    <div class="together-log-head">
      <b>Журнал</b>
      <span v-if="together.logPath">{{ together.logPath }}</span>
    </div>

    <pre ref="box" class="together-log-body">{{
      together.log.join("\n") || "Пока пусто"
    }}</pre>

    <div class="together-log-actions">
      <button class="settings-reset-button" type="button" @click="copy">
        Скопировать
      </button>
      <button
        class="settings-reset-button danger"
        type="button"
        @click="together.clearLog()"
      >
        Очистить
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { Notify } from "quasar";
import { useTogetherStore } from "@/stores/together/index";

const together = useTogetherStore();
const box = ref<HTMLElement | null>(null);

watch(
  () => together.log.length,
  async () => {
    await nextTick();
    if (box.value) box.value.scrollTop = box.value.scrollHeight;
  },
);

async function copy() {
  try {
    await navigator.clipboard.writeText(together.log.join("\n"));
    Notify.create({ message: "Журнал скопирован" });
  } catch {
    Notify.create({ message: "Не удалось скопировать" });
  }
}
</script>

<style scoped>
.together-log {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.together-log-head {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.together-log-head span {
  opacity: 0.55;
  font-size: 11px;
  word-break: break-all;
}

.together-log-body {
  margin: 0;
  max-height: 220px;
  overflow: auto;
  padding: 10px 12px;
  border-radius: 10px;
  background: var(--surface-2, rgba(255, 255, 255, 0.06));
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.together-log-actions {
  display: flex;
  gap: 8px;
}
</style>
