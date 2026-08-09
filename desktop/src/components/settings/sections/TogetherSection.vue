<template>
  <section id="settings-together" class="settings-group">
    <div class="settings-group-head">
      <h2>Слушать вместе</h2>
      <p>
        Синхронное прослушивание по локальной сети или через любой vpn с общей
        подсетью: radmin vpn, hamachi, zerotier, tailscale, netbird. По сети
        передаётся только трек и позиция, звук каждый грузит из своего аккаунта.
      </p>
    </div>

    <div class="setting-row">
      <div class="setting-copy">
        <b>Имя в комнате</b>
        <span>Как вас увидят остальные участники.</span>
      </div>

      <input
        class="together-input"
        type="text"
        placeholder="слушатель"
        :value="together.nick"
        :disabled="together.active"
        @input="onNick"
      />
    </div>

    <div class="setting-row">
      <div class="setting-copy">
        <b>Как подключаемся</b>
        <span>Локально по сети или VPN, либо через сервер-ретранслятор без проброса портов.</span>
      </div>

      <div class="together-transport">
        <button
          type="button"
          :class="{ active: transport === 'local' }"
          :disabled="together.active"
          @click="setTransport('local')"
        >
          Локально
        </button>
        <button
          type="button"
          :class="{ active: transport === 'server' }"
          :disabled="together.active"
          @click="setTransport('server')"
        >
          Через сервер
        </button>
      </div>
    </div>

    <template v-if="transport === 'local'">
      <TogetherHostCard />
      <TogetherJoinCard />
    </template>

    <TogetherServerCard v-else />

    <div v-if="together.active" class="setting-row column">
      <div class="setting-copy">
        <b>{{ together.isHost ? "Вы ведёте" : "Вы слушаете хоста" }}</b>
        <span>{{ together.peers.length }} в комнате</span>
      </div>

      <TogetherPeers
        :peers="together.peers"
        :waiting="together.waiting"
        :manage="together.isHost"
        :host-id="together.hostId"
        @handoff="together.handoff"
      />

      <p v-if="together.waitingNicks.length" class="together-wait">
        Ждём загрузку: {{ together.waitingNicks.join(", ") }}
      </p>

      <p v-if="together.isHost" class="together-hint">
        Ведёт всегда хост. Если включать треки должен другой человек, передайте
        ему хост: он поднимет комнату у себя, остальные переподключатся сами.
      </p>

      <p v-else class="together-hint">
        Ведёт {{ together.hostNick }}, вы повторяете его плеер. Чтобы включать
        треки самому, попросите передать хост.
      </p>
    </div>

    <p v-if="together.error" class="together-error">{{ together.error }}</p>

    <TogetherLog />
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import TogetherHostCard from "@/components/together/TogetherHostCard.vue";
import TogetherJoinCard from "@/components/together/TogetherJoinCard.vue";
import TogetherServerCard from "@/components/together/TogetherServerCard.vue";
import TogetherLog from "@/components/together/TogetherLog.vue";
import TogetherPeers from "@/components/together/TogetherPeers.vue";
import { useTogetherStore } from "@/stores/together/index";
import { TRANSPORT_KEY } from "@/stores/together/protocol";

const together = useTogetherStore();

type Transport = "local" | "server";

function loadTransport(): Transport {
  try {
    return localStorage.getItem(TRANSPORT_KEY) === "local" ? "local" : "server";
  } catch {
    return "server";
  }
}

const transport = ref<Transport>(loadTransport());

function setTransport(next: Transport) {
  if (together.active) return;
  transport.value = next;
  try {
    localStorage.setItem(TRANSPORT_KEY, next);
  } catch {}
}

onMounted(() => {
  void together.init();
});

function onNick(event: Event) {
  together.setNick((event.target as HTMLInputElement).value);
}
</script>

<style scoped>
.together-input {
  width: 190px;
  padding: 7px 10px;
  border-radius: 8px;
  border: 1px solid var(--border, rgba(255, 255, 255, 0.12));
  background: transparent;
  color: inherit;
  font: inherit;
}

.together-transport {
  display: inline-flex;
  gap: 6px;
}

.together-transport button {
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid var(--border, rgba(255, 255, 255, 0.12));
  background: transparent;
  color: inherit;
  font: inherit;
  cursor: pointer;
}

.together-transport button.active {
  background: var(--surface-2, rgba(255, 255, 255, 0.08));
  border-color: var(--accent, #ffcc00);
}

.together-transport button:disabled {
  opacity: 0.5;
  cursor: default;
}

.setting-row.column {
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
}

.together-error {
  margin: 0;
  color: var(--danger, #fa2d48);
  font-size: 13px;
}

.together-wait {
  margin: 0;
  font-size: 13px;
  color: var(--accent, #ffcc00);
}

.together-hint {
  margin: 0;
  opacity: 0.6;
  font-size: 12px;
}
</style>
