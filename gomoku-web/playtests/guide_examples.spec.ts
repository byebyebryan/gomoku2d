import { expect, test } from "@playwright/test";
import axe from "axe-core";

test("guest discovers portable examples, shares and saves without importing played history", async ({ page }) => {
  const database: string[] = [];
  page.on("request", (request) => {
    if (/firestore\.googleapis\.com|google\.firestore/.test(request.url())) database.push(request.url());
  });
  await page.goto("/");
  await page.getByRole("link", { name: "Examples", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Example replays", exact: true })).toBeFocused();
  const examples = page.getByRole("list", { name: "Example replays" });
  await expect(examples.getByRole("listitem")).toHaveCount(2);
  const first = examples.getByRole("link", { name: "Open Review a finished game", exact: true });
  const path = await first.getAttribute("href");
  expect(path).toMatch(/^\/replay\/shared\/#g2r1=/);
  await first.click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 28 / 28");
  await expect(page.getByTestId("replay-player-row-black")).toContainText("Easy Bot");
  await expect(page.getByTestId("replay-player-row-white")).toContainText("Normal Bot");
  expect(await page.evaluate(() => localStorage.getItem("gomoku2d.replay-library.v1"))).toBeNull();
  await page.getByRole("button", { name: "Previous move", exact: true }).click();
  await page.getByRole("button", { name: "Share replay", exact: true }).click();
  const shared = await page.getByRole("textbox", { name: "Replay link", exact: true }).inputValue();
  const exported = JSON.parse(Buffer.from(new URL(shared).hash.slice("#g2r1=".length), "base64url").toString("utf8"));
  expect(exported.at).toBe(27);
  expect(exported.b.k).toBe("b");
  expect(exported.w.k).toBe("b");
  expect(exported.b.u).toBeUndefined();
  expect(exported.w.u).toBeUndefined();
  await page.getByRole("button", { name: "Back to replay", exact: true }).click();
  await page.getByRole("button", { name: "Save replay", exact: true }).click();
  await expect(page.getByRole("button", { name: "Saved replay", exact: true })).toBeDisabled();
  await page.goto(shared);
  await expect(page.getByRole("button", { name: "Saved replay", exact: true })).toBeDisabled();
  await page.goto("/profile");
  const matches = page.locator('[class*="summaryTile"]').filter({ hasText: "Matches" });
  await expect(matches.locator('[class*="summaryValue"]')).toHaveText("0");
  await page.getByRole("tab", { name: "Saved replays", exact: true }).click();
  await expect(page.getByRole("link", { name: "Inspect replay", exact: true })).toHaveCount(1);
  expect(database).toEqual([]);
});

test("Guide section links reflow, focus their destination and expose concise state help", async ({ page }, info) => {
  await page.goto("/guide/#examples");
  for (const [width, height] of [[320, 900], [390, 844], [667, 375], [844, 390], [980, 900], [1280, 900]]) {
    for (const scale of [100, 200]) {
      await page.setViewportSize({ width, height });
      await page.evaluate((scale) => { document.documentElement.style.fontSize = `${scale}%`; }, scale);
      await page.evaluate(() => document.fonts.ready);
      await page.getByRole("navigation", { name: "Guide sections" }).getByRole("link", { name: "Examples", exact: true }).click();
      await expect(page.getByRole("heading", { name: "Example replays", exact: true })).toBeFocused();
      const metrics = await page.evaluate(() => {
        const main = document.querySelector("main")!;
        const examples = document.getElementById("examples")!.closest("section")!;
        return { viewport: innerWidth, doc: document.documentElement.scrollWidth, main: main.clientWidth, scroll: main.scrollWidth,
          examples: examples.clientWidth, examplesScroll: examples.scrollWidth };
      });
      expect(metrics.doc).toBeLessThanOrEqual(metrics.viewport + 1);
      expect(metrics.scroll).toBeLessThanOrEqual(metrics.main + 1);
      expect(metrics.examplesScroll).toBeLessThanOrEqual(metrics.examples + 1);
    }
  }
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/guide/#replays");
  const heading = page.getByRole("heading", { name: "Replays & sharing", exact: true });
  await expect(heading).toBeFocused();
  await expect(heading).toBeInViewport();
  const privacy = page.locator("summary").filter({ hasText: "What stays private?" });
  await privacy.focus();
  await privacy.press("Enter");
  await expect(page.getByText(/The shared record is a dated My games snapshot/)).toBeVisible();
  await page.addScriptTag({ content: axe.source });
  const violations = await page.evaluate(async () => (await axe.run(document, { runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"] } })).violations);
  await info.attach("guide-accessibility", { body: JSON.stringify(violations), contentType: "application/json" });
  expect(violations).toEqual([]);
});

test("an example opens an eligible continuation and cancel preserves an active game", async ({ page }) => {
  await page.goto("/guide/#examples");
  await page.getByRole("link", { name: "Open Try another line", exact: true }).click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 26 / 36");
  await expect(page.getByRole("button", { name: "Play From Here", exact: true })).toBeEnabled();
  await page.getByRole("button", { name: "Play From Here", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Local Match", exact: true })).toBeVisible();
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 26");
  await page.getByRole("link", { name: "Home", exact: true }).click();
  await page.getByRole("link", { name: "Examples", exact: true }).click();
  await page.getByRole("link", { name: "Open Review a finished game", exact: true }).click();
  await page.getByRole("button", { name: "Previous move", exact: true }).click();
  const dialog = page.waitForEvent("dialog");
  const attempt = page.getByRole("button", { name: "Play From Here", exact: true }).click();
  await (await dialog).dismiss();
  await attempt;
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 27 / 28");
  await page.getByRole("link", { name: "Home", exact: true }).click();
  await page.getByRole("link", { name: "Play", exact: true }).click();
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 26");
});
