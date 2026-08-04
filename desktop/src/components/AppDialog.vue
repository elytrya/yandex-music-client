<template>
  <q-dialog v-model="open" persistent>
    <div class="app-modal">
      <div class="app-modal-title">{{ appDialog.title }}</div>
      <div v-if="appDialog.message" class="app-modal-text">
        {{ appDialog.message }}
      </div>

      <div v-if="appDialog.kind === 'prompt'" class="field app-modal-field">
        <input
          ref="input"
          v-model="value"
          type="text"
          spellcheck="false"
          :placeholder="appDialog.placeholder"
          @keyup.enter="confirm"
        />
      </div>

      <div class="app-modal-actions">
        <button class="btn" type="button" @click="cancel">
          {{ appDialog.cancelLabel }}
        </button>
        <button
          class="btn-solid"
          :class="{ danger: appDialog.danger }"
          type="button"
          :disabled="!ready"
          @click="confirm"
        >
          {{ appDialog.okLabel }}
        </button>
      </div>
    </div>
  </q-dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { appDialog, resolveAppDialog } from "@/lib/dialogs";

const input = ref<HTMLInputElement | null>(null);
const value = ref("");

const open = computed({
  get: () => appDialog.open,
  set: (next: boolean) => {
    if (!next) resolveAppDialog(null);
  },
});

const ready = computed(
  () => appDialog.kind !== "prompt" || value.value.trim().length > 0,
);

watch(
  () => appDialog.open,
  (isOpen) => {
    if (!isOpen) return;
    value.value = appDialog.value;
    void nextTick(() => {
      input.value?.focus();
      input.value?.select();
    });
  },
);

function cancel() {
  resolveAppDialog(null);
}

function confirm() {
  if (!ready.value) return;
  resolveAppDialog(appDialog.kind === "prompt" ? value.value : "");
}
</script>
