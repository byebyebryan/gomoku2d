import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import {
  lstat,
  mkdir,
  readFile,
  readdir,
  rename,
  realpath,
  writeFile,
} from "node:fs/promises";
import { promisify } from "node:util";
import { isAbsolute, relative, resolve, sep } from "node:path";
import { build } from "vite";
import { publishReviews } from "./publish_reviews.mjs";
import { readReviewCatalog, webRoot, repoRoot } from "./review_catalog.mjs";
import { readHistoryCatalog } from "./history_catalog.mjs";

const execFileAsync = promisify(execFile);
const exportPrefix = "test-results/developer-exports/";
const [targetArgument, ...extraArguments] = process.argv.slice(2);
if (!targetArgument || extraArguments.length)
  throw new Error(
    "Usage: npm run developer:export -- test-results/developer-exports/<new-name>",
  );

if (!targetArgument.startsWith(exportPrefix))
  throw new Error("Use a new directory under gomoku-web/test-results/developer-exports/");
const exportName = targetArgument.slice(exportPrefix.length);
if (
  !/^[a-zA-Z0-9][a-zA-Z0-9_.-]*$/.test(exportName) ||
  exportName.includes("..") ||
  exportName.includes("/") ||
  exportName.includes("\\")
)
  throw new Error("The developer export name must be one safe directory name.");

const requestedOutput = resolve(webRoot, targetArgument);
const requestedRelative = relative(webRoot, requestedOutput).split(sep).join("/");
if (requestedRelative !== targetArgument || isAbsolute(targetArgument))
  throw new Error("The developer export path must stay under its dedicated output root.");

/** @param {string[]} args */
async function runRtk(args, cwd = repoRoot) {
  const { stdout } = await execFileAsync("rtk", args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 10 * 1024 * 1024,
  });
  return stdout.trim();
}

/** @param {string} path */
async function ensureDirectory(path) {
  try {
    const info = await lstat(path);
    if (!info.isDirectory() || info.isSymbolicLink())
      throw new Error(`Expected a real directory at ${path}`);
  } catch (error) {
    if (error && typeof error === "object" && "code" in error && error.code === "ENOENT") {
      await mkdir(path);
      return;
    }
    throw error;
  }
}

const physicalWebRoot = await realpath(webRoot);
const physicalRepoRoot = await realpath(repoRoot);
const exportParent = resolve(physicalWebRoot, "test-results");
const exportRoot = resolve(exportParent, "developer-exports");
await ensureDirectory(exportParent);
await ensureDirectory(exportRoot);
if ((await realpath(exportRoot)) !== exportRoot)
  throw new Error("The developer export root cannot resolve through a symlink.");
const outputDirectory = resolve(exportRoot, exportName);
const outputRelativeToRepo = relative(physicalRepoRoot, outputDirectory).split(sep).join("/");
if (!outputRelativeToRepo || outputRelativeToRepo.startsWith("../") || isAbsolute(outputRelativeToRepo))
  throw new Error("The developer export path leaves the repository.");

try {
  await runRtk(["git", "check-ignore", "--quiet", "--", outputRelativeToRepo], physicalRepoRoot);
} catch {
  throw new Error("Developer exports must stay under the ignored test-results root.");
}

try {
  await lstat(outputDirectory);
  throw new Error("Refusing to overwrite an existing developer export directory.");
} catch (error) {
  if (!(error && typeof error === "object" && "code" in error && error.code === "ENOENT"))
    throw error;
}
await mkdir(outputDirectory);

const status = await runRtk(
  ["git", "status", "--porcelain", "--untracked-files=normal"],
  physicalRepoRoot,
);
const sourceCommit = await runRtk(["git", "rev-parse", "HEAD"], physicalRepoRoot);
const statusLines = status ? status.split(/\r?\n/) : [];
const trackedSourceDirty = statusLines.some((line) => line && !line.startsWith("??"));
const workingTreeClean = statusLines.length === 0;
const viteInfo = JSON.parse(
  await readFile(resolve(physicalWebRoot, "node_modules/vite/package.json"), "utf8"),
);
const npmVersion = await runRtk(["npm", "--version"], physicalWebRoot);

