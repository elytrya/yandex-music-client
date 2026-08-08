<template>
  <button
    type="button"
    class="ed-switch"
    :class="{ on: modelValue }"
    @click="emit('update:modelValue', !modelValue)"
  >
    <span class="ed-switch-label">{{ label }}</span>
    <span class="ed-switch-track"><i /></span>
  </button>
</template>

<script setup lang="ts">
defineProps<{ label: string; modelValue: boolean }>();

const emit = defineEmits<{
  (e: "update:modelValue", value: boolean): void;
}>();
</script>

<style scoped>
.ed-switch {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 6px 8px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--fg);
  cursor: pointer;
  font: inherit;
  font-size: 12px;
  text-align: left;
  transition: background 0.14s ease;
}

.ed-switch:hover {
  background: var(--hover);
}

.ed-switch-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ed-switch-track {
  position: relative;
  width: 28px;
  height: 16px;
  flex: 0 0 auto;
  border-radius: 999px;
  background: color-mix(in srgb, var(--fg) 18%, transparent);
  transition: background 0.16s ease;
}

.ed-switch-track i {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--fg);
  opacity: 0.75;
  transition:
    transform 0.16s cubic-bezier(0.22, 0.65, 0.28, 1),
    opacity 0.16s ease;
}

.ed-switch.on .ed-switch-track {
  background: color-mix(in srgb, var(--accent) 55%, transparent);
}

.ed-switch.on .ed-switch-track i {
  transform: translateX(12px);
  background: #fff;
  opacity: 1;
}
</style>
