import { defineStore } from "pinia";
import { Notify } from "quasar";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { togetherApi } from "@/api/together";
import type {
  TogetherMode,
  TogetherPeer,
  TogetherStatus,
} from "@/api/together";
import { usePlayerStore } from "@/stores/player/index";
import type { TogetherMessage } from "./protocol";
import {
  DEFAULT_PORT,
  HEARTBEAT_MS,
  JUMP_LIMIT,
  RESEND_MS,
  loadNick,
  saveNick,
} from "./protocol";
import type { TogetherLogEvent } from "./log";
import { append, formatEvent, formatLocal } from "./log";
import { applyState, buildState } from "./sync";

const unlisteners: UnlistenFn[] = [];

let timer: number | null = null;
let lastProgress = 0;
let lastTick = 0;
let lastSent = 0;
let applying = false;

interface TogetherState {
  mode: TogetherMode;
  port: number;
  address: string | null;
  nick: string;
  peers: TogetherPeer[];
  busy: boolean;
  error: string | null;
  ready: boolean;
  log: string[];
  logPath: string;
}

function reason(e: unknown, fallback: string): string {
  return e instanceof Error && e.message ? e.message : fallback;
}

export const useTogetherStore = defineStore("together", {
  state: (): TogetherState => ({
    mode: "off",
    port: DEFAULT_PORT,
    address: null,
    nick: loadNick(),
    peers: [],
    busy: false,
    error: null,
    ready: false,
    log: [],
    logPath: "",
  }),

  getters: {
    active: (s) => s.mode !== "off",
    isHost: (s) => s.mode === "host",
    invite: (s) => (s.address ? `${s.address}:${s.port}` : ""),
  },

  actions: {
    async init() {
      if (this.ready) return;
      this.ready = true;

      unlisteners.push(
        await listen<TogetherStatus>("together://status", (event) =>
          this.apply(event.payload),
        ),
        await listen<TogetherMessage>("together://message", (event) => {
          void this.receive(event.payload);
        }),
        await listen("together://joined", () => this.push()),
        await listen<TogetherLogEvent>("together://log", (event) => {
          this.note(formatEvent(event.payload));
        }),
        await listen<{ reason: string }>("together://closed", (event) => {
          this.stopTimer();
          this.note(formatLocal("ui", event.payload.reason));
          Notify.create({ message: event.payload.reason });
        }),
      );

      try {
        this.apply(await togetherApi.status());
        this.logPath = await togetherApi.logPath();
      } catch (e) {
        this.note(formatLocal("ui", reason(e, "ядро не ответило")));
      }
    },

    note(line: string) {
      this.log = append(this.log, line);
      console.debug(`[together] ${line}`);
    },

    clearLog() {
      this.log = [];
    },

    apply(status: TogetherStatus) {
      this.mode = status.mode;
      this.address = status.address;
      this.peers = status.peers;
      if (status.port) this.port = status.port;
      if (status.nick) this.nick = status.nick;
      if (status.mode === "host") this.startTimer();
      else this.stopTimer();
    },

    async host(port?: number) {
      this.busy = true;
      this.error = null;
      try {
        saveNick(this.nick);
        this.apply(await togetherApi.host(this.nick, port ?? this.port));
        Notify.create({ message: "Комната создана" });
      } catch (e) {
        this.error = reason(e, "Не удалось создать комнату");
        this.note(formatLocal("ui", this.error));
      } finally {
        this.busy = false;
      }
    },

    async join(address: string) {
      this.busy = true;
      this.error = null;
      try {
        saveNick(this.nick);
        this.apply(await togetherApi.join(address, this.nick));
        Notify.create({ message: "Подключились к комнате" });
      } catch (e) {
        this.error = reason(e, "Не удалось подключиться");
        this.note(formatLocal("ui", this.error));
      } finally {
        this.busy = false;
      }
    },

    async leave() {
      this.busy = true;
      try {
        this.apply(await togetherApi.leave());
      } catch (e) {
        this.error = reason(e, "Не удалось отключиться");
        this.note(formatLocal("ui", this.error));
      } finally {
        this.busy = false;
      }
    },

    setNick(nick: string) {
      this.nick = nick;
      saveNick(nick);
    },

    push() {
      if (this.mode !== "host") return;
      lastSent = Date.now();
      void togetherApi
        .send(buildState(usePlayerStore()))
        .catch((e: unknown) => {
          this.note(
            formatLocal("ui", reason(e, "не удалось отправить состояние")),
          );
        });
    },

    async receive(message: TogetherMessage) {
      if (this.mode !== "guest" || applying) return;

      const payload = message.payload;
      if (!payload || payload.kind !== "state") return;

      applying = true;
      try {
        await applyState(usePlayerStore(), payload);
      } catch (e) {
        this.error = reason(e, "Не удалось синхронизироваться");
        this.note(formatLocal("ui", this.error));
      } finally {
        applying = false;
      }
    },

    startTimer() {
      if (timer !== null) return;
      lastTick = Date.now();
      lastProgress = usePlayerStore().progress;
      timer = window.setInterval(() => this.tick(), HEARTBEAT_MS);
    },

    stopTimer() {
      if (timer === null) return;
      window.clearInterval(timer);
      timer = null;
    },

    tick() {
      if (this.mode !== "host") return;

      const player = usePlayerStore();
      const now = Date.now();
      const moved = player.isPlaying ? (now - lastTick) / 1000 : 0;
      const jumped =
        Math.abs(player.progress - (lastProgress + moved)) > JUMP_LIMIT;

      lastProgress = player.progress;
      lastTick = now;

      if (jumped || now - lastSent > RESEND_MS) this.push();
    },
  },
});
