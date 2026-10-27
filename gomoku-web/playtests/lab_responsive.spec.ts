import { expect, test } from "@playwright/test";

const inspectedMatch = "match_0095__search-d1__vs__search-d3_pattern-eval";

test("Analysis summaries and inspected games fit narrow and enlarged-text layouts", async ({ page }) => {
  test.setTimeout(120_000);
  const frame = page.locator(".dev-page");
  async function expectPageToFit(context: string) {
    await expect
      .poll(() => frame.evaluate((node) => node.scrollWidth - node.clientWidth), { message: context })
      .toBeLessThanOrEqual(1);
  }

  for (const { width, height, text } of [
    { width: 1440, height: 900, text: "100%" },
    { width: 844, height: 390, text: "100%" },
    { width: 768, height: 900, text: "100%" },
    { width: 412, height: 900, text: "100%" },
    { width: 320, height: 800, text: "100%" },
    { width: 320, height: 800, text: "200%" },
  ]) {
    await page.setViewportSize({ width, height });
    await page.goto("/lab/?tab=analysis");
    const analysis = page.locator('[data-view="analysis"]');
    const pair = analysis.locator(":scope > div > details").first();
    const summary = pair.locator(":scope > summary");
    await summary.waitFor();
    await page.evaluate(async (size) => {
      document.documentElement.style.fontSize = size;
      await document.fonts.ready;
    }, text);
    await expect(pair).not.toHaveAttribute("open");
    await expectPageToFit(`closed Analysis at ${width}px / ${text}`);
    for (const label of ["Games", "Failures", "Unclear", "Errors"]) {
      const metric = summary.locator(`[data-label="${label}"]`);
      await metric.scrollIntoViewIfNeeded();
      await expect(metric).toBeInViewport({ ratio: 0.99 });
    }

    await summary.click();
    const game = pair.locator(`[data-analysis-match-path="${inspectedMatch}"]`);
    await game.locator(":scope > summary").click();
    const board = game.locator('[data-proof-board="analysis"]').first();
    await expect(board).toBeVisible();
    await expectPageToFit(`opened Analysis at ${width}px / ${text}`);
    for (const label of ["Failure", "Lethal onset", "Setup corridor", "Game len"]) {
      const linesPerWord = await game
        .locator(`:scope > summary [data-label="${label}"] > span`)
        .evaluate((node) => {
          const content = node.firstChild!;
          return [...(content.textContent ?? "").matchAll(/\S+/g)].map((word) => {
            const range = document.createRange();
            range.setStart(content, word.index!);
            range.setEnd(content, word.index! + word[0].length);
            return range.getClientRects().length;
          });
        });
      expect(linesPerWord, `${label} stays readable at ${width}px / ${text}`).toEqual(
        linesPerWord.map(() => 1),
      );
    }
    if (width <= 412) {
      const proof = game.locator('article[aria-label^="Proof frame before move"]').first();
      const scroll = await proof.evaluate((node) => {
        node.scrollLeft = node.scrollWidth;
        return { position: node.scrollLeft, range: node.scrollWidth - node.clientWidth };
      });
      if (scroll.range > 1) expect(scroll.position).toBeGreaterThan(0);
      await expectPageToFit(`proof-board inspection at ${width}px / ${text}`);
    }

    await page.goto(`/lab/?tab=analysis&match=${inspectedMatch}`);
    await page.locator(`[data-analysis-match-path="${inspectedMatch}"][open]`).waitFor();
    await page.evaluate(async (size) => {
      document.documentElement.style.fontSize = size;
      await document.fonts.ready;
    }, text);
    await expectPageToFit(`direct Analysis inspection at ${width}px / ${text}`);
  }
});
