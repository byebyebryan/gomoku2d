#!/usr/bin/env node
import { chromium } from "@playwright/test";
import { createHash } from "node:crypto";
import { execFile } from "node:child_process";
import {
  lstat,
  mkdir,
  readFile,
  readdir,
  realpath,
  writeFile,
} from "node:fs/promises";
import { promisify } from "node:util";
import {
  basename,
  dirname,
  isAbsolute,
  join,
  relative,
  resolve,
  sep,
} from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

/** @typedef {import("@playwright/test").Browser} Browser */
/** @typedef {import("@playwright/test").BrowserContext} BrowserContext */
/** @typedef {import("@playwright/test").Page} Page */
/** @typedef {import("@playwright/test").Route} Route */
/** @typedef {import("@playwright/test").Request} Request */
/** @typedef {import("@playwright/test").ConsoleMessage} ConsoleMessage */
/** @typedef {import("@playwright/test").WebSocketRoute} WebSocketRoute */

/**
 * Config schema:
 * {
 *   schemaVersion: 1,
 *   sources: [{
 *     tag, expectedCommit, worktree, baseUrl, adapter,
 *     buildReceipt: {
 *       recordedAt, tag?, recordedBy?, assumption?, sourceCommit, sourceTree, applicationTree?,
 *       packageLockSha256, cargoLockSha256, wasmSha256, distSha256, indexSha256,
 *       files?: [{path,sha256}],
 *       environment: { node, npm, rustc, wasmPack, firebaseMode,
 *                       declaredNode?, wasmOpt?, wasmBuild?, npmInstall? }
 *     }
 *   }]
 * }
 *
 * distSha256 is SHA-256 of UTF-8 JSON.stringify of the sorted array
 * [{path,sha256}, ...], followed by one newline. File hashes are raw bytes.
 */

const thisDirectory = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(thisDirectory, "..");
const repositoryRoot = resolve(webRoot, "..");
const executeFile = promisify(execFile);
/** @type {Record<string, { adapter: string }>} */
const EXPECTED_SOURCES = Object.freeze({
  "v0.2.4": Object.freeze({ adapter: "v024" }),
  "v0.6.2": Object.freeze({ adapter: "v062" }),
});
const RESERVED_PORTS = new Set([
  3000, // Vite default
  3001, // Vite development server
  4000, // Firebase emulator UI
  5000, // Firebase Hosting emulator
  5001, // Firebase Functions emulator
  8001, // Manual preview
  8013, // Manual service
  8080, // Firestore emulator
  8085, // Firebase Pub/Sub emulator
  9000, // Firebase Realtime Database emulator
  9099, // Firebase Auth emulator
  9199, // Firebase Storage emulator
  5173, // Vite default
  46862, // Carbon preview
]);
const MOBILE_QUERY =
  "(max-width: 720px) and (orientation: portrait) and (hover: none) and (pointer: coarse)";
const FIXTURE = Object.freeze({
  displayName: "Reference Player",
  matchId: "retrospective-capture-match",
  profileId: "retrospective-capture-profile",
  savedAt: "2026-10-24T00:00:00.000Z",
  variant: "freestyle",
  moveCells: Object.freeze([112, 0, 113, 15, 114, 30, 115, 1, 116]),
  winningCells: Object.freeze([
    Object.freeze({ row: 7, col: 7 }),
    Object.freeze({ row: 7, col: 8 }),
    Object.freeze({ row: 7, col: 9 }),
    Object.freeze({ row: 7, col: 10 }),
    Object.freeze({ row: 7, col: 11 }),
  ]),
});
const FONT_FAMILIES = Object.freeze([
  "PixelOperator",
  "PixelOperatorBold",
  "PixelOperator8Bold",
]);
const PROFILES = Object.freeze([
  Object.freeze({
    id: "desktop",
    viewport: Object.freeze({ width: 1280, height: 900 }),
    deviceScaleFactor: 1,
    isMobile: false,
    hasTouch: false,
    userAgentKind: "desktop Chromium default",
  }),
  Object.freeze({
    id: "portrait",
    viewport: Object.freeze({ width: 412, height: 800 }),
    deviceScaleFactor: 1,
    isMobile: true,
    hasTouch: true,
    userAgentKind: "generic Android mobile; not a measured OnePlus profile",
  }),
]);

function usage() {
  return [
    "Usage:",
    "  npm run capture:retrospective -- --config <sources.json> --output <new-directory>",
    "  npm run capture:retrospective -- --config <sources.json> --output <new-directory> --validate-only",
    "",
    "The output directory must not exist and must be outside this checkout and both source worktrees.",
    "Both preview URLs must be explicit loopback origins on distinct, non-reserved ports.",
    "See the script header for the JSON config and build receipt schema.",
  ].join("\n");
}

/**
 * @param {string[]} args
 * @returns {{ configPath?: string, outputPath?: string, validateOnly: boolean, help: boolean }}
 */
export function parseArguments(args) {
  /** @type {{ configPath?: string, outputPath?: string, validateOnly: boolean, help: boolean }} */
  const parsed = { validateOnly: false, help: false };
  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === "--help" || argument === "-h") {
      parsed.help = true;
      continue;
    }
    if (argument === "--validate-only") {
      parsed.validateOnly = true;
      continue;
    }
    if (argument !== "--config" && argument !== "--output") {
      throw new Error("Unknown argument: " + argument);
    }
    const value = args[index + 1];
    if (!value || value.startsWith("--")) {
      throw new Error("Missing value after " + argument);
    }
    index += 1;
    if (argument === "--config") {
      if (parsed.configPath) throw new Error("--config may be supplied only once.");
      parsed.configPath = value;
    } else {
      if (parsed.outputPath) throw new Error("--output may be supplied only once.");
      parsed.outputPath = value;
    }
  }
  if (!parsed.help && (!parsed.configPath || !parsed.outputPath)) {
    throw new Error("Both --config and --output are required.");
  }
  return parsed;
}

/** @param {unknown} value @returns {value is Record<string, any>} */
function isRecord(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** @param {unknown} value @param {string} label */
function requireRecord(value, label) {
  if (!isRecord(value)) throw new Error(label + " must be an object.");
  return value;
}

/** @param {unknown} value @param {string} label */
function requireNonEmptyString(value, label) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(label + " must be a non-empty string.");
  }
  return value.trim();
}

/** @param {unknown} value @param {string} label @param {number} length */
function requireHex(value, label, length) {
  const normalized = requireNonEmptyString(value, label).toLowerCase();
  if (!new RegExp("^[0-9a-f]{" + length + "}$").test(normalized)) {
    throw new Error(label + " must be a " + length + "-character hexadecimal value.");
  }
  return normalized;
}

/**
 * @param {unknown} input
 * @returns {{ schemaVersion: 1, sources: Array<Record<string, any>> }}
 */
