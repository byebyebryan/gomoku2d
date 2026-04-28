import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    include: ["src/cloud/firestore_rules.rules.ts"],
    testTimeout: 30_000,
  },
});
