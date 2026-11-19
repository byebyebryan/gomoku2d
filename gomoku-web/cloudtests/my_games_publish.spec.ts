import type { Page } from "@playwright/test";

import { fixtureId, makeProfileEligible, projectId, readProfile, signIn, test, expect, cloudTestOrigins } from "./helpers.js";
import { seedLocalSavedMatch } from "../playtests/helpers/local_history.js";

const FINISHED_MATCH_MOVES = [112, 0, 113, 15, 114, 30, 115, 1, 116].map((cell) => ({
  row: Math.floor(cell / 15),
  col: cell % 15,
}));

const VIEWPORTS = [
  { width: 1360, height: 1050, fontSize: "100%" },
  { width: 844, height: 390, fontSize: "100%" },
  { width: 390, height: 844, fontSize: "100%" },
  { width: 320, height: 900, fontSize: "200%" },
] as const;

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

async function readPublishedEntries(uid: string): Promise<Array<Record<string, unknown>>> {
  const pages = await Promise.all([0, 1, 2, 3].map((page) => readFirestoreDocument(`public_profiles/${uid}/pages/${page}`)));
  return pages.flatMap((page) => (page?.entries as Array<Record<string, unknown>> | undefined) ?? []);
}

async function seedFinishedHistory(page: Page, id: string): Promise<void> {
  await page.goto("/");
  await seedLocalSavedMatch(page, {
    displayName: "My games publisher",
    id,
    moves: FINISHED_MATCH_MOVES,
    preferredVariant: "freestyle",
    savedAt: new Date().toISOString(),
    status: "black_won",
    variant: "freestyle",
  });
}

async function addOlderFinishedHistory(page: Page, id: string): Promise<void> {
  await page.evaluate((olderId) => {
    const storageKey = "gomoku2d.local-profile.v5";
    const stored = JSON.parse(localStorage.getItem(storageKey) ?? "{\"state\":{},\"version\":0}");
    const history = stored.state?.matchHistory;
    const latest = history?.replayMatches?.[0];
    if (!latest) throw new Error("The newer local match fixture is missing.");
    const older = {
      ...latest,
      id: olderId,
      saved_at: new Date(new Date(latest.saved_at).getTime() - 24 * 60 * 60 * 1_000).toISOString(),
    };
    history.replayMatches.push(older);
    localStorage.setItem(storageKey, JSON.stringify(stored));
  }, id);
}

