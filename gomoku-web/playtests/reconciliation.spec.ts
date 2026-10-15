import { expect, test } from "@playwright/test";

import { seedLocalSavedMatch } from "./helpers/local_history.js";

test("a keyboard player can branch, finish, analyze, and find the saved game", async ({ page }, testInfo) => {
  await page.goto("/");
  await seedLocalSavedMatch(page, {
    displayName: "Guest",
    id: "keyboard-source",
    moves: [
      { row: 7, col: 7 }, { row: 0, col: 0 },
      { row: 7, col: 8 }, { row: 1, col: 0 },
      { row: 7, col: 9 }, { row: 2, col: 0 },
      { row: 7, col: 10 }, { row: 3, col: 0 },
      { row: 7, col: 11 },
    ],
    preferredVariant: "freestyle",
    variant: "freestyle",
    savedAt: "2026-10-15T00:00:00.000Z",
    status: "black_won",
  });
  await page.goto("/replay/keyboard-source");
  const previous = page.getByRole("button", { name: "Previous move", exact: true });
  await previous.focus();
  await previous.press("Enter");
  const branch = page.getByRole("button", { name: "Play From Here" });
  await branch.focus();
  await branch.press("Enter");
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 8");

  const center = page.getByRole("gridcell", { name: /^H8:/ });
  const home = page.getByRole("link", { name: "Home", exact: true });
  await home.focus();
  await home.press("Tab");
  await expect(center).toBeFocused();
  await expect(center).toHaveAttribute("aria-disabled", "true");
  await center.press("Enter");
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 8");
  await center.press("End");
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.press("ArrowLeft");
  const winningCell = page.getByRole("gridcell", { name: /^L8:/ });
  await expect(winningCell).toBeFocused();
  await winningCell.press("Enter");
  await expect(page.getByTestId("match-status")).toHaveText("Guest wins");
  await expect(winningCell).toHaveAccessibleName("L8: Black stone");
  await page.screenshot({ path: testInfo.outputPath("keyboard-finish.png") });

  const analyze = page.getByRole("link", { name: "Analyze", exact: true });
  await analyze.focus();
  await analyze.press("Enter");
  await expect(page).toHaveTitle("Replay Analysis | Gomoku2D");
  await expect(page.getByRole("grid", { name: "Gomoku board" })).toHaveAttribute("aria-readonly", "true");
  const profile = page.getByRole("link", { name: "Profile", exact: true });
  await profile.focus();
  await profile.press("Enter");
  await expect(page.getByRole("button", { name: "Inspect" })).toHaveCount(2);
});

test("settings expose their selected rule, bot, and hints", async ({ page }) => {
  await page.goto("/settings");
  const freestyle = page.getByRole("button", { name: "Freestyle", exact: true });
  const renju = page.getByRole("button", { name: "Renju", exact: true });
  await expect(freestyle).toHaveAttribute("aria-pressed", "true");
  await expect(renju).toHaveAttribute("aria-pressed", "false");
  await renju.focus();
  await renju.press("Space");
  await expect(renju).toHaveAttribute("aria-pressed", "true");
  await expect(freestyle).toHaveAttribute("aria-pressed", "false");
  await expect(page.getByRole("button", { name: /^Normal/ })).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByRole("group", { name: "Evidence hints" }).getByRole("button", { name: "On", exact: true }))
    .toHaveAttribute("aria-pressed", "true");
});

for (const failure of ["denied getter", "write quota", "corrupt document", "newer document"] as const) {
  test(`local play survives ${failure} and explains session-only saves`, async ({ page }, testInfo) => {
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.addInitScript((mode) => {
      if (mode === "denied getter") {
        Object.defineProperty(window, "localStorage", {
          configurable: true,
          get: () => { throw new DOMException("Storage denied", "SecurityError"); },
        });
      } else if (mode === "write quota") {
        Storage.prototype.setItem = () => { throw new DOMException("Storage full", "QuotaExceededError"); };
      } else {
        localStorage.setItem("gomoku2d.local-profile.v5", mode === "corrupt document"
          ? "{not-json" : '{"state":{},"version":6}');
      }
    }, failure);
    await page.goto("/match/local");
    await expect(page.getByRole("status").filter({ hasText: "Local saves are unavailable" })).toBeVisible();
    const center = page.getByRole("gridcell", { name: /^H8:/ });
    await center.focus();
    await center.press("Enter");
    await expect(page.getByTestId("match-move-count")).toHaveText("Move 2");
    await expect(page.getByRole("button", { name: "Undo" })).toBeInViewport();
    expect(errors).toEqual([]);
    if (failure === "corrupt document" || failure === "newer document") {
      expect(await page.evaluate(() => localStorage.getItem("gomoku2d.local-profile.v5")))
        .toBe(failure === "corrupt document" ? "{not-json" : '{"state":{},"version":6}');
    }
    if (failure === "denied getter") {
      await page.screenshot({ path: testInfo.outputPath("storage-fallback.png") });
    }
  });
}

for (const width of [1440, 430]) {
  test(`a repeated bot worker crash offers usable recovery at ${width}px`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: width === 430 ? 932 : 1100 });
    const workerUrl = /\/bot_worker-[^/]+\.js$/;
    await page.route(workerUrl, (route) => route.fulfill({
      contentType: "text/javascript",
      body: `self.onmessage = ({data}) => {
        if (data.type === 'choose_move') setTimeout(() => { throw new Error('Fixture worker crash'); }, 0);
      }; self.postMessage({type: 'ready'});`,
    }));
    await page.goto("/match/local");
    const center = page.getByRole("gridcell", { name: /^H8:/ });
    await center.focus();
    await center.press("Enter");
    await expect(page.getByRole("alert")).toContainText("Retry this turn");
    await expect(page.getByTestId("match-move-count")).toHaveText("Move 1");
    await expect(page.getByRole("button", { name: "Retry bot" })).toBeVisible();
    await page.screenshot({ path: testInfo.outputPath(`bot-recovery-${width}.png`) });
    await page.unroute(workerUrl);
    await page.getByRole("button", { name: "Retry bot" }).click();
    await expect(page.getByTestId("match-move-count")).toHaveText("Move 2");
    await expect(page.getByRole("button", { name: "Retry bot" })).toHaveCount(0);
    await expect(page.getByRole("alert")).toHaveCount(0);
  });
}

test("the published Lab keeps possible findings and escapes qualified", async ({ page }) => {
  await page.goto("/lab/?tab=analysis");
  const analysis = page.locator("[data-view='analysis']");
  await analysis.locator("summary").first().click();
  const game = analysis.locator("summary", { hasText: "#65" });
  await expect(game).toContainText("possible: missed forbidden 4");
  await game.click();
  await expect(analysis.getByText("Black to respond / possible escape", { exact: true })).toBeVisible();
});
