import { defineStore } from "pinia";
import { Notify } from "quasar";
import { watch } from "vue";
import type { WatchStopHandle } from "vue";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { togetherApi } from "@/api/together";
import type {
  TogetherMode,
  TogetherPeer,
  TogetherStatus,
} from "@/api/together";
import { askConfirm } from "@/lib/dialogs";
import { usePlayerStore } from "@/stores/player/index";
import type { TogetherMessage, TogetherPayload } from "./protocol";
import {
  DEFAULT_PORT,
  HANDOFF_DELAY,
  HANDOFF_WINDOW,
  HEARTBEAT_MS,
  RESEND_MS,
  loadNick,
  saveNick,
} from "./protocol";
import type { TogetherLogEvent } from "./log";
import { append, formatEvent, formatLocal } from "./log";
import { reconnect, wait } from "./handoff";
import { keepAlive, nicksOf, withId } from "./roster";
import { applyState, buildState } from "./sync";
import { watchPlayer } from "./watch";
import {
  bindRelay,
  handoffRelay,
  leaveRoom,
  pushRelay,
  relayActive,
  relayIsHost,
  relayView,
} from "./relay";
import type { RelayStatus } from "@/api/relay";

const HOST_ID = 0;

const unlisteners: UnlistenFn[] = [];
let stops: WatchStopHandle[] = [];

let timer: number | null = null;
let lastSent = 0;
let applying = false;
let holding = false;
let resumeAfterWait = false;
let lastReady: boolean | null = null;

let handoffAt = 0;