async function checkMyGamesViewport(page: Page, matchId: string, viewport: (typeof VIEWPORTS)[number]): Promise<void> {
  await page.setViewportSize({ width: viewport.width, height: viewport.height });
  await page.evaluate((fontSize) => { document.documentElement.style.fontSize = fontSize; }, viewport.fontSize);
  await page.evaluate(() => document.fonts.ready);
  const row = page.locator(`[data-match-id="${matchId}"]`);
  const inspect = row.getByRole("button", { name: /^Inspect / });
  const publish = row.getByRole("button", { name: /^Publish / });
  await expect(inspect).toBeVisible();
  await expect(publish).toBeVisible();
  await expect(inspect.locator(".uiActionLabel")).toHaveText("Inspect");
  await expect(inspect.locator(".uiActionLabel")).toBeVisible();
  await expect(publish.locator(".uiActionLabel")).toHaveText("Publish");
  await expect(publish.locator(".uiActionLabel")).toBeVisible();

  const metrics = await page.evaluate((id) => {
    const row = document.querySelector<HTMLElement>(`[data-match-id="${CSS.escape(id)}"]`);
    const layout = document.querySelector<HTMLElement>("main [class*='layout']");
    const game = row?.querySelector<HTMLElement>('[class*="historyGame"]:not([class*="historyGameTop"])');
    const inspect = row?.querySelector<HTMLElement>('button[aria-label^="Inspect "]');
    const publish = row?.querySelector<HTMLElement>('button[aria-label^="Publish "]');
    const inspectLabel = row?.querySelector<HTMLElement>('button[aria-label^="Inspect "] .uiActionLabel');
    const publishLabel = row?.querySelector<HTMLElement>('button[aria-label^="Publish "] .uiActionLabel');
    const box = (element: HTMLElement | null | undefined) => {
      if (!element) return null;
      const rect = element.getBoundingClientRect();
      return { left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom };
    };
    return {
      documentClientWidth: document.documentElement.clientWidth,
      documentScrollWidth: document.documentElement.scrollWidth,
      layoutClientWidth: layout?.clientWidth ?? 0,
      layoutScrollWidth: layout?.scrollWidth ?? 0,
      rowClientWidth: row?.clientWidth ?? 0,
      rowScrollWidth: row?.scrollWidth ?? 0,
      game: box(game),
      inspect: box(inspect),
      publish: box(publish),
      inspectLabel: box(inspectLabel),
      publishLabel: box(publishLabel),
    };
  }, matchId);
  expect(metrics.documentScrollWidth).toBeLessThanOrEqual(metrics.documentClientWidth + 1);
  expect(metrics.layoutScrollWidth).toBeLessThanOrEqual(metrics.layoutClientWidth + 1);
  expect(metrics.rowScrollWidth).toBeLessThanOrEqual(metrics.rowClientWidth + 1);
  expect(metrics.game).not.toBeNull();
  expect(metrics.inspect).not.toBeNull();
  expect(metrics.publish).not.toBeNull();
  expect(metrics.inspectLabel).not.toBeNull();
  expect(metrics.publishLabel).not.toBeNull();
  expect(metrics.inspectLabel!.left).toBeGreaterThanOrEqual(metrics.inspect!.left - 1);
  expect(metrics.inspectLabel!.right).toBeLessThanOrEqual(metrics.inspect!.right + 1);
  expect(metrics.publishLabel!.left).toBeGreaterThanOrEqual(metrics.publish!.left - 1);
  expect(metrics.publishLabel!.right).toBeLessThanOrEqual(metrics.publish!.right + 1);
  if (viewport.width === 844) {
    const gameCenter = (metrics.game!.top + metrics.game!.bottom) / 2;
    const actionCenter = (metrics.publish!.top + metrics.publish!.bottom) / 2;
    expect(Math.abs(gameCenter - actionCenter)).toBeLessThan(20);
  }
}

