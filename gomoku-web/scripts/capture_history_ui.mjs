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
  parse,
  relative,
  resolve,
  sep,
} from "node:path";
import { fileURLToPath } from "node:url";
import {
  CAPTURE_PROFILES,
  FIXTURES,
  localStorageSeed,
  routeForScene,
  validateHistoryCaptureConfig,
} from "./history_capture_adapters.mjs";

/** @typedef {import("@playwright/test").Browser} Browser */
/** @typedef {import("@playwright/test").BrowserContext} BrowserContext */
/** @typedef {import("@playwright/test").Page} Page */
/** @typedef {import("@playwright/test").Route} Route */

const executeFile = promisify(execFile);
const scriptPath = fileURLToPath(import.meta.url);
const webRoot = resolve(dirname(scriptPath), "..");
const repositoryRoot = resolve(webRoot, "..");
const MANIFEST_NAME = "capture.json";
export const PROFILE_NAME_LABEL_PATTERN = /^\s*(?:Name|Display name)\s*$/i;
export const PROFILE_NAME_INPUT_SELECTOR = 'input[aria-label="Name"], input[placeholder="Name"], input[placeholder="Display name"]';
export const SETTINGS_CUSTOM_BUTTON_NAME = /^Custom\b/;
const TRANSIENT_REPLAY_ANALYSIS_STATUS = /(?:loading|analy[sz]ing|tracing|queued|starting)/i;

/** @param {string} value */
export function isSettledReplayAnalysisStatus(value) {
  const text = value.trim();
  return text.length > 0 && !TRANSIENT_REPLAY_ANALYSIS_STATUS.test(text);
}
const ALLOWED_TRANSPORT_EXPORTS = Object.freeze([
  "validateHistoryTransport",
  "historyTransportLaunchOptions",
  "historyTransportContextOptions",
  "prepareHistoryContext",
  "isHistoryTransportURL",
  "isOptionalHistoryBootstrapURL",
]);

export function usage() {
  return [
    "Usage:",
    "  node scripts/capture_history_ui.mjs --config <source.json> --output <new-directory>",
    "  node scripts/capture_history_ui.mjs --config <source.json> --output <new-directory> --validate-only",
    "",
    "Config is one prepared exact-tag source plus explicit screen/state scenes.",
    "The output must be a new directory outside the checkout, source worktree, and build root.",
    "Use a prepared loopback preview; this command never builds, installs, or starts a service.",
    "See docs/working/v0_6_3_history_inventory.md for the config example and scene list.",
  ].join("\n");
}

