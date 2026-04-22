import { expect, test, type Page } from "@playwright/test";

function boardClickPosition(box: { width: number; height: number }, row: number, col: number) {
  const boardSize = 15;
  const cellSize = Math.min(box.width / boardSize, box.height / boardSize);
  const boardHeight = boardSize * cellSize;
  const originX = (box.width - (boardSize - 1) * cellSize) / 2;
  const originY = (box.height - boardHeight) / 2 + cellSize / 2;

  return {
    x: originX + col * cellSize,
    y: originY + row * cellSize,
  };
}

async function openFinishedReplay(page: Page) {
  await page.goto("/profile");

  const displayName = page.getByLabel("Display name");
  await displayName.fill("Bryan Guest");
  await page.getByRole("button", { name: "Renju" }).click();

  await page.getByRole("link", { name: "Play" }).click();
  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();
  await expect(page.getByTestId("match-rule")).toHaveText("Renju");

  const canvas = page.locator("canvas").first();
  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error("board canvas did not report a bounding box");
  }

  for (const [row, col] of [[0, 0], [2, 0], [4, 0], [6, 0], [8, 0]]) {
    const beforeCount = Number((await page.getByTestId("match-move-count").textContent()) ?? "0");
    await canvas.click({ position: boardClickPosition(box, row, col) });
    await expect
      .poll(async () => Number((await page.getByTestId("match-move-count").textContent()) ?? "0"), {
        timeout: 15_000,
      })
      .toBeGreaterThan(beforeCount);
  }

  await expect(page.getByText("Classic Bot wins")).toBeVisible();
  await page.getByRole("link", { name: "Profile" }).click();
  await page.getByRole("button", { name: "Replay" }).first().click();
  await expect(page.getByRole("heading", { name: "Replay" })).toBeVisible();
}

test("local replay opens from profile history and supports stepping plus autoplay", async ({ page }) => {
  await openFinishedReplay(page);
  await expect(page.getByTestId("replay-result")).toHaveText("Classic Bot wins");
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 4 / 10");
  await expect(page.getByTestId("replay-rule")).toHaveText("Renju");
  await expect(page.getByTestId("replay-player-row-black")).toContainText("Bryan Guest");
  await expect(page.getByTestId("replay-player-row-black")).toContainText("Player");
  await expect(page.getByTestId("replay-player-row-white")).toContainText("Classic Bot");
  await expect(page.getByTestId("replay-player-row-white")).toContainText("Bot");
  await expect(page.getByTestId("replay-player-row-black")).toHaveCSS("box-shadow", /rgb/);
  await expect(page.getByRole("button", { name: "Play From Here" })).toBeEnabled();
  await expect(page.locator('[data-testid="replay-step-controls"] button')).toHaveText([
    "Start",
    "End",
    "Previous move",
    "Next move",
  ]);

  await page.getByRole("button", { name: "Start" }).click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 1 / 10");
  await expect(page.getByRole("button", { name: "Play From Here" })).toBeDisabled();
  await expect(page.getByTestId("replay-player-row-white")).toHaveCSS("box-shadow", /rgb/);
  await page.getByRole("button", { name: "End" }).click();
  await expect(page.getByTestId("replay-result")).toHaveText("Classic Bot wins");
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 10 / 10");
  await expect(page.getByRole("button", { name: "Play From Here" })).toBeDisabled();
  await page.getByRole("button", { name: "Start" }).click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 1 / 10");

  await page.setViewportSize({ width: 430, height: 932 });
  const portraitMetrics = await page.evaluate(() => {
    const header = document.querySelector("header");
    const layout = document.querySelector('[class*="layout"]');
    const boardPanel = document.querySelector('[class*="boardPanel"]');
    const deck = document.querySelector('[class*="deck"]');
    const frame = document.querySelector('[class*="frame"]');
    const canvas = document.querySelector("canvas");

    if (!header || !layout || !boardPanel || !deck || !frame || !canvas) {
      return null;
    }

    const layoutBox = layout.getBoundingClientRect();
    const boardPanelBox = boardPanel.getBoundingClientRect();
    const deckBox = deck.getBoundingClientRect();
    const frameBox = frame.getBoundingClientRect();
    const canvasBox = canvas.getBoundingClientRect();

    return {
      boardToLayoutWidth: boardPanelBox.width / layoutBox.width,
      bodyOverflowY: window.getComputedStyle(document.body).overflowY,
      headerTop: header.getBoundingClientRect().top,
      panelGap: deckBox.top - boardPanelBox.bottom,
      layoutOverflowY: window.getComputedStyle(layout).overflowY,
      canvasToFrame: Math.min(
        canvasBox.width / frameBox.width,
        canvasBox.height / frameBox.height,
      ),
    };
  });

  expect(portraitMetrics).not.toBeNull();
  expect(portraitMetrics!.boardToLayoutWidth).toBeGreaterThan(0.98);
  expect(portraitMetrics!.bodyOverflowY).toBe("auto");
  expect(portraitMetrics!.panelGap).toBeGreaterThanOrEqual(18);
  expect(portraitMetrics!.layoutOverflowY).toBe("visible");
  expect(portraitMetrics!.canvasToFrame).toBeGreaterThan(0.98);

  await page.evaluate(() => window.scrollTo(0, 200));
  await page.waitForTimeout(50);
  await expect
    .poll(async () =>
      page.evaluate(() => document.querySelector("header")?.getBoundingClientRect().top ?? 0),
    )
    .toBeLessThan(portraitMetrics!.headerTop - 20);

  await page.getByRole("button", { name: "Auto play" }).click();
  await expect(page.getByRole("button", { name: "Pause" })).toBeVisible();
  await expect
    .poll(async () => page.getByTestId("replay-move-count").textContent(), { timeout: 15_000 })
    .toBe("Move 10 / 10");
  await expect(page.getByText("Classic Bot wins")).toBeVisible();
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
  await expect(page.getByTestId("match-move-count")).toHaveText("5");
  await expect(page.getByTestId("match-status")).toHaveText("Bryan Guest to move");
  await expect(page.getByTestId("player-row-black")).toContainText("Classic Bot");
  await expect(page.getByTestId("player-row-black")).toContainText("Bot");
  await expect(page.getByTestId("player-row-white")).toContainText("Bryan Guest");
  await expect(page.getByTestId("player-row-white")).toContainText("Player");
  await expect(page.getByTestId("player-row-white")).toHaveCSS("box-shadow", /rgb/);
});
