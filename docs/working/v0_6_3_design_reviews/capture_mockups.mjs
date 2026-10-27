#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../../..");
const mediaDir = path.join(here, "media");
const requireFromWeb = createRequire(path.join(repo, "gomoku-web/package.json"));
const { chromium } = requireFromWeb("playwright");
const playwrightVersion = JSON.parse(fs.readFileSync(
  path.join(repo, "gomoku-web/node_modules/playwright/package.json"), "utf8",
)).version;

const reportSnapshots = [
  {
    key: "original-abc",
    receiptPath: "docs/working/v0_6_2_design/evidence.json",
    exportPath: "gomoku-web/test-results/v062-design/export",
    originNote: "Frozen A/B/C profile prototype report.",
  },
  {
    key: "minimal-two-column",
    receiptPath: "docs/working/v0_6_2_design/refinement_evidence.json",
    exportPath: "gomoku-web/test-results/v062-design-refined/export",
    originNote: "Frozen simple identity plus record-and-games two-column refinement.",
  },
  {
    key: "compact-player-card",
    receiptPath: "docs/working/v0_6_2_design/player_card_evidence.json",
    exportPath: "gomoku-web/test-results/v062-player-card/export",
    originNote: "Frozen public player-card refinement with original minimal-panel comparison.",
  },
];

const capturePlan = [
  { id: "profile-mockup-a", reportKey: "original-abc", option: "A", layout: "a" },
  { id: "profile-mockup-b", reportKey: "original-abc", option: "B", layout: "b" },
  { id: "profile-mockup-c", reportKey: "original-abc", option: "C", layout: "c" },
  { id: "profile-mockup-minimal-two-column", reportKey: "minimal-two-column", option: "minimal two-column refinement", layout: "split" },
  { id: "profile-mockup-compact-player-card", reportKey: "compact-player-card", option: "compact player-card refinement", layout: "split", sidebar: "card" },
];

if (fs.existsSync(path.join(here, "media_manifest.json")) || capturePlan.some(
  (capture) => fs.existsSync(path.join(mediaDir, `${capture.id}.png`)),
)) {
  throw new Error("This evidence is frozen. Prepare a new evidence directory for a later capture.");
}

const originalReplayPngs = [
  {
    filename: "replay-046-active-desktop.png",
    sourcePath: "docs/assets/screenshot_v0_4_6_replay_desktop.png",
    blobOid: "6b366913a2978b189bb00276541e0309cdd1ee35",
    bytes: 17366,
    sha256: "8fb0ade3cb9a4e85366e43f75553dadd60fbd54ee3ac5fbcc22f3e8de80c0b79",
  },
  {
    filename: "replay-046-active-mobile.png",
    sourcePath: "docs/assets/screenshot_v0_4_6_replay_mobile.png",
    blobOid: "bc9b4adda2bda9a152a2059a63b9029608aede0b",
    bytes: 8798,
    sha256: "e32fdf2c05c36ffa2eb90cb7a7dfadd56d733d4d114fb1911dfa0b306015a8fd",
  },
  {
    filename: "replay-047-active-desktop.png",
    sourcePath: "docs/assets/screenshot_v0_4_7_replay_desktop.png",
    blobOid: "f0b41caf0862839e4094ee63e4db3a6d85466e71",
    bytes: 16568,
    sha256: "67a299ab97dd84b1c30cf9b6462f40bb0c98a107ddd1c060524db45f19aac5cd",
  },
  {
    filename: "replay-047-active-mobile.png",
    sourcePath: "docs/assets/screenshot_v0_4_7_replay_mobile.png",
    blobOid: "297cb337b483a25f65d96541ebbb7b78175f939b",
    bytes: 8996,
    sha256: "546114f130898979ef06068d765c0a62f0cc8eb528a72ba502ca501ce473077f",
  },
].map((item) => ({ ...item, sourceCommit: "0b93fed092975c172959343a45decc9c8c89fe2e" }));

function jsonAt(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(repo, relativePath), "utf8"));
}

function sha256(buffer) {
  return crypto.createHash("sha256").update(buffer).digest("hex");
}

