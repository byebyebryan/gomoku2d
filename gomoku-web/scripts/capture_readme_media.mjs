import { chromium } from "@playwright/test";
import { execFile } from "node:child_process";
import { copyFile, mkdir, readFile, rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

/** @typedef {import("@playwright/test").Browser} Browser */
/** @typedef {import("@playwright/test").BrowserContext} BrowserContext */
/** @typedef {import("@playwright/test").Page} Page */
/** @typedef {{ x: number, y: number, width: number, height: number }} Box */
/** @typedef {{ frame: number }} CaptureState */
/** @typedef {{ move_cells: number[], result: "black_won" | "white_won" | "draw" }} MatchReport */
/** @typedef {{ path: string, match_report: MatchReport }} AnalysisEntry */

const __dirname = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(__dirname, "..");
const repoRoot = resolve(webRoot, "..");
const docsAssetsDir = join(repoRoot, "docs", "assets");
const publicDir = join(webRoot, "public");
const assetSourceDir = join(webRoot, "assets");
const tmpRoot = process.env.GOMOKU_MEDIA_TMP_DIR ?? "/tmp/gomoku2d-readme-media";
const gameplayFrameDir = join(tmpRoot, "gameplay-frames");
const analysisFrameDir = join(tmpRoot, "analysis-frames");
const labFrameDir = join(tmpRoot, "lab-frames");
const visualsFrameDir = join(tmpRoot, "visuals-frames");
const ogBoardCapture = join(tmpRoot, "og-board.png");

const baseUrl = (process.env.GOMOKU_MEDIA_BASE_URL ?? "http://127.0.0.1:8001").replace(/\/?$/, "/");
const viewport = { width: 1280, height: 720 };
const captureFps = 5;
const gameplayGifFps = 5;
const analysisGifFps = 5;
const labGifFps = 5;
const visualsGifFps = 5;
const gifWidth = 960;
const gameplayAnalysisPath = "match_1129__search-d3_pattern-eval__vs__search-d7_tactical-cap-8_pattern-eval_corridor-proof-c16-d8-w4";
const replayAnalysisPath = "match_0095__search-d1__vs__search-d3_pattern-eval";
const labAnalysisPath = "match_1104__search-d7_tactical-cap-8_pattern-eval_corridor-proof-c16-d8-w4__vs__search-d3_pattern-eval";
const gameplayReplayId = "readme-media-gameplay";
const gameplayStartMoveIndex = 30;
const analysisReplayId = "readme-media-analysis";
const analysisReplayMoveLabels = [
  "Move 28 / 28",
  "Move 26 / 28",
  "Move 24 / 28",
  "Move 22 / 28",
  "Move 20 / 28",
  "Move 18 / 28",
  "Move 16 / 28",
  "Move 14 / 28",
];
const localProfileStorageKey = "gomoku2d.local-profile.v5";

const output = {
  gameplayGif: join(docsAssetsDir, "readme-gameplay.gif"),
  analysisGif: join(docsAssetsDir, "readme-analysis.gif"),
  labGif: join(docsAssetsDir, "readme-lab.gif"),
  visualsGif: join(docsAssetsDir, "readme-visuals.gif"),
  replayStill: join(docsAssetsDir, "readme-replay-analysis.png"),
  ogPublic: join(publicDir, "og.png"),
  ogSource: join(assetSourceDir, "og_source.png"),
};

/** @param {string} path */
function repoPath(path) {
  return path.replace(`${repoRoot}/`, "");
}

/** @param {string} path */
function urlFor(path) {
  return new URL(path.replace(/^\//, ""), baseUrl).toString();
}

/** @param {string} path */
async function fontDataUrl(path) {
  return `data:font/ttf;base64,${(await readFile(path)).toString("base64")}`;
}

/**
 * @param {string} command
 * @param {string[]} args
 * @param {import("node:child_process").ExecFileOptions & { inheritOutput?: boolean }} [options]
 * @returns {Promise<{ stdout: string | Buffer, stderr: string | Buffer }>}
 */
function run(command, args, options = {}) {
  return new Promise((resolveRun, reject) => {
    const child = execFile(command, args, {
      cwd: repoRoot,
      maxBuffer: 16 * 1024 * 1024,
      ...options,
    }, (error, stdout, stderr) => {
      if (error) {
        error.message = `${error.message}\n${stdout}\n${stderr}`;
        reject(error);
        return;
      }
      resolveRun({ stdout, stderr });
    });

    if (options.inheritOutput) {
      child.stdout?.pipe(process.stdout);
      child.stderr?.pipe(process.stderr);
    }
  });
}

async function waitForPreview() {
  const deadline = Date.now() + 20_000;
  let lastError = null;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(baseUrl);
      if (response.ok) {
        return;
      }
      lastError = new Error(`preview returned HTTP ${response.status}`);
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolveDelay) => setTimeout(resolveDelay, 500));
  }

  throw new Error(
    `Could not reach ${baseUrl}. Start a production preview first, for example:\n`
      + `  cd gomoku-web && npm run build && npm run preview -- --host 0.0.0.0 --port 8001\n`
      + `Last error: ${lastError instanceof Error ? lastError.message : String(lastError)}`,
  );
}

