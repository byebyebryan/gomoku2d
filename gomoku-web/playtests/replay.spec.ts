import { expect, test, type Page } from "@playwright/test";

import { seedLocalSavedMatch } from "./helpers/local_history";

async function openFinishedReplay(page: Page) {
  await page.goto("/profile");

  await seedLocalSavedMatch(page, {
    displayName: "Bryan Guest",
    id: "fixture-replay-match",
    moves: [
      { col: 7, row: 7 },
      { col: 6, row: 5 },
      { col: 8, row: 7 },
      { col: 6, row: 6 },
      { col: 9, row: 7 },
      { col: 6, row: 7 },
      { col: 0, row: 0 },
      { col: 6, row: 8 },
      { col: 1, row: 0 },
      { col: 6, row: 9 },
    ],
    preferredVariant: "renju",
    savedAt: "2026-04-22T18:30:00.000Z",
    status: "white_won",
    variant: "renju",
  });

  await page.goto("/profile");
  await expect(page.getByText("vs Practice Bot")).toBeVisible();
  await page.getByRole("button", { name: "Replay" }).first().click();
  await expect(page.getByRole("heading", { name: "Replay" })).toBeVisible();
  await expect(page).toHaveURL(/\/replay\/fixture-replay-match$/);
}

test("replay route uses the clean URL and old local route falls through", async ({ page }) => {
  await page.goto("/replay/not-found");
  await expect(page.getByRole("heading", { name: "Replay unavailable" })).toBeVisible();
  await expect(page).toHaveURL(/\/replay\/not-found$/);

  await page.goto("/replays/local/not-found");
  await expect(page.getByRole("heading", { name: "Gomoku2D" })).toBeVisible();
  await expect(page).toHaveURL(/\/$/);
});

