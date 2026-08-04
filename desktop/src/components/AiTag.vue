<template>
  <span v-if="show" class="ai-tag">
    AI
    <q-tooltip>Помечено как AI-музыка (данные Slopless)</q-tooltip>
  </span>
</template>

<script setup lang="ts">
defineProps<{ show?: boolean }>();
</script>

<style scoped>
.ai-tag {
  display: inline-flex;
  align-items: center;
  margin-left: 6px;
  padding: 0 5px;
  border: 1px solid rgb(239 68 68);
  border-radius: 4px;
  background: rgba(239, 68, 68, 0.12);
  color: rgb(239, 68, 68);
  font-size: 10px;
  font-weight: 600;
  line-height: 1.5;
  letter-spacing: 0.02em;
  vertical-align: middle;
  flex: 0 0 auto;
  cursor: default;
}
</style>
