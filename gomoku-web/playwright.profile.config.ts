import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./perftests",
  timeout: 180_000,
  workers: 1,
  use: { baseURL: "http://127.0.0.1:8012", viewport: { width: 1280, height: 900 } },
  webServer: {
    command: "npm run build && npm run preview -- --host 127.0.0.1 --port 8012 --strictPort",
    url: "http://127.0.0.1:8012",
    reuseExistingServer: false,
    timeout: 120_000,
  },
});
