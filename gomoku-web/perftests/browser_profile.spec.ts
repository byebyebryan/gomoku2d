import { expect, test, type Page } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { cpus, platform, release } from "node:os";
import { dirname, resolve } from "node:path";

import { resolveBotConfig } from "../src/core/bot_config.js";
import type { BotWorkerResponse } from "../src/core/bot_protocol.js";
import { seedLocalSavedMatch } from "../playtests/helpers/local_history.js";

type Game = { match_index: number; move_count: number; move_cells: number[]; result: "black_won" | "white_won" | "draw" };
const measurements: Array<Record<string, unknown>> = [];
const output = process.env.GOMOKU_PROFILE_OUTPUT
  ?? resolve("../gomoku-bot-lab/outputs/browser-profile", `${Date.now()}.json`);
const source = JSON.parse(await readFile("../reports/lab/bot-report.json", "utf8")) as {
  run: { rules: { variant: "freestyle" | "renju" } }; matches: Game[];
};

function game(index: number): Game {
  const found = source.matches.find((entry) => entry.match_index === index);
  if (!found) throw new Error(`Missing profile workload ${index}`);
  return found;
}

function fenFor(game: Game, prefix: number): string {
  if (prefix >= game.move_count) throw new Error("Profile prefix must be ongoing");
  const cells = Array<string>(225).fill(".");
  game.move_cells.slice(0, prefix).forEach((cell, i) => { cells[cell] = i % 2 ? "W" : "B"; });
  return `15/5/${prefix % 2 ? "W" : "B"}/${cells.join("")}`;
}

async function completedAnalysis(page: Page): Promise<string> {
  const status = page.getByTestId("replay-analysis-status");
  await expect(status).toBeVisible();
  await expect(status).not.toHaveText("Tracing the finish", { timeout: 60_000 });
  const text = await status.innerText();
  expect(text).not.toMatch(/unavailable|failed/i);
  return text;
}

test.afterAll(async ({ browser }) => {
  const files = (await readdir("dist/assets")).filter((name) => /\.(js|wasm|css)$/.test(name)).sort();
  const hash = createHash("sha256");
  for (const name of files) { hash.update(name); hash.update(await readFile(`dist/assets/${name}`)); }
  await mkdir(dirname(output), { recursive: true });
  await writeFile(output, JSON.stringify({
    generatedAt: new Date().toISOString(),
    gitCommit: execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
    dirty: Boolean(execFileSync("git", ["status", "--porcelain"], { encoding: "utf8" }).trim()),
    bundleSha256: hash.digest("hex"), browser: browser.version(),
    host: { platform: platform(), release: release(), cpu: cpus()[0]?.model, logicalCpus: cpus().length },
    notes: ["localhost production preview; no internet/network throttling", "fresh contexts and warm repeats, not a physical slow device", "CPU throttling applies to the page target; worker slowdown is not assumed", "diagnostic workload samples, not timing assertions or strength evidence"],
    measurements,
  }, null, 2));
  console.log(`Browser profile: ${output}`);
});

for (const cpuRate of [1, 6]) {
  test(`fresh and warm local startup, page CPU rate ${cpuRate}`, async ({ browser }) => {
    const context = await browser.newContext();
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
      measurements.push({ kind: "startup", phase, cpuRate, homeMs, boardStateReadyMs, assets });
    }
    await context.close();
  });
}

