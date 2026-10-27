import { expect, test } from "@playwright/test";
import { readFileSync } from "node:fs";

const catalog = JSON.parse(
  readFileSync(
    new URL("../src/developer/history.json", import.meta.url),
    "utf8",
  ),
) as { screens: { id: string }[] };

const root = "/dev/reviews/history/";

test("milestones distinguish updates, introductions and report retirement", async ({
  page,
}) => {
  await page.goto(root);
  await expect(page).toHaveTitle(/UI history/);
  await expect(page.locator(".history-milestone-link")).toHaveCount(24);
  await expect(page.locator(".history-milestone-link").first()).toContainText(
    "v0.6.2",
  );
  await expect(page.locator(".history-milestone-link").nth(1)).toContainText(
    "v0.6.1",
  );
  await expect(page.locator(".history-milestone-link").last()).toContainText(
    "v0.2.1",
  );
  await expect(
    page.locator('.history-milestone-link[aria-current="page"]'),
  ).toContainText("v0.6.2");
  await expect(
    page.locator('.history-milestone-link[aria-current="page"]'),
  ).toBeInViewport({ ratio: 1 });
  await page.goto(root + "?version=0.2.1");
  await expect(page.locator(".history-change")).toHaveCount(4);
  for (const change of await page.locator(".history-change").all())
    await expect(change.locator(".review-capture img")).toHaveCount(1);
  await page.goto(root + "?version=0.2.2");
  await expect(
    page.locator(".history-change").first().locator(".review-capture img"),
  ).toHaveCount(2);
  await page
    .getByRole("navigation", { name: "Changes in this milestone" })
    .getByRole("link", { name: /^Profile/ })
    .click();
  await expect(
    page.locator('[data-comparison="profile-ledger.default"]'),
  ).toBeFocused();
  await page.goto(root + "?version=0.5.1");
  for (const id of ["bot-report", "analysis-report"]) {
    const change = page.locator(".history-change").filter({
      has: page.getByRole("heading", {
        name: id === "bot-report" ? "Bot Report" : "Analysis Report",
        exact: true,
      }),
    });
    await expect(change.locator(".history-lifecycle-note").first()).toHaveText(
      "Removed in v0.5.1",
    );
    await expect(
      change.getByRole("link", { name: "Lab", exact: true }),
    ).toHaveCount(1);
    for (const state of await change.locator(".history-state-comparison").all())
      await expect(state.locator(".review-capture img")).toHaveCount(1);
  }
  await page.goto(root + "?version=missing&layout=missing");
  await expect(
    page.locator('.history-milestone-link[aria-current="page"]'),
  ).toContainText("v0.6.2");
  await expect(
    page.getByRole("button", { name: "Desktop", exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
});

test("screen inspection and browser Back restore the milestone's focus, scroll and disclosures", async ({
  page,
}) => {
  await page.goto(root + "?version=0.6.2&layout=portrait");
  const change = page.locator('[data-change="profile-public-controls"]');
  await change
    .locator(".history-source-context")
    .first()
    .evaluate((e) => ((e as HTMLDetailsElement).open = true));
  const inspect = change.getByRole("link", {
    name: "Inspect Profile history",
    exact: true,
  });
  await inspect.scrollIntoViewIfNeeded();
  const scroll = await page.locator(".dev-page").evaluate((e) => e.scrollTop);
  await inspect.click();
  await expect(
    page.getByRole("heading", { name: "Profile, over time", exact: true }),
  ).toBeVisible();
  await expect(page.locator("#history-inspect")).toBeFocused();
  await page.goBack();
  await expect(change).toBeFocused();
  await expect(
    change.locator(".history-source-context").first(),
  ).toHaveAttribute("open", "");
  await expect
    .poll(() => page.locator(".dev-page").evaluate((e) => e.scrollTop))
    .toBeCloseTo(scroll, 0);
  await inspect.click();
  await page
    .locator("#history-inspect")
    .getByRole("button", { name: "Native pixels", exact: true })
    .click();
  await page
    .getByRole("navigation", { name: "Screen histories" })
    .getByRole("link", { name: "Replay", exact: true })
    .click();
  await page.locator(".history-back").first().click();
  await expect(page).toHaveURL(/history\/\?version=0.6.2&layout=portrait$/);
  await expect(change).toBeFocused();
  await page.goto(root + "?version=0.5.1&layout=portrait");
  const retired = page.locator(".history-change").filter({
    has: page.getByRole("heading", { name: "Bot Report", exact: true }),
  });
  await retired.getByRole("link", { name: "Lab", exact: true }).click();
  await expect(page).toHaveURL(/history\/lab\//);
  await page.locator(".history-back").first().click();
  await expect(retired).toBeFocused();
  await expect(page).toHaveURL(/history\/\?version=0.5.1&layout=portrait$/);
});

test("screen trails keep named views, source revisions and retirement without dummy captures", async ({
  page,
}) => {
  await page.goto(
    root + "bot-report/?version=0.5.1&state=default&layout=portrait",
  );
  await expect(page.locator(".history-retired")).toContainText(
    "Removed in v0.5.1",
  );
  await expect(
    page.locator("#history-inspect .review-capture img"),
  ).toHaveCount(1);
  await expect(page.locator(".history-trail-entry").first()).toContainText(
    "v0.5.0",
  );
  await expect(page.locator(".history-trail-entry").last()).toContainText(
    "v0.4.0",
  );
  await page.goto(root + "replay/?version=0.6.1&state=share");
  await expect(
    page
      .getByRole("navigation", { name: "Replay views" })
      .getByRole("link", { name: "Share replay", exact: true }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.locator("#history-inspect .review-capture img"),
  ).toHaveCount(1);
  await page.reload();
  await expect(page.locator("#history-inspect")).toContainText(
    "New view in v0.6.1",
  );
  await page.goto(root + "missing/");
  await expect(
    page.getByRole("heading", {
      name: "Screen history not found",
      exact: true,
    }),
  ).toBeVisible();
  await page
    .getByRole("link", { name: "UI history", exact: true })
    .last()
    .click();
  await expect(page.locator('[data-history-view="milestones"]')).toBeVisible();
});

test("history reflows with enlarged text and an operable mobile milestone disclosure", async ({
  page,
}) => {
  test.setTimeout(90_000);
  for (const width of [1280, 412, 320]) {
    await page.setViewportSize({ width, height: 900 });
    for (const path of [
      root + "?version=0.5.1",
      root + "profile/?version=0.6.2&layout=portrait",
    ]) {
      await page.goto(path);
      await page.evaluate(() => document.fonts.ready);
      await page.evaluate(
        () => (document.documentElement.style.fontSize = "200%"),
      );
      expect(
        await page
          .locator(".dev-page")
          .evaluate((e) => e.scrollWidth - e.clientWidth),
        `${path} at ${width}`,
      ).toBeLessThanOrEqual(1);
    }
  }
  await page.goto(root);
  const rail = page.locator(".history-milestone-rail");
  await expect(rail).not.toHaveAttribute("open", "");
  await rail.locator("summary").click();
  await rail.getByRole("link", { name: /v0.2.1 / }).click();
  await expect(rail).not.toHaveAttribute("open", "");
  await expect(page.locator(".history-milestone-heading > h2")).toBeFocused();
});

test("all history entries remain static public references without private state or engine initialization", async ({
  page,
}) => {
  test.setTimeout(90_000);
  const requests: string[] = [];
  await page.route("**/*", (route) =>
    ["127.0.0.1", "localhost"].includes(new URL(route.request().url()).hostname)
      ? route.continue()
      : route.abort(),
  );
  page.on("request", (request) => requests.push(request.url()));
  await page.addInitScript(() => {
    localStorage.setItem("gomoku2d.local-profile.v5", "private sentinel");
    const reads: string[] = [];
    Object.assign(window, { historyStorageReads: reads });
    const get = Storage.prototype.getItem;
    Storage.prototype.getItem = function (key) {
      reads.push(key);
      return get.call(this, key);
    };
  });
  for (const path of [root, ...catalog.screens.map((s) => root + s.id + "/")]) {
    await page.goto(path);
    await expect(page.locator("#developer-content h1")).toHaveCount(1);
    expect(
      await page.evaluate(() =>
        (
          window as unknown as { historyStorageReads: string[] }
        ).historyStorageReads.filter((key) =>
          /gomoku|firebase|auth/i.test(key),
        ),
      ),
    ).toEqual([]);
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
    requests.filter((url) =>
      /googleapis|identitytoolkit|securetoken|firestore|gapi/.test(url),
    ),
  ).toEqual([]);
});
