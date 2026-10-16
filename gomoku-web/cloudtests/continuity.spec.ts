import { test, expect, signIn, readProfile, protectContext, makeProfileEligible } from "./helpers.js";
import { seedLocalSavedMatch } from "../playtests/helpers/local_history.js";

const ending = [112, 0, 113, 15, 114, 30, 115, 1, 116].map((cell) => ({ row: Math.floor(cell / 15), col: cell % 15 }));
async function seedHistory(page: import("@playwright/test").Page, id: string, preferredVariant: "renju" | "freestyle") {
  await page.goto("/");
  await seedLocalSavedMatch(page, { displayName: "Cloud fixture", id, moves: ending,
    savedAt: new Date().toISOString(), status: "black_won", variant: "freestyle", preferredVariant });
}

test("C1/C2/C8: promote guest history, restore account setup, merge another client's history and reload", async ({ page, browser }) => {
  const email = `cloud-${Date.now()}@example.test`;
  await seedHistory(page, "first-game", "renju");
  const uid = await signIn(page, email);
  await expect.poll(async () => (await readProfile(uid))?.match_history.replay_matches.length).toBe(1);
  expect((await readProfile(uid))?.settings.game_config.ruleset).toBe("renju");
  await makeProfileEligible(uid);
  const context = await browser.newContext();
  await protectContext(context);
  try {
    const second = await context.newPage();
    await seedHistory(second, "second-game", "freestyle");
    await signIn(second, email);
    await expect.poll(async () => (await readProfile(uid))?.match_history.replay_matches.length).toBe(2);
    const profile = await readProfile(uid);
    expect(profile?.settings.game_config.ruleset).toBe("renju");
    expect(profile?.match_history.replay_matches.map((match: { id: string }) => match.id).sort()).toEqual(["first-game", "second-game"]);
    await second.reload();
    await expect(second.getByRole("button", { name: "Sign out", exact: true })).toBeVisible();
    await expect(second.getByRole("button", { name: "Inspect", exact: true })).toHaveCount(2);
    await second.getByRole("link", { name: "Settings", exact: true }).click();
    await expect(second.getByRole("button", { name: "Renju", exact: true })).toHaveAttribute("aria-pressed", "true");
  } finally { await context.close(); }
});
