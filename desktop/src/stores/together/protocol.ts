import type { Track } from "@/api/types";

export const DEFAULT_PORT = 7331;
export const NICK_KEY = "mashiro.together.nick";
export const DOCK_KEY = "mashiro.together.dock";
export const DOCK_POS_KEY = "mashiro.together.dock.pos";
export const SERVER_KEY = "mashiro.together.server";
export const TRANSPORT_KEY = "mashiro.together.transport";

export const DRIFT_LIMIT = 1.2;

export const JUMP_LIMIT = 1.5;

export const HEARTBEAT_MS = 1000;

export const RESEND_MS = 5000;

export const PUSH_DEBOUNCE = 80;

export const QUEUE_LIMIT = 120;

export const HANDOFF_DELAY = 900;

export const HANDOFF_RETRIES = 6;
export const HANDOFF_GAP = 700;

export const HANDOFF_WINDOW = 15000;

export interface StatePayload {
  kind: "state";
  trackId: string | null;
  positionMs: number;
  paused: boolean;
  updatedAt: number;
  title: string | null;

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
