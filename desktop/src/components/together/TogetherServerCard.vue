<template>
  <div class="setting-row">
    <div class="setting-copy">
      <b>Комната через сервер</b>
      <span>
        Работает через интернет без проброса портов и без vpn. Сервер
        пересылает только шифротекст — трек и позицию, звук каждый грузит из
        своего аккаунта.
      </span>
    </div>

    <div class="together-controls">
      <input
        v-model="address"
        class="together-input"
        type="text"
        placeholder="mashiro.onecorporation.cfd"
        :disabled="connected"
      />

      <button
        v-if="!connected"
        class="settings-reset-button"
        type="button"
        :disabled="busy"
        @click="create"
      >
        Создать
      </button>

      <button
        v-else
        class="settings-reset-button danger"
        type="button"
        :disabled="busy"
        @click="leave"
      >
        Выйти
      </button>
    </div>
  </div>

  <div v-if="!connected" class="setting-row">
    <div class="setting-copy">
      <b>Войти по коду</b>
      <span>
        Вставьте код приглашения от друга — адрес сервера у вас уже общий.
      </span>
    </div>

    <div class="together-controls">
      <input
        v-model="invite"
        class="together-input"
        type="text"
        placeholder="код приглашения"
        @keyup.enter="join"
      />

      <button
        class="settings-reset-button"
        type="button"
        :disabled="busy || !invite.trim()"
        @click="join"
      >
        Войти
      </button>
    </div>
  </div>

  <div v-if="connected && status.invite" class="together-invite">
    <b>Код приглашения</b>

    <div class="together-invite-line">
      <code title="Нажмите, чтобы скопировать" @click="copy(status.invite)">{{
        status.invite
      }}</code>

      <button
        class="together-invite-icon"
        type="button"
        title="Скопировать код"
        @click="copy(status.invite)"
      >
        <Icon name="copy" :size="14" />
      </button>
    </div>

    <span>Отдайте этот код друзьям, чтобы они вошли в вашу комнату.</span>
  </div>

  <div class="together-invite">
    <b>Сид-фраза этого устройства</b>
    <span>
      Ваша личность в комнатах. Хранится в системном хранилище ключей и не
      уходит на сервер. Если фразы нет, она создастся сама при входе.
      Перенесите её на другой компьютер, чтобы остаться собой.
    </span>

    <div class="together-controls">
      <span class="together-seed-state">{{
        hasSeed ? "фраза сохранена" : "фразы пока нет"
      }}</span>

      <button
        class="settings-reset-button"
        type="button"
        :disabled="busy"
        @click="makeSeed"
      >
        Создать
      </button>

      <button
        class="settings-reset-button"
        type="button"
        :disabled="!hasSeed"
        @click="showSeed"
      >
        Показать
      </button>

      <button
        class="settings-reset-button danger"
        type="button"
        :disabled="!hasSeed"
        @click="dropSeed"
      >
        Забыть
      </button>
    </div>

    <textarea
      v-model="phrase"
      class="together-input phrase"
      rows="2"
      placeholder="слова через пробел — вставьте, чтобы перенести личность"
    ></textarea>

    <button
      class="settings-reset-button"
      type="button"
      :disabled="!phrase.trim()"
      @click="saveSeed"
    >
      Сохранить фразу
    </button>
  </div>

  <p v-if="reason" class="together-reason">{{ reason }}</p>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import Icon from "@/components/Icon.vue";
import { copyText } from "@/lib/clipboard";
import { seedExists, seedForget, seedNew, seedSet, seedShow } from "@/api/relay";
import {
  createRoom,
  joinRoom,
  leaveRoom,
  relayBusy,
  relayReason,
  relayView,
} from "@/stores/together/relay";
import { useTogetherStore } from "@/stores/together/index";
import { SERVER_KEY } from "@/stores/together/protocol";

const DEFAULT_SERVER = "mashiro.onecorporation.cfd";

const together = useTogetherStore();

const address = ref(localStorage.getItem(SERVER_KEY) || DEFAULT_SERVER);
const invite = ref("");
const phrase = ref("");
const hasSeed = ref(false);

const status = relayView;
const busy = relayBusy;
const reason = relayReason;
const connected = computed(() => status.value.connected);

onMounted(async () => {
  hasSeed.value = await seedExists();
  if (status.value.invite) invite.value = status.value.invite;
});

function remember() {
  localStorage.setItem(SERVER_KEY, address.value.trim());
}

async function copy(text: string) {
  await copyText(text, "Скопировано", "Нечего копировать");
}

async function ensureSeed() {
  if (hasSeed.value) return;
  phrase.value = await seedNew(12);
  hasSeed.value = true;
}

async function makeSeed() {
  phrase.value = await seedNew(12);
  hasSeed.value = true;
}

async function showSeed() {
  phrase.value = (await seedShow()) ?? "";
}

async function saveSeed() {
  phrase.value = await seedSet(phrase.value);
  hasSeed.value = true;
}

async function dropSeed() {
  await seedForget();
  phrase.value = "";
  hasSeed.value = false;
}

async function create() {
  remember();
  await ensureSeed();
  const result = await createRoom(
    address.value,
    together.nick,
    invite.value || undefined,
  );
  if (result) invite.value = result.invite;
}

async function join() {
  if (!invite.value.trim()) return;
  remember();
  await ensureSeed();
  await joinRoom(address.value, together.nick, invite.value);
}

async function leave() {
  await leaveRoom();
}
</script>

<style scoped>
.together-controls {
  display: flex;
  flex-wrap: wrap;
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

.together-input.phrase {
  width: 100%;
  resize: vertical;
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
  cursor: pointer;
}

.together-invite code:hover {
  color: var(--accent, inherit);
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

.together-seed-state {
  font-size: 13px;
  opacity: 0.7;
}

.together-reason {
  margin: 0;
  font-size: 13px;
  color: var(--accent, #ffcc00);
}
</style>
