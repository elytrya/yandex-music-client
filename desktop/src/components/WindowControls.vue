<template>
  <div class="win-controls">
    <div class="win-btn" @click="minimize">
      <Icon name="minimize" :size="15" :width="1.5" />
    </div>
    <div class="win-btn" @click="toggleMaximize">
      <Icon
        :name="maximized ? 'restore' : 'maximize'"
        :size="13"
        :width="1.5"
      />
    </div>
    <div class="win-btn close" @click="close">
      <Icon name="close" :size="15" :width="1.5" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import Icon from "@/components/Icon.vue";

const maximized = ref(false);

type AppWindow = {
  minimize: () => Promise<void>;
  toggleMaximize: () => Promise<void>;
  isMaximized: () => Promise<boolean>;
  close: () => Promise<void>;
};

async function appWindow(): Promise<AppWindow | null> {
  try {
    const mod = await import("@tauri-apps/api/window");
    return mod.getCurrentWindow() as unknown as AppWindow;
  } catch {
    return null;
  }
}

async function minimize() {
  const win = await appWindow();
  await win?.minimize();
}

async function toggleMaximize() {
  const win = await appWindow();
  if (!win) return;
  await win.toggleMaximize();
  maximized.value = await win.isMaximized();
}

async function close() {
  const win = await appWindow();
  await win?.close();
}

onMounted(async () => {
  const win = await appWindow();
  if (win) maximized.value = await win.isMaximized();
});
</script>
