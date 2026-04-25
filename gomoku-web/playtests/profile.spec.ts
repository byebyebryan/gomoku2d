import { expect, test } from "@playwright/test";

test("guest profile persists locally and renders saved local matches", async ({ page }) => {
  await page.setViewportSize({ width: 1024, height: 900 });
  await page.goto("/profile");

  await expect(page.getByRole("heading", { name: "Profile" })).toBeVisible();
  await expect(page.getByText("Reduced motion")).toHaveCount(0);
  await expect(page.getByText("Sound")).toHaveCount(0);

  const displayName = page.getByLabel("Display name");
  await displayName.fill("Bryan Guest");
  await page.reload();
  await expect(displayName).toHaveValue("Bryan Guest");
  await expect(page.getByText("Default rule")).toBeVisible();
  await page.getByRole("button", { name: "Renju" }).click();

  await page.getByRole("link", { name: "Play" }).click();
  await expect(page.getByRole("heading", { name: "Local Match" })).toBeVisible();
  await expect(page.getByText("Bryan Guest to move")).toBeVisible();
  await expect(page.getByTestId("match-rule")).toHaveText("Renju");

  await page.evaluate(() => {
    const storageKey = "gomoku2d.guest-profile.v1";
    const stored = localStorage.getItem(storageKey);
    const parsed = stored
      ? JSON.parse(stored)
      : {
          state: {
            history: [],
            profile: null,
            settings: { preferredVariant: "freestyle" },
          },
          version: 0,
        };

    parsed.state.history = [
      {
        guestStone: "black",
        id: "fixture-finished-match",
        mode: "bot",
        moves: [
          { col: 7, moveNumber: 1, player: 1, row: 7 },
          { col: 6, moveNumber: 2, player: 2, row: 5 },
          { col: 8, moveNumber: 3, player: 1, row: 7 },
          { col: 6, moveNumber: 4, player: 2, row: 6 },
          { col: 9, moveNumber: 5, player: 1, row: 7 },
          { col: 6, moveNumber: 6, player: 2, row: 7 },
        ],
        players: [
          { kind: "human", name: "Bryan Guest", stone: "black" },
          { kind: "bot", name: "Practice Bot", stone: "white" },
        ],
        savedAt: "2026-04-22T18:30:00.000Z",
        status: "white_won",
        variant: "renju",
        winningCells: [
          { row: 5, col: 6 },
          { row: 6, col: 6 },
          { row: 7, col: 6 },
          { row: 8, col: 6 },
          { row: 9, col: 6 },
        ],
      },
    ];
    localStorage.setItem(storageKey, JSON.stringify(parsed));
  });

  await page.goto("/profile");

  await expect(page.getByText("Loss", { exact: true })).toBeVisible();
  await expect(page.getByText("Wins", { exact: true })).toBeVisible();
  await expect(page.getByText("Losses", { exact: true })).toBeVisible();
  await expect(page.getByText("Draws", { exact: true })).toBeVisible();
  await expect(page.getByText("vs Practice Bot")).toBeVisible();
  await expect(page.locator("ol li").first()).toContainText("Renju");
  const overflowMetrics = await page.evaluate(() => {
    const historyBody = document.querySelector('[class*="historyBody"]');

    return {
      historyBodyClientWidth: (historyBody as HTMLElement | null)?.clientWidth ?? 0,
      historyBodyScrollWidth: (historyBody as HTMLElement | null)?.scrollWidth ?? 0,
      pageClientWidth: document.documentElement.clientWidth,
      pageScrollWidth: document.documentElement.scrollWidth,
    };
  });

  expect(overflowMetrics.pageScrollWidth - overflowMetrics.pageClientWidth).toBeLessThanOrEqual(2);
  expect(overflowMetrics.historyBodyScrollWidth - overflowMetrics.historyBodyClientWidth).toBeLessThanOrEqual(2);

  await displayName.fill("Bryan Prime");
  await expect(displayName).toHaveValue("Bryan Prime");

  await page.getByRole("button", { name: "Reset local profile" }).click();
  await expect(displayName).toHaveValue("Guest");
  await expect(page.getByText("Match History")).toBeVisible();
});

