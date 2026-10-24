import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { publishReportArtifact } from "./publish_report_artifact.mjs";

const webRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const destination=resolve(webRoot,process.argv[2] ?? "dist");
const sourceRoot = join(webRoot, "..", "reports", "lab");

const reports = [
  {
    label: "bot",
    sourceFile: "bot-report.json",
    targetDirectory: "bot-report",
  },
  {
    label: "analysis",
    sourceFile: "analysis-report.json",
    targetDirectory: "analysis-report",
  },
];

for (const report of reports) {
  await publishReportArtifact({
    label: report.label,
    sourceRoot,
    sourceFile: report.sourceFile,
    targetLabel: `dist/${report.targetDirectory}/`,
    targetRoot: join(destination, report.targetDirectory),
  });
}
