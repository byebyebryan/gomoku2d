import { expect, test } from "@playwright/test";
import axe from "axe-core";

const routes = [
  "/", "/match/local", "/profile", "/settings", "/rules/",
  "/guide/", "/lab/", "/visuals/", "/privacy/", "/terms/",
];

for (const viewport of [{ width: 1600, height: 1000 }, { width: 390, height: 844 }]) {
  for (const route of routes) {
    test(`accessibility audit ${route} at ${viewport.width}px`, async ({ page }, testInfo) => {
      await page.setViewportSize(viewport);
      await page.goto(route);
      await expect(page.locator("h1")).toBeVisible();
      if (route === "/match/local") {
        await expect(page.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
      }
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
          // Inconclusive checks stay visible; a passing scan is not human acceptance.
          incomplete: report.incomplete,
        };
      });
      await testInfo.attach("accessibility-audit", {
        body: JSON.stringify(result, null, 2), contentType: "application/json",
      });
      expect(result.violations).toEqual([]);
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    });
  }
}
