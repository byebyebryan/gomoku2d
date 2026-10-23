import { defineConfig } from "@playwright/test";
import { cloudPortOffset, cloudTestOrigins } from "./scripts/cloud_test_environment.mjs";

export default defineConfig({
  testDir: "./cloudtests",
  outputDir: cloudPortOffset === 0 ? "test-results/cloud" : `test-results/cloud-${cloudPortOffset}`,
  timeout: 45_000,
  expect: { timeout: 10_000 },
  workers: 1,
  retries: 0,
  use: {
    baseURL: cloudTestOrigins.preview,
    viewport: { width: 1280, height: 900 },
    screenshot: "only-on-failure", trace: "retain-on-failure",
    serviceWorkers: "block",
  },
});
