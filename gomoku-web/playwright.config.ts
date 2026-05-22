import { defineConfig } from "@playwright/test";

const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8001";

export default defineConfig({
  testDir: "./playtests",
  timeout: 30_000,
  use: {
    baseURL,
    channel: "chrome",
    headless: true,
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
    video: "retain-on-failure",
    viewport: {
      width: 1440,
      height: 1100,
    },
  },
  webServer: process.env.PLAYWRIGHT_BASE_URL
    ? undefined
    : {
      command: "npm run build && npm run preview -- --host 127.0.0.1 --port 8001",
      reuseExistingServer: !process.env.CI,
      timeout: 120_000,
      url: baseURL,
    },
});
