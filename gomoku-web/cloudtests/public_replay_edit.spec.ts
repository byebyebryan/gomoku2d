import { fixtureId, makeProfileEligible, projectId, protectContext, signIn, test, expect, cloudTestOrigins } from "./helpers.js";
import { seedLocalSavedMatch } from "../playtests/helpers/local_history.js";

const FINISHED_MATCH_MOVES = [112, 0, 113, 15, 114, 30, 115, 1, 116].map((cell) => ({
  row: Math.floor(cell / 15),
  col: cell % 15,
}));

function decodeFirestoreValue(value: unknown): unknown {
  if (!value || typeof value !== "object") return undefined;
  const field = value as Record<string, unknown>;
  if ("nullValue" in field) return null;
  if ("timestampValue" in field) return field.timestampValue;
  if ("booleanValue" in field) return field.booleanValue;
  if ("stringValue" in field) return field.stringValue;
  if ("integerValue" in field) return Number(field.integerValue);
  if ("doubleValue" in field) return field.doubleValue;
  if ("mapValue" in field) {
    return decodeFirestoreFields((field.mapValue as { fields?: Record<string, unknown> }).fields ?? {});
  }
  if ("arrayValue" in field) {
    return ((field.arrayValue as { values?: unknown[] }).values ?? []).map(decodeFirestoreValue);
  }
  return undefined;
}

function decodeFirestoreFields(fields: Record<string, unknown>): Record<string, unknown> {
  return Object.fromEntries(Object.entries(fields).map(([key, value]) => [key, decodeFirestoreValue(value)]));
}

async function readFirestoreDocument(path: string): Promise<Record<string, unknown> | null> {
  const encodedPath = path.split("/").map(encodeURIComponent).join("/");
  const response = await fetch(`${cloudTestOrigins.firestore}/v1/projects/${projectId}/databases/(default)/documents/${encodedPath}`, {
    headers: { Authorization: "Bearer owner" },
  });
  if (response.status === 404) return null;
  if (!response.ok) throw new Error(`Firestore read failed for ${path}: ${await response.text()}`);
  const document = await response.json() as { fields?: Record<string, unknown> };
  return decodeFirestoreFields(document.fields ?? {});
}

async function readyCloud(page: import("@playwright/test").Page): Promise<void> {
  await expect(page.getByRole("button", { name: "Sign out", exact: true })).toBeVisible();
}

async function seedFinishedHistory(page: import("@playwright/test").Page, id: string): Promise<void> {
  await page.goto("/");
  await seedLocalSavedMatch(page, {
    displayName: "Public editor fixture",
    id,
    moves: FINISHED_MATCH_MOVES,
    preferredVariant: "freestyle",
    savedAt: new Date().toISOString(),
    status: "black_won",
    variant: "freestyle",
  });
}

