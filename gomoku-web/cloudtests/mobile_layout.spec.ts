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
  await expect(page.getByRole("button", { name: "Sign out", exact: true })).toBeVisible();
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
});
