export const DEFAULT_PORT = 7331;
export const NICK_KEY = "mashiro.together.nick";

export const DRIFT_LIMIT = 1.5;
export const JUMP_LIMIT = 2;
export const HEARTBEAT_MS = 1000;
export const RESEND_MS = 4000;

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
