import { test, expect, signIn, readProfile, cloudTestOrigins } from "./helpers.js";

test("emulated redirect sign-in restores the account in an embedded client", async ({ page }) => {
  // Embedding is an actual supported redirect-policy input in every engine.
  // Firefox hasTouch alone does not emulate a coarse pointer.
  await page.route("**/embedded-host", (route) => route.fulfill({ contentType: "text/html",
    body: '<iframe title="Embedded Gomoku2D" src="/profile" style="width:1200px;height:850px"></iframe>' }));
  await page.goto("/embedded-host");
  const frame = await page.getByTitle("Embedded Gomoku2D", { exact: true }).elementHandle().then((element) => element!.contentFrame());
  if (!frame) throw new Error("Embedded client did not load");
  await frame.waitForURL("**/profile");
  const uid = await signIn(frame, `redirect-${Date.now()}@example.test`, { redirect: true, navigate: false });
  await expect.poll(async () => (await readProfile(uid))?.uid).toBe(uid);
  await frame.goto(`${cloudTestOrigins.preview}/profile`);
  await expect(frame.getByRole("button", { name: "Sign out", exact: true })).toBeVisible();
});
