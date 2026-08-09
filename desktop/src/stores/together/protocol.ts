import type { Track } from "@/api/types";

export const DEFAULT_PORT = 7331;
export const NICK_KEY = "mashiro.together.nick";
export const DOCK_KEY = "mashiro.together.dock";
export const DOCK_POS_KEY = "mashiro.together.dock.pos";

// насколько можно разъехаться по позиции, прежде чем подтягивать себя
export const DRIFT_LIMIT = 1.2;
// разбег позиции, который считается перемоткой, а не обычным ходом трека
export const JUMP_LIMIT = 1.5;
// как часто тикает фоновая проверка
export const HEARTBEAT_MS = 1000;
// контрольная пересылка состояния, если ничего не менялось
export const RESEND_MS = 5000;
// склейка частых изменений в одну отправку
export const PUSH_DEBOUNCE = 80;
// сколько треков очереди уезжает в комнату вместе с состоянием
export const QUEUE_LIMIT = 120;
// сколько ждём, пока новый хост поднимет комнату
export const HANDOFF_DELAY = 900;
// сколько раз стучимся к новому хосту и с каким шагом
export const HANDOFF_RETRIES = 6;
export const HANDOFF_GAP = 700;
// в этом окне разрыв связи — часть переезда, а не поломка
export const HANDOFF_WINDOW = 15000;

export interface StatePayload {
  kind: "state";
  trackId: string | null;
  positionMs: number;
  paused: boolean;
  updatedAt: number;
  title: string | null;
  // сам трек и кусок очереди едут целиком: плейлист другого человека
  // может быть недоступен, а трек по своему id откроется
  track: Track | null;
  queue: Track[];
  index: number;
}

export interface ReadyPayload {
  kind: "ready";
  trackId: string | null;
  ready: boolean;
}

export interface NotePayload {
  kind: "note";
  text: string;
}

/** Комната переезжает: участник `to` поднимает её у себя. */
export interface HandoffPayload {
  kind: "handoff";
  to: number;
  nick: string;
  address: string;
  port: number;
}

export type TogetherPayload =
  StatePayload | ReadyPayload | NotePayload | HandoffPayload;

export interface TogetherMessage {
  from: number;
  nick: string;
  payload: TogetherPayload | null;
}

export function loadNick(): string {
  try {
    return localStorage.getItem(NICK_KEY) || "";
  } catch {
    return "";
  }
}

export function saveNick(nick: string): void {
  try {
    localStorage.setItem(NICK_KEY, nick);
  } catch {}
}