async function checkPublishDraftViewport(page: Page, viewport: (typeof VIEWPORTS)[number]): Promise<void> {
  await page.setViewportSize({ width: viewport.width, height: viewport.height });
  await page.evaluate((fontSize) => { document.documentElement.style.fontSize = fontSize; }, viewport.fontSize);
  await page.evaluate(() => document.fonts.ready);
  const panel = page.getByRole("region", { name: "Publish replay", exact: true });
  const choose = panel.getByRole("button", { name: "Choose opening", exact: true });
  const publish = panel.getByRole("button", { name: "Publish this replay", exact: true });
  await expect(panel).toBeVisible();
  await expect(choose).toBeVisible();
  await expect(publish).toBeVisible();
  // The sidebar enters with a horizontal transform that contributes to WebKit's scroll width.
  await panel.evaluate(async (element) => {
    await Promise.all(element.getAnimations().map((animation) => animation.finished));
  });

  const metrics = await page.evaluate(() => {
    const main = document.querySelector<HTMLElement>("main");
    const layout = main?.querySelector<HTMLElement>("[class*='layout']");
    const deck = main?.querySelector<HTMLElement>("aside");
    const panel = main?.querySelector<HTMLElement>('section[aria-label="Publish replay"]');
    const deckRect = deck?.getBoundingClientRect();
    const effectiveDeck = deck && deckRect && deckRect.width > 0 ? deck : panel;
    const buttons = Array.from(panel?.querySelectorAll<HTMLElement>("button") ?? []);
    const choose = buttons.find((button) => button.textContent?.includes("Choose opening"));
    const publish = buttons
      .find((button) => button.textContent?.includes("Publish this replay"));
    return {
      documentClientWidth: document.documentElement.clientWidth,
      documentScrollWidth: document.documentElement.scrollWidth,
      layoutClientWidth: layout?.clientWidth ?? 0,
      layoutScrollWidth: layout?.scrollWidth ?? 0,
      deckClientWidth: effectiveDeck?.clientWidth ?? 0,
      deckScrollWidth: effectiveDeck?.scrollWidth ?? 0,
      choose: choose ? (() => { const rect = choose.getBoundingClientRect(); return { left: rect.left, right: rect.right }; })() : null,
      publish: publish ? (() => { const rect = publish.getBoundingClientRect(); return { left: rect.left, right: rect.right }; })() : null,
      deck: effectiveDeck ? (() => { const rect = effectiveDeck.getBoundingClientRect(); return { left: rect.left, right: rect.right }; })() : null,
    };
  });
  expect(metrics.documentScrollWidth).toBeLessThanOrEqual(metrics.documentClientWidth + 1);
  expect(metrics.layoutScrollWidth).toBeLessThanOrEqual(metrics.layoutClientWidth + 1);
  expect(metrics.deckScrollWidth).toBeLessThanOrEqual(metrics.deckClientWidth + 1);
  expect(metrics.choose).not.toBeNull();
  expect(metrics.publish).not.toBeNull();
  expect(metrics.deck).not.toBeNull();
  for (const action of [metrics.choose!, metrics.publish!]) {
    expect(action.left).toBeGreaterThanOrEqual(metrics.deck!.left - 1);
    expect(action.right).toBeLessThanOrEqual(metrics.deck!.right + 1);
  }
}

