<template>
  <section id="settings-equalizer" class="settings-group">
    <div class="settings-group-head">
      <h2>Эквалайзер</h2>
      <p>10 полос, предусиление и готовые пресеты. Применяется сразу.</p>
    </div>
    <EqualizerPanel />
  </section>
</template>

<script setup lang="ts">
import EqualizerPanel from "@/components/settings/EqualizerPanel.vue";
</script>
