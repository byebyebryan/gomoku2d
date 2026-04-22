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
    .toBe("2|Guest to move");
}

test("home boot and local bot match smoke flow", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Gomoku2D" })).toBeVisible();
  await expect(page.getByText(/five in a row/i)).toBeVisible();

  await page.getByRole("link", { name: "Play" }).click();

  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();
  await expect(page.getByTestId("match-move-count")).toHaveText("0");
  await expect(page.getByTestId("match-rule")).toHaveText("Freestyle");
  await expect(page.getByTestId("match-status")).toHaveText("Guest to move");

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

  await page.getByRole("button", { name: "Renju" }).click();
  await expect(page.getByTestId("match-rule")).toHaveText("Freestyle");
  await expect(page.getByTestId("match-next-rule")).toHaveText("Renju");

  await page.getByRole("button", { name: "New Game" }).click();
  await expect(page.getByTestId("match-move-count")).toHaveText("0");
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
  await page.setViewportSize({ width: 430, height: 932 });
  await page.goto("/match/local");

  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();

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
