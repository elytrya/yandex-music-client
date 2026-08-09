export const DEFAULT_PORT = 7331;
export const NICK_KEY = "mashiro.together.nick";
export const DOCK_KEY = "mashiro.together.dock";

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
// сколько ждём подтверждение своей команды, прежде чем снова слушать хоста
export const CONTROL_GRACE = 1200;

export interface StatePayload {
  kind: "state";
  trackId: string | null;
  positionMs: number;
  paused: boolean;
  updatedAt: number;
  title: string | null;
}

export interface ReadyPayload {
  kind: "ready";
  trackId: string | null;
  ready: boolean;
}

export interface RightsPayload {
  kind: "rights";
  ids: number[];
}

export type TogetherPayload = StatePayload | ReadyPayload | RightsPayload;

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
