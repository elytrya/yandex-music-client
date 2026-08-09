<template>
  <div v-if="together.active" class="together-dock" :class="{ open }">
    <button class="together-dock-head" type="button" @click="toggle">
      <span class="together-dock-dot" :class="{ wait: waiting }" />

      <span class="together-dock-title">
        {{ together.isHost ? "Вы ведёте" : `Слушаете: ${together.hostNick}` }}
      </span>

      <span class="together-dock-count">{{ together.peers.length }}</span>
      <Icon :name="open ? 'minimize' : 'expand'" :size="12" />
    </button>

    <div v-if="open" class="together-dock-body">
      <div v-if="together.isHost" class="together-dock-row">
        <span class="together-dock-label">Адрес</span>
        <code>{{ together.invite || `ваш-ip:${together.port}` }}</code>
        <button class="together-dock-icon" type="button" @click="copyInvite">
          <Icon name="copy" :size="12" />
        </button>
      </div>

      <div class="together-dock-row">
        <span class="together-dock-label">Сейчас</span>
        <span class="together-dock-value">{{ nowPlaying }}</span>
      </div>

      <TogetherPeers
        :peers="together.peers"
        :waiting="together.waiting"
        :controllers="together.controllers"
        :manage="together.isHost"
        @grant="together.grant"
      />

      <p v-if="together.waitingNicks.length" class="together-dock-wait">
        Ждём загрузку: {{ together.waitingNicks.join(", ") }}
      </p>

      <p v-else-if="!together.isHost" class="together-dock-hint">
        {{
          together.rights ? "У вас есть управление комнатой" : "Управляет хост"
        }}
      </p>

      <div class="together-dock-actions">
        <button type="button" @click="openSettings">Настройки</button>
        <button
          type="button"
          class="danger"
          :disabled="together.busy"
          @click="together.leave()"
        >
          {{ together.isHost ? "Закрыть" : "Выйти" }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Notify } from "quasar";
import Icon from "@/components/Icon.vue";
import TogetherPeers from "@/components/together/TogetherPeers.vue";
import { usePlayerStore } from "@/stores/player/index";
import { useTogetherStore } from "@/stores/together/index";
import { DOCK_KEY } from "@/stores/together/protocol";

const router = useRouter();
const together = useTogetherStore();
const player = usePlayerStore();

const open = ref(false);

const waiting = computed(() => together.waiting.length > 0);

const nowPlaying = computed(() =>
  player.current ? player.current.title : "ничего не играет",
);

onMounted(() => {
  void together.init();
  try {
    open.value = localStorage.getItem(DOCK_KEY) === "1";
  } catch {}
});

function toggle() {
  open.value = !open.value;
  try {
    localStorage.setItem(DOCK_KEY, open.value ? "1" : "0");
  } catch {}
}

async function copyInvite() {
  try {
    await navigator.clipboard.writeText(together.invite);
    Notify.create({ message: "Адрес скопирован" });
  } catch {
    Notify.create({ message: "Не удалось скопировать" });
  }
}

function openSettings() {
  void router.push({ path: "/settings", query: { section: "together" } });
}
</script>

<style scoped>
.together-dock {
  position: fixed;
  z-index: 2400;
  bottom: 96px;
  left: 16px;
  width: 268px;
  overflow: hidden;
  border: 1px solid var(--line);
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: 0 18px 40px var(--shadow-soft);
  user-select: none;
}

.together-dock.open {
  width: 320px;
}

.together-dock-head {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border: 0;
  background: transparent;
  color: var(--fg);
  font: inherit;
  font-size: 13px;
  cursor: pointer;
}

.together-dock-head:hover {
  background: var(--hover);
}

.together-dock-dot {
  width: 8px;
  height: 8px;
  flex: 0 0 auto;
  border-radius: 50%;
  background: #3ecf6b;
}

.together-dock-dot.wait {
  background: var(--accent);
}

.together-dock-title {
  flex: 1 1 auto;
  overflow: hidden;
  text-align: left;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.together-dock-count {
  padding: 1px 7px;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--fg-dim);
  font-size: 11px;
}

.together-dock-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 4px 12px 12px;
  border-top: 1px solid var(--line);
}

.together-dock-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-top: 8px;
  font-size: 12px;
}

.together-dock-label {
  flex: 0 0 auto;
  color: var(--fg-faint);
}

.together-dock-row code {
  flex: 1 1 auto;
  overflow: hidden;
  color: var(--fg);
  text-overflow: ellipsis;
  white-space: nowrap;
  user-select: all;
}

.together-dock-value {
  flex: 1 1 auto;
  overflow: hidden;
  color: var(--fg-dim);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.together-dock-icon {
  display: inline-flex;
  padding: 3px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--fg-dim);
  cursor: pointer;
}

.together-dock-icon:hover {
  background: var(--hover);
  color: var(--fg);
}

.together-dock-wait {
  margin: 0;
  color: var(--accent);
  font-size: 12px;
}

.together-dock-hint {
  margin: 0;
  color: var(--fg-faint);
  font-size: 12px;
}

.together-dock-actions {
  display: flex;
  gap: 8px;
}

.together-dock-actions button {
  flex: 1 1 0;
  padding: 7px 10px;
  border: 1px solid var(--line);
  border-radius: calc(var(--radius) * 0.7);
  background: var(--surface-2);
  color: var(--fg);
  font: inherit;
  font-size: 12px;
  cursor: pointer;
}

.together-dock-actions button:hover {
  background: var(--hover);
}

.together-dock-actions button.danger {
  color: var(--accent);
}
</style>
