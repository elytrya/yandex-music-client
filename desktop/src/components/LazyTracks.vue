<template>
  <template v-for="(item, i) in visible" :key="`${item.id}-${i}`">
    <slot :item="item" :index="i" />
  </template>
  <div ref="sentinel" class="lazy-sentinel" />
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { Track } from "@/api/types";

const props = withDefaults(
  defineProps<{ items: Track[]; initial?: number; step?: number }>(),
  { initial: 40, step: 40 },
);

const count = ref(props.initial);
const sentinel = ref<HTMLElement | null>(null);
let observer: IntersectionObserver | null = null;

const visible = computed(() => props.items.slice(0, count.value));

function more() {
  if (count.value >= props.items.length) return;
  count.value = Math.min(props.items.length, count.value + props.step);
}

watch(
  () => props.items,
  () => {
    count.value = props.initial;
  },
);

onMounted(() => {
  if (!sentinel.value) return;
  observer = new IntersectionObserver(
    (entries) => {
      if (entries.some((entry) => entry.isIntersecting)) more();
    },
    { rootMargin: "500px 0px" },
  );
  observer.observe(sentinel.value);
});

onBeforeUnmount(() => {
  observer?.disconnect();
  observer = null;
});
</script>
