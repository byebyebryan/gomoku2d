import { expect, test, type Page } from "@playwright/test";

async function waitForBotReply(page: Page) {
  await expect
    .poll(
      async () => {
        const [moves, status] = await Promise.all([
          page.getByTestId("match-move-count").textContent(),
          page.getByTestId("match-status").textContent(),
        ]);

        return `${moves}|${status}`;
      },
      { timeout: 20_000 },
    )
    .toBe("Move 2|Guest to move");
}

test("home boot and local bot match smoke flow", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Gomoku2D" })).toBeVisible();
  await expect(page.getByText(/five in a row/i)).toBeVisible();

  await page.getByRole("link", { name: "Play" }).click();

  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 0");
  await expect(page.getByTestId("match-rule")).toHaveText("Freestyle");
  await expect(page.getByTestId("match-status")).toHaveText("Guest to move");
  await expect(page.getByRole("button", { name: "Undo" })).toBeDisabled();

  const canvas = page.locator("canvas").first();
  await expect(canvas).toBeVisible();

  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error("board canvas did not report a bounding box");
  }

  await canvas.click({
    position: {
      x: box.width / 2,
      y: box.height / 2,
    },
  });

  await waitForBotReply(page);
  await expect(page.getByRole("button", { name: "Undo" })).toBeEnabled();

  await page.getByRole("button", { name: "Undo" }).click();
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 0");
  await expect(page.getByTestId("match-status")).toHaveText("Guest to move");
  await expect(page.getByRole("button", { name: "Undo" })).toBeDisabled();

  await canvas.click({
    position: {
      x: box.width / 2,
      y: box.height / 2,
    },
  });

  await waitForBotReply(page);

  await page.getByRole("button", { name: "Renju" }).click();
  await expect(page.getByTestId("match-rule")).toHaveText("Freestyle");
  await expect(page.getByTestId("match-next-rule")).toHaveText("Renju");

  await page.getByRole("button", { name: "New Game" }).click();
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 0");
  await expect(page.getByTestId("match-rule")).toHaveText("Renju");

  await canvas.click({
    position: {
      x: box.width / 2,
      y: box.height / 2,
    },
  });

  await waitForBotReply(page);
});

test("local match keeps the board frame stable with the compact HUD", async ({ page }) => {
  await page.goto("/match/local");

  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();

  const metrics = await page.evaluate(() => {
    const frame = document.querySelector('[class*="frame"]');
    const hud = document.querySelector('[class*="hud"]');

    if (!frame || !hud) {
      return null;
    }

    const frameBox = frame.getBoundingClientRect();

    return {
      frameHeight: frameBox.height,
      hudClientHeight: (hud as HTMLElement).clientHeight,
      hudScrollHeight: (hud as HTMLElement).scrollHeight,
      pageClientHeight: document.documentElement.clientHeight,
      pageHeight: document.documentElement.scrollHeight,
    };
  });

  expect(metrics).not.toBeNull();
  expect(metrics!.frameHeight).toBeGreaterThan(400);
  expect(metrics!.pageHeight - metrics!.pageClientHeight).toBeLessThanOrEqual(2);
  expect(metrics!.hudScrollHeight - metrics!.hudClientHeight).toBeLessThanOrEqual(2);
});

test("direct entry to the local match route loads the app", async ({ page }) => {
  await page.goto("/match/local");

  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();
  await expect(page.getByTestId("match-rule")).toHaveText("Freestyle");
  const canvas = page.locator("canvas").first();
  await expect(canvas).toBeVisible();

  const viewport = canvas.locator("xpath=..");
  await expect
    .poll(async () => {
      const [canvasBox, viewportBox] = await Promise.all([
        canvas.boundingBox(),
        viewport.boundingBox(),
      ]);

      if (!canvasBox || !viewportBox) {
        return 0;
      }

      return Math.min(
        canvasBox.width / viewportBox.width,
        canvasBox.height / viewportBox.height,
      );
    })
    .toBeGreaterThan(0.98);
});