function pngDimensions(buffer) {
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  if (buffer.length < 24 || !buffer.subarray(0, 8).equals(signature) || buffer.toString("ascii", 12, 16) !== "IHDR") {
    throw new Error("Output is not a readable PNG with an IHDR chunk.");
  }
  return { width: buffer.readUInt32BE(16), height: buffer.readUInt32BE(20) };
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function localInventory(root) {
  const files = [];
  async function walk(directory, prefix = "") {
    const entries = await fs.promises.readdir(directory, { withFileTypes: true });
    entries.sort((a, b) => a.name.localeCompare(b.name));
    for (const entry of entries) {
      const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
      const fullPath = path.join(directory, entry.name);
      if (entry.isSymbolicLink()) throw new Error(`Unexpected symlink in frozen export: ${fullPath}`);
      if (entry.isDirectory()) {
        await walk(fullPath, relative);
      } else if (entry.isFile()) {
        const contents = await fs.promises.readFile(fullPath);
        files.push({ path: relative, bytes: contents.length, sha256: sha256(contents) });
      } else {
        throw new Error(`Unsupported export entry: ${fullPath}`);
      }
    }
  }
  await walk(root);
  return summarizeInventory(files);
}

function summarizeInventory(files) {
  const sorted = [...files].sort((a, b) => a.path.localeCompare(b.path));
  return {
    fileCount: sorted.length,
    totalBytes: sorted.reduce((total, file) => total + file.bytes, 0),
    files: sorted,
  };
}

function expectedInventory(receipt) {
  const expected = receipt.served_files;
  assert(Array.isArray(expected), "Receipt has no served_files array.");
  return summarizeInventory(expected.map((file) => ({
    path: file.path,
    bytes: file.bytes,
    sha256: file.sha256,
  })));
}

function inventoryMatches(expected, actual) {
  return JSON.stringify(expected) === JSON.stringify(summarizeInventory(actual.files));
}

function receiptSummary(receipt) {
  const publication = receipt.publication || {};
  return {
    fileCount: receipt.file_count ?? publication.file_count,
    totalBytes: receipt.byte_count ?? publication.byte_count,
    reportId: receipt.report_id ?? publication.report_id,
    url: receipt.url ?? publication.url,
    baselineCommit: receipt.baseline_commit ?? null,
    date: receipt.date ?? null,
    status: receipt.status ?? null,
  };
}

function reportBaseUrl(reportId) {
  return `http://starship:8787/reports/${reportId}/`;
}

async function servedInventory(reportId, files) {
  const base = reportBaseUrl(reportId);
  const results = [];
  for (const file of files) {
    const relativeUrl = file.path.split("/").map(encodeURIComponent).join("/");
    const response = await fetch(new URL(relativeUrl, base), { signal: AbortSignal.timeout(15000) });
    const body = Buffer.from(await response.arrayBuffer());
    assert(response.status === 200, `Frozen report file returned HTTP ${response.status}: ${file.path}`);
    results.push({
      path: file.path,
      bytes: body.length,
      sha256: sha256(body),
      httpStatus: response.status,
      lastModified: response.headers.get("last-modified"),
    });
  }
  return summarizeInventory(results);
}

function sameInventory(left, right) {
  const normalize = (inventory) => inventory.files.map(({ path: filePath, bytes, sha256: digest }) => ({
    path: filePath,
    bytes,
    sha256: digest,
  }));
  return JSON.stringify(normalize(left)) === JSON.stringify(normalize(right));
}

async function sourceSnapshot(snapshot, timing) {
  const receipt = jsonAt(snapshot.receiptPath);
  const summary = receiptSummary(receipt);
  const expected = expectedInventory(receipt);
  assert(summary.reportId, `No report ID in ${snapshot.receiptPath}`);
  assert(expected.fileCount === summary.fileCount, `Receipt file count differs from served_files in ${snapshot.receiptPath}.`);
  assert(expected.totalBytes === summary.totalBytes, `Receipt byte count differs from served_files in ${snapshot.receiptPath}.`);
  const exportRoot = path.join(repo, snapshot.exportPath);
  const local = await localInventory(exportRoot);
  const served = await servedInventory(summary.reportId, expected.files);
  const localMatchesReceipt = sameInventory(local, expected);
  const servedMatchesReceipt = sameInventory(served, expected);
  const localMatchesServed = sameInventory(local, served);
  return {
    reportKey: snapshot.key,
    originNote: snapshot.originNote,
    receiptPath: snapshot.receiptPath,
    exportPath: snapshot.exportPath,
    reportId: summary.reportId,
    reportUrl: summary.url || reportBaseUrl(summary.reportId),
    baselineCommit: summary.baselineCommit,
    sourceDate: summary.date,
    receiptStatus: summary.status,
    receiptFileCount: summary.fileCount,
    receiptByteCount: summary.totalBytes,
    receiptInventory: expected,
    localExport: local,
    servedSnapshot: served,
    matchesReceipt: localMatchesReceipt && servedMatchesReceipt,
    localMatchesServed,
    capturedAtStage: timing,
  };
}

async function inspectFrozenPrototype(page, expected) {
  await page.evaluate(async () => {
    await document.fonts.ready;
    await Promise.all(Array.from(document.images, (image) => image.decode().catch(() => undefined)));
    await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
  });
  await page.waitForFunction(() => document.querySelectorAll(".game-row").length === 8, undefined, { timeout: 10000 });
  const state = await page.evaluate(() => {
    const app = document.querySelector("#app");
    const layout = app?.firstElementChild?.className || "";
    const rows = document.querySelectorAll(".game-row").length;
    const pageIndicator = document.querySelector(".page-indicator")?.textContent?.trim() || null;
    const gameCount = document.querySelector(".game-count")?.textContent?.trim() || null;
    const stats = Array.from(document.querySelectorAll(".stats")).some((element) => {
      const style = getComputedStyle(element);
      return style.display !== "none" && style.visibility !== "hidden";
    });
    return {
      url: location.href,
      title: document.title,
      layoutClass: layout,
      studyChromeHidden: document.querySelector("#study-chrome")?.hidden ?? false,
      playerNamePresent: document.body.innerText.includes("Juniper"),
      gameRowsVisible: rows,
      pageIndicator,
      gameCount,
      sharedStatsVisible: stats,
      thumbnailCount: document.querySelectorAll(".thumbnail").length,
      longLabelFixturePresent: document.body.innerText.includes("棋盤の旅人"),
      fontSize: getComputedStyle(document.documentElement).fontSize,
      fontsStatus: document.fonts.status,
      pixelOperatorAvailable: document.fonts.check('16px "PixelOperator"'),
      imageStates: Array.from(document.images, (image) => ({ complete: image.complete, naturalWidth: image.naturalWidth })),
    };
  });
  assert(state.layoutClass.split(/\s+/).includes(`layout-${expected.layout}`), `Selected layout is absent: ${JSON.stringify(state)}`);
  assert(state.studyChromeHidden, "Prototype standalone chrome was visible; capture may include report controls.");
  assert(state.playerNamePresent, "Expected fictional profile name was not rendered.");
  assert(state.gameRowsVisible === 8 && state.pageIndicator === "1 / 4", `Expected first page of 32 games: ${JSON.stringify(state)}`);
  assert(state.sharedStatsVisible, "Shared statistics are not visible.");
  assert(state.thumbnailCount === 0, "Thumbnails are unexpectedly enabled.");
  assert(!state.longLabelFixturePresent, "Long-label test data is unexpectedly enabled.");
  assert(state.fontSize === "16px" && state.fontsStatus === "loaded" && state.pixelOperatorAvailable, "Expected 100% text/font state was not ready.");
  if (expected.sidebar === "card") {
    assert(await page.locator(".player-card").count() === 1, "The compact player-card selector did not take effect.");
  }
  return state;
}

function makeCaptureUrl(reportId, spec) {
  const url = new URL("prototype.html", reportBaseUrl(reportId));
  const params = new URLSearchParams();
  params.set("layout", spec.layout);
  params.set("surface", "visitor");
  params.set("scenario", "full");
  params.set("stats", "on");
  params.set("labels", "normal");
  params.set("text", "100");
  params.set("thumbs", "off");
  if (spec.sidebar) params.set("sidebar", spec.sidebar);
  params.set("embed", "1");
  url.search = params.toString();
  return url.toString();
}

async function captureOne(browser, snapshot, spec) {
  const receipt = jsonAt(snapshot.receiptPath);
  const reportId = receipt.report_id ?? receipt.publication?.report_id;
  const base = reportBaseUrl(reportId);
  const allowedPaths = new Set(receipt.served_files.map((file) => `/${"reports"}/${reportId}/${file.path}`));
  const url = makeCaptureUrl(reportId, spec);
  const context = await browser.newContext({
    viewport: { width: 1280, height: 900 },
    deviceScaleFactor: 1,
    reducedMotion: "reduce",
    colorScheme: "dark",
  });
  const page = await context.newPage();
  const requestLog = [];
  const requestsByObject = new Map();
  const blockedRequests = [];
  const errors = { console: [], page: [], requestFailures: [] };
  page.on("request", (request) => {
    const entry = { url: request.url(), method: request.method(), resourceType: request.resourceType() };
    requestLog.push(entry);
    requestsByObject.set(request, entry);
  });
  page.on("response", (response) => {
    const entry = requestsByObject.get(response.request());
    if (entry) entry.httpStatus = response.status();
  });
  page.on("requestfailed", (request) => {
    const entry = requestsByObject.get(request);
    const failure = request.failure()?.errorText || "request failed";
    if (entry) entry.failure = failure;
    errors.requestFailures.push({ url: request.url(), failure });
  });
  page.on("console", (message) => {
    if (message.type() === "error") errors.console.push(message.text());
  });
  page.on("pageerror", (error) => errors.page.push(error.message));
  await page.route("**/*", async (route) => {
    const requestUrl = new URL(route.request().url());
    const allowed = requestUrl.origin === new URL(base).origin && allowedPaths.has(requestUrl.pathname);
    if (allowed) {
      await route.continue();
    } else {
      const blocked = {
        url: requestUrl.toString(),
        reason: "outside the frozen report file inventory; no external/backend requests allowed",
        resourceType: route.request().resourceType(),
      };
      blockedRequests.push(blocked);
      const entry = requestsByObject.get(route.request());
      if (entry) entry.blocked = true;
      await route.abort("blockedbyclient");
    }
  });
  const navigation = await page.goto(url, { waitUntil: "networkidle", timeout: 30000 });
  assert(navigation?.status() === 200, `Prototype navigation returned ${navigation?.status()}`);
  const state = await inspectFrozenPrototype(page, spec);
  const captureAt = new Date().toISOString();
  const filename = `${spec.id}.png`;
  const imagePath = path.join(mediaDir, filename);
  await page.screenshot({ path: imagePath, fullPage: true, animations: "disabled", caret: "hide" });
  const image = await fs.promises.readFile(imagePath);
  const dimensions = pngDimensions(image);
  assert(dimensions.width === 1280, `Expected 1280px screenshot width, got ${dimensions.width}.`);
  await context.close();
  return {
    id: spec.id,
    option: spec.option,
    reportId,
    reportReceiptPath: snapshot.receiptPath,
    sourceExportPath: snapshot.exportPath,
    origin: "design mockup",
    runtimeProductCapture: false,
    humanAcceptanceEvidence: false,
    url,
    selection: {
      layout: spec.layout,
      sidebar: spec.sidebar || null,
      surface: "visitor",
      scenario: "full",
      gamesInFixture: 32,
      sharedStats: true,
      labels: "normal",
      textScalePercent: 100,
      thumbnails: false,
      embed: true,
      page: 1,
      pageCount: 4,
    },
    viewport: { width: 1280, height: 900, deviceScaleFactor: 1, fullPage: true, reducedMotion: "reduce", colorScheme: "dark" },
    captureAt,
    browser: { engine: "Chromium", version: browser.version(), playwrightVersion },
    renderedState: state,
    image: { path: `docs/working/v0_6_3_design_reviews/media/${filename}`, bytes: image.length, sha256: sha256(image), ...dimensions },
    requests: requestLog,
    blockedRequests,
    errors,
  };
}

async function originalReplayMetadata() {
  const sourceCommit = "0b93fed092975c172959343a45decc9c8c89fe2e";
  const results = [];
  for (const item of originalReplayPngs) {
    const imagePath = path.join(mediaDir, item.filename);
    const image = await fs.promises.readFile(imagePath);
    const dimensions = pngDimensions(image);
    const digest = sha256(image);
    assert(image.length === item.bytes, `Historical PNG byte count mismatch for ${item.filename}.`);
    assert(digest === item.sha256, `Historical PNG SHA256 mismatch for ${item.filename}.`);
    results.push({
      path: `docs/working/v0_6_3_design_reviews/media/${item.filename}`,
      origin: "historical",
      sourceCommit,
      sourcePath: item.sourcePath,
      sourceBlobOid: item.blobOid,
      verifiedCurrentBlobOid: item.blobOid,
      bytes: image.length,
      sha256: digest,
      ...dimensions,
      runtimeOrCaptureIdentity: "not inferred from Git blob metadata; source review captions qualify the active last-escape frame only",
    });
  }
  return results;
}

async function main() {
  await fs.promises.mkdir(mediaDir, { recursive: true });
  const manifest = {
    schemaVersion: 1,
    scope: "Historical replay PNG recovery and bounded frozen public-profile mockup captures.",
    captureClassification: "All five new public-profile images are design mockups from frozen prototype reports, not runtime product captures or human acceptance evidence.",
    captureScript: "docs/working/v0_6_3_design_reviews/capture_mockups.mjs",
    tooling: {
      playwrightPackage: `playwright@${playwrightVersion}`,
      dockerImage: "mcr.microsoft.com/playwright:v1.59.1-noble",
      dockerImageId: "sha256:fc1610b07935476ac20219b7911f6bbe47ac861523635845ba0bc7cc5f0277c4",
      browserEngine: "Chromium",
    },
    historicalReplayPngs: [],
    sourceSnapshots: [],
    captures: [],
    acceptance: {
      allReportsMatchReceiptBeforeAndAfter: false,
      sourceFilesUnchangedAcrossCapture: false,
      allCapturesPassed: false,
      noBlockedOrExternalRequests: false,
      allErrorsClear: false,
    },
  };
  const browser = await chromium.launch({ headless: true });
  try {
    manifest.tooling.browserVersion = browser.version();
    manifest.historicalReplayPngs = await originalReplayMetadata();

    const before = [];
    for (const snapshot of reportSnapshots) before.push(await sourceSnapshot(snapshot, "before"));
    for (const snapshot of before) {
      assert(snapshot.matchesReceipt && snapshot.localMatchesServed, `Frozen source mismatch before capture: ${snapshot.reportKey}`);
    }

    const snapshotsByKey = new Map(reportSnapshots.map((snapshot, index) => [snapshot.key, before[index]]));
    for (const spec of capturePlan) {
      manifest.captures.push(await captureOne(browser, snapshotsByKey.get(spec.reportKey), spec));
    }

    const after = [];
    for (const snapshot of reportSnapshots) after.push(await sourceSnapshot(snapshot, "after"));
    manifest.sourceSnapshots = before.map((snapshot, index) => {
      const later = after[index];
      return {
        reportKey: snapshot.reportKey,
        originNote: snapshot.originNote,
        receiptPath: snapshot.receiptPath,
        exportPath: snapshot.exportPath,
        reportId: snapshot.reportId,
        reportUrl: snapshot.reportUrl,
        baselineCommit: snapshot.baselineCommit,
        sourceDate: snapshot.sourceDate,
        receiptStatus: snapshot.receiptStatus,
        receiptFileCount: snapshot.receiptFileCount,
        receiptByteCount: snapshot.receiptByteCount,
        receiptInventory: snapshot.receiptInventory,
        before: { localExport: snapshot.localExport, servedSnapshot: snapshot.servedSnapshot, matchesReceipt: snapshot.matchesReceipt, localMatchesServed: snapshot.localMatchesServed },
        after: { localExport: later.localExport, servedSnapshot: later.servedSnapshot, matchesReceipt: later.matchesReceipt, localMatchesServed: later.localMatchesServed },
        unchangedDuringCapture: sameInventory(snapshot.localExport, later.localExport) && sameInventory(snapshot.servedSnapshot, later.servedSnapshot),
      };
    });
    manifest.acceptance.allReportsMatchReceiptBeforeAndAfter = manifest.sourceSnapshots.every((snapshot) => snapshot.before.matchesReceipt && snapshot.after.matchesReceipt && snapshot.before.localMatchesServed && snapshot.after.localMatchesServed);
    manifest.acceptance.sourceFilesUnchangedAcrossCapture = manifest.sourceSnapshots.every((snapshot) => snapshot.unchangedDuringCapture);
    manifest.acceptance.allCapturesPassed = manifest.captures.length === 5 && manifest.captures.every((capture) => capture.image.width === 1280 && capture.image.height >= 900 && capture.renderedState.gameRowsVisible === 8);
    manifest.acceptance.noBlockedOrExternalRequests = manifest.captures.every((capture) => capture.blockedRequests.length === 0);
    manifest.acceptance.allErrorsClear = manifest.captures.every((capture) => capture.errors.console.length === 0 && capture.errors.page.length === 0 && capture.errors.requestFailures.length === 0);
    manifest.captureCompletedAt = new Date().toISOString();
    const outputPath = path.join(here, "media_manifest.json");
    await fs.promises.writeFile(outputPath, `${JSON.stringify(manifest, null, 2)}\n`);
    assert(manifest.acceptance.allReportsMatchReceiptBeforeAndAfter, "One or more frozen report outputs differ from their receipt.");
    assert(manifest.acceptance.sourceFilesUnchangedAcrossCapture, "A frozen report output changed during capture.");
    assert(manifest.acceptance.allCapturesPassed, "One or more mockup captures did not satisfy capture assertions.");
    assert(manifest.acceptance.noBlockedOrExternalRequests, "A request outside a frozen report inventory was attempted.");
    assert(manifest.acceptance.allErrorsClear, "A capture produced console, page, or request errors.");
    process.stdout.write(`${JSON.stringify({ manifest: "docs/working/v0_6_3_design_reviews/media_manifest.json", captures: manifest.captures.map((capture) => ({ id: capture.id, path: capture.image.path, bytes: capture.image.bytes, width: capture.image.width, height: capture.image.height, sha256: capture.image.sha256 })), acceptance: manifest.acceptance }, null, 2)}\n`);
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  process.stderr.write(`${error.stack || error}\n`);
  process.exitCode = 1;
});