export function validateConfigShape(input) {
  const config = requireRecord(input, "Config");
  if (config.schemaVersion !== 1) throw new Error("Config schemaVersion must be 1.");
  if (!Array.isArray(config.sources) || config.sources.length !== 2) {
    throw new Error("Config sources must contain exactly v0.2.4 and v0.6.2.");
  }
  const tags = new Set();
  const ports = new Set();
  const sources = config.sources.map((raw, index) => {
    const source = requireRecord(raw, "sources[" + index + "]");
    const tag = requireNonEmptyString(source.tag, "sources[" + index + "].tag");
    const expected = EXPECTED_SOURCES[tag];
    if (!expected) throw new Error("Unsupported source tag: " + tag);
    if (tags.has(tag)) throw new Error("Duplicate source tag: " + tag);
    tags.add(tag);
    const adapter = requireNonEmptyString(source.adapter, tag + ".adapter");
    if (adapter !== expected.adapter) {
      throw new Error(tag + " requires adapter " + expected.adapter + ".");
    }
    const expectedCommit = requireHex(source.expectedCommit, tag + ".expectedCommit", 40);
    const worktree = requireNonEmptyString(source.worktree, tag + ".worktree");
    if (!isAbsolute(worktree)) throw new Error(tag + ".worktree must be absolute.");
    const baseUrl = requireNonEmptyString(source.baseUrl, tag + ".baseUrl");
    let parsedUrl;
    try {
      parsedUrl = new URL(baseUrl);
    } catch {
      throw new Error(tag + ".baseUrl must be an absolute HTTP loopback URL.");
    }
    if (parsedUrl.protocol !== "http:" || !["127.0.0.1", "localhost", "[::1]"].includes(parsedUrl.hostname)
      || parsedUrl.username || parsedUrl.password || parsedUrl.pathname !== "/" || parsedUrl.search || parsedUrl.hash) {
      throw new Error(tag + ".baseUrl must be a root HTTP loopback URL without credentials or query data.");
    }
    const port = Number(parsedUrl.port || "80");
    if (!Number.isInteger(port) || port < 1024 || port > 65535 || RESERVED_PORTS.has(port)) {
      throw new Error(tag + ".baseUrl uses a reserved or invalid port: " + port);
    }
    if (ports.has(port)) throw new Error("Each source must use a different preview port.");
    ports.add(port);
    const buildReceipt = requireRecord(source.buildReceipt, tag + ".buildReceipt");
    return {
      tag,
      adapter,
      expectedCommit,
      worktree,
      baseUrl: parsedUrl.href,
      previewOrigin: parsedUrl.origin,
      previewPort: port,
      buildReceipt,
    };
  });
  if (!tags.has("v0.2.4") || !tags.has("v0.6.2")) {
    throw new Error("Config sources must include exactly v0.2.4 and v0.6.2.");
  }
  return { schemaVersion: 1, sources };
}

/**
 * @param {string} candidate
 * @param {string} root
 */
export function pathOverlaps(candidate, root) {
  const resolvedCandidate = resolve(candidate);
  const resolvedRoot = resolve(root);
  /** @param {string} child @param {string} parent */
  const within = (child, parent) => {
    const relation = relative(parent, child);
    return relation === "" || (!relation.startsWith(".." + sep) && relation !== ".." && !isAbsolute(relation));
  };
  return within(resolvedCandidate, resolvedRoot) || within(resolvedRoot, resolvedCandidate);
}

/**
 * Validate the requested output without creating it. Parent directory must exist.
 *
 * @param {string} outputArgument
 * @param {string[]} protectedPaths
 * @param {"directory" | "file"} kind
 * @returns {Promise<string>}
 */
export async function validateNewOutputPath(outputArgument, protectedPaths, kind = "directory") {
  const requested = resolve(outputArgument);
  const parentPath = dirname(requested);
  let parent;
  try {
    parent = await realpath(parentPath);
  } catch {
    throw new Error("Output parent must already exist: " + parentPath);
  }
  const parentInfo = await lstat(parent);
  if (!parentInfo.isDirectory()) throw new Error("Output parent is not a directory.");
  const outputPath = join(parent, basename(requested));
  try {
    await lstat(outputPath);
    throw new Error("Output already exists; choose a new " + kind + ": " + outputPath);
  } catch (error) {
    if (error instanceof Error && error.message.startsWith("Output already exists")) throw error;
    if (!isRecord(error) || error.code !== "ENOENT") throw error;
  }
  for (const protectedPath of protectedPaths) {
    const realProtectedPath = await realpath(protectedPath);
    if (pathOverlaps(outputPath, realProtectedPath)) {
      throw new Error("Output must be disjoint from protected source/build root: " + realProtectedPath);
    }
  }
  return outputPath;
}

/**
 * Validate the requested output directory without creating it. Parent directory must exist.
 * @param {string} outputArgument
 * @param {string[]} protectedPaths
 * @returns {Promise<string>}
 */
export async function validateOutputPath(outputArgument, protectedPaths) {
  return validateNewOutputPath(outputArgument, protectedPaths, "directory");
}

/** @param {Buffer | Uint8Array} bytes */
function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

/** @param {string} path */
async function sha256File(path) {
  return sha256(await readFile(path));
}

/** @param {string} root */
async function buildFileInventory(root) {
  /** @type {Array<{path: string, sha256: string}>} */
  const inventory = [];
  /** @param {string} left @param {string} right */
  const lexicalCompare = (left, right) => left < right ? -1 : left > right ? 1 : 0;
  /** @param {string} directory @param {string} prefix */
  async function visit(directory, prefix) {
    const entries = await readdir(directory, { withFileTypes: true });
    entries.sort((left, right) => lexicalCompare(left.name, right.name));
    for (const entry of entries) {
      const absolutePath = join(directory, entry.name);
      const relativePath = prefix ? prefix + "/" + entry.name : entry.name;
      if (entry.isSymbolicLink()) {
        throw new Error("Build tree contains a symlink: " + relativePath);
      }
      if (entry.isDirectory()) {
        await visit(absolutePath, relativePath);
      } else if (entry.isFile()) {
        inventory.push({ path: relativePath, sha256: await sha256File(absolutePath) });
      } else {
        throw new Error("Build tree contains an unsupported file: " + relativePath);
      }
    }
  }
  await visit(root, "");
  inventory.sort((left, right) => lexicalCompare(left.path, right.path));
  return inventory;
}

/** @param {Array<{path: string, sha256: string}>} inventory */
function digestFileInventory(inventory) {
  return sha256(Buffer.from(JSON.stringify(inventory) + "\n", "utf8"));
}

/** @param {string} cwd @param {string[]} args */
async function git(cwd, args) {
  const result = await executeFile("git", args, { cwd, encoding: "utf8", maxBuffer: 4 * 1024 * 1024 });
  return String(result.stdout).trim();
}

/** @param {string} path */
async function requireDirectory(path) {
  const info = await lstat(path);
  if (!info.isDirectory() || info.isSymbolicLink()) {
    throw new Error("Expected a real directory: " + path);
  }
}

/** @param {string} path */
async function requireFile(path) {
  const info = await lstat(path);
  if (!info.isFile() || info.isSymbolicLink()) {
    throw new Error("Expected a regular file: " + path);
  }
}

/** @param {string} value @param {string} label */
function requireIsoDate(value, label) {
  const normalized = requireNonEmptyString(value, label);
  if (!Number.isFinite(Date.parse(normalized))) throw new Error(label + " must be an ISO date.");
  return normalized;
}

/**
 * Read the exact source/build identity after a documented build. This function
 * does not build, install, start a server, or write into the source worktree.
 * @param {Record<string, any>} source
 */
