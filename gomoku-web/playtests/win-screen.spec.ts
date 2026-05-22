import { expect, test } from "@playwright/test";

import { seedLocalSavedMatch } from "./helpers/local_history.js";

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

test("finished board click advances to the next round with swapped colors", async ({ page }) => {
  await page.goto("/profile");
  await seedLocalSavedMatch(page, {
    displayName: "Guest",
    id: "fixture-next-round-source",
    moves: [
      { col: 7, row: 7 },
      { col: 0, row: 0 },
      { col: 8, row: 7 },
      { col: 0, row: 1 },
      { col: 9, row: 7 },
      { col: 0, row: 2 },
      { col: 10, row: 7 },
      { col: 0, row: 3 },
      { col: 11, row: 7 },
    ],
    preferredVariant: "freestyle",
    savedAt: "2026-05-18T05:00:00.000Z",
    status: "black_won",
    variant: "freestyle",
  });

  await page.goto("/replay/fixture-next-round-source");
  await expect(page.getByRole("heading", { name: "Replay" })).toBeVisible();
  await page.getByRole("button", { name: "Previous move" }).click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 8 / 9");
  await page.getByRole("button", { name: "Play From Here" }).click();
  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();

  const canvas = page.locator("canvas").first();
  await expect(canvas).toBeVisible();

  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error("board canvas did not report a bounding box");
  }

  const beforeCount = moveCount(await page.getByTestId("match-move-count").textContent());
  await canvas.click({ position: boardClickPosition(box, 7, 11) });
  await expect
    .poll(async () => moveCount(await page.getByTestId("match-move-count").textContent()), {
      timeout: 15_000,
    })
    .toBeGreaterThan(beforeCount);
  await expect(page.getByTestId("match-status")).toHaveText("Guest wins");

  await canvas.click({ position: boardClickPosition(box, 7, 7) });

  await expect(page.getByTestId("player-row-black")).toContainText("Normal Bot");
  await expect(page.getByTestId("player-row-white")).toContainText("Guest");
  await expect(page.getByTestId("player-row-black").getByRole("img", { name: "Bot" })).toBeVisible();
  await expect(page.getByTestId("player-row-white").getByRole("img", { name: "Player" })).toBeVisible();
  await expect(page.getByTestId("match-move-count")).toHaveText(/^Move [01]$/);
});
