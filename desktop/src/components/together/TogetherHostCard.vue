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

    <div class="together-invite-line">
      <code>{{ shown ? invite : "•".repeat(14) }}</code>

      <button
        class="together-invite-icon"
        type="button"
        :title="shown ? 'Скрыть адрес' : 'Показать адрес'"
        @click="shown = !shown"
      >
        <Icon :name="shown ? 'eyeOff' : 'eye'" :size="14" />
      </button>

      <button
        class="together-invite-icon"
        type="button"
        title="Скопировать адрес"
        @click="copyInvite"
      >
        <Icon name="copy" :size="14" />
      </button>
    </div>

    <span>
      Это адрес в обычной локалке. Если сидите через radmin vpn, hamachi,
      zerotier или tailscale, возьмите адрес из окна самой программы и тот же
      порт.
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { Notify } from "quasar";
import Icon from "@/components/Icon.vue";
import { useTogetherStore } from "@/stores/together/index";

const together = useTogetherStore();
const port = ref(together.port);
const shown = ref(false);

const invite = computed(() => together.invite || `ваш-ip:${together.port}`);

async function copyInvite() {
  try {
    await navigator.clipboard.writeText(together.invite || invite.value);
    Notify.create({ message: "Адрес скопирован" });
  } catch {
    Notify.create({ message: "Не удалось скопировать" });
  }
}
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

.together-invite-line {
  display: flex;
  align-items: center;
  gap: 8px;
}

.together-invite code {
  overflow: hidden;
  flex: 1 1 auto;
  font-size: 15px;
  text-overflow: ellipsis;
  white-space: nowrap;
  user-select: all;
}

.together-invite-icon {
  display: inline-flex;
  flex: 0 0 auto;
  padding: 5px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: var(--fg-dim, inherit);
  cursor: pointer;
}

.together-invite-icon:hover {
  background: var(--hover, rgba(255, 255, 255, 0.08));
  color: var(--fg, inherit);
}

.together-invite span {
  opacity: 0.65;
  font-size: 12px;
}
</style>