async function inspectTaggedBuild(source) {
  if (!EXPECTED_SOURCES[source.tag]) throw new Error("Unsupported source tag: " + source.tag);
  const root = await realpath(source.worktree);
  await requireDirectory(root);
  if (pathOverlaps(root, repositoryRoot)) {
    throw new Error(source.tag + " worktree must be separate from the capture tool checkout.");
  }
  const gitRoot = await realpath(await git(root, ["rev-parse", "--show-toplevel"]));
  if (gitRoot !== root) throw new Error(source.tag + " worktree path must be the Git repository root.");
  const head = (await git(root, ["rev-parse", "HEAD"])).toLowerCase();
  const expectedCommit = requireHex(source.expectedCommit, source.tag + ".expectedCommit", 40);
  if (head !== expectedCommit) {
    throw new Error(source.tag + " HEAD mismatch: expected " + expectedCommit + ", got " + head);
  }
  const tagCommit = (await git(root, ["rev-parse", source.tag + "^{commit}"])).toLowerCase();
  if (tagCommit !== expectedCommit) throw new Error(source.tag + " tag does not resolve to expectedCommit.");
  const branch = await git(root, ["branch", "--show-current"]);
  if (branch !== "") throw new Error(source.tag + " source must be a detached worktree.");
  const dirty = await git(root, ["status", "--porcelain", "--untracked-files=all"]);
  if (dirty !== "") throw new Error(source.tag + " source worktree is not clean:\n" + dirty);

  const sourceTree = (await git(root, ["rev-parse", "HEAD^{tree}"])).toLowerCase();
  const applicationTree = (await git(root, ["rev-parse", "HEAD:gomoku-web/src"])).toLowerCase();
  const packagePath = join(root, "gomoku-web", "package.json");
  const packageLockPath = join(root, "gomoku-web", "package-lock.json");
  const cargoLockPath = join(root, "gomoku-bot-lab", "Cargo.lock");
  const distPath = join(root, "gomoku-web", "dist");
  const indexPath = join(distPath, "index.html");
  const wasmPath = join(root, "gomoku-bot-lab", "gomoku-wasm", "pkg", "gomoku_wasm_bg.wasm");
  await Promise.all([
    requireFile(packagePath),
    requireFile(packageLockPath),
    requireFile(cargoLockPath),
    requireFile(wasmPath),
    requireDirectory(distPath),
    requireFile(indexPath),
  ]);
  const packageJson = JSON.parse(await readFile(packagePath, "utf8"));
  if (packageJson.version !== source.tag.slice(1)) {
    throw new Error(source.tag + " package version does not match the tag.");
  }
  const distFiles = await buildFileInventory(distPath);
  return {
    tag: source.tag,
    worktreeRoot: root,
    head,
    commit: head,
    sourceTree,
    applicationTree,
    packageVersion: packageJson.version,
    packageLockSha256: await sha256File(packageLockPath),
    cargoLockSha256: await sha256File(cargoLockPath),
    wasmSha256: await sha256File(wasmPath),
    distSha256: digestFileInventory(distFiles),
    distFiles,
    indexSha256: await sha256File(indexPath),
    indexPath,
  };
}

/**
 * @param {unknown} environmentInput
 * @param {string} label
 */
function normalizedEnvironment(environmentInput, label) {
  const environment = requireRecord(environmentInput, label);
  /** @type {Record<string, string>} */
  const normalized = {
    node: requireNonEmptyString(environment.node, label + ".node"),
    npm: requireNonEmptyString(environment.npm, label + ".npm"),
    rustc: requireNonEmptyString(environment.rustc, label + ".rustc"),
    wasmPack: requireNonEmptyString(environment.wasmPack, label + ".wasmPack"),
    firebaseMode: requireNonEmptyString(environment.firebaseMode, label + ".firebaseMode"),
  };
  if (normalized.firebaseMode !== "unconfigured-local-guest") {
    throw new Error(label + ".firebaseMode must be unconfigured-local-guest for this pilot.");
  }
  for (const optionalField of ["declaredNode", "wasmOpt", "wasmBuild", "npmInstall"]) {
    if (environment[optionalField] !== undefined) {
      normalized[optionalField] = requireNonEmptyString(environment[optionalField], label + "." + optionalField);
    }
  }
  return normalized;
}

/**
 * Generate a build receipt immediately after the documented build. The receipt
 * remains valid only while the clean source, generated Wasm and dist hashes match.
 * @param {{ tag: string, expectedCommit: string, worktree: string, environment: unknown }} input
 */
export async function createBuildReceipt(input) {
  const normalized = {
    tag: requireNonEmptyString(input.tag, "tag"),
    expectedCommit: requireHex(input.expectedCommit, "expectedCommit", 40),
    worktree: requireNonEmptyString(input.worktree, "worktree"),
    environment: normalizedEnvironment(input.environment, "environment"),
  };
  if (!isAbsolute(normalized.worktree)) throw new Error("worktree must be absolute.");
  const inspected = await inspectTaggedBuild(normalized);
  const confirmation = await inspectTaggedBuild(normalized);
  /** @type {Array<keyof typeof inspected>} */
  const receiptFields = [
    "head", "sourceTree", "applicationTree", "packageLockSha256", "cargoLockSha256",
    "wasmSha256", "distSha256", "indexSha256", "distFiles",
  ];
  for (const field of receiptFields) {
    const same = field === "distFiles"
      ? JSON.stringify(inspected[field]) === JSON.stringify(confirmation[field])
      : inspected[field] === confirmation[field];
    if (!same) {
      throw new Error(normalized.tag + " source or build artifacts changed during receipt inspection.");
    }
  }
  return {
    recordedAt: new Date().toISOString(),
    assumption: "Recorded immediately after the documented build; later source or artifact changes invalidate this receipt.",
    tag: inspected.tag,
    recordedBy: { tool: "record_retrospective_build.mjs", node: process.version },
    sourceCommit: inspected.head,
    sourceTree: inspected.sourceTree,
    applicationTree: inspected.applicationTree,
    packageLockSha256: inspected.packageLockSha256,
    cargoLockSha256: inspected.cargoLockSha256,
    wasmSha256: inspected.wasmSha256,
    distSha256: inspected.distSha256,
    files: inspected.distFiles,
    indexSha256: inspected.indexSha256,
    environment: normalized.environment,
  };
}

/**
 * @param {Record<string, any>} source
 * @returns {Promise<Record<string, any>>}
 */
async function inspectSource(source) {
  const inspected = await inspectTaggedBuild(source);
  if (typeof inspected.commit !== "string" || inspected.commit !== inspected.head) {
    throw new Error(source.tag + " source commit identity is missing or inconsistent.");
  }
  const receipt = source.buildReceipt;
  const recordedAt = requireIsoDate(receipt.recordedAt, source.tag + ".buildReceipt.recordedAt");
  const receiptEnvironment = normalizedEnvironment(
    receipt.environment,
    source.tag + ".buildReceipt.environment",
  );
  if (receipt.sourceCommit !== inspected.head
    || receipt.sourceTree !== inspected.sourceTree
    || (receipt.applicationTree !== undefined && receipt.applicationTree !== inspected.applicationTree)
    || receipt.packageLockSha256 !== inspected.packageLockSha256
    || receipt.cargoLockSha256 !== inspected.cargoLockSha256
    || receipt.wasmSha256 !== inspected.wasmSha256
    || receipt.distSha256 !== inspected.distSha256
    || receipt.indexSha256 !== inspected.indexSha256
    || (receipt.files !== undefined && JSON.stringify(receipt.files) !== JSON.stringify(inspected.distFiles))) {
    throw new Error(source.tag + " build receipt does not match the clean source and build artifacts.");
  }
  if (receipt.tag !== undefined && receipt.tag !== source.tag) {
    throw new Error(source.tag + " build receipt tag does not match the configured source.");
  }

  const served = await fetch(source.baseUrl, {
    redirect: "manual",
    signal: AbortSignal.timeout(10_000),
  });
  if (!served.ok) throw new Error(source.tag + " preview returned HTTP " + served.status + ".");
  const servedBytes = Buffer.from(await served.arrayBuffer());
  if (!servedBytes.equals(await readFile(inspected.indexPath))) {
    throw new Error(source.tag + " preview index.html does not match the prepared dist.");
  }

  return {
    ...inspected,
    adapter: source.adapter,
    buildReceipt: {
      tag: source.tag,
      recordedAt,
      ...(typeof receipt.assumption === "string" ? { assumption: receipt.assumption } : {}),
      ...(isRecord(receipt.recordedBy) ? { recordedBy: receipt.recordedBy } : {}),
      environment: receiptEnvironment,
    },
    preview: {
      origin: source.previewOrigin,
      port: source.previewPort,
      indexMatchesBuild: true,
    },
    baseUrl: source.baseUrl,
  };
}

