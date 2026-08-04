import { defineStore } from "pinia";
import { Notify } from "quasar";
import { api } from "@/api/client";
import type { Lyrics, Quality, RepeatMode, Track } from "@/api/types";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  audio,
  cancelFade,
  currentLevel,
  ensureAnalyser,
  fadeIn,
  fadeOut,
  isCurrentToken,
  nextSourceToken,
  resumeAudioContext,
  safePlay,
} from "@/lib/audio";
import { readCache, releaseMemoryCache, swr, writeCache } from "@/lib/cache";
import { proxyStream } from "@/lib/stream";
import { censorUrl, ensureCensorList } from "@/lib/censor";
import { createLogger } from "@/lib/log";
import { useEqualizerStore } from "@/stores/equalizer";
import { useLibraryStore } from "@/stores/library";
import { useSleepStore } from "@/stores/sleep";
import { useStatsStore } from "@/stores/stats";
import { DEFAULT_DISCORD_CLIENT_ID, useUiStore } from "@/stores/ui/index";
import { artistNames } from "@/lib/format";
import { setTrayTooltip } from "@/lib/tray";
import { resetPresenceSignature, syncDiscordPresence } from "./presence";
import { loadSession, saveSession } from "./session";
import type { PlaybackSettings } from "./settings";
import { loadPlaybackSettings, savePlaybackSettings } from "./settings";

const DEFAULT_STATION = "user:onyourwave";
const SEEN_LIMIT = 600;
const WAVE_HISTORY_KEY = "wave.history.v1";
const WAVE_HISTORY_LIMIT = 500;
const PREFETCH_LIMIT = 4;

const log = createLogger("player");

interface PrefetchEntry {
  url: string;
  codec: string | null;
  bitrate: number | null;
  local: boolean;
  censored: boolean;
}

const prefetched = new Map<string, PrefetchEntry>();
const warmup = new Audio();
warmup.preload = "auto";
warmup.muted = true;
warmup.volume = 0;

function rememberPrefetch(id: string, entry: PrefetchEntry): void {
  prefetched.delete(id);
  prefetched.set(id, entry);
  while (prefetched.size > PREFETCH_LIMIT) {
    const oldest = prefetched.keys().next().value;
    if (oldest === undefined) break;
    prefetched.delete(oldest);
  }
}

interface PlayerState {
  queue: Track[];
  sourceQueue: Track[];
  index: number;
  current: Track | null;
  isPlaying: boolean;
  progress: number;
  duration: number;
  volume: number;
  muted: boolean;
  quality: Quality;
  playbackRate: number;
  repeat: RepeatMode;
  shuffle: boolean;
  currentCodec: string | null;
  currentBitrate: number | null;
  playingLocal: boolean;
  censorReplaced: boolean;
  loading: boolean;
  waveBatchId: string | null;
  isWave: boolean;
  stationId: string | null;
  stationName: string | null;
  fetchingMore: boolean;
  seenIds: string[];
  lyrics: Lyrics | null;
  lyricsLoading: boolean;
  lyricsError: string | null;
  showLyrics: boolean;
  waveError: string | null;
  presenceError: string | null;
  presenceConnected: boolean;
  presenceReconnecting: boolean;
  presenceUser: string | null;
  pendingResume: boolean;
}

let presenceTimer: number | null = null;
let lastSessionSave = 0;
let lastProgress = 0;
let quietSince = 0;

