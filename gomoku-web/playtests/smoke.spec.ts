import { expect, test } from "@playwright/test";

test("home boot and local bot match smoke flow", async ({ page }) => {
  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: "Gomoku2D" }),
  ).toBeVisible();
  await expect(page.getByText(/react shell/i)).toBeVisible();

  await page.getByRole("link", { name: "Play Bot" }).click();

  await expect(
    page.getByRole("heading", { name: "Local Match" }),
  ).toBeVisible();
  await expect(page.getByText("0 moves")).toBeVisible();
  await expect(page.getByText("Black to move")).toBeVisible();

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

  await expect(page.getByText("1 moves")).toBeVisible();
  await expect(page.getByText("2 moves")).toBeVisible({ timeout: 15_000 });
  await expect(page.locator("ol li")).toHaveCount(2);
  await expect(page.getByText("Black to move")).toBeVisible();
});