/**
 * Build the tag-specific supported localStorage fixture. This writes through the
 * browser's ordinary localStorage API before app bootstrap; it does not replace it.
 * @param {"v024" | "v062"} adapter
 */
function localStorageSeed(adapter) {
  if (adapter === "v024") {
    const players = [
      { kind: "human", name: FIXTURE.displayName, stone: "black" },
      { kind: "bot", name: "Practice Bot", stone: "white" },
    ];
    const moves = FIXTURE.moveCells.map((cell, index) => ({
      col: cell % 15,
      row: Math.floor(cell / 15),
      moveNumber: index + 1,
      player: index % 2 === 0 ? 1 : 2,
    }));
    return {
      key: "gomoku2d.guest-profile.v1",
      value: {
        state: {
          history: [{
            guestStone: "black",
            id: FIXTURE.matchId,
            mode: "bot",
            moves,
            players,
            savedAt: FIXTURE.savedAt,
            status: "black_won",
            undoFloor: 0,
            variant: FIXTURE.variant,
            winningCells: FIXTURE.winningCells,
          }],
          profile: {
            avatarUrl: null,
            createdAt: FIXTURE.savedAt,
            displayName: FIXTURE.displayName,
            id: FIXTURE.profileId,
            kind: "guest",
            updatedAt: FIXTURE.savedAt,
            username: null,
          },
          settings: { preferredVariant: FIXTURE.variant },
        },
        version: 0,
      },
    };
  }

  const emptyCounter = () => ({ draws: 0, losses: 0, matches: 0, moves: 0, wins: 0 });
  const normalBot = {
    config: { mode: "preset", preset: "normal", version: 1 },
    config_version: 1,
    engine: "search_bot",
    id: "bot",
    label: "Normal",
    lab_spec: "search-d3+pattern-eval",
    version: 1,
  };
  const profile = {
    avatarUrl: null,
    createdAt: FIXTURE.savedAt,
    displayName: FIXTURE.displayName,
    id: FIXTURE.profileId,
    kind: "local",
    updatedAt: FIXTURE.savedAt,
    username: null,
  };
  const replayMatch = {
    board_size: 15,
    id: FIXTURE.matchId,
    match_kind: "local_vs_bot",
    move_cells: [...FIXTURE.moveCells],
    move_count: FIXTURE.moveCells.length,
    player_black: {
      bot: null,
      display_name: FIXTURE.displayName,
      kind: "human",
      local_profile_id: profile.id,
      profile_uid: null,
    },
    player_white: {
      bot: normalBot,
      display_name: "Normal Bot",
      kind: "bot",
      local_profile_id: null,
      profile_uid: null,
    },
    saved_at: FIXTURE.savedAt,
    schema_version: 2,
    source: "local_history",
    status: "black_won",
    trust: "local_only",
    undo_floor: 0,
    ruleset: FIXTURE.variant,
  };
  return {
    key: "gomoku2d.local-profile.v5",
    value: {
      state: {
        matchHistory: {
          archivedStats: {
            archived_before: null,
            archived_count: 0,
            by_opponent_type: { bot: emptyCounter(), human: emptyCounter() },
            by_ruleset: { freestyle: emptyCounter(), renju: emptyCounter() },
            by_side: { black: emptyCounter(), white: emptyCounter() },
            schema_version: 1,
            totals: emptyCounter(),
          },
          replayMatches: [replayMatch],
          summaryMatches: [],
        },
        profile,
        settings: {
          boardHints: {
            immediate: "win_threat",
            imminent: "threat_counter",
          },
          botConfig: { mode: "preset", preset: "normal", version: 1 },
          gameConfig: { opening: "standard", ruleset: FIXTURE.variant },
          touchControl: "touchpad",
        },
      },
      version: 5,
    },
  };
}

/** @param {BrowserContext} context @param {"v024" | "v062"} adapter */
async function installFixture(context, adapter) {
  const seed = localStorageSeed(adapter);
  const serialized = JSON.stringify(seed.value);
  await context.addInitScript(
    ({ key, value }) => {
      window.localStorage.setItem(key, value);
    },
    { key: seed.key, value: serialized },
  );
}

/** @param {string} browserVersion */
function genericAndroidUserAgent(browserVersion) {
  const major = browserVersion.split(".")[0];
  return "Mozilla/5.0 (Linux; Android 15; Mobile) AppleWebKit/537.36 "
    + "(KHTML, like Gecko) Chrome/" + major + ".0.0.0 Mobile Safari/537.36";
}

/**
 * @param {BrowserContext} context
 * @param {string} allowedOrigin
 * @returns {Record<string, any>}
 */
function installNetworkAndErrorObservers(context, allowedOrigin) {
  /** @type {Array<Record<string, string>>} */
  const blockedRequests = [];
  /** @type {Array<Record<string, string>>} */
  const failedRequests = [];
  /** @type {Array<Record<string, string>>} */
  const consoleErrors = [];
  /** @type {Array<Record<string, string>>} */
  const pageErrors = [];
  const blocked = new WeakSet();
  let allowedRequestCount = 0;
  const identityOrDataPath =
    /(?:identitytoolkit\.googleapis\.com|securetoken\.googleapis\.com|firestore\.googleapis\.com|\/(?:api|auth|cloud|firebase|firestore|identity)(?:\/|$)|\/v1\/projects\/.*\/databases|\/__\/auth\/|\/oauth2\/|\/accounts:|\/token(?:\/|$))/i;

  /** @param {Route} route */
  async function intercept(route) {
    const request = route.request();
    const url = new URL(request.url());
    const method = request.method();
    let reason = "";
    if (url.origin !== allowedOrigin) {
      reason = "off-origin";
    } else if (identityOrDataPath.test(url.host + url.pathname)) {
      reason = "identity-or-data-endpoint";
    } else if (method !== "GET" && method !== "HEAD") {
      reason = "non-read-request";
    }
    if (reason) {
      blocked.add(request);
      blockedRequests.push({
        reason,
        method,
        origin: url.origin,
        path: url.pathname,
        resourceType: request.resourceType(),
      });
      await route.abort("blockedbyclient");
      return;
    }
    allowedRequestCount += 1;
    await route.continue();
  }

  /** @param {WebSocketRoute} route */
  async function interceptWebSocket(route) {
    const url = new URL(route.url());
    blockedRequests.push({
      reason: url.origin === allowedOrigin ? "unexpected-websocket" : "off-origin-websocket",
      method: "WEBSOCKET",
      origin: url.origin,
      path: url.pathname,
      resourceType: "websocket",
    });
    await route.close({ code: 1008, reason: "Retrospective capture blocks network connections." });
  }

  /** @param {Page} page */
  function watchPage(page) {
    page.on("console", (message) => {
      if (message.type() === "error") {
        consoleErrors.push({ text: redactText(message.text()) });
      }
    });
    page.on("pageerror", (error) => {
      pageErrors.push({ message: redactText(error.message) });
    });
    page.on("requestfailed", (request) => {
      if (blocked.has(request)) return;
      const url = new URL(request.url());
      failedRequests.push({
        method: request.method(),
        origin: url.origin,
        path: url.pathname,
        failure: redactText(request.failure()?.errorText ?? "unknown"),
      });
    });
  }

  return {
    blockedRequests,
    failedRequests,
    consoleErrors,
    pageErrors,
    allowedRequestCount: () => allowedRequestCount,
    intercept,
    interceptWebSocket,
    watchPage,
  };
}

