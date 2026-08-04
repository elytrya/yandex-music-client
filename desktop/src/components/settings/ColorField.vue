<template>
  <label class="color-field">
    <input type="color" :value="value" @input="onInput" />
    <span>{{ label }}</span>
    <code>{{ value.toUpperCase() }}</code>
  </label>
</template>

<script setup lang="ts">
defineProps<{ label: string; value: string }>();
const emit = defineEmits<{ change: [string] }>();

function onInput(event: Event) {
  emit("change", (event.target as HTMLInputElement).value);
}
</script>