/** @param {string} fixturePath */
async function analysisFixtureMatch(fixturePath) {
  const reportPath = join(repoRoot, "reports", "lab", "analysis-report.json");
  /** @type {{ sections?: Array<{ entries?: AnalysisEntry[] }> }} */
  const report = JSON.parse(await readFile(reportPath, "utf8"));
  for (const section of report.sections ?? []) {
    const entry = section.entries?.find((candidate) => candidate.path === fixturePath);
    if (entry) {
      return entry.match_report;
    }
  }

  throw new Error(`Could not find analysis fixture ${fixturePath} in ${repoPath(reportPath)}`);
}

function emptyCounter() {
  return {
    draws: 0,
    losses: 0,
    matches: 0,
    moves: 0,
    wins: 0,
  };
}

/**
 * @param {MatchReport} matchReport
 * @param {string} id
 */
function seededReplayMatch(matchReport, id) {
  const savedAt = "2026-06-01T12:00:00.000Z";
  const normalBot = {
    config: {
      mode: "preset",
      preset: "normal",
      version: 1,
    },
    config_version: 1,
    engine: "search_bot",
    id: "bot",
    label: "Normal",
    lab_spec: "search-d3+pattern-eval",
    version: 1,
  };

  return {
    board_size: 15,
    id,
    match_kind: "local_vs_bot",
    move_cells: matchReport.move_cells,
    move_count: matchReport.move_cells.length,
    player_black: {
      bot: null,
      display_name: "Guest",
      kind: "human",
      local_profile_id: "readme-media-profile",
      profile_uid: null,
    },
    player_white: {
      bot: normalBot,
      display_name: "Normal Bot",
      kind: "bot",
      local_profile_id: null,
      profile_uid: null,
    },
    saved_at: savedAt,
    schema_version: 2,
    source: "local_history",
    status: matchReport.result,
    trust: "local_only",
    undo_floor: 0,
    ruleset: "renju",
  };
}

/** @param {Array<{ id: string, matchReport: MatchReport }>} matches */
function seededProfile(matches) {
  const savedAt = "2026-06-01T12:00:00.000Z";
  const profile = {
    avatarUrl: null,
    createdAt: "2026-06-01T12:00:00.000Z",
    displayName: "Guest",
    id: "readme-media-profile",
    kind: "local",
    updatedAt: savedAt,
    username: null,
  };

  return {
    state: {
      matchHistory: {
        archivedStats: {
          archived_before: null,
          archived_count: 0,
          by_opponent_type: {
            bot: emptyCounter(),
            human: emptyCounter(),
          },
          by_ruleset: {
            freestyle: emptyCounter(),
            renju: emptyCounter(),
          },
          by_side: {
            black: emptyCounter(),
            white: emptyCounter(),
          },
          schema_version: 1,
          totals: emptyCounter(),
        },
        replayMatches: matches.map((match) => seededReplayMatch(match.matchReport, match.id)),
        summaryMatches: [],
      },
      profile,
      settings: {
        boardHints: {
          evidence: "on",
          immediate: "win_threat",
          imminent: "threat_counter",
        },
        botConfig: {
          mode: "preset",
          preset: "normal",
          version: 1,
        },
        gameConfig: {
          opening: "standard",
          ruleset: "renju",
        },
        touchControl: "touchpad",
      },
    },
    version: 5,
  };
}