/** @param {string[]} args @returns {{validateOnly:boolean, help:boolean, configPath?:string, outputPath?:string}} */
export function parseArguments(args) {
  /** @type {{validateOnly:boolean, help:boolean, configPath?:string, outputPath?:string}} */
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
    if (argument !== "--config" && argument !== "--output") throw new Error("Unknown argument: " + argument);
    const value = args[index + 1];
    if (!value || value.startsWith("--")) throw new Error("Missing value after " + argument);
    index += 1;
    const field = argument === "--config" ? "configPath" : "outputPath";
    if (parsed[field] !== undefined) throw new Error(argument + " may be supplied only once.");
    parsed[field] = value;
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

/** @param {Buffer | Uint8Array} bytes */
function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

/** @param {string} path */
export async function sha256File(path) {
  return sha256(await readFile(path));
}

/** @param {string} left @param {string} right */
function lexicalCompare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

/** @param {Array<{path:string,sha256:string}>} files */
export function digestFileInventory(files) {
  const sorted = [...files].sort((left, right) => lexicalCompare(left.path, right.path));
  return sha256(Buffer.from(JSON.stringify(sorted) + "\n", "utf8"));
}

/** @param {string} candidate @param {string} root */
function pathOverlaps(candidate, root) {
  /** @param {string} child @param {string} parent */
  const within = (child, parent) => {
    const relation = relative(parent, child);
    return relation === "" || (!relation.startsWith(".." + sep) && relation !== ".." && !isAbsolute(relation));
  };
  const resolvedCandidate = resolve(candidate);
  const resolvedRoot = resolve(root);
  return within(resolvedCandidate, resolvedRoot) || within(resolvedRoot, resolvedCandidate);
}

/** @param {string} path */
async function rejectSymlinkComponents(path) {
  const absolute = resolve(path);
  const parsed = parse(absolute);
  const segments = absolute.slice(parsed.root.length).split(sep).filter(Boolean);
  let current = parsed.root;
  for (const segment of segments) {
    current = join(current, segment);
    try {
      const info = await lstat(current);
      if (info.isSymbolicLink()) throw new Error("Output path contains a symlink component: " + current);
    } catch (error) {
      if (isRecord(error) && error.code === "ENOENT") break;
      throw error;
    }
  }
}

/** Resolve and guard a new output path without creating it. */
/** @param {string} outputArgument @param {string[]} protectedPaths */
export async function validateOutputPath(outputArgument, protectedPaths) {
  const requested = resolve(outputArgument);
  await rejectSymlinkComponents(requested);
  const parentPath = dirname(requested);
  let parentReal;
  try {
    parentReal = await realpath(parentPath);
  } catch {
    throw new Error("Output parent must already exist: " + parentPath);
  }
  const parentInfo = await lstat(parentReal);
  if (!parentInfo.isDirectory() || parentInfo.isSymbolicLink()) throw new Error("Output parent must be a real directory.");
  const outputPath = join(parentReal, basename(requested));
  try {
    await lstat(outputPath);
    throw new Error("Output already exists; choose a new directory: " + outputPath);
  } catch (error) {
    if (error instanceof Error && error.message.startsWith("Output already exists")) throw error;
    if (!isRecord(error) || error.code !== "ENOENT") throw error;
  }
  for (const protectedPath of protectedPaths) {
    let protectedReal;
    try { protectedReal = await realpath(protectedPath); }
    catch { protectedReal = resolve(protectedPath); }
    if (pathOverlaps(outputPath, protectedReal)) {
      throw new Error("Output must be disjoint from protected source/build root: " + protectedReal);
    }
  }
  return outputPath;
}

/** @param {string} root @param {string[]} args */
async function git(root, args) {
  const result = await executeFile("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  });
  return String(result.stdout).trim();
}

/** @param {string} path @param {string} label */
async function requireFile(path, label) {
  const info = await lstat(path);
  if (!info.isFile() || info.isSymbolicLink()) throw new Error(label + " must be a regular file.");
}

/** @param {string} root @returns {Promise<Array<{path:string,sha256:string}>>} */
async function fileInventory(root) {
  /** @type {Array<{path:string,sha256:string}>} */
  const files = [];
  /** @param {string} directory @param {string} prefix */
  async function visit(directory, prefix) {
    const entries = await readdir(directory, { withFileTypes: true });
    entries.sort((left, right) => lexicalCompare(left.name, right.name));
    for (const entry of entries) {
      const absolute = join(directory, entry.name);
      const path = prefix ? prefix + "/" + entry.name : entry.name;
      if (entry.isSymbolicLink()) throw new Error("Build tree contains a symlink: " + path);
      if (entry.isDirectory()) await visit(absolute, path);
      else if (entry.isFile()) files.push({ path, sha256: await sha256File(absolute) });
      else throw new Error("Build tree contains an unsupported file: " + path);
    }
  }
  await visit(root, "");
  files.sort((left, right) => lexicalCompare(left.path, right.path));
  return files;
}

/** @param {unknown} value @param {string} label */
function requireHash(value, label) {
  if (typeof value !== "string" || !/^[a-f0-9]{64}$/i.test(value)) {
    throw new Error(label + " must be a 64-character SHA-256 hash.");
  }
  return value.toLowerCase();
}

/** @param {unknown} value @param {string} label */
function requireTreeOid(value, label) {
  if (typeof value !== "string" || !/^[a-f0-9]{40}$/i.test(value)) {
    throw new Error(label + " must be a 40-character Git tree OID.");
  }
  return value.toLowerCase();
}

/** @param {Record<string, any>} source @param {Record<string, any>} inspected */
function sanitizedManifestSource(source, inspected) {
  const receipt = source.buildReceipt;
  return {
    tag: source.tag,
    commit: inspected.commit,
    sourceTree: inspected.sourceTree,
    applicationTree: inspected.applicationTree,
    adapter: source.adapter,
    packageVersion: inspected.packageVersion,
    packageLockSha256: inspected.packageLockSha256,
    cargoLock: receipt.cargoLock,
    wasmSha256: inspected.wasmSha256,
    distSha256: inspected.distSha256,
    indexSha256: inspected.indexSha256,
    reportInputs: inspected.reportInputs,
    buildReceipt: {
      recordedAt: receipt.recordedAt,
      environment: sanitizeEnvironment(receipt.environment),
      assumption: receipt.assumption ?? null,
    },
    previewOrigin: source.origin,
  };
}

/** @param {Record<string, any>} environment @returns {Record<string, any>} */
function sanitizeEnvironment(environment) {
  /** @type {Record<string, any>} */
  const safe = {};
  for (const [key, value] of Object.entries(environment)) {
    if (/api[_-]?key|secret|password|token|credential/i.test(key)) {
      safe[key] = "[redacted]";
      continue;
    }
    if (key === "firebaseEnvironment" && isRecord(value)) {
      safe.firebaseEnvironment = Object.fromEntries(Object.entries(value).map(([name, entry]) => [
        name,
        /api[_-]?key|secret|password|token|credential/i.test(name) ? "[synthetic key redacted]" : entry,
      ]));
      continue;
    }
    safe[key] = typeof value === "string" ? redactText(value) : value;
  }
  return safe;
}

/** @param {any} config */
async function inspectSource(config) {
  const source = config.source;
  const root = await realpath(source.worktree);
  if (pathOverlaps(root, repositoryRoot)) throw new Error(source.tag + " source worktree must be separate from this checkout.");
  const gitRoot = await realpath(await git(root, ["rev-parse", "--show-toplevel"]));
  if (gitRoot !== root) throw new Error(source.tag + " worktree path must be the Git repository root.");
  const commit = (await git(root, ["rev-parse", "HEAD"])).toLowerCase();
  if (commit !== source.expectedCommit) throw new Error(source.tag + " HEAD mismatch: expected " + source.expectedCommit + ", got " + commit);
  const taggedCommit = (await git(root, ["rev-parse", source.tag + "^{commit}"])).toLowerCase();
  if (taggedCommit !== commit) throw new Error(source.tag + " tag does not resolve to configured expectedCommit.");
  const branch = await git(root, ["branch", "--show-current"]);
  if (branch !== "") throw new Error(source.tag + " worktree must be detached.");
  const dirty = await git(root, ["status", "--porcelain", "--untracked-files=all"]);
  if (dirty) throw new Error(source.tag + " worktree is not clean:\n" + dirty);

  const sourceTree = (await git(root, ["rev-parse", "HEAD^{tree}"])).toLowerCase();
  const applicationTree = (await git(root, ["rev-parse", "HEAD:gomoku-web/src"])).toLowerCase();
  if (sourceTree !== requireTreeOid(source.buildReceipt.sourceTree, "buildReceipt.sourceTree")) {
    throw new Error(source.tag + " source tree does not match its build receipt.");
  }
  if (applicationTree !== requireTreeOid(source.buildReceipt.applicationTree, "buildReceipt.applicationTree")) {
    throw new Error(source.tag + " application tree does not match its build receipt.");
  }
  const packagePath = join(root, "gomoku-web", "package.json");
  const packageLockPath = join(root, "gomoku-web", "package-lock.json");
  const cargoLockPath = join(root, "gomoku-bot-lab", "Cargo.lock");
  const wasmPath = join(root, "gomoku-bot-lab", "gomoku-wasm", "pkg", "gomoku_wasm_bg.wasm");
  const distPath = join(root, "gomoku-web", "dist");
  const indexPath = join(distPath, "index.html");
  await Promise.all([
    requireFile(packagePath, "Tagged package.json"),
    requireFile(packageLockPath, "Tagged package-lock.json"),
    requireFile(wasmPath, "Tag-local generated Wasm package"),
    requireFile(indexPath, "Prepared dist/index.html"),
  ]);

  const packageTracked = await git(root, ["ls-files", "--error-unmatch", "gomoku-web/package-lock.json"]);
  if (packageTracked !== "gomoku-web/package-lock.json") throw new Error(source.tag + " package-lock.json is not tracked by the tag.");
  const cargoTracked = await git(root, ["ls-files", "--", "gomoku-bot-lab/Cargo.lock"]);
  const cargoExists = await lstat(cargoLockPath).then(() => true, (error) => {
    if (isRecord(error) && error.code === "ENOENT") return false;
    throw error;
  });
  const cargoLock = source.buildReceipt.cargoLock;
  if (cargoLock.kind === "tracked") {
    if (cargoTracked !== "gomoku-bot-lab/Cargo.lock" || !cargoExists) throw new Error(source.tag + " tracked Cargo.lock is missing or not tracked.");
    if (await sha256File(cargoLockPath) !== requireHash(cargoLock.sha256, "buildReceipt.cargoLock.sha256")) {
      throw new Error(source.tag + " tracked Cargo.lock hash does not match its build receipt.");
    }
  } else if (cargoTracked || cargoExists) {
    throw new Error(source.tag + " generated-external Cargo.lock must be absent from the clean historical worktree.");
  }

  const packageJson = JSON.parse(await readFile(packagePath, "utf8"));
  if (source.buildReceipt.packageVersion !== undefined && source.buildReceipt.packageVersion !== packageJson.version) {
    throw new Error(source.tag + " packageVersion receipt mismatch.");
  }
  const packageLockSha256 = await sha256File(packageLockPath);
  if (packageLockSha256 !== requireHash(source.buildReceipt.packageLockSha256, "buildReceipt.packageLockSha256")) {
    throw new Error(source.tag + " package-lock.json hash does not match its build receipt.");
  }
  const wasmSha256 = await sha256File(wasmPath);
  if (wasmSha256 !== requireHash(source.buildReceipt.wasmSha256, "buildReceipt.wasmSha256")) {
    throw new Error(source.tag + " generated Wasm hash does not match its build receipt.");
  }
  const files = await fileInventory(distPath);
  const receiptFiles = source.buildReceipt.files;
  if (JSON.stringify(files) !== JSON.stringify(receiptFiles)) throw new Error(source.tag + " prepared dist file hashes differ from the build receipt.");
  const distSha256 = digestFileInventory(files);
  if (distSha256 !== requireHash(source.buildReceipt.distSha256, "buildReceipt.distSha256")) {
    throw new Error(source.tag + " prepared dist digest does not match its build receipt.");
  }
  const indexSha256 = await sha256File(indexPath);
  if (indexSha256 !== requireHash(source.buildReceipt.indexSha256, "buildReceipt.indexSha256")) {
    throw new Error(source.tag + " dist index hash does not match its build receipt.");
  }

  /** @type {Array<{path:string,sha256:string}>} */
  const reportInputs = [];
  for (const item of source.buildReceipt.reportInputs) {
    const path = item.path;
    const hash = requireHash(item.sha256, "buildReceipt.reportInputs[].sha256");
    if (config.expectedReportInputs.includes(path)) {
      const actual = await sha256File(join(root, path));
      if (actual !== hash) throw new Error(source.tag + " tagged report input hash mismatch: " + path);
      reportInputs.push({ path, sha256: actual });
    }
  }
  const missingReports = config.expectedReportInputs.filter((/** @type {string} */ path) => !reportInputs.some((item) => item.path === path));
  if (missingReports.length) throw new Error(source.tag + " report receipt is incomplete: " + missingReports.join(", "));

  return {
    root,
    commit,
    sourceTree,
    applicationTree,
    packageVersion: String(packageJson.version),
    packageLockSha256,
    cargoLock: { kind: cargoLock.kind, sha256: cargoLock.sha256 },
    wasmSha256,
    files,
    distSha256,
    indexSha256,
    reportInputs,
    indexPath,
    distPath,
    clean: true,
  };
}

/** @param {string} origin @param {string} path */
function assetUrl(origin, path) {
  return new URL("/" + path.split("/").map((segment) => encodeURIComponent(segment)).join("/"), origin).href;
}

/** @template T @param {T[]} items @param {number} concurrency @param {(item:T,index:number)=>Promise<void>} callback */
async function fetchWithLimit(items, concurrency, callback) {
  let next = 0;
  const workers = Array.from({ length: Math.min(concurrency, items.length) }, async () => {
    while (true) {
      const index = next;
      next += 1;
      if (index >= items.length) return;
      await callback(items[index], index);
    }
  });
  await Promise.all(workers);
}

/** @param {any} config @param {any} inspected */
async function verifyServedBytes(config, inspected) {
  await fetchWithLimit(inspected.files, 8, async (file) => {
    const response = await fetch(assetUrl(config.source.origin, file.path), {
      redirect: "manual",
      signal: AbortSignal.timeout(15_000),
    });
    if (!response.ok) throw new Error(config.source.tag + " preview returned HTTP " + response.status + " for " + file.path + ".");
    const actual = sha256(Buffer.from(await response.arrayBuffer()));
    if (actual !== file.sha256) throw new Error(config.source.tag + " served bytes differ from receipt for " + file.path + ".");
  });
  return { filesChecked: inspected.files.length, allMatchReceipt: true };
}

/** @param {any} config */
async function loadTransport(config) {
  if (!config.transport) return undefined;
  const moduleUrl = new URL("./history_capture_transport.mjs", import.meta.url);
  let transportModule;
  try {
    transportModule = await import(moduleUrl.href);
  } catch (error) {
    if (isRecord(error) && error.code === "ERR_MODULE_NOT_FOUND") {
      throw new Error("Synthetic transport was requested, but the primary-owned scripts/history_capture_transport.mjs is not available.");
    }
    throw error;
  }
  const missing = ALLOWED_TRANSPORT_EXPORTS.filter((name) => typeof transportModule[name] !== "function");
  if (missing.length) throw new Error("Synthetic transport module is missing required exports: " + missing.join(", "));
  const normalized = await transportModule.validateHistoryTransport(config.transport);
  if (!isRecord(normalized)) throw new Error("validateHistoryTransport must return the normalized transport object.");
  return { module: transportModule, normalized };
}

/** @param {string} url @param {string} method @param {string} allowedOrigin @param {boolean} [transportAllows] */
function requestPolicy(url, method, allowedOrigin, transportAllows = false) {
  let parsed;
  try { parsed = new URL(url); }
  catch { return { allowed: false, reason: "invalid-url" }; }
  if (parsed.protocol === "data:" || parsed.protocol === "blob:") return { allowed: true, reason: "inline" };
  if (transportAllows) return { allowed: true, reason: "synthetic-transport" };
  if (parsed.origin !== allowedOrigin) return { allowed: false, reason: "off-origin" };
  if (/(?:identitytoolkit\.googleapis\.com|securetoken\.googleapis\.com|firestore\.googleapis\.com|\/(?:api|auth|cloud|firebase|firestore|identity)(?:\/|$)|\/v1\/projects\/.*\/databases|\/__\/auth\/|\/oauth2\/|\/accounts:|\/token(?:\/|$))/i.test(parsed.host + parsed.pathname)) {
    return { allowed: false, reason: "identity-or-data-endpoint" };
  }
  if (method.toUpperCase() !== "GET" && method.toUpperCase() !== "HEAD") return { allowed: false, reason: "non-read-method" };
  return { allowed: true, reason: "tagged-preview-read" };
}

/** @param {string} url @param {string} method @param {string} allowedOrigin @param {boolean} [transportAllows] */
export function isAllowedRequest(url, method, allowedOrigin, transportAllows = false) {
  return requestPolicy(url, method, allowedOrigin, transportAllows).allowed;
}

/** @param {{url:string,method:string,resourceType:string,failure:string,responseStatus:number|null}} event @param {unknown} transportMode @param {boolean} transportAllows */
export function isExpectedFirestoreChannelCancellation(event, transportMode, transportAllows) {
  if (!event || transportMode !== "native-emulators" || transportAllows !== true
    || typeof event.method !== "string" || typeof event.resourceType !== "string"
    || typeof event.failure !== "string") return false;
  let parsed;
  try { parsed = new URL(event.url); }
  catch { return false; }
  return parsed.origin === "http://127.0.0.1:28081"
    && [
      "/google.firestore.v1.Firestore/Write/channel",
      "/google.firestore.v1.Firestore/Listen/channel",
    ].includes(parsed.pathname)
    && event.method.toUpperCase() === "GET"
    && event.resourceType === "fetch"
    && event.failure === "net::ERR_ABORTED"
    && event.responseStatus === 200;
}

/** @param {string} raw */
function safeUrl(raw) {
  try {
    const url = new URL(raw);
    url.search = "";
    url.hash = "";
    url.username = "";
    url.password = "";
    return url.href;
  } catch {
    return "[invalid-url]";
  }
}

/** @param {string} input */
export function redactText(input) {
  return String(input)
    .replace(/((?:api[_-]?key|access[_-]?token|refresh[_-]?token|id[_-]?token|authorization|password|secret|token)\s*[:=]\s*)([^\s&,;]+)/gi, "$1[redacted]")
    .replace(/(Bearer\s+)[A-Za-z0-9._~+/-]+=*/gi, "$1[redacted]")
    .replace(/(https?:\/\/[^\s?#]+)\?[^\s#]*/gi, "$1?[redacted]");
}

/** @param {unknown} error @param {string[]} [roots] */
function safeError(error, roots = []) {
  let message = error instanceof Error ? error.message : String(error);
  message = redactText(message);
  for (const root of roots.filter(Boolean)) message = message.split(String(root)).join("<protected-path>");
  return message.split("\n")[0].slice(0, 800);
}

/** @param {any} manifest */
function validateManifestForWrite(manifest) {
  if (!isRecord(manifest) || manifest.schemaVersion !== 1 || !Array.isArray(manifest.sources) || manifest.sources.length !== 1) {
    throw new Error("History capture manifest must contain schemaVersion 1 and exactly one source.");
  }
  if (!Array.isArray(manifest.captures)) throw new Error("History capture manifest captures must be an array.");
  const seen = new Set();
  for (const [index, capture] of manifest.captures.entries()) {
    if (!isRecord(capture) || typeof capture.screen !== "string" || typeof capture.state !== "string") {
      throw new Error("Manifest capture " + index + " needs separate screen and state IDs.");
    }
    if (capture.status !== "captured") throw new Error("Manifest capture " + index + " is not marked captured.");
    const key = [capture.profile?.id, capture.screen, capture.state].join("/");
    if (seen.has(key)) throw new Error("Manifest contains duplicate capture " + key + ".");
    seen.add(key);
    if (typeof capture.file !== "string" || capture.file.startsWith("/") || capture.file.split("/").includes("..")) {
      throw new Error("Manifest capture " + index + " has an unsafe screenshot filename.");
    }
    requireHash(capture.sha256, "captures[" + index + "].sha256");
    if (!Number.isInteger(capture.width) || capture.width < 1 || !Number.isInteger(capture.height) || capture.height < 1) {
      throw new Error("Manifest capture " + index + " has invalid screenshot dimensions.");
    }
  }
  return true;
}

export { validateManifestForWrite };

/** @param {string} outputPath @param {any} manifest */
async function writeManifest(outputPath, manifest) {
  if (manifest.status === "complete") validateManifestForWrite(manifest);
  await writeFile(join(outputPath, MANIFEST_NAME), JSON.stringify(manifest, null, 2) + "\n", "utf8");
}

/** @param {string} browserVersion */
function genericAndroidUserAgent(browserVersion) {
  const major = browserVersion.split(".")[0];
  return "Mozilla/5.0 (Linux; Android 15; Mobile) AppleWebKit/537.36 "
    + "(KHTML, like Gecko) Chrome/" + major + ".0.0.0 Mobile Safari/537.36";
}

/** @returns {Record<string, any>} */
function newNetworkLog() {
  return {
    allowed: [],
    forbidden: [],
    failed: [],
    expectedTransportCancellations: [],
    consoleErrors: [],
    pageErrors: [],
    expectedBootstrapBlocked: {
      requests: [],
      requestFailures: [],
      consoleErrors: [],
    },
  };
}

/** @param {string} url @param {any} transportModule @param {any} normalizedTransport */
export function isOptionalBootstrapURL(url, transportModule, normalizedTransport) {
  if (typeof transportModule?.isOptionalHistoryBootstrapURL !== "function") return false;
  try {
    return transportModule.isOptionalHistoryBootstrapURL(url, normalizedTransport) === true;
  } catch {
    return false;
  }
}

/** @param {BrowserContext} context @param {any} source @param {any} transport @param {any} network */
async function installObservers(context, source, transport, network) {
  const transportModule = transport?.module;
  const normalizedTransport = transport?.normalized;
  const identityOrDataPath = /(?:identitytoolkit\.googleapis\.com|securetoken\.googleapis\.com|firestore\.googleapis\.com|\/(?:api|auth|cloud|firebase|firestore|identity)(?:\/|$)|\/v1\/projects\/.*\/databases|\/__\/auth\/|\/oauth2\/|\/accounts:|\/token(?:\/|$))/i;
  const observedAt = performance.now();
  /** @type {WeakMap<import('@playwright/test').Request, {status:number,receivedAfterMs:number,timing:Record<string,number>}>} */
  const responsesByRequest = new WeakMap();

  /** @param {Route} route */
  await context.route("**/*", async (route) => {
    const request = route.request();
    const rawUrl = request.url();
    const parsed = (() => { try { return new URL(rawUrl); } catch { return undefined; } })();
    let allowedByTransport = false;
    if (transportModule) {
      try { allowedByTransport = transportModule.isHistoryTransportURL(rawUrl, normalizedTransport) === true; }
      catch { allowedByTransport = false; }
    }
    if (isOptionalBootstrapURL(rawUrl, transportModule, normalizedTransport)) {
      network.expectedBootstrapBlocked.requests.push({
        method: request.method(),
        url: rawUrl,
        resourceType: request.resourceType(),
        reason: "optional hosted popup bootstrap is blocked by capture policy",
      });
      await route.abort("blockedbyclient");
      return;
    }
    const policy = parsed && identityOrDataPath.test(parsed.host + parsed.pathname) && !allowedByTransport
      ? { allowed: false, reason: "identity-or-data-endpoint" }
      : requestPolicy(rawUrl, request.method(), source.origin, allowedByTransport);
    const event = {
      method: request.method(),
      url: safeUrl(rawUrl),
      resourceType: request.resourceType(),
      reason: policy.reason,
    };
    if (!policy.allowed) {
      network.forbidden.push(event);
      await route.abort("blockedbyclient");
      return;
    }
    network.allowed.push(event);
    await route.continue();
  });

  /** @param {import("@playwright/test").WebSocketRoute} route */
  await context.routeWebSocket("**/*", async (route) => {
    const rawUrl = route.url();
    network.forbidden.push({
      method: "WEBSOCKET",
      url: safeUrl(rawUrl),
      resourceType: "websocket",
      reason: "websocket-blocked",
    });
    await route.close({ code: 1008, reason: "History capture blocks unapproved WebSocket connections." });
  });

  context.on("page", (page) => {
    page.on("response", (response) => {
      const request = response.request();
      responsesByRequest.set(request, {
        status: response.status(),
        receivedAfterMs: Math.round(performance.now() - observedAt),
        timing: request.timing(),
      });
    });
    page.on("console", (message) => {
      if (message.type() !== "error") return;
      const location = message.location();
      const event = {
        text: redactText(message.text()).slice(0, 800),
        location: {
          url: location.url ? safeUrl(location.url) : null,
          lineNumber: location.lineNumber,
          columnNumber: location.columnNumber,
        },
      };
      if (location.url && isOptionalBootstrapURL(location.url, transportModule, normalizedTransport)) {
        network.expectedBootstrapBlocked.consoleErrors.push(event);
      } else {
        network.consoleErrors.push(event);
      }
    });
    page.on("pageerror", (error) => network.pageErrors.push({ message: redactText(error.message).slice(0, 800) }));
    page.on("requestfailed", (request) => {
      const rawUrl = request.url();
      const optionalBootstrap = isOptionalBootstrapURL(rawUrl, transportModule, normalizedTransport);
      const event = {
        method: request.method(),
        url: optionalBootstrap ? rawUrl : safeUrl(rawUrl),
        resourceType: request.resourceType(),
        failure: redactText(request.failure()?.errorText ?? "unknown").slice(0, 300),
      };
      if (optionalBootstrap) {
        network.expectedBootstrapBlocked.requestFailures.push(event);
      } else {
        const response = responsesByRequest.get(request);
        const timing = request.timing();
        const transportAllows = Boolean(transportModule && (() => {
          try { return transportModule.isHistoryTransportURL(rawUrl, normalizedTransport) === true; }
          catch { return false; }
        })());
        const cancellationEvent = {
          ...event,
          responseStatus: response?.status ?? null,
          responseReceivedAfterMs: response?.receivedAfterMs ?? null,
          failureObservedAfterMs: Math.round(performance.now() - observedAt),
          requestTiming: timing,
        };
        if (isExpectedFirestoreChannelCancellation(cancellationEvent, normalizedTransport?.mode, transportAllows)) {
          network.expectedTransportCancellations.push({
            ...cancellationEvent,
            classification: "approved native-emulator Firestore WebChannel GET returned HTTP 200 before ERR_ABORTED",
          });
        } else {
          network.failed.push(cancellationEvent);
        }
      }
    });
  });
}

/** @param {BrowserContext} context @param {string} tag @param {string} fixtureId @param {string} origin @param {{emptyHistory?:boolean}} [options] */
async function seedLocalStorage(context, tag, fixtureId, origin, options = {}) {
  const seed = localStorageSeed(tag, fixtureId, options);
  const entries = [
    { key: seed.key, value: seed.value },
    ...(seed.additional ?? []),
  ].map(({ key, value }) => ({ key, value: JSON.stringify(value) }));
  await context.addInitScript((/** @type {{origin:string,entries:Array<{key:string,value:string}>}} */ { origin: expectedOrigin, entries: fixtureEntries }) => {
    if (window.location.origin !== expectedOrigin) return;
    for (const entry of fixtureEntries) window.localStorage.setItem(entry.key, entry.value);
  }, { origin, entries });
}

/** @param {Page} page */
async function waitForFonts(page) {
  const result = await page.evaluate(async () => {
    const families = [...new Set([...document.fonts].map((face) => face.family.replace(/^['"]|['"]$/g, "")))];
    const loadedByFamily = await Promise.all(families.map(async (family) => ({
      family,
      faces: await document.fonts.load('16px "' + family.replace(/"/g, "\\\"") + '"'),
    })));
    await document.fonts.ready;
    return {
      status: document.fonts.status,
      faces: loadedByFamily.map(({ family, faces }) => ({
        family,
        loadedFaceCount: faces.length,
        loaded: faces.length > 0 && faces.every((face) => face.status === "loaded"),
        status: faces.every((face) => face.status === "loaded") ? "loaded" : "error",
      })),
    };
  });
  const notReady = result.faces.filter((font) => !font.loaded || font.status !== "loaded");
  if (result.status !== "loaded") throw new Error("Document font set did not settle to loaded state.");
  if (notReady.length) throw new Error("Application font faces did not finish loading: " + notReady.map((font) => font.family).join(", "));
  return { status: result.status, faces: result.faces };
}

/** @param {Page} page */
async function waitForHeading(page) {
  const heading = page.getByRole("heading", { level: 1 }).first();
  await heading.waitFor({ state: "visible", timeout: 30_000 });
  const text = (await heading.innerText()).trim();
  if (!text) throw new Error("Route rendered an empty H1.");
  return text;
}

/** @param {Page} page */
async function inspectBoard(page) {
  return page.evaluate(() => {
    /** @param {Element} element */
    const visible = (element) => {
      const rect = element.getBoundingClientRect();
      const style = getComputedStyle(element);
      return style.display !== "none" && style.visibility !== "hidden" && rect.width > 1 && rect.height > 1;
    };
    const canvases = [...document.querySelectorAll("canvas")].filter(visible).map((canvas) => {
      const rect = canvas.getBoundingClientRect();
      return {
        width: Math.round(rect.width),
        height: Math.round(rect.height),
        renderer: canvas.getAttribute("data-renderer") ?? "unknown",
      };
    });
    const grids = [...document.querySelectorAll('[role="grid"], [data-testid*="board" i], [class*="boardPanel" i]')]
      .filter(visible).map((element) => {
        const rect = element.getBoundingClientRect();
        return { width: Math.round(rect.width), height: Math.round(rect.height) };
      });
    return { canvases, grids };
  });
}

/** @param {Page} page @param {string} tag @param {string} screen */
async function assertBoard(page, tag, screen) {
  const board = await inspectBoard(page);
  const hasCanvas = board.canvases.some((canvas) => canvas.width > 100 && canvas.height > 100);
  const hasGrid = board.grids.some((grid) => grid.width > 100 && grid.height > 100);
  if (!hasCanvas && !hasGrid) {
    throw new Error(tag + " " + screen + " has no nonzero board canvas or grid. Observed=" + JSON.stringify(board));
  }
  return { ...board, rendererObservation: board.canvases[0]?.renderer ?? "unknown" };
}

/** @param {any} source @param {any} scene @param {any} transportMetadata */
async function routePathFor(source, scene, transportMetadata) {
  if (scene.screen === "public-player") {
    const uid = transportMetadata?.publicUid;
    if (typeof uid !== "string" || !/^[A-Za-z0-9_-]{1,128}$/.test(uid)) {
      throw new Error("Synthetic transport did not provide a valid deterministic publicUid.");
    }
    return "/player?id=" + encodeURIComponent(uid);
  }
  if (scene.screen === "shared-replay") {
    const sharedReplayUrl = transportMetadata?.sharedReplayUrl;
    if (typeof sharedReplayUrl !== "string") throw new Error("Synthetic transport did not provide a real sharedReplayUrl fixture.");
    const parsed = new URL(sharedReplayUrl, source.baseUrl);
    if (parsed.origin !== source.origin || !parsed.pathname.startsWith("/replay/shared")) {
      throw new Error("Synthetic transport sharedReplayUrl must use the tagged loopback preview's /replay/shared route.");
    }
    if (!parsed.hash.startsWith("#g2r1=")) throw new Error("Synthetic transport shared replay URL has no supported portable replay fragment.");
    if (scene.state === "from-profile") {
      if (source.tag < "v0.6.2") throw new Error(source.tag + " predates Shared Replay return-to-profile navigation.");
      if (typeof transportMetadata.publicUid !== "string") throw new Error("Synthetic transport did not provide a public identity for Shared Replay return state.");
      parsed.searchParams.set("fromUid", transportMetadata.publicUid);
      parsed.searchParams.set("fromPage", "1");
    }
    return parsed.pathname + parsed.search + parsed.hash;
  }
  return routeForScene(source.tag, scene.screen, FIXTURES["missed_defense_closed_four"].id, scene.state);
}

/** @param {Page} page @param {any} source @param {string} path */
async function navigateTo(page, source, path) {
  const target = new URL(path, source.baseUrl);
  if (target.origin !== source.origin) throw new Error("Capture route escaped the prepared preview origin.");
  await page.goto(target.href, { waitUntil: "domcontentloaded", timeout: 45_000 });
  const current = new URL(page.url());
  if (current.origin !== source.origin) throw new Error("Tagged application navigated away from its preview origin.");
  return current.pathname + current.search + current.hash;
}

/** @param {Page} page */
async function replayPly(page) {
  const timeline = page.getByRole("slider", { name: "Replay timeline" });
  if (await timeline.count()) {
    const value = await timeline.inputValue();
    const maximum = await timeline.getAttribute("max");
    return { ply: Number(value), total: Number(maximum) };
  }
  const counter = page.getByTestId("replay-move-count");
  if (await counter.count()) {
    const text = await counter.first().innerText();
    const match = text.match(/move\s+(\d+)\s*(?:\/|of)\s*(\d+)/i);
    if (match) return { ply: Number(match[1]), total: Number(match[2]) };
  }
  const body = await page.locator("body").innerText();
  const match = body.match(/move\s+(\d+)\s*(?:\/|of)\s*(\d+)/i);
  return match ? { ply: Number(match[1]), total: Number(match[2]) } : { ply: NaN, total: NaN };
}

/** @param {Page} page @param {number} expectedMoves */
async function seekReplayFinal(page, expectedMoves) {
  const timeline = page.getByRole("slider", { name: "Replay timeline" });
  if (await timeline.count()) {
    const maximum = Number(await timeline.getAttribute("max"));
    if (maximum !== expectedMoves) throw new Error("Replay fixture move total mismatch: expected " + expectedMoves + ", got " + maximum + ".");
    await timeline.focus();
    await timeline.press("End");
  } else {
    const next = page.getByRole("button", { name: "Next move", exact: true });
    const state = await replayPly(page);
    if (!Number.isInteger(state.total) || state.total !== expectedMoves) throw new Error("Replay move total could not be verified: " + JSON.stringify(state));
    for (let index = state.ply; index < expectedMoves; index += 1) await next.click();
  }
  await page.waitForFunction((/** @type {number} */ expected) => {
    const slider = document.querySelector('input[aria-label="Replay timeline"]');
    if (slider instanceof HTMLInputElement) return Number(slider.value) === expected;
    const counter = document.querySelector('[data-testid="replay-move-count"]');
    return Boolean(counter && new RegExp("move\\s+" + expected + "\\s*(?:/|of)\\s*" + expected, "i").test(counter.textContent ?? ""));
  }, expectedMoves, { timeout: 20_000 });
  const final = await replayPly(page);
  if (final.ply !== expectedMoves || final.total !== expectedMoves) {
    throw new Error("Replay did not reach final ply " + expectedMoves + ": " + JSON.stringify(final));
  }
  return final;
}

/** @param {Page} page */
async function waitReplayAnalysis(page) {
  const status = page.getByTestId("replay-analysis-status");
  if (!(await status.count())) return { status: "not-present" };
  // This section is intentionally display:none in the tagged portrait layout.
  await status.waitFor({ state: "attached", timeout: 30_000 });
  await page.waitForFunction((/** @type {string} */ transientPattern) => {
    const element = document.querySelector('[data-testid="replay-analysis-status"]');
    const text = element?.textContent?.trim() ?? "";
    return text !== "" && !new RegExp(transientPattern, "i").test(text);
  }, TRANSIENT_REPLAY_ANALYSIS_STATUS.source, { timeout: 90_000 });
  const label = (await status.innerText()).trim();
  if (!isSettledReplayAnalysisStatus(label)) {
    throw new Error("Replay analysis status changed before capture: " + label);
  }
  return { status: label, visible: await status.isVisible() };
}

/** @param {string} tag */
function expectedBotName(tag) {
  return tag >= "v0.4.5" ? "Normal Bot" : tag < "v0.2.4" ? "Classic Bot" : "Practice Bot";
}

/** @type {Readonly<Record<string, readonly string[]>>} */
const BOT_SEARCH_HEADERS = Object.freeze({
  "v0.4.1": Object.freeze(["Avg nodes", "Eval", "Cand gen", "Breadth", "Legal", "TT hit/cut"]),
  "v0.4.2": Object.freeze(["Avg nodes", "Eval", "Cand gen", "Breadth", "Legal", "TT hit/cut"]),
  "v0.4.3": Object.freeze(["Avg nodes", "Eval", "Cand gen", "Breadth", "Legal", "Portal", "Exit", "TT hit/cut"]),
  "v0.4.4": Object.freeze(["Nodes", "Move gen", "Ordering", "Scoring", "Threat detection", "Proof", "Other", "TT", "Pool"]),
});

/** @param {string} tag @param {string[]} headers @returns {string[]} */
export function validateBotSearchHeaders(tag, headers) {
  const normalized = headers.map((header) => header.replace(/\s+/g, " ").trim());
  const expected = BOT_SEARCH_HEADERS[tag];
  if (!expected) return normalized;
  const missing = expected.filter((name) => !normalized.some((header) => {
    const actual = header.toLowerCase();
    const required = name.toLowerCase();
    return actual === required || actual.startsWith(required + " ");
  }));
  if (missing.length) {
    throw new Error(tag + " Bot Report Search headers do not match the tagged report; missing " + missing.join(", ") + "; observed " + JSON.stringify(normalized));
  }
  return normalized;
}

/** @param {unknown} value */
export function isProfileResetConfirmation(value) {
  return typeof value === "string"
    && /\b(?:resets?|clears?)\b/i.test(value)
    && /\b(?:profile|settings|history|games|replays|data|cloud|device)\b/i.test(value);
}

/** @param {any} assetEvidence @param {any} fontReadiness */
export function validateFontPreviewEvidence(assetEvidence, fontReadiness) {
  if (!Number.isInteger(assetEvidence?.fontCardCount) || assetEvidence.fontCardCount < 1) {
    throw new Error("Tagged font preview has no rendered font specimen cards.");
  }
  if (!Number.isInteger(assetEvidence.expectedFontFaceCount) || assetEvidence.expectedFontFaceCount < 1) {
    throw new Error("Tagged font preview must declare the expected font-face count from its specimen cards.");
  }
  if (typeof assetEvidence.sampleText !== "string" || !assetEvidence.sampleText.includes("Practice Bot vs Local Player")) {
    throw new Error("Tagged font preview has no expected live text specimen.");
  }
  const loadedFaces = (fontReadiness?.faces ?? []).filter((/** @type {any} */ face) =>
    /^PixelOperator/.test(face.family) && face.loaded === true && face.loadedFaceCount > 0);
  if (loadedFaces.length < assetEvidence.expectedFontFaceCount) {
    throw new Error("Tagged font preview loaded " + loadedFaces.length + " of " + assetEvidence.expectedFontFaceCount + " expected PixelOperator font faces.");
  }
  return {
    family: "fonts",
    presentation: "tagged live text rendered with the actual font faces",
    specimenCards: assetEvidence.fontCardCount,
    specimenText: assetEvidence.sampleText,
    loadedFontFaceCount: loadedFaces.length,
    loadedFontFamilies: loadedFaces.map((/** @type {any} */ face) => face.family),
  };
}

/** @param {string} tag @param {string} state @returns {{role:"button"|"tab",label:string}|null} */
export function visualsControlForState(tag, state) {
  if (tag < "v0.5.1" || !["sprites", "icons", "fonts"].includes(state)) return null;
  const label = state === "fonts" ? (tag === "v0.5.1" ? "Guide" : "Style")
    : state === "sprites" ? "Sprites" : "Icons";
  return { role: tag === "v0.5.1" ? "button" : "tab", label };
}

/** @param {string} tag @param {string} state @returns {{role:"button"|"tab",label:string}|null} */
export function labControlForState(tag, state) {
  if (tag < "v0.5.1" || !["search", "analysis"].includes(state)) return null;
  return {
    role: tag >= "v0.5.2" ? "tab" : "button",
    label: state === "search" ? "Search" : "Analysis",
  };
}

/** @param {string} state @param {any} evidence @param {any} fontReadiness */
export function validateReactVisualEvidence(state, evidence, fontReadiness = null) {
  if (!["sprites", "icons", "fonts"].includes(state)) {
    throw new Error("Unsupported React Visuals evidence state: " + state);
  }
  const requiredHeading = state === "sprites" ? "Sprites" : state === "icons" ? "Icons" : "Typography";
  if (typeof evidence?.heading !== "string" || evidence.heading.trim().toLowerCase() !== requiredHeading.toLowerCase()) {
    throw new Error("React Visuals " + state + " state is missing its tagged " + requiredHeading + " heading.");
  }
  if (!Number.isInteger(evidence.specimenCount) || evidence.specimenCount < 1) {
    throw new Error("React Visuals " + state + " state has no rendered tagged specimens.");
  }
  if (state === "fonts") {
    const sampleTexts = Array.isArray(evidence.sampleTexts)
      ? evidence.sampleTexts.filter((/** @type {any} */ text) => typeof text === "string" && text.trim() !== "")
      : [];
    if (sampleTexts.length < evidence.specimenCount) {
      throw new Error("React Visuals Typography is missing live text from one or more specimen cards.");
    }
    const loadedFaces = (fontReadiness?.faces ?? []).filter((/** @type {any} */ face) =>
      /^PixelOperator/i.test(face.family) && face.loaded === true && face.loadedFaceCount > 0);
    if (loadedFaces.length < 1) {
      throw new Error("React Visuals Typography has no loaded tagged PixelOperator font face.");
    }
    return {
      family: state,
      presentation: "tagged React Visuals typography samples rendered with loaded PixelOperator font faces",
      heading: requiredHeading,
      specimenCards: evidence.specimenCount,
      specimenTexts: sampleTexts,
      loadedFontFaceCount: loadedFaces.length,
      loadedFontFamilies: loadedFaces.map((/** @type {any} */ face) => face.family),
      pageTab: evidence.pageTab,
    };
  }
  const requiredSupportingHeading = state === "sprites" ? "Source Sheets" : "Exported SVGs";
  if (typeof evidence.supportingHeading !== "string"
    || evidence.supportingHeading.trim().toLowerCase() !== requiredSupportingHeading.toLowerCase()) {
    throw new Error("React Visuals " + state + " state is missing its tagged " + requiredSupportingHeading + " section.");
  }
  return {
    family: state,
    presentation: "tagged React Visuals specimen section",
    heading: requiredHeading,
    supportingHeading: requiredSupportingHeading,
    specimenCount: evidence.specimenCount,
    pageTab: evidence.pageTab,
  };
}

/** @param {any} evidence */
export function validateLabAnalysisEvidence(evidence) {
  if (evidence?.resultRows < 1) {
    throw new Error("Lab Analysis has no rendered analysis report rows.");
  }
  for (const heading of ["Results", "Lab Notes", "Provenance"]) {
    if (!Array.isArray(evidence?.headings)
      || !evidence.headings.some((/** @type {any} */ actual) =>
        typeof actual === "string" && actual.trim().toLowerCase() === heading.toLowerCase())) {
      throw new Error("Lab Analysis is missing its tagged " + heading + " section.");
    }
  }
  return {
    id: "analysis",
    routeState: "tab=analysis",
    resultRows: evidence.resultRows,
    headings: evidence.headings,
  };
}

/** @param {any} evidence */
export function validateSharedReplayContext(evidence) {
  if (evidence?.attached !== true || typeof evidence.text !== "string"
    || evidence.text.trim().toLowerCase() !== "shared replay") {
    throw new Error("Shared Replay is missing its tagged Shared replay context label.");
  }
  return {
    label: "Shared replay",
    visible: evidence.visible === true,
    visibilityNote: evidence.visible === true
      ? "visible in the tagged layout"
      : "attached but hidden by the tagged compact portrait layout",
  };
}

/** @param {any} evidence */
export function validateReturnToPublicProfileControl(evidence) {
  if (evidence?.role !== "button" || evidence?.label !== "Return to public profile" || evidence?.visible !== true) {
    throw new Error("Shared Replay from-profile state is missing its visible tagged Return to public profile button.");
  }
  return { role: evidence.role, label: evidence.label, visible: true };
}

/** @param {any} evidence */
export function validateGuideExamplesEvidence(evidence) {
  if (typeof evidence?.heading !== "string" || evidence.heading.trim() !== "Example replays") {
    throw new Error("Guide Examples state is missing its tagged Example replays section heading.");
  }
  if (!Array.isArray(evidence.entries) || evidence.entries.length !== 2) {
    throw new Error("Guide Examples state must render the two tagged replay example rows.");
  }
  const entries = evidence.entries.map((/** @type {any} */ entry) => {
    if (typeof entry?.title !== "string" || !entry.title.trim()
      || typeof entry?.href !== "string" || !entry.href.startsWith("/replay/shared/")) {
      throw new Error("Guide Examples row is missing its tagged title or real shared-replay link.");
    }
    return { title: entry.title.trim(), href: entry.href };
  });
  return { heading: "Example replays", rowCount: entries.length, linkCount: entries.length, entries };
}

/** @param {Page} page @param {any} source @param {string} state @param {any} transportMetadata */
async function assertProfileFixture(page, source, state, transportMetadata) {
  let nameInput = page.locator(PROFILE_NAME_INPUT_SELECTOR).first();
  if (!(await nameInput.count())) {
    nameInput = page.locator("label").filter({ hasText: PROFILE_NAME_LABEL_PATTERN }).locator("input").first();
  }
  await nameInput.waitFor({ state: "attached", timeout: 20_000 });
  const name = await nameInput.inputValue();
  if (name !== FIXTURES["missed_defense_closed_four"].displayName) {
    throw new Error(source.tag + " Profile identity mismatch: expected Reference Player, got " + JSON.stringify(name));
  }

  const botName = expectedBotName(source.tag);
  /** @type {Record<string, any>} */
  const observed = { displayName: name, historyOpponent: botName };
  if (state === "default") {
    const historyRows = await page.locator("li").allInnerTexts();
    if (!historyRows.some((row) => row.includes(botName) && /win/i.test(row))) {
      throw new Error(source.tag + " Profile history fixture is missing the completed win vs " + botName + ".");
    }
    observed.localHistory = "same seeded completed win";
  } else if (state === "empty") {
    const historyPanel = page.getByText("Match History", { exact: true }).locator("xpath=../..");
    const historyRows = await historyPanel.locator("li").allInnerTexts();
    if (historyRows.length > 0 || await historyPanel.locator("ol").count() > 0) {
      throw new Error(source.tag + " Profile empty fixture unexpectedly rendered history rows: " + JSON.stringify(historyRows));
    }
    if (source.tag === "v0.5.4") {
      await page.getByText("No matches yet", { exact: true }).waitFor({ state: "visible", timeout: 20_000 });
      await page.getByText("Finish a game to inspect it here.", { exact: true }).waitFor({ state: "visible", timeout: 20_000 });
      await page.getByRole("link", { name: "Play a game", exact: true }).waitFor({ state: "visible", timeout: 20_000 });
      observed.localHistory = {
        matchRows: 0,
        emptyCopy: "No matches yet / Finish a game to inspect it here.",
        action: "Play a game",
      };
    } else {
      if (await page.getByText("No matches yet", { exact: true }).count()) {
        throw new Error("v0.5.3 tagged Profile does not include the v0.5.4 empty-history copy.");
      }
      observed.localHistory = {
        matchRows: 0,
        emptyCopy: "not present in tagged Profile; history area is blank",
      };
    }
  } else if (state === "account") {
    const signOut = page.locator("button").filter({ hasText: "Sign out" }).first();
    await signOut.waitFor({ state: "attached", timeout: 30_000 });
    if (!(await signOut.isEnabled())) throw new Error(source.tag + " Profile signed-in control is disabled.");
    observed.account = {
      session: "signed-in synthetic demo account",
      uid: transportMetadata?.uid ?? null,
      signOutControlAttached: true,
      signOutControlVisible: await signOut.isVisible(),
    };
  } else if (state === "sync") {
    await page.waitForFunction(() => {
      const body = document.body.innerText;
      return /synced|queued|syncing|pending|local only/i.test(body) && !/checking|loading profile|loading history/i.test(body);
    }, undefined, { timeout: 45_000 });
    observed.syncStatus = await page.locator("body").innerText().then((body) => {
      const line = body.split("\n").find((part) => /synced|queued|syncing|pending|local only/i.test(part));
      return line?.trim() ?? "observed";
    });
  } else if (state === "reset") {
    const details = page.locator("details.accountControls");
    if (await details.count()) {
      const summary = details.locator("summary").first();
      if (!(await details.evaluate((element) => element instanceof HTMLDetailsElement && element.open))) await summary.click();
    }
    const resetButton = page.getByRole("button", { name: /reset (?:local profile|profile|private data)/i }).first();
    await resetButton.waitFor({ state: "visible", timeout: 20_000 });
    await resetButton.click();
    const confirmationCopy = page.locator('p[class*="resetConfirmText"]').first();
    await confirmationCopy.waitFor({ state: "visible", timeout: 20_000 });
    const copy = (await confirmationCopy.innerText()).trim();
    if (!isProfileResetConfirmation(copy)) {
      throw new Error(source.tag + " Profile reset confirmation has unexpected copy: " + JSON.stringify(copy));
    }
    const confirmAction = page.getByRole("button", { name: /^(?:confirm reset|reset)$/i }).first();
    const cancelAction = page.getByRole("button", { name: "Cancel", exact: true }).first();
    await confirmAction.waitFor({ state: "visible", timeout: 20_000 });
    await cancelAction.waitFor({ state: "visible", timeout: 20_000 });
    observed.resetConfirmation = {
      copy,
      confirmAction: (await confirmAction.innerText()).trim(),
      cancelAction: true,
      activated: false,
    };
  } else if (state === "saved-replays") {
    const savedTab = page.getByRole("tab", { name: "Saved replays", exact: true });
    await savedTab.waitFor({ state: "visible", timeout: 20_000 });
    if (await savedTab.getAttribute("aria-selected") !== "true") throw new Error(source.tag + " Profile Saved replays tab is not selected.");
    const panel = page.locator('section[aria-label="Saved replays"]');
    await panel.getByRole("heading", { name: "Saved replays", exact: true }).waitFor({ state: "visible", timeout: 20_000 });
    const rows = await panel.locator("li").allInnerTexts();
    if (!rows.some((row) => row.includes("Reference Player") && row.includes(botName) && /9 moves/i.test(row))) {
      throw new Error(source.tag + " Profile saved-replays tab is missing its seeded legal nine-move fixture: " + JSON.stringify(rows));
    }
    observed.savedReplays = { selected: await savedTab.getAttribute("aria-selected"), rows: rows.length, fixture: "same completed local match" };
  } else if (state === "public-settings") {
    const publicTab = page.getByRole("tab", { name: "Public profile", exact: true });
    await publicTab.waitFor({ state: "visible", timeout: 20_000 });
    if (await publicTab.getAttribute("aria-selected") !== "true") throw new Error(source.tag + " Profile Public profile tab is not selected.");
    await page.getByRole("heading", { name: "Public profile", exact: true }).waitFor({ state: "visible", timeout: 30_000 });
    await page.getByLabel("Public profile link").waitFor({ state: "visible", timeout: 30_000 });
    await page.getByText("Published games", { exact: true }).waitFor({ state: "visible", timeout: 30_000 });
    observed.publicSettings = { selected: await publicTab.getAttribute("aria-selected"), publicUid: transportMetadata?.uid ?? null };
  }
  return observed;
}

/** @param {Page} page @param {any} source @param {any} scene @param {any} transportMetadata @returns {Promise<Record<string, any>>} */
async function assertRouteContent(page, source, scene, transportMetadata) {
  const heading = await waitForHeading(page);
  /** @type {Record<string, any>} */
  const observed = { heading };
  if (scene.screen === "match") {
    const expected = source.tag === "v0.2.1" ? 0 : 8;
    const countTest = page.getByTestId("match-move-count");
    const countSelector = source.tag === "v0.2.1" ? page.locator('[class*="historyCount"]').first() : countTest;
    const countText = (await countSelector.innerText()).trim();
    const count = Number(countText.match(/\d+/)?.[0]);
    if (count !== expected) throw new Error(source.tag + " Match move count mismatch: expected " + expected + ", saw " + countText);
    const statusSelector = source.tag === "v0.2.1" ? page.locator('p[class*="statusText"]').first() : page.getByTestId("match-status");
    const status = (await statusSelector.innerText()).trim();
    if (!/Reference Player to move/i.test(status)) throw new Error(source.tag + " Match did not settle on the idle human Black turn: " + status);
    const black = source.tag === "v0.2.1" ? page.locator('article[class*="playerCardBlack"]').first() : page.getByTestId("player-row-black");
    const blackText = await black.innerText();
    if (!blackText.includes("Reference Player") || !/human|player/i.test(blackText)) throw new Error(source.tag + " Match black row is not the seeded human.");
    const white = source.tag === "v0.2.1" ? page.locator('article[class*="playerCardWhite"]').first() : page.getByTestId("player-row-white");
    const whiteText = await white.innerText();
    if (!whiteText.includes(expectedBotName(source.tag))) throw new Error(source.tag + " Match white row does not use its tagged default bot.");
    observed.match = {
      moves: count,
      status,
      black: "Reference Player (human)",
      white: expectedBotName(source.tag),
      semanticSource: source.tag === "v0.2.1" ? "historical row text/labels" : "data-testid rows and tagged Player/Bot labels",
    };
    observed.board = await assertBoard(page, source.tag, scene.screen);
  } else if (scene.screen === "profile") {
    observed.fixture = await assertProfileFixture(page, source, scene.state, transportMetadata);
  } else if (scene.screen === "replay") {
    const expectedMoves = FIXTURES["missed_defense_closed_four"].moveCells.length;
    const ply = await seekReplayFinal(page, expectedMoves);
    const blackRow = page.getByTestId("replay-player-row-black");
    const whiteRow = page.getByTestId("replay-player-row-white");
    const replayText = await page.locator("body").innerText();
    const black = await blackRow.count() ? await blackRow.innerText() : replayText;
    const white = await whiteRow.count() ? await whiteRow.innerText() : replayText;
    if (!black.includes("Reference Player") || !white.includes(expectedBotName(source.tag))) {
      throw new Error(source.tag + " Replay player rows do not match the shared fixture.");
    }
    observed.replay = {
      ply: ply.ply,
      total: ply.total,
      black: "Reference Player",
      white: expectedBotName(source.tag),
      semanticSource: await blackRow.count() ? "player rows" : "historical replay summary/move list",
    };
    if (source.tag >= "v0.4.6") observed.analysis = await waitReplayAnalysis(page);
    observed.board = await assertBoard(page, source.tag, scene.screen);
    if (scene.state === "share") {
      const panel = page.locator('section[aria-label="Replay sharing"]');
      await panel.waitFor({ state: "visible", timeout: 20_000 });
      const input = panel.getByLabel("Replay link");
      await input.waitFor({ state: "visible", timeout: 30_000 });
      const link = await input.inputValue();
      if (!link.includes("/replay/shared/") || !link.includes("#g2r1=")) {
        throw new Error(source.tag + " Replay share state did not produce the tagged portable replay link.");
      }
      observed.replayAction = { panel: "share", portableLinkReady: true, copied: false };
    } else if (scene.state === "publish") {
      const panel = page.locator('section[aria-label="Publish replay"]');
      await panel.waitFor({ state: "visible", timeout: 20_000 });
      await panel.getByLabel("Public replay title").waitFor({ state: "visible", timeout: 30_000 });
      const publishButton = panel.getByRole("button", { name: "Publish this replay", exact: true });
      await publishButton.waitFor({ state: "visible", timeout: 20_000 });
      if (await publishButton.isDisabled()) throw new Error(source.tag + " Replay publish preview is disabled for the deterministic fixture.");
      observed.replayAction = { panel: "publish", previewReady: true, mutationSubmitted: false };
    }
  } else if (scene.screen === "public-player") {
    const headingText = heading.toLowerCase();
    if (!(headingText.includes("reference") || headingText.includes("public"))) {
      throw new Error("Public Player rendered an unexpected identity heading: " + heading);
    }
    await page.getByRole("region", { name: "Shared player statistics" }).waitFor({ state: "visible", timeout: 30_000 });
    observed.publicProfileUid = transportMetadata?.publicUid ?? null;
    observed.publicCatalogRows = await page.locator("[aria-label^='Public games'] li").count();
    if (observed.publicCatalogRows < 1) throw new Error("Public Player default state did not load its synthetic catalog fixture.");
    observed.statsVisible = true;
  } else if (scene.screen === "shared-replay") {
    await page.getByRole("heading", { name: "Replay Analysis", exact: true }).waitFor({ state: "visible", timeout: 30_000 });
    const sharedReplayLabel = page.getByText("Shared replay", { exact: true }).first();
    await sharedReplayLabel.waitFor({ state: "attached", timeout: 20_000 });
    observed.sharedReplayContext = validateSharedReplayContext({
      attached: true,
      text: await sharedReplayLabel.textContent(),
      visible: await sharedReplayLabel.isVisible(),
    });
    if (scene.state === "from-profile") {
      const returnButton = page.getByRole("button", { name: "Return to public profile", exact: true });
      await returnButton.waitFor({ state: "visible", timeout: 20_000 });
      observed.returnToProfileControl = validateReturnToPublicProfileControl({
        role: await returnButton.evaluate((element) => element.getAttribute("role") ?? element.tagName.toLowerCase()),
        label: await returnButton.getAttribute("aria-label"),
        visible: await returnButton.isVisible(),
      });
    }
    observed.portableReplay = true;
    observed.returnToProfile = scene.state === "from-profile";
    observed.board = await assertBoard(page, source.tag, scene.screen);
  } else if (scene.screen === "settings" && scene.state === "custom") {
    const custom = page.getByRole("button", { name: SETTINGS_CUSTOM_BUTTON_NAME });
    await custom.waitFor({ state: "visible", timeout: 20_000 });
    const className = await custom.getAttribute("class") ?? "";
    if (!className.includes("presetCardActive")) throw new Error("Settings Custom state did not become active.");
    observed.customSettings = "selected in tagged Settings UI";
  } else if (scene.state === "search" && (scene.screen === "bot-report" || scene.screen === "lab")) {
    if (scene.screen === "bot-report") {
      const radio = page.locator("#view-search");
        if (await radio.count() && !(await radio.isChecked())) throw new Error("Static Bot Report Search metrics state is not selected.");
        const metricHeaders = await page.locator(".entrant-head .metric-search").allInnerTexts();
        const taggedMetrics = validateBotSearchHeaders(source.tag, metricHeaders);
        if (source.tag >= "v0.4.1" && source.tag <= "v0.4.4" && metricHeaders.length === 0) {
          throw new Error(source.tag + " Search state has no visible tagged metric headers.");
      }
        observed.reportView = { id: "search", taggedMetrics };
    } else {
      const current = new URL(page.url());
      if (current.searchParams.get("tab") !== "search") throw new Error("Lab Search view did not activate its tagged Search tab.");
      observed.reportView = { id: "search", routeState: "tab=search" };
    }
  } else if (scene.state === "pairwise" && scene.screen === "bot-report") {
    const radio = page.locator("#view-pairwise");
    if (await radio.count() && !(await radio.isChecked())) throw new Error("Static Bot Report Pairwise view is not selected.");
    observed.reportView = { id: "pairwise" };
  } else if (scene.screen === "lab" && scene.state === "analysis") {
    const current = new URL(page.url());
    if (current.searchParams.get("tab") !== "analysis") throw new Error("Lab Analysis view did not activate its tagged Analysis tab.");
    const results = page.getByRole("heading", { name: /^Results$/i });
    const notes = page.getByRole("heading", { name: /^Lab Notes$/i });
    const provenance = page.getByRole("heading", { name: /^Provenance$/i });
    await results.waitFor({ state: "visible", timeout: 45_000 });
    await notes.waitFor({ state: "visible", timeout: 20_000 });
    await provenance.waitFor({ state: "visible", timeout: 20_000 });
    const analysisPanel = page.locator('[data-view="analysis"]');
    await analysisPanel.waitFor({ state: "visible", timeout: 20_000 });
    observed.reportView = validateLabAnalysisEvidence({
      headings: [
        (await results.innerText()).trim(),
        (await notes.innerText()).trim(),
        (await provenance.innerText()).trim(),
      ],
      resultRows: await analysisPanel.locator("details").count(),
    });
  } else if (scene.screen === "analysis-report" && scene.state === "proof-open") {
    const entry = page.locator("details.analysis-entry").first();
    if (!(await entry.evaluate((element) => element instanceof HTMLDetailsElement && element.open))) {
      throw new Error("Standalone Analysis Report proof entry did not open.");
    }
    observed.proof = {
      entryOpen: true,
      nestedDetailsOpen: await entry.locator("details[open]").count() > 0,
      frameCount: await entry.locator(".proof-frame").count(),
    };
  } else if (scene.screen === "visuals" && ["sprites", "icons", "fonts"].includes(scene.state)) {
    const route = new URL(page.url());
    if (source.tag < "v0.5.1") {
      if (!route.pathname.endsWith("/" + scene.state + "/preview.html")) {
        throw new Error("Visual asset family state did not land on the tagged " + scene.state + " page: " + route.pathname);
      }
      if (scene.state === "fonts") {
        const fontCardCount = await page.locator(".fontCard").count();
        if (fontCardCount > 0) {
          const sampleText = (await page.locator(".fontCard .sampleBody").first().innerText()).trim();
          observed.assetPage = {
            fontCardCount,
            sampleText,
            expectedFontFaceCount: fontCardCount,
            requiresFontFaceProof: true,
          };
        } else {
          const specimens = await page.locator("img, svg, canvas").count();
          if (specimens < 1) throw new Error("Visual asset family fonts page has neither rendered text cards nor specimen assets.");
          observed.assetPage = { family: scene.state, specimenElements: specimens, presentation: "tagged static page" };
        }
      } else {
        const specimens = await page.locator("img, svg, canvas").count();
        if (specimens < 1) throw new Error("Visual asset family page has no rendered specimen assets.");
        observed.assetPage = { family: scene.state, specimenElements: specimens, presentation: "tagged static page" };
      }
    } else {
      const headingName = scene.state === "fonts" ? "Typography" : scene.state === "sprites" ? "Sprites" : "Icons";
      const headingLocator = page.getByRole("heading", { name: new RegExp("^" + headingName + "$", "i") });
      await headingLocator.waitFor({ state: "visible", timeout: 30_000 });
      let evidence;
      if (scene.state === "fonts") {
        evidence = await headingLocator.evaluate((element) => {
          const section = element.closest("section");
          const articles = section ? [...section.querySelectorAll("article")] : [];
          return {
            heading: element.textContent?.trim() ?? "",
            specimenCount: articles.length,
            sampleTexts: articles.map((article) => article.querySelector("p")?.textContent?.trim() ?? ""),
            pageTab: "guide typography section",
          };
        });
        observed.assetPage = { ...evidence, family: "fonts", requiresReactFontFaceProof: true };
      } else {
        const supportingHeading = scene.state === "sprites" ? "Source Sheets" : "Exported SVGs";
        const supportingHeadingLocator = page.getByRole("heading", { name: new RegExp("^" + supportingHeading + "$", "i") });
        await supportingHeadingLocator.waitFor({ state: "visible", timeout: 30_000 });
        evidence = {
          heading: (await headingLocator.innerText()).trim(),
          supportingHeading: (await supportingHeadingLocator.innerText()).trim(),
          specimenCount: await page.getByRole("img").count(),
          pageTab: scene.state,
        };
        observed.assetPage = validateReactVisualEvidence(scene.state, evidence);
      }
    }
  } else if (scene.screen === "visuals") {
    observed.assetPage = { family: "guide/default", route: new URL(page.url()).pathname };
  } else if (scene.screen === "guide" && scene.state === "examples") {
    const section = page.locator('section[aria-labelledby="examples"]');
    const heading = section.getByRole("heading", { name: "Example replays", exact: true });
    await heading.waitFor({ state: "visible", timeout: 20_000 });
    const entries = await section.locator('ul[aria-label="Example replays"] > li').evaluateAll((rows) => rows.map((row) => ({
      title: row.querySelector("h3")?.textContent?.trim() ?? "",
      href: row.querySelector('a[aria-label^="Open "]')?.getAttribute("href") ?? "",
    })));
    observed.examples = validateGuideExamplesEvidence({ heading: (await heading.innerText()).trim(), entries });
  }
  return observed;
}

/** @param {Page} page @param {any} source @param {any} fixture */
async function prepareMatchScene(page, source, fixture) {
  if (source.tag === "v0.2.1") {
    await navigateTo(page, source, "/match/local");
    return { initialRoute: "/match/local", branchFromReplay: false };
  }
  const replayPath = routeForScene(source.tag, "replay", fixture.id);
  await navigateTo(page, source, replayPath);
  await waitForHeading(page);
  const finalPly = await seekReplayFinal(page, fixture.moveCells.length);
  await page.getByRole("button", { name: "Previous move", exact: true }).click();
  await page.waitForFunction((/** @type {number} */ expected) => {
    const slider = document.querySelector('input[aria-label="Replay timeline"]');
    return slider instanceof HTMLInputElement && Number(slider.value) === expected;
  }, fixture.moveCells.length - 1, { timeout: 20_000 });
  const branchPly = await replayPly(page);
  if (branchPly.ply !== 8 || branchPly.total !== 9) {
    throw new Error(source.tag + " Match branch must use one Previous step from ply 9 to ply 8; saw " + JSON.stringify(branchPly));
  }
  const playFromHere = page.getByRole("button", { name: /play from here/i });
  await playFromHere.waitFor({ state: "visible", timeout: 20_000 });
  await playFromHere.click();
  await page.waitForURL((url) => url.origin === source.origin && url.pathname === "/match/local", { timeout: 20_000 });
  return {
    initialRoute: replayPath,
    branchFromReplay: true,
    finalPly: finalPly.ply,
    previousStep: branchPly.ply,
    branchAction: "Play From Here",
  };
}

/** @param {Page} page @param {any} scene @param {any} source */
async function prepareSceneState(page, scene, source) {
  if (scene.screen === "visuals" && ["sprites", "icons", "fonts"].includes(scene.state)
    && source.tag >= "v0.5.1") {
    const control = visualsControlForState(source.tag, scene.state);
    if (!control) throw new Error(source.tag + " has no tagged Visuals control for " + scene.state + ".");
    const selector = page.getByRole(control.role, { name: control.label, exact: true });
    await selector.waitFor({ state: "visible", timeout: 20_000 });
    if (scene.state !== "fonts" || control.role === "tab") {
      if (control.role !== "tab" || await selector.getAttribute("aria-selected") !== "true") {
        await selector.click();
      }
    }
  } else if (scene.screen === "guide" && scene.state === "examples") {
    const examplesNavigation = page.locator('nav[aria-label="Guide sections"] a[href="/guide/#examples"]');
    await examplesNavigation.waitFor({ state: "visible", timeout: 20_000 });
    await examplesNavigation.click();
    await page.waitForURL((url) => url.pathname === "/guide/" && url.hash === "#examples", { timeout: 20_000 });
    await page.locator('section[aria-labelledby="examples"]')
      .getByRole("heading", { name: "Example replays", exact: true })
      .waitFor({ state: "visible", timeout: 20_000 });
  } else if (scene.screen === "settings" && scene.state === "custom") {
    const custom = page.getByRole("button", { name: SETTINGS_CUSTOM_BUTTON_NAME });
    await custom.waitFor({ state: "visible", timeout: 20_000 });
    await custom.click();
  } else if (scene.screen === "bot-report" && ["search", "pairwise"].includes(scene.state)) {
    const view = scene.state;
    const radio = page.locator("#view-" + view);
    if (await radio.count()) {
      await page.locator('label[for="view-' + view + '"]').click();
    } else {
      const button = page.getByRole("button", { name: new RegExp(view, "i") }).first();
      if (await button.count()) await button.click();
      else await page.getByRole("link", { name: new RegExp(view, "i") }).first().click();
    }
  } else if (scene.screen === "profile" && ["saved-replays", "public-settings"].includes(scene.state)) {
    const label = scene.state === "saved-replays" ? "Saved replays" : "Public profile";
    const tab = page.getByRole("tab", { name: label, exact: true });
    await tab.waitFor({ state: "visible", timeout: 20_000 });
    await tab.click();
    await page.waitForFunction((/** @type {string} */ expectedLabel) => {
      const normalizedLabel = expectedLabel.trim().toLowerCase();
      return [...document.querySelectorAll('[role="tab"]')].some((element) =>
        element.getAttribute("aria-selected") === "true"
        && (element.textContent ?? "").trim().toLowerCase() === normalizedLabel);
    }, label, { timeout: 20_000 });
  } else if (scene.screen === "replay" && ["share", "publish"].includes(scene.state)) {
    const label = scene.state === "share" ? "Share replay" : "Publish replay";
    await page.getByRole("button", { name: label, exact: true }).waitFor({ state: "visible", timeout: 20_000 });
    await page.getByRole("button", { name: label, exact: true }).click();
  } else if (scene.screen === "visuals" && ["sprites", "icons", "fonts"].includes(scene.state) && source.tag >= "v0.5.1") {
    const label = scene.state === "sprites" ? "Sprites" : scene.state === "icons" ? "Icons" : "Guide";
    await page.getByRole("button", { name: label, exact: true }).waitFor({ state: "visible", timeout: 20_000 });
    await page.getByRole("button", { name: label, exact: true }).click();
  } else if (scene.screen === "lab" && ["search", "analysis"].includes(scene.state)) {
    const control = labControlForState(source.tag, scene.state);
    if (!control) throw new Error(source.tag + " has no tagged Lab control for " + scene.state + ".");
    const view = page.getByRole(control.role, { name: control.label, exact: true });
    await view.waitFor({ state: "visible", timeout: 20_000 });
    if (control.role !== "tab" || await view.getAttribute("aria-selected") !== "true") await view.click();
  } else if (scene.screen === "analysis-report" && scene.state === "proof-open") {
    const entry = page.locator("details.analysis-entry").first();
    await entry.waitFor({ state: "visible", timeout: 30_000 });
    if (!(await entry.evaluate((element) => element instanceof HTMLDetailsElement && element.open))) {
      await entry.locator("summary").click();
    }
    const nested = entry.locator("details").first();
    if (await nested.count() && !(await nested.evaluate((element) => element instanceof HTMLDetailsElement && element.open))) {
      await nested.locator("summary").click();
    }
  }
}

/** @param {Buffer | Uint8Array} bytes */
function pngDimensions(bytes) {
  const buffer = Buffer.from(bytes);
  if (buffer.length < 24 || buffer.subarray(0, 8).toString("hex") !== "89504e470d0a1a0a") {
    throw new Error("Playwright did not return a valid PNG screenshot.");
  }
  return { width: buffer.readUInt32BE(16), height: buffer.readUInt32BE(20) };
}

/**
 * Select the user-visible CSS viewport for profile verification.
 * A legacy page can grow its layout viewport when a disclosure opens while
 * retaining the configured visual viewport; preserve both measurements in
 * the manifest, and use inner dimensions only when visualViewport is absent.
 * @param {Record<string, any>} metrics
 */
export function cssViewportDimensions(metrics) {
  const visualWidth = metrics.visualViewportWidth;
  const visualHeight = metrics.visualViewportHeight;
  if (Number.isFinite(visualWidth) && visualWidth > 0
    && Number.isFinite(visualHeight) && visualHeight > 0) {
    return { width: visualWidth, height: visualHeight, basis: "visualViewport" };
  }
  const innerWidth = metrics.innerWidth;
  const innerHeight = metrics.innerHeight;
  if (Number.isFinite(innerWidth) && innerWidth > 0
    && Number.isFinite(innerHeight) && innerHeight > 0) {
    return { width: innerWidth, height: innerHeight, basis: "innerWidth fallback" };
  }
  throw new Error("Browser did not expose usable CSS viewport dimensions: " + JSON.stringify(metrics));
}

/** @param {{left:number,top:number,right:number,bottom:number,width:number,height:number}} rect
 * @param {{width:number,height:number}} viewport
 * @param {string} label
 * @param {boolean} [requireTopAlignment]
 * @param {boolean} [requireFullVisibility]
 */
export function validateFramingRect(rect, viewport, label, requireTopAlignment = false, requireFullVisibility = false) {
  const values = [rect?.left, rect?.top, rect?.right, rect?.bottom, rect?.width, rect?.height, viewport?.width, viewport?.height];
  if (!values.every(Number.isFinite) || rect.width <= 0 || rect.height <= 0 || viewport.width <= 0 || viewport.height <= 0) {
    throw new Error(label + " framing measurement is incomplete: " + JSON.stringify({ rect, viewport }));
  }
  const visible = rect.right > 0 && rect.left < viewport.width && rect.bottom > 0 && rect.top < viewport.height;
  if (!visible) throw new Error(label + " anchor is outside the visible visual viewport: " + JSON.stringify({ rect, viewport }));
  const fullyVisible = rect.left >= 0 && rect.top >= 0 && rect.right <= viewport.width && rect.bottom <= viewport.height;
  if (requireFullVisibility && !fullyVisible) {
    throw new Error(label + " anchor is clipped by the visible visual viewport: " + JSON.stringify({ rect, viewport }));
  }
  const topAligned = rect.top >= -2 && rect.top <= 24;
  if (requireTopAlignment && !topAligned) {
    throw new Error(label + " anchor did not align near the visual viewport top: " + JSON.stringify({ rect, viewport }));
  }
  return { visible, fullyVisible, topAligned };
}

/** @param {Page} page @param {import('@playwright/test').Locator} locator @param {string} selector @param {string} label @param {boolean} requireTopAlignment @param {boolean} [scrollElement] @param {boolean} [scrollBlockStart] @param {boolean} [requireFullVisibility] */
async function frameLocatorInViewport(page, locator, selector, label, requireTopAlignment, scrollElement = true, scrollBlockStart = requireTopAlignment, requireFullVisibility = false) {
  await locator.waitFor({ state: "visible", timeout: 20_000 });
  if (scrollElement) {
    await locator.evaluate((element, blockStart) => {
      element.scrollIntoView({ behavior: "instant", block: blockStart ? "start" : "nearest", inline: "nearest" });
    }, scrollBlockStart);
  }
  await page.evaluate(() => new Promise((resolveFrame) => requestAnimationFrame(() => requestAnimationFrame(resolveFrame))));
  const evidence = await locator.evaluate((element, locatorSelector) => {
    const rect = element.getBoundingClientRect();
    const visual = window.visualViewport;
    const offsetLeft = visual?.offsetLeft ?? 0;
    const offsetTop = visual?.offsetTop ?? 0;
    const ancestors = [];
    for (let parent = element.parentElement; parent; parent = parent.parentElement) {
      const style = getComputedStyle(parent);
      const parentRect = parent.getBoundingClientRect();
      ancestors.push({
        tag: parent.tagName.toLowerCase(),
        id: parent.id || null,
        className: typeof parent.className === "string" ? parent.className.slice(0, 120) : null,
        rect: {
          left: Math.round((parentRect.left - offsetLeft) * 10) / 10,
          top: Math.round((parentRect.top - offsetTop) * 10) / 10,
          right: Math.round((parentRect.right - offsetLeft) * 10) / 10,
          bottom: Math.round((parentRect.bottom - offsetTop) * 10) / 10,
          width: Math.round(parentRect.width * 10) / 10,
          height: Math.round(parentRect.height * 10) / 10,
        },
        overflowX: style.overflowX,
        overflowY: style.overflowY,
        scrollLeft: parent.scrollLeft,
        scrollTop: parent.scrollTop,
        clientWidth: parent.clientWidth,
        clientHeight: parent.clientHeight,
        scrollWidth: parent.scrollWidth,
        scrollHeight: parent.scrollHeight,
      });
    }
    const nestedScrollRegion = ancestors.find((parent) =>
      !["body", "html"].includes(parent.tag)
      && /(auto|scroll|overlay)/.test(parent.overflowY)
      && parent.scrollHeight > parent.clientHeight);
    const documentScroller = document.scrollingElement ?? document.documentElement;
    const documentMaxScrollTop = Math.max(0, documentScroller.scrollHeight - documentScroller.clientHeight);
    const scrollRegion = nestedScrollRegion ? {
      ...nestedScrollRegion,
      element: "nearest nested overflow scroll region",
      maxScrollTop: Math.max(0, nestedScrollRegion.scrollHeight - nestedScrollRegion.clientHeight),
      clamp: nestedScrollRegion.scrollTop <= 1
        ? "start"
        : nestedScrollRegion.scrollTop >= Math.max(0, nestedScrollRegion.scrollHeight - nestedScrollRegion.clientHeight) - 1
          ? "end"
          : "intermediate",
    } : {
      tag: documentScroller.tagName.toLowerCase(),
      element: "document.scrollingElement",
      id: documentScroller.id || null,
      className: typeof documentScroller.className === "string" ? documentScroller.className.slice(0, 120) : null,
      rect: {
        left: 0,
        top: 0,
        right: visual?.width ?? window.innerWidth,
        bottom: visual?.height ?? window.innerHeight,
        width: visual?.width ?? window.innerWidth,
        height: visual?.height ?? window.innerHeight,
      },
      overflowX: getComputedStyle(documentScroller).overflowX,
      overflowY: getComputedStyle(documentScroller).overflowY,
      scrollLeft: documentScroller.scrollLeft,
      scrollTop: documentScroller.scrollTop,
      clientWidth: documentScroller.clientWidth,
      clientHeight: documentScroller.clientHeight,
      scrollWidth: documentScroller.scrollWidth,
      scrollHeight: documentScroller.scrollHeight,
      maxScrollTop: documentMaxScrollTop,
      clamp: documentScroller.scrollTop <= 1
        ? "start"
        : documentScroller.scrollTop >= documentMaxScrollTop - 1
          ? "end"
          : "intermediate",
    };
    return {
      selector: locatorSelector,
      text: (element instanceof HTMLElement ? element.innerText : element.textContent ?? "").trim().slice(0, 160),
      rect: {
        left: Math.round((rect.left - offsetLeft) * 10) / 10,
        top: Math.round((rect.top - offsetTop) * 10) / 10,
        right: Math.round((rect.right - offsetLeft) * 10) / 10,
        bottom: Math.round((rect.bottom - offsetTop) * 10) / 10,
        width: Math.round(rect.width * 10) / 10,
        height: Math.round(rect.height * 10) / 10,
      },
      rawRect: {
        left: Math.round(rect.left * 10) / 10,
        top: Math.round(rect.top * 10) / 10,
        right: Math.round(rect.right * 10) / 10,
        bottom: Math.round(rect.bottom * 10) / 10,
      },
      visualViewport: {
        width: visual?.width ?? window.innerWidth,
        height: visual?.height ?? window.innerHeight,
        offsetLeft,
        offsetTop,
        scale: visual?.scale ?? 1,
      },
      scrollRegion,
      scroll: {
        windowX: window.scrollX,
        windowY: window.scrollY,
        documentElementX: document.documentElement.scrollLeft,
        documentElementY: document.documentElement.scrollTop,
        bodyX: document.body?.scrollLeft ?? 0,
        bodyY: document.body?.scrollTop ?? 0,
        ancestors,
      },
    };
  }, selector);
  let result;
  try {
    result = validateFramingRect(evidence.rect, evidence.visualViewport, label, requireTopAlignment, requireFullVisibility);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(message + "; observed scrollRegion=" + JSON.stringify(evidence.scrollRegion) + "; scroll=" + JSON.stringify(evidence.scroll));
  }
  return { ...evidence, ...result, requireTopAlignment, scrollBlockStart };
}

/** @param {Page} page @param {any} profile @param {any} scene */
async function frameSceneContent(page, profile, scene) {
  if (scene.screen === "replay" && ["share", "publish"].includes(scene.state) && profile.id === "portrait") {
    const isShare = scene.state === "share";
    const panelLabel = isShare ? "Replay sharing" : "Publish replay";
    const panelHeading = isShare ? "Share replay" : "Publish replay";
    const panel = page.locator(`section[aria-label="${panelLabel}"]`);
    const panelFrame = await frameLocatorInViewport(
      page,
      panel,
      `section[aria-label="${panelLabel}"]`,
      `${panelHeading} panel`,
      false,
      true,
      true,
      false,
    );
    const heading = await frameLocatorInViewport(
      page,
      panel.getByRole("heading", { name: panelHeading, exact: true }),
      `section[aria-label="${panelLabel}"] h2`,
      `${panelHeading} heading`,
      false,
      false,
      false,
      true,
    );
    const controls = isShare ? {
      copyLink: await frameLocatorInViewport(
        page,
        panel.getByRole("button", { name: "Copy link", exact: true }),
        'section[aria-label="Replay sharing"] button.uiActionPrimary',
        "Copy link control",
        false,
        false,
        false,
        true,
      ),
      replayLink: await frameLocatorInViewport(
        page,
        panel.getByLabel("Replay link", { exact: true }),
        'section[aria-label="Replay sharing"] label input[readonly]',
        "Replay link field",
        false,
        false,
        false,
        true,
      ),
    } : {
      publicReplayTitle: await frameLocatorInViewport(
        page,
        panel.getByRole("textbox", { name: "Public replay title", exact: true }),
        'section[aria-label="Publish replay"] label input.uiInput',
        "Public replay title field",
        false,
        false,
        false,
        true,
      ),
      publish: await frameLocatorInViewport(
        page,
        panel.getByRole("button", { name: "Publish this replay", exact: true }),
        'section[aria-label="Publish replay"] button.uiActionPrimary',
        "Publish this replay action",
        false,
        false,
        false,
        true,
      ),
    };
    return {
      replayPanel: {
        state: scene.state,
        panel: panelFrame,
        heading,
        controls,
        applied: "actual Replay panel framed within its tagged scroll region; no viewport or source layout changes",
      },
      preserveScroll: true,
    };
  }
  if (scene.screen === "analysis-report" && scene.state === "proof-open") {
    const firstFrame = page.locator("details.analysis-entry").first().locator(".proof-frame").first();
    if (!(await firstFrame.count())) throw new Error("Analysis proof-open state has no first rendered proof frame.");
    return {
      proof: await frameLocatorInViewport(page, firstFrame, "first .proof-frame in first details.analysis-entry", "First proof frame", true, true, true, true),
      applied: "first .proof-frame aligned to visual viewport top",
    };
  }
  if (scene.screen === "shared-replay" && scene.state === "from-profile") {
    const returnButton = page.getByRole("button", { name: "Return to public profile", exact: true });
    return {
      returnToProfile: await frameLocatorInViewport(
        page,
        returnButton,
        'button[aria-label="Return to public profile"]',
        "Return to public profile control",
        false,
        false,
        false,
        true,
      ),
      preserveScroll: true,
      applied: "tagged Return to public profile control measured in its natural header position",
    };
  }
  if (scene.screen === "guide" && scene.state === "examples") {
    const section = page.locator('section[aria-labelledby="examples"]');
    const heading = section.getByRole("heading", { name: "Example replays", exact: true });
    const sectionFrame = await frameLocatorInViewport(page, section, 'section[aria-labelledby="examples"]', "Guide Example replays section", false, true, true);
    const headingFrame = await frameLocatorInViewport(page, heading, 'section[aria-labelledby="examples"] h2', "Guide Example replays heading", false, false, false, true);
    const links = section.locator('ul[aria-label="Example replays"] > li a[aria-label^="Open "]');
    const linkCount = await links.count();
    if (linkCount !== 2) throw new Error("Guide Examples framing expected two real example links, saw " + linkCount + ".");
    const entryFrames = [];
    for (let index = 0; index < linkCount; index++) {
      const row = section.locator('ul[aria-label="Example replays"] > li').nth(index);
      const link = links.nth(index);
      entryFrames.push({
        row: await frameLocatorInViewport(page, row, `example replay row ${index + 1}`, `Guide example row ${index + 1}`, false, false, false, true),
        link: await frameLocatorInViewport(page, link, `example replay link ${index + 1}`, `Guide example link ${index + 1}`, false, false, false, true),
      });
    }
    return {
      examples: { section: sectionFrame, heading: headingFrame, entries: entryFrames },
      preserveScroll: true,
      applied: "tagged Examples navigation link selected; section and both replay rows retained in the capture viewport",
    };
  }
  if (scene.screen === "settings" && scene.state === "custom") {
    const selectedCustom = page.getByRole("button", { name: SETTINGS_CUSTOM_BUTTON_NAME });
    const portrait = profile.id === "portrait";
    const selected = await frameLocatorInViewport(
      page,
      selectedCustom,
      'button[class*="presetCardActive"]',
      "Selected Custom settings card",
      false,
      portrait,
      portrait,
      true,
    );
    const advancedControlLabel = page.getByText(/^Advanced controls$/i).first();
    const advancedControls = await frameLocatorInViewport(
      page,
      advancedControlLabel,
      'text=/^Advanced controls$/i',
      "Advanced Controls section label",
      false,
      false,
      false,
      true,
    );
    const depthLabel = page.getByText("Depth", { exact: true }).first();
    const depthControl = page.getByRole("button", { name: /^D\d+$/ }).first();
    const firstAdvancedControls = {
      depthLabel: await frameLocatorInViewport(page, depthLabel, 'text="Depth"', "First advanced control label", false, false, false, true),
      depthControl: await frameLocatorInViewport(page, depthControl, 'button[name=/^D\\d+$/]:first', "First advanced control", false, false, false, true),
    };
    return {
      customSettings: {
        selectedCard: selected,
        advancedControls,
        firstAdvancedControls,
        applied: portrait ? "selected Custom card aligned to its tagged scroll region; fixed page header retained" : "desktop scroll position retained; Custom and first Advanced Controls measured in viewport",
      },
      preserveScroll: portrait,
    };
  }
  return null;
}

/** @param {any} profile @param {string | undefined} userAgent */
function profileMetadata(profile, userAgent) {
  return {
    id: profile.id,
    viewport: { ...profile.viewport },
    deviceScaleFactor: profile.deviceScaleFactor,
    isMobile: profile.isMobile,
    hasTouch: profile.hasTouch,
    userAgentKind: profile.userAgentKind,
    userAgent,
    locale: "en-US",
    timezoneId: "UTC",
    reducedMotion: "reduce",
    screenshotScale: "css",
    fullPage: false,
    portraitCalibration: profile.id === "portrait" ? "generic viewport; not measured on a physical device" : undefined,
  };
}

/** @param {Browser} browser @param {any} config @param {any} profile @param {any} scene @param {string} outputPath @param {any} manifest @param {any} transport */
async function captureOne(browser, config, profile, scene, outputPath, manifest, transport) {
  const source = config.source;
  const fixture = FIXTURES[config.fixture];
  const userAgent = profile.isMobile ? genericAndroidUserAgent(browser.version()) : undefined;
  /** @type {Record<string, any>} */
  const capture = {
    screen: scene.screen,
    state: scene.state,
    profile: { id: profile.id },
    status: "running",
    file: null,
    errors: { console: [], page: [] },
    network: newNetworkLog(),
    readiness: {},
    renderer: "unknown",
  };
  manifest.captures.push(capture);
  let context;
  /** @type {Page | undefined} */
  let page;
  try {
    const transportContextOptions = transport
      ? await transport.module.historyTransportContextOptions(transport.normalized)
      : {};
    context = await browser.newContext({
      ...(isRecord(transportContextOptions) ? transportContextOptions : {}),
      viewport: { ...profile.viewport },
      deviceScaleFactor: profile.deviceScaleFactor,
      isMobile: profile.isMobile,
      hasTouch: profile.hasTouch,
      ...(userAgent ? { userAgent } : {}),
      locale: "en-US",
      timezoneId: "UTC",
      reducedMotion: "reduce",
      serviceWorkers: "block",
    });
    await seedLocalStorage(context, source.tag, config.fixture, source.origin, {
      emptyHistory: scene.screen === "profile" && scene.state === "empty",
    });
    const network = newNetworkLog();
    capture.network = network;
    await installObservers(context, source, transport, network);
    page = await context.newPage();
    let transportMetadata;
    if (transport) {
      transportMetadata = await transport.module.prepareHistoryContext(context, source, profile, transport.normalized, scene);
      if (transportMetadata !== undefined && !isRecord(transportMetadata)) {
        throw new Error("prepareHistoryContext must return an object when it returns metadata.");
      }
    }
    let routePath = await routePathFor(source, scene, transportMetadata);
    let matchSetup;
    if (scene.screen === "match") matchSetup = await prepareMatchScene(page, source, fixture);
    else await navigateTo(page, source, routePath);
    if (scene.screen === "match") routePath = "/match/local";

    const heading = await waitForHeading(page);
    await prepareSceneState(page, scene, source);
    const content = await assertRouteContent(page, source, scene, transportMetadata);
    const fonts = await waitForFonts(page);
    const sceneFraming = await frameSceneContent(page, profile, scene);
    if (sceneFraming?.proof) content.proof = { ...content.proof, framing: sceneFraming.proof, applied: sceneFraming.applied };
    if (sceneFraming?.examples) content.examples = { ...content.examples, framing: sceneFraming.examples, applied: sceneFraming.applied };
    if (sceneFraming?.customSettings) content.customSettings = sceneFraming.customSettings;
    if (sceneFraming?.replayPanel) content.replayAction = { ...content.replayAction, framing: sceneFraming.replayPanel };
    if (content.assetPage?.requiresFontFaceProof) {
      content.assetPage = validateFontPreviewEvidence(content.assetPage, fonts);
    }
    if (content.assetPage?.requiresReactFontFaceProof) {
      content.assetPage = validateReactVisualEvidence("fonts", content.assetPage, fonts);
      await page.getByRole("heading", { name: "Typography", exact: true }).scrollIntoViewIfNeeded();
      content.assetPage.viewportAnchor = "Typography heading";
    }
    const viewportMetrics = await page.evaluate(() => ({
      innerWidth: window.innerWidth,
      innerHeight: window.innerHeight,
      visualViewportWidth: window.visualViewport?.width ?? null,
      visualViewportHeight: window.visualViewport?.height ?? null,
      visualViewportScale: window.visualViewport?.scale ?? null,
      documentScrollWidth: Math.max(document.documentElement.scrollWidth, document.body?.scrollWidth ?? 0),
      documentScrollHeight: Math.max(document.documentElement.scrollHeight, document.body?.scrollHeight ?? 0),
      devicePixelRatio: window.devicePixelRatio,
      maxTouchPoints: navigator.maxTouchPoints,
      userAgent: navigator.userAgent,
      mobileQueryMatches: matchMedia("(max-width: 720px) and (orientation: portrait) and (hover: none) and (pointer: coarse)").matches,
    }));
    const viewport = { ...viewportMetrics, ...cssViewportDimensions(viewportMetrics) };
    if (viewport.width !== profile.viewport.width || viewport.height !== profile.viewport.height
      || viewport.devicePixelRatio !== 1) {
      throw new Error("Capture viewport or DPR differs from configured profile: " + JSON.stringify(viewport));
    }
    if (profile.isMobile && (!viewport.maxTouchPoints || !viewport.mobileQueryMatches)) {
      throw new Error("Portrait context did not expose generic touch-device media features: " + JSON.stringify(viewport));
    }
    if (scene.screen === "match" || scene.screen === "replay" || scene.screen === "shared-replay") {
      capture.readiness.board = content.board;
      capture.renderer = content.board.rendererObservation ?? "unknown";
    }

    await page.mouse.move(2, 2);
    await page.evaluate(() => new Promise((resolveFrame) => requestAnimationFrame(() => requestAnimationFrame(resolveFrame))));
    const preserveTypographyScroll = scene.screen === "visuals" && scene.state === "fonts" && source.tag >= "v0.5.1";
    const preserveSceneScroll = sceneFraming?.preserveScroll === true || sceneFraming?.proof !== undefined;
    if (!preserveTypographyScroll && !preserveSceneScroll) await page.evaluate(() => window.scrollTo(0, 0));

    const file = routePathForCapture(source.tag, profile.id, scene);
    const filePath = join(outputPath, file);
    await mkdir(dirname(filePath), { recursive: true });
    const png = await page.screenshot({ type: "png", fullPage: false, scale: "css" });
    const dimensions = pngDimensions(png);
    if (dimensions.width !== profile.viewport.width || dimensions.height !== profile.viewport.height) {
      throw new Error("PNG dimensions do not match CSS viewport: " + JSON.stringify(dimensions));
    }
    await writeFile(filePath, png, { flag: "wx" });

    capture.status = "captured";
    capture.file = file;
    capture.width = dimensions.width;
    capture.height = dimensions.height;
    capture.sha256 = sha256(png);
    capture.route = {
      requested: manifestRoute(new URL(routePath, source.baseUrl).href),
      observed: manifestRoute(page.url()),
      heading,
    };
    capture.profile = profileMetadata(profile, viewport.userAgent);
    capture.context = {
      locale: "en-US",
      timezoneId: "UTC",
      deviceScaleFactor: viewport.devicePixelRatio,
      isMobile: profile.isMobile,
      hasTouch: profile.hasTouch,
      maxTouchPoints: viewport.maxTouchPoints,
      reducedMotion: "reduce",
      screenshotScale: "css",
      fullPage: false,
      mobileMediaQuery: viewport.mobileQueryMatches,
      animationNote: "Reduced motion is requested; Phaser animation state is not frozen or assumed deterministic.",
    };
    if (transportMetadata) {
      capture.transport = {
        project: transportMetadata.project,
        session: transportMetadata.session,
        mode: transportMetadata.transport,
        responseSource: transportMetadata.responseSource,
        rulesSha256: transportMetadata.rulesSha256,
        publicFixture: transportMetadata.publicFixture,
        optionalBootstrap: transportMetadata.optionalBootstrap ?? null,
        routingAttestation: transport.normalized.mode === "legacy-proxy"
          ? "primary transport returned successfully after the reserved .invalid browser-proxy probe; probe is intercepted and not forwarded"
          : "primary transport approved the exact owned native-emulator service URLs",
        addressNote: "SDK service URLs are permitted only when the primary-owned adapter recognizes them and routes them through the owned proxy or emulators.",
      };
    }
    capture.readiness = {
      heading,
      fonts,
      fixture: content.fixture ?? null,
      match: content.match ?? null,
      replay: content.replay ?? null,
      analysis: content.analysis ?? null,
      customSettings: content.customSettings ?? null,
      reportView: content.reportView ?? null,
      proof: content.proof ?? null,
      framing: sceneFraming ?? null,
      assetPage: content.assetPage ?? null,
      examples: content.examples ?? null,
      publicProfileUid: content.publicProfileUid ?? null,
      publicCatalogRows: content.publicCatalogRows ?? null,
      statsVisible: content.statsVisible ?? null,
      portableReplay: content.portableReplay ?? null,
      returnToProfile: content.returnToProfile ?? null,
      returnToProfileControl: content.returnToProfileControl ?? null,
      publicSettings: content.publicSettings ?? null,
      savedReplays: content.savedReplays ?? null,
      replayAction: content.replayAction ?? null,
      sharedReplayContext: content.sharedReplayContext ?? null,
      matchSetup: matchSetup ?? null,
      pointer: "moved to 2,2 before screenshot",
      viewport,
    };
    capture.errors = {
      console: network.consoleErrors,
      page: network.pageErrors,
      expectedBootstrapBlocked: network.expectedBootstrapBlocked,
      expectedTransportCancellations: network.expectedTransportCancellations,
    };
    if (network.forbidden.length) throw new Error("Forbidden network requests were blocked: " + JSON.stringify(network.forbidden.slice(0, 5)));
    if (network.failed.length) throw new Error("Unexpected failed network requests observed: " + JSON.stringify(network.failed.slice(0, 5)));
    if (network.consoleErrors.length) throw new Error("Browser console errors observed: " + JSON.stringify(network.consoleErrors.slice(0, 3)));
    if (network.pageErrors.length) throw new Error("Browser page errors observed: " + JSON.stringify(network.pageErrors.slice(0, 3)));
    return capture;
  } catch (error) {
    capture.status = "failed";
    /** @type {{route:string,testIds:Array<{id:string|null,text:string}>}} */
    let observed = { route: "unavailable", testIds: [] };
    if (page) {
      try {
        observed = await page.evaluate(() => ({
          route: location.pathname + (location.search ? "?[redacted]" : "") + (location.hash ? "#[portable-fragment]" : ""),
          testIds: [...document.querySelectorAll("[data-testid]")].slice(0, 40).map((element) => ({
            id: element.getAttribute("data-testid"),
            text: (element.textContent ?? "").replace(/\s+/g, " ").trim().slice(0, 180),
          })),
        }));
        observed.testIds = observed.testIds.map(({ id, text }) => ({ id, text: redactText(text) }));
      } catch {
        // Keep the original capture failure when the page no longer evaluates.
      }
    }
    capture.failureContext = observed;
    capture.error = source.tag + "/" + profile.id + "/" + scene.screen + "--" + scene.state + ": "
      + safeError(error, [source.worktree, repositoryRoot]) + "; observed=" + JSON.stringify(observed).slice(0, 2600);
    if (capture.network) {
      capture.errors = {
        console: capture.network.consoleErrors ?? [],
        page: capture.network.pageErrors ?? [],
        expectedBootstrapBlocked: capture.network.expectedBootstrapBlocked ?? {
          requests: [],
          requestFailures: [],
          consoleErrors: [],
        },
      };
    }
    throw new Error(capture.error, { cause: error });
  } finally {
    if (context) await context.close().catch(() => undefined);
  }
}

/** @param {string} tag @param {string} profile @param {any} scene */
function routePathForCapture(tag, profile, scene) {
  const path = [tag, profile, scene.screen + "--" + scene.state].join("/");
  if (path.split("/").some((part) => part === ".." || part === "." || !/^[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(part))) {
    throw new Error("Unsafe screenshot output path.");
  }
  return path + ".png";
}

/** @param {string} raw */
function manifestRoute(raw) {
  const url = new URL(raw);
  return url.pathname + (url.search ? "?[redacted]" : "") + (url.hash ? "#[portable-fragment]" : "");
}

/** @param {string} outputPath @param {any} config @param {any} manifest */
async function verifyCaptureFiles(outputPath, config, manifest) {
  const expectedCount = config.scenes.length * config.profiles.length;
  if (manifest.captures.length !== expectedCount || manifest.captures.some((/** @type {any} */ capture) => capture.status !== "captured")) {
    throw new Error("Capture manifest is incomplete: expected " + expectedCount + " successful screenshot records, got " + manifest.captures.length + ".");
  }
  validateManifestForWrite({ ...manifest, status: "complete" });
  for (const capture of manifest.captures) {
    const filePath = resolve(outputPath, capture.file);
    if (!pathOverlaps(filePath, outputPath) || relative(outputPath, filePath).startsWith("..")) {
      throw new Error("Capture filename escapes the new output directory: " + capture.file);
    }
    const info = await lstat(filePath);
    if (!info.isFile() || info.isSymbolicLink()) throw new Error("Capture output is not a regular PNG: " + capture.file);
    const bytes = await readFile(filePath);
    if (sha256(bytes) !== capture.sha256) throw new Error("Capture PNG hash mismatch: " + capture.file);
    const dimensions = pngDimensions(bytes);
    if (dimensions.width !== capture.width || dimensions.height !== capture.height) {
      throw new Error("Capture PNG dimensions changed after writing: " + capture.file);
    }
  }
}

/** @param {any} source */
function sourceIdentity(source) {
  return JSON.stringify({
    commit: source.commit,
    sourceTree: source.sourceTree,
    applicationTree: source.applicationTree,
    packageVersion: source.packageVersion,
    packageLockSha256: source.packageLockSha256,
    cargoLock: source.cargoLock,
    wasmSha256: source.wasmSha256,
    distSha256: source.distSha256,
    indexSha256: source.indexSha256,
    reportInputs: source.reportInputs,
  });
}

/** @param {any} config */
function manifestFixture(config) {
  const fixture = FIXTURES[config.fixture];
  const seed = localStorageSeed(config.source.tag, config.fixture);
  const emptyProfileStates = config.scenes
    .filter((/** @type {any} */ scene) => scene.screen === "profile" && scene.state === "empty")
    .map(() => {
      const emptySeed = localStorageSeed(config.source.tag, config.fixture, { emptyHistory: true });
      return {
        screen: "profile",
        state: "empty",
        localStorageKey: emptySeed.key,
        localHistorySchema: config.source.adapter,
        replayMatches: 0,
        summaryMatches: 0,
        presentation: config.source.tag === "v0.5.4"
          ? "No matches yet; Finish a game to inspect it here; Play a game"
          : "tagged blank history area; empty-state copy not yet present",
      };
    });
  return {
    id: config.fixture,
    identity: fixture.displayName,
    profileId: fixture.profileId,
    matchId: fixture.id,
    savedAt: fixture.savedAt,
    ruleset: fixture.variant,
    mode: "local human Black vs tagged default bot White",
    botName: expectedBotName(config.source.tag),
    status: "completed; Black win",
    moveCells: [...fixture.moveCells],
    moveCount: fixture.moveCells.length,
    localStorageKey: seed.key,
    additionalLocalStorageKeys: (seed.additional ?? []).map((/** @type {{key:string}} */ item) => item.key),
    localStorageSchema: config.source.adapter,
    additionalProfileStates: emptyProfileStates,
    matchCapture: config.source.tag === "v0.2.1"
      ? "fresh human-to-move Match because Replay branching is not present"
      : "Replay final ply 9, Previous once to ply 8, Play From Here; Black human idle",
  };
}

/** @param {any} config @returns {Record<string, any>} */
function initialManifest(config) {
  return {
    schemaVersion: 1,
    kind: "historical-ui-capture",
    status: "running",
    capturedAt: new Date().toISOString(),
    captureTool: {
      filename: basename(scriptPath),
      sha256: null,
      version: 1,
    },
    tools: {
      node: process.version,
      platform: process.platform,
      architecture: process.arch,
      browser: null,
    },
    sources: [{
      tag: config.source.tag,
      commit: config.source.expectedCommit,
      adapter: config.source.adapter,
      buildReceipt: {
        recordedAt: config.source.buildReceipt.recordedAt,
        sourceTree: config.source.buildReceipt.sourceTree,
        applicationTree: config.source.buildReceipt.applicationTree,
        packageLockSha256: config.source.buildReceipt.packageLockSha256,
        cargoLock: config.source.buildReceipt.cargoLock,
        wasmSha256: config.source.buildReceipt.wasmSha256,
        distSha256: config.source.buildReceipt.distSha256,
        indexSha256: config.source.buildReceipt.indexSha256,
        environment: sanitizeEnvironment(config.source.buildReceipt.environment),
      },
      verification: { status: "pending" },
    }],
    fixture: manifestFixture(config),
    profiles: config.profiles.map((/** @type {string} */ id) => ({ ...CAPTURE_PROFILES[/** @type {keyof typeof CAPTURE_PROFILES} */ (id)] })),
    transport: config.transport ? {
      mode: config.transport.mode,
      session: config.transport.session,
      scope: "synthetic demo Auth/Firestore only; no real accounts or production data",
    } : { mode: "none", scope: "tagged local fixture; all identity/data/external requests blocked" },
    limitations: [
      "These screenshots recreate the tagged source layout; they do not recover the historical browser or device runtime.",
      "Portrait is a generic Android touch context at 412×800 DPR1, not a measured OnePlus viewport.",
      "Reduced motion is requested; Phaser animation is not frozen or assumed deterministic.",
    ],
    captures: [],
    errors: [],
  };
}

/** @param {any} config @param {any} inspected */
async function createReadyManifestSource(config, inspected) {
  const publicInspection = inspected;
  return {
    ...sanitizedManifestSource(config.source, inspected),
    commit: publicInspection.commit,
    sourceTree: publicInspection.sourceTree,
    applicationTree: publicInspection.applicationTree,
    packageVersion: publicInspection.packageVersion,
    packageLockSha256: publicInspection.packageLockSha256,
    cargoLock: publicInspection.cargoLock,
    wasmSha256: publicInspection.wasmSha256,
    distSha256: publicInspection.distSha256,
    indexSha256: publicInspection.indexSha256,
    reportInputs: publicInspection.reportInputs,
    distFiles: publicInspection.files,
    verification: {
      status: "passed",
      sourceClean: inspected.clean,
      exactTagCommit: inspected.commit === config.source.expectedCommit,
      servedIndexMatches: true,
      servedDistFiles: inspected.files.length,
      allServedFilesMatchReceipt: true,
    },
  };
}

/** @param {string[]} args */
async function runCli(args) {
  const cli = parseArguments(args);
  if (cli.help) {
    process.stdout.write(usage() + "\n");
    return 0;
  }
  if (!cli.configPath || !cli.outputPath) throw new Error("Both --config and --output are required.");
  const configPath = resolve(cli.configPath);
  const configInfo = await lstat(configPath);
  if (!configInfo.isFile() || configInfo.isSymbolicLink()) throw new Error("Config must be a regular non-symlink file.");
  const input = JSON.parse(await readFile(configPath, "utf8"));
  const config = validateHistoryCaptureConfig(input);
  const outputPath = await validateOutputPath(cli.outputPath, [
    repositoryRoot,
    config.source.worktree,
    configPath,
  ]);

  if (cli.validateOnly) {
    const inspected = await inspectSource(config);
    const served = await verifyServedBytes(config, inspected);
    const confirmation = await inspectSource(config);
    if (sourceIdentity(inspected) !== sourceIdentity(confirmation)) {
      throw new Error(config.source.tag + " source or build artifacts changed during validate-only inspection.");
    }
    const servedAfter = await verifyServedBytes(config, confirmation);
    if (served.filesChecked !== servedAfter.filesChecked) {
      throw new Error(config.source.tag + " preview file inventory changed during validate-only inspection.");
    }
    process.stdout.write(JSON.stringify({
      status: "valid",
      tag: config.source.tag,
      commit: inspected.commit,
      sourceTree: inspected.sourceTree,
      applicationTree: inspected.applicationTree,
      adapter: config.source.adapter,
      packageVersion: inspected.packageVersion,
      distFiles: served.filesChecked,
      servedByteChecks: 2,
      profiles: config.profiles,
      scenes: config.scenes,
      outputPath: "new external directory validated; no files written",
    }, null, 2) + "\n");
    return 0;
  }

  await mkdir(outputPath, { recursive: false });
  const manifest = initialManifest(config);
  manifest.captureTool.sha256 = await sha256File(scriptPath);
  await writeManifest(outputPath, manifest);
  let browser;
  let statusCode = 0;
  let inspectedBefore;
  try {
    inspectedBefore = await inspectSource(config);
    const served = await verifyServedBytes(config, inspectedBefore);
    const readySource = await createReadyManifestSource(config, inspectedBefore);
    readySource.verification.servedDistFiles = served.filesChecked;
    manifest.sources[0] = readySource;
    await writeManifest(outputPath, manifest);

    const transport = await loadTransport(config);
    const transportLaunchOptions = transport
      ? await transport.module.historyTransportLaunchOptions(transport.normalized)
      : {};
    if (transportLaunchOptions !== undefined && !isRecord(transportLaunchOptions)) {
      throw new Error("historyTransportLaunchOptions must return an object.");
    }
    browser = await chromium.launch({
      ...(transportLaunchOptions ?? {}),
      headless: true,
    });
    manifest.tools.browser = { name: "Chromium", version: browser.version() };
    await writeManifest(outputPath, manifest);

    for (const profileId of config.profiles) {
      const profile = CAPTURE_PROFILES[/** @type {keyof typeof CAPTURE_PROFILES} */ (profileId)];
      for (const scene of config.scenes) {
        await captureOne(browser, config, profile, scene, outputPath, manifest, transport);
        await writeManifest(outputPath, manifest);
      }
    }

    const inspectedAfter = await inspectSource(config);
    if (sourceIdentity(inspectedBefore) !== sourceIdentity(inspectedAfter)) {
      throw new Error(config.source.tag + " source or build artifacts changed during capture.");
    }
    manifest.sources[0].verification.sourceCleanAfterCapture = inspectedAfter.clean;
    const servedAfter = await verifyServedBytes(config, inspectedAfter);
    manifest.sources[0].verification.servedDistFilesAfter = servedAfter.filesChecked;
    await verifyCaptureFiles(outputPath, config, manifest);
    manifest.status = "complete";
    manifest.completedAt = new Date().toISOString();
  } catch (error) {
    statusCode = 1;
    manifest.status = "failed";
    manifest.failedAt = new Date().toISOString();
    manifest.errors.push({ message: safeError(error, [config.source.worktree, repositoryRoot]) });
    const sourceRecord = manifest.sources[0];
    if (sourceRecord && sourceRecord.verification?.status === "pending") {
      sourceRecord.verification = { status: "failed", reason: safeError(error, [config.source.worktree, repositoryRoot]) };
    }
  } finally {
    if (browser) {
      try { await browser.close(); }
      catch (error) {
        statusCode = 1;
        manifest.status = "failed";
        manifest.errors.push({ message: "Browser close failed: " + safeError(error, [config.source.worktree, repositoryRoot]) });
      }
    }
  }

  await writeManifest(outputPath, manifest);
  process.stdout.write(JSON.stringify({
    status: manifest.status,
    tag: config.source.tag,
    captures: manifest.captures.filter((/** @type {any} */ capture) => capture.status === "captured").length,
    failedCaptures: manifest.captures.filter((/** @type {any} */ capture) => capture.status === "failed").length,
    output: "external capture directory retained",
    error: manifest.errors.at(-1)?.message ?? null,
  }, null, 2) + "\n");
  return statusCode;
}

if (process.argv[1] && resolve(process.argv[1]) === scriptPath) {
  runCli(process.argv.slice(2)).then((status) => {
    process.exitCode = status;
  }).catch((error) => {
    process.stderr.write("History capture failed before output initialization: " + safeError(error, [repositoryRoot]) + "\n");
    process.exitCode = 1;
  });
}





