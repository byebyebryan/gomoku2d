import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

function sharedPlugins() {
  return [wasm(), topLevelAwait()];
}

export default defineConfig(({ command }) => ({
  base: command === "build" ? "./" : "/",
  plugins: sharedPlugins(),
  worker: {
    format: "es",
    plugins: () => [wasm()],
  },
  resolve: {
    alias: {
      "@": "/src",
    },
  },
  server: {
    port: 3000,
    open: true,
  },
}));
