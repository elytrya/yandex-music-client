<template>
  <div class="setting-row">
    <div class="setting-copy">
      <b>Сервер</b>
      <span>
        Куда подключаемся. По умолчанию общий сервер mashiro. Можно вбить
        свой адрес или 127.0.0.1:7332 для локального.
      </span>
    </div>

    <input
      v-model="address"
      class="together-input wide"
      type="text"
      placeholder="mashiro.onecorporation.cfd"
      :disabled="connected"
    />
  </div>

  <div v-if="!connected" class="setting-row">
    <div class="setting-copy">
      <b>Создать комнату</b>
      <span>
        Поднимем комнату на сервере и дадим код из шести слов — просто
        перешлите его друзьям.
      </span>
    </div>

    <button
      class="settings-reset-button"
      type="button"
      :disabled="busy"
      @click="create"
    >
      Создать комнату
    </button>
  </div>

  <div v-if="!connected" class="setting-row">
    <div class="setting-copy">
      <b>Войти по коду</b>
      <span>Вставьте код из слов, который прислал друг.</span>
    </div>

    <div class="together-controls">
      <input
        v-model="invite"
        class="together-input wide"
        type="text"
        placeholder="olive canyon spirit velvet ladder onion"
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

  <div v-if="roomCode" class="together-invite">
    <b>Код вашей комнаты</b>

    <div class="together-invite-line">
      <code title="Нажмите, чтобы скопировать" @click="copy(roomCode)">{{
        roomCode
      }}</code>

      <button
        class="together-invite-icon"
        type="button"
        title="Скопировать код"
        @click="copy(roomCode)"
      >
        <Icon name="copy" :size="14" />
      </button>
    </div>

    <span>Перешлите эти шесть слов друзьям — по ним они войдут в комнату.</span>
  </div>

  <div v-if="connected" class="setting-row">
    <div class="setting-copy">
      <b>Вы в комнате</b>
      <span>{{ peopleLabel }}</span>
    </div>

    <button
      class="settings-reset-button danger"
      type="button"
      :disabled="busy"
      @click="leave"
    >
      Выйти
    </button>
  </div>

  <button
    class="together-advanced-toggle"
    type="button"
    @click="advanced = !advanced"
  >
    {{ advanced ? "Скрыть дополнительное" : "Дополнительное" }}
  </button>

  <div v-if="advanced" class="together-invite">
    <b>Ключ-аккаунт</b>
    <span>
      Создаётся сам при первом входе и хранится на этом пк. Ничего
      вводить не надо. Скопируйте эти слова, только если хотите зайти под
      тем же именем на другом устройстве.
    </span>

    <div class="together-controls">
      <span class="together-seed-state">{{
        hasSeed ? "аккаунт готов" : "создастся при входе"
      }}</span>

      <button
        class="settings-reset-button"
        type="button"
        :disabled="busy || !hasSeed"
        @click="showSeed"
      >
        Показать ключ
      </button>

      <button
        class="settings-reset-button danger"
        type="button"
        :disabled="busy"
        @click="resetSeed"
      >
        Сбросить
      </button>
    </div>

    <textarea
      v-if="phrase"
      v-model="phrase"
      class="together-input phrase"
      rows="2"
    ></textarea>

    <div v-if="phrase" class="together-controls">
      <button
        class="settings-reset-button"
        type="button"
        @click="copy(phrase)"
      >
        Скопировать
      </button>

      <button
        class="settings-reset-button"
        type="button"
        :disabled="!phrase.trim()"
        @click="saveSeed"
      >
        Сохранить ключ
      </button>
    </div>
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
const advanced = ref(false);

const status = relayView;
const busy = relayBusy;
const reason = relayReason;
const connected = computed(() => status.value.connected);
const roomCode = computed(() => status.value.invite);
const peopleLabel = computed(() => {
  const total = status.value.peers.length || 1;
  return `участников: ${total}`;
});

onMounted(async () => {
  hasSeed.value = await seedExists();
});

function remember() {
  localStorage.setItem(SERVER_KEY, address.value.trim());
}

async function copy(text: string) {
  await copyText(text, "Скопировано", "Нечего копировать");
}

async function ensureSeed() {
  if (hasSeed.value) return;
  await seedNew(12);
  hasSeed.value = true;
}

async function resetSeed() {
  await seedForget();
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

async function create() {
  remember();
  await ensureSeed();
  await createRoom(address.value, together.nick, invite.value || undefined);
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

.together-input.wide {
  width: 280px;
  max-width: 100%;
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

.together-advanced-toggle {
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

.together-advanced-toggle:hover {
  opacity: 1;
}

.together-reason {
  margin: 0;
  font-size: 13px;
  color: var(--accent, #ffcc00);
}
</style>
