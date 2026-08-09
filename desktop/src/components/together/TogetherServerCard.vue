<template>
  <q-card flat bordered class="together-server">
    <q-card-section class="column q-gutter-sm">
      <div class="text-subtitle2">Комната через сервер</div>
      <div class="text-caption text-grey-7">
        Работает через интернет без проброса портов. Сервер видит только шифротекст.
      </div>

      <q-input
        v-model="address"
        dense
        outlined
        label="Адрес сервера"
        placeholder="wss://together.example.org/ws"
        :disable="connected"
      />

      <q-input v-model="nick" dense outlined label="Имя в комнате" :disable="connected" />

      <q-input v-model="invite" dense outlined label="Код приглашения" :disable="connected">
        <template #append>
          <q-btn flat dense icon="content_copy" :disable="!invite" @click="copy(invite)">
            <q-tooltip>Скопировать код</q-tooltip>
          </q-btn>
        </template>
      </q-input>
    </q-card-section>

    <q-card-section class="column q-gutter-sm">
      <div class="text-subtitle2">Сид-фраза</div>
      <div class="text-caption text-grey-7">
        Это вся ваша личность. Хранится в хранилище ключей системы, на сервер не уходит.
      </div>

      <div class="row q-gutter-sm items-center">
        <q-chip dense :color="hasSeed ? 'green-8' : 'grey-7'" text-color="white">
          {{ hasSeed ? 'фраза сохранена' : 'фразы нет' }}
        </q-chip>
        <q-btn dense flat label="Создать" :loading="busy" @click="makeSeed" />
        <q-btn dense flat label="Показать" :disable="!hasSeed" @click="showSeed" />
        <q-btn dense flat label="Забыть" :disable="!hasSeed" @click="dropSeed" />
      </div>

      <q-input
        v-model="phrase"
        dense
        outlined
        type="textarea"
        autogrow
        label="Ввести или посмотреть фразу"
      >
        <template #append>
          <q-btn flat dense icon="content_copy" :disable="!phrase" @click="copy(phrase)">
            <q-tooltip>Скопировать фразу</q-tooltip>
          </q-btn>
        </template>
      </q-input>

      <q-btn dense outline label="Сохранить фразу" :disable="!phrase" @click="saveSeed" />
    </q-card-section>

    <q-card-section v-if="connected" class="column q-gutter-xs">
      <div class="text-subtitle2">В комнате</div>
      <div class="text-caption text-grey-7">Билет комнаты: {{ status.room }}</div>
      <div v-for="peer in status.peers" :key="peer.id" class="row items-center q-gutter-sm">
        <q-chip dense :color="peer.id === status.host ? 'primary' : 'grey-8'" text-color="white">
          {{ peer.nick }}
        </q-chip>
        <q-btn
          v-if="isHost && peer.id !== status.selfId"
          dense
          flat
          label="Передать ведущего"
          @click="pass(peer.id)"
        />
      </div>
    </q-card-section>

    <q-card-section v-if="reason" class="text-caption text-orange-8">{{ reason }}</q-card-section>

    <q-card-actions align="right">
      <q-btn v-if="connected" flat label="Выйти" :loading="busy" @click="leave" />
      <template v-else>
        <q-btn flat label="Войти по коду" :loading="busy" :disable="!invite" @click="join" />
        <q-btn unelevated color="primary" label="Создать комнату" :loading="busy" @click="create" />
      </template>
    </q-card-actions>
  </q-card>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { copyToClipboard, useQuasar } from 'quasar'

import { seedExists, seedForget, seedNew, seedSet, seedShow } from '@/api/relay'
import {
  createRoom,
  handoffRelay,
  joinRoom,
  leaveRoom,
  relayBusy,
  relayIsHost,
  relayReason,
  relayView,
} from '@/stores/together/relay'

const ADDRESS_KEY = 'mashiro.together.server'
const NICK_KEY = 'mashiro.together.nick'
const DEFAULT_SERVER = 'mashiro.onecorporation.cfd'

const quasar = useQuasar()

const address = ref(localStorage.getItem(ADDRESS_KEY) || DEFAULT_SERVER)
const nick = ref(localStorage.getItem(NICK_KEY) ?? '')
const invite = ref('')
const phrase = ref('')
const hasSeed = ref(false)

const status = relayView
const busy = relayBusy
const reason = relayReason
const isHost = relayIsHost
const connected = computed(() => status.value.connected)

onMounted(async () => {
  hasSeed.value = await seedExists()
  if (status.value.invite) {
    invite.value = status.value.invite
  }
})

function remember() {
  localStorage.setItem(ADDRESS_KEY, address.value.trim())
  localStorage.setItem(NICK_KEY, nick.value.trim())
}

function notify(text: string) {
  quasar.notify({ message: text, position: 'bottom-right', timeout: 2500 })
}

async function copy(text: string) {
  await copyToClipboard(text)
  notify('Скопировано')
}

async function makeSeed() {
  phrase.value = await seedNew(12)
  hasSeed.value = true
  notify('Новая сид-фраза сохранена')
}

async function showSeed() {
  phrase.value = (await seedShow()) ?? ''
}

async function saveSeed() {
  phrase.value = await seedSet(phrase.value)
  hasSeed.value = true
  notify('Сид-фраза сохранена')
}

async function dropSeed() {
  await seedForget()
  phrase.value = ''
  hasSeed.value = false
  notify('Сид-фраза удалена')
}

async function ensureSeed() {
  if (hasSeed.value) return
  phrase.value = await seedNew(12)
  hasSeed.value = true
  notify('Создана сид-фраза для этого устройства')
}

async function create() {
  remember()
  await ensureSeed()
  const result = await createRoom(address.value, nick.value, invite.value || undefined)
  if (result) {
    invite.value = result.invite
    notify('Комната создана, код приглашения готов')
  }
}

async function join() {
  remember()
  await ensureSeed()
  await joinRoom(address.value, nick.value, invite.value)
}

async function leave() {
  await leaveRoom()
}

async function pass(id: number) {
  await handoffRelay(id)
}
</script>

<style scoped>
.together-server {
  min-width: 320px;
}
</style>