export const usePlayerStore = defineStore("player", {
  state: (): PlayerState => {
    const s = loadPlaybackSettings();
    return {
      queue: [],
      sourceQueue: [],
      index: -1,
      current: null,
      isPlaying: false,
      progress: 0,
      duration: 0,
      volume: s.volume,
      muted: s.muted,
      quality: s.quality,
      playbackRate: s.playbackRate,
      repeat: s.repeat,
      shuffle: false,
      currentCodec: null,
      currentBitrate: null,
      playingLocal: false,
      censorReplaced: false,
      loading: false,
      waveBatchId: null,
      isWave: false,
      stationId: null,
      stationName: null,
      fetchingMore: false,
      seenIds: [],
      lyrics: null,
      lyricsLoading: false,
      lyricsError: null,
      showLyrics: false,
      waveError: null,
      presenceError: null,
      presenceConnected: false,
      presenceReconnecting: false,
      presenceUser: null,
      pendingResume: false,
    };
  },

  getters: {
    hasNext: (s) => s.index < s.queue.length - 1 || s.repeat === "all",
    hasPrev: (s) => s.index > 0,
    waveActive: (s) => s.isWave && Boolean(s.current),
    qualityLabel: (s) => {
      if (!s.currentCodec) return "";
      const codec = s.currentCodec.toUpperCase();
      return s.currentBitrate ? `${codec} ${s.currentBitrate} kbps` : codec;
    },
    activeLine: (s) => {
      const lines = s.lyrics?.lines || [];
      if (!s.lyrics?.synced || !lines.length) return -1;
      const ms = s.progress * 1000;
      let found = -1;
      for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        if (line && line.time_ms <= ms + 150) found = i;
        else break;
      }
      return found;
    },
  },

  actions: {
    persist() {
      const payload: PlaybackSettings = {
        volume: this.volume,
        quality: this.quality,
        playbackRate: this.playbackRate,
        repeat: this.repeat,
        muted: this.muted,
      };
      savePlaybackSettings(payload);
    },

    bind() {
      if (useUiStore().settings.censorBypass) void ensureCensorList();
      audio.volume = this.muted ? 0 : this.volume;
      audio.playbackRate = this.playbackRate;
      if (useUiStore().settings.trimSilence) ensureAnalyser();
      audio.ontimeupdate = () => {
        const previous = lastProgress;
        this.progress = audio.currentTime;
        this.duration = audio.duration || 0;
        lastProgress = this.progress;

        const delta = this.progress - previous;
        if (delta > 0 && delta < 6 && !audio.paused) {
          useStatsStore().tick(delta);
        }

        const settings = useUiStore().settings;
        if (settings.trimSilence && this.duration > 30 && !audio.paused) {
          const left = this.duration - this.progress;
          if (left > 1.5 && left < 25) {
            if (currentLevel() < 0.004) {
              if (quietSince === 0) quietSince = Date.now();
              else if (Date.now() - quietSince > 1100) {
                quietSince = 0;
                void this.next(true);
              }
            } else {
              quietSince = 0;
            }
          } else {
            quietSince = 0;
          }
        }

        const now = Date.now();
        if (now - lastSessionSave > 5000) {
          lastSessionSave = now;
          this.persistSession();
        }
      };
      audio.onended = () => {
        if (useSleepStore().onTrackEnded()) return;
        void this.next(true);
      };
      audio.onplay = () => {
        this.isPlaying = true;
        resumeAudioContext();
        void this.syncPresence();
        void this.syncTray();
      };
      audio.onpause = () => {
        this.isPlaying = false;
        void this.syncPresence();
        this.persistSession();
        void this.syncTray();
      };

      useEqualizerStore().apply();

      if (presenceTimer === null) {
        presenceTimer = window.setInterval(() => {
          void this.syncPresence();
        }, 15000);
      }
    },

    async syncPresence(force = false) {
      const error = await syncDiscordPresence(
        {
          track: this.current,
          isPlaying: this.isPlaying,
          progress: this.progress,
          duration: this.duration,
          playbackRate: this.playbackRate,
        },
        { force },
      );
      this.presenceError = error;
      void this.refreshPresenceStatus();
      return error;
    },

    async refreshPresenceStatus() {
      try {
        const status = await api.discordPresenceStatus();
        this.presenceConnected = Boolean(status?.connected);
        this.presenceUser = status?.user ?? null;
        this.presenceError = status?.lastError ?? null;
        log.info("presence status", status);
      } catch (error) {
        this.presenceConnected = false;
        this.presenceUser = null;
        log.warn("presence status failed", error);
      }
      return this.presenceConnected;
    },

    async reconnectPresence() {
      const settings = useUiStore().settings;
      const clientId =
        settings.discordClientId.trim() || DEFAULT_DISCORD_CLIENT_ID;

      this.presenceReconnecting = true;
      try {
        const status = await api.reconnectDiscordPresence(clientId);
        this.presenceConnected = Boolean(status?.connected ?? true);
        this.presenceUser = status?.user ?? null;
        this.presenceError = null;
        log.info("presence reconnected", status);
        resetPresenceSignature();
        const error = await this.syncPresence(true);
        Notify.create(
          error
            ? { type: "negative", message: error }
            : { message: "Discord подключён заново" },
        );
        return error;
      } catch (error) {
        const message =
          error instanceof Error ? error.message : "Discord недоступен";
        this.presenceConnected = false;
        this.presenceError = message;
        Notify.create({ type: "negative", message });
        return message;
      } finally {
        this.presenceReconnecting = false;
      }
    },

    async testPresence() {
      resetPresenceSignature();
      const error = await this.syncPresence(true);
      if (error) {
        Notify.create({ type: "negative", message: error });
      } else if (!this.current) {
        Notify.create({ message: "Включи трек - статус появится в Discord" });
      } else {
        Notify.create({ message: "Статус отправлен в Discord" });
      }
    },

    persistSession() {
      if (!this.current || !this.queue.length) return;
      saveSession({
        queue: this.queue,
        sourceQueue: this.sourceQueue.length ? this.sourceQueue : this.queue,
        shuffle: this.shuffle,
        index: this.index,
        progress: this.progress,
        isWave: this.isWave,
        stationId: this.stationId,
        stationName: this.stationName,
        savedAt: Date.now(),
      });
    },

    async restoreSession(autoplay: boolean) {
      if (this.current) return false;
      const data = loadSession();
      if (!data) return false;
      const track = data.queue[data.index];
      if (!track) return false;
      this.queue = data.queue;
      this.sourceQueue = data.sourceQueue.length
        ? [...data.sourceQueue]
        : [...data.queue];
      this.shuffle = data.shuffle;
      this.index = data.index;
      this.isWave = data.isWave;
      this.stationId = data.stationId;
      this.stationName = data.stationName;
      this.seenIds = data.queue.map((t) => t.id).slice(-2000);
      this.current = track;
      this.duration = (track.duration_ms ?? 0) / 1000;
      this.progress = data.progress;
      this.pendingResume = true;
      void this.syncTray();
      log.info("session restored", track.title, data.progress);
      if (autoplay) {
        await this.loadCurrent({ startAt: data.progress, autoplay: true });
      }
      return true;
    },

    async syncTray() {
      const track = this.current;
      const text = track
        ? `${track.title} - ${artistNames(track.artists)}`
        : "Mashiro";
      await setTrayTooltip(text);
    },

    releaseMemory() {
      prefetched.clear();
      warmup.removeAttribute("src");
      warmup.load();
      releaseMemoryCache();
      this.lyrics = null;
      this.showLyrics = false;
      if (this.queue.length > 120) {
        const from = Math.max(0, this.index - 20);
        const slice = this.queue.slice(from, from + 120);
        this.index = Math.max(0, this.index - from);
        this.queue = slice;
        this.sourceQueue = [...slice];
      }
      if (this.seenIds.length > 2000) this.seenIds = this.seenIds.slice(-2000);
      log.info("memory released");
    },

    remember(tracks: Track[]) {
      for (const t of tracks) this.seenIds.push(t.id);
      try {
        const hist = readCache<string[]>(WAVE_HISTORY_KEY) ?? [];
        const set = new Set(hist);
        for (const t of tracks) {
          if (!set.has(t.id)) {
            set.add(t.id);
            hist.push(t.id);
          }
        }
        writeCache(WAVE_HISTORY_KEY, hist.slice(-WAVE_HISTORY_LIMIT));
      } catch {}
      if (this.seenIds.length > SEEN_LIMIT) {
        this.seenIds = this.seenIds.slice(-SEEN_LIMIT);
      }
    },

    async startWave(stationId?: string, stationName?: string) {
      this.loading = true;
      this.waveError = null;
      const station = stationId ?? this.stationId ?? DEFAULT_STATION;
      if (station !== this.stationId) this.seenIds = [];
      try {
        await api
          .sendFeedback({ type: "radioStarted", station_id: station })
          .catch(() => undefined);

        const cachedLast = readCache<Track[]>(`wave.last.${station}`) ?? [];
        const history = readCache<string[]>(WAVE_HISTORY_KEY) ?? [];
        let cursor: string | undefined =
          this.seenIds[this.seenIds.length - 1] ??
          history[history.length - 1] ??
          cachedLast[cachedLast.length - 1]?.id;

        const recent = new Set<string>([...history, ...this.seenIds]);
        const fresh: Track[] = [];
        let batchId: string | null = null;
        let fallback: Track[] = [];

        for (let attempt = 0; attempt < 6 && fresh.length < 8; attempt++) {
          const res = await api.wave(cursor, station);
          if (!res.tracks.length) break;
          batchId = res.batch_id ?? batchId;
          if (!fallback.length) fallback = res.tracks;
          for (const t of res.tracks) {
            if (recent.has(t.id) || fresh.some((c) => c.id === t.id)) continue;
            recent.add(t.id);
            fresh.push(t);
          }
          const nextCursor = res.tracks[res.tracks.length - 1]?.id;
          if (!nextCursor || nextCursor === cursor) break;
          cursor = nextCursor;
        }

        let tracks = fresh;
        if (!tracks.length) {
          const seenInBatch = new Set<string>();
          const deduped = fallback.filter((t) => {
            if (seenInBatch.has(t.id)) return false;
            seenInBatch.add(t.id);
            return true;
          });
          const offset = deduped.length
            ? Math.floor(Date.now() / 1000) % deduped.length
            : 0;
          tracks = offset
            ? [...deduped.slice(offset), ...deduped.slice(0, offset)]
            : deduped;
        }
        if (!tracks.length) throw new Error("Ротор вернул пустой плейлист");

        this.stationId = station;
        if (stationName) this.stationName = stationName;
        else if (station === DEFAULT_STATION) this.stationName = "Моя волна";
        this.waveBatchId = batchId;
        this.queue = tracks;
        this.sourceQueue = tracks;
        this.shuffle = false;
        this.remember(tracks);
        this.index = 0;
        this.isWave = true;
        writeCache(`wave.last.${station}`, tracks.slice(0, 30));
        await this.loadCurrent();
      } catch (e) {
        const message =
          e instanceof Error ? e.message : "Не удалось запустить волну";
        this.waveError = message;
        Notify.create({ type: "negative", message });
      } finally {
        this.loading = false;
      }
    },

    async playQueue(
      tracks: Track[],
      startIndex = 0,
      opts?: { wave?: boolean; batchId?: string | null },
    ) {
      this.sourceQueue = tracks;
      this.queue = tracks;
      this.shuffle = false;
      this.index = startIndex;
      this.isWave = opts?.wave ?? false;
      this.waveBatchId = opts?.batchId ?? null;
      await this.loadCurrent();
    },

    async loadCurrent(opts?: { startAt?: number; autoplay?: boolean }) {
      const track = this.queue[this.index];
      if (!track) return;
      const startAt = Math.max(0, opts?.startAt ?? 0);
      const autoplay = opts?.autoplay ?? true;
      this.pendingResume = false;
      this.current = track;
      this.progress = startAt;
      lastProgress = startAt;
      quietSince = 0;
      useStatsStore().begin(track);
      this.duration = (track.duration_ms ?? 0) / 1000;
      this.loading = true;
      this.lyrics = null;
      this.lyricsError = null;
      const token = nextSourceToken();
      const ui = useUiStore().settings;
      const fadeMs = ui.crossfadeEnabled
        ? Math.round(Math.max(0, ui.crossfadeSeconds) * 1000)
        : 0;
      try {
        if (fadeMs > 0) await fadeOut(Math.min(fadeMs, 1200));

        let source: string | null = null;
        this.playingLocal = false;
        this.censorReplaced = false;

        if (ui.censorBypass) {
          await ensureCensorList();
          const replacement = censorUrl(track.id);
          if (replacement) {
            source = replacement;
            this.censorReplaced = true;
            this.currentCodec = null;
            this.currentBitrate = null;
            log.info("source from FckCensor", track.title);
          }
        }

        const ready = source ? undefined : prefetched.get(track.id);
        if (ready) {
          prefetched.delete(track.id);
          source = ready.url;
          this.playingLocal = ready.local;
          this.censorReplaced = ready.censored;
          this.currentCodec = ready.codec;
          this.currentBitrate = ready.bitrate;
          log.info("source from prefetch", track.title);
        }

        if (!source && ui.preferLocalFiles) {
          const local = await api
            .findLocalTrack(track.id, ui.downloadDir || null)
            .catch(() => null);
          if (local) {
            source = convertFileSrc(local);
            this.playingLocal = true;
            this.currentCodec = local.split(".").pop() ?? null;
            this.currentBitrate = null;
            log.info("source from disk", local);
          }
        }

        if (!source) {
          const stream = await log.time(`stream ${track.id}`, () =>
            api.stream(track.id, this.quality),
          );
          if (!isCurrentToken(token)) return;
          this.currentCodec = stream.codec;
          this.currentBitrate = stream.bitrate;
          source = proxyStream(stream.url);
          log.info("source from network", stream.codec, stream.bitrate);
        }

        if (!isCurrentToken(token)) return;
        audio.src = source;
        audio.playbackRate = this.playbackRate;
        const target = this.muted ? 0 : this.volume;
        if (fadeMs > 0 && autoplay) audio.volume = 0;
        else audio.volume = target;
        useEqualizerStore().apply();
        if (startAt > 0) {
          const seekOnce = () => {
            audio.removeEventListener("loadedmetadata", seekOnce);
            try {
              audio.currentTime = startAt;
            } catch {}
          };
          audio.addEventListener("loadedmetadata", seekOnce);
        }
        if (autoplay) {
          await safePlay(token);
          if (fadeMs > 0 && isCurrentToken(token)) {
            void fadeIn(target, Math.min(fadeMs, 2500));
          }
        } else {
          audio.load();
          this.isPlaying = false;
        }
        this.persistSession();
        void this.syncTray();
        void this.syncPresence(true);
        void this.prefetchNext();
        if (this.isWave) {
          void api.sendFeedback({
            type: "trackStarted",
            track_id: track.id,
            batch_id: this.waveBatchId,
            station_id: this.stationId ?? undefined,
          });
        }
        if (this.showLyrics) void this.loadLyrics();
      } catch (e) {
        const name = (e as DOMException | undefined)?.name;
        log.error("loadCurrent failed", e);
        if (!isCurrentToken(token) || name === "AbortError") return;
        Notify.create({
          type: "negative",
          message:
            e instanceof Error ? e.message : "Не удалось воспроизвести трек",
        });
      } finally {
        this.loading = false;
      }
    },

    async prefetchNext() {
      const next = this.queue[this.index + 1];
      if (!next || prefetched.has(next.id)) return;
      const ui = useUiStore().settings;
      if (ui.censorBypass) {
        const censored = censorUrl(next.id);
        if (censored) {
          rememberPrefetch(next.id, {
            url: censored,
            codec: null,
            bitrate: null,
            local: false,
            censored: true,
          });
          warmup.src = proxyStream(censored);
          warmup.load();
          return;
        }
      }
      try {
        if (ui.preferLocalFiles) {
          const local = await api
            .findLocalTrack(next.id, ui.downloadDir || null)
            .catch(() => null);
          if (local) {
            rememberPrefetch(next.id, {
              url: convertFileSrc(local),
              codec: local.split(".").pop() ?? null,
              bitrate: null,
              local: true,
              censored: false,
            });
            log.info("prefetched from disk", next.title);
            return;
          }
        }

        const stream = await api.stream(next.id, this.quality);
        rememberPrefetch(next.id, {
          url: proxyStream(stream.url),
          codec: stream.codec,
          bitrate: stream.bitrate,
          local: false,
          censored: false,
        });
        warmup.src = proxyStream(stream.url);
        warmup.load();
        log.info("prefetched from network", next.title, stream.codec);
      } catch (error) {
        log.warn("prefetch failed", next.title, error);
      }
    },

    async loadLyrics() {
      const track = this.current;
      if (!track || this.lyrics?.track_id === track.id) return;
      this.lyricsLoading = true;
      this.lyricsError = null;

      const key = `lyrics.${track.id}`;
      const cached = readCache<Lyrics>(key);
      if (cached) {
        this.lyrics = cached;
        this.lyricsLoading = false;
      }

      await swr<Lyrics>(key, () => api.lyrics(track.id), {
        onData: (data) => {
          if (this.current?.id !== track.id) return;
          this.lyrics = data;
          this.lyricsError = null;
        },
        onError: () => {
          if (this.current?.id !== track.id || this.lyrics) return;
          this.lyrics = null;
          this.lyricsError = "Текста для этого трека нет";
        },
        onSettled: () => {
          this.lyricsLoading = false;
        },
      });
    },

    toggleLyrics() {
      if (!this.current) {
        this.showLyrics = false;
        return;
      }
      this.showLyrics = !this.showLyrics;
      if (this.showLyrics) void this.loadLyrics();
    },

    closeLyrics() {
      this.showLyrics = false;
    },

    async setQuality(quality: Quality) {
      if (quality === this.quality) return;
      this.quality = quality;
      this.persist();
      if (!this.current) return;
      const position = audio.currentTime;
      const wasPlaying = !audio.paused;
      try {
        const stream = await api.stream(this.current.id, quality);
        this.currentCodec = stream.codec;
        this.currentBitrate = stream.bitrate;
        audio.src = proxyStream(stream.url);
        audio.currentTime = position;
        audio.playbackRate = this.playbackRate;
        if (wasPlaying) await safePlay();
      } catch {
        Notify.create({
          type: "negative",
          message: "Не удалось сменить качество",
        });
      }
    },

    setPlaybackRate(rate: number) {
      this.playbackRate = rate;
      audio.playbackRate = rate;
      this.persist();
      void this.syncPresence(true);
    },

    cycleRepeat() {
      const order: RepeatMode[] = ["off", "all", "one"];
      const next = order[(order.indexOf(this.repeat) + 1) % order.length];
      this.repeat = next ?? "off";
      this.persist();
    },

    setRepeat(mode: RepeatMode) {
      this.repeat = mode;
      this.persist();
    },

    toggleShuffle() {
      if (!this.queue.length) return;
      if (this.shuffle) {
        const currentId = this.current?.id;
        this.queue = [...this.sourceQueue];
        this.index = Math.max(
          0,
          this.queue.findIndex((t) => t.id === currentId),
        );
        this.shuffle = false;
        return;
      }

      const rest = this.queue.filter((_, i) => i !== this.index);
      for (let i = rest.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        const a = rest[i];
        const b = rest[j];
        if (a && b) {
          rest[i] = b;
          rest[j] = a;
        }
      }
      const head = this.queue[this.index];
      this.queue = head ? [head, ...rest] : rest;
      this.index = 0;
      this.shuffle = true;
    },

    toggle() {
      if (!this.current) return;
      if (this.pendingResume) {
        void this.loadCurrent({ startAt: this.progress, autoplay: true });
        return;
      }
      if (audio.paused) void safePlay();
      else audio.pause();
    },

    async next(auto = false) {
      if (auto && this.repeat === "one") {
        audio.currentTime = 0;
        await safePlay();
        return;
      }

      if (this.isWave && this.current) {
        await api
          .sendFeedback({
            type: auto ? "trackFinished" : "skip",
            track_id: this.current.id,
            batch_id: this.waveBatchId,
            station_id: this.stationId ?? undefined,
            total_played_seconds: this.progress,
          })
          .catch(() => undefined);
      }

      if (this.isWave && this.index >= this.queue.length - 2) {
        await this.extendWave();
      }

      if (this.index < this.queue.length - 1) {
        this.index += 1;
        await this.loadCurrent();
        return;
      }

      if (this.repeat === "all" && this.queue.length) {
        this.index = 0;
        await this.loadCurrent();
        return;
      }

      this.isPlaying = false;
      audio.pause();
    },

    async extendWave() {
      if (this.fetchingMore) return 0;
      this.fetchingMore = true;
      let added = 0;
      try {
        for (let attempt = 0; attempt < 4 && added === 0; attempt++) {
          const last = this.queue[this.queue.length - 1];
          const res = await api.wave(last?.id, this.stationId ?? undefined);
          if (res.batch_id) this.waveBatchId = res.batch_id;

          const known = new Set([
            ...this.seenIds,
            ...this.queue.map((t) => t.id),
          ]);
          const fresh: Track[] = [];
          for (const t of res.tracks) {
            if (known.has(t.id)) continue;
            known.add(t.id);
            fresh.push(t);
          }
          if (fresh.length) {
            this.queue.push(...fresh);
            this.sourceQueue.push(...fresh);
            this.remember(fresh);
            added = fresh.length;
          }
        }
      } catch {
        added = 0;
      } finally {
        this.fetchingMore = false;
      }
      return added;
    },

    async prev() {
      if (this.progress > 3) {
        audio.currentTime = 0;
        return;
      }
      if (this.hasPrev) {
        this.index -= 1;
        await this.loadCurrent();
      }
    },

    seek(seconds: number) {
      audio.currentTime = seconds;
      this.progress = seconds;
      void this.syncPresence(true);
    },

    setVolume(v: number) {
      this.volume = v;
      this.muted = v === 0;
      audio.volume = v;
      this.persist();
    },

    toggleMute() {
      this.muted = !this.muted;
      audio.volume = this.muted ? 0 : this.volume;
      this.persist();
    },

    playNext(track: Track) {
      if (!this.queue.length) {
        void this.playQueue([track], 0);
        return;
      }
      this.queue.splice(this.index + 1, 0, track);
      this.sourceQueue = [...this.queue];
      Notify.create({ message: "Играет следующим" });
    },

    enqueue(track: Track) {
      if (!this.queue.length) {
        void this.playQueue([track], 0);
        return;
      }
      this.queue.push(track);
      this.sourceQueue = [...this.queue];
      Notify.create({ message: "Добавил в конец очереди" });
    },

    async waveByTrack(track: Track) {
      await this.startWave(`track:${track.id}`, `Волна по «${track.title}»`);
    },

    async like() {
      if (!this.current) return;
      await useLibraryStore().toggleLike(this.current);
    },

    async dislike() {
      if (!this.current) return;
      const track = this.current;
      await useLibraryStore().dislike(track);
      if (this.isWave) {
        await api
          .sendFeedback({
            type: "dislike",
            track_id: track.id,
            station_id: this.stationId ?? undefined,
          })
          .catch(() => undefined);
      }
      await this.next(false);
    },
  },
});