test("portrait local match keeps the board frame tight to the canvas", async ({ page }) => {
  await page.setViewportSize({ width: 430, height: 760 });
  await page.goto("/match/local");

  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();

  const ratios = await page.evaluate(() => {
    const header = document.querySelector("header");
    const layout = document.querySelector('[class*="layout"]');
    const boardPanel = document.querySelector('[class*="boardPanel"]');
    const playerRows = document.querySelector('[class*="playerRows"]');
    const matchActions = document.querySelector('[class*="matchActions"]');
    const ruleRow = document.querySelector('[class*="ruleRow"]');
    const statusSection = document.querySelector('[class*="statusSection"]');
    const matchLabel = document.querySelector('[class*="matchLabel"]');
    const headerActions = document.querySelector('[class*="headerActions"]');
    const frame = document.querySelector('[class*="frame"]');
    const viewport = document.querySelector('[class*="viewport"]');
    const canvas = document.querySelector("canvas");
    const ruleButtons = Array.from(
      document.querySelectorAll('[class*="variantButtons"] button'),
    ) as HTMLElement[];

    if (
      !header ||
      !layout ||
      !boardPanel ||
      !playerRows ||
      !matchActions ||
      !ruleRow ||
      !statusSection ||
      !matchLabel ||
      !headerActions ||
      !frame ||
      !viewport ||
      !canvas
    ) {
      return null;
    }

    const layoutBox = layout.getBoundingClientRect();
    const boardPanelBox = boardPanel.getBoundingClientRect();
    const playerRowsBox = playerRows.getBoundingClientRect();
    const matchActionsBox = matchActions.getBoundingClientRect();
    const ruleRowBox = ruleRow.getBoundingClientRect();
    const headerBox = header.getBoundingClientRect();
    const actionButtons = Array.from(headerActions.querySelectorAll("a,button"));
    const headerLabels = Array.from(headerActions.querySelectorAll('[class*="uiActionLabel"]'));
    const frameBox = frame.getBoundingClientRect();
    const viewportBox = viewport.getBoundingClientRect();
    const canvasBox = canvas.getBoundingClientRect();

    return {
      actionRows: new Set(
        actionButtons.map((button) => Math.round((button as HTMLElement).getBoundingClientRect().top)),
      ).size,
      boardToLayoutWidth: boardPanelBox.width / layoutBox.width,
      boardFitsLayout: boardPanelBox.right <= layoutBox.right + 1,
      boardPanelWidth: boardPanelBox.width,
      headerFitsLayout: headerBox.right <= document.documentElement.clientWidth + 1,
      layoutOverflowY: window.getComputedStyle(layout).overflowY,
      matchLabelHidden: window.getComputedStyle(matchLabel).display === "none",
      headerLabelsHidden: headerLabels.every((label) => window.getComputedStyle(label).display === "none"),
      playerRowsGap: boardPanelBox.top - playerRowsBox.bottom,
      playerRowsFitLayout: playerRowsBox.right <= layoutBox.right + 1,
      actionGap: matchActionsBox.top - boardPanelBox.bottom,
      ruleGap: ruleRowBox.top - matchActionsBox.bottom,
      pageScrollRange: document.documentElement.scrollHeight - document.documentElement.clientHeight,
      statusHidden: window.getComputedStyle(statusSection).display === "none",
      canvasToFrame: Math.min(
        canvasBox.width / frameBox.width,
        canvasBox.height / frameBox.height,
      ),
      viewportToFrame: Math.min(
        viewportBox.width / frameBox.width,
        viewportBox.height / frameBox.height,
      ),
      minRuleButtonHeight:
        ruleButtons.length > 0
          ? Math.min(...ruleButtons.map((button) => button.getBoundingClientRect().height))
          : 0,
      squareDelta: Math.abs(canvasBox.width - canvasBox.height),
    };
  });

  expect(ratios).not.toBeNull();
  expect(ratios!.actionRows).toBe(1);
  expect(ratios!.boardToLayoutWidth).toBeGreaterThan(0.98);
  expect(ratios!.boardPanelWidth).toBeLessThanOrEqual(430);
  expect(ratios!.boardFitsLayout).toBe(true);
  expect(ratios!.headerFitsLayout).toBe(true);
  expect(ratios!.layoutOverflowY).toBe("hidden");
  expect(ratios!.matchLabelHidden).toBe(true);
  expect(ratios!.headerLabelsHidden).toBe(true);
  expect(ratios!.playerRowsGap).toBeGreaterThanOrEqual(8);
  expect(ratios!.playerRowsFitLayout).toBe(true);
  expect(ratios!.actionGap).toBeGreaterThanOrEqual(8);
  expect(ratios!.ruleGap).toBeGreaterThanOrEqual(8);
  expect(ratios!.pageScrollRange).toBeLessThanOrEqual(2);
  expect(ratios!.statusHidden).toBe(true);
  expect(ratios!.canvasToFrame).toBeGreaterThan(0.9);
  expect(ratios!.viewportToFrame).toBeGreaterThan(0.9);
  expect(ratios!.minRuleButtonHeight).toBeGreaterThanOrEqual(44);
  expect(ratios!.squareDelta).toBeLessThanOrEqual(1);

  await page.evaluate(() => window.scrollTo(0, 200));
  await page.waitForTimeout(50);
  await expect.poll(async () => page.evaluate(() => window.scrollY)).toBe(0);
});