interface TogetherState {
  mode: TogetherMode;
  port: number;
  address: string | null;
  nick: string;
  selfId: number;
  hostId: number;
  peers: TogetherPeer[];
  waiting: number[];
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
    hostId: 0,
    peers: [],
    waiting: [],
    busy: false,
    error: null,
    ready: false,
    log: [],
    logPath: "",
  }),

  getters: {
    active: (s) => relayActive.value || s.mode !== "off",
    isHost: (s) => (relayActive.value ? relayIsHost.value : s.mode === "host"),
    isGuest: (s) =>
      relayActive.value ? !relayIsHost.value : s.mode === "guest",
    invite: (s) =>
      relayActive.value
        ? relayView.value.invite
        : s.address
          ? `${s.address}:${s.port}`
          : "",
    isRelay: () => relayActive.value,
    ping: () => (relayActive.value ? relayView.value.ping : 0),
    waitingNicks: (s) => nicksOf(s.waiting, s.peers),
    hostNick: (s) =>
      s.peers.find((peer) => peer.id === s.hostId)?.nick ?? "хост",
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

          if (Date.now() - handoffAt < HANDOFF_WINDOW) return;
          Notify.create({ message: event.payload.reason });
        }),
      );

      if (!stops.length) {
        stops = watchPlayer(usePlayerStore(), {
          change: (why) => this.onLocalChange(why),
          loading: (busy) => this.reportReady(!busy),
        });
      }

      await bindRelay({
        onMessage: (from, nick, payload) => {
          void this.receive({
            from,
            nick,
            payload: payload as unknown as TogetherPayload,
          });
        },
        onJoined: () => this.push(),
        onClosed: (closed) => {
          this.stopTimer();
          this.note(formatLocal("ui", closed));
        },
      });

      stops.push(
        watch(
          () => relayView.value,
          (status) => this.applyRelay(status),
          { deep: true, immediate: true },
        ),
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

      if (status.mode === "off") {
        this.waiting = [];
        lastReady = null;
        holding = false;
        resumeAfterWait = false;
        this.stopTimer();
        return;
      }

      if (status.mode === "host" && !this.waiting.length) this.release();
      this.startTimer();
    },

    applyRelay(status: RelayStatus) {
      if (status.connected) {
        this.peers = status.peers.map((peer) => ({
          id: peer.id,
          nick: peer.nick,
        }));
        this.selfId = status.selfId;
        this.hostId = status.host;
        if (status.nick) this.nick = status.nick;

        this.waiting = keepAlive(this.waiting, this.peers);
        if (this.isHost && !this.waiting.length) this.release();
        this.startTimer();
        return;
      }

      if (this.mode === "off") {
        this.peers = [];
        this.selfId = 0;
        this.hostId = 0;
        this.waiting = [];
        lastReady = null;
        holding = false;
        resumeAfterWait = false;
        this.stopTimer();
      }
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
        if (relayActive.value) {
          await leaveRoom();
        } else {
          this.apply(await togetherApi.leave());
        }
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
      if (relayActive.value) {
        void pushRelay(payload as unknown as Record<string, unknown>);
        return;
      }
      if (this.mode === "off") return;
      void togetherApi.send(payload).catch((e: unknown) => {
        this.note(
          formatLocal("ui", reason(e, "не удалось отправить сообщение")),
        );
      });
    },

    push(full = true) {
      if (!this.isHost) return;

      lastSent = Date.now();
      this.send(buildState(usePlayerStore(), full));
    },

    onLocalChange(why: string) {
      if (applying || !this.isHost) return;

      this.push();
      this.note(formatLocal("ui", `${why}: разослал комнате`));
    },

    async handoff(id: number) {
      if (relayActive.value) {
        if (!this.isHost || this.busy) return;

        const target = this.peers.find((item) => item.id === id);
        if (!target || id === this.hostId) return;

        const agree = await askConfirm({
          title: `Передать комнату: ${target.nick}?`,
          message:
            "Участник станет хостом и будет вести плеер. Остальные, включая вас, продолжат слушать уже его.",
          okLabel: "Передать",
        });
        if (!agree) return;

        await handoffRelay(id);
        this.note(formatLocal("ui", `передаю комнату: ${target.nick}`));
        Notify.create({ message: `Комната переезжает к ${target.nick}` });
        return;
      }

      if (this.mode !== "host" || id === HOST_ID || this.busy) return;

      const peer = this.peers.find((item) => item.id === id);
      if (!peer) return;

      const ok = await askConfirm({
        title: `Передать комнату: ${peer.nick}?`,
        message:
          "Участник станет хостом и будет вести плеер. Остальные, включая вас, переподключатся к нему автоматически.",
        okLabel: "Передать",
      });
      if (!ok) return;

      let address: string | null = null;
      try {
        address = await togetherApi.peerAddress(id);
      } catch (e) {
        this.note(formatLocal("ui", reason(e, "адрес участника не узнать")));
      }

      if (!address) {
        this.error = `Не видно адреса участника ${peer.nick}`;
        Notify.create({ message: this.error });
        return;
      }

      const target = `${address}:${this.port}`;
      handoffAt = Date.now();

      this.send({
        kind: "handoff",
        to: id,
        nick: peer.nick,
        address,
        port: this.port,
      });
      this.note(formatLocal("ui", `передаю комнату: ${peer.nick} (${target})`));
      Notify.create({ message: `Комната переезжает к ${peer.nick}` });

      await wait(HANDOFF_DELAY / 2);
      await this.leave();
      await wait(HANDOFF_DELAY);
      await this.follow(target);
    },

    async takeOver(port: number) {
      handoffAt = Date.now();

      await this.leave();
      await wait(200);
      await this.host(port);

      if (this.mode !== "host") {
        this.error = "Не удалось поднять комнату у себя";
        Notify.create({ message: this.error });
        return;
      }

      this.note(formatLocal("ui", "комната теперь у нас"));
      this.push();
    },

    async follow(target: string) {
      const done = await reconnect(target, {
        join: (address) => this.join(address),
        joined: () => this.mode === "guest",
        note: (text) => this.note(formatLocal("ui", text)),
      });

      if (done) {
        this.error = null;
        return;
      }

      this.error = `Не удалось подключиться к новому хосту (${target})`;
      this.note(formatLocal("ui", this.error));
      Notify.create({ message: this.error });
    },

    reportReady(ready: boolean) {
      if (!this.isGuest || lastReady === ready) return;

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
      if (!this.isHost || holding) return;

      const player = usePlayerStore();
      holding = true;
      resumeAfterWait = player.isPlaying;
      if (player.isPlaying) player.toggle();

      this.push(false);
      this.note(formatLocal("ui", "ждём загрузку у участников"));
    },

    release() {
      if (!holding) return;

      const player = usePlayerStore();
      holding = false;
      if (resumeAfterWait && !player.isPlaying) player.toggle();
      resumeAfterWait = false;

      this.push(false);
      this.note(formatLocal("ui", "все загрузились, продолжаем"));
    },

    async receive(message: TogetherMessage) {
      const payload = message.payload;
      if (!payload) return;

      if (payload.kind === "note") {
        this.note(formatLocal("net", `${message.nick}: ${payload.text}`));
        Notify.create({ message: `${message.nick}: ${payload.text}` });
        return;
      }

      if (payload.kind === "handoff") {
        await this.onHandoff(
          payload.to,
          payload.nick,
          payload.address,
          payload.port,
        );
        return;
      }

      if (payload.kind === "ready") {
        if (!this.isHost) return;
        this.setWaiting(message.from, !payload.ready, message.nick);
        return;
      }

      if (payload.kind !== "state" || applying) return;
      if (!this.isGuest) return;

      const player = usePlayerStore();
      if (payload.trackId && payload.trackId !== player.current?.id) {
        this.reportReady(false);
      }

      applying = true;
      try {
        await applyState(player, payload);
      } catch (e) {
        this.error = reason(e, "Не удалось синхронизироваться");
        this.note(formatLocal("ui", this.error));
      } finally {
        applying = false;
      }

      this.reportReady(!player.loading);
    },

    async onHandoff(to: number, nick: string, address: string, port: number) {
      if (this.mode !== "guest") return;
      handoffAt = Date.now();

      if (to === this.selfId) {
        this.note(formatLocal("ui", "хост передал комнату нам"));
        Notify.create({ message: "Теперь хостите вы" });
        await this.takeOver(port);
        return;
      }

      const target = `${address}:${port}`;
      this.note(formatLocal("ui", `комната переезжает к ${nick} (${target})`));
      Notify.create({ message: `Теперь хостит ${nick}` });

      await wait(HANDOFF_DELAY);
      await this.follow(target);
    },

    startTimer() {
      if (timer !== null) return;
      timer = window.setInterval(() => this.tick(), HEARTBEAT_MS);
    },

    stopTimer() {
      if (timer === null) return;
      window.clearInterval(timer);
      timer = null;
    },

    tick() {
      if (this.isGuest) {
        this.reportReady(!usePlayerStore().loading);
        return;
      }

      if (!this.isHost) return;
      if (Date.now() - lastSent > RESEND_MS) this.push(false);
    },
  },
});