/** @param {string} value */
export function redactText(value) {
  return value
    .replace(/https?:\/\/[^\s"'<>]+/g, (raw) => {
      try {
        const url = new URL(raw);
        return url.origin + url.pathname + (url.search ? "?[redacted]" : "");
      } catch {
        return "[URL]";
      }
    })
    .replace(/(apiKey|access_token|id_token|refresh_token|token)=([^&\s]+)/gi, "$1=[redacted]")
    .replace(/\/home\/[^/\s]+/g, "[private-path]")
    .slice(0, 1000);
}

/** @param {Page} page @param {string} baseUrl @param {string} path */
async function navigate(page, baseUrl, path) {
  const target = new URL(path.replace(/^\/+/, ""), baseUrl).href;
  await page.goto(target, { waitUntil: "domcontentloaded" });
  const actual = new URL(page.url());
  const expected = new URL(target);
  if (actual.origin !== expected.origin || actual.pathname !== expected.pathname) {
    throw new Error("Unexpected route after navigation: " + actual.pathname);
  }
}

/** @param {Page} page */
async function waitForFonts(page) {
  const result = await page.evaluate(async (families) => {
    const loadedByFamily = await Promise.all(families.map(async (family) => ({
      family,
      faces: await document.fonts.load('16px "' + family + '"'),
    })));
    await document.fonts.ready;
    return loadedByFamily.map(({ family, faces }) => ({
      family,
      loadedFaceCount: faces.length,
      loaded: faces.length > 0 && faces.every((face) => face.status === "loaded"),
      status: document.fonts.status,
    }));
  }, [...FONT_FAMILIES]);
  for (const font of result) {
    if (!font.loaded || font.status !== "loaded") {
      throw new Error("Font not ready: " + font.family);
    }
  }
  return result.map(({ family, loadedFaceCount }) => ({ family, loadedFaceCount, loaded: true }));
}

/** @param {Page} page */
async function waitForBoard(page) {
  await page.waitForFunction(() => {
    const canvas = document.querySelector("canvas");
    return Boolean(canvas && canvas.width > 0 && canvas.height > 0);
  }, undefined, { timeout: 20_000 });
  const observation = await page.evaluate(() => {
    const canvas = document.querySelector("canvas");
    const grid = document.querySelector('[role="grid"][aria-label="Gomoku board"]');
    return {
      canvas: canvas ? { width: canvas.width, height: canvas.height } : null,
      gridVisible: Boolean(grid),
      renderer: {
        value: "unknown",
        reason: "The app does not expose its Phaser renderer; the canvas context was not probed.",
      },
    };
  });
  if (!observation.canvas || observation.canvas.width <= 0 || observation.canvas.height <= 0) {
    throw new Error("Board canvas did not have nonzero intrinsic dimensions.");
  }
  return observation;
}

/** @param {Page} page @param {{width:number,height:number}} viewport */
async function movePointerOffBoard(page, viewport) {
  const canvas = page.locator("canvas").first();
  const box = await canvas.boundingBox();
  const candidates = [
    { x: viewport.width - 2, y: viewport.height - 2 },
    { x: 2, y: viewport.height - 2 },
    { x: viewport.width - 2, y: 2 },
    { x: 2, y: 2 },
  ];
  const target = candidates.find((point) => !box || point.x < box.x || point.x > box.x + box.width
    || point.y < box.y || point.y > box.y + box.height);
  if (!target) throw new Error("Could not place pointer outside the board.");
  await page.mouse.move(target.x, target.y);
}

/** @param {Page} page @param {string} family */
async function heading(page, family) {
  await page.getByRole("heading", { name: family, exact: true }).waitFor({ state: "visible" });
}

/**
 * @param {Page} page
 * @param {string} expected
 * @param {string} testId
 * @param {string} context
 */
async function waitForTestIdText(page, testId, expected, context = "capture readiness") {
  try {
    await page.waitForFunction(({ id, value }) => {
      const element = document.querySelector('[data-testid="' + id + '"]');
      return element?.textContent?.trim() === value;
    }, { id: testId, value: expected }, { timeout: 20_000 });
  } catch (error) {
    const observed = await readTestIdText(page, testId);
    throw new Error(context + " timed out waiting for " + testId + "=" + JSON.stringify(expected)
      + "; observed " + JSON.stringify(observed) + "; "
      + (error instanceof Error ? error.message : String(error)));
  }
}

/** @param {Page} page @param {string} testId */
async function readTestIdText(page, testId) {
  return page.locator('[data-testid="' + testId + '"]').evaluateAll((elements) =>
    elements.map((element) => element.textContent?.trim() ?? ""));
}

/**
 * Move the replay to its final ply using the visible native timeline control.
 * This is an ordinary keyboard interaction; no app state/storage is overridden.
 * @param {Page} page
 * @param {string} tag
 * @param {Record<string, any>} profile
 * @param {string} testId
 * @param {string} expected
 */
async function seekReplayToFinalMove(page, tag, profile, testId, expected) {
  const context = tag + "/" + profile.id + " Replay final ply";
  await heading(page, tag === "v0.2.4" ? "Replay" : "Replay Analysis");
  const timeline = page.locator('input[type="range"]');
  await timeline.first().waitFor({ state: "visible" });
  const timelineCount = await timeline.count();
  if (timelineCount !== 1) {
    throw new Error(context + " expected one visible replay timeline; found " + timelineCount + ".");
  }
  const before = await readTestIdText(page, testId);
  try {
    await timeline.focus();
    await timeline.press("End");
    await timeline.evaluate((element) => element.blur());
  } catch (error) {
    throw new Error(context + " could not seek the replay timeline to its end; initial "
      + testId + "=" + JSON.stringify(before) + "; "
      + (error instanceof Error ? error.message : String(error)));
  }
  await waitForTestIdText(page, testId, expected, context);
  return { method: "native replay timeline End key", from: before, to: expected };
}

/** @param {Page} page @param {string} expected @param {string} testId */
async function requireTestIdText(page, testId, expected) {
  const locator = page.getByTestId(testId);
  const actual = (await locator.textContent())?.trim() ?? "";
  if (actual !== expected) {
    throw new Error(testId + " expected " + JSON.stringify(expected) + " but got " + JSON.stringify(actual));
  }
}

/**
 * @param {Page} page
 * @param {string} testId
 * @param {string} expectedFragment
 * @param {number} [timeout]
 * @param {string} [context]
 * @returns {Promise<string>}
 */
async function waitForTestIdTextContains(
  page,
  testId,
  expectedFragment,
  timeout = 20_000,
  context = "capture readiness",
) {
  try {
    await page.waitForFunction(({ id, fragment }) => {
      const element = document.querySelector('[data-testid="' + id + '"]');
      return element?.textContent?.includes(fragment) ?? false;
    }, { id: testId, fragment: expectedFragment }, { timeout });
  } catch (error) {
    const observed = await readTestIdText(page, testId);
    throw new Error(context + " timed out waiting for " + testId + " to contain "
      + JSON.stringify(expectedFragment) + "; observed " + JSON.stringify(observed) + "; "
      + (error instanceof Error ? error.message : String(error)));
  }
  return (await page.getByTestId(testId).textContent())?.trim() ?? "";
}

/** @param {Page} page @param {string} testId @param {string} expected */
async function requireTestIdContainsText(page, testId, expected) {
  const actual = (await page.getByTestId(testId).textContent())?.trim() ?? "";
  if (!actual.includes(expected)) {
    throw new Error(testId + " did not contain " + JSON.stringify(expected) + ".");
  }
}