test("C23: My games publication resumes after separate profile setup and duplicates open the same listing", async ({ page }) => {
  test.setTimeout(90_000);
  const matchId = fixtureId("my-games-publish");
  const newerMatchId = fixtureId("my-games-publish-newer");
  const email = `${fixtureId("my-games-publish-owner")}@example.test`;
  const publicName = "Direct publication owner";
  const title = "A finished game with a chosen opening";

  await seedFinishedHistory(page, newerMatchId);
  await addOlderFinishedHistory(page, matchId);
  const uid = await signIn(page, email);
  await expect.poll(async () => {
    const matches = (await readProfile(uid))?.match_history?.replay_matches ?? [];
    const ids = new Set(matches.map((match: { id: string }) => match.id));
    return ids.has(matchId) && ids.has(newerMatchId);
  }).toBe(true);
  await makeProfileEligible(uid);
  await page.reload();
  await expect(page.getByRole("button", { name: "Sign out", exact: true })).toBeVisible();

  await page.goto("/profile");
  const row = page.getByRole("listitem").filter({ has: page.locator(`#my-games-publish-${matchId}`) });
  const publishFromGames = page.locator(`#my-games-publish-${matchId}`);
  const publishHeading = page.getByRole("heading", { name: "Publish replay", exact: true });
  const inspect = row.getByRole("button", { name: /^Inspect / });
  await expect(row).toHaveCount(1);
  const newerRow = page.locator(`[data-match-id="${newerMatchId}"]`);
  await expect(newerRow).toHaveCount(1);
  const listedMatchIds = await page.locator("main ol [data-match-id]").evaluateAll((elements) =>
    elements.map((element) => element.getAttribute("data-match-id")));
  expect(listedMatchIds.slice(0, 2)).toEqual([newerMatchId, matchId]);
  await expect(publishFromGames).toBeVisible();
  const publishName = await publishFromGames.getAttribute("aria-label");
  const inspectName = await inspect.getAttribute("aria-label");
  expect(publishName).toMatch(/^Publish\b/);
  expect(inspectName).toMatch(/^Inspect\b/);
  for (const accessibleName of [publishName, inspectName]) {
    expect(accessibleName).toContain("Normal Bot");
    expect(accessibleName).toMatch(/\d{1,2}\/\d{1,2}\/\d{2}/);
    expect(accessibleName).toMatch(/\d{1,2}:\d{2}/);
  }
  for (const viewport of VIEWPORTS) {
    await checkMyGamesViewport(page, viewport.width === 844 ? newerMatchId : matchId, viewport);
  }
  await page.setViewportSize({ width: 1360, height: 1050 });
  await page.evaluate(() => { document.documentElement.style.fontSize = "100%"; });

  // Focusing the revealed row action keeps the direct path keyboard reachable.
  await publishFromGames.focus();
  await expect(publishFromGames).toBeInViewport();
  await publishFromGames.press("Enter");
  await expect(page).toHaveTitle("Replay Analysis | Gomoku2D");
  await expect(publishHeading).toBeFocused();
  await expect(publishHeading).toBeInViewport();
  await expect(page.getByRole("button", { name: "Back to My games", exact: true })).toBeVisible();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 9 / 9");
  const titleInput = page.getByRole("textbox", { name: "Public replay title", exact: true });
  await expect(page.getByRole("button", { name: "Manage public profile", exact: true })).toBeVisible();
  expect(await readFirestoreDocument(`public_profiles/${uid}`)).toBeNull();

  // The direct sidebar can return to the original My games list without a write.
  await page.getByRole("button", { name: "Back to My games", exact: true }).click();
  await expect(page).toHaveURL(/\/profile(?:\?tab=games)?$/);
  await expect(page.getByRole("tab", { name: "My games", exact: true })).toHaveAttribute("aria-selected", "true");
  await expect(publishFromGames).toBeVisible();
  await row.getByRole("button", { name: /^Inspect / }).click();
  await expect(page).toHaveURL(new RegExp(`/replay/${matchId}`));
  await expect(page.getByRole("button", { name: "Publish replay", exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Publish replay", exact: true }).click();
  await expect(page.getByRole("button", { name: "Back to replay", exact: true })).toBeVisible();
  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByRole("button", { name: "Manage public profile", exact: true }).press("Enter");

  const publicTab = page.getByRole("tab", { name: "Public profile", exact: true });
  const publicNameInput = page.getByLabel("Choose your public name", { exact: true });
  const profileHeading = page.getByRole("heading", { name: "Public profile", exact: true });
  await expect(publicNameInput).toBeVisible();
  await expect(publicTab).toHaveAttribute("aria-selected", "true");
  await expect(profileHeading).toBeFocused();
  await expect(profileHeading).toBeInViewport();
  await publicNameInput.fill(publicName);
  await page.getByRole("button", { name: "Enable public profile", exact: true }).click();
  await page.getByRole("button", { name: "Confirm enable", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: "Public profile enabled with an empty catalog" })).toBeVisible();
  const enabledRoot = await readFirestoreDocument(`public_profiles/${uid}`);
  expect(enabledRoot).toMatchObject({ enabled: true, display_name: publicName, catalog_page_counts: [0, 0, 0, 0] });
  expect(await readPublishedEntries(uid)).toEqual([]);
  await expect(page.getByRole("button", { name: "Back to replay", exact: true })).toBeVisible();

  await page.getByRole("button", { name: "Continue publishing this game", exact: true }).click();
  await expect(page).toHaveURL(new RegExp(`/replay/${matchId}`));
  await expect(publishHeading).toBeFocused();
  await expect(page.getByRole("button", { name: "Back to replay", exact: true })).toBeVisible();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 9 / 9");
  await expect(titleInput).toHaveValue("Replay after move 9");
  for (const viewport of VIEWPORTS) await checkPublishDraftViewport(page, viewport);
  await page.setViewportSize({ width: 1360, height: 1050 });
  await page.evaluate(() => { document.documentElement.style.fontSize = "100%"; });
  await titleInput.fill(title);

  await page.getByRole("button", { name: "Choose opening", exact: true }).click();
  const timeline = page.getByRole("slider", { name: "Replay timeline", exact: true });
  await expect(timeline).toBeVisible();
  await timeline.press("Home");
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 0 / 9");
  await page.evaluate(() => window.history.back());
  await expect(page.getByRole("heading", { name: "Choose opening", exact: true })).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Publish replay", exact: true })).toBeVisible();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 9 / 9");
  await expect(titleInput).toHaveValue(title);
  // The board remains keyboard-accessible while the portrait picker is open.
  await page.setViewportSize({ width: 390, height: 844 });
  const chooseOpening = page.getByRole("button", { name: "Choose opening", exact: true });
  await chooseOpening.click();
  await expect(timeline).toBeFocused();
  await timeline.press("Home");
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 0 / 9");
  const useOpening = page.getByRole("button", { name: "Use move 0", exact: true });
  const steps = page.getByRole("region", { name: "Publish replay", exact: true }).getByTestId("replay-step-controls");
  const stepBounds = await steps.boundingBox();
  const useBounds = await useOpening.boundingBox();
  expect(useBounds!.y).toBeGreaterThanOrEqual(stepBounds!.y + stepBounds!.height);
  const boardCell = page.getByRole("grid", { name: "Gomoku board" }).locator('button[tabindex="0"]');
  await boardCell.focus();
  await expect(boardCell).toBeFocused();
  await boardCell.press("Escape");
  await expect(chooseOpening).toBeFocused();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 9 / 9");
  await expect(titleInput).toHaveValue(title);
  await chooseOpening.click();
  await timeline.press("Home");
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 0 / 9");
  await useOpening.click();
  await expect(chooseOpening).toBeFocused();
  await expect(titleInput).toHaveValue(title);
  await expect(page.getByRole("group", { name: "Publication preview", exact: true }).getByText("Move 0 / 9", { exact: true })).toBeVisible();

  // Opening selection is review only; the public catalog changes only on the explicit final action.
  const profileBeforePublish = await readFirestoreDocument(`public_profiles/${uid}`);
  expect(profileBeforePublish?.revision).toBe(enabledRoot?.revision);
  expect(await readPublishedEntries(uid)).toEqual([]);
  await page.getByRole("button", { name: "Publish this replay", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: `Published “${title}” to ${publicName}'s public profile.` })).toBeVisible();
  const firstListing = await readPublishedEntries(uid);
  expect(firstListing).toHaveLength(1);
  expect(firstListing[0]).toMatchObject({ title, view_ply: 0 });

  const firstViewListing = page.getByRole("link", { name: "View published games", exact: true });
  await expect(firstViewListing).toBeFocused();
  await firstViewListing.click();
  await expect(page).toHaveURL(/\/profile\?tab=public$/);
  const editListing = page.getByRole("button", { name: `Edit ${title}`, exact: true });
  await expect(editListing).toBeFocused();

  // Inspect still opens the replay route, where publishing the same opening is recognized as this listing.
  await page.getByRole("tab", { name: "My games", exact: true }).click();
  const gameRow = page.getByRole("listitem").filter({ has: page.locator(`#my-games-publish-${matchId}`) });
  await gameRow.getByRole("button", { name: /^Inspect / }).click();
  await expect(page).toHaveURL(new RegExp(`/replay/${matchId}`));
  await expect(page.getByRole("button", { name: "Publish replay", exact: true })).toBeVisible();
  await page.getByRole("slider", { name: "Replay timeline", exact: true }).press("Home");
  await page.getByRole("button", { name: "Publish replay", exact: true }).click();
  await expect(page.getByRole("button", { name: "Back to replay", exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Publish this replay", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: "This game is already published on your public profile." })).toBeVisible();
  const duplicateViewListing = page.getByRole("link", { name: "View published games", exact: true });
  await expect(duplicateViewListing).toBeFocused();
  await duplicateViewListing.click();
  await expect(page.getByRole("button", { name: `Edit ${title}`, exact: true })).toBeFocused();
  expect(await readPublishedEntries(uid)).toHaveLength(1);
});
