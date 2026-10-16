import { expect, test, type Browser, type Page } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { cpus, platform, release } from "node:os";
import { dirname, resolve } from "node:path";

import { resolveBotConfig, type BotPresetId } from "../src/core/bot_config.js";
import type { BotMove, BotWorkerResponse, GameVariant } from "../src/core/bot_protocol.js";
import { seedLocalSavedMatch } from "../playtests/helpers/local_history.js";

type Position = { id: string; variant: GameVariant; matchIndex: number; prefix: number; phase: string; fen: string; legalMoves: BotMove[] };
type Replay = { id: string; variant: GameVariant; matchIndex: number; moveCells: number[]; result: "black_won" | "white_won" | "draw" };
// Node validates every prefix/replay with the freshly built Wasm core before
// any timings. No profiling entry point is added to the production application.
const workloads = JSON.parse(execFileSync(process.execPath,
  ["--experimental-wasm-modules", "scripts/profile_workloads.mjs"], { encoding: "utf8" })) as {
  manifestSha256: string; sourceSha256: string; wasmSha256: string; positions: Position[]; replays: Replay[];
};
const measurements: Array<Record<string, unknown>> = [];
const stages: string[] = [];
const output = process.env.GOMOKU_PROFILE_OUTPUT
  ?? resolve("../gomoku-bot-lab/outputs/browser-profile", `${Date.now()}.json`);

function record(row: Record<string, unknown>) { measurements.push(row); console.log(JSON.stringify(row)); }

test.afterAll(async ({ browser }) => {
  const files = (await readdir("dist/assets")).filter((name) => /\.(js|wasm|css)$/.test(name)).sort();
  const hash = createHash("sha256");
  for (const name of files) { hash.update(name); hash.update(await readFile(`dist/assets/${name}`)); }
  const git = (...args: string[]) => execFileSync("git", args, { encoding: "utf8" }).trim();
  await mkdir(dirname(output), { recursive: true });
  await writeFile(output, JSON.stringify({
    generatedAt: new Date().toISOString(), gitCommit: git("rev-parse", "HEAD"),
    trackedDirty: Boolean(git("status", "--porcelain", "--untracked-files=no")),
    untracked: git("ls-files", "--others", "--exclude-standard").split("\n").filter(Boolean),
    bundleSha256: hash.digest("hex"), browser: browser.version(), node: process.version,
    host: { platform: platform(), release: release(), cpu: cpus()[0]?.model, logicalCpus: cpus().length },
    manifestSha256: workloads.manifestSha256, sourceSha256: workloads.sourceSha256, wasmSha256: workloads.wasmSha256,
    expected: { startupPairs: 5, coldBotRequests: 108, reusedWorkerGroups: 9, reusedWorkerRequests: 36, warmRepeatRequests: 27, uncachedAnalyses: 18, cachedAnalyses: 18, navigationCycles: 30 },
    completedStages: stages,
    notes: ["localhost production preview; no network throttling", "desktop Chromium, not a physical slow device", "rate-6 diagnostic throttles only the page target; worker slowdown is not assumed", "fixed-corpus timings are diagnostic, not device-population or strength evidence", "all watchdog/error rows remain in the denominator"],
    measurements,
  }, null, 2));
  console.log(`Browser profile: ${output}`);
});

