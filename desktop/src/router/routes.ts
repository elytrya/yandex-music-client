import type { RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
  {
    path: "/login",
    component: () => import("@/pages/LoginPage.vue"),
    meta: { public: true },
  },
  {
    path: "/",
    component: () => import("@/layouts/MainLayout.vue"),
    children: [
      {
        path: "",
        name: "home",
        component: () => import("@/pages/HomePage.vue"),
      },
      {
        path: "wave",
        name: "wave",
        component: () => import("@/pages/WavePage.vue"),
      },
      {
        path: "search",
        name: "search",
        component: () => import("@/pages/SearchPage.vue"),
      },
      {
        path: "liked",
        name: "liked",
        component: () => import("@/pages/LikedPage.vue"),
      },
      {
        path: "playlists",
        name: "playlists",
        component: () => import("@/pages/PlaylistsPage.vue"),
      },
      {
        path: "settings",
        name: "settings",
        component: () => import("@/pages/SettingsPage.vue"),
      },
      {
        path: "library",
        name: "library",
        component: () => import("@/pages/LibrarySearchPage.vue"),
      },
      {
        path: "stats",
        name: "stats",
        component: () => import("@/pages/StatsPage.vue"),
      },
      {
        path: "genius/:id/songs",
        name: "genius-songs",
        component: () => import("@/pages/GeniusSongsPage.vue"),
        props: true,
      },
      {
        path: "genius/:id",
        name: "genius-artist",
        component: () => import("@/pages/GeniusArtistPage.vue"),
        props: true,
      },
      {
        path: "playlists/:kind",
        name: "playlist",
        component: () => import("@/pages/PlaylistPage.vue"),
        props: true,
      },
      {
        path: "artist/:id/tracks",
        name: "artistTracks",
        component: () => import("@/pages/ArtistTracksPage.vue"),
        props: true,
      },
      {
        path: "artist/:id",
        name: "artist",
        component: () => import("@/pages/ArtistPage.vue"),
        props: true,
      },
      {
        path: "album/:id",
        name: "album",
        component: () => import("@/pages/AlbumPage.vue"),
        props: true,
      },
    ],
  },
  {
    path: "/:catchAll(.*)*",
    component: () => import("@/pages/ErrorNotFound.vue"),
  },
];

export default routes;
