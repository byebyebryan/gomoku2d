import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./cloudtests",
  outputDir: "test-results/cloud",
  timeout: 45_000,
  expect: { timeout: 10_000 },
  workers: 1,
  retries: 0,
  use: {
    baseURL: "http://127.0.0.1:8013",
    viewport: { width: 1280, height: 900 },
    screenshot: "only-on-failure", trace: "retain-on-failure",
    serviceWorkers: "block",
  },
});