/**
 * @param {BrowserContext} context
 * @param {Array<{ id: string, matchReport: MatchReport }>} matches
 */
async function installSeed(context, matches) {
  const payload = JSON.stringify(seededProfile(matches));
  await context.addInitScript(
    /** @param {{ key: string, value: string }} seed */
    (seed) => {
      window.localStorage.setItem(seed.key, seed.value);
    },
    {
      key: localProfileStorageKey,
      value: payload,
    },
  );
}

/** @param {Page} page */
async function waitForFonts(page) {
  await page.evaluate(async () => {
    await document.fonts?.ready;
  });
}

/** @param {Page} page */
async function waitForReplayAnalysis(page) {
  await page.waitForFunction(() => {
    const status = document.querySelector('[data-testid="replay-analysis-status"]');
    const text = status?.textContent ?? "";
    return status && !/analyzing|loading|checking|tracing/i.test(text);
  }, undefined, { timeout: 120_000 });
}

/**
 * @param {Page} page
 * @param {string} moveLabel
 */
async function goToReplayMove(page, moveLabel) {
  for (let attempts = 0; attempts < 12; attempts += 1) {
    const current = (await page.getByTestId("replay-move-count").textContent())?.trim();
    if (current === moveLabel) {
      return;
    }
    await page.getByRole("button", { name: "Previous turn" }).click();
    await page.waitForTimeout(250);
  }
  throw new Error(`Could not reach replay frame ${moveLabel}`);
}

/**
 * @param {Page} page
 * @param {CaptureState} state
 * @param {string} targetFrameDir
 * @param {number} frames
 * @param {number} [delayMs]
 */
async function frameCapture(page, state, targetFrameDir, frames, delayMs = 1000 / captureFps) {
  for (let index = 0; index < frames; index += 1) {
    const path = join(targetFrameDir, `frame-${String(state.frame++).padStart(4, "0")}.png`);
    await page.screenshot({ path });
    await page.waitForTimeout(delayMs);
  }
}

/**
 * @param {Page} page
 * @param {CaptureState} state
 * @param {string} targetFrameDir
 * @param {number} maxFrames
 * @param {() => Promise<boolean>} isComplete
 */
async function frameCaptureUntil(page, state, targetFrameDir, maxFrames, isComplete) {
  for (let index = 0; index < maxFrames; index += 1) {
    const path = join(targetFrameDir, `frame-${String(state.frame++).padStart(4, "0")}.png`);
    await page.screenshot({ path });
    if (await isComplete()) {
      return true;
    }
    await page.waitForTimeout(1000 / captureFps);
  }
  return false;
}

/**
 * @param {Box} box
 * @param {number} row
 * @param {number} col
 */
function boardPoint(box, row, col) {
  const boardSize = 15;
  const cellSize = Math.min(box.width / boardSize, box.height / boardSize);
  const boardHeight = boardSize * cellSize;
  const originX = (box.width - (boardSize - 1) * cellSize) / 2;
  const originY = (box.height - boardHeight) / 2 + cellSize / 2;

  return {
    x: box.x + originX + col * cellSize,
    y: box.y + originY + row * cellSize,
  };
}

/** @param {number} cell */
function cellPosition(cell) {
  return {
    col: cell % 15,
    row: Math.floor(cell / 15),
  };
}

/**
 * @param {Page} page
 * @param {CaptureState} state
 * @param {string} targetFrameDir
 * @param {MatchReport} matchReport
 */