test("C20: owner edits a public title and opening without changing copied replay links", async ({ page, browser }, testInfo) => {
  const matchId = fixtureId("public-edit-source");
  const email = `${fixtureId("public-edit-owner")}@example.test`;
  const publicName = "Replay study examples";
  const originalTitle = "White's first defense";
  const updatedTitle = "Replay from the start";
  const capture = testInfo.project.name.includes("chromium");

  await seedFinishedHistory(page, matchId);
  const uid = await signIn(page, email);
  await readyCloud(page);
  await expect.poll(async () => {
    const matchHistory = (await readFirestoreDocument(`profiles/${uid}`))?.match_history;
    const matches = matchHistory && typeof matchHistory === "object"
      ? (matchHistory as { replay_matches?: Array<{ id?: string }> }).replay_matches ?? []
      : [];
    return matches.some((match) => match.id === matchId);
  }).toBe(true);
  await makeProfileEligible(uid);
  await page.reload();
  await readyCloud(page);

  await page.goto("/profile?tab=public");
  await readyCloud(page);
  await expect(page.getByRole("button", { name: "Reload public profile", exact: true })).toBeEnabled();
  await page.getByLabel("Choose your public name", { exact: true }).fill(publicName);
  await page.getByRole("button", { name: "Enable public profile", exact: true }).click();
  await page.getByRole("button", { name: "Confirm enable", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: "Public profile enabled with an empty catalog" })).toBeVisible();

  await page.goto(`/replay/${matchId}`);
  await expect(page.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
  const timeline = page.getByRole("slider", { name: "Replay timeline", exact: true });
  await timeline.press("Home");
  for (let index = 0; index < 4; index += 1) await timeline.press("ArrowRight");
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 4 / 9");
  await page.getByRole("button", { name: "Publish replay", exact: true }).click();
  await page.getByRole("textbox", { name: "Public replay title", exact: true }).fill(originalTitle);
  await page.getByRole("button", { name: "Publish this replay", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: `Published “${originalTitle}”` })).toBeVisible();

  const visitorContext = await browser.newContext({ viewport: { width: 390, height: 844 } });
  await protectContext(visitorContext);
  try {
    const visitor = await visitorContext.newPage();
    await visitor.goto(`/player/?id=${encodeURIComponent(uid)}`);
    await expect(visitor.getByRole("heading", { name: publicName, exact: true })).toBeVisible();
    await expect(visitor.getByText(originalTitle, { exact: true })).toBeVisible();
    if (capture) {
      await expect(visitor.getByTestId("replay-thumbnail").locator("canvas")).toHaveAttribute("data-ready", "true");
      await visitor.evaluate(() => document.fonts.ready);
      await visitor.screenshot({ path: testInfo.outputPath("public-replay-before-edit.png") });
    }
    const oldCopiedUrl = await visitor.getByRole("link", { name: `Replay ${originalTitle}`, exact: true }).getAttribute("href");
    expect(oldCopiedUrl).toMatch(/^\/replay\/shared\/\?fromUid=.+&fromPage=1#g2r1=/);

    const openOldCopy = async () => {
      const shared = await visitorContext.newPage();
      try {
        await shared.goto(oldCopiedUrl!);
        await expect(shared.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
        await expect(shared.getByTestId("replay-move-count")).toHaveText("Move 4 / 9");
        await expect(shared.getByTestId("replay-rule")).toHaveText("Freestyle");
      } finally {
        await shared.close();
      }
    };
    await openOldCopy();

    await page.goto("/profile?tab=public");
    await expect(page.getByRole("button", { name: `Edit ${originalTitle}`, exact: true })).toBeVisible();
    await page.getByRole("button", { name: `Edit ${originalTitle}`, exact: true }).click();
    await expect(page).toHaveTitle("Replay Analysis | Gomoku2D");
    const editor = page.getByRole("region", { name: "Edit publication", exact: true });
    await expect(editor).toBeVisible();
    await expect(page.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
    await expect(editor.getByTestId("replay-timeline")).toBeVisible();
    await expect(editor.getByTestId("replay-details-opening")).toHaveText("Move 4 / 9");

    const editUrl = new URL(page.url());
    expect(editUrl.pathname).toMatch(/^\/replay\/published-edit-[A-Za-z0-9_-]{43}$/);
    const fingerprint = editUrl.pathname.slice("/replay/published-edit-".length);
    const pageId = Number(editUrl.searchParams.get("page"));
    const rootPath = `public_profiles/${uid}`;
    const catalogPath = `${rootPath}/pages/${pageId}`;
    const rootBefore = await readFirestoreDocument(rootPath);
    const catalogBefore = await readFirestoreDocument(catalogPath);
    expect(rootBefore?.enabled).toBe(true);
    expect(catalogBefore?.entries).toHaveLength(1);
    const originalEntry = (catalogBefore?.entries as Array<Record<string, unknown>>).find((entry) => entry.fingerprint === fingerprint);
    expect(originalEntry).toMatchObject({ title: originalTitle, view_ply: 4 });

    await editor.getByRole("textbox", { name: "Public title", exact: true }).fill(updatedTitle);
    if (capture) {
      await page.evaluate(() => document.fonts.ready);
      await page.screenshot({ path: testInfo.outputPath("public-replay-editor.png") });
    }
    await editor.getByRole("slider", { name: "Replay timeline", exact: true }).press("Home");
    await editor.getByRole("button", { name: "Use move 0", exact: true }).click();
    await expect(editor.getByTestId("replay-details-opening")).toHaveText("Move 0 / 9");
    await editor.getByRole("button", { name: "Save public changes", exact: true }).click();
    await expect(page).toHaveURL(/\/profile\?tab=public$/);
    await expect(page.getByRole("button", { name: `Edit ${updatedTitle}`, exact: true })).toBeFocused();

    const rootAfter = await readFirestoreDocument(rootPath);
    const catalogAfter = await readFirestoreDocument(catalogPath);
    const updatedEntry = (catalogAfter?.entries as Array<Record<string, unknown>>).find((entry) => entry.fingerprint === fingerprint);
    expect(rootAfter?.revision).toBe((rootBefore?.revision as number) + 1);
    expect(rootAfter?.generation).toBe(rootBefore?.generation);
    expect(rootAfter?.catalog_page_counts).toEqual(rootBefore?.catalog_page_counts);
    expect(rootAfter?.shared_stats).toEqual(rootBefore?.shared_stats);
    expect(catalogAfter?.revision).toBe((catalogBefore?.revision as number) + 1);
    expect(catalogAfter?.generation).toBe(catalogBefore?.generation);
    expect(catalogAfter?.entries).toHaveLength(1);
    expect(updatedEntry).toMatchObject({ fingerprint, title: updatedTitle, view_ply: 0 });
    const updatedWire = JSON.parse(Buffer.from(String(updatedEntry?.payload), "base64url").toString("utf8")) as Record<string, unknown>;
    expect(updatedWire).toMatchObject({ at: 0, r: "freestyle" });

    await visitor.goto(`/player/?id=${encodeURIComponent(uid)}`);
    await expect(visitor.getByText(updatedTitle, { exact: true })).toBeVisible();
    const thumbnail = visitor.getByTestId("replay-thumbnail");
    await expect(thumbnail).toHaveAttribute("data-opening", "0");
    await expect(thumbnail.locator("canvas")).toHaveAttribute("data-ready", "true");
    await expect(visitor.getByText(/Rule:\s*Freestyle/)).toBeVisible();
    if (capture) {
      await visitor.evaluate(() => document.fonts.ready);
      await visitor.screenshot({ path: testInfo.outputPath("public-replay-after-edit.png") });
    }
    const currentLink = visitor.getByRole("link", { name: `Replay ${updatedTitle}`, exact: true });
    const currentUrl = await currentLink.getAttribute("href");
    const editedReplay = await visitorContext.newPage();
    try {
      await editedReplay.goto(currentUrl!);
      await expect(editedReplay.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
      await expect(editedReplay.getByTestId("replay-move-count")).toHaveText("Move 0 / 9");
      await expect(editedReplay.getByTestId("replay-rule")).toHaveText("Freestyle");
    } finally {
      await editedReplay.close();
    }

    await openOldCopy();
  } finally {
    await visitorContext.close();
  }
});
