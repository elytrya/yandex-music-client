import { reactive, computed } from 'vue'
import type { UnlistenFn } from '@tauri-apps/api/event'

import {
  emptyRelayStatus,
  onRelayClosed,
  onRelayJoined,
  onRelayMessage,
  onRelayStatus,
  relayCreate,
  relayHandoff,
  relayJoin,
  relayLeave,
  relaySend,
  relayStatus,
  type RelayMessage,
  type RelayStatus,
} from '@/api/relay'

export interface RelayHandlers {
  onMessage: (from: number, nick: string, payload: Record<string, unknown>) => void
  onJoined: (id: number, nick: string) => void
  onClosed: (reason: string) => void
}

const state = reactive({
  status: emptyRelayStatus(),

  reason: '',
  busy: false,
})

let unlisten: UnlistenFn[] = []

export const relayActive = computed(() => state.status.connected)

export const relayIsHost = computed(
  () => state.status.connected && state.status.selfId !== 0 && state.status.selfId === state.status.host,
)

export const relayView = computed(() => state.status)
export const relayReason = computed(() => state.reason)
export const relayBusy = computed(() => state.busy)

export async function bindRelay(handlers: RelayHandlers): Promise<void> {
  await unbindRelay()

  unlisten = await Promise.all([
    onRelayStatus((status: RelayStatus) => {
      state.status = status
    }),
    onRelayMessage((message: RelayMessage) => {
      handlers.onMessage(message.from, message.nick, message.payload)
    }),
    onRelayJoined((peer) => {
      handlers.onJoined(peer.id, peer.nick)
    }),
    onRelayClosed((closed) => {
      state.status = emptyRelayStatus()
      state.reason = closed.reason
      handlers.onClosed(closed.reason)
    }),
  ])

  state.status = await relayStatus()
}

export async function unbindRelay(): Promise<void> {
  for (const stop of unlisten) {
    stop()
  }
  unlisten = []
}

async function guard<T>(work: () => Promise<T>): Promise<T | null> {
  if (state.busy) {
    return null
  }

  state.busy = true
  try {
    const result = await work()
    state.reason = ''
    return result
  } catch (error) {
    state.reason = String(error)
    return null
  } finally {
    state.busy = false
  }
}

export function createRoom(address: string, nick: string, invite?: string) {
  return guard(async () => {
    state.status = await relayCreate(address, nick, invite)
    return state.status
  })
}

export function joinRoom(address: string, nick: string, invite: string) {
  return guard(async () => {
    state.status = await relayJoin(address, nick, invite)
    return state.status
  })
}

export function leaveRoom() {
  return guard(async () => {
    await relayLeave()
    state.status = emptyRelayStatus()
    return true
  })
}

export async function pushRelay(payload: Record<string, unknown>): Promise<boolean> {
  if (!state.status.connected) {
    return false
  }

  try {
    await relaySend(payload)
    return true
  } catch (error) {
    state.reason = String(error)
    return false
  }
}

export async function handoffRelay(to: number): Promise<boolean> {
  if (!relayIsHost.value) {
    return false
  }

  try {
    await relayHandoff(to)
    return true
  } catch (error) {
    state.reason = String(error)
    return false
  }
}
