<template>
  <span class="artists-line">
    <span v-for="(a, i) in shown" :key="a.id || a.name">
      <span class="link" @click.stop="open(a.id)">{{ a.name }}</span>
      <span v-if="i < shown.length - 1">, </span>
    </span>
    <span v-if="hidden > 0" class="more-link" @click.stop>
      &nbsp;ещё {{ hidden }}
      <q-menu
        class="panel menu"
        anchor="bottom start"
        self="top start"
        @show="loadAvatars"
      >
        <div
          class="menu-body"
          style="min-width: 200px; max-height: 320px; overflow-y: auto"
        >
          <div
            v-for="a in artists"
            :key="a.id || a.name"
            class="menu-item"
            v-close-popup
            @click="open(a.id)"
          >
            <div class="cover menu-cover round">
              <img
                v-if="avatars[a.id]"
                :src="avatars[a.id] as string"
                loading="lazy"
                decoding="async"
              />
              <Icon v-else name="artist" :size="14" class="faint" />
            </div>
            <span class="ellipsis">{{ a.name }}</span>
          </div>
        </div>
      </q-menu>
    </span>
  </span>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import Icon from "@/components/Icon.vue";
import type { Artist } from "@/api/types";
import { cachedAvatar, fetchAvatars } from "@/lib/artistAvatar";

const props = withDefaults(
  defineProps<{ artists: Artist[]; limit?: number }>(),
  {
    limit: 3,
  },
);

const router = useRouter();

const artists = computed(() => props.artists);
const shown = computed(() => props.artists.slice(0, props.limit));
const hidden = computed(() => Math.max(0, props.artists.length - props.limit));

const avatars = ref<Record<string, string | null>>({});

async function loadAvatars() {
  const ids = props.artists.map((a) => a.id).filter(Boolean);
  const local: Record<string, string | null> = { ...avatars.value };
  for (const id of ids) {
    const cached = cachedAvatar(id);
    if (cached) local[id] = cached;
  }
  avatars.value = local;

  const missing = ids.filter((id) => !local[id]);
  if (!missing.length) return;
  const loaded = await fetchAvatars(missing);
  avatars.value = { ...avatars.value, ...loaded };
}

function open(id: string) {
  if (id) void router.push(`/artist/${id}`);
}
</script>
