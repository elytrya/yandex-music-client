<template>
  <button
    type="button"
    class="setting-row setting-toggle-row"
    role="switch"
    :aria-checked="modelValue"
    @click="emit('update:modelValue', !modelValue)"
  >
    <span class="setting-copy">
      <b>{{ label }}</b>
      <span>{{ description }}</span>
    </span>
    <span class="setting-switch" :class="{ on: modelValue }"><i /></span>
  </button>
</template>

<script setup lang="ts">
defineProps<{ label: string; description: string; modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [boolean] }>();
</script>
