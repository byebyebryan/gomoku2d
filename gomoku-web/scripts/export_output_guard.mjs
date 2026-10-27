import { execFile } from "node:child_process";
import { lstat, mkdir, realpath } from "node:fs/promises";
import { promisify } from "node:util";
import { isAbsolute, relative, resolve, sep } from "node:path";

const execFileAsync = promisify(execFile);

/** @param {string} root @param {string} path @param {boolean} [allowEqual] */
function isContained(root, path, allowEqual = false) {
  const pathFromRoot = relative(root, path);
  return (
    (allowEqual && pathFromRoot === "") ||
    (pathFromRoot !== "" &&
      pathFromRoot !== ".." &&
      !pathFromRoot.startsWith(`..${sep}`) &&
      !isAbsolute(pathFromRoot))
  );
}

/** @param {string} base @param {string} target @param {boolean} createMissing */
async function ensureDirectoryPath(base, target, createMissing) {
  if (!isContained(base, target, true))
    throw new Error(`Export directory leaves its allowed root: ${target}`);

  let current = base;
  const segments = relative(base, target).split(sep).filter(Boolean);
  for (const segment of segments) {
    current = resolve(current, segment);
    try {
      const info = await lstat(current);
      if (!info.isDirectory() || info.isSymbolicLink())
        throw new Error(`Export path components must be real directories: ${current}`);
    } catch (error) {
      if (!(error && typeof error === "object" && "code" in error && error.code === "ENOENT"))
        throw error;
      if (!createMissing) throw error;
      try {
        await mkdir(current);
      } catch (createError) {
        if (
          !(createError && typeof createError === "object" && "code" in createError && createError.code === "EEXIST")
        )
          throw createError;
        const info = await lstat(current);
        if (!info.isDirectory() || info.isSymbolicLink())
          throw new Error(`Export path components must be real directories: ${current}`);
      }
    }
  }
}

/**
 * Create one new directory below an ignored export root after checking the full physical path.
 * @param {{ outputDirectory: string; exportRoot: string; webRoot: string; repoRoot: string }} options
 */
export async function createIgnoredExportDirectory({
  outputDirectory,
  exportRoot,
  webRoot,
  repoRoot,
}) {
  const physicalRepoRoot = await realpath(repoRoot);
  const physicalWebRoot = await realpath(webRoot);
  const testResultsRoot = resolve(physicalWebRoot, "test-results");
  const physicalExportRoot = resolve(exportRoot);
  const physicalOutput = resolve(outputDirectory);

  if (!isContained(physicalRepoRoot, physicalWebRoot))
    throw new Error("The web root must stay under the physical repository root.");
  if (!isContained(testResultsRoot, physicalExportRoot))
    throw new Error("The export root must stay under gomoku-web/test-results.");
  if (!isContained(physicalExportRoot, physicalOutput))
    throw new Error("The export path must stay under its dedicated output root.");

  await ensureDirectoryPath(physicalWebRoot, physicalExportRoot, true);
  if ((await realpath(physicalExportRoot)) !== physicalExportRoot)
    throw new Error("The export root cannot resolve through a symlink.");

  const outputParent = resolve(physicalOutput, "..");
  await ensureDirectoryPath(physicalExportRoot, outputParent, false);
  const outputRelativeToRepo = relative(physicalRepoRoot, physicalOutput).split(sep).join("/");
  if (!outputRelativeToRepo || outputRelativeToRepo.startsWith("../") || isAbsolute(outputRelativeToRepo))
    throw new Error("The export path must stay under the physical repository root.");

  try {
    await execFileAsync("git", ["check-ignore", "--quiet", "--", outputRelativeToRepo], {
      cwd: physicalRepoRoot,
      encoding: "utf8",
    });
  } catch {
    throw new Error("Exports must stay under the ignored test-results root.");
  }

  try {
    const info = await lstat(physicalOutput);
    if (info.isSymbolicLink())
      throw new Error(`Refusing a symlinked export target: ${physicalOutput}`);
    throw new Error(`Refusing to overwrite an existing export target: ${physicalOutput}`);
  } catch (error) {
    if (!(error && typeof error === "object" && "code" in error && error.code === "ENOENT"))
      throw error;
  }

  await mkdir(physicalOutput);
  const createdPath = await realpath(physicalOutput);
  if (createdPath !== physicalOutput || !isContained(physicalExportRoot, createdPath))
    throw new Error("The created export directory escaped its physical output root.");

  return {
    physicalRepoRoot,
    physicalWebRoot,
    outputDirectory: physicalOutput,
    outputRelativeToRepo,
  };
}
