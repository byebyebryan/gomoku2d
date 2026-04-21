import { expect, test } from "@playwright/test";

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

test("home boot and local bot match smoke flow", async ({ page }) => {
  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: "Gomoku2D" }),
  ).toBeVisible();
  await expect(page.getByText(/five in a row/i)).toBeVisible();

  await page.getByRole("link", { name: "Play Bot" }).click();

  await expect(
    page.getByRole("heading", { name: "Local Match" }),
  ).toBeVisible();
  await expect(page.getByText("0 moves")).toBeVisible();
  await expect(page.getByText("Guest to move")).toBeVisible();

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

  await expect
    .poll(async () => page.locator("ol li").count())
    .toBeGreaterThan(0);
  await expect(page.getByText("2 moves")).toBeVisible({ timeout: 15_000 });
  await expect(page.locator("ol li")).toHaveCount(2);
  await expect(page.getByText("Guest to move")).toBeVisible();
  await expect(page.getByText("Current: Freestyle")).toBeVisible();

  await page.getByRole("button", { name: "Renju" }).click();
  await expect(page.getByText("Current: Freestyle")).toBeVisible();
  await expect(page.getByText("Next game: Renju")).toBeVisible();

  await page.getByRole("button", { name: "New Game" }).click();
  await expect(page.getByText("0 moves")).toBeVisible();
  await expect(page.getByText("Moves appear here as the game unfolds.")).toBeVisible();
  await expect(page.getByText("Current: Renju")).toBeVisible();

  await canvas.click({
    position: {
      x: box.width / 2,
      y: box.height / 2,
    },
  });

  await expect
    .poll(async () => page.locator("ol li").count())
    .toBeGreaterThan(0);
  await expect(page.getByText("2 moves")).toBeVisible({ timeout: 15_000 });
  await expect(page.locator("ol li")).toHaveCount(2);
});

test("board frame stays stable as move history grows", async ({ page }) => {
  await page.goto("/match/local");

  await expect(
    page.getByRole("heading", { name: "Local Match" }),
  ).toBeVisible();

  const canvas = page.locator("canvas").first();
  await expect(canvas).toBeVisible();

  const initialMetrics = await page.evaluate(() => {
    const frame = document.querySelector('[class*="frame"]');
    const canvas = document.querySelector("canvas");

    if (!frame || !canvas) {
      return null;
    }

    const frameBox = frame.getBoundingClientRect();
    const canvasBox = canvas.getBoundingClientRect();

    return {
      canvasHeight: canvasBox.height,
      frameHeight: frameBox.height,
    };
  });

  expect(initialMetrics).not.toBeNull();

  await page.evaluate(() => {
    const historyBody = document.querySelector('[class*="historyBody"]');
    if (!historyBody) {
      throw new Error("history body not found");
    }

    const filler = document.createElement("div");
    filler.setAttribute("data-testid", "history-growth-fixture");
    filler.style.display = "grid";
    filler.style.gap = "10px";
    filler.style.marginTop = "14px";

    for (let index = 0; index < 18; index += 1) {
      const row = document.createElement("div");
      row.textContent = `Fixture move ${index + 1}`;
      row.style.padding = "10px 12px";
      row.style.border = "1px solid rgba(255, 255, 255, 0.12)";
      row.style.background = "rgba(255, 255, 255, 0.04)";
      filler.appendChild(row);
    }

    historyBody.appendChild(filler);
  });

  await page.waitForTimeout(100);

  const finalMetrics = await page.evaluate(() => {
    const frame = document.querySelector('[class*="frame"]');
    const canvas = document.querySelector("canvas");
    const historyBody = document.querySelector('[class*="historyBody"]');
    const sidebar = document.querySelector('[class*="sidebar"]');

    if (!frame || !canvas || !historyBody || !sidebar) {
      return null;
    }

    const frameBox = frame.getBoundingClientRect();
    const canvasBox = canvas.getBoundingClientRect();

    return {
      canvasHeight: canvasBox.height,
      frameHeight: frameBox.height,
      historyBodyClientHeight: (historyBody as HTMLElement).clientHeight,
      historyBodyScrollHeight: (historyBody as HTMLElement).scrollHeight,
      pageClientHeight: document.documentElement.clientHeight,
      pageHeight: document.documentElement.scrollHeight,
      sidebarClientHeight: (sidebar as HTMLElement).clientHeight,
      sidebarScrollHeight: (sidebar as HTMLElement).scrollHeight,
    };
  });

  expect(finalMetrics).not.toBeNull();
  expect(Math.abs(finalMetrics!.frameHeight - initialMetrics!.frameHeight)).toBeLessThanOrEqual(2);
  expect(Math.abs(finalMetrics!.canvasHeight - initialMetrics!.canvasHeight)).toBeLessThanOrEqual(2);
  expect(finalMetrics!.pageHeight - finalMetrics!.pageClientHeight).toBeLessThanOrEqual(2);
  expect(finalMetrics!.sidebarScrollHeight - finalMetrics!.sidebarClientHeight).toBeLessThanOrEqual(2);
  expect(finalMetrics!.historyBodyScrollHeight).toBeGreaterThan(finalMetrics!.historyBodyClientHeight);
});

