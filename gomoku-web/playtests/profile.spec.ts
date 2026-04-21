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

test("guest profile persists locally and captures finished local matches", async ({ page }) => {
  await page.goto("/profile");

  await expect(page.getByRole("heading", { name: "Profile" })).toBeVisible();

  const displayName = page.getByLabel("Display name");
  await displayName.fill("Bryan Guest");
  await page.reload();
  await expect(displayName).toHaveValue("Bryan Guest");

  await page.getByRole("link", { name: "Play Bot" }).click();
  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();
  await expect(page.getByText("Bryan Guest")).toBeVisible();

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

  await expect(page.getByText("1 local match")).toBeVisible();
  await expect(page.getByText("Search Bot wins")).toBeVisible();
  await expect(page.getByText("Wins", { exact: true })).toBeVisible();
  await expect(page.getByText("Losses", { exact: true })).toBeVisible();
  await expect(page.getByText("Draws", { exact: true })).toBeVisible();
  await expect(page.getByText("Bryan Guest (black) vs Search Bot (white)")).toBeVisible();

  await displayName.fill("Bryan Prime");
  await expect(page.getByText("Bryan Prime (black) vs Search Bot (white)")).toBeVisible();

  await page.getByRole("button", { name: "Reset local profile" }).click();
  await expect(displayName).toHaveValue("Guest");
  await expect(page.getByText("0 local matches")).toBeVisible();
});
