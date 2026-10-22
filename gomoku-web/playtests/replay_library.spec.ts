import { expect, test, type Page, type TestInfo } from "@playwright/test";
import axe from "axe-core";

const WIN_MOVES = [110, 0, 111, 1, 112, 2, 113, 3, 114];

function sharedLink(viewPly: number): string {
  const wire = {
    v: 1,
    r: "freestyle",
    m: Buffer.from(WIN_MOVES).toString("base64url"),
    b: { k: "h", n: "Alice" },
    w: { k: "h", n: "Bob" },
    at: viewPly,
  };
  return `/replay/shared/#g2r1=${Buffer.from(JSON.stringify(wire)).toString("base64url")}`;
}

async function expectProfileStat(page: Page, label: string, value: number) {
  const tile = page.locator('[class*="summaryTile"]').filter({ hasText: label });
  await expect(tile.locator('[class*="summaryValue"]')).toHaveText(String(value));
}

async function expectNoHorizontalOverflow(page: Page) {
  const overflow = await page.evaluate(() => (
    document.documentElement.scrollWidth - document.documentElement.clientWidth
  ));
  expect(overflow).toBeLessThanOrEqual(2);
}

async function stabilizeScreenshot(page: Page) {
  await page.evaluate(async () => {
    await document.fonts.ready;
    await new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
  });
}

async function auditAccessibility(page: Page, testInfo: TestInfo, stateName: string) {
  await page.evaluate(() => document.fonts.ready);
  await page.addScriptTag({ content: axe.source });
  const result = await page.evaluate(async () => {
    const report = await axe.run(document, {
      runOnly: {
        type: "tag",
        values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "best-practice"],
      },
    });
    return {
      testEngine: report.testEngine,
      url: report.url,
      violations: report.violations,
      // Inconclusive checks remain available for manual follow-up.
      incomplete: report.incomplete,
    };
  });
  await testInfo.attach(`accessibility-audit-${stateName}`, {
    body: JSON.stringify(result, null, 2), contentType: "application/json",
  });
  expect(result.violations).toEqual([]);
}

async function captureChromiumScreenshot(page: Page, screenshotPath: string, testInfo: TestInfo) {
  await stabilizeScreenshot(page);
  if (page.context().browser()?.browserType().name() === "chromium") {
    await page.screenshot({ path: process.env.GOMOKU_WRITE_EVIDENCE === "1"
      ? screenshotPath : testInfo.outputPath(screenshotPath.split("/").pop()!), fullPage: true });
  }
}

async function captureEvidence(
  page: Page,
  testInfo: TestInfo,
  stateName: string,
  screenshotPath: string,
) {
  await auditAccessibility(page, testInfo, stateName);
  await captureChromiumScreenshot(page, screenshotPath, testInfo);
}

test("shared replays save once across viewing plies, survive reload, and remove independently of played history", async ({ page }, testInfo) => {
  await page.goto(sharedLink(8));
  await expect(page.getByRole("heading", { name: "Replay Analysis" })).toBeVisible();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 8 / 9");

  await page.getByRole("button", { name: "Save replay", exact: true }).click();
  await expect(page.getByRole("button", { name: "Saved replay", exact: true })).toBeDisabled();
  await expect(page.getByText("Saved on this device", { exact: true })).toBeVisible();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 8 / 9");

  // Viewing another moment changes the shared URL, not the saved game's identity.
  await page.goto(sharedLink(3));
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 3 / 9");
  await expect(page.getByRole("button", { name: "Saved replay", exact: true })).toBeDisabled();

  await page.goto("/profile");
  await expect(page.getByRole("heading", { name: "Profile", exact: true })).toBeVisible();
  await expect(page.getByRole("tab", { name: "My games", exact: true })).toHaveAttribute("aria-selected", "true");
  await page.setViewportSize({ width: 390, height: 844 });
  await expectNoHorizontalOverflow(page);

  await page.reload();
  await expect(page.getByRole("heading", { name: "Profile", exact: true })).toBeVisible();
  await page.getByRole("tab", { name: "Saved replays", exact: true }).click();
  await expect(page.getByRole("link", { name: "Inspect replay", exact: true })).toHaveCount(1);
  await expect(page.getByRole("button", { name: "Remove replay", exact: true })).toHaveCount(1);
  await expectNoHorizontalOverflow(page);
  await captureEvidence(
    page, testInfo, "saved-replays-mobile",
    "../docs/working/v0_6_evidence/portable_replay_library_mobile.png",
  );

  await page.setViewportSize({ width: 1440, height: 1100 });
  await expect(page.getByRole("tab", { name: "My games", exact: true })).toBeVisible();
  await expect(page.getByRole("tab", { name: "Saved replays", exact: true })).toBeVisible();
  await expect(page.getByRole("tab", { name: "Saved replays", exact: true })).toHaveAttribute("aria-selected", "true");
  await captureEvidence(
    page, testInfo, "saved-replays-desktop",
    "../docs/working/v0_6_evidence/portable_replay_library_desktop.png",
  );

  await page.getByRole("button", { name: "Remove replay", exact: true }).click();
  await expect(page.getByRole("link", { name: "Inspect replay", exact: true })).toHaveCount(0);
  await page.getByRole("tab", { name: "My games", exact: true }).click();
  await expectProfileStat(page, "Matches", 0);
  await expectProfileStat(page, "Wins", 0);
});

