<template>
  <div class="setting-row">
    <div class="setting-copy">
      <b>Своя комната</b>
      <span>
        Поднимает комнату на вашем компьютере. Остальные подключаются к вашему
        адресу и следуют за вашим плеером.
      </span>
    </div>

    <div class="together-controls">
      <input
        v-model.number="port"
        class="together-input port"
        type="number"
        min="1024"
        max="65535"
        :disabled="together.active"
      />

      <button
        v-if="!together.isHost"
        class="settings-reset-button"
        type="button"
        :disabled="together.busy || together.active"
        @click="together.host(port)"
      >
        Создать
      </button>

      <button
        v-else
        class="settings-reset-button danger"
        type="button"
        :disabled="together.busy"
        @click="together.leave()"
      >
        Закрыть
      </button>
    </div>
  </div>

  <div v-if="together.isHost" class="together-invite">
    <b>Адрес для друзей</b>
    <code>{{ together.invite || `ваш-ip:${together.port}` }}</code>
    <span>
      Это адрес в обычной локалке. Если сидите через radmin vpn, hamachi,
      zerotier или tailscale, возьмите адрес из окна самой программы и тот же
      порт.
    </span>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useTogetherStore } from "@/stores/together/index";

const together = useTogetherStore();
const port = ref(together.port);
</script>

<style scoped>
.together-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.together-input {
  width: 190px;
  padding: 7px 10px;
  border-radius: 8px;
  border: 1px solid var(--border, rgba(255, 255, 255, 0.12));
  background: transparent;
  color: inherit;
  font: inherit;
}

.together-input.port {
  width: 92px;
}

.together-invite {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px 14px;
  border-radius: 10px;
  background: var(--surface-2, rgba(255, 255, 255, 0.06));
}

.together-invite code {
  font-size: 15px;
  user-select: all;
}

.together-invite span {
  opacity: 0.65;
  font-size: 12px;
}
</style>