await build({
  configFile: resolve(physicalWebRoot, "vite.config.ts"),
  root: physicalWebRoot,
  base: "./",
  publicDir: false,
  define: { "import.meta.env.BASE_URL": "globalThis.__GOMOKU_REPORT_BASE__" },
  build: {
    outDir: outputDirectory,
    emptyOutDir: false,
    rolldownOptions: { input: resolve(physicalWebRoot, "developer.html") },
  },
});

await rename(resolve(outputDirectory, "developer.html"), resolve(outputDirectory, "index.html"));
await runRtk(
  ["node", "scripts/publish_visual_design_assets.mjs", targetArgument],
  physicalWebRoot,
);
await runRtk(["node", "scripts/publish_reports.mjs", targetArgument], physicalWebRoot);
/** @type {Array<{ comparisons: Array<{ media: unknown[] }> }>} */
const reviews = await readReviewCatalog();
/** @type {{ milestones: unknown[]; snapshots: Array<{ media: Record<string, unknown> }> }} */
const history = await readHistoryCatalog();
await publishReviews(outputDirectory);

for (const report of ["bot-report/report.json", "analysis-report/report.json"]) {
  await readFile(resolve(outputDirectory, report));
}

/** @typedef {{ path: string; sizeBytes: number; sha256: string }} InventoryFile */

/** @param {string} directory @param {string} [prefix] @returns {Promise<InventoryFile[]>} */
async function inventory(directory, prefix = "") {
  const files = [];
  const entries = await readdir(directory, { withFileTypes: true });
  entries.sort((left, right) => left.name.localeCompare(right.name));
  for (const entry of entries) {
    const relativePath = prefix ? `${prefix}/${entry.name}` : entry.name;
    const absolutePath = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await inventory(absolutePath, relativePath)));
      continue;
    }
    if (!entry.isFile()) throw new Error(`Unexpected non-file export entry: ${relativePath}`);
    const bytes = await readFile(absolutePath);
    files.push({
      path: relativePath,
      sizeBytes: bytes.byteLength,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    });
  }
  return files;
}

const files = await inventory(outputDirectory);
const reviewComparisons = reviews.reduce((count, record) => count + record.comparisons.length, 0);
const reviewMedia = reviews.reduce(
  (count, record) => count + record.comparisons.reduce((pairs, pair) => pairs + pair.media.length, 0),
  0,
);
const historyMedia = history.snapshots.reduce(
  (count, snapshot) => count + Object.values(snapshot.media).filter(Boolean).length,
  0,
);
const receipt = {
  schema: 1,
  source: {
    commit: sourceCommit,
    trackedSourceDirty,
    workingTreeClean,
    qualification: workingTreeClean ? "commit" : "working tree",
  },
  builtAt: new Date().toISOString(),
  tools: { node: process.version, npm: npmVersion, vite: viteInfo.version },
  mode: { rendering: "static", routing: "hash" },
  counts: {
    files: files.length + 1,
    inventoriedFiles: files.length,
    reviewRecords: reviews.length,
    reviewComparisons,
    reviewMedia,
    historyMilestones: history.milestones.length,
    historySnapshots: history.snapshots.length,
    historyMedia,
    visualAssets: files.filter((file) => file.path.startsWith("assets/")).length,
    reports: 2,
  },
  inventoryScope: "All bundle files except export.json, which contains this inventory.",
  files,
};
await writeFile(resolve(outputDirectory, "export.json"), JSON.stringify(receipt, null, 2) + "\n");

console.log(
  `Prepared the static developer preview with ${receipt.counts.files} files at ${outputRelativeToRepo}.`,
);
console.log(
  `Source ${sourceCommit}; tracked source dirty: ${trackedSourceDirty}; tree clean: ${workingTreeClean}.`,
);
