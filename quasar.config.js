import { defineConfig } from "#q-app/wrappers";
import { fileURLToPath } from "node:url";

export default defineConfig(() => {
  return {
    boot: ["native", "auth"],

    css: ["app.scss", "interface.scss"],

    extras: ["roboto-font", "material-icons"],

    build: {
      target: {
        browser: ["es2022", "edge111", "firefox115", "chrome115", "safari15"],
        node: "node20",
      },
      typescript: {
        strict: true,
        vueShim: true,
      },
      vueRouterMode: "hash",
      distDir: "dist/spa",
      alias: {
        "@": fileURLToPath(new URL("./src", import.meta.url)),
      },
    },

    devServer: {
      port: 9000,
      open: false,
    },

    framework: {
      config: {
        dark: true,
      },
      plugins: ["Notify", "Loading"],
    },

    animations: [],

    ssr: false,
  };
});