async function startup(browser: Browser, trial: number, cpuRate = 1) {
  const context = await browser.newContext();
  try {
    const page = await context.newPage();
    const cdp = await context.newCDPSession(page);
    await cdp.send("Emulation.setCPUThrottlingRate", { rate: cpuRate });
    for (const phase of ["fresh-context", "warm-reload"]) {
      const start = performance.now();
      await page.goto("/");
      await expect(page.getByRole("link", { name: "Play", exact: true })).toBeVisible();
      const homeMs = performance.now() - start;
      const playStart = performance.now();
      await page.getByRole("link", { name: "Play", exact: true }).click();
      await expect(page.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
      const boardStateReadyMs = performance.now() - playStart;
      const assets = await page.evaluate(() => performance.getEntriesByType("resource")
        .map((entry) => entry as PerformanceResourceTiming)
        .filter((entry) => entry.name.startsWith(location.origin))
        .map((entry) => ({ path: new URL(entry.name).pathname, transfer: entry.transferSize, decoded: entry.decodedBodySize })));
      record({ kind: "startup", trial, phase, cpuRate, homeMs, boardStateReadyMs, assets });
    }
  } finally { await context.close(); }
}

async function botTrial(page: Page, workerUrl: string, position: Position, preset: BotPresetId, trial: number, kind: string, repeats = 1) {
  const spec = resolveBotConfig({ mode: "preset", preset, version: 1 });
  const result = await page.evaluate(async ({ url, position, spec, repeats }) => {
    const worker = new Worker(url, { type: "module" });
    const readyStart = performance.now();
    const gaps: number[] = [];
    let previous = performance.now();
    const heartbeat = setInterval(() => { const now = performance.now(); gaps.push(now - previous); previous = now; }, 16);
    const samples: Array<{ durationMs: number; timeout: boolean; move: BotMove | null; error: string | null; phase: string }> = [];
    try {
      await new Promise<void>((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error("Worker boot timeout")), 15_000);
        worker.onerror = (event) => { clearTimeout(timer); reject(new Error(event.message)); };
        worker.onmessage = ({ data }: MessageEvent<BotWorkerResponse>) => {
          if (data.type === "ready") { clearTimeout(timer); resolve(); }
        };
      });
      const readyMs = performance.now() - readyStart;
      const slot = position.prefix % 2;
      worker.postMessage({ type: "configure", specs: slot === 0 ? [spec, { kind: "human" }] : [{ kind: "human" }, spec] });
      for (let requestId = 1; requestId <= repeats; requestId += 1) {
        const start = performance.now();
        const reply = await new Promise<BotWorkerResponse | null>((resolve, reject) => {
          const timer = setTimeout(() => resolve(null), 15_000);
          worker.onerror = (event) => { clearTimeout(timer); reject(new Error(event.message)); };
          worker.onmessage = ({ data }: MessageEvent<BotWorkerResponse>) => {
            if (data.type === "error" || (data.type === "move" && data.requestId === requestId)) { clearTimeout(timer); resolve(data); }
          };
          worker.postMessage({ type: "choose_move", requestId, slot, variant: position.variant, fen: position.fen });
        });
        samples.push({ phase: requestId === 1 ? "first-on-worker" : "reused-worker", durationMs: performance.now() - start, timeout: reply === null,
          error: reply?.type === "error" ? reply.message : null, move: reply?.type === "move" ? reply.move : null });
        if (!reply || reply.type === "error") break;
      }
      return { readyMs, samples, maxMainThreadHeartbeatGapMs: Math.max(0, ...gaps) };
    } catch (error) {
      return { readyMs: performance.now() - readyStart, samples, error: String(error), maxMainThreadHeartbeatGapMs: Math.max(0, ...gaps) };
    } finally { clearInterval(heartbeat); worker.terminate(); }
  }, { url: workerUrl, position, spec, repeats });
  record({ kind, id: position.id, preset, trial, ...result });
  expect.soft(result.error).toBeUndefined();
  expect.soft(result.samples).toHaveLength(repeats);
  for (const sample of result.samples) {
    expect.soft(sample.timeout).toBe(false);
    expect.soft(sample.error).toBeNull();
    expect.soft(position.legalMoves).toContainEqual(sample.move);
  }
  return result.samples;
}

async function seedReplay(page: Page, replay: Replay) {
  await seedLocalSavedMatch(page, {
    id: replay.id, displayName: "Profile fixture", savedAt: "2026-10-16T00:00:00Z",
    moves: replay.moveCells.map((cell) => ({ row: Math.floor(cell / 15), col: cell % 15 })),
    preferredVariant: replay.variant, variant: replay.variant, status: replay.result,
  });
}

