<template>
  <q-layout view="lHh Lpr lFf" class="app-shell">
    <div
      class="titlebar"
      style="justify-content: flex-end"
      @mousedown="startWindowDrag"
      @dblclick="toggleWindowMaximize"
    >
      <WindowControls />
    </div>

    <q-page-container>
      <q-page class="flex flex-center">
        <div class="panel login-panel">
          <div class="login-brand">
            <div class="login-logo">
              <Icon name="wave" :size="20" />
            </div>
            <div>
              <div class="h2 login-title">Mashiro</div>
              <div class="faint t-12">Клиент Яндекс Музыки</div>
            </div>
          </div>

          <template v-if="!waiting">
            <p class="dim t-13 login-lead">
              Войди через аккаунт Яндекса - откроется браузер. Токен приложение
              получит само, копировать ничего не нужно.
            </p>

            <button
              type="button"
              class="login-primary"
              :disabled="auth.loading"
              @click="loginAuto"
            >
              <q-spinner v-if="auth.loading" size="16px" color="white" />
              <span v-else>Войти через Яндекс</span>
            </button>
          </template>

          <div v-else class="login-waiting">
            <p class="dim t-13 login-wait-lead">
              В браузере открылась страница входа Яндекса. Войди и введи там
              этот код:
            </p>
            <div class="login-code-row">
              <div class="login-code">{{ userCode }}</div>
              <button
                type="button"
                class="login-copy"
                :title="copied ? 'Скопировано' : 'Скопировать код'"
                @click="copyCode"
              >
                <Icon :name="copied ? 'check' : 'copy'" :size="18" />
              </button>
            </div>
            <div class="login-wait-row">
              <q-spinner size="14px" />
              <span class="t-12 dim">Ждём подтверждения…</span>
              <button type="button" class="login-link" @click="cancelAuto">
                Отмена
              </button>
            </div>
            <p class="faint t-11 login-url">
              Если браузер не открылся, зайди вручную на {{ verifyUrl }}
            </p>
          </div>

          <div v-if="autoError" class="t-12 login-error">{{ autoError }}</div>

          <button type="button" class="login-toggle" @click="manual = !manual">
            <Icon name="key" :size="14" />
            <span>Ввести токен вручную</span>
            <Icon
              name="chevronDown"
              :size="14"
              class="login-toggle-chev"
              :class="{ open: manual }"
            />
          </button>

          <div v-if="manual" class="login-manual">
            <div class="field login-field">
              <input
                v-model="raw"
                :type="reveal ? 'text' : 'password'"
                placeholder="Токен y0_Ag… или ссылка с access_token"
                @keyup.enter="submitManual"
              />
              <button
                type="button"
                class="login-link"
                @click="reveal = !reveal"
              >
                {{ reveal ? "Скрыть" : "Показать" }}
              </button>
            </div>
            <button
              type="button"
              class="btn full-width q-mt-sm"
              :disabled="auth.loading || !raw.trim()"
              @click="submitManual"
            >
              <span>Войти с токеном</span>
            </button>
            <p class="faint t-11 login-hint">
              Токен можно взять из адресной строки после входа - вставь целиком,
              нужная часть извлечётся сама.
            </p>
          </div>

          <div v-if="auth.error" class="t-12 login-error">{{ auth.error }}</div>

          <div class="sep login-sep" />

          <div class="faint t-11">
            Токен хранится только на твоём компьютере и используется лишь для
            API Яндекс Музыки.
          </div>
        </div>
      </q-page>
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import Icon from "@/components/Icon.vue";
import WindowControls from "@/components/WindowControls.vue";
import { startWindowDrag, toggleWindowMaximize } from "@/lib/window";
import { onBeforeUnmount, ref } from "vue";
import { useRouter } from "vue-router";
import { useAuthStore } from "@/stores/auth";
import type { UnlistenFn } from "@tauri-apps/api/event";

type DeviceInfo = { user_code: string; verification_url: string };

const router = useRouter();
const auth = useAuthStore();

const raw = ref("");
const reveal = ref(false);
const manual = ref(false);
const waiting = ref(false);
const autoError = ref("");
const userCode = ref("");
const verifyUrl = ref("");
const copied = ref(false);
let unlisteners: UnlistenFn[] = [];

