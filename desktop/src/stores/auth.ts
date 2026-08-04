import { defineStore } from "pinia";
import { api, getToken, setToken } from "@/api/client";
import type { Profile } from "@/api/types";

interface AuthState {
  token: string | null;
  profile: Profile | null;
  loading: boolean;
  error: string | null;
}

export const useAuthStore = defineStore("auth", {
  state: (): AuthState => ({
    token: null,
    profile: null,
    loading: false,
    error: null,
  }),

  getters: {
    isAuthenticated: (state) => Boolean(state.token && state.profile),
  },

  actions: {
    restore() {
      this.token = getToken();
    },

    async login(token: string): Promise<boolean> {
      this.loading = true;
      this.error = null;
      const clean = token.trim();
      try {
        this.profile = await api.login(clean);
        setToken(clean);
        this.token = clean;
        return true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : "Ошибка авторизации";
        this.logout();
        return false;
      } finally {
        this.loading = false;
      }
    },

    async fetchProfile(): Promise<void> {
      const token = this.token ?? getToken();
      if (!token) return;
      try {
        this.profile = await api.login(token);
        this.token = token;
      } catch (e) {
        this.error = e instanceof Error ? e.message : "Ошибка";
        this.logout();
      }
    },

    logout() {
      void api.logout().catch(() => undefined);
      void api.clearDiscordPresence().catch(() => undefined);
      setToken(null);
      this.token = null;
      this.profile = null;
    },
  },
});
