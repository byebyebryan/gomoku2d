import { test, expect, signIn, readProfile, protectContext, makeProfileEligible, projectId, cloudTestOrigins, fixtureId } from "./helpers.js";
import { seedLocalSavedMatch } from "../playtests/helpers/local_history.js";

const ending = [112, 0, 113, 15, 114, 30, 115, 1, 116].map((cell) => ({ row: Math.floor(cell / 15), col: cell % 15 }));
async function seedHistory(page: import("@playwright/test").Page, id: string, preferredVariant: "renju" | "freestyle") {
  await page.goto("/");
  await seedLocalSavedMatch(page, { displayName: "Cloud fixture", id, moves: ending,
    savedAt: new Date().toISOString(), status: "black_won", variant: "freestyle", preferredVariant });
}

async function readyCloud(page: import("@playwright/test").Page) {
  await expect(page.getByText(/^(Cloud sync is on\.|Local history synced to cloud\.)$/)).toBeVisible();
}

async function readPrivateLibrary(uid: string): Promise<{ revision: number; entries: unknown[] } | null> {
  const response = await fetch(`${cloudTestOrigins.firestore}/v1/projects/${projectId}/databases/(default)/documents/replay_libraries/${uid}`, {
    headers: { Authorization: "Bearer owner" },
  });
  if (response.status === 404) return null;
  if (!response.ok) throw new Error(await response.text());
  const document = await response.json() as { fields: Record<string, { integerValue?: string; stringValue?: string }> };
  const fields = document.fields;
  return {
    revision: Number(fields.revision?.integerValue),
    entries: JSON.parse(fields.entries_json?.stringValue ?? "[]") as unknown[],
  };
}

async function finishBranch(page: import("@playwright/test").Page, sourceId: string) {
  await page.goto(`/replay/${sourceId}`);
  await page.getByRole("button", { name: "Previous move", exact: true }).click();
  await page.getByRole("button", { name: "Play From Here", exact: true }).click();
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 8");
  const winningCell = page.getByRole("gridcell", { name: /^L8:/ });
  await winningCell.focus();
  await winningCell.press("Enter");
  await expect(page.getByTestId("match-status")).toHaveText(/wins$/);
}

async function resetProfile(page: import("@playwright/test").Page) {
  await page.getByRole("button", { name: "Reset Profile", exact: true }).click();
  await page.getByRole("button", { name: "Reset", exact: true }).click();
}

