<template>
  <div class="together-peers">
    <div v-for="peer in peers" :key="peer.id" class="together-peer">
      <span>{{ peer.nick }}</span>

      <b v-if="peer.id === hostId">хостит</b>
      <i v-if="waiting.includes(peer.id)">грузит</i>

      <button
        v-if="manage && peer.id !== hostId"
        class="together-give"
        type="button"
        title="Участник станет хостом, остальные переподключатся"
        @click="emit('handoff', peer.id)"
      >
        передать хост
      </button>
    </div>

    <p v-if="!peers.length" class="together-empty">Пока никого нет</p>
  </div>
</template>

<script setup lang="ts">
import type { TogetherPeer } from "@/api/together";

withDefaults(
  defineProps<{
    peers: TogetherPeer[];
    waiting?: number[];
    manage?: boolean;
    hostId?: number;
  }>(),
  {
    waiting: () => [],
    manage: false,
    hostId: 0,
  },
);

const emit = defineEmits<{ (e: "handoff", id: number): void }>();
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

.together-peer i {
  opacity: 0.75;
  font-style: normal;
  font-size: 11px;
  text-transform: uppercase;
  color: var(--accent, #ffcc00);
}

.together-give {
  padding: 0;
  border: 0;
  background: transparent;
  color: inherit;
  opacity: 0.55;
  font: inherit;
  font-size: 11px;
  cursor: pointer;
  text-decoration: underline;
}

.together-give:hover {
  opacity: 0.9;
}

.together-empty {
  margin: 0;
  opacity: 0.6;
  font-size: 13px;
}
</style>