test("product bot workers on published legal positions", async ({ page }) => {
  await page.goto("/");
  const workerFile = (await readdir("dist/assets")).find((name) => /^bot_worker-.*\.js$/.test(name));
  if (!workerFile) throw new Error("Production bot worker not found");
  for (const preset of ["easy", "normal", "hard"] as const) {
    for (const [matchIndex, prefix] of [[65, 4], [577, 12], [1089, 20]]) {
      const fen = fenFor(game(matchIndex), prefix);
      const spec = resolveBotConfig({ mode: "preset", preset, version: 1 });
      const result = await page.evaluate(async ({ url, fen, spec, slot, variant }) => {
        const worker = new Worker(url, { type: "module" });
        const readyStart = performance.now();
        const gaps: number[] = [];
        let previous = performance.now();
        const heartbeat = setInterval(() => { const now = performance.now(); gaps.push(now - previous); previous = now; }, 16);
        try {
          await new Promise<void>((resolve, reject) => {
            const timer = setTimeout(() => reject(new Error("Worker boot timeout")), 15_000);
            worker.onerror = (event) => { clearTimeout(timer); reject(new Error(event.message)); };
            worker.onmessage = ({ data }: MessageEvent<BotWorkerResponse>) => {
              if (data.type === "ready") { clearTimeout(timer); resolve(); }
            };
          });
          const readyMs = performance.now() - readyStart;
          worker.postMessage({ type: "configure", specs: slot === 0 ? [spec, { kind: "human" }] : [{ kind: "human" }, spec] });
          const samples: Array<{ durationMs: number; timeout: boolean; move: unknown }> = [];
          for (let requestId = 1; requestId <= 3; requestId += 1) {
            const start = performance.now();
            const reply = await new Promise<BotWorkerResponse | null>((resolve, reject) => {
              const timer = setTimeout(() => resolve(null), 15_000);
              worker.onerror = (event) => { clearTimeout(timer); reject(new Error(event.message)); };
              worker.onmessage = ({ data }: MessageEvent<BotWorkerResponse>) => {
                if (data.type === "error" || (data.type === "move" && data.requestId === requestId)) {
                  clearTimeout(timer); resolve(data);
                }
              };
              worker.postMessage({ type: "choose_move", requestId, slot, variant, fen });
            });
            if (reply?.type === "error") throw new Error(reply.message);
            samples.push({ durationMs: performance.now() - start, timeout: reply === null, move: reply?.type === "move" ? reply.move : null });
            if (!reply) break;
          }
          return { readyMs, samples, maxMainThreadHeartbeatGapMs: Math.max(0, ...gaps) };
        } finally { clearInterval(heartbeat); worker.terminate(); }
      }, { url: `/assets/${workerFile}`, fen, spec, slot: prefix % 2, variant: source.run.rules.variant });
      const row = { kind: "bot", preset, matchIndex, prefix, ...result };
      measurements.push(row);
      console.log(JSON.stringify(row));
    }
  }
});

test("analysis first use, warm cache and repeated worker disposal", async ({ page, context }) => {
  await page.goto("/");
  const entry = game(65);
  await seedLocalSavedMatch(page, {
    id: "profile-real-ending", displayName: "Profile fixture", savedAt: "2026-10-15T00:00:00Z",
    moves: entry.move_cells.map((cell) => ({ row: Math.floor(cell / 15), col: cell % 15 })),
    preferredVariant: source.run.rules.variant, variant: source.run.rules.variant, status: entry.result,
  });
  const start = performance.now();
  await page.goto("/replay/profile-real-ending");
  await page.getByTestId("replay-timeline-analyzed").waitFor({ state: "attached", timeout: 60_000 });
  const firstAnnotationMs = performance.now() - start;
  const status = await completedAnalysis(page);
  const completeMs = performance.now() - start;
  await page.getByRole("link", { name: "Home", exact: true }).click();
  await expect.poll(() => page.workers().length).toBe(0);
  await page.getByRole("link", { name: "Profile", exact: true }).click();
  await expect(page.getByRole("button", { name: "Inspect" })).toBeVisible();
  await page.getByRole("link", { name: "Home", exact: true }).click();
  const cdp = await context.newCDPSession(page);
  await cdp.send("Performance.enable");
  const heap = async () => {
    await cdp.send("HeapProfiler.collectGarbage");
    const result = await cdp.send("Performance.getMetrics");
    return result.metrics.find((metric: { name: string }) => metric.name === "JSHeapUsedSize")?.value;
  };
  const heapBefore = await heap();
  const warmMs: number[] = [];
  const heapMilestones: Array<{ cycle: number; bytes: number | undefined }> = [];
  for (let i = 0; i < 30; i += 1) {
    await page.getByRole("link", { name: "Profile", exact: true }).click();
    await expect(page.getByRole("button", { name: "Inspect" })).toBeVisible();
    const start = performance.now();
    await page.getByRole("button", { name: "Inspect" }).click();
    await completedAnalysis(page);
    warmMs.push(performance.now() - start);
    await page.getByRole("link", { name: "Home", exact: true }).click();
    await expect.poll(() => page.workers().length).toBe(0);
    await expect(page.locator("canvas")).toHaveCount(0);
    if ([4, 9, 19, 29].includes(i)) heapMilestones.push({ cycle: i + 1, bytes: await heap() });
  }
  measurements.push({ kind: "analysis", matchIndex: 65, firstAnnotationMs, completeMs, status, warmMs,
    workerCountAfterCycles: page.workers().length, mainHeapBefore: heapBefore, heapMilestones,
    heapScope: "page JS heap after forced GC across SPA cycles; excludes worker/Wasm/GPU memory" });
});