function extractToken(value: string): string {
  const s = value.trim();
  const match = s.match(/access_token=([^&#\s]+)/i);
  if (match?.[1]) return decodeURIComponent(match[1]);
  return s;
}

async function finish(token: string): Promise<void> {
  const clean = extractToken(token);
  if (!clean) return;
  const ok = await auth.login(clean);
  if (ok) void router.replace("/");
}

async function stopWaiting(): Promise<void> {
  waiting.value = false;
  userCode.value = "";
  copied.value = false;
  for (const off of unlisteners) off();
  unlisteners = [];
}

async function loginAuto(): Promise<void> {
  autoError.value = "";
  try {
    const [{ invoke }, { listen }] = await Promise.all([
      import("@tauri-apps/api/core"),
      import("@tauri-apps/api/event"),
    ]);
    unlisteners.push(
      await listen<string>("oauth-token", async (event) => {
        await stopWaiting();
        await finish(event.payload);
      }),
    );
    unlisteners.push(
      await listen<string>("oauth-error", async (event) => {
        await stopWaiting();
        autoError.value = event.payload || "Вход не завершён. Попробуй снова.";
      }),
    );
    const info = await invoke<DeviceInfo>("oauth_device_start");
    userCode.value = info.user_code;
    verifyUrl.value = info.verification_url;
    waiting.value = true;
  } catch {
    await stopWaiting();
    autoError.value = "Не удалось начать вход. Введи токен вручную ниже.";
    manual.value = true;
  }
}

async function cancelAuto(): Promise<void> {
  await stopWaiting();
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("oauth_cancel");
  } catch {}
}

async function submitManual(): Promise<void> {
  if (!raw.value.trim()) return;
  await finish(raw.value);
}

async function copyCode(): Promise<void> {
  if (!userCode.value) return;
  try {
    await navigator.clipboard.writeText(userCode.value);
    copied.value = true;
    window.setTimeout(() => (copied.value = false), 1500);
  } catch {}
}

onBeforeUnmount(() => {
  void stopWaiting();
});
</script>

<style scoped>
.login-panel {
  width: 380px;
  max-width: calc(100vw - 48px);
  padding: 26px;
}

.login-brand {
  display: flex;
  align-items: center;
  gap: 12px;
}

.login-logo {
  display: grid;
  width: 44px;
  height: 44px;
  flex: none;
  place-items: center;
  border-radius: 13px;
  background: color-mix(in srgb, var(--accent) 16%, transparent);
  color: var(--accent);
}

.login-title {
  line-height: 1.1;
}

.login-lead {
  margin: 18px 0 20px;
  line-height: 1.5;
}

.login-primary {
  display: flex;
  width: 100%;
  height: 44px;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: 0;
  border-radius: 12px;
  background: var(--accent);
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition:
    filter 0.12s ease,
    opacity 0.12s ease;
}

.login-primary:hover {
  filter: brightness(1.07);
}

.login-primary:disabled {
  cursor: default;
  opacity: 0.6;
}

.login-waiting {
  margin: 18px 0 4px;
}

.login-wait-lead {
  margin: 0 0 12px;
  line-height: 1.5;
}

.login-code-row {
  display: flex;
  gap: 8px;
}

.login-code {
  display: flex;
  flex: 1;
  height: 52px;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  background: var(--surface-2);
  color: var(--fg);
  font-size: 26px;
  font-weight: 600;
  letter-spacing: 4px;
  font-variant-numeric: tabular-nums;
}

.login-copy {
  display: grid;
  width: 52px;
  height: 52px;
  flex: none;
  place-items: center;
  border: 0;
  border-radius: 12px;
  background: var(--surface-2);
  color: var(--fg-dim);
  cursor: pointer;
  transition:
    background 0.12s ease,
    color 0.12s ease;
}

.login-copy:hover {
  background: var(--hover);
  color: var(--fg);
}

.login-wait-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 14px;
}

.login-url {
  margin: 12px 2px 0;
  line-height: 1.45;
  word-break: break-all;
}

.login-toggle {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 8px;
  margin-top: 16px;
  padding: 0;
  border: 0;
  background: none;
  color: var(--fg-dim);
  font-size: 12.5px;
  cursor: pointer;
  transition: color 0.12s ease;
}

.login-toggle:hover {
  color: var(--fg);
}

.login-toggle-chev {
  margin-left: auto;
  transition: transform 0.16s ease;
}

.login-toggle-chev.open {
  transform: rotate(180deg);
}

.login-manual {
  margin-top: 12px;
}

.login-field {
  display: flex;
  align-items: center;
  gap: 8px;
}

.login-link {
  flex: none;
  padding: 0 2px;
  border: 0;
  background: none;
  color: var(--fg-faint);
  font-size: 11.5px;
  cursor: pointer;
  transition: color 0.12s ease;
}

.login-link:hover {
  color: var(--fg);
}

.login-hint {
  margin: 10px 2px 0;
  line-height: 1.45;
}

.login-error {
  margin-top: 10px;
  color: #f87171;
}

.login-sep {
  margin: 20px 0 14px;
}
</style>
