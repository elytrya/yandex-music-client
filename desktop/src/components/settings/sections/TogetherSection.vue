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

    <TogetherHostCard />
    <TogetherJoinCard />

    <div v-if="together.active" class="setting-row column">
      <div class="setting-copy">
        <b>{{ together.isHost ? "Вы ведёте" : "Вы слушаете хоста" }}</b>
        <span>{{ together.peers.length }} в комнате</span>
      </div>

      <TogetherPeers :peers="together.peers" />
    </div>

    <p v-if="together.error" class="together-error">{{ together.error }}</p>

    <TogetherLog />
  </section>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import TogetherHostCard from "@/components/together/TogetherHostCard.vue";
import TogetherJoinCard from "@/components/together/TogetherJoinCard.vue";
import TogetherLog from "@/components/together/TogetherLog.vue";
import TogetherPeers from "@/components/together/TogetherPeers.vue";
import { useTogetherStore } from "@/stores/together/index";

const together = useTogetherStore();

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
</style>