test("profile history keeps summary pinned while the history list scrolls", async ({ page }) => {
  await page.goto("/profile");

  await expect(page.getByRole("heading", { name: "Profile" })).toBeVisible();
  await expect(page.getByText("Finished", { exact: true })).toBeVisible();
  await expect(page.getByText("Wins", { exact: true })).toBeVisible();
  await expect(page.getByText("Losses", { exact: true })).toBeVisible();
  await expect(page.getByText("Draws", { exact: true })).toBeVisible();

  await page.evaluate(() => {
    const historyBody = document.querySelector('[class*="historyBody"]');
    if (!historyBody) {
      throw new Error("history body not found");
    }

    const filler = document.createElement("div");
    filler.setAttribute("data-testid", "profile-history-growth-fixture");
    filler.style.display = "grid";
    filler.style.gap = "12px";
    filler.style.marginTop = "16px";

    for (let index = 0; index < 18; index += 1) {
      const row = document.createElement("div");
      row.textContent = `Fixture match ${index + 1}`;
      row.style.padding = "14px";
      row.style.border = "1px solid rgba(255, 255, 255, 0.12)";
      row.style.background = "rgba(255, 255, 255, 0.04)";
      filler.appendChild(row);
    }

    historyBody.appendChild(filler);
  });

  await page.waitForTimeout(100);

  const metrics = await page.evaluate(() => {
    const historyBody = document.querySelector('[class*="historyBody"]');

    if (!historyBody) {
      return null;
    }

    return {
      historyBodyClientHeight: (historyBody as HTMLElement).clientHeight,
      historyBodyScrollHeight: (historyBody as HTMLElement).scrollHeight,
      pageClientHeight: document.documentElement.clientHeight,
      pageHeight: document.documentElement.scrollHeight,
    };
  });

  expect(metrics).not.toBeNull();
  expect(metrics!.pageHeight - metrics!.pageClientHeight).toBeLessThanOrEqual(2);
  expect(metrics!.historyBodyScrollHeight).toBeGreaterThan(metrics!.historyBodyClientHeight);
});

test("profile uses quieter labels and compact shared action defaults", async ({ page }) => {
  await page.goto("/profile");

  await expect(page.getByRole("heading", { name: "Profile" })).toBeVisible();

  const metrics = await page.evaluate(() => {
    const sectionLabel = document.querySelector(".uiSectionLabel");
    const fieldLabel = document.querySelector('[class*="fieldLabel"]');
    const action = Array.from(document.querySelectorAll(".uiAction")).find((element) =>
      element.textContent?.includes("Play"),
    ) as HTMLElement | undefined;
    const icon = action?.querySelector(".uiIcon") as HTMLElement | null;

    if (!sectionLabel || !fieldLabel || !action || !icon) {
      return null;
    }

    const sectionLabelStyle = window.getComputedStyle(sectionLabel);
    const fieldLabelStyle = window.getComputedStyle(fieldLabel);
    const actionStyle = window.getComputedStyle(action);
    const iconStyle = window.getComputedStyle(icon);

    return {
      actionGap: actionStyle.gap,
      actionPaddingLeft: actionStyle.paddingLeft,
      actionPaddingRight: actionStyle.paddingRight,
      fieldLabelOpacity: Number(fieldLabelStyle.opacity),
      iconWidth: Number.parseFloat(iconStyle.width),
      sectionLabelOpacity: Number(sectionLabelStyle.opacity),
    };
  });

  expect(metrics).not.toBeNull();
  expect(metrics!.sectionLabelOpacity).toBeLessThan(1);
  expect(metrics!.fieldLabelOpacity).toBeLessThan(1);
  expect(metrics!.actionGap).toBe("8px");
  expect(metrics!.actionPaddingLeft).toBe("16px");
  expect(metrics!.actionPaddingRight).toBe("16px");
  expect(metrics!.iconWidth).toBe(24);
});

