#!/usr/bin/env node
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { createBuildReceipt, validateNewOutputPath } from "./capture_retrospective_ui.mjs";

/** @param {string[]} args */
export function parseArguments(args) {
  /** @type {{ configPath?: string, outputPath?: string, help: boolean }} */
  const parsed = { help: false };
  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === "--help" || argument === "-h") {
      parsed.help = true;
      continue;
    }
    if (argument !== "--config" && argument !== "--output") {
      throw new Error("Unknown argument: " + argument);
    }
    const value = args[index + 1];
    if (!value || value.startsWith("--")) throw new Error("Missing value after " + argument);
    index += 1;
    const key = argument === "--config" ? "configPath" : "outputPath";
    if (parsed[key]) throw new Error(argument + " may be supplied only once.");
    parsed[key] = value;
  }
  if (!parsed.help && (!parsed.configPath || !parsed.outputPath)) {
    throw new Error("Both --config and --output are required.");
  }
  return parsed;
}

function usage() {
  return [
    "Usage:",
    "  npm run capture:retrospective:receipt -- --config <build.json> --output <new-receipt.json>",
    "",
    "Config shape:",
    "  { schemaVersion: 1, tag, expectedCommit, worktree, environment }",
    "  environment requires node, npm, rustc, wasmPack, firebaseMode; optional fields:",
    "  declaredNode, wasmOpt, wasmBuild, npmInstall.",
    "",
    "Run this read-only recorder immediately after the documented build. It does not build, install, or start a server.",
    "The receipt output must be a new JSON file outside the source worktree.",
  ].join("\n");
}

/** @param {unknown} value */
function requireRecord(value) {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("Build receipt config must be an object.");
  }
  return /** @type {Record<string, any>} */ (value);
}

/** @param {string} configPath @param {string} outputArgument */
async function main(configPath, outputArgument) {
  const absoluteConfig = resolve(configPath);
  const config = requireRecord(JSON.parse(await readFile(absoluteConfig, "utf8")));
  if (config.schemaVersion !== 1) throw new Error("Build receipt config schemaVersion must be 1.");
  if (typeof config.worktree !== "string" || !config.worktree.startsWith("/")) {
    throw new Error("Build receipt config worktree must be an absolute path.");
  }
  const receiptInput = {
    tag: config.tag,
    expectedCommit: config.expectedCommit,
    worktree: config.worktree,
    environment: config.environment,
  };
  if (!/\.json$/i.test(outputArgument)) throw new Error("Receipt output filename must end in .json.");

  const outputPath = await validateNewOutputPath(
    outputArgument,
    [config.worktree, absoluteConfig],
    "file",
  );
  const receipt = await createBuildReceipt(receiptInput);
  const document = { schemaVersion: 1, ...receipt };
  await writeFile(outputPath, JSON.stringify(document, null, 2) + "\n", { flag: "wx" });
  process.stdout.write(JSON.stringify({
    status: "recorded",
    tag: receipt.tag,
    sourceCommit: receipt.sourceCommit,
    sourceTree: receipt.sourceTree,
    distSha256: receipt.distSha256,
    output: outputPath,
    assumption: receipt.assumption,
  }, null, 2) + "\n");
}

/** @param {string | undefined} value */
function isDirectInvocation(value) {
  return Boolean(value && pathToFileURL(resolve(value)).href === import.meta.url);
}

if (isDirectInvocation(process.argv[1])) {
  try {
    const parsed = parseArguments(process.argv.slice(2));
    if (parsed.help) {
      process.stdout.write(usage() + "\n");
    } else if (parsed.configPath && parsed.outputPath) {
      await main(parsed.configPath, parsed.outputPath);
    }
  } catch (error) {
    process.stderr.write((error instanceof Error ? error.message : String(error)) + "\n");
    process.exitCode = 1;
  }
}