async function captureGameplayScene(page, state, targetFrameDir, matchReport) {
  await page.goto(urlFor(`/replay/${gameplayReplayId}?media=showcase`));
  await page.locator("canvas").first().waitFor({ state: "visible" });
  await waitForReplayAnalysis(page);
  await goToReplayMove(page, `Move ${gameplayStartMoveIndex} / ${matchReport.move_cells.length}`);
  await page.getByRole("button", { name: "Play From Here" }).click();
  await page.getByRole("heading", { name: "Local Match" }).waitFor();
  await page.locator("canvas").first().waitFor({ state: "visible" });
  await waitForFonts(page);
  const canvas = page.locator("canvas").first();
  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error("Local match canvas did not report a bounding box");
  }

  await frameCapture(page, state, targetFrameDir, 6);

  for (let moveIndex = gameplayStartMoveIndex; moveIndex < matchReport.move_cells.length; moveIndex += 2) {
    const cell = cellPosition(matchReport.move_cells[moveIndex]);
    const move = boardPoint(box, cell.row, cell.col);
    await page.mouse.move(move.x, move.y);
    await frameCapture(page, state, targetFrameDir, moveIndex === gameplayStartMoveIndex ? 5 : 4);
    await page.mouse.click(move.x, move.y);
    await frameCapture(page, state, targetFrameDir, 3);

    const targetMoveCount = Math.min(moveIndex + 2, matchReport.move_cells.length);
    const botReplied = await frameCaptureUntil(page, state, targetFrameDir, 36, async () => {
      const current = (await page.getByTestId("match-move-count").textContent())?.trim();
      return current === `Move ${targetMoveCount}`;
    });
    if (!botReplied) {
      throw new Error(`Timed out waiting for move ${targetMoveCount} while capturing gameplay media`);
    }

    await frameCapture(page, state, targetFrameDir, targetMoveCount === matchReport.move_cells.length ? 12 : 4);
  }
}

/** @param {Page} page */
async function captureOgBoardScene(page) {
  await page.goto(urlFor(`/replay/${analysisReplayId}?media=og`));
  await page.locator("canvas").first().waitFor({ state: "visible" });
  await waitForReplayAnalysis(page);
  await waitForFonts(page);
  await goToReplayMove(page, "Move 26 / 28");
  await page.getByRole("button", { name: "Play From Here" }).click();
  await page.getByRole("heading", { name: "Local Match" }).waitFor();
  await page.locator("canvas").first().waitFor({ state: "visible" });
  await waitForFonts(page);
  const canvas = page.locator("canvas").first();
  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error("OG board canvas did not report a bounding box");
  }

  const blackMove = boardPoint(box, 5, 10);
  await page.mouse.click(blackMove.x, blackMove.y);
  await page.waitForFunction(() => (
    document.querySelector('[data-testid="match-move-count"]')?.textContent?.trim() === "Move 28"
  ), undefined, { timeout: 60_000 });
  await page.waitForTimeout(500);
  await canvas.screenshot({ path: ogBoardCapture });
}

/**
 * @param {Page} page
 * @param {string} moveLabel
 */
async function stepReplayBackward(page, moveLabel) {
  await page.getByRole("button", { name: "Previous turn" }).click();
  for (let attempts = 0; attempts < 12; attempts += 1) {
    const current = (await page.getByTestId("replay-move-count").textContent())?.trim();
    if (current === moveLabel) {
      return;
    }
    await page.waitForTimeout(100);
  }
  throw new Error(`Could not rewind replay frame to ${moveLabel}`);
}

/**
 * @param {Page} page
 * @param {CaptureState} state
 * @param {string} targetFrameDir
 */
async function captureReplayScene(page, state, targetFrameDir) {
  await page.goto(urlFor(`/replay/${analysisReplayId}`));
  await page.locator("canvas").first().waitFor({ state: "visible" });
  await waitForReplayAnalysis(page);
  await waitForFonts(page);
  await goToReplayMove(page, analysisReplayMoveLabels[0]);
  for (const [index, moveLabel] of analysisReplayMoveLabels.entries()) {
    if (index > 0) {
      await stepReplayBackward(page, moveLabel);
    }
    const isFirst = index === 0;
    const isLast = index === analysisReplayMoveLabels.length - 1;
    await frameCapture(page, state, targetFrameDir, isFirst || isLast ? 8 : 5);
  }
  await page.screenshot({ path: output.replayStill });
}

/**
 * @param {Page} page
 * @param {"Ranking" | "Search" | "Analysis"} tabName
 */
async function waitForLabTab(page, tabName) {
  await page.getByRole("tab", { name: tabName }).waitFor({ state: "visible" });
  await page.waitForFunction((name) => {
    const selected = document.querySelector('[role="tab"][aria-selected="true"]');
    return selected?.textContent?.trim() === name;
  }, tabName);
}

/** @param {import("@playwright/test").Locator} details */
async function openDetails(details) {
  await details.waitFor({ state: "visible", timeout: 45_000 });
  const isOpen = await details.evaluate((element) => element instanceof HTMLDetailsElement && element.open);
  if (!isOpen) {
    await details.locator("summary").first().click();
  }
}

