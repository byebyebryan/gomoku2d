import { expect, test } from "@playwright/test";

test("developer direct entries keep private state and engine imports untouched", async ({
  page,
}) => {
  const requests: string[] = [];
  await page.route("**/*", (route) => {
    const host = new URL(route.request().url()).hostname;
    return host === "127.0.0.1" || host === "localhost" ? route.continue() : route.abort();
  });
  page.on("request", (request) => requests.push(request.url()));
  await page.addInitScript(() => {
    localStorage.setItem("gomoku2d.local-profile.v5", "private sentinel, do not read");
    const reads: string[] = [];
    Object.assign(window, { developerStorageReads: reads });
    const get = Storage.prototype.getItem;
    Storage.prototype.getItem = function (key) {
      reads.push(key);
      return get.call(this, key);
    };
  });
  for (const path of [
    "/dev/",
    "/visuals/?view=icons",
    "/lab/",
    "/dev/process/",
    "/dev/reviews/",
    "/dev/unknown/",
  ]) {
    await page.goto(path);
    await expect(page.locator(".dev-nav")).toBeVisible();
    await expect(page.locator("#developer-content h1")).toHaveCount(1);
    const reads = await page.evaluate(
      () => (window as unknown as { developerStorageReads: string[] }).developerStorageReads,
    );
    expect(reads.filter((key) => /gomoku|firebase|auth/i.test(key))).toEqual([]);
    expect(await page.evaluate(() => localStorage.length)).toBe(1);
  }
  expect(
    requests.filter((url) =>
      /\/assets\/(PlayerApp-|firebase-|cloud_|Board-|wasm_bridge-|bot_worker-|replay_analysis_worker-)|\.wasm(?:$|\?)/.test(
        url,
      ),
    ),
  ).toEqual([]);
  expect(
    requests.filter((url) => /googleapis|identitytoolkit|securetoken|firestore|gapi/.test(url)),
  ).toEqual([]);
});

test("canonical icon selection, comparison, filters and history survive navigation", async ({
  page,
}) => {
  await page.goto("/visuals/?view=icons&compare=share,save,publish");
  await expect(page.locator("[data-inspect]")).toHaveCount(3);
  await expect(page.locator(".asset-catalog-open")).toHaveCount(41);
  await page.getByRole("button", { name: "Compare Home", exact: true }).click();
  await expect(page.locator("[data-inspect]")).toHaveCount(4);
  await page.getByRole("button", { name: "Compare Bot", exact: true }).click();
  await expect(page.getByRole("status")).toContainText("up to four");
  await page.goBack();
  await expect(page.locator("[data-inspect]")).toHaveCount(3);
  await page.goForward();
  await page.reload();
  await expect(page.locator("[data-inspect]")).toHaveCount(4);
  await page.getByRole("searchbox").fill("sign");
  await expect(page.locator(".asset-catalog-open")).toHaveCount(2);
  await page.getByRole("searchbox").fill("no-match-here");
  await expect(page.getByText("No matching icons.")).toBeVisible();
  await page.goto("/visuals/?view=icons&compare=missing,share,share,../../secret");
  await expect(page.locator("[data-inspect]")).toHaveCount(1);
  await expect(page.locator('[data-inspect="share"]')).toBeVisible();
});

test("developer pages and inspector detail views reflow with enlarged text", async ({ page }) => {
  test.setTimeout(90_000);
  for (const [width, height] of [
    [1280, 900],
    [390, 844],
    [320, 800],
    [844, 390],
  ])
    for (const font of ["100%", "200%"]) {
      await page.setViewportSize({ width, height });
      for (const path of [
        "/dev/",
        "/dev/process/",
        "/dev/reviews/",
        "/visuals/",
        "/visuals/?view=icons&compare=share,save,publish,home",
        "/visuals/?view=sprites",
        "/lab/",
        "/lab/?tab=search",
      ]) {
        await page.goto(path);
        await expect(page.locator("#developer-content h1")).toHaveCount(1);
        await page.evaluate((size) => (document.documentElement.style.fontSize = size), font);
        await page.evaluate(() => document.fonts.ready);
        if (path.startsWith("/lab/"))
          await page
            .locator(
              `[data-view="${path.includes("search") ? "search" : "ranking"}"] > div details`,
            )
            .last()
            .waitFor();
        if (path === "/visuals/") await page.locator(".asset-swatch").last().waitFor();
        if (path.includes("view=sprites"))
          await page.locator('canvas[data-ready="true"]').first().waitFor();
        if (path === "/dev/reviews/") await page.locator(".review-index article").last().waitFor();
        const overflow = await page
          .locator(".dev-page")
          .evaluate((e) => e.scrollWidth - e.clientWidth);
        expect(overflow, `${path} ${width} ${font}`).toBeLessThanOrEqual(1);
      }
    }
});