/**
 * @param {Page} page
 * @param {string} tag
 * @param {string} screen
 * @param {string} route
 * @param {string} headingText
 * @param {string | null} [testId]
 * @param {string | null} [expectedText]
 */
async function assertScreenReady(page, tag, screen, route, headingText, testId = null, expectedText = null) {
  await heading(page, headingText);
  if (new URL(page.url()).pathname !== route) {
    throw new Error(tag + " " + screen + " expected route " + route + ".");
  }
  if (testId && expectedText) {
    await waitForTestIdText(page, testId, expectedText, tag + "/" + screen + " screen readiness");
  }
  const fonts = await waitForFonts(page);
  let board = null;
  if (screen === "match" || screen === "replay") {
    board = await waitForBoard(page);
    await movePointerOffBoard(page, page.viewportSize() ?? { width: 1280, height: 900 });
  }
  await page.evaluate(() => new Promise((resolveFrame) => {
    requestAnimationFrame(() => requestAnimationFrame(resolveFrame));
  }));
  return {
    route: new URL(page.url()).pathname,
    heading: headingText,
    fonts,
    board,
    reducedMotion: await page.evaluate(() => matchMedia("(prefers-reduced-motion: reduce)").matches),
  };
}

/**
 * @param {Page} page
 * @param {string} outputPath
 * @param {string} relativePath
 * @param {Record<string, any>} metadata
 */
async function saveScreenshot(page, outputPath, relativePath, metadata) {
  const absolutePath = join(outputPath, relativePath);
  await mkdir(dirname(absolutePath), { recursive: true });
  const bytes = await page.screenshot({
    path: absolutePath,
    type: "png",
    scale: "css",
    fullPage: false,
    animations: "disabled",
  });
  const dimensions = pngDimensions(bytes);
  const viewport = page.viewportSize();
  if (!viewport || dimensions.width !== viewport.width || dimensions.height !== viewport.height) {
    throw new Error("Screenshot dimensions did not match the CSS viewport for " + relativePath + ".");
  }
  return {
    ...metadata,
    file: relativePath,
    sha256: sha256(bytes),
    width: dimensions.width,
    height: dimensions.height,
    capturedAt: new Date().toISOString(),
  };
}

/** @param {Buffer | Uint8Array} bytes */
function pngDimensions(bytes) {
  const buffer = Buffer.from(bytes);
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  if (buffer.length < 24 || !buffer.subarray(0, 8).equals(signature) || buffer.toString("ascii", 12, 16) !== "IHDR") {
    throw new Error("Screenshot output was not a valid PNG.");
  }
  return { width: buffer.readUInt32BE(16), height: buffer.readUInt32BE(20) };
}

/** @param {string} outputPath @param {Record<string, any>} manifest */
async function writeManifest(outputPath, manifest) {
  await writeFile(join(outputPath, "capture.json"), JSON.stringify(manifest, null, 2) + "\n");
}

/**
 * @param {Record<string, any>} source
 * @param {Record<string, any>} inspected
 * @returns {Promise<void>}
 */
async function assertSourceStillClean(source, inspected) {
  const head = (await git(inspected.worktreeRoot, ["rev-parse", "HEAD"])).toLowerCase();
  const dirty = await git(inspected.worktreeRoot, ["status", "--porcelain", "--untracked-files=all"]);
  if (head !== source.expectedCommit) {
    throw new Error(source.tag + " source commit changed during capture.");
  }
  if (dirty !== "") {
    throw new Error(source.tag + " source worktree changed during capture:\n" + dirty);
  }
}

/**
 * @param {Record<string, any>} inputConfig
 * @param {string} configPath
 * @param {string} outputArgument
 */
async function preflight(inputConfig, configPath, outputArgument) {
  const config = validateConfigShape(inputConfig);
  const realConfigPath = await realpath(configPath);
  const sources = [];
  for (const source of config.sources) sources.push(await inspectSource(source));
  const protectedPaths = [
    repositoryRoot,
    realConfigPath,
    ...sources.map((source) => source.worktreeRoot),
  ];
  const outputPath = await validateOutputPath(outputArgument, protectedPaths);
  return { config, sources, outputPath };
}

/**
 * @param {string} configPath
 * @param {string} outputArgument
 * @param {boolean} validateOnly
 */
