import { invoke } from "@tauri-apps/api/core";

export type TogetherMode = "off" | "host" | "guest";

export interface TogetherPeer {
  id: number;
  nick: string;
}

export interface TogetherStatus {
  mode: TogetherMode;
  port: number;
  nick: string;
  address: string | null;
  selfId: number;
  peers: TogetherPeer[];
}

async function call<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw new Error(typeof e === "string" ? e : "Ошибка вызова ядра");
  }
}

export const togetherApi = {
  host: (nick: string, port?: number | null) =>
    call<TogetherStatus>("together_host", { nick, port: port ?? null }),

  join: (address: string, nick: string) =>
    call<TogetherStatus>("together_join", { address, nick }),

  leave: () => call<TogetherStatus>("together_leave"),

  send: (payload: unknown) => call<void>("together_send", { payload }),

  status: () => call<TogetherStatus>("together_status"),

  logPath: () => call<string>("together_log_path"),
};
