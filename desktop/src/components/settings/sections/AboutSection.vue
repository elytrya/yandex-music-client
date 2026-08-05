<template>
  <section id="settings-about" class="settings-group">
    <div class="settings-group-head">
      <h2>О проекте</h2>
      <p>Автор, версия сборки, лицензия и использованные проекты.</p>
    </div>

    <div class="about-card">
      <img class="about-avatar" :src="avatar" alt="elytrya" />
      <div class="about-ident">
        <div class="about-name">Mashiro</div>
        <div class="about-sub">
          неофициальный десктоп-клиент Яндекс Музыки
        </div>
        <div class="about-links">
          автор:
          <a :href="AUTHOR_URL" @click.prevent="openLink(AUTHOR_URL)">elytrya</a>
        </div>
      </div>
    </div>

    <div class="about-rows">
      <div class="about-row">
        <span class="about-key">Версия</span>
        <span class="about-val">a0.3</span>
      </div>
      <div class="about-row">
        <span class="about-key">Лицензия</span>
        <span class="about-val">
          <a :href="LICENSE_URL" @click.prevent="openLink(LICENSE_URL)">GPL-3.0</a>
        </span>
      </div>
      <div class="about-row">
        <span class="about-key">Авторство</span>
        <span class="about-val">
          ООО «Яндекс» к проекту отношения не имеет
        </span>
      </div>
    </div>

    <div class="about-block">
      <h3>Благодарности</h3>
      <p class="about-note">
        Проекты, без которых этот клиент не работал бы в текущем виде.
      </p>
      <a
        v-for="item in credits"
        :key="item.url"
        class="credit"
        :href="item.url"
        :title="item.url"
        @click.prevent="openLink(item.url)"
      >
        <span class="credit-name">{{ item.name }}</span>
        <span class="credit-desc">{{ item.desc }}</span>
      </a>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "@/api/client";
import { cachedImage, loadCachedImage } from "@/lib/imageCache";

const AUTHOR_URL = "https://github.com/elytrya";
const AVATAR_URL = "https://github.com/elytrya.png?size=200";
const LICENSE_URL = "https://www.gnu.org/licenses/gpl-3.0.html";
const FALLBACK = "icons/128x128.png";

const avatar = ref(cachedImage("elytrya") ?? FALLBACK);

function openLink(url: string) {
  void api.openExternal(url);
}

onMounted(async () => {
  avatar.value = await loadCachedImage("elytrya", AVATAR_URL);
});

const credits = [
  {
    name: "MarshalX/yandex-music-api",
    desc: "описание неофициального API Яндекс Музыки",
    url: "https://github.com/MarshalX/yandex-music-api",
  },
  {
    name: "vyfor/yandex-music-rs",
    desc: "схемы ответов API для Rust-части",
    url: "https://github.com/vyfor/yandex-music-rs",
  },
  {
    name: "Hazzz895/FckCensor",
    desc: "оттуда позаимствована логика подмены зацензуренных треков",
    url: "https://github.com/Hazzz895/FckCensor",
  },
  {
    name: "Hazzz895/FckCensorData",
    desc: "база ссылок на незацензуренные версии треков",
    url: "https://github.com/Hazzz895/FckCensorData",
  },
  {
    name: "alexeyfv/slopless",
    desc: "база артистов с музыкой, сгенерированной нейросетями",
    url: "https://github.com/alexeyfv/slopless",
  },
];
</script>

<style scoped>
.about-card {
  display: flex;
  align-items: center;
  gap: 14px;
  margin: 18px 0 20px;
}
.about-avatar {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  object-fit: cover;
  flex: 0 0 auto;
}
.about-name {
  font-size: 16px;
  font-weight: 600;
}
.about-sub {
  font-size: 12px;
  opacity: 0.6;
  margin-top: 2px;
}
.about-links {
  margin-top: 6px;
  font-size: 12px;
  opacity: 0.8;
}
.about-links a,
.about-val a,
.credit-name {
  color: var(--accent, #fa2d48);
  text-decoration: none;
  cursor: pointer;
}
.about-links a:hover,
.about-val a:hover,
.credit:hover .credit-name {
  text-decoration: underline;
}
.about-rows {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.about-row {
  display: flex;
  gap: 12px;
  font-size: 13px;
  line-height: 1.45;
}
.about-key {
  flex: 0 0 110px;
  opacity: 0.6;
}
.about-val {
  flex: 1 1 auto;
}
.about-block {
  margin-top: 26px;
  padding-top: 18px;
  border-top: 1px solid var(--line, rgba(255, 255, 255, 0.08));
}
.about-block h3 {
  margin: 0 0 4px;
  font-size: 14px;
  font-weight: 600;
}
.about-note {
  margin: 0 0 12px;
  font-size: 12px;
  opacity: 0.6;
}
.credit {
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 0;
  border-bottom: 1px solid var(--line, rgba(255, 255, 255, 0.06));
  text-decoration: none;
}
.credit:last-child {
  border-bottom: none;
}
.credit-name {
  font-size: 13px;
  font-weight: 500;
}
.credit-desc {
  font-size: 12px;
  color: var(--fg-dim);
}
</style>
