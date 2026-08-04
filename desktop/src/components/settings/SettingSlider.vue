<template>
  <div class="setting-row setting-slider-row">
    <div class="setting-copy">
      <b>{{ label }}</b>
      <span>{{ description }}</span>
    </div>
    <div class="setting-slider-control">
      <span class="setting-number">{{ displayValue }}</span>
      <q-slider
        :model-value="modelValue"
        :min="min"
        :max="max"
        :step="step"
        @update:model-value="
          emit('update:modelValue', Number($event ?? modelValue))
        "
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  label: string;
  description: string;
  modelValue: number;
  min: number;
  max: number;
  step: number;
  suffix?: string;
}>();
const emit = defineEmits<{ "update:modelValue": [number] }>();
const displayValue = computed(() => `${props.modelValue}${props.suffix || ""}`);
</script>
