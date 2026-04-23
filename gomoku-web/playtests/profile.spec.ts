import { expect, test } from "@playwright/test";

function moveCount(value: string | null): number {
  const match = value?.match(/\d+/);
  return match ? Number(match[0]) : NaN;
}

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

test("guest profile persists locally and captures finished local matches", async ({ page }) => {
  await page.setViewportSize({ width: 1024, height: 900 });
  await page.goto("/profile");

  await expect(page.getByRole("heading", { name: "Profile" })).toBeVisible();
  await expect(page.getByText("Reduced motion")).toHaveCount(0);
  await expect(page.getByText("Sound")).toHaveCount(0);

  const displayName = page.getByLabel("Display name");
  await displayName.fill("Bryan Guest");
  await page.reload();
  await expect(displayName).toHaveValue("Bryan Guest");
  await expect(page.getByText("Preferred rules")).toBeVisible();
  await page.getByRole("button", { name: "Renju" }).click();

  await page.getByRole("link", { name: "Play" }).click();
  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();
  await expect(page.getByText("Bryan Guest to move")).toBeVisible();
  await expect(page.getByTestId("match-rule")).toHaveText("Renju");

  const canvas = page.locator("canvas").first();
  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error("board canvas did not report a bounding box");
  }

  for (const [row, col] of [[0, 0], [2, 0], [4, 0], [6, 0], [8, 0]]) {
    const beforeCount = moveCount(await page.getByTestId("match-move-count").textContent());
    await canvas.click({ position: boardClickPosition(box, row, col) });
    await expect
      .poll(async () => moveCount(await page.getByTestId("match-move-count").textContent()), {
        timeout: 15_000,
      })
      .toBeGreaterThan(beforeCount);
  }

  await expect(page.getByTestId("match-move-count")).toHaveText("Move 10");
  await expect(page.getByText("Classic Bot wins")).toBeVisible();
  await page.getByRole("link", { name: "Profile" }).click();

  await expect(page.getByText("1 local match")).toBeVisible();
  await expect(page.getByText("Loss", { exact: true })).toBeVisible();
  await expect(page.getByText("Wins", { exact: true })).toBeVisible();
  await expect(page.getByText("Losses", { exact: true })).toBeVisible();
  await expect(page.getByText("Draws", { exact: true })).toBeVisible();
  await expect(page.getByText("vs Classic Bot")).toBeVisible();
  await expect(page.locator("ol li").first()).toContainText("Renju");
  const overflowMetrics = await page.evaluate(() => {
    const historyBody = document.querySelector('[class*="historyBody"]');

    return {
      historyBodyClientWidth: (historyBody as HTMLElement | null)?.clientWidth ?? 0,
      historyBodyScrollWidth: (historyBody as HTMLElement | null)?.scrollWidth ?? 0,
      pageClientWidth: document.documentElement.clientWidth,
      pageScrollWidth: document.documentElement.scrollWidth,
    };
  });

  expect(overflowMetrics.pageScrollWidth - overflowMetrics.pageClientWidth).toBeLessThanOrEqual(2);
  expect(overflowMetrics.historyBodyScrollWidth - overflowMetrics.historyBodyClientWidth).toBeLessThanOrEqual(2);

  await displayName.fill("Bryan Prime");
  await expect(displayName).toHaveValue("Bryan Prime");

  await page.getByRole("button", { name: "Reset local profile" }).click();
  await expect(displayName).toHaveValue("Guest");
  await expect(page.getByText("0 local matches")).toBeVisible();
});

