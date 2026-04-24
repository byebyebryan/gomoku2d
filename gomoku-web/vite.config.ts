import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { readFileSync } from "node:fs";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

const appVersion = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf8")).version;

function normalizeBasePath(basePath: string | undefined): string {
  if (!basePath || basePath === "/") {
    return "/";
  }

  const withLeadingSlash = basePath.startsWith("/") ? basePath : `/${basePath}`;
  return withLeadingSlash.endsWith("/") ? withLeadingSlash : `${withLeadingSlash}/`;
}

function sharedPlugins() {
  return [react(), wasm(), topLevelAwait()];
}

export default defineConfig(() => ({
  base: normalizeBasePath(process.env.GOMOKU_BASE_PATH),
  build: {
    target: "esnext",
  },
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
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
  preview: {
    host: "0.0.0.0",
    port: 8001,
    allowedHosts: true,
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
    setupFiles: "./vitest.setup.ts",
  },
}));