test("playing from a shared frame creates a separate practice replay with its source attribution", async ({ page }, testInfo) => {
  await page.goto(sharedLink(8));
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 8 / 9");
  await expect(page.getByTestId("replay-player-row-black")).toContainText("Alice");
  await expect(page.getByTestId("replay-player-row-white")).toContainText("Bob");
  await captureEvidence(
    page, testInfo, "shared-replay-desktop",
    "../docs/working/v0_6_evidence/portable_replay_shared_desktop.png",
  );

  await page.setViewportSize({ width: 390, height: 844 });
  await captureEvidence(
    page, testInfo, "shared-replay-mobile",
    "../docs/working/v0_6_evidence/portable_replay_shared_mobile.png",
  );
  await expectNoHorizontalOverflow(page);

  await page.getByRole("button", { name: "Save replay", exact: true }).click();
  await expect(page.getByRole("button", { name: "Saved replay", exact: true })).toBeDisabled();
  await page.goto("/profile");
  await page.getByRole("tab", { name: "Saved replays", exact: true }).click();
  const sourceHref = await page.getByRole("link", { name: "Inspect replay", exact: true }).getAttribute("href");
  const sourceFingerprint = sourceHref?.match(/\/replay\/portable-([A-Za-z0-9_-]{43})$/)?.[1];
  expect(sourceFingerprint).toBeTruthy();

  await page.goto(sharedLink(8));
  const dialogs: string[] = [];
  page.on("dialog", async (dialog) => {
    dialogs.push(dialog.message());
    await dialog.dismiss();
  });
  await page.getByRole("button", { name: "Play From Here", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();
  expect(dialogs).toEqual([]);
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(page.getByText(/practice|inherited/i).first()).toBeVisible();
  await expectNoHorizontalOverflow(page);

  // The accessible board starts focused at H8. Two right-arrow moves reach J8,
  // the winning fifth stone, and Enter places it without a pointer gesture.
  const board = page.getByRole("grid", { name: "Gomoku board" });
  await board.getByRole("gridcell", { name: "H8: Black stone" }).focus();
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("ArrowRight");
  const winningCell = board.getByRole("gridcell", { name: "J8: empty" });
  await expect(winningCell).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 9");
  await expect(page.getByRole("link", { name: "Analyze" })).toBeVisible();

  await page.getByRole("link", { name: "Analyze" }).click();
  await expect(page).toHaveURL(/\/replay\/portable-[A-Za-z0-9_-]{43}$/);
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 9 / 9");
  await expect(page.getByText(/first 8 moves are inherited from another game/i)).toBeVisible();
  await expect(page.getByTestId("replay-player-row-black")).toContainText("Guest");
  await expect(page.getByTestId("replay-player-row-white")).toContainText("Normal Bot");

  await page.getByRole("button", { name: "Share replay", exact: true }).click();
  const replayLink = await page.getByRole("textbox", { name: "Replay link", exact: true }).inputValue();
  const fragment = new URL(replayLink).hash.slice("#g2r1=".length);
  const sharedWire = JSON.parse(Buffer.from(fragment, "base64url").toString("utf8")) as {
    b: { n: string };
    m: string;
    o: { f: string; p: number };
    w: { n: string };
  };
  expect(sharedWire.b.n).toBe("Guest");
  expect(sharedWire.w.n).toBe("Normal Bot");
  expect(sharedWire.o).toEqual({ f: sourceFingerprint, p: 8 });
  expect(Array.from(Buffer.from(sharedWire.m, "base64url"))).toEqual(WIN_MOVES);

  const sourcePage = await page.context().newPage();
  await sourcePage.goto(sharedLink(8));
  await expect(sourcePage.getByTestId("replay-move-count")).toHaveText("Move 8 / 9");
  await expect(sourcePage.getByTestId("replay-player-row-black")).toContainText("Alice");
  await expect(sourcePage.getByTestId("replay-player-row-white")).toContainText("Bob");
  await sourcePage.close();

  await page.goto("/profile");
  await expect(page.getByRole("tab", { name: "My games", exact: true })).toHaveAttribute("aria-selected", "true");
  await expect(page.getByRole("heading", { name: "Practice games", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "Inspect replay", exact: true })).toHaveCount(1);
  await expectProfileStat(page, "Matches", 0);
  await expectProfileStat(page, "Wins", 0);
  await expectNoHorizontalOverflow(page);
  await captureChromiumScreenshot(page, "../docs/working/v0_6_evidence/portable_replay_practice_mobile.png", testInfo);

  await page.setViewportSize({ width: 1440, height: 1100 });
  await captureChromiumScreenshot(page, "../docs/working/v0_6_evidence/portable_replay_practice_desktop.png", testInfo);

  await page.getByRole("tab", { name: "Saved replays", exact: true }).click();
  const sourceEntry = page.getByRole("listitem").filter({ hasText: /Alice.*Bob.*9 moves/ });
  await expect(sourceEntry).toBeVisible();
  await sourceEntry.getByRole("button", { name: "Remove replay", exact: true }).click();
  await expect(page.getByRole("link", { name: "Inspect replay", exact: true })).toHaveCount(0);

  await page.getByRole("tab", { name: "My games", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Practice games", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "Inspect replay", exact: true })).toHaveCount(1);
  await expectProfileStat(page, "Matches", 0);
  await expectProfileStat(page, "Wins", 0);
});
