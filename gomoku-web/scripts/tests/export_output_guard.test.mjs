import assert from "node:assert/strict";
import { test } from "node:test";
import {
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  rmdir,
  symlink,
  writeFile,
} from "node:fs/promises";
import { join, resolve } from "node:path";
import { createIgnoredExportDirectory } from "../export_output_guard.mjs";

const webRoot = resolve(import.meta.dirname, "../..");
const repoRoot = resolve(webRoot, "..");
const testResults = resolve(webRoot, "test-results");

test("export output guard rejects physical escapes and creates only new ignored directories", async (t) => {
  let createdTestResults = false;
  try {
    await lstat(testResults);
  } catch (error) {
    if (!(error && typeof error === "object" && "code" in error && error.code === "ENOENT"))
      throw error;
    await mkdir(testResults);
    createdTestResults = true;
  }
  const probeRoot = await mkdtemp(join(testResults, ".export-output-guard-"));
  t.after(async () => {
    await rm(probeRoot, { recursive: true, force: true });
    if (createdTestResults) await rmdir(testResults);
  });

  const guard = (exportRoot, outputDirectory) =>
    createIgnoredExportDirectory({
      outputDirectory,
      exportRoot,
      webRoot,
      repoRoot,
    });

  await t.test("rejects a symlinked export root", async () => {
    const realRoot = join(probeRoot, "root-target");
    const linkedRoot = join(probeRoot, "review-exports-link");
    const output = join(linkedRoot, "new-export");
    await mkdir(realRoot);
    await symlink(realRoot, linkedRoot, "dir");
    await assert.rejects(guard(linkedRoot, output), /real directories/);
    await assert.rejects(lstat(join(realRoot, "new-export")), { code: "ENOENT" });
  });

  await t.test("rejects a symlinked output ancestor", async () => {
    const root = join(probeRoot, "ancestor-root");
    const outside = join(probeRoot, "ancestor-target");
    await mkdir(root);
    await mkdir(outside);
    await symlink(outside, join(root, "nested"), "dir");
    await assert.rejects(
      guard(root, join(root, "nested", "new-export")),
      /real directories/,
    );
    await assert.rejects(lstat(join(outside, "new-export")), { code: "ENOENT" });
  });

  await t.test("rejects a symlinked export target", async () => {
    const root = join(probeRoot, "target-root");
    const outside = join(probeRoot, "target-outside");
    const output = join(root, "export-link");
    await mkdir(root);
    await mkdir(outside);
    await symlink(outside, output, "dir");
    await assert.rejects(guard(root, output), /symlinked export target/);
    await assert.rejects(lstat(join(outside, "index.html")), { code: "ENOENT" });
  });

  await t.test("rejects existing directory and file targets", async () => {
    const root = join(probeRoot, "existing-root");
    const existingDirectory = join(root, "existing-directory");
    const existingFile = join(root, "existing-file");
    await mkdir(root);
    await mkdir(existingDirectory);
    await writeFile(existingFile, "owned probe\n");
    await assert.rejects(guard(root, existingDirectory), /existing export target/);
    await assert.rejects(guard(root, existingFile), /existing export target/);
    assert.equal(await readFile(existingFile, "utf8"), "owned probe\n");
  });

  await t.test("rejects source and dist paths outside the dedicated root", async () => {
    const root = join(probeRoot, "escape-root");
    await mkdir(root);
    for (const destination of [
      resolve(webRoot, "src", "review-export-probe"),
      resolve(webRoot, "dist", "review-export-probe"),
    ]) {
      await assert.rejects(guard(root, destination), /dedicated output root/);
      await assert.rejects(lstat(destination), { code: "ENOENT" });
    }
  });

  await t.test("creates a physically contained ignored output exclusively", async () => {
    const root = join(probeRoot, "safe-root");
    const output = join(root, "new-export");
    await mkdir(root);
    const created = await guard(root, output);
    assert.equal(created.outputDirectory, output);
    assert.equal(await lstat(output).then((info) => info.isDirectory()), true);
    assert.match(created.outputRelativeToRepo, /^gomoku-web\/test-results\//);
    await assert.rejects(guard(root, output), /existing export target/);
  });
});