/**
 * @param {Page} page
 * @param {CaptureState} state
 * @param {string} targetFrameDir
 */
async function captureLabScene(page, state, targetFrameDir) {
  await page.goto(urlFor("/lab/"));
  await waitForFonts(page);
  await waitForLabTab(page, "Ranking");
  await page.getByRole("heading", { name: "Results" }).waitFor({ state: "visible" });
  await frameCapture(page, state, targetFrameDir, 12);

  const rankingPanel = page.locator('section[data-view="ranking"]');
  const topBot = rankingPanel.locator("details").first();
  await openDetails(topBot);
  await frameCapture(page, state, targetFrameDir, 2);

  const firstPair = topBot.locator("details").first();
  await openDetails(firstPair);
  await frameCapture(page, state, targetFrameDir, 2);

  const firstGame = firstPair.locator("details").first();
  await openDetails(firstGame);
  await frameCapture(page, state, targetFrameDir, 2);

  const finishedBoard = firstGame.locator('[data-report-board="finished"]').first();
  await finishedBoard.waitFor({ state: "visible", timeout: 45_000 });
  await finishedBoard.scrollIntoViewIfNeeded();
  await frameCapture(page, state, targetFrameDir, 16);

  await page.getByRole("tab", { name: "Search" }).click();
  await waitForLabTab(page, "Search");
  await page.getByRole("heading", { name: "Results" }).waitFor({ state: "visible" });
  await frameCapture(page, state, targetFrameDir, 8);
  for (let index = 0; index < 4; index += 1) {
    await page.mouse.wheel(0, 220);
    await frameCapture(page, state, targetFrameDir, 1, 160);
  }
  await frameCapture(page, state, targetFrameDir, 5);

  await page.getByRole("tab", { name: "Analysis" }).click();
  await waitForLabTab(page, "Analysis");
  await page.locator('section[data-view="analysis"]').waitFor({ state: "visible" });
  await page.getByRole("heading", { name: "Results" }).waitFor({ state: "visible" });
  await frameCapture(page, state, targetFrameDir, 8);

  const targetPair = page
    .locator('section[data-view="analysis"] details')
    .filter({ hasText: /Normal[\s\S]*Hard|Hard[\s\S]*Normal/ })
    .first();
  await targetPair.locator("summary").first().click();
  const targetMatch = page.locator(`[data-analysis-match-path="${labAnalysisPath}"]`);
  await targetMatch.waitFor({ state: "visible", timeout: 45_000 });
  await targetMatch.scrollIntoViewIfNeeded();
  await frameCapture(page, state, targetFrameDir, 4);

  await targetMatch.locator("summary").click();
  await page.waitForFunction((matchPath) => {
    const match = document.querySelector(`[data-analysis-match-path="${matchPath}"]`);
    return match instanceof HTMLDetailsElement && match.open;
  }, labAnalysisPath);
  await frameCapture(page, state, targetFrameDir, 3);

  const firstProofBoard = targetMatch.locator('[data-proof-board="analysis"]').first();
  await firstProofBoard.waitFor({ state: "visible", timeout: 45_000 });
  await firstProofBoard.scrollIntoViewIfNeeded();
  await frameCapture(page, state, targetFrameDir, 17);

  for (let index = 0; index < 8; index += 1) {
    await page.mouse.wheel(0, 260);
    await frameCapture(page, state, targetFrameDir, 1, 160);
  }
  await frameCapture(page, state, targetFrameDir, 10);
}

/**
 * @param {Page} page
 * @param {"Style" | "Icons" | "Sprites"} tabName
 */
async function waitForVisualsTab(page, tabName) {
  await page.getByRole("tab", { name: tabName }).waitFor({ state: "visible" });
  await page.waitForFunction((name) => {
    const selected = document.querySelector('[role="tab"][aria-selected="true"]');
    return selected?.textContent?.trim() === name;
  }, tabName);
}

/** @param {Page} page */
async function scrollVisualsToTop(page) {
  await page.evaluate(() => {
    document.querySelector("main")?.scrollTo({ top: 0 });
  });
}

/**
 * @param {Page} page
 * @param {CaptureState} state
 * @param {string} targetFrameDir
 * @param {number} steps
 * @param {number} deltaY
 */
