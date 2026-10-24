import { mkdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import type { Page, TestInfo } from "@playwright/test";
import axe from "axe-core";

import { fixtureId, makeProfileEligible, projectId, protectContext, readProfile, signIn, test, expect, cloudTestOrigins, cloudTestPorts } from "./helpers.js";
import { seedLocalSavedMatch } from "../playtests/helpers/local_history.js";

const WIN_MOVES = [110, 0, 111, 1, 112, 2, 113, 3, 114];
const EVIDENCE_DIRECTORY = resolve(dirname(fileURLToPath(import.meta.url)), "../../docs/working/v0_6_1_evidence");
const FINISHED_MATCH_MOVES = [112, 0, 113, 15, 114, 30, 115, 1, 116].map((cell) => ({
  row: Math.floor(cell / 15),
  col: cell % 15,
}));

function sharedReplayUrl(blackName = "Alice", whiteName = "Bob", moves: number[] = WIN_MOVES): string {
  const wire = {
    v: 1,
    r: "freestyle",
    m: Buffer.from(moves).toString("base64url"),
    b: { k: "h", n: blackName },
    w: { k: "h", n: whiteName },
    at: 4,
  };
  return `/replay/shared/#g2r1=${Buffer.from(JSON.stringify(wire)).toString("base64url")}`;
}

function watchFirestoreRequests(page: Page): string[] {
  const requests: string[] = [];
  page.on("request", (request) => {
    const url = new URL(request.url());
    if (url.port === String(cloudTestPorts.firestore) || /firestore\.googleapis\.com|google\.firestore/.test(url.href)) {
      requests.push(`${request.method()} ${url.pathname}`);
    }
  });
  return requests;
}

function decodeRestValue(value: unknown): unknown {
  if (!value || typeof value !== "object") return undefined;
  const field = value as Record<string, unknown>;
  if ("nullValue" in field) return null;
  if ("timestampValue" in field) return field.timestampValue;
  if ("booleanValue" in field) return field.booleanValue;
  if ("stringValue" in field) return field.stringValue;
  if ("integerValue" in field) return Number(field.integerValue);
  if ("doubleValue" in field) return field.doubleValue;
  if ("mapValue" in field) {
    return decodeRestFields((field.mapValue as { fields?: Record<string, unknown> }).fields ?? {});
  }
  if ("arrayValue" in field) {
    const values = (field.arrayValue as { values?: unknown[] }).values ?? [];
    return values.map(decodeRestValue);
  }
  return undefined;
}

function decodeRestFields(fields: Record<string, unknown>): Record<string, unknown> {
  return Object.fromEntries(Object.entries(fields).map(([key, value]) => [key, decodeRestValue(value)]));
}

function encodeRestValue(value: unknown): unknown {
  if (value === null) return { nullValue: null };
  if (typeof value === "string") return { stringValue: value };
  if (typeof value === "boolean") return { booleanValue: value };
  if (typeof value === "number" && Number.isInteger(value)) return { integerValue: String(value) };
  if (Array.isArray(value)) return { arrayValue: { values: value.map(encodeRestValue) } };
  if (value && typeof value === "object") {
    if ("__timestampValue" in value && typeof (value as { __timestampValue?: unknown }).__timestampValue === "string") {
      return { timestampValue: (value as { __timestampValue: string }).__timestampValue };
    }
    return { mapValue: { fields: encodeRestFields(value as Record<string, unknown>) } };
  }
  throw new Error("Unsupported Firestore fixture value.");
}

function firestoreTimestamp(value: string): { __timestampValue: string } {
  return { __timestampValue: value };
}

function encodeRestFields(fields: Record<string, unknown>): Record<string, unknown> {
  return Object.fromEntries(Object.entries(fields).map(([key, value]) => [key, encodeRestValue(value)]));
}

async function readFirestoreDocument(path: string): Promise<Record<string, unknown> | null> {
  const encodedPath = path.split("/").map(encodeURIComponent).join("/");
  const response = await fetch(`${cloudTestOrigins.firestore}/v1/projects/${projectId}/databases/(default)/documents/${encodedPath}`, {
    headers: { Authorization: "Bearer owner" },
  });
  if (response.status === 404) return null;
  if (!response.ok) throw new Error(`Firestore REST read failed for ${path}: ${await response.text()}`);
  const document = await response.json() as { fields?: Record<string, unknown> };
  return decodeRestFields(document.fields ?? {});
}

async function seedFirestoreDocument(path: string, fields: Record<string, unknown>): Promise<void> {
  const encodedPath = path.split("/").map(encodeURIComponent).join("/");
  const response = await fetch(`${cloudTestOrigins.firestore}/v1/projects/${projectId}/databases/(default)/documents/${encodedPath}`, {
    method: "PATCH",
    headers: { Authorization: "Bearer owner", "Content-Type": "application/json" },
    body: JSON.stringify({ fields: encodeRestFields(fields) }),
  });
  if (!response.ok) throw new Error(`Firestore REST seed failed for ${path}: ${await response.text()}`);
}

async function auditAccessibility(page: Page, testInfo: TestInfo, name: string): Promise<void> {
  await page.evaluate(() => document.fonts.ready);
  await page.addScriptTag({ content: axe.source });
  const result = await page.evaluate(async () => {
    const report = await axe.run(document, {
      runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "best-practice"] },
    });
    return { testEngine: report.testEngine, url: report.url, violations: report.violations, incomplete: report.incomplete };
  });
  const auditPath = testInfo.outputPath(`sharing-accessibility-${name}.json`);
  await mkdir(dirname(auditPath), { recursive: true });
  await writeFile(auditPath, JSON.stringify(result, null, 2));
  await testInfo.attach(`sharing-accessibility-${name}`, {
    path: auditPath,
    contentType: "application/json",
  });
  expect(result.violations).toEqual([]);
}

