import { defineStore } from "pinia";
import { Notify } from "quasar";
import { api } from "@/api/client";

export const usePanelsStore = defineStore("panels", {
  state: () => ({
    queueOpen: false,
    hidden: false,
    mini: false,
    miniBusy: false,
  }),

  actions: {
    toggleQueue() {
      this.queueOpen = !this.queueOpen;
    },

    openQueue() {
      this.queueOpen = true;
    },

    closeQueue() {
      this.queueOpen = false;
    },

    async enterMini() {
      if (this.miniBusy || this.mini) return;
      this.miniBusy = true;
      try {
        await api.enterMiniPlayer();
        this.mini = true;
      } catch (e) {
        this.mini = false;
        Notify.create({
          type: "negative",
          message:
            e instanceof Error ? e.message : "Не удалось включить мини-плеер",
        });
      } finally {
        this.miniBusy = false;
      }
    },

    async exitMini() {
      this.miniBusy = true;
      this.mini = false;
      try {
        await api.exitMiniPlayer();
      } catch {
      } finally {
        this.mini = false;
        this.miniBusy = false;
      }
    },

    async toggleMini() {
      if (this.mini) await this.exitMini();
      else await this.enterMini();
    },
  },
});
