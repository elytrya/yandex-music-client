<template>
  <Teleport to="body">
    <transition name="viewer-fade">
      <div v-if="open" class="viewer" @click.self="close">
        <button
          class="viewer-close"
          type="button"
          title="Закрыть"
          @click="close"
        >
          <Icon name="close" :size="20" />
        </button>

        <button
          v-if="images.length > 1"
          class="viewer-nav prev"
          type="button"
          title="Предыдущее"
          @click.stop="prev"
        >
          <Icon name="chevronLeft" :size="24" />
        </button>

        <div class="viewer-stage" @click.self="close">
          <img :src="images[cur]" :alt="title" class="viewer-img" />
          <div class="viewer-bar">
            <span v-if="images.length > 1" class="viewer-count">
              {{ cur + 1 }} / {{ images.length }}
            </span>
            <button
              class="btn viewer-download"
              type="button"
              :disabled="saving"
              @click.stop="download"
            >
              <Icon name="download" :size="15" />
              <span>{{ saving ? "Сохранение…" : "Скачать" }}</span>
            </button>
          </div>
        </div>

        <button
          v-if="images.length > 1"
          class="viewer-nav next"
          type="button"
          title="Следующее"
          @click.stop="next"
        >
          <Icon name="chevronRight" :size="24" />
        </button>

        <div v-if="images.length > 1" class="viewer-thumbs" @click.stop>
          <button
            v-for="(img, i) in images"
            :key="i"
            class="viewer-thumb"
            :class="{ active: i === cur }"
            type="button"
            @click="cur = i"
          >
            <img :src="img" :alt="`${title} ${i + 1}`" />
          </button>
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<script setup lang="ts">
import { onUnmounted, ref, watch } from "vue";
import { Notify } from "quasar";
import Icon from "@/components/Icon.vue";
import { api } from "@/api/client";

const props = withDefaults(
  defineProps<{
    open: boolean;
    images: string[];
    title?: string;
    initialIndex?: number;
  }>(),
  { title: "", initialIndex: 0 },
);

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
}>();

const cur = ref(props.initialIndex);
const saving = ref(false);

function close() {
  emit("update:open", false);
}

function prev() {
  if (!props.images.length) return;
  cur.value = (cur.value - 1 + props.images.length) % props.images.length;
}

function next() {
  if (!props.images.length) return;
  cur.value = (cur.value + 1) % props.images.length;
}

async function download() {
  const url = props.images[cur.value];
  if (!url) return;
  saving.value = true;
  try {
    const suffix = props.images.length > 1 ? ` ${cur.value + 1}` : "";
    const path = await api.downloadImage(
      url,
      `${props.title || "cover"}${suffix}`,
    );
    Notify.create({ message: `Сохранено: ${path}` });
  } catch {
    Notify.create({ message: "Не удалось скачать изображение" });
  } finally {
    saving.value = false;
  }
}

function onKey(e: KeyboardEvent) {
  if (!props.open) return;
  if (e.key === "Escape") close();
  else if (e.key === "ArrowLeft" && props.images.length > 1) prev();
  else if (e.key === "ArrowRight" && props.images.length > 1) next();
}

watch(
  () => props.open,
  (v) => {
    if (v) {
      const max = Math.max(props.images.length - 1, 0);
      cur.value = Math.min(props.initialIndex, max);
      window.addEventListener("keydown", onKey);
    } else {
      window.removeEventListener("keydown", onKey);
    }
  },
);

onUnmounted(() => window.removeEventListener("keydown", onKey));
</script>

<style scoped>
.viewer {
  position: fixed;
  inset: 0;
  z-index: 4000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.82);
  backdrop-filter: blur(8px);
}
.viewer-stage {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  max-width: 82vw;
  max-height: 84vh;
}
.viewer-img {
  max-width: 82vw;
  max-height: 72vh;
  border-radius: 12px;
  object-fit: contain;
  box-shadow: 0 24px 60px rgba(0, 0, 0, 0.55);
  user-select: none;
}
.viewer-bar {
  display: flex;
  align-items: center;
  gap: 14px;
}
.viewer-count {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.75);
}
.viewer-download {
  color: #fff;
}
.viewer-close {
  position: absolute;
  top: 18px;
  right: 20px;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}
.viewer-close:hover {
  background: rgba(255, 255, 255, 0.22);
}
.viewer-nav {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 46px;
  height: 46px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}
.viewer-nav:hover {
  background: rgba(255, 255, 255, 0.22);
}
.viewer-nav.prev {
  left: 24px;
}
.viewer-nav.next {
  right: 24px;
}
.viewer-thumbs {
  position: absolute;
  bottom: 18px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 8px;
  max-width: 80vw;
  overflow-x: auto;
  padding: 6px;
}
.viewer-thumb {
  width: 52px;
  height: 52px;
  border-radius: 8px;
  overflow: hidden;
  border: 2px solid transparent;
  padding: 0;
  cursor: pointer;
  background: none;
  flex: 0 0 auto;
}
.viewer-thumb.active {
  border-color: #fff;
}
.viewer-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.viewer-fade-enter-active,
.viewer-fade-leave-active {
  transition: opacity 0.18s ease;
}
.viewer-fade-enter-from,
.viewer-fade-leave-to {
  opacity: 0;
}
</style>
