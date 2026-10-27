import { chromium, firefox, webkit } from "@playwright/test";
import axe from "axe-core";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import {
  decodeReviewImages,
  persistDeveloperAudit,
} from "./developer_visuals_helpers.mjs";

const base = process.env.DEVELOPER_CHECK_URL;
if (!base) throw new Error("Set DEVELOPER_CHECK_URL to an owned integrated preview");
const customOutput = process.env.DEVELOPER_CHECK_OUTPUT;
const output = resolve(customOutput ?? "test-results/v063-visuals");
await mkdir(output, { recursive: customOutput === undefined });

const engines = { chromium, firefox, webkit };
const routes = [
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
];
const widths = [1280, 390];
const expectedViews = Object.keys(engines).length * routes.length * widths.length;
/** @type {Array<Record<string, unknown>>} */
const results = [];
await persistDeveloperAudit({
  output,
  results,
  expectedViews,
  status: "incomplete",
});

/** @type {unknown} */
let runFailure;
try {
  for (const [name, engine] of Object.entries(engines)) {
    const browser = await engine.launch();
    try {
      const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
      /** @type {string[]} */
      const errors = [];
      page.on("pageerror", (error) => errors.push(error.message));
      for (const [label, path] of routes) {
        for (const width of widths) {
          await page.setViewportSize({ width, height: 900 });
          await page.goto(new URL(path, base).href);
          await page.locator("#developer-content h1").waitFor();
          await page.evaluate(() => document.fonts.ready);
          if (label === "icons") await page.locator('[data-inspect="publish"]').waitFor();
          if (label === "sprites")
            await page.locator('canvas[data-ready="true"]').first().waitFor();
          if (label === "search")
            await page.locator('[data-view="search"] details').last().waitFor();
          if (label === "style") await page.locator(".asset-swatch").last().waitFor();
          if (label === "lab")
            await page.locator('[data-view="ranking"] details').last().waitFor();
          if (label === "analysis")
            await page.locator('[data-proof-board="analysis"]').first().waitFor();
          if (label === "review") await decodeReviewImages(page);
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
          await persistDeveloperAudit({
            output,
            results,
            expectedViews,
            status: "incomplete",
          });
        }
      }
      if (errors.length) throw new Error(errors.join(" | "));
    } finally {
      await browser.close();
    }
  }
} catch (error) {
  runFailure = error;
}

const violations = results.reduce((count, result) => {
  const viewViolations = result.violations;
  return count + (Array.isArray(viewViolations) ? viewViolations.length : 0);
}, 0);
const status =
  runFailure !== undefined
    ? results.length < expectedViews
      ? "incomplete"
      : "failed"
    : results.length !== expectedViews
      ? "incomplete"
      : violations
        ? "failed"
        : "complete";
await persistDeveloperAudit({
  output,
  results,
  expectedViews,
  status,
  error: runFailure,
});
console.log(
  JSON.stringify({
    status,
    views: results.length,
    expectedViews,
    violations,
    incomplete: results.reduce((count, result) => {
      const viewIncomplete = result.incomplete;
      return count + (Array.isArray(viewIncomplete) ? viewIncomplete.length : 0);
    }, 0),
    output,
  }),
);
if (runFailure !== undefined) console.error(runFailure);
if (status !== "complete") process.exitCode = 1;