test("C3: a completed continuation syncs as a regular game and opens from a second client", async ({ page, browser }) => {
  await seedHistory(page, "branch-source", "freestyle");
  const email = `${fixtureId("finish")}@example.test`;
  const uid = await signIn(page, email);
  await readyCloud(page);
  await makeProfileEligible(uid);
  await page.reload(); await readyCloud(page);
  await finishBranch(page, "branch-source");
  await page.getByRole("link", { name: "Profile", exact: true }).click();
  await expect(page.getByRole("button", { name: "Inspect", exact: true })).toHaveCount(2);
  await expect(page.getByRole("region", { name: "Practice games" })).toHaveCount(0);
  await expect.poll(async () => (await readProfile(uid))?.match_history.replay_matches.length).toBe(2);
  const continuation = (await readProfile(uid))?.match_history.replay_matches.find((match: { id: string }) => match.id !== "branch-source");
  expect(continuation.replay_origin).toMatchObject({ inherited_ply: 8, source_fingerprint: expect.stringMatching(/^[A-Za-z0-9_-]{43}$/) });
  expect(continuation.undo_floor).toBe(8);
  expect(await readPrivateLibrary(uid)).toBeNull();

  const context = await browser.newContext(); await protectContext(context);
  try {
    const second = await context.newPage(); await signIn(second, email); await readyCloud(second);
    await expect(second.getByRole("button", { name: "Inspect", exact: true })).toHaveCount(2);
    await second.goto(`/replay/${continuation.id}`);
    await expect(second.getByTestId("replay-move-count")).toHaveText("Move 9 / 9");
    await expect(second.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
    await expect(second.getByText("Practice game", { exact: true })).toHaveCount(0);
    await second.getByRole("button", { name: "Share replay", exact: true }).click();
    const url = await second.getByRole("textbox", { name: "Replay link", exact: true }).inputValue();
    const wire = JSON.parse(Buffer.from(new URL(url).hash.slice("#g2r1=".length), "base64url").toString("utf8"));
    expect(wire.o).toEqual({ f: continuation.replay_origin.source_fingerprint, p: 8 });
    expect(wire.b.u).toBeUndefined(); // Profile references remain opt-in when exporting.
  } finally { await context.close(); }
});

test("C4: a stale client cannot overwrite another client's setup", async ({ page, browser }) => {
  const email = `${fixtureId("conflict")}@example.test`;
  const uid = await signIn(page, email);
  await readyCloud(page);
  await makeProfileEligible(uid);
  await page.reload(); await readyCloud(page);
  const context = await browser.newContext(); await protectContext(context);
  try {
    const second = await context.newPage();
    await signIn(second, email); await readyCloud(second);
    await page.getByRole("link", { name: "Settings", exact: true }).click();
    await page.getByRole("button", { name: "Renju", exact: true }).click();
    await expect.poll(async () => (await readProfile(uid))?.settings.game_config.ruleset).toBe("renju");
    await second.getByRole("link", { name: "Settings", exact: true }).click();
    await second.getByRole("button", { name: /^Easy/ }).click();
    await second.getByRole("link", { name: "Profile", exact: true }).click();
    await expect(second.getByText(/Cloud profile changed on another device/)).toBeVisible();
    expect((await readProfile(uid))?.settings.game_config.ruleset).toBe("renju");
    expect((await readProfile(uid))?.settings.bot_config.preset).toBe("normal");
    await second.getByRole("link", { name: "Settings", exact: true }).click();
    await expect(second.getByRole("button", { name: /^Easy/ })).toHaveAttribute("aria-pressed", "true");
  } finally { await context.close(); }
});

test("C5/C8: failed transport preserves a completed continuation in My games and syncs it once on recovery", async ({ page, context }) => {
  await seedHistory(page, "offline-source", "freestyle");
  const email = `${fixtureId("offline")}@example.test`;
  const uid = await signIn(page, email); await readyCloud(page);
  await makeProfileEligible(uid); await page.reload(); await readyCloud(page);
  await page.goto("/replay/offline-source");
  await page.getByRole("button", { name: "Previous move", exact: true }).click();
  await page.getByRole("button", { name: "Play From Here", exact: true }).click();
  await expect(page.getByTestId("match-move-count")).toHaveText("Move 8");
  await context.route(`${cloudTestOrigins.firestore}/**`, (route) => route.abort("connectionfailed"));
  const winningCell = page.getByRole("gridcell", { name: /^L8:/ });
  await winningCell.focus(); await winningCell.press("Enter");
  await expect(page.getByTestId("match-status")).toHaveText(/wins$/);
  await expect(page.getByRole("link", { name: "Analyze", exact: true })).toBeVisible();
  await page.getByRole("link", { name: "Profile", exact: true }).click();
  await expect(page.getByRole("button", { name: "Inspect", exact: true })).toHaveCount(2);
  expect((await readProfile(uid))?.match_history.replay_matches.map((match: { id: string }) => match.id)).toEqual(["offline-source"]);
  expect(await readPrivateLibrary(uid)).toBeNull();

  await page.reload();
  await expect(page.getByRole("button", { name: "Inspect", exact: true })).toHaveCount(2);
  await context.unroute(`${cloudTestOrigins.firestore}/**`);
  await page.reload(); await readyCloud(page);
  await expect.poll(async () => (await readProfile(uid))?.match_history.replay_matches.length).toBe(2);
  await page.reload(); await readyCloud(page);
  await expect(page.getByRole("button", { name: "Inspect", exact: true })).toHaveCount(2);
  expect((await readProfile(uid))?.match_history.replay_matches.filter((match: { id: string }) => match.id !== "offline-source")).toHaveLength(1);
  expect(await readPrivateLibrary(uid)).toBeNull();
});

test("C7/C9: reset survives a stale client and immediate repeated reset is denied", async ({ page, browser }) => {
  await seedHistory(page, "pre-reset", "freestyle");
  const email = `${fixtureId("reset")}@example.test`;
  const uid = await signIn(page, email); await readyCloud(page);
  await makeProfileEligible(uid); await page.reload(); await readyCloud(page);
  const context = await browser.newContext(); await protectContext(context);
  try {
    const second = await context.newPage(); await signIn(second, email); await readyCloud(second);
    await resetProfile(page);
    await expect.poll(async () => (await readProfile(uid))?.match_history.replay_matches.length).toBe(0);
    const resetAt = (await readProfile(uid))?.reset_at;
    expect(resetAt).toBeTruthy();
    await expect(page.getByRole("button", { name: "Reset Profile", exact: true })).toBeEnabled();
    await resetProfile(page);
    await expect(page.getByRole("alert").filter({ hasText: /Missing or insufficient permissions|permission.denied/i })).toBeVisible();
    expect((await readProfile(uid))?.reset_at).toBe(resetAt);
    await second.getByRole("link", { name: "Settings", exact: true }).click();
    await second.getByRole("button", { name: /^Easy/ }).click();
    await second.getByRole("link", { name: "Profile", exact: true }).click();
    await expect(second.getByText(/Cloud profile changed on another device/)).toBeVisible();
    await second.reload();
    await expect(second.getByRole("button", { name: "Sign out", exact: true })).toBeVisible();
    expect((await readProfile(uid))?.match_history.replay_matches).toEqual([]);
    expect((await readProfile(uid))?.reset_at).toBe(resetAt);
  } finally { await context.close(); }
});

test("C6: a delayed account-A load cannot replace account B after switching", async ({ page, browser }) => {
  const emailA = `${fixtureId("account-a")}@example.test`;
  const uidA = await signIn(page, emailA); await readyCloud(page);
  await page.getByRole("link", { name: "Settings", exact: true }).click();
  await page.getByRole("button", { name: "Renju", exact: true }).click();
  await expect.poll(async () => (await readProfile(uidA))?.settings.game_config.ruleset).toBe("renju");
  await page.getByRole("link", { name: "Profile", exact: true }).click();
  const context = await browser.newContext(); await protectContext(context);
  let release = () => {};
  const barrier = new Promise<void>((resolve) => { release = resolve; });
  let observed = () => {};
  const held = new Promise<void>((resolve) => { observed = resolve; });
  let intercepted = false;
  try {
    const second = await context.newPage();
    const emailB = `${fixtureId("account-b")}@example.test`;
    const uidB = await signIn(second, emailB); await readyCloud(second);
    await page.route("**/google.firestore.v1.Firestore/Listen/channel**", async (route) => {
      if (!intercepted && route.request().method() === "POST") {
        intercepted = true; observed(); await barrier;
      }
      await route.continue().catch(() => {}); // Auth switching can cancel the old transport.
    });
    await page.reload(); await held;
    await page.getByRole("button", { name: "Sign out", exact: true }).click();
    await signIn(page, emailB, { navigate: false });
    release(); await readyCloud(page);
    await page.getByRole("link", { name: "Settings", exact: true }).click();
    await expect(page.getByRole("button", { name: "Freestyle", exact: true })).toHaveAttribute("aria-pressed", "true");
    expect((await readProfile(uidB))?.uid).toBe(uidB);
    expect((await readProfile(uidB))?.settings.game_config.ruleset).toBe("freestyle");
    expect((await readProfile(uidA))?.settings.game_config.ruleset).toBe("renju");
  } finally { release(); await context.close(); }
});

test("C1/C2/C8: promote guest history, restore account setup, merge another client's history and reload", async ({ page, browser }) => {
  const email = `${fixtureId("cloud")}@example.test`;
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
