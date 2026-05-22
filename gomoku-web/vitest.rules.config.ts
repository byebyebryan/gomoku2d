import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    include: ["tests/firestore_rules.test.ts"],
    testTimeout: 30_000,
  },
});
