import { mkdir, writeFile } from "node:fs/promises";
import type { Page, TestInfo } from "@playwright/test";

import { test, expect, fixtureId, signIn, makeProfileEligible, readProfile } from "./helpers.js";
import { seedLocalSavedMatch } from "../playtests/helpers/local_history.js";

test.use({ hasTouch: true });

async function capture(page: Page, info: TestInfo, name: string) {
  await page.evaluate(() => document.fonts.ready);
  await mkdir(info.outputDir, { recursive: true });
  await page.screenshot({ path: info.outputPath(`${name}.png`), fullPage: true, animations: "disabled" });
  const metrics = await page.evaluate(() => ({
    viewport: { width: innerWidth, height: innerHeight },
    width: document.documentElement.scrollWidth,
    height: document.documentElement.scrollHeight,
    tabsTop: document.querySelector('[role="tablist"]')?.getBoundingClientRect().y ?? null,
  }));
  expect(metrics.width).toBeLessThanOrEqual(metrics.viewport.width + 1);
  await writeFile(info.outputPath(`${name}.json`), JSON.stringify(metrics, null, 2));
}

test("M01: mobile record, account, saved and sharing surfaces remain reachable", async ({ page }, info) => {
  test.setTimeout(90_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  const matchId = fixtureId("mobile-finished");
  await seedLocalSavedMatch(page, {
    displayName: "Mobile Player", id: matchId,
    moves: [112, 0, 113, 15, 114, 30, 115, 1, 116].map((cell) => ({ row: Math.floor(cell / 15), col: cell % 15 })),
    preferredVariant: "freestyle", savedAt: "2026-10-23T12:00:00.000Z", status: "black_won", variant: "freestyle",
  });
  await page.goto("/profile");
  await expect(page.getByRole("tab", { name: "My games", exact: true })).toBeVisible();
  const controls = page.getByText("Profile controls", { exact: true });
  await expect(page.getByRole("button", { name: "Reset Profile", exact: true })).toHaveCount(0);
  const tabs = page.getByRole("tab");
  const boxes = await Promise.all((await tabs.all()).map((tab) => tab.boundingBox()));
  expect(Math.max(...boxes.map((box) => box!.y)) - Math.min(...boxes.map((box) => box!.y))).toBeLessThanOrEqual(1);
  expect(boxes[0]!.y).toBeLessThan(260);
  await controls.click();
  await expect(page.getByRole("button", { name: "Reset Profile", exact: true })).toBeVisible();
  await controls.click();
  await capture(page, info, "guest-games-390");
  await page.getByRole("tab", { name: "Saved replays", exact: true }).click();
  await capture(page, info, "guest-saved-390");
  await page.getByRole("tab", { name: "Public profile", exact: true }).click();
  await capture(page, info, "guest-public-390");

  const prefersRedirect = await page.evaluate(() => (
    navigator.maxTouchPoints > 0 && matchMedia("(pointer: coarse)").matches
  ));
  await info.attach("touch-auth-mode", { body: JSON.stringify({ prefersRedirect }), contentType: "application/json" });
  const uid = await signIn(page, `${fixtureId("mobile-owner")}@example.test`, { redirect: prefersRedirect });
  await expect.poll(async () => (await readProfile(uid))?.match_history?.replay_matches?.some((match: { id: string }) => match.id === matchId)).toBe(true);
  await makeProfileEligible(uid);
  await page.reload();
  const signOut = page.getByRole("button", { name: "Sign out", exact: true });
  const displayName = page.getByRole("textbox", { name: "Name", exact: true });
  const inspect = page.getByRole("button", { name: "Inspect", exact: true }).first();
  await expect(signOut).toBeVisible();
  await expect(displayName).toBeVisible();
  await expect(inspect).toBeVisible();
  for (const fontSize of ["100%", "200%"] as const) {
    await page.setViewportSize({ width: 844, height: 390 });
    await page.evaluate((size) => { document.documentElement.style.fontSize = size; }, fontSize);
    await page.evaluate(() => document.fonts.ready);
    await expect(signOut).toBeVisible();
    await expect(displayName).toBeVisible();
    await expect(inspect).toBeVisible();
    const tabLabels = ["My games", "Saved replays", "Public profile"] as const;
    for (const label of tabLabels) await expect(page.getByRole("tab", { name: label, exact: true })).toBeVisible();

    const metrics = await page.evaluate(() => {
      const main = document.querySelector("main") as HTMLElement | null;
      const layout = main?.querySelector('[class*="layout"]') as HTMLElement | null;
      const historyBody = main?.querySelector('[class*="historyBody"]') as HTMLElement | null;
      const historyItem = historyBody?.querySelector("ol > li") as HTMLElement | null;
      const tablist = main?.querySelector('[role="tablist"]') as HTMLElement | null;
      const name = main?.querySelector('input[aria-label="Name"]') as HTMLElement | null;
      const auth = main?.querySelector('button[aria-label="Sign out"]') as HTMLElement | null;
      const inspect = main?.querySelector('button[aria-label="Inspect"]') as HTMLElement | null;
      if (!main || !layout || !historyBody || !historyItem || !tablist || !name || !auth || !inspect) return null;
      const mainRect = main.getBoundingClientRect();
      const bounds = [name, auth, inspect, ...Array.from(tablist.querySelectorAll('[role="tab"]'))]
        .map((element) => {
          const rect = element.getBoundingClientRect();
          return { left: rect.left, right: rect.right };
        });
      return {
        bounds,
        historyBodyClientWidth: historyBody.clientWidth,
        historyBodyScrollWidth: historyBody.scrollWidth,
        historyItemClientWidth: historyItem.clientWidth,
        historyItemScrollWidth: historyItem.scrollWidth,
        layoutClientWidth: layout.clientWidth,
        layoutScrollWidth: layout.scrollWidth,
        mainClientWidth: main.clientWidth,
        mainLeft: mainRect.left,
        mainRight: mainRect.right,
        mainScrollWidth: main.scrollWidth,
        tabClientWidth: tablist.clientWidth,
        tabScrollWidth: tablist.scrollWidth,
      };
    });
    expect(metrics).not.toBeNull();
    expect(metrics!.mainScrollWidth - metrics!.mainClientWidth).toBeLessThanOrEqual(1);
    expect(metrics!.layoutScrollWidth - metrics!.layoutClientWidth).toBeLessThanOrEqual(1);
    expect(metrics!.historyBodyScrollWidth - metrics!.historyBodyClientWidth).toBeLessThanOrEqual(1);
    expect(metrics!.historyItemScrollWidth - metrics!.historyItemClientWidth).toBeLessThanOrEqual(1);
    expect(metrics!.tabScrollWidth - metrics!.tabClientWidth).toBeLessThanOrEqual(1);
    for (const bounds of metrics!.bounds) {
      expect(bounds.left).toBeGreaterThanOrEqual(metrics!.mainLeft - 1);
      expect(bounds.right).toBeLessThanOrEqual(metrics!.mainRight + 1);
    }
  }
  await page.setViewportSize({ width: 390, height: 844 });
  await page.evaluate(() => { document.documentElement.style.fontSize = "100%"; });
  await page.getByRole("tab", { name: "Public profile", exact: true }).click();
  await page.getByLabel("Choose your public name", { exact: true }).fill("Mobile Player");
  await page.getByRole("checkbox", { name: /I want to enable this public profile/ }).check();
  await page.getByRole("button", { name: "Enable public profile", exact: true }).click();
  await expect(page.getByRole("button", { name: "Save public name", exact: true })).toBeVisible();
  await page.getByRole("group", { name: "Share My games statistics publicly", exact: true }).getByRole("button", { name: "On", exact: true }).click();
  await expect(page.getByRole("button", { name: "Update public stats", exact: true })).toBeVisible();
  await page.evaluate(() => window.scrollTo(0, 0));
  await capture(page, info, "owner-public-390");

  await page.goto(`/replay/${matchId}`);
  await expect(page.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
  await page.getByRole("button", { name: "Previous move", exact: true }).click();
  await capture(page, info, "replay-390");
  await page.getByRole("button", { name: "Share replay", exact: true }).click();
  const back = page.getByRole("button", { name: "Back to replay", exact: true });
  await expect(back).toBeInViewport();
  await capture(page, info, "share-390");
  const sharedUrl = await page.getByRole("textbox", { name: "Replay link", exact: true }).inputValue();
  await back.click();
  for (const width of [667, 844]) {
    await page.setViewportSize({ width, height: 390 });
    await expect(page.getByRole("grid", { name: "Gomoku board" })).toBeInViewport();
    await expect(page.getByRole("button", { name: "Next move", exact: true })).toBeInViewport();
    await capture(page, info, `replay-landscape-${width}`);
  }
  await page.getByRole("button", { name: "Share replay", exact: true }).click();
  await expect(back).toBeInViewport();
  await capture(page, info, "share-landscape");
  await page.getByRole("textbox", { name: "Replay link", exact: true }).focus();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("button", { name: "Share replay", exact: true })).toBeFocused();
  await expect(page.getByRole("button", { name: "Share replay", exact: true })).toBeInViewport();
  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByRole("button", { name: "Publish replay", exact: true }).click();
  await page.getByRole("textbox", { name: "Public replay title", exact: true }).fill("A mobile example");
  await capture(page, info, "publish-390");
  await page.getByRole("button", { name: "Publish this replay", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: "Published “A mobile example”" })).toBeVisible();

  await page.goto(sharedUrl);
  await page.getByRole("button", { name: "Save replay", exact: true }).click();
  await expect(page.getByRole("button", { name: "Saved replay", exact: true })).toBeVisible();
  await page.goto("/profile");
  await page.getByRole("tab", { name: "Saved replays", exact: true }).click();
  await expect(page.getByRole("link", { name: "Inspect replay", exact: true })).toBeVisible();
  await page.evaluate(() => window.scrollTo(0, 0));
  await capture(page, info, "owner-saved-390");
  for (const [width, height] of [[320, 900], [844, 390]] as const) {
    await page.setViewportSize({ width, height });
    await capture(page, info, `owner-saved-${width}`);
  }
  await page.setViewportSize({ width: 320, height: 900 });
  await page.getByRole("tab", { name: "Public profile", exact: true }).click();
  await capture(page, info, "owner-public-320");
  await page.goto(`/player/?id=${uid}`);
  await expect(page.getByText("A mobile example", { exact: true })).toBeVisible();
  await capture(page, info, "public-player-320");
  for (const [width, height, fontSize] of [
    [320, 900, "100%"], [320, 900, "200%"],
    [390, 844, "100%"], [390, 844, "200%"],
    [560, 320, "100%"], [560, 320, "200%"],
    [667, 375, "100%"], [667, 375, "200%"],
    [720, 390, "100%"], [720, 390, "200%"],
    [721, 390, "100%"], [721, 390, "200%"],
    [844, 390, "100%"], [844, 390, "200%"],
    [980, 500, "100%"], [980, 500, "200%"],
  ] as const) {
    await page.setViewportSize({ width, height });
    await page.evaluate((size) => { document.documentElement.style.fontSize = size; }, fontSize);
    await page.evaluate(() => document.fonts.ready);
    const navigation = page.getByRole("navigation", { name: "Main navigation", exact: true });
    const actions = ["Play", "My profile", "Home"] as const;
    for (const label of actions) await expect(navigation.getByRole("link", { name: label, exact: true })).toBeVisible();

    const metrics = await page.evaluate(() => {
      const main = document.querySelector("main") as HTMLElement | null;
      const shell = main?.querySelector('[class*="shell"]') as HTMLElement | null;
      const header = main?.querySelector("header") as HTMLElement | null;
      const navigation = main?.querySelector('nav[aria-label="Main navigation"]') as HTMLElement | null;
      if (!main || !shell || !header || !navigation) return null;
      const mainRect = main.getBoundingClientRect();
      const bounds = Array.from(navigation.querySelectorAll("a")).map((element) => {
        const rect = element.getBoundingClientRect();
        return { bottom: rect.bottom, left: rect.left, right: rect.right };
      });
      return {
        bounds,
        documentClientWidth: document.documentElement.clientWidth,
        documentScrollWidth: document.documentElement.scrollWidth,
        headerClientWidth: header.clientWidth,
        headerScrollWidth: header.scrollWidth,
        mainClientWidth: main.clientWidth,
        mainLeft: mainRect.left,
        mainRight: mainRect.right,
        mainScrollWidth: main.scrollWidth,
        navigationClientWidth: navigation.clientWidth,
        navigationScrollWidth: navigation.scrollWidth,
        shellClientWidth: shell.clientWidth,
        shellScrollWidth: shell.scrollWidth,
        viewportHeight: window.innerHeight,
      };
    });
    expect(metrics).not.toBeNull();
    expect(metrics!.documentScrollWidth - metrics!.documentClientWidth).toBeLessThanOrEqual(1);
    expect(metrics!.mainScrollWidth - metrics!.mainClientWidth).toBeLessThanOrEqual(1);
    expect(metrics!.shellScrollWidth - metrics!.shellClientWidth).toBeLessThanOrEqual(1);
    expect(metrics!.headerScrollWidth - metrics!.headerClientWidth).toBeLessThanOrEqual(1);
    expect(metrics!.navigationScrollWidth - metrics!.navigationClientWidth).toBeLessThanOrEqual(1);
    for (const bounds of metrics!.bounds) {
      expect(bounds.left).toBeGreaterThanOrEqual(metrics!.mainLeft - 1);
      expect(bounds.right).toBeLessThanOrEqual(metrics!.mainRight + 1);
      expect(bounds.bottom).toBeLessThanOrEqual(metrics!.viewportHeight + 1);
    }
  }
});
