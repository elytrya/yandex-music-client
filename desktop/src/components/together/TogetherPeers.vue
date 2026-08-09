<template>
  <div class="together-peers">
    <div v-for="peer in peers" :key="peer.id" class="together-peer">
      <span>{{ peer.nick }}</span>
      <b v-if="peer.id === 0">ведёт</b>
    </div>

    <p v-if="!peers.length" class="together-empty">Пока никого нет</p>
  </div>
</template>

<script setup lang="ts">
import type { TogetherPeer } from "@/api/together";

defineProps<{ peers: TogetherPeer[] }>();
</script>

<style scoped>
.together-peers {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.together-peer {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  border-radius: 999px;
  background: var(--surface-2, rgba(255, 255, 255, 0.06));
  font-size: 13px;
}

.together-peer b {
  opacity: 0.6;
  font-weight: 500;
  font-size: 11px;
  text-transform: uppercase;
}

.together-empty {
  margin: 0;
  opacity: 0.6;
  font-size: 13px;
}
</style>
