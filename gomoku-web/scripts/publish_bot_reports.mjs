import { access, cp, mkdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = join(root, "..");
const sourceRoot = join(repoRoot, "gomoku-bot-lab", "reports");
const targetRoot = join(root, "dist", "bot-report");

try {
  await access(sourceRoot);
} catch {
  console.log("No bot reports to publish.");
  process.exit(0);
}

await mkdir(targetRoot, { recursive: true });
await cp(sourceRoot, targetRoot, {
  recursive: true,
  force: true,
});

console.log("Published bot reports to dist/bot-report/.");
