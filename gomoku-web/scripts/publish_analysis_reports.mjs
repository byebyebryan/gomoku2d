import { access, cp, mkdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = join(root, "..");
const sourceRoot = join(repoRoot, "gomoku-bot-lab", "analysis-reports");
const targetRoot = join(root, "dist", "analysis-report");
const requiredFiles = ["index.html", "latest.json"];
const allowMissingReports = process.env.GOMOKU_ALLOW_MISSING_REPORTS === "1";

try {
  await Promise.all([
    access(sourceRoot),
    ...requiredFiles.map((file) => access(join(sourceRoot, file))),
  ]);
} catch (error) {
  const message = [
    "Missing curated analysis report artifacts.",
    `Expected ${requiredFiles.join(" and ")} under ${sourceRoot}.`,
    "Generate the report or set GOMOKU_ALLOW_MISSING_REPORTS=1 for local/dev builds.",
  ].join(" ");

  if (allowMissingReports) {
    console.warn(`${message} Skipping analysis report publish.`);
    process.exit(0);
  }

  console.error(message);
  throw error;
}

await mkdir(targetRoot, { recursive: true });
await cp(sourceRoot, targetRoot, {
  recursive: true,
  force: true,
});

console.log("Published analysis reports to dist/analysis-report/.");
