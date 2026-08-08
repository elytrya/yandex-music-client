import { defineStore } from "pinia";
import { api } from "@/api/client";
import type {
  GeniusArtist,
  GeniusHit,
  GeniusPersonHit,
  GeniusSong,
  Track,
} from "@/api/types";
import { artistNames } from "@/lib/format";

const STORAGE_KEY = "mashiro.genius";

export interface GeniusSettings {
  token: string;
  enabled: boolean;
  showQuotes: boolean;
}

const defaults: GeniusSettings = {
  token: "",
  enabled: false,
  showQuotes: true,
};

function readSettings(): GeniusSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...defaults };
    const saved = JSON.parse(raw) as Partial<GeniusSettings>;
    return { ...defaults, ...saved };
  } catch {
    return { ...defaults };
  }
}

function message(error: unknown): string {
  const text = error instanceof Error ? error.message : String(error);
  return text || "Genius не ответил";
}

export const useGeniusStore = defineStore("genius", {
  state: () => ({
    ...readSettings(),

    song: null as GeniusSong | null,
    songKey: "",
    loading: false,
    error: "",

    hits: [] as GeniusHit[],
    hitsLoading: false,

    people: [] as GeniusPersonHit[],
    peopleLoading: false,

    person: null as GeniusArtist | null,
    personLoading: false,
    personError: "",

    checking: false,
    checkResult: "",
  }),

  getters: {
    ready(state): boolean {
      return state.enabled && state.token.trim().length > 0;
    },
    producers(state): GeniusSong["credits"] {
      return (state.song?.credits || []).filter((p) => p.role === "Продюсер");
    },
    writers(state): GeniusSong["credits"] {
      return (state.song?.credits || []).filter((p) => p.role === "Автор");
    },
    people(state): GeniusSong["credits"] {
      const seen = new Set<string>();
      const out: GeniusSong["credits"] = [];
      for (const person of state.song?.credits || []) {
        const key = `${person.id}|${person.role}`;
        if (seen.has(key)) continue;
        seen.add(key);
        out.push(person);
      }
      return out;
    },
  },

  actions: {
    persist() {
      const payload: GeniusSettings = {
        token: this.token,
        enabled: this.enabled,
        showQuotes: this.showQuotes,
      };
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(payload));
      } catch {}
    },

    apply(patch: Partial<GeniusSettings>) {
      if (patch.token !== undefined) this.token = patch.token;
      if (patch.enabled !== undefined) this.enabled = patch.enabled;
      if (patch.showQuotes !== undefined) this.showQuotes = patch.showQuotes;
      this.persist();
      if (patch.token !== undefined || patch.enabled !== undefined) {
        this.forget();
        this.checkResult = "";
      }
    },

    forget() {
      this.song = null;
      this.songKey = "";
      this.error = "";
      this.hits = [];
      this.person = null;
      this.personError = "";
    },

    async refresh(track: Track | null) {
      this.songKey = "";
      await this.fetchSong(track, true);
    },

    async check() {
      this.checking = true;
      this.checkResult = "";
      try {
        this.checkResult = await api.geniusCheck(this.token.trim());
      } catch (error) {
        this.checkResult = message(error);
      } finally {
        this.checking = false;
      }
    },

    async fetchSong(
      track: Track | null,
      force = false,
    ): Promise<GeniusSong | null> {
      if (!track || !this.ready) return null;

      const key = String(track.id);
      if (!force && this.songKey === key) return this.song;

      this.songKey = key;
      this.song = null;
      this.error = "";
      this.person = null;
      this.hits = [];
      this.loading = true;

      try {
        const song = await api.geniusLookup(
          this.token.trim(),
          track.title,
          artistNames(track.artists),
          force,
        );
        if (this.songKey !== key) return null;
        this.song = song;
        if (!song) this.error = "На Genius нет подходящей страницы";
        return song;
      } catch (error) {
        if (this.songKey !== key) return null;
        this.song = null;
        this.error = message(error);
        return null;
      } finally {
        if (this.songKey === key) this.loading = false;
      }
    },

    async lyricsFor(track: Track | null, force = false): Promise<string[]> {
      const song = await this.fetchSong(track, force);
      return song?.lyrics || [];
    },

    async searchHits(query: string) {
      const value = query.trim();
      if (!this.ready || !value) {
        this.hits = [];
        return;
      }
      this.hitsLoading = true;
      try {
        this.hits = await api.geniusSearch(this.token.trim(), value);
      } catch (error) {
        this.hits = [];
        this.error = message(error);
      } finally {
        this.hitsLoading = false;
      }
    },

    async searchPeople(query: string) {
      const value = query.trim();
      if (!this.ready || value.length < 2) {
        this.people = [];
        return;
      }
      this.peopleLoading = true;
      try {
        this.people = await api.geniusSearchPeople(this.token.trim(), value);
      } catch {
        this.people = [];
      } finally {
        this.peopleLoading = false;
      }
    },

    async pickHit(hit: GeniusHit) {
      if (!this.ready) return;
      this.loading = true;
      this.error = "";
      this.hits = [];
      try {
        this.song = await api.geniusSong(this.token.trim(), hit.id);
      } catch (error) {
        this.error = message(error);
      } finally {
        this.loading = false;
      }
    },

    async loadPerson(id: number, force = false) {
      if (!this.ready || !id) return;
      this.personLoading = true;
      this.personError = "";
      this.person = null;
      try {
        this.person = await api.geniusArtist(this.token.trim(), id, force);
      } catch (error) {
        this.personError = message(error);
      } finally {
        this.personLoading = false;
      }
    },

    closePerson() {
      this.person = null;
      this.personError = "";
      this.personLoading = false;
    },

    async clearCache() {
      this.forget();
      try {
        await api.geniusClearCache();
      } catch {}
    },
  },
});