async function main(configPath, outputArgument, validateOnly) {
  const absoluteConfig = resolve(configPath);
  const rawConfig = JSON.parse(await readFile(absoluteConfig, "utf8"));
  const { config, sources, outputPath } = await preflight(rawConfig, absoluteConfig, outputArgument);
  if (validateOnly) {
    process.stdout.write(JSON.stringify({
      ok: true,
      outputDirectoryIsNew: true,
      sources: sources.map((source) => {
        if (typeof source.commit !== "string" || source.commit.length !== 40) {
          throw new Error(source.tag + " validate-only output is missing a source commit.");
        }
        return {
          tag: source.tag,
          commit: source.commit,
          sourceTree: source.sourceTree,
          applicationTree: source.applicationTree,
          distSha256: source.distSha256,
          preview: source.preview,
        };
      }),
    }, null, 2) + "\n");
    return;
  }

  const playwrightPackage = JSON.parse(await readFile(
    join(webRoot, "node_modules", "@playwright", "test", "package.json"),
    "utf8",
  ));
  if (typeof playwrightPackage.version !== "string") {
    throw new Error("Could not record the installed @playwright/test version.");
  }
  const captureScriptSha256 = await sha256File(fileURLToPath(import.meta.url));

  await mkdir(outputPath);
  const startedAt = new Date().toISOString();
  /** @type {Record<string, any>} */
  const manifest = {
    schemaVersion: 1,
    status: "running",
    captureType: "retrospective-ui",
    startedAt,
    finishedAt: null,
    captureTool: {
      node: process.version,
      script: "scripts/capture_retrospective_ui.mjs",
      scriptSha256: captureScriptSha256,
      playwrightPackage: "@playwright/test",
      playwrightVersion: playwrightPackage.version,
      browserName: "chromium",
      browserVersion: null,
    },
    contract: {
      profiles: PROFILES.map((profile) => ({
        id: profile.id,
        viewport: profile.viewport,
        deviceScaleFactor: profile.deviceScaleFactor,
        isMobile: profile.isMobile,
        hasTouch: profile.hasTouch,
        userAgentKind: profile.userAgentKind,
      })),
      locale: "en-US",
      timezoneId: "UTC",
      textScale: "100% browser default; no text zoom applied",
      reducedMotion: "reduce",
      screenshotScale: "css",
      fullPage: false,
      normalScenes: ["home", "match", "replay", "profile"],
      limitations: [
        "These are newly recreated retrospective layouts, not original-period browser captures.",
        "Portrait is a generic Android touch viewport, not a measured OnePlus/browser-chrome profile.",
        "Automated readiness does not establish human or physical-device acceptance.",
        "Phaser canvas animation is not guaranteed to freeze; board captures record the observed frame.",
        "Phaser renderer is recorded as unknown unless safely exposed by the page.",
      ],
    },
    fixture: {
      profileId: FIXTURE.profileId,
      displayName: FIXTURE.displayName,
      matchId: FIXTURE.matchId,
      savedAt: FIXTURE.savedAt,
      ruleset: FIXTURE.variant,
      moveCells: [...FIXTURE.moveCells],
      winningMoves: FIXTURE.winningCells,
      result: "black_won",
      matchPosition: { ply: 8, sideToMove: "black", human: "black", status: "playing" },
      replayPosition: { ply: 9, status: "black_won" },
      profileAdapters: Object.fromEntries(sources.map((source) => [source.tag, source.adapter])),
      storageAdapters: {
        v0_2_4: {
          key: "gomoku2d.guest-profile.v1",
          storageVersion: 0,
          historyPath: "state.history",
          moveEncoding: "array of {col,row,moveNumber,player}",
        },
        v0_6_2: {
          key: "gomoku2d.local-profile.v5",
          storageVersion: 5,
          historyPath: "state.matchHistory.replayMatches",
          matchSchemaVersion: 2,
          moveEncoding: "flat move_cells array",
        },
      },
      identityModel: {
        v0_2_4: "guest profile; Practice Bot opponent",
        v0_6_2: "local profile; Normal Bot opponent; cloud deliberately unconfigured",
      },
    },
    sources: sources.map((source) => ({
      tag: source.tag,
      adapter: source.adapter,
      commit: source.commit,
      sourceTree: source.sourceTree,
      applicationTree: source.applicationTree,
      packageVersion: source.packageVersion,
      packageLockSha256: source.packageLockSha256,
      cargoLockSha256: source.cargoLockSha256,
      wasmSha256: source.wasmSha256,
      distSha256: source.distSha256,
      indexSha256: source.indexSha256,
      buildReceipt: source.buildReceipt,
      preview: source.preview,
      applicationInputsCleanBefore: true,
      applicationInputsCleanAfter: null,
    })),
    contexts: [],
    captures: [],
    network: [],
    errors: [],
    failures: [],
  };
  let browser = null;
  /** @type {BrowserContext[]} */
  const contexts = [];
  /** @type {Error[]} */
  const failures = [];
  await writeManifest(outputPath, manifest);
  try {
    browser = await chromium.launch({ headless: true });
    manifest.captureTool.browserVersion = browser.version();
    await writeManifest(outputPath, manifest);
    for (const source of sources) {
      for (const profile of PROFILES) {
        const context = await browser.newContext({
          viewport: profile.viewport,
          deviceScaleFactor: profile.deviceScaleFactor,
          isMobile: profile.isMobile,
          hasTouch: profile.hasTouch,
          userAgent: profile.isMobile ? genericAndroidUserAgent(browser.version()) : undefined,
          locale: "en-US",
          timezoneId: "UTC",
          colorScheme: "light",
          reducedMotion: "reduce",
          serviceWorkers: "block",
        });
        contexts.push(context);
        /** @type {Record<string, any>} */
        const contextRecord = {
          source: source.tag,
          profile: profile.id,
          contextId: source.tag + ":" + profile.id,
          viewport: profile.viewport,
          deviceScaleFactor: profile.deviceScaleFactor,
          isMobile: profile.isMobile,
          hasTouch: profile.hasTouch,
          userAgentKind: profile.userAgentKind,
          userAgent: profile.isMobile ? genericAndroidUserAgent(browser.version()) : null,
          locale: "en-US",
          timezoneId: "UTC",
          textScale: "100% browser default; no text zoom applied",
          reducedMotion: "reduce",
          screenshotScale: "css",
          fullPage: false,
          browserVersion: browser.version(),
          state: "setup",
        };
        manifest.contexts.push(contextRecord);
        const realObserver = installNetworkAndErrorObservers(context, source.preview.origin);
        await context.route("**/*", realObserver.intercept);
        await context.routeWebSocket("**/*", realObserver.interceptWebSocket);
        await installFixture(context, source.adapter);
        const page = await context.newPage();
        realObserver.watchPage(page);
        page.setDefaultTimeout(20_000);
        contextRecord.userAgent = await page.evaluate(() => navigator.userAgent);
        contextRecord.state = "running";
        let captureError = null;
        try {
          await captureContext(page, source, profile, outputPath, manifest);
        } catch (error) {
          const message = error instanceof Error ? error.message : String(error);
          const observedTestIds = await page.locator("[data-testid]").evaluateAll((elements) =>
            elements.slice(0, 40).map((element) => ({
              id: element.getAttribute("data-testid"),
              text: (element.textContent ?? "").trim().slice(0, 200),
            }))).catch(() => []);
          captureError = new Error(source.tag + "/" + profile.id + " scene "
            + new URL(page.url()).pathname + ": " + message
            + "; observed test IDs " + JSON.stringify(observedTestIds));
        } finally {
          const summary = {
            source: source.tag,
            profile: profile.id,
            allowedSameOriginRequests: realObserver.allowedRequestCount(),
            blockedRequests: [...realObserver.blockedRequests],
            failedRequests: [...realObserver.failedRequests],
            consoleErrors: [...realObserver.consoleErrors],
            pageErrors: [...realObserver.pageErrors],
          };
          manifest.network.push(summary);
          manifest.errors.push({
            source: source.tag,
            profile: profile.id,
            console: summary.consoleErrors,
            page: summary.pageErrors,
          });
          contextRecord.network = {
            allowedSameOriginRequests: summary.allowedSameOriginRequests,
            blockedRequests: summary.blockedRequests.length,
          };
          contextRecord.state = captureError ? "failed" : "complete";
          if (summary.blockedRequests.length > 0) {
            failures.push(new Error(source.tag + "/" + profile.id + " network policy: forbidden requests were blocked."));
          }
          if (summary.pageErrors.length > 0) {
            failures.push(new Error(source.tag + "/" + profile.id + " page error observer recorded browser errors."));
          }
          try {
            await context.close();
          } finally {
            const contextIndex = contexts.indexOf(context);
            if (contextIndex >= 0) contexts.splice(contextIndex, 1);
            await writeManifest(outputPath, manifest);
          }
        }
        if (captureError) throw captureError;
        const networkSummary = manifest.network[manifest.network.length - 1];
        if (networkSummary.blockedRequests.length > 0 || networkSummary.pageErrors.length > 0) {
          throw new Error(source.tag + "/" + profile.id + " network/page observer reported capture failures.");
        }
      }
    }
  } catch (error) {
    failures.push(error instanceof Error ? error : new Error(String(error)));
  } finally {
    for (const context of contexts) {
      try {
        await context.close();
      } catch (error) {
        failures.push(new Error("Browser context cleanup failed: " + String(error)));
      }
    }
    if (browser) {
      try {
        await browser.close();
      } catch (error) {
        failures.push(new Error("Browser cleanup failed: " + String(error)));
      }
    }
    for (let index = 0; index < sources.length; index += 1) {
      try {
        const originalSource = config.sources.find((source) => source.tag === sources[index].tag);
        if (!originalSource) throw new Error("Missing config entry for " + sources[index].tag + ".");
        await assertSourceStillClean(originalSource, sources[index]);
        manifest.sources[index].applicationInputsCleanAfter = true;
      } catch (error) {
        manifest.sources[index].applicationInputsCleanAfter = false;
        failures.push(error instanceof Error ? error : new Error(String(error)));
      }
    }
  }
  manifest.finishedAt = new Date().toISOString();
  manifest.status = failures.length === 0 ? "complete" : "failed";
  manifest.failures = failures.map((error) => redactText(error.message));
  await writeManifest(outputPath, manifest);
  if (failures.length > 0) {
    throw new Error(
      "Retrospective capture failed after preserving partial output at "
        + outputPath + ":\n- " + manifest.failures.join("\n- "),
    );
  }
  /** @type {Array<Record<string, any>>} */
  const captures = manifest.captures;
  process.stdout.write(JSON.stringify({
    status: manifest.status,
    outputDirectory: outputPath,
    captures: captures.map((capture) => capture.file),
  }, null, 2) + "\n");
}

/**
 * @param {Page} page
 * @param {Record<string, any>} source
 * @param {Record<string, any>} profile
 * @param {string} outputPath
 * @param {Record<string, any>} manifest
 */
