import {
  test,
  expect,
  fixtureId,
  signIn,
  readProfile,
  makeProfileEligible,
  cloudTestPorts,
} from "./helpers.js";
import { seedLocalSavedMatch } from "../playtests/helpers/local_history.js";

test("D01: developer entry ignores a persisted signed-in session and returning restores private history", async ({
  context,
  page,
}) => {
  test.setTimeout(90_000);
  await page.goto("/");
  const id = fixtureId("developer-history");
  await seedLocalSavedMatch(page, {
    displayName: "Developer boundary fixture",
    id,
    moves: [112, 0, 113, 15, 114, 30, 115, 1, 116].map((cell) => ({
      row: Math.floor(cell / 15),
      col: cell % 15,
    })),
    preferredVariant: "freestyle",
    savedAt: "2026-10-24T00:00:00Z",
    status: "black_won",
    variant: "freestyle",
  });
  const uid = await signIn(page, `${fixtureId("developer")}@example.test`);
  await expect
    .poll(async () =>
      Boolean(
        (await readProfile(uid))?.match_history?.replay_matches?.some(
          (match: { id: string }) => match.id === id,
        ),
      ),
    )
    .toBe(true);
  await page.close();
  const before = await readProfile(uid);
  const developer = await context.newPage();
  const requests: string[] = [];
  developer.on("request", (request) => requests.push(request.url()));
  for (const path of [
    "/dev/",
    "/visuals/?view=icons",
    "/lab/?tab=analysis",
    "/dev/reviews/mobile-records-062/",
  ]) {
    await developer.goto(path);
    await expect(developer.locator("#developer-content h1")).toHaveCount(1);
  }
  expect(
    requests.filter(
      (url) =>
        new URL(url).port === String(cloudTestPorts.auth) ||
        new URL(url).port === String(cloudTestPorts.firestore),
    ),
  ).toEqual([]);
  expect(
    requests.filter((url) => /\/assets\/(PlayerApp-|firebase-|cloud_|Board-)|\.wasm$/.test(url)),
  ).toEqual([]);
  expect(await readProfile(uid)).toEqual(before);
  await developer
    .getByRole("navigation", { name: "Project links" })
    .getByRole("link", { name: "Play", exact: true })
    .click();
  await developer.getByRole("link", { name: "Profile", exact: true }).click();
  await expect(developer.getByRole("button", { name: "Sign out", exact: true })).toBeVisible();
  await expect(developer.getByRole("button", { name: "Inspect", exact: true })).toHaveCount(1);
});

test("D02: a pending private snapshot survives game-to-developer navigation", async ({
  page,
  context,
}) => {
  test.setTimeout(90_000);
  const uid = await signIn(page, `${fixtureId("developer-inflight")}@example.test`);
  await expect.poll(() => readProfile(uid)).not.toBeNull();
  await makeProfileEligible(uid);
  await page.reload();
  await expect(page.getByRole("button", { name: "Sign out", exact: true })).toBeVisible();
  let release!: () => void;
  const gate = new Promise<void>((resolve) => {
    release = resolve;
  });
  let intercepted = false;
  await context.route(`http://127.0.0.1:${cloudTestPorts.firestore}/**`, async (route) => {
    if (
      route.request().method() === "POST" &&
      /Write\/channel|:commit/.test(route.request().url())
    ) {
      intercepted = true;
      await gate;
    }
    await route.continue();
  });
  try {
    await page.getByRole("link", { name: "Settings", exact: true }).click();
    await page.getByRole("button", { name: "Renju", exact: true }).click();
    await expect.poll(() => intercepted).toBe(true);
    await page.getByRole("link", { name: "Home", exact: true }).click();
    await page.getByRole("link", { name: "Developer", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Inside Gomoku2D", exact: true })).toBeVisible();
    release();
    await expect
      .poll(async () => (await readProfile(uid))?.settings?.game_config?.ruleset)
      .toBe("renju");
    await page
      .getByRole("navigation", { name: "Project links" })
      .getByRole("link", { name: "Play", exact: true })
      .click();
    await page.getByRole("link", { name: "Settings", exact: true }).click();
    await expect(page.getByRole("button", { name: "Renju", exact: true })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  } finally {
    release();
    await context.unroute(`http://127.0.0.1:${cloudTestPorts.firestore}/**`);
  }
});
