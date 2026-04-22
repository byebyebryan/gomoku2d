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

test("finished board click advances to the next round with swapped colors", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("link", { name: "Play" }).click();

  const canvas = page.locator("canvas").first();
  await expect(canvas).toBeVisible();

  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error("board canvas did not report a bounding box");
  }

  const losingMoves: Array<[number, number]> = [
    [0, 0],
    [2, 0],
    [4, 0],
    [6, 0],
    [8, 0],
  ];

  for (const [row, col] of losingMoves) {
    const beforeCount = Number((await page.getByTestId("match-move-count").textContent()) ?? "0");
    await canvas.click({ position: boardClickPosition(box, row, col) });
    await expect
      .poll(async () => Number((await page.getByTestId("match-move-count").textContent()) ?? "0"), {
        timeout: 15_000,
      })
      .toBeGreaterThan(beforeCount);
  }

  await expect(page.getByText("Classic Bot wins")).toBeVisible();

  await canvas.click({ position: boardClickPosition(box, 7, 7) });

  await expect(page.getByTestId("player-row-black")).toContainText("Classic Bot");
  await expect(page.getByTestId("player-row-black")).toContainText("Bot");
  await expect(page.getByTestId("player-row-white")).toContainText("Guest");
  await expect(page.getByTestId("player-row-white")).toContainText("Player");
  await expect(page.getByTestId("match-move-count")).toHaveText(/^[01]$/);
});
