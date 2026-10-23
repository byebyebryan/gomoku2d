// Run in the repository's qualified Playwright container. This reviews static
// icon geometry only; it never signs in or changes app/cloud state.
import { createRequire } from "node:module";
import { readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(new URL("../../../gomoku-web/package.json", import.meta.url));
const { chromium, expect } = require("@playwright/test");
const here = dirname(fileURLToPath(import.meta.url));
const geometry = JSON.parse(await readFile(join(here, "geometry.json"), "utf8"));
const expectedMasks = {};
for (const name of Object.keys(geometry.icons)) {
  let source;
  try { source = await readFile(join(here, `${name}.svg`), "utf8"); }
  catch (error) {
    if (error.code !== "ENOENT") throw error;
    source = await readFile(join(here, "../../../gomoku-web/assets/icons", `${name}.svg`), "utf8");
  }
  const mask = Array(576).fill(0);
  for (const rect of source.matchAll(/<rect\s+([^>]+)>/g)) {
    const attributes = Object.fromEntries([...rect[1].matchAll(/(\w+)="([^"]+)"/g)].map((match) => [match[1], Number(match[2])]));
    const { x, y, width, height } = attributes;
    for (let row = y; row < y + height; row++) for (let col = x; col < x + width; col++) mask[row * 24 + col] = 1;
  }
  expectedMasks[name] = mask;
}
const revision = createHash("sha256").update(await readFile(join(here, "index.html"))).digest("hex").slice(0, 12);
const browser = await chromium.launch();
try {
  const page = await browser.newPage({ viewport: { width: 1080, height: 1400 }, deviceScaleFactor: 1 });
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("response", (response) => { if (!response.ok()) errors.push(`${response.status()} ${response.url()}`); });
  await page.goto(`http://127.0.0.1:8013/icon-review/actions/?revision=${revision}`);
  await page.evaluate(() => document.fonts.ready);
  await expect(page.locator(".card")).toHaveCount(15);
  const raster = await page.evaluate(async (expectedMasks) => {
    const results = [];
    for (const card of document.querySelectorAll(".card")) {
      const source = card.querySelector(".native svg");
      const svg = source.cloneNode(true);
      svg.setAttribute("style", "color:#f5f5f5");
      const url = URL.createObjectURL(new Blob([new XMLSerializer().serializeToString(svg)], { type: "image/svg+xml" }));
      const img = new Image(); img.src = url; await img.decode();
      const mask = Array(576).fill(0);
      for (const rect of source.querySelectorAll("rect")) {
        const [x, y, w, h] = ["x", "y", "width", "height"].map((key) => Number(rect.getAttribute(key)));
        for (let row = y; row < y + h; row++) for (let col = x; col < x + w; col++) mask[row * 24 + col] = 1;
      }
      if (mask.some((pixel, index) => pixel !== expectedMasks[card.id][index])) {
        throw new Error(`${card.id}: preview differs from the current source SVG`);
      }
      for (const scale of [1, 2, 10]) {
        const canvas = document.createElement("canvas"); canvas.width = canvas.height = 24 * scale;
        const ctx = canvas.getContext("2d"); ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
        const rgba = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
        for (let y = 0; y < canvas.height; y++) for (let x = 0; x < canvas.width; x++) {
          const i = (y * canvas.width + x) * 4;
          const expected = mask[Math.floor(y / scale) * 24 + Math.floor(x / scale)];
          if (rgba[i + 3] !== expected * 255 || (expected && rgba[i] !== 245)) {
            throw new Error(`${card.id}: raster mismatch at ${x},${y} @${scale}x`);
          }
        }
      }
      results.push({ icon: card.querySelector("h2").textContent, scales: [1, 2, 10], exact_pixels: true, current_source: true });
      URL.revokeObjectURL(url);
    }
    return results;
  }, expectedMasks);
  await page.locator("main").screenshot({ path: join(here, "comparison.png") });
  const draftBounds = await page.evaluate(() => {
    const first = document.querySelector(".card").getBoundingClientRect();
    const sixth = document.querySelectorAll(".card")[5].getBoundingClientRect();
    const section = document.querySelector(".comparison").getBoundingClientRect();
    return { x: section.x, y: first.y, width: section.width, height: sixth.bottom - first.y };
  });
  await page.screenshot({ path: join(here, "new-icons.png"), clip: draftBounds });
  await page.getByRole("button", { name: "Grid on", exact: true }).click();
  await expect(page.locator("body")).toHaveClass("no-grid");
  await page.getByRole("button", { name: "Grid off", exact: true }).click();
  await page.getByRole("button", { name: "Light surface", exact: true }).click();
  await expect(page.locator("body")).toHaveClass("light");
  await page.screenshot({ path: join(here, "new-icons-light.png"), clip: draftBounds });
  await page.setViewportSize({ width: 320, height: 900 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(320);
  expect(errors).toEqual([]);
  await writeFile(join(here, "render.json"), JSON.stringify({
    browser: browser.version(), raster, preview_errors: errors, mobile_width: 320, mobile_overflow: false,
  }, null, 2) + "\n");
  console.log("15 SVGs match source pixels at 1x/2x/10x; grid, theme and 320px reflow checks passed.");
} finally {
  await browser.close();
}
