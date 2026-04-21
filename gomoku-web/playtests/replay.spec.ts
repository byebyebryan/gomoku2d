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

test("local replay opens from profile history and supports stepping plus autoplay", async ({ page }) => {
  await page.goto("/profile");

  const displayName = page.getByLabel("Display name");
  await displayName.fill("Bryan Guest");

  await page.getByRole("link", { name: "Play Bot" }).click();
  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();

  const canvas = page.locator("canvas").first();
  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error("board canvas did not report a bounding box");
  }

  for (const [row, col] of [[0, 0], [2, 0], [4, 0], [6, 0], [8, 0]]) {
    const beforeCount = await page.locator("ol li").count();
    await canvas.click({ position: boardClickPosition(box, row, col) });
    await expect
      .poll(async () => page.locator("ol li").count(), { timeout: 15_000 })
      .toBeGreaterThan(beforeCount);
  }

  await expect(page.getByText("White wins")).toBeVisible();
  await page.getByRole("link", { name: "Profile" }).click();

  await page.getByRole("button", { name: "Open replay" }).first().click();

  await expect(page.getByRole("heading", { name: "Replay" })).toBeVisible();
  await expect(page.getByText("Move 0 / 10")).toBeVisible();
  await expect(page.getByText("Bryan Guest (black) vs Search Bot (white)")).toBeVisible();
  await expect(page.locator('[data-testid="replay-step-controls"] button')).toHaveText([
    "Start",
    "End",
    "Previous move",
    "Next move",
  ]);

  await page.getByRole("button", { name: "Next move" }).click();
  await expect(page.getByText("Move 1 / 10")).toBeVisible();
  await page.getByRole("button", { name: "End" }).click();
  await expect(page.getByText("Move 10 / 10")).toBeVisible();
  await page.getByRole("button", { name: "Start" }).click();
  await expect(page.getByText("Move 0 / 10")).toBeVisible();

  await page.getByRole("button", { name: "Auto play" }).click();
  await expect(page.getByRole("button", { name: "Pause" })).toBeVisible();
  await expect
    .poll(async () => page.getByText(/Move \d+ \/ 10/).textContent(), { timeout: 15_000 })
    .toBe("Move 10 / 10");
  await expect(page.getByText("Search Bot wins")).toBeVisible();
  await expect(page.getByRole("button", { name: "Auto play" })).toBeVisible();
});
