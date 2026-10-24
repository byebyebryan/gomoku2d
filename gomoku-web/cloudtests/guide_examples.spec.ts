import { test, expect, signIn, fixtureId, makeProfileEligible, cloudTestOrigins, cloudTestPorts, projectId } from "./helpers.js";

async function publicRoot(uid: string): Promise<Record<string, unknown> | null> {
  const response = await fetch(`${cloudTestOrigins.firestore}/v1/projects/${projectId}/databases/(default)/documents/public_profiles/${uid}`, {
    headers: { Authorization: "Bearer owner" },
  });
  if (response.status === 404) return null;
  if (!response.ok) throw new Error(await response.text());
  return (await response.json()).fields;
}

test("C19: explicit owner enable and account-free examples preserve visibility and private-read boundaries", async ({ page }, info) => {
  test.setTimeout(90_000);
  const uid = await signIn(page, `${fixtureId("guide-owner")}@example.test`);
  await makeProfileEligible(uid);
  await page.reload();
  await page.getByRole("tab", { name: "Public profile", exact: true }).click();
  const name = page.getByRole("textbox", { name: "Choose your public name", exact: true });
  await name.fill("Example owner");
  await page.getByRole("button", { name: "Enable public profile", exact: true }).click();
  await expect(page.getByRole("button", { name: "Confirm enable", exact: true })).toBeFocused();
  expect(await publicRoot(uid)).toBeNull();
  await page.getByRole("button", { name: "Cancel enable", exact: true }).click();
  await expect(page.getByRole("button", { name: "Enable public profile", exact: true })).toBeFocused();
  expect(await publicRoot(uid)).toBeNull();
  await page.getByRole("button", { name: "Enable public profile", exact: true }).click();
  await name.fill("Revised example owner");
  await expect(page.getByRole("button", { name: "Confirm enable", exact: true })).toHaveCount(0);
  expect(await publicRoot(uid)).toBeNull();
  await page.getByRole("button", { name: "Enable public profile", exact: true }).click();
  await page.screenshot({ path: info.outputPath("enable-confirmation.png"), animations: "disabled" });
  await page.getByRole("button", { name: "Confirm enable", exact: true }).click();
  await expect(page.getByRole("button", { name: "Save public name", exact: true })).toBeVisible();
  const before = await publicRoot(uid);
  expect(before?.enabled).toEqual({ booleanValue: true });
  expect(before?.shared_stats).toEqual({ nullValue: null });
  await page.goto("/guide/#examples");
  const href = await page.getByRole("link", { name: "Open Try another line", exact: true }).getAttribute("href");
  expect(href).toMatch(/^\/replay\/shared\/#g2r1=/);
  const standalone = await page.context().newPage();
  const database: string[] = [];
  standalone.on("request", (request) => {
    if (new URL(request.url()).port === String(cloudTestPorts.firestore)) database.push(request.url());
  });
  await standalone.goto(href!);
  await expect(standalone.getByTestId("replay-move-count")).toHaveText("Move 26 / 36");
  await expect(standalone.getByRole("button", { name: "Save replay", exact: true })).toBeVisible();
  await expect(standalone.getByRole("button", { name: "Publish replay", exact: true })).toHaveCount(0);
  await expect(standalone.getByRole("button", { name: "Play From Here", exact: true })).toBeEnabled();
  expect(database).toEqual([]);
  expect(await publicRoot(uid)).toEqual(before);
  await standalone.close();
});