async function readyCloud(page: Page): Promise<void> {
  // The cloud-status paragraph is intentionally hidden in the mobile layout;
  // sign-in visibility is the stable auth-ready signal across breakpoints.
  await expect(page.getByRole("button", { name: "Sign out", exact: true })).toBeVisible();
}

async function prepareEligibleProfile(page: Page, uid: string): Promise<void> {
  await expect.poll(async () => readProfile(uid)).not.toBeNull();
  await makeProfileEligible(uid);
  await page.reload();
  await readyCloud(page);
}

async function seedFinishedHistory(page: Page, id: string): Promise<void> {
  await page.goto("/");
  await seedLocalSavedMatch(page, {
    displayName: "Cloud fixture",
    id,
    moves: FINISHED_MATCH_MOVES,
    preferredVariant: "freestyle",
    savedAt: new Date().toISOString(),
    status: "black_won",
    variant: "freestyle",
  });
}

async function openSavedReplays(page: Page): Promise<void> {
  await page.getByRole("tab", { name: "Saved replays", exact: true }).click();
  await expect(page.getByRole("button", { name: "Reload cloud library", exact: true })).toBeEnabled();
}

async function openPublicProfile(page: Page): Promise<void> {
  await page.getByRole("tab", { name: "Public profile", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Public profile", exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Reload public profile", exact: true })).toBeEnabled();
}

async function expectNoHorizontalOverflow(page: Page): Promise<void> {
  const widths = await page.evaluate(() => ({
    viewport: window.innerWidth,
    document: document.documentElement.scrollWidth,
    mainClientWidth: document.querySelector("main")?.clientWidth ?? 0,
    mainScrollWidth: document.querySelector("main")?.scrollWidth ?? 0,
  }));
  expect(widths.document).toBeLessThanOrEqual(widths.viewport);
  expect(widths.mainScrollWidth - widths.mainClientWidth).toBeLessThanOrEqual(2);
}

async function captureSharingEvidence(page: Page, testInfo: TestInfo, name: string, selector: string): Promise<void> {
  const project = testInfo.project.name.replace(/[^A-Za-z0-9_-]+/g, "-");
  const filename = `sharing-${project}-${name}.png`;
  const writeRepositoryEvidence = process.env.GOMOKU_WRITE_EVIDENCE === "1";
  const screenshotPath = writeRepositoryEvidence
    ? resolve(EVIDENCE_DIRECTORY, filename)
    : testInfo.outputPath(filename);
  if (writeRepositoryEvidence) await mkdir(EVIDENCE_DIRECTORY, { recursive: true });
  await page.locator(selector).screenshot({
    path: screenshotPath,
  });
  await testInfo.attach(`sharing-${name}`, {
    path: screenshotPath,
    contentType: "image/png",
  });
}

async function navigateWithinApp(page: Page, path: string): Promise<void> {
  // Keep the in-memory library revision loaded so this can model a stale tab.
  await page.evaluate((nextPath) => {
    window.history.pushState({}, "", nextPath);
    window.dispatchEvent(new PopStateEvent("popstate"));
  }, path);
}

