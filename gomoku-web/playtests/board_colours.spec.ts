import { expect, test } from "@playwright/test";
import { expectBoardRenderer, expectCellColour, observeBlackAnimation, prepareBoardPixels } from "./helpers/board_pixels.js";

const MOVES = [110, 0, 111, 1, 112, 2, 113, 3, 114];
const BLACK = [64, 64, 64] as const;
const WHITE = [255, 255, 255] as const;

function sharedLink(ply: number): string {
  const wire = { v: 1, r: "freestyle", m: Buffer.from(MOVES).toString("base64url"),
    b: { k: "h", n: "Black" }, w: { k: "h", n: "White" }, at: ply };
  return `/replay/shared/#g2r1=${Buffer.from(JSON.stringify(wire)).toString("base64url")}`;
}

for (const mode of ["canvas", "webgl"] as const) {
  test.describe(`board colours with verified ${mode}`, () => {
    test.beforeEach(async ({ page, browserName }) => {
      // Chromium qualifies WebGL explicitly; Canvas is checked in every engine.
      test.skip(mode === "webgl" && browserName !== "chromium", "WebGL pixel qualification uses Chromium; all engines cover forced Canvas");
      await prepareBoardPixels(page, mode);
    });

    test("replay stones keep their colours through focus, removal, rebuild and reduced motion", async ({ page }, testInfo) => {
      await page.goto(sharedLink(4));
      const canvas = page.locator("canvas");
      await expectBoardRenderer(canvas, mode);
      await expectCellColour(canvas, 7, 5, BLACK);
      await expectCellColour(canvas, 0, 0, WHITE);
      await page.getByRole("button", { name: "Next move", exact: true }).click();
      await expect(page.getByTestId("replay-move-count")).toHaveText("Move 5 / 9");
      await expectCellColour(canvas, 7, 7, BLACK);
      await page.emulateMedia({ reducedMotion: "no-preference" });
      const animation = await observeBlackAnimation(canvas, 7, 7, 2300);
      expect(animation.frames).toBeGreaterThan(8);
      expect(animation.distinctFrames).toBeGreaterThan(1);
      expect(animation.maximumGrey).toBeLessThanOrEqual(65);
      expect(animation.blackPixels).toBeGreaterThan(0);
      expect(animation.glError).toBe(0);
      await testInfo.attach("black-replay-animation", { body: JSON.stringify(animation), contentType: "application/json" });
      await page.getByRole("button", { name: "Previous move", exact: true }).click();
      await expect(page.getByTestId("replay-move-count")).toHaveText("Move 4 / 9");
      const removal = await observeBlackAnimation(canvas, 7, 7, 450);
      expect(removal.maximumGrey).toBeLessThanOrEqual(65);
      await page.getByRole("button", { name: "Next move", exact: true }).click();
      await page.emulateMedia({ reducedMotion: "reduce" });
      await expectCellColour(canvas, 7, 7, BLACK);
      await page.setViewportSize({ width: 600, height: 900 });
      await expectCellColour(canvas, 7, 5, BLACK);
      await expectCellColour(canvas, 0, 0, WHITE);
    });

    test("teaching diagrams retain tactical, winning and forbidden colours", async ({ page }) => {
      await page.goto("/guide/");
      for (const [name, row, col, colour] of [
        ["Immediate threat diagram: Black has four stones and one winning target.", 3, 6, [255, 68, 68]],
        ["Imminent threat diagram: Black has an open three with two replies.", 3, 1, [255, 122, 182]],
        ["Counter threat diagram: White answers an open three by making a four.", 1, 3, [184, 119, 255]],
      ] as const) {
        const canvas = page.getByRole("img", { name, exact: true }).locator("canvas");
        await expectBoardRenderer(canvas, mode);
        await expectCellColour(canvas, row, col, colour, 7);
      }
      const focus = page.getByRole("img", { name: "Forced corridor sequence: 1. Forced reply 1 board diagram.", exact: true }).locator("canvas");
      await expectCellColour(focus, 1, 3, BLACK, 7);
      await expectCellColour(focus, 0, 3, WHITE, 7);
      await page.goto("/rules/");
      const forbidden = page.getByRole("img", { name: "Renju forbidden move examples: Overline board diagram.", exact: true }).locator("canvas");
      await expectBoardRenderer(forbidden, mode);
      await expectCellColour(forbidden, 3, 5, [255, 68, 68], 7);
      const winning = page.getByRole("img", { name: "Freestyle win diagram: Black has five connected stones.", exact: true }).locator("canvas");
      await expectCellColour(winning, 3, 3, [68, 221, 68], 7);
    });

    test("both player pointers and an animated new Black stone use the player colour", async ({ page }, testInfo) => {
      for (const [ply, colour] of [[5, WHITE], [4, BLACK]] as const) {
        await page.goto(sharedLink(ply));
        await page.getByRole("button", { name: "Play From Here", exact: true }).click();
        await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();
        const canvas = page.locator("canvas");
        await expectBoardRenderer(canvas, mode);
        // The semantic grid passes physical pointer events to Phaser's canvas.
        await page.getByRole("gridcell", { name: /^F7:/ }).hover({ force: true });
        await expectCellColour(canvas, 6, 5, colour);
      }
      await page.emulateMedia({ reducedMotion: "no-preference" });
      const placementCell = page.getByRole("gridcell", { name: /^H8:/ });
      await placementCell.focus();
      await placementCell.press("Enter");
      const canvas = page.locator("canvas");
      const placement = await observeBlackAnimation(canvas, 7, 7, 3000);
      expect(placement.frames).toBeGreaterThan(8);
      expect(placement.maximumGrey).toBeLessThanOrEqual(65);
      expect(placement.blackPixels).toBeGreaterThan(0);
      expect(placement.glError).toBe(0);
      await testInfo.attach("black-placement-animation", { body: JSON.stringify(placement), contentType: "application/json" });
      await page.emulateMedia({ reducedMotion: "reduce" });
      await expectCellColour(canvas, 7, 7, BLACK);
    });
  });
}
