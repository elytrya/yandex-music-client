import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface RelayPeer {
  id: number
  nick: string
}

export interface RelayStatus {
  connected: boolean
  address: string
  room: string
  invite: string
  nick: string
  selfId: number
  host: number
  peers: RelayPeer[]
  ping: number
}

export interface RelayMessage {
  from: number
  nick: string
  payload: Record<string, unknown>
}

export interface RelayJoined {
  id: number
  nick: string
}

export interface RelayClosed {
  reason: string
}

export const emptyRelayStatus = (): RelayStatus => ({
  connected: false,
  address: '',
  room: '',
  invite: '',
  nick: '',
  selfId: 0,
  host: 0,
  peers: [],
  ping: 0,
})

export function relayCreate(address: string, nick: string, invite?: string) {
  return invoke<RelayStatus>('together_relay_create', { address, nick, invite: invite ?? null })
}

export function relayJoin(address: string, nick: string, invite: string) {
  return invoke<RelayStatus>('together_relay_join', { address, nick, invite })
}

export function relayLeave() {
  return invoke<void>('together_relay_leave')
}

export function relaySend(payload: Record<string, unknown>) {
  return invoke<void>('together_relay_send', { payload })
}

export function relayHandoff(to: number) {
  return invoke<void>('together_relay_handoff', { to })
}

export function relayStatus() {
  return invoke<RelayStatus>('together_relay_status')
}

export function seedExists() {
  return invoke<boolean>('together_relay_seed_exists')
}

export function seedShow() {
  return invoke<string | null>('together_relay_seed_show')
}

export function seedSet(phrase: string) {
  return invoke<string>('together_relay_seed_set', { phrase })
}

export function seedNew(words: 12 | 24 = 12) {
  return invoke<string>('together_relay_seed_new', { words })
}

export function seedForget() {
  return invoke<void>('together_relay_seed_forget')
}

export function relayIdentity() {
  return invoke<string | null>('together_relay_identity')
}

export function relayInvite(short = false) {
  return invoke<string>('together_relay_invite', { short })
}

export function onRelayStatus(handler: (status: RelayStatus) => void): Promise<UnlistenFn> {
  return listen<RelayStatus>('together://relay-status', (event) => handler(event.payload))
}

export function onRelayMessage(handler: (message: RelayMessage) => void): Promise<UnlistenFn> {
  return listen<RelayMessage>('together://relay-message', (event) => handler(event.payload))
}

export function onRelayJoined(handler: (peer: RelayJoined) => void): Promise<UnlistenFn> {
  return listen<RelayJoined>('together://relay-joined', (event) => handler(event.payload))
}

export function onRelayClosed(handler: (closed: RelayClosed) => void): Promise<UnlistenFn> {
  return listen<RelayClosed>('together://relay-closed', (event) => handler(event.payload))
}