test("portrait touch input uses Place instead of auto-placing on release", async ({ browser }) => {
  const context = await browser.newContext({
    hasTouch: true,
    viewport: { width: 430, height: 760 },
  });
  const page = await context.newPage();

  try {
    const baseUrl = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:4173";
    await page.goto(new URL("/match/local", baseUrl).toString());

    await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();

    const placeButton = page.getByRole("button", { name: "Place" });
    await expect(placeButton).toBeDisabled();

    const canvas = page.locator("canvas").first();
    await expect(canvas).toBeVisible();

    const box = await canvas.boundingBox();
    if (!box) {
      throw new Error("board canvas did not report a bounding box");
    }

    await page.touchscreen.tap(box.x + box.width / 2, box.y + box.height / 2);
    await page.waitForTimeout(150);

    await expect(page.getByTestId("match-move-count")).toHaveText("Move 0");
    await expect(placeButton).toBeEnabled();

    await page.getByRole("button", { name: "New Game" }).click();
    await expect(page.getByTestId("match-move-count")).toHaveText("Move 0");
    await expect(placeButton).toBeDisabled();

    await page.touchscreen.tap(box.x + box.width / 2, box.y + box.height / 2);
    await page.waitForTimeout(150);
    await expect(placeButton).toBeEnabled();

    await placeButton.click();
    await waitForBotReply(page);
    await expect(placeButton).toBeDisabled();
  } finally {
    await context.close();
  }
});

test("narrow non-touch portrait keeps direct click placement without Place mode", async ({ page }) => {
  await page.setViewportSize({ width: 430, height: 760 });
  await page.goto("/match/local");

  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Place" })).toHaveCount(0);

  const canvas = page.locator("canvas").first();
  await expect(canvas).toBeVisible();

  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error("board canvas did not report a bounding box");
  }

  await canvas.click({
    position: {
      x: box.width / 2,
      y: box.height / 2,
    },
  });

  await waitForBotReply(page);
});

test("canvas stays matched to the board viewport after resizing into portrait", async ({ page }) => {
  await page.setViewportSize({ width: 1200, height: 800 });
  await page.goto("/match/local");
  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();

  await page.setViewportSize({ width: 430, height: 932 });

  await expect
    .poll(async () => {
      return page.evaluate(() => {
        const viewport = document.querySelector('[class*="viewport"]');
        const canvas = document.querySelector("canvas");

        if (!viewport || !canvas) {
          return 0;
        }

        const viewportBox = viewport.getBoundingClientRect();
        const canvasBox = canvas.getBoundingClientRect();

        return Math.min(
          canvasBox.width / viewportBox.width,
          canvasBox.height / viewportBox.height,
        );
      });
    })
    .toBeGreaterThan(0.98);
});