test("profile history keeps summary pinned while the history list scrolls", async ({ page }) => {
  await page.goto("/profile");

  await expect(page.getByRole("heading", { name: "Profile" })).toBeVisible();
  await expect(page.getByText("Finished", { exact: true })).toBeVisible();
  await expect(page.getByText("Wins", { exact: true })).toBeVisible();
  await expect(page.getByText("Losses", { exact: true })).toBeVisible();
  await expect(page.getByText("Draws", { exact: true })).toBeVisible();

  await page.evaluate(() => {
    const historyBody = document.querySelector('[class*="historyBody"]');
    if (!historyBody) {
      throw new Error("history body not found");
    }

    const filler = document.createElement("div");
    filler.setAttribute("data-testid", "profile-history-growth-fixture");
    filler.style.display = "grid";
    filler.style.gap = "12px";
    filler.style.marginTop = "16px";

    for (let index = 0; index < 18; index += 1) {
      const row = document.createElement("div");
      row.textContent = `Fixture match ${index + 1}`;
      row.style.padding = "14px";
      row.style.border = "1px solid rgba(255, 255, 255, 0.12)";
      row.style.background = "rgba(255, 255, 255, 0.04)";
      filler.appendChild(row);
    }

    historyBody.appendChild(filler);
  });

  await page.waitForTimeout(100);

  const metrics = await page.evaluate(() => {
    const historyBody = document.querySelector('[class*="historyBody"]');

    if (!historyBody) {
      return null;
    }

    return {
      historyBodyClientHeight: (historyBody as HTMLElement).clientHeight,
      historyBodyScrollHeight: (historyBody as HTMLElement).scrollHeight,
      pageClientHeight: document.documentElement.clientHeight,
      pageHeight: document.documentElement.scrollHeight,
    };
  });

  expect(metrics).not.toBeNull();
  expect(metrics!.pageHeight - metrics!.pageClientHeight).toBeLessThanOrEqual(2);
  expect(metrics!.historyBodyScrollHeight).toBeGreaterThan(metrics!.historyBodyClientHeight);
});

test("portrait profile scrolls the page instead of the history pane", async ({ page }) => {
  await page.setViewportSize({ width: 430, height: 932 });
  await page.goto("/profile");

  await expect(page.getByRole("heading", { name: "Profile" })).toBeVisible();
  await expect(page.getByText("Local history")).toBeVisible();

  await page.evaluate(() => {
    const historyBody = document.querySelector('[class*="historyBody"]');
    if (!historyBody) {
      throw new Error("history body not found");
    }

    const filler = document.createElement("div");
    filler.setAttribute("data-testid", "profile-portrait-scroll-fixture");
    filler.style.display = "grid";
    filler.style.gap = "12px";
    filler.style.marginTop = "16px";

    for (let index = 0; index < 18; index += 1) {
      const row = document.createElement("div");
      row.textContent = `Fixture match ${index + 1}`;
      row.style.padding = "14px";
      row.style.border = "1px solid rgba(255, 255, 255, 0.12)";
      row.style.background = "rgba(255, 255, 255, 0.04)";
      filler.appendChild(row);
    }

    historyBody.appendChild(filler);
  });

  const before = await page.evaluate(() => {
    const header = document.querySelector("header");
    const historyBody = document.querySelector('[class*="historyBody"]');
    const layout = document.querySelector('[class*="layout"]');
    const recordHeader = document.querySelector('[class*="recordHeader"]');

    if (!header || !historyBody || !layout || !recordHeader) {
      return null;
    }

    return {
      bodyOverflowY: window.getComputedStyle(document.body).overflowY,
      headerTop: header.getBoundingClientRect().top,
      recordHeaderBottom: recordHeader.getBoundingClientRect().bottom,
      historyOverflowY: window.getComputedStyle(historyBody).overflowY,
      layoutOverflowY: window.getComputedStyle(layout).overflowY,
      viewportHeight: window.innerHeight,
    };
  });

  expect(before).not.toBeNull();
  expect(before!.bodyOverflowY).toBe("auto");
  expect(before!.recordHeaderBottom).toBeLessThanOrEqual(before!.viewportHeight);
  expect(before!.historyOverflowY).toBe("visible");
  expect(before!.layoutOverflowY).toBe("visible");

  await page.evaluate(() => window.scrollTo(0, 200));
  await page.waitForTimeout(50);
  await expect
    .poll(async () =>
      page.evaluate(() => document.querySelector("header")?.getBoundingClientRect().top ?? 0),
    )
    .toBeLessThan(before!.headerTop - 20);
});
