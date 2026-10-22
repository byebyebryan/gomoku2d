import { expect, test } from "@playwright/test";
import axe from "axe-core";

const WIN_MOVES = [110, 0, 111, 1, 112, 2, 113, 3, 114];

function drawMoves() {
  const black: number[] = [];
  const white: number[] = [];
  for (let row = 0; row < 15; row++) {
    for (let col = 0; col < 15; col++) {
      (((((row + 2 * col) & 3) >> 1) ^ (row & 1)) === 1 ? black : white).push(row * 15 + col);
    }
  }
  return black.flatMap((cell, index) => index < white.length ? [cell, white[index]!] : [cell]);
}

function link(black: string, white: string, moves = WIN_MOVES, profiles = false) {
  const wire = { v: 1, r: "freestyle", m: Buffer.from(moves).toString("base64url"),
    b: { k: "h", n: black, ...(profiles ? { u: "public_black" } : {}) },
    w: { k: "h", n: white, ...(profiles ? { u: "public_white" } : {}) }, at: moves.length };
  return `/replay/shared/#g2r1=${Buffer.from(JSON.stringify(wire)).toString("base64url")}`;
}

for (const [width, fontSize] of [[320, "100%"], [640, "200%"]] as const) {
  test(`maximum replay and labels reflow at ${width}px with ${fontSize} root text`, async ({ page }, testInfo) => {
    const requests: string[] = [];
    page.on("request", (request) => {
      if (/firestore\.googleapis\.com|google\.firestore/.test(request.url())) requests.push(request.url());
    });
    await page.setViewportSize({ width, height: 900 });
    await page.goto(link("W".repeat(80), "💠".repeat(80), drawMoves(), true));
    await expect(page.getByTestId("replay-move-count")).toHaveText("Move 225 / 225");
    await page.addStyleTag({ content: `html { font-size: ${fontSize}; }` });
    const nameHeadings = page.locator('[data-testid^="replay-player-row-"] h2');
    for (const name of await nameHeadings.all()) {
      const { height, lineHeight } = await name.evaluate((element) => ({
        height: element.getBoundingClientRect().height,
        lineHeight: Number.parseFloat(getComputedStyle(element).lineHeight),
      }));
      expect(height).toBeLessThanOrEqual(lineHeight * 2 + 1);
      expect(await name.getAttribute("title")).toBe(await name.textContent());
    }
    await expect(page.getByRole("button", { name: "Play From Here", exact: true })).toBeDisabled();
    await page.getByRole("button", { name: "Share replay", exact: true }).click();
    await expect(page.getByRole("textbox", { name: "Replay link", exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Play From Here", exact: true })).toHaveCount(0);
    expect(await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth)).toBeLessThanOrEqual(2);
    await page.addScriptTag({ content: axe.source });
    const report = await page.evaluate(async () => {
      const { violations, incomplete } = await axe.run(document, {
        runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "best-practice"] },
      });
      return { violations, incomplete };
    });
    await testInfo.attach("reflow-accessibility", { body: JSON.stringify(report), contentType: "application/json" });
    expect(report.violations).toEqual([]);
    await page.screenshot({ path: testInfo.outputPath(`maximum-replay-${width}.png`), fullPage: true });
    expect(requests).toEqual([]);
  });
}

test("hostile player labels remain text and sharing only exports profile links when selected", async ({ page }) => {
  const hostile = "<img src=x onerror=alert(1)>";
  const dialogs: string[] = [];
  page.on("dialog", async (dialog) => { dialogs.push(dialog.message()); await dialog.dismiss(); });
  await page.goto(link(hostile, "javascript:alert(1)", WIN_MOVES, true));
  await expect(page.getByTestId("replay-player-row-black")).toContainText(hostile);
  await expect(page.locator('img[src="x"]')).toHaveCount(0);
  await expect(page.locator('a[href^="javascript:"]')).toHaveCount(0);
  await page.getByRole("button", { name: "Share replay", exact: true }).click();
  const exported = async () => {
    const url = await page.getByRole("textbox", { name: "Replay link", exact: true }).inputValue();
    return JSON.parse(Buffer.from(new URL(url).hash.slice("#g2r1=".length), "base64url").toString("utf8"));
  };
  expect((await exported()).b).toEqual({ k: "h", n: hostile });
  await page.getByRole("checkbox", { name: "Include available player profile links" }).check();
  expect((await exported()).b.u).toBe("public_black");
  await page.getByRole("button", { name: "Generic names", exact: true }).click();
  expect((await exported()).b).toEqual({ k: "h", n: "Player 1" });
  expect((await exported()).w).toEqual({ k: "h", n: "Player 2" });
  const profileLinks = page.getByRole("checkbox", { name: "Include available player profile links" });
  await expect(profileLinks).toBeDisabled();
  await expect(profileLinks).not.toBeChecked();
  await page.getByRole("button", { name: "As played", exact: true }).click();
  await expect(profileLinks).toBeEnabled();
  await expect(profileLinks).toBeChecked();
  expect((await exported()).b.u).toBe("public_black");
  expect(dialogs).toEqual([]);
});
