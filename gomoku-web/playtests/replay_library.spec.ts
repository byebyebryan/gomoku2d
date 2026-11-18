import { createHash } from "node:crypto";
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

  page.once("dialog", (dialog) => dialog.accept());
  await page.getByRole("button", { name: "Remove replay", exact: true }).click();
  await expect(page.getByRole("link", { name: "Inspect replay", exact: true })).toHaveCount(0);
  await page.getByRole("tab", { name: "My games", exact: true }).click();
  await expectProfileStat(page, "Matches", 0);
  await expectProfileStat(page, "Wins", 0);
});

test("a replay kept after a storage failure is clearly temporary and disappears on reload", async ({ page }, testInfo) => {
  await page.addInitScript(() => {
    const setItem = Storage.prototype.setItem;
    Storage.prototype.setItem = function (key, value) {
      if (key === "gomoku2d.replay-library.v2") {
        throw new DOMException("Storage full", "QuotaExceededError");
      }
      setItem.call(this, key, value);
    };
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(sharedLink(8));
  await page.getByRole("button", { name: "Save replay", exact: true }).click();

  await expect(page.getByRole("button", { name: "Saved for this session", exact: true })).toBeDisabled();
  await expect(page.getByText(/lost when you reload or close this tab/i)).toBeVisible();
  await expect(page.getByText(/keep the replay link/i)).toBeVisible();
  await expect(page.getByRole("alert")).toContainText("Storage full");
  await expect(page.getByText("Saved on this device", { exact: true })).toHaveCount(0);
  await expectNoHorizontalOverflow(page);
  await auditAccessibility(page, testInfo, "session-only-replay-mobile");

  await page.reload();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 8 / 9");
  await expect(page.getByRole("button", { name: "Save replay", exact: true })).toBeEnabled();
  expect(await page.evaluate(() => localStorage.getItem("gomoku2d.replay-library.v2"))).toBeNull();
});

test("private details follow a saved replay and do not enter a portable link", async ({ page }, testInfo) => {
  await page.goto(sharedLink(8));
  await page.getByRole("button", { name: "Save replay", exact: true }).click();
  await expect(page.getByText("Saved on this device", { exact: true })).toBeVisible();
  await page.goto("/profile?tab=saved");
  const inspect = page.getByRole("link", { name: "Inspect replay", exact: true });
  await inspect.click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 9 / 9");
  await page.getByRole("button", { name: "Edit details", exact: true }).click();
  const details = page.getByRole("region", { name: "Replay details", exact: true });
  await expect(details).toBeVisible();
  await expect(details.getByTestId("replay-details-opening")).toHaveText("Final move");
  await details.getByLabel("Private title").fill("Position to study");
  await details.getByLabel("Private note").fill("Review White's reply.\n🟡 Then find a defense.");
  await details.getByRole("slider", { name: "Replay timeline" }).press("Home");
  await details.getByRole("button", { name: "Next turn", exact: true }).click();
  await expect(details.getByTestId("replay-details-opening")).toHaveText("Final move");
  await details.getByRole("button", { name: "Use move 2", exact: true }).click();

  // Browser Back is guarded independently from the sidebar Back button.
  const editorUrl = page.url();
  const discardPrompt = page.waitForEvent("dialog");
  await page.evaluate(() => history.back());
  const prompt = await discardPrompt;
  expect(prompt.message()).toBe("Discard unsaved replay details?");
  await prompt.dismiss();
  await expect(page).toHaveURL(editorUrl);
  await expect(details.getByLabel("Private title")).toHaveValue("Position to study");
  await expect(details.getByTestId("replay-details-opening")).toHaveText("Move 2 / 9");

  await details.getByRole("button", { name: "Save details", exact: true }).click();
  await expect(page.getByRole("button", { name: "Edit details", exact: true })).toBeFocused();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 2 / 9");
  await page.getByRole("link", { name: "Saved replays", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Position to study" })).toBeVisible();
  await expect(inspect).toBeFocused();
  const thumbnail = page.getByTestId("replay-thumbnail");
  await expect(thumbnail).toHaveAttribute("data-opening", "2");
  const canvas = thumbnail.locator("canvas");
  await expect(canvas).toHaveAttribute("data-ready", "true");
  const pixels = await canvas.evaluate((element) => {
    const board = element as HTMLCanvasElement;
    const step = board.width / 15;
    const offset = Math.floor(step / 2);
    const context = board.getContext("2d")!;
    return [110, 0, 111].map((cell) => Array.from(context.getImageData(
      offset + step * (cell % 15), offset + step * Math.floor(cell / 15), 1, 1,
    ).data));
  });
  expect(pixels).toEqual([[64, 64, 64, 255], [255, 255, 255, 255], [161, 125, 66, 255]]);
  await page.reload();
  await inspect.click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 2 / 9");
  await page.getByRole("button", { name: "Share replay", exact: true }).click();
  const shareUrl = await page.getByLabel("Replay link", { exact: true }).inputValue();
  const embedded = JSON.parse(Buffer.from(new URL(shareUrl).hash.slice("#g2r1=".length), "base64url").toString("utf8"));
  expect(embedded.at).toBe(2);
  expect(embedded.b.n).toBe("Alice");
  expect(Object.keys(embedded).sort()).toEqual(["at", "b", "m", "r", "v", "w"]);
  await page.getByRole("button", { name: "Back to replay", exact: true }).click();
  await page.getByRole("button", { name: "Edit details", exact: true }).click();
  await expect(details.getByLabel("Private note")).toHaveValue("Review White's reply.\n🟡 Then find a defense.");
  await details.getByRole("button", { name: "Use default", exact: true }).click();
  await details.getByRole("button", { name: "Save details", exact: true }).click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 2 / 9");
  await page.getByRole("link", { name: "Saved replays", exact: true }).click();
  await inspect.click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 9 / 9");

  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByRole("button", { name: "Edit details", exact: true }).click();
  await expectNoHorizontalOverflow(page);
  await auditAccessibility(page, testInfo, "private-replay-details");
  await captureChromiumScreenshot(page, testInfo.outputPath("private-replay-details-portrait.png"), testInfo);
});

test("a full legacy library draws visible rows and keeps the edit through migration", async ({ page }) => {
  const legacyKey = "gomoku2d.replay-library.v1";
  const currentKey = "gomoku2d.replay-library.v2";
  const entries = Array.from({ length: 64 }, (_, i) => {
    const wire = JSON.stringify({
      v: 1, r: "freestyle", m: Buffer.from(WIN_MOVES).toString("base64url"),
      b: { k: "h", n: `Black ${i + 1}` }, w: { k: "h", n: "White" },
    });
    return {
      fingerprint: createHash("sha256").update("gomoku2d-portable-replay-v1\0").update(wire).digest("base64url"),
      payload: Buffer.from(wire).toString("base64url"), kind: "saved", saved_at: "2026-11-18T00:00:00.000Z",
    };
  });
  await page.goto("/");
  await page.evaluate(({ legacyKey, entries }) => {
    localStorage.setItem(legacyKey, JSON.stringify({ schema_version: 1, owners: { guest: { entries, dirty: true } } }));
  }, { legacyKey, entries });
  await page.goto("/profile?tab=saved");
  const thumbnails = page.getByTestId("replay-thumbnail");
  await expect(thumbnails).toHaveCount(8);
  expect(await page.evaluate((key) => localStorage.getItem(key), currentKey)).toBeNull();
  const more = page.getByRole("button", { name: "Show more replays", exact: true });
  for (let count = 16; count <= 64; count += 8) {
    await more.click();
    await expect(thumbnails).toHaveCount(count);
  }
  await expect(more).toHaveCount(0);
  await expect(thumbnails.last().locator("canvas")).toHaveAttribute("data-ready", "true");
  await page.setViewportSize({ width: 1100, height: 800 });
  const lastReplay = page.getByRole("link", { name: "Inspect replay", exact: true }).last();
  const profileLayoutScrollTop = () => page.locator("main > section").filter({ has: lastReplay })
    .evaluate((element) => element.scrollTop);
  await lastReplay.scrollIntoViewIfNeeded();
  const originalScrollTop = await profileLayoutScrollTop();
  expect(originalScrollTop).toBeGreaterThan(100);
  await lastReplay.click();
  await page.getByRole("button", { name: "Edit details", exact: true }).click();
  await page.getByLabel("Private title", { exact: true }).fill("Last copy, new name");
  await page.getByRole("button", { name: "Save details", exact: true }).click();
  await expect(page.getByRole("button", { name: "Edit details", exact: true })).toBeFocused();
  await page.getByRole("link", { name: "Saved replays", exact: true }).click();
  await expect(lastReplay).toBeFocused();
  expect(Math.abs(await profileLayoutScrollTop() - originalScrollTop)).toBeLessThanOrEqual(1);
  await expect(thumbnails).toHaveCount(64);
  expect(await page.evaluate(({ legacyKey, currentKey }) => ({
    old: localStorage.getItem(legacyKey),
    entries: JSON.parse(localStorage.getItem(currentKey)!).owners.guest.entries.length,
  }), { legacyKey, currentKey })).toEqual({ old: null, entries: 64 });

  // An older tab can still write its retired key. Reloading the upgraded
  // library must use the v2 copy and its details.
  await page.evaluate((key) => localStorage.setItem(key, '{"schema_version":1,"owners":{}}'), legacyKey);
  await page.reload();
  await expect(thumbnails.first()).toBeVisible();
  while (await more.count()) await more.click();
  await expect(page.getByRole("heading", { name: "Last copy, new name" })).toBeVisible();
  await expect(thumbnails).toHaveCount(64);
});

test("concurrent tabs preserve the first saved edit and keep the stale draft visible", async ({ page, context }) => {
  await page.goto(sharedLink(8));
  await page.getByRole("button", { name: "Save replay", exact: true }).click();
  await expect(page.getByText("Saved on this device", { exact: true })).toBeVisible();
  await page.goto("/profile?tab=saved");
  await page.getByRole("link", { name: "Inspect replay", exact: true }).click();
  const other = await context.newPage();
  try {
    await other.goto(page.url());
    for (const tab of [page, other]) {
      await tab.getByRole("button", { name: "Edit details", exact: true }).click();
    }
    await page.getByLabel("Private note").fill("First tab saved this note");
    await other.getByLabel("Private note").fill("Second tab still has this draft");
    await page.getByRole("button", { name: "Save details", exact: true }).click();
    await expect(page.getByRole("button", { name: "Edit details", exact: true })).toBeFocused();

    await other.getByRole("button", { name: "Save details", exact: true }).click();
    await expect(other.getByRole("region", { name: "Replay details" }).getByRole("alert"))
      .toContainText(/changed in another tab/i);
    await expect(other.getByLabel("Private note")).toHaveValue("Second tab still has this draft");

    other.once("dialog", (dialog) => dialog.accept());
    await other.getByRole("button", { name: "Back to replay", exact: true }).click();
    await other.reload();
    await other.getByRole("button", { name: "Edit details", exact: true }).click();
    await expect(other.getByLabel("Private note")).toHaveValue("First tab saved this note");
  } finally {
    await other.close();
  }
});

test("playing from a shared frame records a regular game and keeps its source attribution", async ({ page }, testInfo) => {
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
  const trackUnexpectedDialog = async (dialog: import("@playwright/test").Dialog) => {
    dialogs.push(dialog.message());
    await dialog.dismiss();
  };
  page.on("dialog", trackUnexpectedDialog);
  await page.getByRole("button", { name: "Play From Here", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();
  expect(dialogs).toEqual([]);
  await expect(page.getByRole("dialog")).toHaveCount(0);
  page.off("dialog", trackUnexpectedDialog);
  await expect(page.getByText(/excluded from played-game totals/i)).toHaveCount(0);
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
  await expect(page).toHaveURL(/\/replay\/(?!portable-)[A-Za-z0-9-]+$/);
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 9 / 9");
  await expect(page.getByRole("region", { name: "Practice origin" })).toHaveCount(0);
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
  await expect(page.getByRole("heading", { name: "Practice games", exact: true })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Inspect", exact: true })).toHaveCount(1);
  await expectProfileStat(page, "Matches", 1);
  await expectProfileStat(page, "Wins", 1);
  await expectNoHorizontalOverflow(page);
  await page.reload();
  await expect(page.getByRole("button", { name: "Inspect", exact: true })).toHaveCount(1);
  await expectProfileStat(page, "Matches", 1);
  await captureChromiumScreenshot(page, "../docs/working/v0_6_evidence/portable_replay_practice_mobile.png", testInfo);

  await page.setViewportSize({ width: 1440, height: 1100 });
  await captureChromiumScreenshot(page, "../docs/working/v0_6_evidence/portable_replay_practice_desktop.png", testInfo);

  await page.getByRole("tab", { name: "Saved replays", exact: true }).click();
  const sourceEntry = page.getByRole("listitem").filter({ hasText: /Alice.*Bob.*9 moves/ });
  await expect(sourceEntry).toBeVisible();
  page.once("dialog", (dialog) => dialog.accept());
  await sourceEntry.getByRole("button", { name: "Remove replay", exact: true }).click();
  await expect(page.getByRole("link", { name: "Inspect replay", exact: true })).toHaveCount(0);

  await page.getByRole("tab", { name: "My games", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Practice games", exact: true })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Inspect", exact: true })).toHaveCount(1);
  await expectProfileStat(page, "Matches", 1);
  await expectProfileStat(page, "Wins", 1);
});
