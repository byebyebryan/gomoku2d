import { access, copyFile, mkdir, rm } from "node:fs/promises";
import { join } from "node:path";

const REQUIRED_REPORT_FILES = ["report.json"];

/**
 * @param {{
 *   label: string;
 *   sourceRoot: string;
 *   targetRoot: string;
 *   targetLabel: string;
 * }} options
 */
export async function publishReportArtifact({
  label,
  sourceRoot,
  targetRoot,
  targetLabel,
}) {
  const allowMissingReports = process.env.GOMOKU_ALLOW_MISSING_REPORTS === "1";

  try {
    await Promise.all([
      access(sourceRoot),
      ...REQUIRED_REPORT_FILES.map((file) => access(join(sourceRoot, file))),
    ]);
  } catch (error) {
    const message = [
      `Missing curated ${label} report artifacts.`,
      `Expected ${REQUIRED_REPORT_FILES.join(" and ")} under ${sourceRoot}.`,
      "Generate the report or set GOMOKU_ALLOW_MISSING_REPORTS=1 for local/dev builds.",
    ].join(" ");

    if (allowMissingReports) {
      console.warn(`${message} Skipping ${label} report publish.`);
      return;
    }

    console.error(message);
    throw error;
  }

  await rm(targetRoot, { force: true, recursive: true });
  await mkdir(targetRoot, { recursive: true });
  await copyFile(join(sourceRoot, "report.json"), join(targetRoot, "report.json"));

  console.log(`Published ${label} reports to ${targetLabel}.`);
}
