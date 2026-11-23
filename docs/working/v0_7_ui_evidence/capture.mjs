import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { mkdir, mkdtemp, rm } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "../../..");
const webRoot = resolve(process.env.GOMOKU_WEB_ROOT ?? join(repoRoot, "gomoku-web"));
const requireFromWeb = createRequire(resolve(webRoot, "package.json"));
const { chromium, expect } = requireFromWeb("@playwright/test");
const baselineUrl = (process.env.GOMOKU_MEDIA_BASELINE_URL ?? "http://127.0.0.1:8015").replace(/\/$/, "");
const candidateUrl = (process.env.GOMOKU_MEDIA_CANDIDATE_URL ?? "http://127.0.0.1:8016").replace(/\/$/, "");
const assetsDir = resolve(repoRoot, "docs/assets");
let framesRoot;

function positionFor(box, row, col) {
  const size = 15;
  const cell = Math.min(box.width / size, box.height / size);
  const boardHeight = size * cell;
  return {
    x: (box.width - (size - 1) * cell) / 2 + col * cell,
    y: (box.height - boardHeight) / 2 + cell / 2 + row * cell,
  };
}

async function activate(locator, touch) {
  if (touch) await locator.tap();
  else await locator.click();
}

async function openFixture(page, baseUrl, touch = false) {
  await page.goto(`${baseUrl}/guide/#examples`);
  await activate(page.getByRole("link", { name: "Open Review a finished game", exact: true }), touch);
  await expect(page.getByRole("heading", { name: "Replay Analysis", exact: true })).toBeVisible();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 28 / 28");
}

async function branchAtMove27(page, touch = false) {
  await activate(page.getByRole("button", { name: "Previous move", exact: true }), touch);
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 27 / 28");
  await activate(page.getByRole("button", { name: "Play From Here", exact: true }), touch);
  await expect(page.getByRole("heading", { name: "Local Match", exact: true })).toBeVisible();
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 27");
}

async function playContinuationMove(page, touch = false) {
  const canvas = page.locator("canvas").first();
  const box = await canvas.boundingBox();
  if (!box) throw new Error("Match board did not report its bounds");
  const position = positionFor(box, 10, 5);
  if (touch) await canvas.tap({ position });
  else await canvas.click({ position });
  const place = page.getByRole("button", { name: "Place", exact: true });
  if (await place.count()) {
    await expect(place).toBeEnabled();
    await activate(place, touch);
  }
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 28");
  await expect(page.getByTestId("match-status")).toHaveText("Guest wins");
}

async function prepareScreenshot(page, width, height) {
  await page.evaluate(() => {
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
    window.scrollTo(0, 0);
    return document.fonts.ready;
  });
  await page.mouse.move(width - 3, height - 3);
  await page.waitForTimeout(120);
}

async function screenshot(page, filename, width, height) {
  await prepareScreenshot(page, width, height);
  await page.screenshot({ path: join(here, filename), animations: "disabled", caret: "hide" });
}

async function comparison(browser, label, baseUrl, width, height, touch) {
  const context = await browser.newContext({
    viewport: { width, height },
    deviceScaleFactor: 1,
    hasTouch: touch,
  });
  const page = await context.newPage();
  await openFixture(page, baseUrl, touch);
  await branchAtMove27(page, touch);

  const layout = width === 1440 ? "desktop" : "compact";
  const filename = `match-${layout}-${label === "baseline" ? "before" : "after"}.png`;
  await screenshot(page, filename, width, height);

  if (label === "match") {
    await activate(page.getByRole("link", { name: "Source replay", exact: true }), touch);
    await expect(page.getByRole("heading", { name: "Replay Analysis", exact: true })).toBeFocused();
    await expect(page.getByTestId("replay-move-count")).toHaveText("Move 27 / 28");
    await page.goBack();
    await expect(page.getByRole("heading", { name: "Local Match", exact: true })).toBeVisible();
    await expect(page.getByTestId("match-move-count")).toHaveText("Move 27");
  } else {
    await page.goBack();
    await expect(page.getByRole("heading", { name: "Replay Analysis", exact: true })).toBeVisible();
    await expect(page.getByTestId("replay-move-count")).toHaveText("Move 28 / 28");
  }

  const returnState = {
    path: await page.evaluate(() => location.pathname),
    replayMove: await page.getByTestId("replay-move-count").count()
      ? await page.getByTestId("replay-move-count").textContent()
      : null,
    matchMove: await page.getByTestId("match-move-count").count()
      ? await page.getByTestId("match-move-count").textContent()
      : null,
  };
  await context.close();
  return { build: label, width, height, screenshot: filename, returnState };
}

