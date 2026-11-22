import { cloudTestOrigins, expect, fixtureId, projectId, readProfile, signIn, test } from "./helpers.js";

test("unreadable device replay cache blocks private reset before the cloud library is cleared", async ({ page }) => {
  const uid = await signIn(page, `${fixtureId("reset-cache-preflight")}@example.test`);
  await expect.poll(() => readProfile(uid)).not.toBeNull();
  const profileBefore = await readProfile(uid);
  const payload = {
    v: 1,
    r: "freestyle",
    m: Buffer.from([110, 0, 111, 1, 112, 2, 113, 3, 114]).toString("base64url"),
    b: { k: "h", n: "Reset probe" },
    w: { k: "h", n: "Guest" },
    at: 4,
  };
  await page.goto(`/replay/shared/#g2r1=${Buffer.from(JSON.stringify(payload)).toString("base64url")}`);
  await page.getByRole("button", { name: "Save replay", exact: true }).click();
  await expect(page.getByText("Saved in your account library.", { exact: true })).toBeVisible();

  const readLibrary = async () => {
    const response = await fetch(`${cloudTestOrigins.firestore}/v1/projects/${projectId}/databases/(default)/documents/replay_libraries/${uid}`, {
      headers: { Authorization: "Bearer owner" },
    });
    expect(response.ok).toBe(true);
    return (await response.json()).fields as Record<string, { stringValue?: string; integerValue?: string }>;
  };
  const libraryBefore = await readLibrary();
  expect(JSON.parse(libraryBefore.entries_json.stringValue!)).toHaveLength(1);

  const storageKey = "gomoku2d.replay-library.v2";
  const cacheBytes = "unreadable local fixture";
  await page.evaluate(([key, value]) => localStorage.setItem(key, value), [storageKey, cacheBytes]);
  await page.goto("/profile");
  await expect(page.getByRole("button", { name: "Sign out", exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Reset Profile", exact: true }).click();
  await page.getByRole("button", { name: "Reset", exact: true }).click();
  await expect(page.getByRole("alert")).toHaveText("Replay library cache is not valid JSON.");

  const libraryAfter = await readLibrary();
  const profileAfter = await readProfile(uid);
  expect(libraryAfter.entries_json.stringValue).toBe(libraryBefore.entries_json.stringValue);
  expect(libraryAfter.revision.integerValue).toBe(libraryBefore.revision.integerValue);
  expect(profileAfter?.reset_at).toEqual(profileBefore?.reset_at);
  expect(await page.evaluate((key) => localStorage.getItem(key), storageKey)).toBe(cacheBytes);
  await expect(page.getByRole("button", { name: "Reset", exact: true })).toBeVisible();
});