async function completedAnalysis(page: Page, timeout = 60_000) {
  const status = page.getByTestId("replay-analysis-status");
  await expect(status).toBeVisible();
  await expect(status).not.toHaveText("Tracing the finish", { timeout });
  const text = await status.innerText();
  expect.soft(text).not.toMatch(/unavailable|failed/i);
  return text;
}

async function analysisTrial(browser: Browser, replay: Replay, trial: number, kind = "analysis") {
  const context = await browser.newContext();
  try {
    const page = await context.newPage();
    await page.goto("/"); await seedReplay(page, replay);
    const start = performance.now();
    await page.goto(`/replay/${replay.id}`);
    let firstAnnotationMs: number | null = null;
    let status: string | null = null;
    let error: string | null = null;
    try {
      await page.getByTestId("replay-timeline-analyzed").waitFor({ state: "attached", timeout: 60_000 });
      firstAnnotationMs = performance.now() - start;
      status = await completedAnalysis(page, Math.max(1, 60_000 - (performance.now() - start)));
    } catch (caught) { error = String(caught); }
    const completeMs = performance.now() - start;
    const row = { kind, id: replay.id, plies: replay.moveCells.length, trial, firstAnnotationMs, completeMs, status, error };
    record(row); expect.soft(error).toBeNull();
    await page.getByRole("link", { name: "Home", exact: true }).click();
    await expect.poll(() => page.workers().length).toBe(0);
    if (!error) {
      await page.getByRole("link", { name: "Profile", exact: true }).click();
      const cachedStart = performance.now();
      await page.getByRole("button", { name: "Inspect" }).click();
      const cachedStatus = await completedAnalysis(page);
      record({ kind: "analysis-cached", parentKind: kind, id: replay.id, trial, completeMs: performance.now() - cachedStart,
        status: cachedStatus, workers: page.workers().length });
      expect.soft(cachedStatus).toBe(status);
      expect.soft(page.workers()).toHaveLength(0);
    }
    return row;
  } finally { await context.close(); }
}

async function lifecycle(browser: Browser) {
  const context = await browser.newContext();
  try {
    const page = await context.newPage();
    const long = workloads.replays.find((entry) => entry.id === "renju-replay-18")!;
    const short = workloads.replays[0];
    await page.goto("/"); await seedReplay(page, short);
    const savedShort = await page.evaluate(() => JSON.parse(localStorage.getItem("gomoku2d.local-profile.v5")!).state.matchHistory.replayMatches[0]);
    await seedReplay(page, long);
    await page.evaluate((match) => {
      const key = "gomoku2d.local-profile.v5";
      const data = JSON.parse(localStorage.getItem(key)!);
      data.state.matchHistory.replayMatches.push(match);
      localStorage.setItem(key, JSON.stringify(data));
    }, savedShort);
    await page.goto(`/replay/${long.id}`);
    await expect.poll(() => page.workers().length).toBe(1);
    await expect(page.getByTestId("replay-analysis-status")).toHaveText("Tracing the finish");
    const oldWorker = page.workers()[0];
    const closed = oldWorker.waitForEvent("close");
    const start = performance.now();
    // Browser history navigation exercises a pending replacement in the same
    // mounted Replay route. No application store or worker callback is patched.
    await page.evaluate((id) => {
      history.pushState({}, "", `/replay/${id}`);
      dispatchEvent(new PopStateEvent("popstate"));
    }, short.id);
    await closed;
    await expect(page.getByTestId("replay-move-count")).toHaveText(`Move ${short.moveCells.length} / ${short.moveCells.length}`);
    const status = await completedAnalysis(page);
    record({ kind: "replacement", from: long.id, to: short.id, completeMs: performance.now() - start, status, oldWorkerClosed: true });
    await page.getByRole("link", { name: "Home", exact: true }).click();
    await expect.poll(() => page.workers().length).toBe(0);
    await page.evaluate(() => localStorage.removeItem("gomoku2d:replay-analysis-cache:v1"));
    await page.getByRole("link", { name: "Profile", exact: true }).click();
    await page.getByRole("button", { name: "Inspect" }).first().click();
    await expect.poll(() => page.workers().length).toBe(1);
    await expect(page.getByTestId("replay-analysis-status")).toHaveText("Tracing the finish");
    const cancelStart = performance.now();
    await page.getByRole("link", { name: "Home", exact: true }).click();
    await expect.poll(() => page.workers().length).toBe(0);
    await expect(page.locator("canvas")).toHaveCount(0);
    record({ kind: "route-cancellation", idleMs: performance.now() - cancelStart, workers: page.workers().length });
    const cdp = await context.newCDPSession(page); await cdp.send("Performance.enable");
    const heap = async () => {
      await cdp.send("HeapProfiler.collectGarbage");
      const result = await cdp.send("Performance.getMetrics");
      return result.metrics.find((metric: { name: string }) => metric.name === "JSHeapUsedSize")?.value;
    };
    const heapBefore = await heap();
    const cycles: Array<Record<string, unknown>> = [];
    for (let cycle = 1; cycle <= 30; cycle += 1) {
      await page.getByRole("link", { name: "Profile", exact: true }).click();
      const start = performance.now();
      await page.getByRole("button", { name: "Inspect" }).last().click();
      await completedAnalysis(page);
      const completeMs = performance.now() - start;
      await page.getByRole("link", { name: "Home", exact: true }).click();
      await expect.poll(() => page.workers().length).toBe(0);
      await expect(page.locator("canvas")).toHaveCount(0);
      cycles.push({ cycle, completeMs, workers: page.workers().length,
        ...([5, 10, 20, 30].includes(cycle) ? { pageHeapBytes: await heap() } : {}) });
    }
    record({ kind: "navigation", heapBefore, cycles,
      heapScope: "page JS heap after forced GC; excludes worker/Wasm/GPU memory" });
  } finally { await context.close(); }
}

