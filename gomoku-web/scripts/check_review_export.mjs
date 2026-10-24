import { chromium, firefox, webkit } from "@playwright/test";
import { writeFile } from "node:fs/promises";
const base = process.env.REVIEW_CHECK_URL;
if (!base) throw new Error("Set REVIEW_CHECK_URL to a prepared/published export (trailing slash)");
/** @type {import('../src/developer/review_record.js').ReviewRecord} */
const record = await (await fetch(new URL("record.json", base))).json();
const pair = record.comparisons.at(-1);
if (!pair) throw new Error("No exported comparisons");
const results = [];
for (const [engine, type] of Object.entries({ chromium, firefox, webkit })) {
  const browser = await type.launch();
  const page = await browser.newPage({ viewport: { width: 390, height: 844 } });
  /** @type {string[]} */ const errors = [];
  /** @type {string[]} */ const outside = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("request", (request) => {
    if (!request.url().startsWith(new URL(base).origin)) outside.push(request.url());
  });
  try {
    await page.goto(base);
    await page.getByRole("heading", { name: record.title, exact: true }).waitFor();
    const select = page.getByRole("combobox", { name: "Screen and conditions" });
    if (await select.count()) await select.selectOption(pair.id);
    else await page.getByRole("button", { name: pair.title, exact: true }).click();
    await page.locator(`.review-capture img[src$="${pair.media.at(-1)?.file}"]`).waitFor();
    for (const image of await page.locator(".review-capture img").all()) {
      await image.scrollIntoViewIfNeeded();
      await image.evaluate((image) => /** @type {HTMLImageElement} */ (image).decode());
    }
    const widths = await page
      .locator(".review-capture img")
      .evaluateAll((images) =>
        images.map((image) => /** @type {HTMLImageElement} */ (image).naturalWidth),
      );
    if (widths.join(",") !== pair.media.map((media) => media.width).join(","))
      throw new Error("Wrong selected media");
    await page.getByRole("button", { name: "Show full captures", exact: true }).click();
    await page.getByRole("button", { name: "Native pixels", exact: true }).click();
    await page.reload();
    await page.locator(".review-native").waitFor();
    await page.goBack();
    await page.locator(".review-native").waitFor({ state: "hidden" });
    await page.goForward();
    await page.locator(".review-native").waitFor();
    await page.getByText("Technical evidence, sources and media identity", { exact: true }).click();
    await page.setViewportSize({ width: 320, height: 800 });
    await page.evaluate(() => (document.documentElement.style.fontSize = "200%"));
    await page.evaluate(() => document.fonts.ready);
    if (
      await page
        .locator(".dev-page")
        .evaluate((element) => element.scrollWidth > element.clientWidth + 1)
    )
      throw new Error("Export overflows narrow enlarged text");
    if (errors.length || outside.length) throw new Error(JSON.stringify({ errors, outside }));
    results.push({
      engine,
      passed: true,
      selected_images: widths.length,
      history: true,
      reload: true,
      native_and_full: true,
      narrow_200_percent: true,
      errors,
      outside,
    });
  } catch (error) {
    console.log(
      JSON.stringify({
        engine,
        error: String(error),
        overflow: await page.evaluate(() =>
          [...document.querySelectorAll(".dev-page *")]
            .map((element) => ({
              tag: element.tagName,
              cls: String(element.className),
              text: element.textContent?.slice(0, 80),
              width: element.clientWidth,
              scroll: element.scrollWidth,
              right: element.getBoundingClientRect().right,
            }))
            .filter((element) => element.scroll > element.width + 1)
            .slice(-12),
        ),
      }),
    );
    await page.screenshot({ path: `test-results/v063-export-${engine}-failure.png` });
    throw error;
  } finally {
    await browser.close();
  }
}
await writeFile(
  "test-results/v063-export-browser.json",
  JSON.stringify({ base, record: record.id, results }, null, 2) + "\n",
);
console.log(JSON.stringify(results));
