import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

function sharedPlugins() {
  return [react(), wasm(), topLevelAwait()];
}

export default defineConfig(({ command }) => ({
  base: command === "build" ? "./" : "/",
  build: {
    target: "esnext",
  },
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
    port: 3001,
    open: true,
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
    setupFiles: "./vitest.setup.ts",
  },
}));