test("desktop profile prioritizes the record summary over the identity rail", async ({ page }) => {
  await page.goto("/profile");

  await expect(page.getByRole("heading", { name: "Profile" })).toBeVisible();

  await page.evaluate(() => {
    localStorage.setItem(
      "gomoku2d.guest-profile.v1",
      JSON.stringify({
        state: {
          history: [
            {
              guestStone: "black",
              id: "fixture-match",
              mode: "bot",
              moves: [
                { col: 7, row: 7, stone: "black" },
                { col: 7, row: 8, stone: "white" },
                { col: 8, row: 7, stone: "black" },
                { col: 8, row: 8, stone: "white" },
              ],
              players: [
                { kind: "human", name: "Guest", stone: "black" },
                { kind: "bot", name: "Practice Bot", stone: "white" },
              ],
              savedAt: "2026-04-22T18:30:00.000Z",
              status: "white_won",
              variant: "renju",
              winningCells: [],
            },
          ],
          profile: {
            avatarUrl: null,
            createdAt: "2026-04-22T18:00:00.000Z",
            displayName: "Guest",
            id: "fixture-profile",
            kind: "guest",
            updatedAt: "2026-04-22T18:30:00.000Z",
            username: null,
          },
          settings: {
            preferredVariant: "freestyle",
          },
        },
        version: 0,
      }),
    );
  });

  await page.reload();
  await expect(page.getByRole("heading", { name: "Profile" })).toBeVisible();

  const metrics = await page.evaluate(() => {
    const sideSection = document.querySelector('[class*="sideSection"]');
    const badge = document.querySelector('[class*="badge"]');
    const summaryValue = document.querySelector('[class*="summaryValue"]');
    const summaryGrid = document.querySelector('[class*="summaryGrid"]');
    const summaryTile = document.querySelector('[class*="summaryTile"]');
    const historyHeadLabel = document.querySelector('[class*="historyHeadLabel"]');
    const historyItem = document.querySelector('[class*="historyItem"]');

    if (
      !sideSection ||
      !badge ||
      !summaryValue ||
      !summaryGrid ||
      !summaryTile ||
      !historyHeadLabel ||
      !historyItem
    ) {
      return null;
    }

    const sideSectionStyle = window.getComputedStyle(sideSection);
    const badgeStyle = window.getComputedStyle(badge);
    const summaryValueStyle = window.getComputedStyle(summaryValue);
    const summaryGridStyle = window.getComputedStyle(summaryGrid);
    const summaryTileStyle = window.getComputedStyle(summaryTile);
    const historyHeadLabelStyle = window.getComputedStyle(historyHeadLabel);
    const historyItemStyle = window.getComputedStyle(historyItem);

    return {
      badgePaddingLeft: badgeStyle.paddingLeft,
      badgePaddingTop: badgeStyle.paddingTop,
      historyHeadLabelColor: historyHeadLabelStyle.color,
      historyHeadLabelOpacity: Number(historyHeadLabelStyle.opacity),
      historyItemColumnGap: historyItemStyle.columnGap,
      historyItemRowGap: historyItemStyle.rowGap,
      sideSectionGap: sideSectionStyle.gap,
      sideSectionPaddingTop: sideSectionStyle.paddingTop,
      summaryGridMarginTop: summaryGridStyle.marginTop,
      summaryTileGap: summaryTileStyle.gap,
      summaryValueFontSize: Number.parseFloat(summaryValueStyle.fontSize),
    };
  });

  expect(metrics).not.toBeNull();
  expect(metrics!.sideSectionGap).toBe("12px");
  expect(metrics!.sideSectionPaddingTop).toBe("16px");
  expect(metrics!.badgePaddingTop).toBe("4px");
  expect(metrics!.badgePaddingLeft).toBe("8px");
  expect(metrics!.summaryGridMarginTop).toBe("12px");
  expect(metrics!.summaryTileGap).toBe("4px");
  expect(metrics!.summaryValueFontSize).toBeGreaterThanOrEqual(30);
  expect(metrics!.historyHeadLabelColor).toBe("rgb(143, 141, 135)");
  expect(metrics!.historyHeadLabelOpacity).toBeLessThanOrEqual(0.82);
  expect(metrics!.historyItemColumnGap).toBe("18px");
  expect(metrics!.historyItemRowGap).toBe("10px");
});

test("portrait profile scrolls the page instead of the history pane", async ({ page }) => {
  await page.setViewportSize({ width: 430, height: 932 });
  await page.goto("/profile");

  await expect(page.getByRole("heading", { name: "Profile" })).toBeVisible();
  await expect(page.getByText("Match History")).toBeVisible();

  await page.evaluate(() => {
    const historyBody = document.querySelector('[class*="historyBody"]');
    if (!historyBody) {
      throw new Error("history body not found");
    }

    const filler = document.createElement("div");
    filler.setAttribute("data-testid", "profile-portrait-scroll-fixture");
    filler.style.display = "grid";
    filler.style.gap = "12px";
    filler.style.marginTop = "16px";

    for (let index = 0; index < 18; index += 1) {
      const row = document.createElement("div");
      row.textContent = `Fixture match ${index + 1}`;
      row.style.padding = "14px";
      row.style.border = "1px solid rgba(255, 255, 255, 0.12)";
      row.style.background = "rgba(255, 255, 255, 0.04)";
      filler.appendChild(row);
    }

    historyBody.appendChild(filler);
  });

  const before = await page.evaluate(() => {
    const header = document.querySelector("header");
    const historyBody = document.querySelector('[class*="historyBody"]');
    const layout = document.querySelector('[class*="layout"]');
    const recordHeader = document.querySelector('[class*="recordHeader"]');

    if (!header || !historyBody || !layout || !recordHeader) {
      return null;
    }

    return {
      bodyOverflowY: window.getComputedStyle(document.body).overflowY,
      headerTop: header.getBoundingClientRect().top,
      recordHeaderBottom: recordHeader.getBoundingClientRect().bottom,
      historyOverflowY: window.getComputedStyle(historyBody).overflowY,
      layoutOverflowY: window.getComputedStyle(layout).overflowY,
      viewportHeight: window.innerHeight,
    };
  });

  expect(before).not.toBeNull();
  expect(before!.bodyOverflowY).toBe("auto");
  expect(before!.recordHeaderBottom).toBeLessThanOrEqual(before!.viewportHeight);
  expect(before!.historyOverflowY).toBe("visible");
  expect(before!.layoutOverflowY).toBe("visible");

  await page.evaluate(() => window.scrollTo(0, 200));
  await page.waitForTimeout(50);
  await expect
    .poll(async () =>
      page.evaluate(() => document.querySelector("header")?.getBoundingClientRect().top ?? 0),
    )
    .toBeLessThan(before!.headerTop - 20);
});