test("C10: fresh signed-in shared/public route variants do no private Firestore work or history promotion", async ({ page, context }, testInfo) => {
  const email = `${fixtureId("shared-auth")}@example.test`;
  const historyId = fixtureId("local-only-history");
  const uid = await signIn(page, email);
  await readyCloud(page);
  await prepareEligibleProfile(page, uid);
  const baseline = await readProfile(uid);
  expect(baseline).not.toBeNull();
  await openPublicProfile(page);
  await page.getByLabel("Choose your public name", { exact: true }).fill("Network-isolated public player");
  await page.getByRole("checkbox", { name: /I want to enable this public profile/ }).check();
  await page.getByRole("button", { name: "Enable public profile", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: "Public profile enabled with an empty catalog" })).toBeVisible();

  // Seed after sign-in so the source page's already-running cloud session never
  // sees it; the new page hydrates this unsynced history from local storage.
  await seedLocalSavedMatch(page, {
    displayName: "Pending local history",
    id: historyId,
    moves: FINISHED_MATCH_MOVES,
    preferredVariant: "freestyle",
    savedAt: new Date().toISOString(),
    status: "black_won",
    variant: "freestyle",
  });
  await page.close();

  const fragment = sharedReplayUrl().slice(sharedReplayUrl().indexOf("#"));
  const paths = ["/replay/shared/", "/Replay/Shared/", "/replay/shared//", "/replay/%73hared/"];
  const publicPaths = ["/player/", "/PLAYER/", "/player//", "/%70layer/"];
  const observations = [];
  for (const path of [...paths, ...publicPaths]) {
    const isPublic = publicPaths.includes(path);
    const freshPage = await context.newPage();
    const firestoreRequests = watchFirestoreRequests(freshPage);
    const targets: string[] = [];
    freshPage.on("request", (request) => {
      if (!request.postData()) return;
      let body = request.postData()!;
      try { body = decodeURIComponent(body); } catch { /* JSON bodies can contain literal percent signs. */ }
      if (body.includes("/documents/")) targets.push(body);
    });
    try {
      await freshPage.goto(isPublic ? `${path}?id=${encodeURIComponent(uid)}` : `${path}${fragment}`);
      if (isPublic) {
        await expect(freshPage.getByRole("heading", { name: "Network-isolated public player", exact: true })).toBeVisible();
      } else {
        await expect(freshPage).toHaveTitle("Replay Analysis | Gomoku2D");
        await expect(freshPage.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
      }
      // Allow persisted auth hydration and any unintended private session to settle.
      await freshPage.waitForTimeout(1_000);
      const privateTargets = targets.filter((body) => /\/documents\/(?:profiles|replay_libraries)\//.test(body));
      expect(privateTargets).toEqual([]);
      if (isPublic) expect(targets.some((body) => body.includes("/documents/public_profiles/"))).toBe(true);
      else expect(firestoreRequests).toEqual([]);
      observations.push({ path, firestoreRequests: firestoreRequests.length, privateTargets: privateTargets.length });
    } finally {
      await freshPage.close();
    }
  }
  await testInfo.attach("signed-in-route-network-boundaries", {
    body: JSON.stringify(observations, null, 2), contentType: "application/json",
  });
  const after = await readProfile(uid);
  expect(after?.match_history?.replay_matches).toEqual(baseline?.match_history?.replay_matches);
  expect(after?.match_history?.replay_matches?.some((match: { id: string }) => match.id === historyId)).toBe(false);
  expect(await readFirestoreDocument(`replay_libraries/${uid}`)).toBeNull();
});

test("C11: public opt-in, visitor replay, unpublish, disable, re-enable, and copied-link behavior", async ({ page, browser }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const email = `${fixtureId("public-owner")}@example.test`;
  const matchId = fixtureId("publishable-match");
  const title = fixtureId("Public replay");
  const publicName = `Public ${fixtureId("player")}`;
  await seedFinishedHistory(page, matchId);
  const uid = await signIn(page, email);
  await readyCloud(page);
  await expect.poll(async () => (await readProfile(uid))?.match_history?.replay_matches?.some(
    (match: { id: string }) => match.id === matchId,
  )).toBe(true);
  await prepareEligibleProfile(page, uid);

  await openPublicProfile(page);
  await page.getByLabel("Choose your public name", { exact: true }).fill(publicName);
  await page.getByRole("checkbox", { name: /I want to enable this public profile/ }).check();
  await page.getByRole("button", { name: "Enable public profile", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: "Public profile enabled with an empty catalog" })).toBeVisible();

  const publicStatsToggle = page.getByRole("group", { name: "Share My games statistics publicly", exact: true }).getByRole("button", { name: "On", exact: true });
  await expect(publicStatsToggle).toBeEnabled();
  await publicStatsToggle.click();
  await expect.poll(async () => (await readFirestoreDocument(`public_profiles/${uid}`))?.shared_stats).not.toBeNull();
  const publishedStatsRoot = await readFirestoreDocument(`public_profiles/${uid}`);
  expect(publishedStatsRoot?.shared_stats).toMatchObject({ matches: 1, wins: 1, losses: 0, draws: 0 });
  await expect(page.getByText(/Published [A-Z]/)).toBeVisible();
  const firstStatsRevision = publishedStatsRoot?.revision as number;
  await page.getByRole("button", { name: "Update public stats", exact: true }).click();
  await expect.poll(async () => (await readFirestoreDocument(`public_profiles/${uid}`))?.revision as number)
    .toBeGreaterThan(firstStatsRevision);
  await page.getByRole("group", { name: "Share My games statistics publicly", exact: true }).getByRole("button", { name: "Off", exact: true }).click();
  await expect.poll(async () => (await readFirestoreDocument(`public_profiles/${uid}`))?.shared_stats).toBeNull();
  await publicStatsToggle.click();
  await expect.poll(async () => (await readFirestoreDocument(`public_profiles/${uid}`))?.shared_stats).not.toBeNull();
  await auditAccessibility(page, testInfo, "public-manager");
  await expectNoHorizontalOverflow(page);
  await captureSharingEvidence(page, testInfo, "profile-manager", 'section[aria-labelledby="public-profile-manager-title"]');
  await page.setViewportSize({ width: 1280, height: 1800 });
  await expectNoHorizontalOverflow(page);
  await captureSharingEvidence(page, testInfo, "profile-manager-desktop", 'section[aria-labelledby="public-profile-manager-title"]');
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto(`/replay/${matchId}`);
  await expect(page).toHaveTitle("Replay Analysis | Gomoku2D");
  await page.getByRole("button", { name: "Previous move", exact: true }).click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 8 / 9");
  const canvas = await page.locator("canvas").elementHandle();
  const publishTrigger = page.getByRole("button", { name: "Publish replay", exact: true });
  await expect(publishTrigger.locator("svg")).toHaveCount(1);
  await page.getByRole("button", { name: "Auto play", exact: true }).click();
  await publishTrigger.click();
  const backToReplay = page.getByRole("button", { name: "Back to replay", exact: true });
  await expect(backToReplay).toBeFocused();
  await expect(page.getByRole("button", { name: "Next move", exact: true })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Share replay", exact: true })).toHaveCount(0);
  expect(await canvas!.evaluate((element) => element.isConnected)).toBe(true);
  await expect(page.getByRole("textbox", { name: "Public replay title", exact: true })).toBeVisible();
  // Escape restores the same paused position and the originating action.
  const openingTitle = await page.getByRole("textbox", { name: "Public replay title", exact: true }).inputValue();
  await page.keyboard.press("Escape");
  await expect(publishTrigger).toBeFocused();
  await expect(page.getByRole("button", { name: "Auto play", exact: true })).toBeVisible();
  const openingMove = Number(openingTitle.match(/\d+$/)![0]);
  await expect(page.getByTestId("replay-move-count")).toHaveText(`Move ${openingMove} / 9`);
  await publishTrigger.click();
  await page.getByRole("textbox", { name: "Public replay title", exact: true }).fill(title);
  await auditAccessibility(page, testInfo, "publish-form");
  await page.setViewportSize({ width: 1280, height: 1000 });
  await expectNoHorizontalOverflow(page);
  await captureSharingEvidence(page, testInfo, "publish-replay-desktop", 'section[aria-label="Publish replay"]');
  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByRole("button", { name: "Publish this replay", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: `Published “${title}”` })).toBeVisible();
  await expectNoHorizontalOverflow(page);
  await captureSharingEvidence(page, testInfo, "publish-replay", 'section[aria-label="Publish replay"]');

  const visitorContext = await browser.newContext({ viewport: { width: 390, height: 844 } });
  await protectContext(visitorContext);
  try {
    const visitor = await visitorContext.newPage();
    await visitor.goto(`/player/?id=${encodeURIComponent(uid)}`);
    await expect(visitor.getByRole("heading", { name: publicName, exact: true })).toBeVisible();
    await expect(visitor.getByText(title, { exact: true })).toBeVisible();
    await expect(visitor.getByText("Matches", { exact: true })).toBeVisible();
    await auditAccessibility(visitor, testInfo, "public-visitor-profile");
    await expectNoHorizontalOverflow(visitor);
    await captureSharingEvidence(visitor, testInfo, "public-profile", "main");
    const portableUrl = await visitor.getByRole("link", { name: `Replay ${title}`, exact: true }).getAttribute("href");
    expect(portableUrl).toMatch(/^\/replay\/shared\/\?fromUid=.+&fromPage=1#g2r1=/);

    const openPortableCopy = async () => {
      // A fresh page avoids attributing a delayed public-profile SDK request to
      // this portable route; it has no signed-in or private-history session.
      const sharedPage = await visitorContext.newPage();
      const firestoreRequests = watchFirestoreRequests(sharedPage);
      try {
        await sharedPage.goto(portableUrl!);
        await expect(sharedPage).toHaveTitle("Replay Analysis | Gomoku2D");
        await expect(sharedPage.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
        await expectNoHorizontalOverflow(sharedPage);
        // The shared link is a portable copy and opens without auth or history reads.
        await sharedPage.waitForTimeout(250);
        expect(firestoreRequests).toEqual([]);
      } finally {
        await sharedPage.close();
      }
    };
    await openPortableCopy();

    await page.goto("/profile");
    await openPublicProfile(page);
    await expect(page.getByText(title, { exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Unpublish", exact: true }).click();
    await expect(page.getByRole("status").filter({ hasText: "Publication removed from the public catalog." })).toBeVisible();

    await visitor.goto(`/player/?id=${encodeURIComponent(uid)}`);
    await expect(visitor.getByText("No games have been published.", { exact: true })).toBeVisible();

    await page.getByRole("button", { name: "Disable public profile", exact: true }).click();
    await page.getByRole("button", { name: "Confirm disable", exact: true }).click();
    await expect(page.getByRole("status").filter({ hasText: "Public profile disabled. Statistics and catalog publications were removed." })).toBeVisible();
    await visitor.reload();
    await expect(visitor.getByRole("heading", { name: "Public profile unavailable", exact: true })).toBeVisible();

    await page.getByRole("checkbox", { name: /I want to enable this public profile/ }).check();
    await page.getByRole("button", { name: "Enable public profile", exact: true }).click();
    await expect(page.getByRole("status").filter({ hasText: "Public profile enabled with an empty catalog" })).toBeVisible();
    await visitor.goto(`/player/?id=${encodeURIComponent(uid)}`);
    await expect(visitor.getByRole("heading", { name: publicName, exact: true })).toBeVisible();
    await expect(visitor.getByText("No games have been published.", { exact: true })).toBeVisible();

    await openPortableCopy();
  } finally {
    await visitorContext.close();
  }
});

test("C12: two clients explicitly reload and remove a saved replay from the account library", async ({ page, browser }) => {
  const email = `${fixtureId("library-sync")}@example.test`;
  const portableUrl = sharedReplayUrl("Library owner");
  await signIn(page, email);
  await page.goto(portableUrl);
  await expect(page).toHaveTitle("Replay Analysis | Gomoku2D");
  await page.getByRole("button", { name: "Save replay", exact: true }).click();
  await expect(page.getByRole("button", { name: "Saved replay", exact: true })).toBeVisible();
  await expect(page.getByText("Saved in your account library.", { exact: true })).toBeVisible();

  await page.goto("/profile");
  await openSavedReplays(page);
  await page.getByRole("button", { name: "Reload cloud library", exact: true }).click();
  await expect(page.getByText("Account library synced.", { exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "Inspect replay", exact: true })).toHaveCount(1);

  const secondContext = await browser.newContext();
  await protectContext(secondContext);
  try {
    const second = await secondContext.newPage();
    await signIn(second, email);
    await readyCloud(second);
    await openSavedReplays(second);
    await second.getByRole("button", { name: "Reload cloud library", exact: true }).click();
    await expect(second.getByText("Account library synced.", { exact: true })).toBeVisible();
    await expect(second.getByRole("link", { name: "Inspect replay", exact: true })).toHaveCount(1);
    await second.getByRole("button", { name: "Remove replay", exact: true }).click();
    await expect(second.getByText("No saved replays yet.", { exact: true })).toBeVisible();
    await expect(second.getByText("Account library synced.", { exact: true })).toBeVisible();

    await page.getByRole("button", { name: "Reload cloud library", exact: true }).click();
    await expect(page.getByText("No saved replays yet.", { exact: true })).toBeVisible();
    await expect(page.getByText("Account library synced.", { exact: true })).toBeVisible();
  } finally {
    await secondContext.close();
  }
});

test("C13: stale offline library data requires review and cannot restore another client's deletion", async ({ page, browser }) => {
  const email = `${fixtureId("library-conflict")}@example.test`;
  const firstUrl = sharedReplayUrl("First client");
  const secondUrl = sharedReplayUrl("Offline client", "Bob", [95, 1, 96, 2, 97, 3, 98, 4, 99]);
  await signIn(page, email);

  await page.goto(firstUrl);
  await page.getByRole("button", { name: "Save replay", exact: true }).click();
  await expect(page.getByRole("button", { name: "Saved replay", exact: true })).toBeVisible();
  await expect(page.getByText("Saved in your account library.", { exact: true })).toBeVisible();
  await page.goto("/profile");
  await openSavedReplays(page);
  await page.getByRole("button", { name: "Reload cloud library", exact: true }).click();
  await expect(page.getByText("Account library synced.", { exact: true })).toBeVisible();

  const secondContext = await browser.newContext();
  await protectContext(secondContext);
  try {
    const second = await secondContext.newPage();
    await signIn(second, email);
    await readyCloud(second);
    await openSavedReplays(second);
    await second.getByRole("button", { name: "Reload cloud library", exact: true }).click();
    await expect(second.getByText("Account library synced.", { exact: true })).toBeVisible();
    await expect(second.getByText("First client · Bob · 9 moves · Freestyle", { exact: true })).toBeVisible();

    await page.getByRole("button", { name: "Remove replay", exact: true }).click();
    await expect(page.getByText("No saved replays yet.", { exact: true })).toBeVisible();
    await expect(page.getByText("Account library synced.", { exact: true })).toBeVisible();

    // Stay in this browser runtime so its already-loaded base revision remains
    // stale after Client A removes the cloud copy.
    await navigateWithinApp(second, secondUrl);
    await second.getByRole("button", { name: "Save replay", exact: true }).click();
    await expect(second.getByRole("button", { name: "Saved replay", exact: true })).toBeVisible();
    await expect(second.getByText(/Saved on this device; account sync is pending/)).toBeVisible();

    await second.getByRole("link", { name: "Profile", exact: true }).click();
    await openSavedReplays(second);
    await expect(second.getByText(/Review library changes: 2 local copies; 0 cloud copies\./)).toBeVisible();
    await expect(second.getByText("First client · Bob · 9 moves · Freestyle", { exact: true })).toBeVisible();
    await expect(second.getByText("Offline client · Bob · 9 moves · Freestyle", { exact: true })).toBeVisible();

    second.on("dialog", async (dialog) => { await dialog.accept(); });
    await second.getByRole("button", { name: "Use cloud library", exact: true }).click();
    await expect(second.getByText("No saved replays yet.", { exact: true })).toBeVisible();
    await expect(second.getByText("Account library synced.", { exact: true })).toBeVisible();

    await page.getByRole("button", { name: "Reload cloud library", exact: true }).click();
    await expect(page.getByText("No saved replays yet.", { exact: true })).toBeVisible();
    await expect(page.getByText("Account library synced.", { exact: true })).toBeVisible();
  } finally {
    await secondContext.close();
  }
});

test("C14: private reset keeps public sharing, while Delete Cloud clears markers and reviews local copies", async ({ page }) => {
  const email = `${fixtureId("reset-delete-cloud")}@example.test`;
  const matchId = fixtureId("public-before-reset");
  const publicName = fixtureId("Reset-safe public name");
  const title = fixtureId("Reset-safe publication");
  const firstPrivateUrl = sharedReplayUrl("Private before reset");
  const retainedPrivateUrl = sharedReplayUrl("Local after delete", "Bob", [95, 1, 96, 2, 97, 3, 98, 4, 99]);

  await seedFinishedHistory(page, matchId);
  const uid = await signIn(page, email);
  await readyCloud(page);
  await expect.poll(async () => (await readProfile(uid))?.match_history?.replay_matches?.some(
    (match: { id: string }) => match.id === matchId,
  )).toBe(true);
  await prepareEligibleProfile(page, uid);

  await page.goto(firstPrivateUrl);
  await page.getByRole("button", { name: "Save replay", exact: true }).click();
  await expect(page.getByText("Saved in your account library.", { exact: true })).toBeVisible();
  await page.goto("/profile");
  await openSavedReplays(page);
  await expect(page.getByRole("link", { name: "Inspect replay", exact: true })).toHaveCount(1);

  await openPublicProfile(page);
  await page.getByLabel("Choose your public name", { exact: true }).fill(publicName);
  await page.getByRole("checkbox", { name: /I want to enable this public profile/ }).check();
  await page.getByRole("button", { name: "Enable public profile", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: "Public profile enabled with an empty catalog" })).toBeVisible();

  const publicStatsToggle = page.getByRole("group", { name: "Share My games statistics publicly", exact: true }).getByRole("button", { name: "On", exact: true });
  await expect(publicStatsToggle).toBeEnabled();
  await publicStatsToggle.click();
  await expect.poll(async () => (await readFirestoreDocument(`public_profiles/${uid}`))?.shared_stats).not.toBeNull();

  await page.goto(`/replay/${matchId}`);
  await page.getByRole("button", { name: "Publish replay", exact: true }).click();
  await page.getByRole("textbox", { name: "Public replay title", exact: true }).fill(title);
  await page.getByRole("button", { name: "Publish this replay", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: `Published “${title}”` })).toBeVisible();

  const publicRootBeforeReset = await readFirestoreDocument(`public_profiles/${uid}`);
  const publicPagesBeforeReset = await Promise.all([0, 1, 2, 3].map((id) =>
    readFirestoreDocument(`public_profiles/${uid}/pages/${id}`)));
  const privateBeforeReset = await readFirestoreDocument(`replay_libraries/${uid}`);
  expect(publicRootBeforeReset).toMatchObject({ enabled: true, display_name: publicName });
  expect(publicPagesBeforeReset.flatMap((page) => (page?.entries as unknown[]) ?? [])).toHaveLength(1);
  expect(privateBeforeReset).not.toBeNull();
  expect(JSON.parse(privateBeforeReset!.entries_json as string)).toHaveLength(1);

  await page.goto("/profile");
  await page.getByRole("button", { name: "Reset Profile", exact: true }).click();
  await expect(page.getByText(/Your public profile, shared statistics and publications stay unchanged/)).toBeVisible();
  await page.getByRole("button", { name: "Reset", exact: true }).click();
  await expect(page.getByRole("button", { name: "Reset Profile", exact: true })).toBeVisible();
  await expect.poll(async () => (await readProfile(uid))?.match_history?.replay_matches).toEqual([]);

  const publicRootAfterReset = await readFirestoreDocument(`public_profiles/${uid}`);
  const publicPagesAfterReset = await Promise.all([0, 1, 2, 3].map((id) =>
    readFirestoreDocument(`public_profiles/${uid}/pages/${id}`)));
  const privateAfterReset = await readFirestoreDocument(`replay_libraries/${uid}`);
  expect(publicRootAfterReset).toEqual(publicRootBeforeReset);
  expect(publicRootAfterReset?.shared_stats).toMatchObject({ matches: 1, wins: 1, losses: 0, draws: 0 });
  expect(publicPagesAfterReset).toEqual(publicPagesBeforeReset);
  expect(privateAfterReset?.entries_json).toBe("[]");
  expect(privateAfterReset?.revision).toBe((privateBeforeReset?.revision as number) + 1);

  await page.goto(retainedPrivateUrl);
  await page.getByRole("button", { name: "Save replay", exact: true }).click();
  await expect(page.getByText("Saved in your account library.", { exact: true })).toBeVisible();
  const privateBeforeDelete = await readFirestoreDocument(`replay_libraries/${uid}`);
  expect(JSON.parse(privateBeforeDelete?.entries_json as string)).toHaveLength(1);
  await page.goto("/profile");
  await page.getByRole("button", { name: "Reset Profile", exact: true }).click();
  await page.getByRole("button", { name: "Delete Cloud", exact: true }).click();
  await expect(page.getByText(/Delete your cloud profile, history, saved replays, public name, shared statistics and publications/)).toBeVisible();
  await page.getByRole("button", { name: "Delete", exact: true }).click();
  await expect(page.getByRole("button", { name: "Sign in", exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Sign out", exact: true })).toHaveCount(0);

  const deletedProfile = await readFirestoreDocument(`profiles/${uid}`);
  const publicRootAfterDelete = await readFirestoreDocument(`public_profiles/${uid}`);
  const publicPagesAfterDelete = await Promise.all([0, 1, 2, 3].map((id) =>
    readFirestoreDocument(`public_profiles/${uid}/pages/${id}`)));
  const privateAfterDelete = await readFirestoreDocument(`replay_libraries/${uid}`);
  expect(deletedProfile).toBeNull();
  expect(publicRootAfterDelete).toMatchObject({ enabled: false, display_name: "" });
  expect(publicRootAfterDelete?.shared_stats).toBeNull();
  expect(publicRootAfterDelete?.catalog_page_counts).toEqual([0, 0, 0, 0]);
  expect(publicRootAfterDelete?.generation).toBeGreaterThan(publicRootAfterReset?.generation as number);
  expect(publicPagesAfterDelete).toHaveLength(4);
  for (const pageSnapshot of publicPagesAfterDelete) {
    expect(pageSnapshot).toMatchObject({ generation: publicRootAfterDelete?.generation, entries: [] });
  }
  expect(privateAfterDelete?.entries_json).toBe("[]");
  expect(privateAfterDelete?.revision).toBeGreaterThan(privateBeforeDelete?.revision as number);

  const signedInUid = await signIn(page, email, { navigate: false });
  expect(signedInUid).toBe(uid);
  await page.reload();
  await readyCloud(page);
  await expect.poll(async () => (await readProfile(uid))?.match_history?.replay_matches).toEqual([]);
  await openSavedReplays(page);
  await expect(page.getByText("Review library changes: 1 local copies; 0 cloud copies.", { exact: true })).toBeVisible();
  await expect(page.getByText("Local after delete · Bob · 9 moves · Freestyle", { exact: true })).toBeVisible();
  const privateAfterSignin = await readFirestoreDocument(`replay_libraries/${uid}`);
  expect(privateAfterSignin?.entries_json).toBe("[]");
  expect(privateAfterSignin?.revision).toBe(privateAfterDelete?.revision);
});

test("C15: maximum Unicode public fields remain editable and reflow for owners and visitors", async ({ page, browser }, testInfo) => {
  await page.setViewportSize({ width: 320, height: 900 });
  const matchId = fixtureId("maximum-public-fields");
  const unicodeName = "💠".repeat(80);
  const publicName = "W".repeat(80);
  const title = "🎯".repeat(80);
  await seedFinishedHistory(page, matchId);
  const uid = await signIn(page, `${fixtureId("unicode-owner")}@example.test`);
  await readyCloud(page);
  await expect.poll(async () => (await readProfile(uid))?.match_history?.replay_matches?.some(
    (match: { id: string }) => match.id === matchId,
  )).toBe(true);
  await prepareEligibleProfile(page, uid);
  await openPublicProfile(page);
  const chosenName = page.getByRole("textbox", { name: "Choose your public name", exact: true });
  await chosenName.fill(unicodeName);
  await expect(chosenName).toHaveValue(unicodeName);
  await page.getByRole("checkbox", { name: /I want to enable this public profile/ }).check();
  await page.getByRole("button", { name: "Enable public profile", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: "Public profile enabled with an empty catalog" })).toBeVisible();
  await page.getByRole("textbox", { name: "Public name", exact: true }).fill(publicName);
  await page.getByRole("textbox", { name: "Public name", exact: true }).press("End");
  await page.getByRole("textbox", { name: "Public name", exact: true }).pressSequentially("Z");
  await expect(page.getByRole("textbox", { name: "Public name", exact: true })).toHaveValue(publicName);
  await page.getByRole("button", { name: "Save public name", exact: true }).click();
  await expect(page.getByText("Public name saved.", { exact: true })).toBeVisible();
  const statsToggle = page.getByRole("group", { name: "Share My games statistics publicly", exact: true }).getByRole("button", { name: "On", exact: true });
  await expect(statsToggle).toBeEnabled();
  await statsToggle.click();
  await expect(page.getByRole("button", { name: "Update public stats", exact: true })).toBeEnabled();
  await expectNoHorizontalOverflow(page);
  await captureSharingEvidence(page, testInfo, "maximum-owner", 'section[aria-labelledby="public-profile-manager-title"]');
  await page.setViewportSize({ width: 640, height: 900 });
  await page.addStyleTag({ content: "html { font-size: 200%; }" });
  await expectNoHorizontalOverflow(page);
  await auditAccessibility(page, testInfo, "maximum-owner-enlarged");
  await captureSharingEvidence(page, testInfo, "maximum-owner-enlarged", 'section[aria-labelledby="public-profile-manager-title"]');

  await page.goto(`/replay/${matchId}`);
  await page.setViewportSize({ width: 320, height: 900 });
  await page.getByRole("button", { name: "Publish replay", exact: true }).click();
  const titleInput = page.getByRole("textbox", { name: "Public replay title", exact: true });
  await titleInput.fill(title);
  await expect(titleInput).toHaveValue(title);
  await page.getByRole("button", { name: "Publish this replay", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: `Published “${title}”` })).toBeVisible();
  await expectNoHorizontalOverflow(page);
  await page.setViewportSize({ width: 640, height: 900 });
  await page.addStyleTag({ content: "html { font-size: 200%; }" });
  await expectNoHorizontalOverflow(page);
  await auditAccessibility(page, testInfo, "maximum-publish-enlarged");
  await captureSharingEvidence(page, testInfo, "maximum-publish-enlarged", 'section[aria-label="Publish replay"]');

  const visitorContext = await browser.newContext({ viewport: { width: 320, height: 900 } });
  await protectContext(visitorContext);
  try {
    const visitor = await visitorContext.newPage();
    await visitor.goto(`/player/?id=${uid}`);
    await expect(visitor.getByRole("heading", { name: publicName, exact: true })).toBeVisible();
    await expect(visitor.getByText(title, { exact: true })).toBeVisible();
    await expectNoHorizontalOverflow(visitor);
    await auditAccessibility(visitor, testInfo, "maximum-public-fields");
    await captureSharingEvidence(visitor, testInfo, "maximum-visitor", "main");
    await visitor.setViewportSize({ width: 640, height: 900 });
    await visitor.addStyleTag({ content: "html { font-size: 200%; }" });
    await expectNoHorizontalOverflow(visitor);
    await captureSharingEvidence(visitor, testInfo, "maximum-visitor-enlarged", "main");
  } finally {
    await visitorContext.close();
  }
});

test("C16: sparse catalog pages, zero stats, last-page removal and legacy roots remain navigable", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 900 });
  const uid = fixtureId("sparse-public-catalog");
  const pageCounts = [5, 0, 8, 5];
  const updatedAt = "2026-10-22T00:00:00.000Z";
  const rootFields = {
    schema_version: 1,
    uid,
    display_name: "Sparse catalog player",
    enabled: true,
    generation: 1,
    revision: 1,
    updated_at: firestoreTimestamp(updatedAt),
    shared_stats: { matches: 0, wins: 0, losses: 0, draws: 0, updated_at: firestoreTimestamp(updatedAt) },
    catalog_page_counts: pageCounts,
  };
  await seedFirestoreDocument(`public_profiles/${uid}`, rootFields);
  let lastPageEntries: Array<{ fingerprint: string; payload: string; title: string; view_ply: number }> = [];
  for (const pageId of [0, 1, 2, 3]) {
    const count = pageCounts[pageId]!;
    const entries = Array.from({ length: count }, (_, index) => ({
      fingerprint: Buffer.from(`${pageId}-${index}`).toString("base64url").padEnd(43, "x"),
      payload: "YWJj",
      title: `Storage ${pageId} game ${index + 1}`,
      view_ply: 0,
    }));
    if (pageId === 3) lastPageEntries = entries;
    await seedFirestoreDocument(`public_profiles/${uid}/pages/${pageId}`, {
      schema_version: 1,
      generation: 1,
      revision: 1,
      updated_at: firestoreTimestamp(updatedAt),
      entries,
    });
  }

  await page.goto(`/player/?id=${encodeURIComponent(uid)}`);
  await expect(page.getByRole("heading", { name: "Sparse catalog player", exact: true })).toBeVisible();
  await expect(page.getByText("Page 1 of 3")).toBeVisible();
  await expect(page.getByText("—", { exact: true })).toBeVisible();
  await expect(page.getByRole("listitem")).toHaveCount(8);
  await expect(page.getByText("Storage 0 game 5", { exact: true })).toBeVisible();
  await expect(page.getByText("Storage 2 game 3", { exact: true })).toBeVisible();
  await expectNoHorizontalOverflow(page);

  await page.getByRole("button", { name: "Next page", exact: true }).click();
  await expect(page.getByText("Page 2 of 3")).toBeVisible();
  await expect(page.getByRole("listitem")).toHaveCount(8);
  await expect(page.getByText("Storage 2 game 4", { exact: true })).toBeVisible();
  await expect(page.getByText("Storage 3 game 3", { exact: true })).toBeVisible();

  await page.getByRole("button", { name: "Next page", exact: true }).click();
  await expect(page.getByText("Page 3 of 3")).toBeVisible();
  await expect(page.getByRole("listitem")).toHaveCount(2);
  await expect(page.getByText("Storage 3 game 4", { exact: true })).toBeVisible();
  await expect(page.getByText("Storage 3 game 5", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Next page", exact: true })).toBeDisabled();
  await expectNoHorizontalOverflow(page);

  // Model removal of the two entries on the last logical page. The UI must
  // recalculate from current content after reload, with no empty third page.
  await seedFirestoreDocument(`public_profiles/${uid}/pages/3`, {
    schema_version: 1, generation: 1, revision: 2,
    updated_at: firestoreTimestamp(updatedAt), entries: lastPageEntries.slice(0, 3),
  });
  await seedFirestoreDocument(`public_profiles/${uid}`, {
    ...rootFields, revision: 2, catalog_page_counts: [5, 0, 8, 3],
  });
  await page.reload();
  // The page query survives reload; a removed last page clamps to the last
  // remaining logical page rather than resetting the visitor to page one.
  await expect(page.getByText("Page 2 of 2")).toBeVisible();
  await expect(page.getByRole("listitem")).toHaveCount(8);
  await expect(page.getByRole("button", { name: "Next page", exact: true })).toBeDisabled();

  // A legacy root has no manifest. The same real SDK route derives counts
  // from its four bounded pages without requiring a migration write.
  const { catalog_page_counts: _counts, ...legacyFields } = rootFields;
  await seedFirestoreDocument(`public_profiles/${uid}`, { ...legacyFields, revision: 3 });
  await page.reload();
  await expect(page.getByText("Page 2 of 2")).toBeVisible();
  await page.getByRole("button", { name: "Previous page", exact: true }).click();
  await expect(page.getByText("Page 1 of 2")).toBeVisible();
  await expect(page.getByRole("listitem")).toHaveCount(8);
  await page.getByRole("button", { name: "Next page", exact: true }).click();
  await expect(page.getByText("Page 2 of 2")).toBeVisible();
  await expect(page.getByRole("listitem")).toHaveCount(8);
  expect((await readFirestoreDocument(`public_profiles/${uid}`))?.catalog_page_counts).toBeUndefined();
  await expectNoHorizontalOverflow(page);
});

test("C17: public replay returns preserve logical page, scroll and originating control", async ({ page }, testInfo) => {
  const uid = fixtureId("public-return");
  const updatedAt = firestoreTimestamp("2026-10-23T00:00:00.000Z");
  await seedFirestoreDocument(`public_profiles/${uid}`, {
    schema_version: 1, uid, display_name: "Return navigation player", enabled: true,
    generation: 1, revision: 1, updated_at: updatedAt,
    shared_stats: null, catalog_page_counts: [8, 8, 0, 0],
  });
  for (const pageId of [0, 1]) {
    const entries = Array.from({ length: 8 }, (_, index) => {
      const number = pageId * 8 + index + 1;
      const wire = {
        v: 1, r: "freestyle", m: Buffer.from(WIN_MOVES).toString("base64url"),
        b: { k: "h", n: `Player ${number}`, u: uid },
        w: { k: "h", n: "Opponent" }, at: 4,
      };
      return {
        fingerprint: Buffer.from(`return-${number}`).toString("base64url").padEnd(43, "x"),
        payload: Buffer.from(JSON.stringify(wire)).toString("base64url"),
        title: `Return game ${number}`, view_ply: 4,
      };
    });
    await seedFirestoreDocument(`public_profiles/${uid}/pages/${pageId}`, {
      schema_version: 1, generation: 1, revision: 1, updated_at: updatedAt, entries,
    });
  }
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`/player/?id=${uid}`);
  await expect(page.getByText("Page 1 of 2", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Next page", exact: true }).click();
  await expect(page.getByText("Page 2 of 2", { exact: true })).toBeVisible();
  await expect(page.getByRole("region", { name: "Shared player statistics" })).toHaveCount(0);
  const trigger = page.getByRole("link", { name: "Replay Return game 15", exact: true });
  await trigger.scrollIntoViewIfNeeded();
  await trigger.focus();
  const publicScrollTop = () => page.locator("main").evaluate((main) => (
    main.scrollHeight > main.clientHeight + 1 ? main.scrollTop : document.scrollingElement?.scrollTop ?? 0
  ));
  const originalScroll = await publicScrollTop();
  expect(originalScroll).toBeGreaterThan(100);
  await trigger.click();
  await expect(page.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
  expect(new URL(page.url()).searchParams.get("fromPage")).toBe("2");
  await expectNoHorizontalOverflow(page);
  await page.setViewportSize({ width: 320, height: 900 });
  await expectNoHorizontalOverflow(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await captureSharingEvidence(page, testInfo, "public-origin-replay", "main");

  await page.getByRole("button", { name: "Return to public profile", exact: true }).click();
  await expect(page.getByText("Page 2 of 2", { exact: true })).toBeVisible();
  await expect(trigger).toBeFocused();
  expect(await publicScrollTop()).toBeCloseTo(originalScroll, 0);
  await trigger.click();
  await expect(page.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
  await page.goBack();
  await expect(page.getByText("Page 2 of 2", { exact: true })).toBeVisible();
  await expect(trigger).toBeFocused();
  expect(await publicScrollTop()).toBeCloseTo(originalScroll, 0);
  await expectNoHorizontalOverflow(page);

  // Removing the originating entry leaves the same logical page available.
  // A return must focus the public name rather than a stale or private control.
  await trigger.click();
  await expect(page.getByRole("grid", { name: "Gomoku board" })).toBeVisible();
  const existingPage = await readFirestoreDocument(`public_profiles/${uid}/pages/1`);
  const entries = existingPage!.entries as Array<{ title: string }>;
  await seedFirestoreDocument(`public_profiles/${uid}/pages/1`, {
    ...existingPage, updated_at: updatedAt, revision: 2,
    entries: entries.filter((entry) => entry.title !== "Return game 15"),
  });
  await seedFirestoreDocument(`public_profiles/${uid}`, {
    schema_version: 1, uid, display_name: "Return navigation player", enabled: true,
    generation: 1, revision: 2, updated_at: updatedAt,
    shared_stats: null, catalog_page_counts: [8, 7, 0, 0],
  });
  await page.getByRole("button", { name: "Return to public profile", exact: true }).click();
  await expect(page.getByText("Page 2 of 2", { exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Return navigation player", exact: true })).toBeFocused();
});