test("public navigation, keyboard entry and unknown-route recovery work", async ({ page }) => {
  await page.goto("/dev/");
  await expect(page.getByRole("heading", { name: "Inside Gomoku2D", exact: true })).toBeVisible();
  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: "Skip to content", exact: true })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.locator("#developer-content")).toBeFocused();
  await page
    .getByRole("navigation", { name: "Developer sections" })
    .getByRole("link", { name: "Process", exact: true })
    .click();
  await expect(page.locator("#developer-content")).toBeFocused();
  await page.goBack();
  await expect(page.getByRole("heading", { name: "Inside Gomoku2D", exact: true })).toBeVisible();
  await page.goto("/dev/not-a-page/");
  await expect(page.getByRole("heading", { name: "Reference not found" })).toBeVisible();
  await page.getByRole("link", { name: "Developer overview", exact: true }).click();
  await expect(page).toHaveURL(/\/dev\/$/);
});

test("curated reviews retain frozen media and comparison controls", async ({ page }) => {
  await page.goto("/dev/reviews/mobile-records-062/");
  await expect(
    page.getByRole("heading", { name: "Keep mobile records within reach", exact: true }),
  ).toBeVisible();
  await page
    .locator(".review-capture img")
    .evaluateAll((images) =>
      Promise.all(images.map((image) => (image as HTMLImageElement).decode())),
    );
  expect(
    await page
      .locator(".review-capture img")
      .evaluateAll((images) =>
        images.every((image) => (image as HTMLImageElement).naturalWidth === 390),
      ),
  ).toBe(true);
  await page.getByRole("button", { name: "Native pixels", exact: true }).click();
  const panes = page.locator(".review-capture");
  await panes.first().evaluate((element) => {
    element.scrollTop = element.scrollHeight;
  });
  await expect
    .poll(() =>
      panes.nth(1).evaluate((element) =>
        Math.abs(element.scrollTop - (element.scrollHeight - element.clientHeight)),
      ),
    )
    .toBeLessThanOrEqual(1);
  await page.getByRole("button", { name: "Scroll together on", exact: true }).click();
  const held = await panes.nth(1).evaluate((element) => element.scrollTop);
  await panes.first().evaluate((element) => {
    element.scrollTop = 0;
  });
  await page.evaluate(
    () => new Promise<void>((resolve) =>
      requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
    ),
  );
  expect(await panes.nth(1).evaluate((element) => element.scrollTop)).toBe(held);
  await page.getByRole("button", { name: "Scroll together off", exact: true }).click();
  await page.getByRole("button", { name: "Fit image width", exact: true }).click();
  await page.getByRole("button", { name: "Show full captures", exact: true }).click();
  await expect(page.locator(".review-full")).toBeVisible();
  await page.getByRole("button", { name: "Public player replay rows", exact: true }).click();
  await expect(page.locator('img[src$="public-after.png"]')).toBeVisible();
  await page.reload();
  await expect(page.locator('img[src$="public-after.png"]')).toBeVisible();
  await page.getByRole("button", { name: "Native pixels", exact: true }).click();
  await expect(page.locator(".review-native")).toBeVisible();
  await page.setViewportSize({ width: 320, height: 800 });
  await page.evaluate(() => (document.documentElement.style.fontSize = "200%"));
  expect(
    await page.locator(".dev-page").evaluate((e) => e.scrollWidth - e.clientWidth),
  ).toBeLessThanOrEqual(1);
  await page.goto("/dev/reviews/core-screens-062/");
  await page
    .getByRole("combobox", { name: "Screen and conditions" })
    .selectOption("profile-stress");
  await page.setViewportSize({ width: 320, height: 800 });
  await page.evaluate(() => (document.documentElement.style.fontSize = "200%"));
  await page
    .getByRole("combobox", { name: "Screen and conditions" })
    .selectOption("public-player-stress");
  await page.getByText("Technical evidence, sources and media identity", { exact: true }).click();
  expect(
    await page
      .locator(".dev-page")
      .evaluate((element) => element.scrollWidth - element.clientWidth),
  ).toBeLessThanOrEqual(1);
  await expect(page.locator('img[src$="public-player-enlarged.png"]')).toBeVisible();
  await page.goto("/dev/reviews/unknown/");
  await expect(page.getByRole("heading", { name: "Review not found", exact: true })).toBeVisible();
});