async function captureContext(page, source, profile, outputPath, manifest) {
  // The page/context were created with a pre-bootstrap localStorage fixture and
  // request interception. Keep the existing context here so the fixture is stable.
  const adapter = source.adapter === "v024"
    ? {
      replayPath: "/replays/local/" + FIXTURE.matchId,
      replayHeading: "Replay",
      botText: "Practice Bot",
      profileInput: "Display name",
    }
    : {
      replayPath: "/replay/" + FIXTURE.matchId,
      replayHeading: "Replay Analysis",
      botText: "Normal Bot",
      profileInput: "Name",
    };
  /** @param {string} route */
  const path = (route) => new URL(route.replace(/^\/+/, ""), source.baseUrl).pathname;
  const baseCapture = {
    source: source.tag,
    profile: profile.id,
    contextId: source.tag + ":" + profile.id,
  };

  await navigate(page, source.baseUrl, "/");
  /** @type {Record<string, any>} */
  let readiness = await assertScreenReady(page, source.tag, "home", path("/"), "Gomoku2D");
  manifest.captures.push(await saveScreenshot(page, outputPath,
    source.tag + "/" + profile.id + "/home.png",
    { ...baseCapture, screen: "home", route: readiness.route, readiness }));

  await navigate(page, source.baseUrl, "/profile");
  readiness = await assertScreenReady(page, source.tag, "profile", path("/profile"), "Profile");
  const nameInput = page.getByLabel(adapter.profileInput, { exact: true });
  await nameInput.waitFor({ state: "attached" });
  const localName = await nameInput.inputValue();
  if (localName !== FIXTURE.displayName) {
    throw new Error(source.tag + " Profile did not load the synthetic local identity.");
  }
  await page.getByText("Win", { exact: true }).waitFor({ state: "visible" });
  const profileText = ((await page.locator("body").textContent()) ?? "").toLowerCase();
  if (!profileText.includes(adapter.botText.toLowerCase())) {
    throw new Error(source.tag + " Profile did not show the seeded match history.");
  }
  readiness = { ...readiness, identity: localName, nonemptyHistory: true, opponent: adapter.botText };
  if (source.tag === "v0.6.2") {
    const cloudStatus = page.getByText("Cloud sync unavailable.", { exact: true });
    await cloudStatus.first().waitFor({ state: "attached" });
    const signIn = page.locator('button[aria-label="Sign in"]');
    await signIn.first().waitFor({ state: "attached" });
    const signInDisabled = await signIn.evaluateAll((buttons) =>
      buttons.length > 0 && buttons.every((button) => button instanceof HTMLButtonElement && button.disabled));
    if (!signInDisabled) {
      throw new Error("v0.6.2 cloud sign-in was enabled in the unconfigured local guest capture.");
    }
    const controls = page.locator("details").filter({ hasText: "Profile controls" });
    const profileControlsOpen = await controls.count() > 0
      ? await controls.first().evaluate((element) => element instanceof HTMLDetailsElement && element.open)
      : null;
    if (profile.isMobile && profileControlsOpen !== false) {
      throw new Error("v0.6.2 compact Profile controls were not in their default closed state.");
    }
    readiness = {
      ...readiness,
      cloudSync: "unavailable",
      cloudStatusVisible: await cloudStatus.first().isVisible(),
      signInDisabled,
      profileControlsOpen,
    };
  }
  manifest.captures.push(await saveScreenshot(page, outputPath,
    source.tag + "/" + profile.id + "/profile.png",
    { ...baseCapture, screen: "profile", route: readiness.route, readiness }));

  await navigate(page, source.baseUrl, adapter.replayPath);
  const replayFinalPlySeek = await seekReplayToFinalMove(
    page,
    source.tag,
    profile,
    "replay-move-count",
    "Move 9 / 9",
  );
  readiness = await assertScreenReady(
    page, source.tag, "replay", path(adapter.replayPath), adapter.replayHeading,
    "replay-move-count", "Move 9 / 9",
  );
  readiness = { ...readiness, finalPlySeek: replayFinalPlySeek };
  if (source.tag === "v0.6.2") {
    const analysisStatus = await waitForTestIdTextContains(
      page,
      "replay-analysis-status",
      "Black has won",
      60_000,
      source.tag + "/" + profile.id + " Replay analysis settle",
    );
    await page.evaluate(() => new Promise((resolveFrame) => {
      requestAnimationFrame(() => requestAnimationFrame(resolveFrame));
    }));
    readiness = { ...readiness, analysisStatus, analysisSettled: true };
  }
  await requireTestIdContainsText(page, "replay-player-row-black", FIXTURE.displayName);
  const replayBotRow = (await page.getByTestId("replay-player-row-white").textContent()) ?? "";
  if (!replayBotRow.includes(adapter.botText)) {
    throw new Error(source.tag + " Replay did not show the tag's default bot identity.");
  }
  readiness = {
    ...readiness,
    ply: 9,
    moveCount: "Move 9 / 9",
    result: "black_won",
    blackPlayer: FIXTURE.displayName,
    whitePlayer: adapter.botText,
  };
  manifest.captures.push(await saveScreenshot(page, outputPath,
    source.tag + "/" + profile.id + "/replay.png",
    { ...baseCapture, screen: "replay", route: readiness.route, readiness }));

  await page.getByRole("button", { name: "Previous move", exact: true }).click();
  await waitForTestIdText(
    page,
    "replay-move-count",
    "Move 8 / 9",
    source.tag + "/" + profile.id + " Replay Previous move to ply 8",
  );
  const playFromHere = page.getByRole("button", { name: "Play From Here", exact: true });
  if (!await playFromHere.isEnabled()) {
    throw new Error(source.tag + " does not allow branching from ply 8.");
  }
  await playFromHere.click();
  readiness = await assertScreenReady(
    page,
    source.tag,
    "match",
    path("/match/local"),
    "Local Match",
    "match-status",
    FIXTURE.displayName + " to move",
  );
  await requireTestIdText(page, "match-move-count", "Move 8");
  const blackRow = (await page.getByTestId("player-row-black").textContent()) ?? "";
  const whiteRow = (await page.getByTestId("player-row-white").textContent()) ?? "";
  if (!blackRow.includes(FIXTURE.displayName) || !whiteRow.includes(adapter.botText)) {
    throw new Error(source.tag + " branched Match player identities do not match the fixture.");
  }
  const activeHumanStatus = (await page.getByTestId("match-status").textContent())?.trim() ?? "";
  if (activeHumanStatus !== FIXTURE.displayName + " to move") {
    throw new Error(source.tag + " branched Match is not idle with Black/human to move.");
  }
  const touchQueryMatches = await page.evaluate((query) => matchMedia(query).matches, MOBILE_QUERY);
  if (touchQueryMatches !== profile.isMobile) {
    throw new Error(source.tag + " input profile did not match the game's touch media query.");
  }
  readiness = {
    ...readiness,
    ply: 8,
    moveCount: "Move 8",
    status: activeHumanStatus,
    activePlayer: FIXTURE.displayName,
    opponent: adapter.botText,
    compactTouchQueryMatches: touchQueryMatches,
  };
  manifest.captures.push(await saveScreenshot(page, outputPath,
    source.tag + "/" + profile.id + "/match.png",
    { ...baseCapture, screen: "match", route: readiness.route, readiness }));
}

/** @param {string} value */
function isDirectInvocation(value) {
  if (!value) return false;
  return pathToFileURL(resolve(value)).href === import.meta.url;
}

if (isDirectInvocation(process.argv[1])) {
  try {
    const parsed = parseArguments(process.argv.slice(2));
    if (parsed.help) {
      process.stdout.write(usage() + "\n");
    } else if (parsed.configPath && parsed.outputPath) {
      await main(parsed.configPath, parsed.outputPath, parsed.validateOnly);
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(message + "\n");
    process.exitCode = 1;
  }
}
