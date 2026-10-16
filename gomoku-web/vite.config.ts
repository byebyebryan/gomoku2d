import { defineConfig, type PluginOption } from "vite";
import react from "@vitejs/plugin-react";
import { readFileSync } from "node:fs";
import * as wasmModule from "vite-plugin-wasm";

const appVersion = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf8")).version;
const wasm = wasmModule.default as unknown as () => PluginOption;

function normalizeBasePath(basePath: string | undefined): string {
  if (!basePath || basePath === "/") {
    return "/";
  }

  const withLeadingSlash = basePath.startsWith("/") ? basePath : `/${basePath}`;
  return withLeadingSlash.endsWith("/") ? withLeadingSlash : `${withLeadingSlash}/`;
}

function sharedPlugins(): PluginOption[] {
  return [react(), wasm()];
}

const popupAuthHeaders = {
  "Cross-Origin-Opener-Policy": "same-origin-allow-popups",
  "Referrer-Policy": "no-referrer-when-downgrade",
};

export default defineConfig(() => ({
  base: normalizeBasePath(process.env.GOMOKU_BASE_PATH),
  build: {
    // Wasm loading uses native top-level await, including in ES workers.
    target: "esnext",
    modulePreload: {
      // Failed modulepreloads can poison WebKit's cache across reloads:
      // https://bugs.webkit.org/show_bug.cgi?id=270357
      // Let dynamic import fetch its JS graph; retain CSS and entry preloads.
      resolveDependencies: (_filename, dependencies, { hostType }) => hostType === "js"
        ? dependencies.filter((path) => path.endsWith(".css"))
        : dependencies,
    },
  },
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
  },
  plugins: sharedPlugins(),
  worker: {
    format: "es" as const,
    plugins: () => [wasm()],
  },
  resolve: {
    alias: {
      "@": "/src",
    },
  },
  server: {
    headers: popupAuthHeaders,
    port: 3001,
    open: true,
  },
  preview: {
    host: "0.0.0.0",
    headers: popupAuthHeaders,
    port: 8001,
    allowedHosts: true as const,
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
    setupFiles: "./vitest.setup.ts",
  },
}));