async function captureFrames(page, directory, count) {
  await mkdir(directory, { recursive: true });
  const start = count.index;
  const path = join(directory, `frame-${String(start).padStart(3, "0")}.png`);
  await prepareScreenshot(page, 1280, 800);
  await page.screenshot({ path, animations: "disabled", caret: "hide" });
  for (let hold = 1; hold < count.hold; hold += 1) {
    const duplicatePath = join(directory, `frame-${String(start + hold).padStart(3, "0")}.png`);
    execFileSync("cp", [path, duplicatePath]);
  }
  count.index += count.hold;
}

function encodeGif(frameDirectory, outputPath) {
  const palette = join(frameDirectory, "palette.png");
  const input = join(frameDirectory, "frame-%03d.png");
  execFileSync("ffmpeg", [
    "-hide_banner", "-loglevel", "error", "-y", "-framerate", "5", "-i", input,
    "-vf", "fps=5,scale=960:-1:flags=lanczos,palettegen=stats_mode=diff",
    "-frames:v", "1", palette,
  ]);
  execFileSync("ffmpeg", [
    "-hide_banner", "-loglevel", "error", "-y", "-framerate", "5", "-i", input,
    "-i", palette, "-filter_complex",
    "[0:v]fps=5,scale=960:-1:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=3",
    "-loop", "0", outputPath,
  ]);
}

async function captureReadmeGifs(browser, analysisOnly) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 });

  const replay = await context.newPage();
  await openFixture(replay, candidateUrl);
  await expect(replay.getByTestId("replay-analysis-status")).toHaveText("White has won", { timeout: 120_000 });
  await expect(replay.getByTestId("replay-timeline-analyzed")).toBeVisible();
  const analysisFrames = join(framesRoot, "analysis");
  const frameState = { index: 0, hold: 5 };
  for (const [step, expected] of [
    [null, "Move 28 / 28"],
    ["Previous move", "Move 27 / 28"],
    ["Previous move", "Move 26 / 28"],
    ["Previous move", "Move 25 / 28"],
    ["Next move", "Move 26 / 28"],
    ["Next move", "Move 27 / 28"],
  ]) {
    if (step) await replay.getByRole("button", { name: step, exact: true }).click();
    await expect(replay.getByTestId("replay-move-count")).toHaveText(expected);
    await captureFrames(replay, analysisFrames, frameState);
  }
  const analysisGif = join(assetsDir, "readme-analysis.gif");
  encodeGif(analysisFrames, analysisGif);

  if (analysisOnly) {
    await context.close();
    return { analysisGif };
  }

  const gameplay = await context.newPage();
  await openFixture(gameplay, candidateUrl);
  const gameplayFrames = join(framesRoot, "gameplay");
  const gameState = { index: 0, hold: 5 };
  await captureFrames(gameplay, gameplayFrames, gameState);
  await gameplay.getByRole("button", { name: "Previous move", exact: true }).click();
  await expect(gameplay.getByTestId("replay-move-count")).toHaveText("Move 27 / 28");
  await captureFrames(gameplay, gameplayFrames, gameState);
  await gameplay.getByRole("button", { name: "Play From Here", exact: true }).click();
  await expect(gameplay.getByTestId("match-move-count")).toHaveText("Move 27");
  await captureFrames(gameplay, gameplayFrames, gameState);
  await playContinuationMove(gameplay);
  gameState.hold = 15;
  await captureFrames(gameplay, gameplayFrames, gameState);
  const gameplayGif = join(assetsDir, "readme-gameplay.gif");
  encodeGif(gameplayFrames, gameplayGif);

  await context.close();
  return { analysisGif, gameplayGif };
}

await mkdir(here, { recursive: true });
await mkdir(assetsDir, { recursive: true });
framesRoot = await mkdtemp(join(tmpdir(), "gomoku2d-v07-ui-evidence-"));
const browser = await chromium.launch({ headless: true });
const result = {
  playwright: requireFromWeb.resolve("@playwright/test"),
  browserVersion: browser.version(),
  captures: [],
};
try {
  const analysisOnly = process.argv.includes("--analysis-only");
  if (!analysisOnly) {
    result.captures.push(await comparison(browser, "baseline", baselineUrl, 1440, 900, false));
    result.captures.push(await comparison(browser, "baseline", baselineUrl, 390, 844, true));
    result.captures.push(await comparison(browser, "match", candidateUrl, 1440, 900, false));
    result.captures.push(await comparison(browser, "match", candidateUrl, 390, 844, true));
  }
  result.gifs = await captureReadmeGifs(browser, analysisOnly);
} finally {
  await browser.close();
  await rm(framesRoot, { recursive: true, force: true });
}
console.log(JSON.stringify(result, null, 2));
