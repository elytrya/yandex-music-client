<template>
  <q-layout view="lHh Lpr lFf" class="app-shell">
    <q-page-container>
      <q-page class="flex flex-center column">
        <div class="h1">404</div>
        <div class="dim t-13 q-mt-sm">Такой страницы нет</div>
        <div class="btn-solid q-mt-lg" @click="router.replace('/')">
          На главную
        </div>
      </q-page>
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();
</script>
