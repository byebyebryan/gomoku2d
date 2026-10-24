import { chromium } from "@playwright/test";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { copyFile, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const base = process.env.GOMOKU_MEDIA_BASE_URL;
const profileId = process.env.GOMOKU_MEDIA_PUBLIC_PROFILE_ID;
if (!base || !["127.0.0.1", "localhost"].includes(new URL(base).hostname)) {
  throw new Error("Set GOMOKU_MEDIA_BASE_URL to an explicitly owned loopback preview.");
}
if (!profileId || !/^[A-Za-z0-9_-]{1,128}$/.test(profileId)) {
  throw new Error("Set GOMOKU_MEDIA_PUBLIC_PROFILE_ID to the reviewed local demo UID.");
}
const root = resolve(import.meta.dirname, "../..");
const output = resolve(root, "docs/assets");
const temporary = resolve(process.env.GOMOKU_MEDIA_TMP_DIR ?? resolve(root, "gomoku-web/test-results/v062-completion/media"));
const run = promisify(execFile);
const browser = await chromium.launch();

/** @param {import('@playwright/test').Page} page */
async function settle(page) {
  await page.evaluate(async () => {
    await document.fonts.ready;
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
    await new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done)));
  });
}

/** @param {import('@playwright/test').Page} page @param {string} name */
async function clipRecorder(page, name) {
  const folder = resolve(temporary, name);
  await rm(folder, { recursive: true, force: true });
  await mkdir(folder, { recursive: true });
  let frame = 0;
  return {
    /** @param {number} count */
    async hold(count) {
      await settle(page);
      const first = resolve(folder, `${String(frame).padStart(4, "0")}.png`);
      await page.screenshot({ path: first, animations: "disabled" });
      for (let offset = 1; offset < count; offset++) {
        await copyFile(first, resolve(folder, `${String(frame + offset).padStart(4, "0")}.png`));
      }
      frame += count;
    },
    async finish() {
      return frame;
    },
  };
}

try {
  await mkdir(output, { recursive: true });
  await mkdir(temporary, { recursive: true });
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, serviceWorkers: "block" });
  await page.goto(new URL("guide/#examples", base).href);
  const examples = page.getByRole("heading", { name: "Example replays", exact: true });
  await examples.waitFor();
  await settle(page);
  await page.screenshot({ path: resolve(temporary, "examples.png"), animations: "disabled" });
  const sharing = await clipRecorder(page, "readme-sharing");
  await sharing.hold(8);
  const example = page.getByRole("link", { name: "Open Review a finished game", exact: true });
  const examplePath = await example.getAttribute("href");
  if (!examplePath) throw new Error("Missing portable example link.");
  await example.click();
  await page.getByRole("grid", { name: "Gomoku board" }).waitFor();
  await page.getByTestId("replay-move-count").filter({ hasText: "Move 28 / 28" }).waitFor();
  await sharing.hold(6);
  await page.getByRole("button", { name: "Previous turn", exact: true }).click();
  await sharing.hold(4);
  await page.getByRole("button", { name: "Share replay", exact: true }).click();
  await page.getByRole("textbox", { name: "Replay link", exact: true }).waitFor();
  await sharing.hold(12);
  const sharingFrames = await sharing.finish();

  await page.goto(new URL(examplePath, base).href);
  await page.getByRole("grid", { name: "Gomoku board" }).waitFor();
  await page.getByTestId("replay-analysis-status").filter({ hasText: /./ }).waitFor();
  await settle(page);
  await page.screenshot({ path: resolve(output, "readme-replay-analysis.png"), animations: "disabled" });
  const analysis = await clipRecorder(page, "readme-analysis");
  await analysis.hold(6);
  for (let turn = 0; turn < 7; turn++) {
    await page.getByRole("button", { name: "Previous turn", exact: true }).click();
    await analysis.hold(3);
  }
  const analysisFrames = await analysis.finish();

  await page.goto(new URL(`player/?id=${encodeURIComponent(profileId)}`, base).href);
  await page.getByRole("heading", { name: "Demo Player", exact: true }).waitFor();
  await page.getByText("Page 1 of 3", { exact: true }).waitFor();
  await settle(page);
  await page.screenshot({ path: resolve(output, "readme-public-profile.png"), animations: "disabled" });
  const version = JSON.parse(await readFile(resolve(root, "gomoku-web/package.json"), "utf8")).version;
  const { stdout: sourceCommit } = await run("git", ["rev-parse", "HEAD"], { cwd: root });
  const { stdout: sourceChanges } = await run("git", ["status", "--porcelain", "--", "gomoku-web/src", "gomoku-web/package.json", "gomoku-web/package-lock.json"], { cwd: root });
  await writeFile(resolve(temporary, "capture.json"), JSON.stringify({ version, sourceCommit: sourceCommit.trim(), applicationInputsClean: sourceChanges.trim() === "", base, profileId, viewport: { width: 1280, height: 800 }, gifWidth: 960, fps: 5,
    sharingFrames, analysisFrames, encoded: false, examplePath, source: "Existing app UI; only focus was cleared, no content or style substitution." }, null, 2) + "\n");
  if (process.env.GOMOKU_MEDIA_FRAMES_ONLY !== "1") {
    await run(process.execPath, [resolve(root, "gomoku-web/scripts/encode_sharing_media.mjs")], { env: process.env });
  }
  console.log(JSON.stringify({ version, sharingFrames, analysisFrames, assets: ["readme-sharing.gif", "readme-analysis.gif", "readme-replay-analysis.png", "readme-public-profile.png"] }));
} finally {
  await browser.close();
}
