import { chromium, firefox, webkit } from "@playwright/test";
import axe from "axe-core";
import { mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
const base = process.env.DEVELOPER_CHECK_URL;
if (!base) throw new Error("Set DEVELOPER_CHECK_URL to an owned integrated preview");
const output = resolve("test-results/v063-visuals");
await mkdir(output, { recursive: true });
const results = [];
for (const [name, engine] of Object.entries({ chromium, firefox, webkit })) {
  const browser = await engine.launch();
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  /** @type {string[]} */
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  try {
    for (const [label, path] of [
      ["style", "visuals/"],
      ["search", "lab/?tab=search"],
      ["review-index", "dev/reviews/"],
      ["reference", "dev/reviews/core-screens-062/"],
      ["overview", "dev/"],
      ["icons", "visuals/?view=icons&compare=share,save,publish"],
      ["sprites", "visuals/?view=sprites"],
      ["lab", "lab/"],
      ["analysis", "lab/?tab=analysis&match=match_0095__search-d1__vs__search-d3_pattern-eval"],
      ["review", "dev/reviews/mobile-records-062/"],
      ["process", "dev/process/"],
    ]) {
      for (const width of [1280, 390]) {
        await page.setViewportSize({ width, height: 900 });
        await page.goto(new URL(path, base).href);
        await page.locator("#developer-content h1").waitFor();
        await page.evaluate(() => document.fonts.ready);
        if (label === "icons") await page.locator('[data-inspect="publish"]').waitFor();
        if (label === "sprites") await page.locator('canvas[data-ready="true"]').first().waitFor();
        if (label === "search") await page.locator('[data-view="search"] details').last().waitFor();
      if(label === "style") await page.locator(".asset-swatch").last().waitFor();
      if (label === "lab") await page.locator('[data-view="ranking"] details').last().waitFor();
        if (label === "analysis")
          await page.locator('[data-proof-board="analysis"]').first().waitFor();
        if (label === "review")
          await page
            .locator(".review-capture img")
            .evaluateAll((images) =>
              Promise.all(images.map((image) => /** @type {HTMLImageElement} */ (image).decode())),
            );
        await page.screenshot({ path: resolve(output, `${name}-${label}-${width}.png`) });
        await page.addScriptTag({ content: axe.source });
        const report = await page.evaluate(async () => {
          const report = await axe.run(document, {
            runOnly: {
              type: "tag",
              values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "best-practice"],
            },
          });
          return { violations: report.violations, incomplete: report.incomplete };
        });
        results.push({ engine: name, label, width, ...report });
      }
    }
    if (errors.length) throw new Error(errors.join(" | "));
  } finally {
    await browser.close();
  }
}
await writeFile(resolve(output, "accessibility.json"), JSON.stringify(results, null, 2) + "\n");
const violations = results.reduce((n, result) => n + result.violations.length, 0);
console.log(
  JSON.stringify({
    views: results.length,
    violations,
    incomplete: results.reduce((n, result) => n + result.incomplete.length, 0),
    output,
  }),
);
if (violations) process.exitCode = 1;