test("bounded production Wasm workload matrix", async ({ browser, page }) => {
  const wasmFile = (await readdir("dist/assets")).find((name) => name.endsWith(".wasm"));
  if (!wasmFile) throw new Error("Production Wasm asset missing");
  expect(createHash("sha256").update(await readFile(`dist/assets/${wasmFile}`)).digest("hex")).toBe(workloads.wasmSha256);
  for (let trial = 1; trial <= 5; trial += 1) await startup(browser, trial);
  await startup(browser, 1, 6); stages.push("startup");
  await page.goto("/");
  const workerFile = (await readdir("dist/assets")).find((name) => /^bot_worker-.*\.js$/.test(name));
  if (!workerFile) throw new Error("Production bot worker not found");
  for (const preset of ["easy", "normal", "hard"] as const) {
    for (const position of workloads.positions) {
      const samples = [];
      for (let trial = 1; trial <= 3; trial += 1) samples.push(...await botTrial(page, `/assets/${workerFile}`, position, preset, trial, "bot-cold"));
      const trigger = preset === "normal" ? 1000 : preset === "hard" ? 3000 : Infinity;
      if (samples.some((sample) => sample.durationMs > trigger)) {
        for (let trial = 1; trial <= 5; trial += 1) await botTrial(page, `/assets/${workerFile}`, position, preset, trial, "bot-investigation");
      }
    }
  }
  stages.push("bot-cold");
  for (const position of [workloads.positions[0], workloads.positions[2], workloads.positions[10]]) {
    for (const preset of ["easy", "normal", "hard"] as const) await botTrial(page, `/assets/${workerFile}`, position, preset, 1, "bot-reused-worker", 4);
  }
  stages.push("bot-warm");
  for (const replay of workloads.replays) {
    const samples = [];
    for (let trial = 1; trial <= 3; trial += 1) samples.push(await analysisTrial(browser, replay, trial));
    if (replay.moveCells.length <= 32 && samples.some((sample) => sample.firstAnnotationMs === null || sample.firstAnnotationMs > 2000 || sample.completeMs > 5000)) {
      for (let trial = 1; trial <= 5; trial += 1) await analysisTrial(browser, replay, trial, "analysis-investigation");
    }
  }
  stages.push("analysis");
  await lifecycle(browser); stages.push("lifecycle");
});