async function captureWheelScroll(page, state, targetFrameDir, steps, deltaY) {
  for (let index = 0; index < steps; index += 1) {
    await page.mouse.wheel(0, deltaY);
    await frameCapture(page, state, targetFrameDir, 1, 160);
  }
}

/**
 * @param {Page} page
 * @param {CaptureState} state
 * @param {string} targetFrameDir
 */
async function captureVisualsScene(page, state, targetFrameDir) {
  await page.goto(urlFor("/visuals/"));
  await waitForFonts(page);
  await waitForVisualsTab(page, "Style");
  await page.getByRole("heading", { name: "Style Guide" }).waitFor({ state: "visible" });
  await frameCapture(page, state, targetFrameDir, 9);
  await captureWheelScroll(page, state, targetFrameDir, 5, 260);
  await frameCapture(page, state, targetFrameDir, 5);

  await page.getByRole("tab", { name: "Icons" }).click();
  await waitForVisualsTab(page, "Icons");
  await scrollVisualsToTop(page);
  await page.getByRole("heading", { name: "Icons" }).waitFor({ state: "visible" });
  await page.locator('[aria-labelledby="visuals-tab-icons"] img').first().waitFor({ state: "visible" });
  await frameCapture(page, state, targetFrameDir, 10);
  await captureWheelScroll(page, state, targetFrameDir, 5, 260);
  await frameCapture(page, state, targetFrameDir, 5);

  await page.getByRole("tab", { name: "Sprites" }).click();
  await waitForVisualsTab(page, "Sprites");
  await scrollVisualsToTop(page);
  await page.getByRole("heading", { name: "Sprites" }).waitFor({ state: "visible" });
  await page.waitForTimeout(700);
  await frameCapture(page, state, targetFrameDir, 10);
  await captureWheelScroll(page, state, targetFrameDir, 8, 300);
  await frameCapture(page, state, targetFrameDir, 8);
}

/**
 * @param {string} targetFrameDir
 * @param {string} outputPath
 * @param {string} name
 * @param {number} fps
 */
async function renderGif(targetFrameDir, outputPath, name, fps) {
  const palettePath = join(tmpRoot, `${name}-palette.png`);
  const sourcePattern = join(targetFrameDir, "frame-%04d.png");
  await run("ffmpeg", [
    "-y",
    "-framerate",
    String(fps),
    "-i",
    sourcePattern,
    "-vf",
    `fps=${fps},scale=${gifWidth}:-1:flags=lanczos,palettegen=max_colors=96`,
    palettePath,
  ]);
  await run("ffmpeg", [
    "-y",
    "-framerate",
    String(fps),
    "-i",
    sourcePattern,
    "-i",
    palettePath,
    "-lavfi",
    `fps=${fps},scale=${gifWidth}:-1:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=5`,
    outputPath,
  ]);
}

