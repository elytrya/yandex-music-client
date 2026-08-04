import { defineStore } from "pinia";
import { api } from "@/api/client";
import { createLogger } from "@/lib/log";
import {
  HOTKEYS,
  defaultGlobalMap,
  defaultLocalMap,
} from "@/lib/hotkeys/defaults";
import type {
  GlobalHotkeyMap,
  HotkeyAction,
  HotkeyMap,
} from "@/lib/hotkeys/defaults";

const log = createLogger("hotkeys-store");
const STORAGE_KEY = "mashiro.hotkeys.v2";

interface SavedHotkeys {
  enabled: boolean;
  globalEnabled: boolean;
  local: Partial<HotkeyMap>;
  global: Partial<GlobalHotkeyMap>;
}

export const useHotkeysStore = defineStore("hotkeys", {
  state: () => ({
    enabled: true,
    globalEnabled: true,
    local: defaultLocalMap(),
    global: defaultGlobalMap(),
    failed: [] as string[],
    loaded: false,
  }),

  getters: {
    rows: (state) =>
      HOTKEYS.map((meta) => ({
        action: meta.action,
        label: meta.label,
        keys: state.local[meta.action] || [],
        accelerator: state.global[meta.action] || "",
      })),
  },

  actions: {
    load() {
      if (this.loaded) return;
      this.loaded = true;
      try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (!raw) return;
        const saved = JSON.parse(raw) as SavedHotkeys;
        this.enabled = saved.enabled !== false;
        this.globalEnabled = saved.globalEnabled !== false;
        const local = defaultLocalMap();
        const global = defaultGlobalMap();
        for (const meta of HOTKEYS) {
          const keys = saved.local?.[meta.action];
          if (Array.isArray(keys)) local[meta.action] = keys.filter(Boolean);
          const accelerator = saved.global?.[meta.action];
          if (typeof accelerator === "string")
            global[meta.action] = accelerator;
        }
        this.local = local;
        this.global = global;
      } catch (error) {
        log.warn("load failed", error);
      }
    },

    persist() {
      try {
        const payload: SavedHotkeys = {
          enabled: this.enabled,
          globalEnabled: this.globalEnabled,
          local: this.local,
          global: this.global,
        };
        localStorage.setItem(STORAGE_KEY, JSON.stringify(payload));
      } catch (error) {
        log.warn("persist failed", error);
      }
    },

    resolve(signature: string): HotkeyAction | null {
      if (!this.enabled || !signature) return null;
      for (const meta of HOTKEYS) {
        if ((this.local[meta.action] || []).includes(signature))
          return meta.action;
      }
      return null;
    },

    conflictLabel(signature: string, action: HotkeyAction): string | null {
      for (const meta of HOTKEYS) {
        if (meta.action === action) continue;
        if ((this.local[meta.action] || []).includes(signature))
          return meta.label;
      }
      return null;
    },

    setLocal(action: HotkeyAction, index: number, signature: string) {
      const keys = [...(this.local[action] || [])];
      for (const meta of HOTKEYS) {
        if (meta.action === action) continue;
        this.local[meta.action] = (this.local[meta.action] || []).filter(
          (key) => key !== signature,
        );
      }
      if (index < keys.length) keys[index] = signature;
      else keys.push(signature);
      this.local[action] = keys.filter(
        (key, i) => key && keys.indexOf(key) === i,
      );
      this.persist();
    },

    clearLocal(action: HotkeyAction, index: number) {
      const keys = [...(this.local[action] || [])];
      keys.splice(index, 1);
      this.local[action] = keys;
      this.persist();
    },

    async setGlobal(action: HotkeyAction, accelerator: string) {
      this.global[action] = accelerator;
      this.persist();
      await this.applyGlobal();
    },

    async setGlobalEnabled(enabled: boolean) {
      this.globalEnabled = enabled;
      this.persist();
      await this.applyGlobal();
    },

    setEnabled(enabled: boolean) {
      this.enabled = enabled;
      this.persist();
    },

    async reset() {
      this.enabled = true;
      this.globalEnabled = true;
      this.local = defaultLocalMap();
      this.global = defaultGlobalMap();
      this.persist();
      await this.applyGlobal();
    },

    async applyGlobal() {
      const bindings = this.globalEnabled
        ? HOTKEYS.filter((meta) => (this.global[meta.action] || "").trim()).map(
            (meta) => ({
              action: meta.action,
              accelerator: (this.global[meta.action] || "").trim(),
            }),
          )
        : [];
      try {
        this.failed = await api.setGlobalHotkeys(bindings);
      } catch (error) {
        log.warn("apply global failed", error);
        this.failed = [];
      }
    },
  },
});
