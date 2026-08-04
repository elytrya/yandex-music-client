import { defineStore } from "pinia";
import type { Track } from "@/api/types";
import { artistNames } from "@/lib/format";

const KEY = "mashiro.stats";
const LIMIT = 4000;
const MIN_SECONDS = 20;

export interface PlayEvent {
  id: string;
  title: string;
  artist: string;
  artistId: string;
  cover: string | null;
  at: number;
  seconds: number;
}

export interface RankRow {
  key: string;
  label: string;
  sub: string;
  cover: string | null;
  plays: number;
  seconds: number;
}

function startOfDay(value: number | Date): number {
  const d = new Date(value);
  return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
}

function periodStart(days: number): number {
  return startOfDay(Date.now()) - Math.max(0, days - 1) * 86400000;
}

function dayKey(value: number | Date): string {
  const d = new Date(value);
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${d.getFullYear()}-${month}-${day}`;
}

function counts(seconds: number): number {
  return seconds >= MIN_SECONDS ? 1 : 0;
}

function load(): PlayEvent[] {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed as PlayEvent[];
  } catch {
    return [];
  }
}

let saveTimer: number | null = null;

export const useStatsStore = defineStore("stats", {
  state: () => ({
    events: load() as PlayEvent[],
    activeId: "" as string,
  }),

  getters: {
    totalSeconds:
      (s) =>
      (days: number): number => {
        const from = periodStart(days);
        return s.events.reduce(
          (sum, e) => (e.at >= from ? sum + e.seconds : sum),
          0,
        );
      },

    totalPlays:
      (s) =>
      (days: number): number => {
        const from = periodStart(days);
        return s.events.filter((e) => e.at >= from && e.seconds >= MIN_SECONDS)
          .length;
      },

    uniqueTracks:
      (s) =>
      (days: number): number => {
        const from = periodStart(days);
        const ids = new Set<string>();
        for (const e of s.events) {
          if (e.at >= from && e.seconds >= MIN_SECONDS) ids.add(e.id);
        }
        return ids.size;
      },

    activeDays:
      (s) =>
      (days: number): number => {
        const from = periodStart(days);
        const seen = new Set<string>();
        for (const e of s.events) {
          if (e.at >= from && e.seconds > 0) seen.add(dayKey(e.at));
        }
        return seen.size;
      },

    topTracks:
      (s) =>
      (days: number, limit = 10): RankRow[] => {
        const from = periodStart(days);
        const map = new Map<string, RankRow>();
        for (const e of s.events) {
          if (e.at < from) continue;
          const row = map.get(e.id);
          if (row) {
            row.plays += counts(e.seconds);
            row.seconds += e.seconds;
          } else {
            map.set(e.id, {
              key: e.id,
              label: e.title,
              sub: e.artist,
              cover: e.cover,
              plays: counts(e.seconds),
              seconds: e.seconds,
            });
          }
        }
        return [...map.values()]
          .sort((a, b) => b.seconds - a.seconds)
          .slice(0, limit);
      },

    topArtists:
      (s) =>
      (days: number, limit = 10): RankRow[] => {
        const from = periodStart(days);
        const map = new Map<string, RankRow>();
        for (const e of s.events) {
          if (e.at < from) continue;
          const key = e.artist || "Без исполнителя";
          const row = map.get(key);
          if (row) {
            row.plays += counts(e.seconds);
            row.seconds += e.seconds;
            if (!row.cover) row.cover = e.cover;
          } else {
            map.set(key, {
              key: e.artistId || key,
              label: key,
              sub: "",
              cover: e.cover,
              plays: counts(e.seconds),
              seconds: e.seconds,
            });
          }
        }
        return [...map.values()]
          .sort((a, b) => b.seconds - a.seconds)
          .slice(0, limit);
      },

    daily:
      (s) =>
      (
        days: number,
      ): Array<{ day: string; label: string; seconds: number }> => {
        const out: Array<{ day: string; label: string; seconds: number }> = [];
        const first = periodStart(days);
        for (let i = 0; i < days; i++) {
          const d = new Date(first + i * 86400000);
          out.push({
            day: dayKey(d),
            label: `${d.getDate()}.${String(d.getMonth() + 1).padStart(2, "0")}`,
            seconds: 0,
          });
        }
        const index = new Map(out.map((row) => [row.day, row]));
        for (const e of s.events) {
          const row = index.get(dayKey(e.at));
          if (row) row.seconds += e.seconds;
        }
        return out;
      },
  },

  actions: {
    persist(force = false) {
      if (saveTimer !== null && !force) return;
      const write = () => {
        saveTimer = null;
        try {
          localStorage.setItem(KEY, JSON.stringify(this.events));
        } catch {
          this.events = this.events.slice(-1000);
        }
      };
      if (force) {
        if (saveTimer !== null) window.clearTimeout(saveTimer);
        write();
        return;
      }
      saveTimer = window.setTimeout(write, 4000);
    },

    begin(track: Track) {
      const last = this.events[this.events.length - 1];
      if (last && last.id === track.id && Date.now() - last.at < 4000) return;
      this.events.push({
        id: track.id,
        title: track.title,
        artist: artistNames(track.artists),
        artistId: track.artists[0]?.id ?? "",
        cover: track.cover_url,
        at: Date.now(),
        seconds: 0,
      });
      if (this.events.length > LIMIT)
        this.events = this.events.slice(-Math.floor(LIMIT * 0.8));
      this.activeId = track.id;
      this.persist(true);
    },

    tick(seconds: number) {
      if (!(seconds > 0) || seconds > 6) return;
      const last = this.events[this.events.length - 1];
      if (!last || (this.activeId && last.id !== this.activeId)) return;
      last.seconds += seconds;
      this.persist();
    },

    clear() {
      this.events = [];
      this.activeId = "";
      this.persist(true);
    },
  },
});
