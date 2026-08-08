/* eslint-disable */

import { defineConfig } from "@quasar/app-vite/wrappers";
import { fileURLToPath } from "node:url";
var __quasar_inject_import_meta_url__ = "file:///C:/Users/sorok/Music/yamp3/desktop/quasar.config.js";
var quasar_config_default = defineConfig(() => {
  return {
    boot: ["auth"],
    css: ["app.scss"],
    extras: ["roboto-font", "material-icons"],
    build: {
      target: {
        browser: ["es2022", "edge111", "firefox115", "chrome115", "safari15"],
        node: "node20"
      },
      typescript: {
        strict: true,
        vueShim: true
      },
      vueRouterMode: "hash",
      distDir: "dist/spa",
      alias: {
        "@": fileURLToPath(new URL("./src", __quasar_inject_import_meta_url__))
      }
    },
    devServer: {
      port: 9e3,
      open: false
    },
    framework: {
      config: {
        dark: true
      },
      plugins: ["Notify", "Loading"]
    },
    animations: [],
    ssr: false
  };
});
export {
  quasar_config_default as default
};
