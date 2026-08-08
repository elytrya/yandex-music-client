<template>
  <section id="settings-genius" class="settings-group">
    <div class="settings-group-head">
      <h2>Genius</h2>
      <p>
        Разборы строчек, авторы и продюсеры с genius.com. Нужен бесплатный
        Client Access Token — порядок действий ниже. Чем берётся сам текст,
        настраивается в разделе «Текст песни».
      </p>
    </div>

    <SettingToggle
      :model-value="genius.enabled"
      label="Включить Genius"
      description="В окне текста песни появится второй источник с цитатами и участниками записи."
      @update:model-value="genius.apply({ enabled: $event })"
    />

    <label class="setting-row setting-field-row">
      <span class="setting-copy">
        <b>Client Access Token</b>
        <span>
          genius.com/api-clients → New API Client. App Name - любое, например
          Mashiro. App Website URL и Redirect URI обязательные, подойдёт
          https://example.com - без них форма не сохраняется. После Save нажми
          Generate Access Token и вставь строку сюда. Токен хранится только на
          этом компьютере.
        </span>
      </span>
      <input
        :value="genius.token"
        type="password"
        spellcheck="false"
        autocomplete="off"
        placeholder="вставь токен"
        @change="setToken"
      />
    </label>

    <div class="setting-row setting-row-column">
      <div class="setting-copy">
        <b>Проверка и ссылки</b>
        <span>{{ genius.checkResult || "Токен ещё не проверяли." }}</span>
      </div>
      <div class="genius-actions">
        <button
          class="settings-reset-button"
          type="button"
          :disabled="genius.checking || !genius.token.trim()"
          @click="genius.check()"
        >
          {{ genius.checking ? "Проверяем…" : "Проверить токен" }}
        </button>
        <button
          class="settings-reset-button"
          type="button"
          @click="open('https://genius.com/api-clients')"
        >
          Получить токен
        </button>
        <button
          class="settings-reset-button"
          type="button"
          @click="open('https://docs.genius.com/')"
        >
          Документация
        </button>
        <button class="settings-reset-button" type="button" @click="clear">
          Очистить кеш Genius
        </button>
      </div>
    </div>

    <SettingToggle
      :model-value="genius.showQuotes"
      label="Показывать цитаты и разборы"
      description="Комментарии сообщества Genius к отдельным строчкам."
      @update:model-value="genius.apply({ showQuotes: $event })"
    />
  </section>
</template>

<script setup lang="ts">
import { Notify } from "quasar";
import { api } from "@/api/client";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import { useGeniusStore } from "@/stores/genius";

const genius = useGeniusStore();

function setToken(event: Event) {
  const input = event.target as HTMLInputElement;
  const value = input.value.trim();
  input.value = value;
  genius.apply({ token: value, enabled: value ? true : genius.enabled });
}

function open(url: string) {
  void api.openExternal(url);
}

async function clear() {
  await genius.clearCache();
  Notify.create({ message: "Кеш Genius очищен" });
}
</script>

<style scoped>
.genius-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
</style>
