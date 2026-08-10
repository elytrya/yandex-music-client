<template>
  <section id="settings-together" class="settings-group">
    <div class="settings-group-head">
      <h2>Слушать вместе</h2>
      <p>
        Синхронное прослушивание по локальной сети, через VPN (radmin, hamachi,
        zerotier) или сервер-ретранслятор. По сети идут только трек и позиция -
        звук каждый грузит из своего аккаунта.
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
        <span>По сети/VPN или через сервер без проброса портов.</span>
      </div>

      <div class="together-switch" :class="{ locked: together.active }">
        <span class="together-switch-thumb" :class="transport" />

        <button
          type="button"
          class="together-switch-option"
          :class="{ active: transport === 'local' }"
          :disabled="together.active"
          @click="setTransport('local')"
        >
          <Icon name="wave" :size="15" />
          <span>Локально</span>
        </button>

        <button
          type="button"
          class="together-switch-option"
          :class="{ active: transport === 'server' }"
          :disabled="together.active"
          @click="setTransport('server')"
        >
          <Icon name="globe" :size="15" />
          <span>Через сервер</span>
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
        <b>{{ together.isHost ? "Хостите" : "Слушаете хоста" }}</b>
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
        Треки для всех включаете вы. Чтобы включал кто-то другой - передайте ему
        хост.
      </p>

      <p v-else class="together-hint">
        Хостит {{ together.hostNick }} - ваш плеер повторяет за ним. Чтобы
        самому включать треки, попросите передать вам хост.
      </p>
    </div>

    <SettingToggle
      :model-value="ui.settings.togetherShowDock"
      label="Плашка комнаты поверх окна"
      description="Маленькая панель с текстом «Хостите» / «Слушаете» и списком участников. Комната при этом продолжает работать."
      @update:model-value="ui.set('togetherShowDock', $event)"
    />

    <p v-if="together.error" class="together-error">{{ together.error }}</p>

    <button
      class="together-log-toggle"
      type="button"
      @click="showLog = !showLog"
    >
      {{ showLog ? "Скрыть журнал" : "Журнал подключения" }}
    </button>

    <TogetherLog v-if="showLog" />
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import Icon from "@/components/Icon.vue";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import TogetherHostCard from "@/components/together/TogetherHostCard.vue";
import TogetherJoinCard from "@/components/together/TogetherJoinCard.vue";
import TogetherServerCard from "@/components/together/TogetherServerCard.vue";
import TogetherLog from "@/components/together/TogetherLog.vue";
import TogetherPeers from "@/components/together/TogetherPeers.vue";
import { useTogetherStore } from "@/stores/together/index";
import { TRANSPORT_KEY } from "@/stores/together/protocol";
import { useUiStore } from "@/stores/ui/index";

const together = useTogetherStore();
const ui = useUiStore();

type Transport = "local" | "server";

function loadTransport(): Transport {
  try {
    return localStorage.getItem(TRANSPORT_KEY) === "server"
      ? "server"
      : "local";
  } catch {
    return "local";
  }
}

const transport = ref<Transport>(loadTransport());
const showLog = ref(false);

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

.together-switch {
  position: relative;
  display: inline-flex;
  padding: 3px;
  border-radius: 11px;
  border: 1px solid var(--border, rgba(255, 255, 255, 0.1));
  background: var(--surface-2, rgba(255, 255, 255, 0.06));
}

.together-switch.locked {
  opacity: 0.55;
}

.together-switch-thumb {
  position: absolute;
  top: 3px;
  bottom: 3px;
  left: 3px;
  width: calc(50% - 3px);
  border-radius: 8px;
  background: var(--surface, rgba(255, 255, 255, 0.16));
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.25);
  transition: transform 0.18s ease;
}

.together-switch-thumb.server {
  transform: translateX(100%);
}

.together-switch-option {
  position: relative;
  z-index: 1;
  display: inline-flex;
  min-width: 130px;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 8px 14px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--fg-dim, inherit);
  font: inherit;
  font-size: 13px;
  cursor: pointer;
  opacity: 0.7;
  transition:
    opacity 0.15s ease,
    color 0.15s ease;
}

.together-switch-option.active {
  color: var(--fg, inherit);
  opacity: 1;
}

.together-switch-option:disabled {
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

.together-log-toggle {
  align-self: flex-start;
  padding: 4px 0;
  border: 0;
  background: transparent;
  color: var(--fg-dim, inherit);
  font: inherit;
  font-size: 13px;
  cursor: pointer;
  opacity: 0.75;
}

.together-log-toggle:hover {
  opacity: 1;
}
</style>