/** @param {Browser} browser */
async function renderOgImage(browser) {
  const page = await browser.newPage({ viewport: { width: 1200, height: 630 } });
  const bodyFont = await fontDataUrl(join(assetSourceDir, "fonts", "PixelOperator.ttf"));
  const gameFont = await fontDataUrl(join(assetSourceDir, "fonts", "PixelOperator8-Bold.ttf"));
  const boardDataUrl = `data:image/png;base64,${(await readFile(ogBoardCapture)).toString("base64")}`;

  await page.setContent(`<!doctype html>
    <html>
      <head>
        <style>
          @font-face { font-family: PixelOperator; src: url("${bodyFont}") format("truetype"); }
          @font-face { font-family: PixelOperator8Bold; src: url("${gameFont}") format("truetype"); }
          * { box-sizing: border-box; }
          body {
            margin: 0;
            width: 1200px;
            height: 630px;
            overflow: hidden;
            background:
              radial-gradient(circle at 15% 14%, rgba(252, 203, 87, 0.18), transparent 28%),
              radial-gradient(circle at 86% 74%, rgba(95, 139, 130, 0.16), transparent 30%),
              linear-gradient(135deg, #1e1e1e 0%, #242424 52%, #171717 100%);
            color: #f5f5f5;
            font-family: PixelOperator, Trebuchet MS, sans-serif;
          }
          .card {
            display: grid;
            grid-template-columns: 488px 1fr;
            gap: 26px;
            height: 100%;
            padding: 45px;
          }
          .copy {
            align-self: center;
            display: grid;
            gap: 24px;
          }
          .eyebrow {
            color: #fccb57;
            font-family: PixelOperator8Bold, PixelOperator, monospace;
            font-size: 28px;
            letter-spacing: 0.06em;
            text-transform: uppercase;
          }
          h1 {
            color: #fff;
            font-family: PixelOperator8Bold, PixelOperator, monospace;
            font-size: 58px;
            line-height: 0.98;
            margin: 0;
            text-shadow: 8px 8px 0 rgba(0, 0, 0, 0.46);
          }
          .tagline {
            color: #a6a6a0;
            font-family: PixelOperator8Bold, PixelOperator, monospace;
            font-size: 27px;
            line-height: 1.22;
            margin: 0;
            max-width: 480px;
          }
          .preview {
            align-self: center;
            display: grid;
            place-items: center;
          }
          .boardShot {
            display: block;
            width: 540px;
            height: 540px;
            image-rendering: pixelated;
            object-fit: contain;
          }
        </style>
      </head>
      <body>
        <main class="card">
          <section class="copy">
            <div class="eyebrow">ByeByeBryan's</div>
            <h1>Gomoku2D</h1>
            <p class="tagline">An old favorite, built properly.</p>
          </section>
          <section class="preview">
            <img class="boardShot" alt="" src="${boardDataUrl}" />
          </section>
        </main>
      </body>
    </html>`, { waitUntil: "load" });
  await page.evaluate(async () => {
    await document.fonts.ready;
  });
  await page.screenshot({ path: output.ogPublic });
  await page.close();
  await copyFile(output.ogPublic, output.ogSource);
}

async function main() {
  await waitForPreview();
  await mkdir(docsAssetsDir, { recursive: true });
  await mkdir(tmpRoot, { recursive: true });
  await rm(gameplayFrameDir, { recursive: true, force: true });
  await rm(analysisFrameDir, { recursive: true, force: true });
  await rm(labFrameDir, { recursive: true, force: true });
  await rm(visualsFrameDir, { recursive: true, force: true });
  await mkdir(gameplayFrameDir, { recursive: true });
  await mkdir(analysisFrameDir, { recursive: true });
  await mkdir(labFrameDir, { recursive: true });
  await mkdir(visualsFrameDir, { recursive: true });

  const gameplayMatchReport = await analysisFixtureMatch(gameplayAnalysisPath);
  const analysisMatchReport = await analysisFixtureMatch(replayAnalysisPath);
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    deviceScaleFactor: 1,
    viewport,
  });
  await installSeed(context, [
    { id: gameplayReplayId, matchReport: gameplayMatchReport },
    { id: analysisReplayId, matchReport: analysisMatchReport },
  ]);
  const page = await context.newPage();
  page.setDefaultTimeout(45_000);

  await captureGameplayScene(page, { frame: 0 }, gameplayFrameDir, gameplayMatchReport);
  await captureReplayScene(page, { frame: 0 }, analysisFrameDir);
  await captureOgBoardScene(page);
  await captureLabScene(page, { frame: 0 }, labFrameDir);
  await captureVisualsScene(page, { frame: 0 }, visualsFrameDir);
  await renderGif(gameplayFrameDir, output.gameplayGif, "readme-gameplay", gameplayGifFps);
  await renderGif(analysisFrameDir, output.analysisGif, "readme-analysis", analysisGifFps);
  await renderGif(labFrameDir, output.labGif, "readme-lab", labGifFps);
  await renderGif(visualsFrameDir, output.visualsGif, "readme-visuals", visualsGifFps);
  await renderOgImage(browser);
  await browser.close();

  console.log(`Generated ${repoPath(output.gameplayGif)}`);
  console.log(`Generated ${repoPath(output.analysisGif)}`);
  console.log(`Generated ${repoPath(output.labGif)}`);
  console.log(`Generated ${repoPath(output.visualsGif)}`);
  console.log(`Generated ${repoPath(output.replayStill)}`);
  console.log(`Generated ${repoPath(output.ogPublic)}`);
  console.log(`Generated ${repoPath(output.ogSource)}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
