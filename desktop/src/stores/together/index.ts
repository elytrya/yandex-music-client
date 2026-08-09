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
import type {
  StatePayload,
  TogetherMessage,
  TogetherPayload,
} from "./protocol";
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
import { keepAlive, nicksOf, toggleId, withId } from "./roster";
import { applyState, buildState, expectedPosition } from "./sync";

type Player = ReturnType<typeof usePlayerStore>;

const unlisteners: UnlistenFn[] = [];

let timer: number | null = null;
let lastProgress = 0;
let lastTick = 0;
let lastSent = 0;
let applying = false;
let holding = false;
let resumeAfterWait = false;
let lastApplied: StatePayload | null = null;
let lastReady: boolean | null = null;

interface TogetherState {
  mode: TogetherMode;
  port: number;
  address: string | null;
  nick: string;
  selfId: number;
  peers: TogetherPeer[];
  waiting: number[];
  controllers: number[];
  rights: boolean;
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
    selfId: 0,
    peers: [],
    waiting: [],
    controllers: [],
    rights: false,
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
    canControl: (s) => s.mode === "host" || (s.mode === "guest" && s.rights),
    waitingNicks: (s) => nicksOf(s.waiting, s.peers),
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
        await listen("together://joined", () => {
          this.push();
          this.shareRights();
        }),
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
      this.selfId = status.selfId;
      if (status.port) this.port = status.port;
      if (status.nick) this.nick = status.nick;

      this.waiting = keepAlive(this.waiting, status.peers);
      this.controllers = keepAlive(this.controllers, status.peers);

      if (status.mode === "off") {
        this.waiting = [];
        this.controllers = [];
        this.rights = false;
        lastApplied = null;
        lastReady = null;
        holding = false;
        resumeAfterWait = false;
        this.stopTimer();
        return;
      }

      if (status.mode === "host" && !this.waiting.length) this.release();
      this.startTimer();
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

    send(payload: TogetherPayload) {
      if (this.mode === "off") return;
      void togetherApi.send(payload).catch((e: unknown) => {
        this.note(
          formatLocal("ui", reason(e, "не удалось отправить сообщение")),
        );
      });
    },

    push() {
      if (this.mode !== "host") return;
      lastSent = Date.now();
      this.send(buildState(usePlayerStore()));
    },

    grant(id: number) {
      if (this.mode !== "host" || id === 0) return;

      this.controllers = toggleId(this.controllers, id);
      const nick = this.peers.find((peer) => peer.id === id)?.nick ?? `#${id}`;
      const allowed = this.controllers.includes(id);

      this.note(
        formatLocal(
          "ui",
          allowed ? `${nick} может управлять` : `${nick} больше не управляет`,
        ),
      );
      this.shareRights();
    },

    shareRights() {
      if (this.mode !== "host") return;
      this.send({ kind: "rights", ids: [...this.controllers] });
    },

    reportReady(ready: boolean) {
      if (this.mode !== "guest" || lastReady === ready) return;

      lastReady = ready;
      this.send({
        kind: "ready",
        trackId: usePlayerStore().current?.id ?? null,
        ready,
      });
      this.note(formatLocal("ui", ready ? "трек готов" : "гружу трек"));
    },

    setWaiting(id: number, waiting: boolean, nick: string) {
      if (this.waiting.includes(id) === waiting) return;

      this.waiting = withId(this.waiting, id, waiting);
      this.note(
        formatLocal("ui", waiting ? `${nick} грузит трек` : `${nick} готов`),
      );

      if (this.waiting.length) this.hold();
      else this.release();
    },

    hold() {
      if (this.mode !== "host" || holding) return;

      const player = usePlayerStore();
      holding = true;
      resumeAfterWait = player.isPlaying;
      if (player.isPlaying) player.toggle();

      this.push();
      this.note(formatLocal("ui", "ждём загрузку у участников"));
    },

    release() {
      if (!holding) return;

      const player = usePlayerStore();
      holding = false;
      if (resumeAfterWait && !player.isPlaying) player.toggle();
      resumeAfterWait = false;

      this.push();
      this.note(formatLocal("ui", "все загрузились, продолжаем"));
    },

    async receive(message: TogetherMessage) {
      const payload = message.payload;
      if (!payload) return;

      if (payload.kind === "ready") {
        if (this.mode !== "host") return;
        this.setWaiting(message.from, !payload.ready, message.nick);
        return;
      }

      if (payload.kind === "rights") {
        if (this.mode !== "guest") return;

        const allowed = payload.ids.includes(this.selfId);
        if (allowed === this.rights) return;

        this.rights = allowed;
        this.note(
          formatLocal(
            "ui",
            allowed ? "хост выдал управление" : "хост забрал управление",
          ),
        );
        Notify.create({
          message: allowed
            ? "Хост выдал вам управление"
            : "Управление вернулось хосту",
        });
        return;
      }

      if (payload.kind !== "state" || applying) return;

      if (this.mode === "host") {
        if (!this.controllers.includes(message.from)) return;
        await this.adopt(payload, message.nick);
        return;
      }

      if (this.mode === "guest") await this.follow(payload);
    },

    async adopt(payload: StatePayload, nick: string) {
      applying = true;
      try {
        await applyState(usePlayerStore(), payload);
        this.note(formatLocal("ui", `команда от ${nick}`));
      } catch (e) {
        this.error = reason(e, "Не удалось выполнить команду участника");
        this.note(formatLocal("ui", this.error));
      } finally {
        applying = false;
      }

      this.push();
    },

    async follow(payload: StatePayload) {
      const player = usePlayerStore();
      if (payload.trackId && payload.trackId !== player.current?.id) {
        this.reportReady(false);
      }

      applying = true;
      try {
        await applyState(player, payload);
        lastApplied = payload;
      } catch (e) {
        this.error = reason(e, "Не удалось синхронизироваться");
        this.note(formatLocal("ui", this.error));
      } finally {
        applying = false;
      }

      this.reportReady(!player.loading);
    },

    pushControl(player: Player) {
      if (!lastApplied) return;

      const state = buildState(player);
      const drift = Math.abs(
        state.positionMs / 1000 - expectedPosition(lastApplied),
      );
      const same =
        state.trackId === lastApplied.trackId &&
        state.paused === lastApplied.paused &&
        drift <= JUMP_LIMIT;

      if (same) return;

      lastApplied = state;
      this.send(state);
      this.note(formatLocal("ui", "отправил команду хосту"));
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
      const player = usePlayerStore();
      const now = Date.now();

      if (this.mode === "guest") {
        this.reportReady(!player.loading);
        if (this.rights && !applying && !player.loading) {
          this.pushControl(player);
        }
        return;
      }

      if (this.mode !== "host") return;

      const moved = player.isPlaying ? (now - lastTick) / 1000 : 0;
      const jumped =
        Math.abs(player.progress - (lastProgress + moved)) > JUMP_LIMIT;

      lastProgress = player.progress;
      lastTick = now;

      if (jumped || now - lastSent > RESEND_MS) this.push();
    },
  },
});
