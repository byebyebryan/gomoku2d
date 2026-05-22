import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { publishReportArtifact } from "./publish_report_artifact.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = join(root, "..");
const sourceRoot = join(repoRoot, "gomoku-bot-lab", "analysis-reports");
const targetRoot = join(root, "dist", "analysis-report");

await publishReportArtifact({
  label: "analysis",
  sourceRoot,
  targetLabel: "dist/analysis-report/",
  targetRoot,
});