test("sprite previews preserve player colour, stepping and reduced motion", async ({ page }) => {
  await page.goto("/visuals/?view=sprites&animation=stone-idle-1&tint=black");
  const canvas = page.locator(".asset-sprite-stage canvas");
  await expect(canvas).toHaveAttribute("data-ready", "true");
  const rgba = await canvas.evaluate((element) =>
    Array.from((element as HTMLCanvasElement).getContext("2d")!.getImageData(0, 0, 16, 16).data),
  );
  expect(Math.max(...rgba.filter((_, index) => index % 4 !== 3))).toBeLessThanOrEqual(64);
  expect(rgba.some((value, index) => index % 4 === 3 && value > 0)).toBe(true);
  await page.getByRole("button", { name: "Next frame", exact: true }).click();
  await expect(canvas).toHaveAttribute("data-frame", "1");
  await page.emulateMedia({ reducedMotion: "reduce" });
  await expect(page.getByRole("button", { name: "Play animation", exact: true })).toBeDisabled();
  await page.getByRole("button", { name: "Next frame", exact: true }).click();
  await expect(canvas).toHaveAttribute("data-frame", "2");
  await page.getByRole("button", { name: "Reset frame", exact: true }).click();
  await expect(canvas).toHaveAttribute("data-frame", "0");
  await expect(page.getByRole("heading", { name: "Board layering", exact: true })).toBeVisible();
});

test("native icon rasters and published sources retain canonical pixels", async ({ page }) => {
  const { readFile } = await import("node:fs/promises");
  const { resolve } = await import("node:path");
  const manifest = JSON.parse(await readFile(resolve("assets/icons/manifest.json"), "utf8")) as {
    icons: { name: string }[];
  };
  for (const icon of manifest.icons) {
    const response = await page.request.get(`/assets/icons/${icon.name}.svg`);
    expect(response.ok()).toBe(true);
    expect(await response.body()).toEqual(await readFile(resolve(`assets/icons/${icon.name}.svg`)));
  }
  const raw = await readFile(resolve("assets/icons/share.svg"), "utf8");
  await page.goto("/visuals/?view=icons&icon=share&grid=off");
  await expect(page.locator('[data-inspect="share"]')).toBeVisible();
  await page.evaluate(() => document.fonts.ready);
  for (const [selector, scale] of [
    [".asset-native-one", 1],
    [".asset-native-two", 2],
    [".asset-pixel-grid", 10],
  ] as const) {
    const png = await page.locator(selector).screenshot();
    const mismatch = await page.evaluate(
      async ({ png, raw, scale }) => {
        const svg = new DOMParser().parseFromString(raw, "image/svg+xml"),
          mask = new Set<number>();
        for (const rect of svg.querySelectorAll("rect")) {
          const x = Number(rect.getAttribute("x")),
            y = Number(rect.getAttribute("y")),
            w = Number(rect.getAttribute("width")),
            h = Number(rect.getAttribute("height"));
          for (let row = y; row < y + h; row++)
            for (let col = x; col < x + w; col++) mask.add(row * 24 + col);
        }
        const image = new Image();
        image.src = `data:image/png;base64,${png}`;
        await image.decode();
        const canvas = document.createElement("canvas");
        canvas.width = image.width;
        canvas.height = image.height;
        const context = canvas.getContext("2d")!;
        context.drawImage(image, 0, 0);
        const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data;
        let best = Infinity;
        for (let dy = -1; dy <= 1; dy++)
          for (let dx = -1; dx <= 1; dx++) {
            let differences = 0;
            for (let y = 0; y < canvas.height; y++)
              for (let x = 0; x < canvas.width; x++) {
                const sx = Math.floor((x - dx) / scale),
                  sy = Math.floor((y - dy) / scale);
                const expected = sx >= 0 && sx < 24 && sy >= 0 && sy < 24 && mask.has(sy * 24 + sx);
                if (pixels[(y * canvas.width + x) * 4] > 200 !== expected) differences++;
              }
            best = Math.min(best, differences);
          }
        const colours = new Set(
          Array.from({ length: pixels.length / 4 }, (_, index) =>
            Array.from(pixels.slice(index * 4, index * 4 + 4)).join(","),
          ),
        );
        const antialias = [...colours].filter(
          (colour) => !["30,30,30,255", "245,245,245,255", "87,87,86,255"].includes(colour),
        );
        return { best, antialias };
      },
      { png: png.toString("base64"), raw, scale },
    );
    expect(
      mismatch.best,
      `${scale}x source pixel mask, allowing only fractional screenshot crop translation`,
    ).toBe(0);
    expect(mismatch.antialias).toEqual([]);
  }
});
