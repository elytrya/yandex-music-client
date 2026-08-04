<template>
  <section id="settings-hotkeys" class="settings-group">
    <div class="settings-group-head">
      <h2>Горячие клавиши</h2>
      <p>Сочетания внутри приложения и глобальные, работающие в фоне.</p>
    </div>

    <SettingToggle
      label="Горячие клавиши в приложении"
      description="Управление плеером с клавиатуры, пока окно активно"
      :model-value="hotkeys.enabled"
      @update:model-value="hotkeys.setEnabled($event)"
    />

    <SettingToggle
      label="Глобальные горячие клавиши"
      description="Работают в любых приложениях, даже когда Mashiro в трее"
      :model-value="hotkeys.globalEnabled"
      @update:model-value="onGlobalEnabled"
    />

    <div v-if="hotkeys.failed.length" class="hk-warn">
      Система не отдала эти сочетания (их занял кто-то другой):
      {{ hotkeys.failed.join(", ") }}
    </div>

    <div class="hk-table">
      <div class="hk-table-head">
        <span class="col">Действие</span>
        <span class="hk-table-col">В приложении</span>
        <span class="hk-table-col">Глобально</span>
      </div>

      <div v-for="row in hotkeys.rows" :key="row.action" class="hk-table-row">
        <span class="col hk-table-label">{{ row.label }}</span>

        <div class="hk-table-col hk-cell">
          <button
            v-for="(key, i) in row.keys"
            :key="`${key}-${i}`"
            class="hk-key editable"
            type="button"
            @click="startCapture(row.action, i, false)"
            @contextmenu.prevent="hotkeys.clearLocal(row.action, i)"
          >
            {{ capturingLabel(row.action, i, false) }}
          </button>
          <button
            class="hk-key add"
            type="button"
            @click="startCapture(row.action, row.keys.length, false)"
          >
            {{ capturingLabel(row.action, row.keys.length, false) || "+" }}
          </button>
        </div>

        <div class="hk-table-col hk-cell">
          <button
            class="hk-key editable"
            type="button"
            @click="startCapture(row.action, 0, true)"
            @contextmenu.prevent="clearGlobal(row.action)"
          >
            {{ globalLabel(row) }}
          </button>
          <div class="icon-btn xs round">
            <Icon name="more" :size="14" />
            <q-menu class="menu">
              <div class="menu-body">
                <div
                  v-for="preset in MEDIA_PRESETS"
                  :key="preset.value"
                  class="menu-item"
                  v-close-popup
                  @click="hotkeys.setGlobal(row.action, preset.value)"
                >
                  <span>{{ preset.label }}</span>
                </div>
                <div
                  class="menu-item danger"
                  v-close-popup
                  @click="clearGlobal(row.action)"
                >
                  <span>Убрать</span>
                </div>
              </div>
            </q-menu>
          </div>
        </div>
      </div>
    </div>

    <div class="hk-hint faint t-12">
      Нажмите на клавишу, чтобы задать новую. Правый клик - убрать. Для
      глобальных сочетаний используйте модификатор: Ctrl, Alt или Shift.
    </div>

    <div class="row q-mt-md">
      <button class="btn" type="button" @click="hotkeys.reset()">
        Вернуть стандартные
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";
import { Notify } from "quasar";
import Icon from "@/components/Icon.vue";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import type { HotkeyAction } from "@/lib/hotkeys/defaults";
import {
  displayAcceleratorParts,
  displayParts,
  eventSignature,
  isModifierEvent,
  toAccelerator,
} from "@/lib/hotkeys/keys";
import { useHotkeysStore } from "@/stores/hotkeys";

const hotkeys = useHotkeysStore();

async function onGlobalEnabled(enabled: boolean) {
  await hotkeys.setGlobalEnabled(enabled);
  if (enabled && hotkeys.failed.length) {
    Notify.create({
      type: "warning",
      message: `Система не отдала: ${hotkeys.failed.join(", ")}`,
    });
  }
}

const MEDIA_PRESETS = [
  { label: "Медиа: Play / Pause", value: "MediaPlayPause" },
  { label: "Медиа: следующий трек", value: "MediaTrackNext" },
  { label: "Медиа: предыдущий трек", value: "MediaTrackPrevious" },
  { label: "Медиа: стоп", value: "MediaStop" },
];

const capture = ref<{
  action: HotkeyAction;
  index: number;
  global: boolean;
} | null>(null);

function capturingLabel(
  action: HotkeyAction,
  index: number,
  global: boolean,
): string {
  const active = capture.value;
  if (
    active &&
    active.action === action &&
    active.index === index &&
    active.global === global
  )
    return "Жду…";
  if (global) return "";
  const key = (hotkeys.local[action] || [])[index];
  return key ? displayParts(key).join(" + ") : "";
}

function globalLabel(row: { action: HotkeyAction; accelerator: string }) {
  const active = capture.value;
  if (active && active.action === row.action && active.global) return "Жду…";
  if (!row.accelerator) return "Не задано";
  return displayAcceleratorParts(row.accelerator).join(" + ");
}

function startCapture(action: HotkeyAction, index: number, global: boolean) {
  capture.value = { action, index, global };
  window.addEventListener("keydown", onCapture, true);
}

function stopCapture() {
  capture.value = null;
  window.removeEventListener("keydown", onCapture, true);
}

function onCapture(event: KeyboardEvent) {
  if (isModifierEvent(event)) return;
  event.preventDefault();
  event.stopPropagation();
  const active = capture.value;
  if (!active) return;
  if (event.key === "Escape") {
    stopCapture();
    return;
  }
  const signature = eventSignature(event);
  if (active.global) {
    const accelerator = toAccelerator(signature);
    if (!accelerator.includes("+")) {
      Notify.create({
        type: "negative",
        message:
          "Для глобальной клавиши нужен модификатор: Ctrl, Alt или Shift",
      });
      stopCapture();
      return;
    }
    void hotkeys.setGlobal(active.action, accelerator);
  } else {
    const conflict = hotkeys.conflictLabel(signature, active.action);
    hotkeys.setLocal(active.action, active.index, signature);
    if (conflict) Notify.create({ message: `Клавиша снята с «${conflict}»` });
  }
  stopCapture();
}

function clearGlobal(action: HotkeyAction) {
  void hotkeys.setGlobal(action, "");
}

onBeforeUnmount(stopCapture);
</script>
