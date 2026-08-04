import { defineStore } from "pinia";
import { Notify } from "quasar";
import { api } from "@/api/client";
import type { AlbumPage, Playlist, Track } from "@/api/types";

export interface LikedAlbum {
  id: string;
  title: string;
  cover_url: string | null;
  year: number | null;
  artists: string;
  track_count: number;
  liked_at: number;
}

interface LibraryState {
  likedIds: string[];
  likedAlbums: LikedAlbum[];
  playlists: Playlist[];
  pinned: number[];
  loading: boolean;
}

const PIN_KEY = "mashiro.pinned";
const ALBUM_KEY = "mashiro.likedAlbums";

function readLikedAlbums(): LikedAlbum[] {
  try {
    const raw = localStorage.getItem(ALBUM_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (item): item is LikedAlbum =>
        Boolean(item) && typeof (item as LikedAlbum).id === "string",
    );
  } catch {
    return [];
  }
}

function readPinned(): number[] {
  try {
    const raw = localStorage.getItem(PIN_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.map((v) => Number(v)).filter((v) => Number.isFinite(v));
  } catch {
    return [];
  }
}

function notifyError(e: unknown, fallback: string) {
  Notify.create({
    type: "negative",
    message: e instanceof Error ? e.message : fallback,
  });
}

export const useLibraryStore = defineStore("library", {
  state: (): LibraryState => ({
    likedIds: [],
    likedAlbums: readLikedAlbums(),
    playlists: [],
    pinned: readPinned(),
    loading: false,
  }),

  getters: {
    liked: (s) => (id: string) => s.likedIds.includes(id),
    albumLiked: (s) => (id: string) =>
      s.likedAlbums.some((album) => album.id === String(id)),
    sortedLikedAlbums: (s) =>
      [...s.likedAlbums].sort((a, b) => (b.liked_at || 0) - (a.liked_at || 0)),
    ownPlaylists: (s) => s.playlists,
    isPinned: (s) => (kind: number | string) => s.pinned.includes(Number(kind)),
    sortedPlaylists: (s) => {
      const rank = (kind: number) => {
        const at = s.pinned.indexOf(Number(kind));
        return at === -1 ? s.pinned.length + 1 : at;
      };
      return [...s.playlists].sort((a, b) => rank(a.kind) - rank(b.kind));
    },
  },

  actions: {
    async init() {
      if (this.loading) return;
      this.loading = true;
      const [ids, playlists] = await Promise.all([
        api.likedIds().catch(() => [] as string[]),
        api.playlists().catch(() => [] as Playlist[]),
      ]);
      this.likedIds = ids;
      this.playlists = playlists;
      this.loading = false;
    },

    persistLikedAlbums() {
      try {
        localStorage.setItem(ALBUM_KEY, JSON.stringify(this.likedAlbums));
        return true;
      } catch {
        Notify.create({
          type: "negative",
          message: "Не удалось сохранить любимые альбомы",
        });
        return false;
      }
    },

    toggleAlbumLike(album: AlbumPage) {
      const id = String(album.id);
      const already = this.likedAlbums.some((item) => item.id === id);
      if (already) {
        this.likedAlbums = this.likedAlbums.filter((item) => item.id !== id);
      } else {
        const entry: LikedAlbum = {
          id,
          title: album.title,
          cover_url: album.cover_url,
          year: album.year,
          artists: album.artists.map((artist) => artist.name).join(", "),
          track_count: album.tracks.length,
          liked_at: Date.now(),
        };
        this.likedAlbums = [entry, ...this.likedAlbums];
      }
      this.persistLikedAlbums();
      Notify.create({
        message: already
          ? "Убрал альбом из коллекции"
          : "Альбом добавлен в коллекцию",
      });
      return !already;
    },

    removeAlbumLike(id: string) {
      this.likedAlbums = this.likedAlbums.filter(
        (item) => item.id !== String(id),
      );
      this.persistLikedAlbums();
    },

    togglePin(kind: number | string) {
      const id = Number(kind);
      if (this.pinned.includes(id))
        this.pinned = this.pinned.filter((v) => v !== id);
      else this.pinned = [...this.pinned, id];
      try {
        localStorage.setItem(PIN_KEY, JSON.stringify(this.pinned));
      } catch {
        Notify.create({
          type: "negative",
          message: "Не удалось сохранить закреплённые",
        });
        return;
      }
      Notify.create({
        message: this.pinned.includes(id)
          ? "Закрепил плейлист"
          : "Открепил плейлист",
      });
    },

    async refreshPlaylists() {
      this.playlists = await api.playlists().catch(() => this.playlists);
    },

    async createPlaylist(title: string, isPublic = false) {
      try {
        const created = await api.playlistCreate(title, isPublic);
        this.playlists = [created, ...this.playlists];
        Notify.create({ message: `Создал «${created.title}»` });
        return created;
      } catch (e) {
        notifyError(e, "Не удалось создать плейлист");
        return null;
      }
    },

    async renamePlaylist(kind: number, title: string) {
      try {
        await api.playlistRename(kind, title);
        const target = this.playlists.find((p) => p.kind === kind);
        if (target) target.title = title.trim();
        Notify.create({ message: "Переименовал плейлист" });
        return true;
      } catch (e) {
        notifyError(e, "Не удалось переименовать");
        return false;
      }
    },

    async setPlaylistPublic(kind: number, isPublic: boolean) {
      try {
        await api.playlistSetVisibility(kind, isPublic);
        Notify.create({
          message: isPublic
            ? "Плейлист теперь публичный"
            : "Плейлист теперь личный",
        });
        return true;
      } catch (e) {
        notifyError(e, "Не удалось изменить доступ");
        return false;
      }
    },

    async deletePlaylist(kind: number) {
      try {
        await api.playlistDelete(kind);
        this.playlists = this.playlists.filter((p) => p.kind !== kind);
        this.pinned = this.pinned.filter((v) => v !== Number(kind));
        try {
          localStorage.setItem(PIN_KEY, JSON.stringify(this.pinned));
        } catch {}
        Notify.create({ message: "Удалил плейлист" });
        return true;
      } catch (e) {
        notifyError(e, "Не удалось удалить плейлист");
        return false;
      }
    },

    async clearPlaylist(kind: number) {
      try {
        const removed = await api.playlistClear(kind);
        const target = this.playlists.find((p) => p.kind === kind);
        if (target) target.track_count = 0;
        Notify.create({
          message: removed ? `Убрал треков: ${removed}` : "Плейлист и так пуст",
        });
        return true;
      } catch (e) {
        notifyError(e, "Не удалось очистить плейлист");
        return false;
      }
    },

    async toggleLike(track: Track) {
      const isLiked = this.likedIds.includes(track.id);
      try {
        await api.setLike(track.id, isLiked);
        if (isLiked)
          this.likedIds = this.likedIds.filter((id) => id !== track.id);
        else this.likedIds = [track.id, ...this.likedIds];
        Notify.create({
          message: isLiked
            ? "Убрал из «Мне нравится»"
            : "Добавил в «Мне нравится»",
        });
      } catch (e) {
        notifyError(e, "Не удалось изменить оценку");
      }
    },

    async dislike(track: Track) {
      try {
        await api.setDislike(track.id, false);
        this.likedIds = this.likedIds.filter((id) => id !== track.id);
        Notify.create({ message: "Отметил «Не нравится»" });
      } catch (e) {
        notifyError(e, "Не удалось отметить трек");
      }
    },

    async addToPlaylist(kind: number, track: Track, silent = false) {
      try {
        await api.playlistAdd(kind, track.id, track.album_id || track.id, 0);
        const target = this.playlists.find((p) => p.kind === kind);
        if (target) target.track_count += 1;
        if (!silent) Notify.create({ message: "Добавил в плейлист" });
        return true;
      } catch (e) {
        if (!silent) notifyError(e, "Не удалось добавить в плейлист");
        return false;
      }
    },

    async addTracksToPlaylist(kind: number, tracks: Track[]) {
      let added = 0;
      for (const track of tracks) {
        if (await this.addToPlaylist(kind, track, true)) added += 1;
      }
      const target = this.playlists.find((p) => p.kind === kind);
      Notify.create({
        type: added ? "positive" : "negative",
        message: added
          ? `Добавил ${added} из ${tracks.length} в «${target?.title ?? "плейлист"}»`
          : "Не удалось добавить треки",
      });
      return added;
    },

    async removeFromPlaylist(kind: number, track: Track, at: number) {
      try {
        await api.playlistRemove(
          kind,
          track.id,
          track.album_id || track.id,
          at,
        );
        const target = this.playlists.find((p) => p.kind === kind);
        if (target && target.track_count > 0) target.track_count -= 1;
        Notify.create({ message: "Убрал из плейлиста" });
        return true;
      } catch (e) {
        notifyError(e, "Не удалось убрать трек");
        return false;
      }
    },

    async moveInPlaylist(kind: number, track: Track, from: number, to: number) {
      try {
        await api.playlistMove(
          kind,
          track.id,
          track.album_id || track.id,
          from,
          to,
        );
        return true;
      } catch (e) {
        notifyError(e, "Не удалось переместить трек");
        return false;
      }
    },
  },
});