test("local replay opens from profile history and supports stepping plus autoplay", async ({ page }) => {
  await openFinishedReplay(page);
  await expect(page.getByTestId("replay-result")).toHaveText("Practice Bot wins");
  await expect(page.getByText("Replay timeline")).toHaveCount(0);
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 4 / 10");
  await expect(page.getByTestId("replay-rule")).toHaveText("Renju");
  await expect(page.getByTestId("replay-player-row-black")).toContainText("Bryan Guest");
  await expect(page.getByTestId("replay-player-row-white")).toContainText("Practice Bot");
  await expect(page.getByTestId("replay-player-row-black").getByRole("img", { name: "Player" })).toBeVisible();
  await expect(page.getByTestId("replay-player-row-white").getByRole("img", { name: "Bot" })).toBeVisible();
  await expect(page.getByTestId("replay-player-row-black")).toHaveCSS("box-shadow", /rgb/);
  await expect(page.getByRole("button", { name: "Play From Here" })).toBeEnabled();
  await expect(page.locator('[data-testid="replay-step-controls"] button')).toHaveCount(5);
  await expect(page.getByRole("button", { name: "Start" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Previous move" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Auto play" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Next move" })).toBeVisible();
  await expect(page.getByRole("button", { name: "End" })).toBeVisible();

  await page.getByRole("button", { name: "Start" }).click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 1 / 10");
  await expect(page.getByRole("button", { name: "Play From Here" })).toBeDisabled();
  await expect(page.getByTestId("replay-player-row-white")).toHaveCSS("box-shadow", /rgb/);
  await page.getByRole("button", { name: "End" }).click();
  await expect(page.getByTestId("replay-result")).toHaveText("Practice Bot wins");
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 10 / 10");
  await expect(page.getByRole("button", { name: "Play From Here" })).toBeDisabled();
  await page.getByRole("button", { name: "Start" }).click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 1 / 10");

  await page.setViewportSize({ width: 430, height: 932 });
  const portraitMetrics = await page.evaluate(() => {
    const header = document.querySelector("header");
    const layout = document.querySelector('[class*="layout"]');
    const boardPanel = document.querySelector('[class*="boardPanel"]');
    const playerRows = document.querySelector('[class*="playerRows"]');
    const timeline = document.querySelector('[class*="timeline"]');
    const controlsRow = document.querySelector('[class*="controlsRow"]');
    const resumeAction = document.querySelector('[class*="resumeAction"]');
    const metaRows = document.querySelector('[class*="metaRows"]');
    const resultSection = document.querySelector('[class*="resultSection"]');
    const headerLabels = Array.from(document.querySelectorAll('[class*="headerActions"] [class*="uiActionLabel"]'));
    const frame = document.querySelector('[class*="frame"]');
    const canvas = document.querySelector("canvas");

    if (
      !header ||
      !layout ||
      !boardPanel ||
      !playerRows ||
      !timeline ||
      !controlsRow ||
      !resumeAction ||
      !metaRows ||
      !resultSection ||
      !frame ||
      !canvas
    ) {
      return null;
    }

    const layoutBox = layout.getBoundingClientRect();
    const boardPanelBox = boardPanel.getBoundingClientRect();
    const playerRowsBox = playerRows.getBoundingClientRect();
    const timelineBox = timeline.getBoundingClientRect();
    const controlsRowBox = controlsRow.getBoundingClientRect();
    const resumeActionBox = resumeAction.getBoundingClientRect();
    const metaRowsBox = metaRows.getBoundingClientRect();
    const frameBox = frame.getBoundingClientRect();
    const canvasBox = canvas.getBoundingClientRect();

    return {
      boardToLayoutWidth: boardPanelBox.width / layoutBox.width,
      boardFitsLayout: boardPanelBox.right <= layoutBox.right + 1,
      headerLabelsHidden: headerLabels.every((label) => window.getComputedStyle(label).display === "none"),
      playerRowsGap: boardPanelBox.top - playerRowsBox.bottom,
      timelineGap: timelineBox.top - boardPanelBox.bottom,
      controlsGap: controlsRowBox.top - timelineBox.bottom,
      resumeGap: resumeActionBox.top - controlsRowBox.bottom,
      metaGap: metaRowsBox.top - resumeActionBox.bottom,
      layoutOverflowY: window.getComputedStyle(layout).overflowY,
      pageScrollRange: document.documentElement.scrollHeight - document.documentElement.clientHeight,
      resultHidden: window.getComputedStyle(resultSection).display === "none",
      canvasToFrame: Math.min(
        canvasBox.width / frameBox.width,
        canvasBox.height / frameBox.height,
      ),
    };
  });

  expect(portraitMetrics).not.toBeNull();
  expect(portraitMetrics!.boardToLayoutWidth).toBeGreaterThan(0.98);
  expect(portraitMetrics!.boardFitsLayout).toBe(true);
  expect(portraitMetrics!.headerLabelsHidden).toBe(true);
  expect(portraitMetrics!.playerRowsGap).toBeGreaterThanOrEqual(8);
  expect(portraitMetrics!.timelineGap).toBeGreaterThanOrEqual(8);
  expect(portraitMetrics!.controlsGap).toBeGreaterThanOrEqual(8);
  expect(portraitMetrics!.resumeGap).toBeGreaterThanOrEqual(8);
  expect(portraitMetrics!.metaGap).toBeGreaterThanOrEqual(8);
  expect(portraitMetrics!.layoutOverflowY).toBe("visible");
  expect(portraitMetrics!.pageScrollRange).toBeGreaterThanOrEqual(0);
  expect(portraitMetrics!.resultHidden).toBe(true);
  expect(portraitMetrics!.canvasToFrame).toBeGreaterThan(0.98);

  await page.getByRole("button", { name: "Auto play" }).click();
  await expect(page.getByRole("button", { name: "Pause" })).toBeVisible();
  await expect
    .poll(async () => page.getByTestId("replay-move-count").textContent(), { timeout: 15_000 })
    .toBe("Move 10 / 10");
  await expect(page.getByTestId("replay-result")).toHaveText("Practice Bot wins");
  await expect(page.getByRole("button", { name: "Auto play" })).toBeVisible();
});

test("local replay can start a new local match from the current replay frame", async ({ page }) => {
  await openFinishedReplay(page);

  await page.getByRole("button", { name: "Next move" }).click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 5 / 10");
  await expect(page.getByTestId("replay-player-row-white")).toHaveCSS("box-shadow", /rgb/);
  await expect(page.getByRole("button", { name: "Play From Here" })).toBeEnabled();

  await page.getByRole("button", { name: "Play From Here" }).click();

  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();
  await expect(page.getByTestId("match-rule")).toHaveText("Renju");
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 5");
  await expect(page.getByTestId("match-status")).toHaveText("Bryan Guest to move");
  await expect(page.getByTestId("player-row-black")).toContainText("Practice Bot");
  await expect(page.getByTestId("player-row-white")).toContainText("Bryan Guest");
  await expect(page.getByTestId("player-row-black").getByRole("img", { name: "Bot" })).toBeVisible();
  await expect(page.getByTestId("player-row-white").getByRole("img", { name: "Player" })).toBeVisible();
  await expect(page.getByTestId("player-row-white")).toHaveCSS("box-shadow", /rgb/);
});

test("desktop replay rail keeps compact transport and player spacing", async ({ page }) => {
  await openFinishedReplay(page);

  const metrics = await page.evaluate(() => {
    const resultSection = document.querySelector('[class*="resultSection"]');
    const matchSection = document.querySelector('[class*="matchSection"]');
    const playbackSection = document.querySelector('[class*="playbackSection"]');
    const metaRows = document.querySelector('[class*="metaRows"]');
    const playerRows = document.querySelector('[class*="playerRows"]');
    const playerRow = document.querySelector('[data-testid^="replay-player-row-"]');
    const timeline = document.querySelector('[class*="timeline"]');
    const playbackHeader = document.querySelector('[class*="playbackHeader"]');
    const controlsRow = document.querySelector('[class*="controlsRow"]');
    const controlsIcon = controlsRow?.querySelector('.uiIcon');

    if (
      !resultSection ||
      !matchSection ||
      !playbackSection ||
      !metaRows ||
      !playerRows ||
      !playerRow ||
      !timeline ||
      !playbackHeader ||
      !controlsRow ||
      !controlsIcon
    ) {
      return null;
    }

    const resultStyle = window.getComputedStyle(resultSection);
    const matchStyle = window.getComputedStyle(matchSection);
    const playbackStyle = window.getComputedStyle(playbackSection);
    const metaRowsStyle = window.getComputedStyle(metaRows);
    const playerRowsStyle = window.getComputedStyle(playerRows);
    const playerRowStyle = window.getComputedStyle(playerRow);
    const timelineStyle = window.getComputedStyle(timeline);
    const playbackHeaderStyle = window.getComputedStyle(playbackHeader);
    const controlsRowStyle = window.getComputedStyle(controlsRow);
    const controlsIconStyle = window.getComputedStyle(controlsIcon);

    return {
      controlsGap: controlsRowStyle.gap,
      controlsIconWidth: Number.parseFloat(controlsIconStyle.width),
      matchSectionGap: matchStyle.gap,
      matchSectionPaddingTop: matchStyle.paddingTop,
      metaRowsGap: metaRowsStyle.gap,
      playbackHeaderGap: playbackHeaderStyle.gap,
      playbackSectionGap: playbackStyle.gap,
      playbackSectionPaddingTop: playbackStyle.paddingTop,
      playerRowPaddingTop: playerRowStyle.paddingTop,
      playerRowsGap: playerRowsStyle.gap,
      playerRowsPaddingTop: playerRowsStyle.paddingTop,
      resultSectionGap: resultStyle.gap,
      resultSectionPaddingTop: resultStyle.paddingTop,
      timelineGap: timelineStyle.gap,
    };
  });

  expect(metrics).not.toBeNull();
  expect(metrics!.resultSectionGap).toBe("12px");
  expect(metrics!.resultSectionPaddingTop).toBe("16px");
  expect(metrics!.matchSectionGap).toBe("12px");
  expect(metrics!.matchSectionPaddingTop).toBe("16px");
  expect(metrics!.metaRowsGap).toBe("8px");
  expect(metrics!.playerRowsGap).toBe("8px");
  expect(metrics!.playerRowsPaddingTop).toBe("8px");
  expect(metrics!.playerRowPaddingTop).toBe("10px");
  expect(metrics!.playbackSectionGap).toBe("12px");
  expect(metrics!.playbackSectionPaddingTop).toBe("16px");
  expect(metrics!.playbackHeaderGap).toBe("8px");
  expect(metrics!.timelineGap).toBe("8px");
  expect(metrics!.controlsGap).toBe("8px");
  expect(metrics!.controlsIconWidth).toBe(24);
});