test("move history auto-scrolls to the latest move once it overflows", async ({ page }) => {
  await page.goto("/match/local");

  await expect(
    page.getByRole("heading", { name: "Local Match" }),
  ).toBeVisible();

  const canvas = page.locator("canvas").first();
  await expect(canvas).toBeVisible();

  const moves: Array<[number, number]> = [
    [0, 0],
    [2, 2],
    [4, 4],
    [6, 6],
    [8, 8],
  ];

  for (const [row, col] of moves) {
    const beforeCount = await page.locator("ol li").count();
    const box = await canvas.boundingBox();
    if (!box) {
      throw new Error("board canvas did not report a bounding box");
    }

    await canvas.click({ position: boardClickPosition(box, row, col) });
    await expect
      .poll(async () => page.locator("ol li").count(), { timeout: 15_000 })
      .toBeGreaterThan(beforeCount);
  }

  const metrics = await page.evaluate(() => {
    const historyBody = document.querySelector('[class*="historyBody"]') as HTMLElement | null;
    if (!historyBody) {
      return null;
    }

    return {
      clientHeight: historyBody.clientHeight,
      scrollHeight: historyBody.scrollHeight,
      scrollTop: historyBody.scrollTop,
    };
  });

  expect(metrics).not.toBeNull();
  expect(metrics!.scrollHeight).toBeGreaterThan(metrics!.clientHeight);
  expect(metrics!.scrollTop + metrics!.clientHeight).toBeGreaterThanOrEqual(metrics!.scrollHeight - 4);
});

test("direct entry to the local match route loads the app", async ({ page }) => {
  await page.goto("/match/local");

  await expect(
    page.getByRole("heading", { name: "Local Match" }),
  ).toBeVisible();
  await expect(page.getByText("Current: Freestyle")).toBeVisible();
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
  await page.setViewportSize({ width: 430, height: 932 });
  await page.goto("/match/local");

  await expect(
    page.getByRole("heading", { name: "Local Match" }),
  ).toBeVisible();

  const ratios = await page.evaluate(() => {
    const frame = document.querySelector('[class*="frame"]');
    const viewport = document.querySelector('[class*="viewport"]');
    const canvas = document.querySelector("canvas");

    if (!frame || !viewport || !canvas) {
      return null;
    }

    const frameBox = frame.getBoundingClientRect();
    const viewportBox = viewport.getBoundingClientRect();
    const canvasBox = canvas.getBoundingClientRect();

    return {
      canvasToFrame: Math.min(
        canvasBox.width / frameBox.width,
        canvasBox.height / frameBox.height,
      ),
      viewportToFrame: Math.min(
        viewportBox.width / frameBox.width,
        viewportBox.height / frameBox.height,
      ),
      squareDelta: Math.abs(canvasBox.width - canvasBox.height),
    };
  });

  expect(ratios).not.toBeNull();
  expect(ratios!.canvasToFrame).toBeGreaterThan(0.9);
  expect(ratios!.viewportToFrame).toBeGreaterThan(0.9);
  expect(ratios!.squareDelta).toBeLessThanOrEqual(1);
});

test("canvas stays matched to the board viewport after resizing into portrait", async ({ page }) => {
  await page.setViewportSize({ width: 1200, height: 800 });
  await page.goto("/match/local");
  await expect(
    page.getByRole("heading", { name: "Local Match" }),
  ).toBeVisible();

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
