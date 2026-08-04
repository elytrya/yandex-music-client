import { defineRouter } from "#q-app/wrappers";
import { createRouter, createWebHashHistory } from "vue-router";
import { getToken } from "@/api/client";
import routes from "./routes";

export default defineRouter(() => {
  const Router = createRouter({
    history: createWebHashHistory(),
    routes,
    scrollBehavior: () => ({ left: 0, top: 0 }),
  });

  Router.beforeEach((to) => {
    const authed = Boolean(getToken());
    if (!to.meta.public && !authed) return { path: "/login" };
    if (to.path === "/login" && authed) return { path: "/" };
    return true;
  });

  return Router;
});
