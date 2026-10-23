import { mkdir, readFile, writeFile } from "node:fs/promises";

import {
  assertFails,
  assertSucceeds,
  initializeTestEnvironment,
  type RulesTestEnvironment,
} from "@firebase/rules-unit-testing";
import type { FieldValue, FirebaseFirestore } from "@firebase/firestore-types";
import firebase from "firebase/compat/app";
import "firebase/compat/firestore";
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import type { Firestore, Transaction } from "firebase/firestore";

const projectId = "demo-gomoku2d-sharing-rules-test";
const sharingPublicModule = "../src/cloud/sharing_public.js";
const sharingPrivateModule = "../src/cloud/sharing_private.js";
const sharingLifecycleModule = "../src/cloud/sharing_lifecycle.js";
let testEnv: RulesTestEnvironment;

const firestoreCallCounts = vi.hoisted(() => {
  let getDocFromServer = 0;
  let transactionGet = 0;
  let transactionSet = 0;
  let byPath: Record<string, { transactionGet: number; transactionSet: number }> = {};
  const pathOf = (reference: unknown): string => {
    if (typeof reference !== "object" || reference === null || !("path" in reference)) return "<unknown>";
    const path = (reference as { path?: unknown }).path;
    return typeof path === "string" ? path : "<unknown>";
  };
  const record = (path: string, operation: "transactionGet" | "transactionSet") => {
    const counts = byPath[path] ?? { transactionGet: 0, transactionSet: 0 };
    counts[operation] += 1;
    byPath[path] = counts;
  };
  return {
    reset() {
      getDocFromServer = 0;
      transactionGet = 0;
      transactionSet = 0;
      byPath = {};
    },
    recordServerGet() { getDocFromServer += 1; },
    recordTransactionGet(reference: unknown) {
      transactionGet += 1;
      record(pathOf(reference), "transactionGet");
    },
    recordTransactionSet(reference: unknown) {
      transactionSet += 1;
      record(pathOf(reference), "transactionSet");
    },
    snapshot() {
      return {
        getDocFromServer,
        transactionGet,
        transactionSet,
        byPath: JSON.parse(JSON.stringify(Object.fromEntries(
          Object.entries(byPath).sort(([left], [right]) => left < right ? -1 : left > right ? 1 : 0),
        ))) as typeof byPath,
      };
    },
  };
});

const clearTransactionPause = vi.hoisted(() => {
  let pauseNextTransaction = false;
  let resolveReadsComplete: (() => void) | null = null;
  let resolveRelease: (() => void) | null = null;
  let readsComplete = Promise.resolve();
  let released = Promise.resolve();

  return {
    arm() {
      pauseNextTransaction = true;
      readsComplete = new Promise<void>((resolve) => { resolveReadsComplete = resolve; });
      released = new Promise<void>((resolve) => { resolveRelease = resolve; });
    },
    consumePause() {
      if (!pauseNextTransaction) return false;
      pauseNextTransaction = false;
      return true;
    },
    async afterSixthRead() {
      resolveReadsComplete?.();
      await released;
    },
    waitForReads() {
      return readsComplete;
    },
    release() {
      resolveRelease?.();
    },
    reset() {
      pauseNextTransaction = false;
      resolveRelease?.();
      resolveReadsComplete = null;
      resolveRelease = null;
    },
  };
});

vi.mock("firebase/firestore", async (importOriginal) => {
  const actual = await importOriginal<typeof import("firebase/firestore")>();
  const getDocFromServer = ((...args: Parameters<typeof actual.getDocFromServer>) => {
    firestoreCallCounts.recordServerGet();
    return actual.getDocFromServer(...args);
  }) as typeof actual.getDocFromServer;
  const runTransaction = ((firestore: Firestore, updateFunction: (transaction: Transaction) => Promise<unknown>, options?: unknown) => {
    const pauseAfterSixthRead = clearTransactionPause.consumePause();
    return actual.runTransaction(firestore, async (transaction) => {
      let reads = 0;
      const pausedTransaction = new Proxy(transaction, {
        get(target, property) {
          if (property === "get") {
            return async (...args: unknown[]) => {
              const snapshot = await (target.get as (...getArgs: unknown[]) => Promise<unknown>)(...args);
              firestoreCallCounts.recordTransactionGet(args[0]);
              reads += 1;
              if (pauseAfterSixthRead && reads === 6) await clearTransactionPause.afterSixthRead();
              return snapshot;
            };
          }
          if (property === "set") {
            return (...args: unknown[]) => {
              firestoreCallCounts.recordTransactionSet(args[0]);
              return (target.set as (...setArgs: unknown[]) => unknown)(...args);
            };
          }
          const value = Reflect.get(target, property, target) as unknown;
          return typeof value === "function" ? value.bind(target) : value;
        },
      });
      return updateFunction(pausedTransaction);
    }, options as never);
  }) as typeof actual.runTransaction;

  return { ...actual, getDocFromServer, runTransaction };
});

const firebaseFirestore = (firebase as unknown as {
  firestore: {
    FieldValue: { serverTimestamp: () => FieldValue };
  };
}).firestore;

function serverTimestamp(): FieldValue {
  return firebaseFirestore.FieldValue.serverTimestamp();
}

function ownerDb(uid = "uid-1"): FirebaseFirestore {
  return testEnv.authenticatedContext(uid).firestore() as unknown as FirebaseFirestore;
}

function apiDb(uid = "uid-1"): Firestore {
  return ownerDb(uid) as unknown as Firestore;
}

function entry(index = 1): Record<string, unknown> {
  return {
    fingerprint: String(index).padStart(43, "a"),
    payload: "AQID",
    title: `Replay ${index}`,
    view_ply: 10,
  };
}

function privateEntry(index = 1): Record<string, unknown> {
  return {
    fingerprint: String(index).padStart(43, "b"),
    payload: "AQID",
    kind: "saved",
    saved_at: "2026-10-21T12:00:00.000Z",
  };
}

async function createPublicProfile(uid = "uid-1", displayName = "Public Name") {
  const { setPublicProfile } = await import(sharingPublicModule);
  return setPublicProfile(uid, displayName, true, null, { firestore: apiDb(uid) });
}

async function seedLegacyPublicProfile(entriesByPage: unknown[][] = [[], [], [], []]) {
  await testEnv.withSecurityRulesDisabled(async (context) => {
    const db = context.firestore();
    const timestamp = new Date("2020-01-01T00:00:00.000Z");
    await db.doc("public_profiles/uid-1").set({
      schema_version: 1,
      uid: "uid-1",
      display_name: "Legacy Name",
      enabled: true,
      generation: 1,
      revision: 1,
      updated_at: timestamp,
    });
    for (let pageId = 0; pageId < 4; pageId += 1) {
      await db.doc(`public_profiles/uid-1/pages/${pageId}`).set({
        schema_version: 1,
        generation: 1,
        revision: 1,
        updated_at: timestamp,
        entries: entriesByPage[pageId],
      });
    }
  });
}

beforeAll(async () => {
  const rules = await readFile(new URL("../../firestore.rules", import.meta.url), "utf8");
  testEnv = await initializeTestEnvironment({ projectId, firestore: { rules } });
});

beforeEach(async () => {
  await testEnv.clearFirestore();
  firestoreCallCounts.reset();
});

afterAll(async () => {
  await testEnv.cleanup();
});

describe("Firestore public replay sharing rules and APIs", () => {
  it("blocks legacy profile deletion until newer sharing data is cleared", async () => {
    const { savePrivateLibrary } = await import("../src/cloud/sharing_private.js");
    const { clearSharingData } = await import("../src/cloud/sharing_lifecycle.js");
    // Profile-delete rules depend on ownership and sharing state, not the
    // legacy profile contents. Seed only the document needed by this scenario.
    await testEnv.withSecurityRulesDisabled(async (context) => {
      await context.firestore().doc("profiles/uid-1").set({ uid: "uid-1" });
    });
    await createPublicProfile();
    await assertFails(ownerDb().doc("profiles/uid-1").delete());
    await clearSharingData("uid-1", { firestore: apiDb() });
    await savePrivateLibrary("uid-1", [privateEntry()], 1, { firestore: apiDb() });
    await assertFails(ownerDb().doc("profiles/uid-1").delete());
    await clearSharingData("uid-1", { firestore: apiDb() });
    await assertSucceeds(ownerDb().doc("profiles/uid-1").delete());
    // A pre-v0.6 account with neither family retains its deletion behavior.
    await testEnv.withSecurityRulesDisabled(async (context) => {
      await context.firestore().doc("profiles/uid-2").set({ uid: "uid-2" });
    });
    await assertSucceeds(ownerDb("uid-2").doc("profiles/uid-2").delete());
  });

  it("blocks legacy deletion while a disabled public root still has shared statistics", async () => {
    await testEnv.withSecurityRulesDisabled(async (context) => {
      const db = context.firestore();
      await db.doc("profiles/uid-1").set({ uid: "uid-1" });
      await db.doc("public_profiles/uid-1").set({
        schema_version: 1,
        uid: "uid-1",
        display_name: "",
        enabled: false,
        generation: 1,
        revision: 1,
        updated_at: new Date("2020-01-01T00:00:00.000Z"),
        shared_stats: {
          matches: 1, wins: 1, losses: 0, draws: 0,
          updated_at: new Date("2020-01-01T00:00:00.000Z"),
        },
      });
    });
    await assertFails(ownerDb().doc("profiles/uid-1").delete());
  });

  it("creates all four empty pages atomically and exposes only enabled known-UID reads", async () => {
    const { loadCatalogPage, loadPublicProfile } = await import(sharingPublicModule);
    const created = await createPublicProfile();
    expect(created).toMatchObject({ uid: "uid-1", displayName: "Public Name", enabled: true, generation: 1, revision: 1 });
    for (let pageId = 0; pageId < 4; pageId += 1) {
      const page = await ownerDb().doc(`public_profiles/uid-1/pages/${pageId}`).get();
      expect(page.data()).toMatchObject({ generation: 1, revision: 1, entries: [] });
    }
    await expect(loadPublicProfile("uid-1", { firestore: apiDb() })).resolves.toMatchObject({ enabled: true, revision: 1 });
    await expect(loadCatalogPage("uid-1", 0, { firestore: apiDb() })).resolves.toMatchObject({ generation: 1, entries: [] });
    await assertSucceeds(testEnv.unauthenticatedContext().firestore().doc("public_profiles/uid-1").get());
    await assertSucceeds(testEnv.unauthenticatedContext().firestore().doc("public_profiles/uid-1/pages/0").get());
    await assertSucceeds(ownerDb("uid-2").doc("public_profiles/uid-1").get());
    await assertSucceeds(ownerDb("uid-2").doc("public_profiles/uid-1/pages/0").get());
    await assertFails(testEnv.unauthenticatedContext().firestore().collection("public_profiles").get());
    await assertFails(ownerDb().collection("public_profiles").get());
  });

  it("lets an owner read a disabled profile and its pages while hiding both from visitors", async () => {
    const { loadCatalogPage, loadPublicProfile, setPublicProfile } = await import(sharingPublicModule);
    await createPublicProfile();
    await setPublicProfile("uid-1", "", false, 1, { firestore: apiDb() });
    await expect(loadPublicProfile("uid-1", { firestore: apiDb() })).resolves.toMatchObject({ enabled: false, displayName: "", generation: 2 });
    await expect(loadCatalogPage("uid-1", 0, { firestore: apiDb() })).resolves.toMatchObject({ generation: 2, entries: [] });
    await assertFails(testEnv.unauthenticatedContext().firestore().doc("public_profiles/uid-1").get());
    await assertFails(testEnv.unauthenticatedContext().firestore().doc("public_profiles/uid-1/pages/0").get());
  });

  it("enforces page IDs, eight-entry limits, exact fields, ASCII tokens, and entry bounds", async () => {
    await createPublicProfile();
    const page = ownerDb().doc("public_profiles/uid-1/pages/0");
    const original = (await page.get()).data()!;
    const writeEntries = (entries: unknown[]) => page.set({ ...original, revision: 2, updated_at: serverTimestamp(), entries });
    await assertFails(ownerDb().doc("public_profiles/uid-1/pages/4").set({ ...original, updated_at: serverTimestamp() }));
    await assertFails(writeEntries(Array.from({ length: 9 }, (_, index) => entry(index + 1))));
    await assertFails(writeEntries([{ ...entry(), extra: true }]));
    await assertFails(writeEntries([{ ...entry(), payload: `${"a".repeat(8192)}=` }]));
    await assertFails(writeEntries([{ ...entry(), payload: "ab+cd" }]));
    await assertFails(writeEntries([{ ...entry(), payload: "a".repeat(8193) }]));
    await assertFails(writeEntries([{ ...entry(), fingerprint: "a".repeat(42) }]));
    await assertFails(writeEntries([{ ...entry(), title: "t".repeat(81) }]));
    await assertFails(writeEntries([{ ...entry(), view_ply: 226 }]));
  });

  it("rejects stale page/profile revisions and rejects invalid API inputs before writing", async () => {
    const { saveCatalogPage, setPublicProfile } = await import(sharingPublicModule);
    await createPublicProfile();
    await saveCatalogPage("uid-1", 0, [entry()], 1, 1, 1, { firestore: apiDb() });
    await expect(saveCatalogPage("uid-1", 0, [], 1, 1, 1, { firestore: apiDb() })).rejects.toMatchObject({ name: "SharingConflictError" });
    await setPublicProfile("uid-1", "Other", true, 2, { firestore: apiDb() });
    await expect(setPublicProfile("uid-1", "Third", true, 1, { firestore: apiDb() })).rejects.toMatchObject({ name: "SharingConflictError" });
    await expect(saveCatalogPage("uid-1", 4, [], 3, 1, 2, { firestore: apiDb() })).rejects.toThrow(/page/);
    await expect(saveCatalogPage("uid-1", 0, Array.from({ length: 9 }, (_, index) => entry(index + 1)), 3, 2, 2, { firestore: apiDb() })).rejects.toThrow(/at most 8/);
    await expect(saveCatalogPage("uid-1", 0, [entry(), entry()], 3, 1, 2, { firestore: apiDb() })).rejects.toThrow(/repeat a fingerprint/);
    await expect(setPublicProfile("uid-1", "", true, 2, { firestore: apiDb() })).rejects.toThrow(/Public name/);
  });

  it("accepts Unicode names and titles at both client size limits", async () => {
    const { saveCatalogPage, setPublicProfile } = await import(sharingPublicModule);
    const name = "🙂".repeat(80);
    const title = "🙂".repeat(80);
    await setPublicProfile("uid-1", name, true, null, { firestore: apiDb() });
    const firstPage = await saveCatalogPage("uid-1", 0, [{ ...entry(), title }], 1, 1, 1, { firestore: apiDb() });
    expect(firstPage.page).toMatchObject({ revision: 2, entries: [{ title }] });
    const payload = "A".repeat(8192);
    const secondPage = await saveCatalogPage("uid-1", 1, [{ ...entry(2), payload }], 2, 1, 1, { firestore: apiDb() });
    expect(secondPage.page).toMatchObject({ revision: 2, entries: [{ payload }] });
    await expect(setPublicProfile("uid-1", "🙂".repeat(81), true, 3, { firestore: apiDb() })).rejects.toThrow(/Public name/);
  });

  it("supports eight entries and rejects ninth entries in Firestore rules", async () => {
    const { saveCatalogPage } = await import(sharingPublicModule);
    await createPublicProfile();
    const entries = Array.from({ length: 8 }, (_, index) => entry(index + 1));
    const saved = await saveCatalogPage("uid-1", 0, entries, 1, 1, 1, { firestore: apiDb() });
    expect(saved.page).toMatchObject({ revision: 2, entries });
    const page = ownerDb().doc("public_profiles/uid-1/pages/0");
    const current = (await page.get()).data()!;
    await assertFails(page.set({
      ...current,
      revision: 3,
      updated_at: serverTimestamp(),
      entries: [...entries, entry(9)],
    }));
  });

  it("makes visibility transitions and clear operations atomic across the four retained pages", async () => {
    const { saveCatalogPage, setPublicProfile } = await import(sharingPublicModule);
    await createPublicProfile();
    await saveCatalogPage("uid-1", 0, [entry()], 1, 1, 1, { firestore: apiDb() });
    const disabled = await setPublicProfile("uid-1", "", false, 2, { firestore: apiDb() });
    expect(disabled).toMatchObject({ enabled: false, displayName: "", generation: 2, revision: 3 });
    for (let pageId = 0; pageId < 4; pageId += 1) {
      const page = (await ownerDb().doc(`public_profiles/uid-1/pages/${pageId}`).get()).data();
      expect(page).toMatchObject({ generation: 2, revision: pageId === 0 ? 3 : 2, entries: [] });
    }
    const enabledAgain = await setPublicProfile("uid-1", "Fresh Name", true, 3, { firestore: apiDb() });
    expect(enabledAgain).toMatchObject({ enabled: true, generation: 3, revision: 4 });
    for (let pageId = 0; pageId < 4; pageId += 1) {
      const page = (await ownerDb().doc(`public_profiles/uid-1/pages/${pageId}`).get()).data();
      expect(page).toMatchObject({ generation: 3, revision: pageId === 0 ? 4 : 3, entries: [] });
    }

    const root = (await ownerDb().doc("public_profiles/uid-1").get()).data()!;
    await assertFails(ownerDb().doc("public_profiles/uid-1").set({
      ...root,
      enabled: false,
      display_name: "",
      generation: root.generation + 1,
      revision: root.revision + 1,
      updated_at: serverTimestamp(),
    }));
    await assertFails(ownerDb().doc("public_profiles/uid-1").delete());
    await assertFails(ownerDb().doc("public_profiles/uid-1/pages/0").delete());
  });

  it("preserves the exact shared-stats server timestamp across name and catalog writes", async () => {
    const { saveCatalogPage, setPublicProfile, setPublicProfileStats } = await import(sharingPublicModule);
    const counters = { matches: 3, wins: 2, losses: 1, draws: 0 };
    const created = await createPublicProfile();
    const shared = await setPublicProfileStats("uid-1", counters, created.revision, { firestore: apiDb() });
    const rootRef = ownerDb().doc("public_profiles/uid-1");
    const sharedRoot = (await rootRef.get()).data()!;
    const originalStatsTimestamp = (sharedRoot.shared_stats as Record<string, unknown>).updated_at as {
      isEqual?: (other: unknown) => boolean;
    };
    const rootWithoutStats = { ...sharedRoot };
    delete rootWithoutStats.shared_stats;
    await assertFails(rootRef.set({
      ...rootWithoutStats,
      revision: (sharedRoot.revision as number) + 1,
      updated_at: serverTimestamp(),
    }));

    const renamed = await setPublicProfile("uid-1", "Renamed", true, shared.revision, { firestore: apiDb() });
    const renamedRoot = (await rootRef.get()).data()!;
    const renamedStatsTimestamp = (renamedRoot.shared_stats as Record<string, unknown>).updated_at as typeof originalStatsTimestamp;
    expect(renamedRoot.shared_stats).toMatchObject(counters);
    expect(renamedStatsTimestamp.isEqual?.(originalStatsTimestamp)).toBe(true);

    const published = await saveCatalogPage("uid-1", 0, [entry()], renamed.revision, renamed.generation, 1, { firestore: apiDb() });
    const publishedRoot = (await rootRef.get()).data()!;
    const publishedStatsTimestamp = (publishedRoot.shared_stats as Record<string, unknown>).updated_at as typeof originalStatsTimestamp;
    expect(published.profile).toMatchObject({ revision: renamed.revision + 1, sharedStats: counters });
    expect(publishedStatsTimestamp.isEqual?.(originalStatsTimestamp)).toBe(true);
    await expect(setPublicProfileStats("uid-1", { matches: 1, wins: 1, losses: 0, draws: 0 }, shared.revision, { firestore: apiDb() }))
      .rejects.toMatchObject({ name: "SharingConflictError" });

    const optedOut = await setPublicProfileStats("uid-1", null, published.profile.revision, { firestore: apiDb() });
    expect(optedOut.sharedStats).toBeNull();
    expect((await rootRef.get()).data()?.shared_stats).toBeNull();
    await expect(setPublicProfileStats("uid-1", counters, published.profile.revision, { firestore: apiDb() }))
      .rejects.toMatchObject({ name: "SharingConflictError" });
    const refreshed = await setPublicProfileStats("uid-1", counters, optedOut.revision, { firestore: apiDb() });
    const disabled = await setPublicProfile("uid-1", "", false, refreshed.revision, { firestore: apiDb() });
    expect(disabled).toMatchObject({ enabled: false, sharedStats: null, catalogPageCounts: [0, 0, 0, 0] });
    expect((await rootRef.get()).data()?.shared_stats).toBeNull();
    await expect(setPublicProfileStats("uid-1", counters, disabled.revision, { firestore: apiDb() }))
      .rejects.toMatchObject({ name: "SharingDataError" });
    await expect(setPublicProfileStats("uid-1", { matches: 2, wins: 1, losses: 0, draws: 0 }, disabled.revision, { firestore: apiDb() }))
      .rejects.toThrow(/matches must equal/);
  });

  it("validates catalog manifest agreement and rejects page-only or mismatched root writes", async () => {
    const { saveCatalogPage } = await import(sharingPublicModule);
    await createPublicProfile();
    const page = ownerDb().doc("public_profiles/uid-1/pages/0");
    const root = ownerDb().doc("public_profiles/uid-1");
    const currentPage = (await page.get()).data()!;
    await assertFails(page.set({
      ...currentPage,
      revision: currentPage.revision + 1,
      updated_at: serverTimestamp(),
      entries: [entry()],
    }));
    const saved = await saveCatalogPage("uid-1", 0, [entry()], 1, 1, 1, { firestore: apiDb() });
    const publishedPage = (await page.get()).data()!;
    await assertFails(page.set({
      ...publishedPage,
      revision: publishedPage.revision + 1,
      updated_at: serverTimestamp(),
      entries: [entry(2)],
    }));
    const currentRootAfterPublish = (await root.get()).data()!;
    const rootWithoutCounts = { ...currentRootAfterPublish };
    delete rootWithoutCounts.catalog_page_counts;
    await assertFails(root.set({
      ...rootWithoutCounts,
      revision: (currentRootAfterPublish.revision as number) + 1,
      updated_at: serverTimestamp(),
    }));
    await assertFails(root.set({
      ...currentRootAfterPublish,
      revision: (currentRootAfterPublish.revision as number) + 1,
      updated_at: serverTimestamp(),
      catalog_page_counts: [2, 0, 0, 0],
    }));
    await assertFails(root.set({
      ...currentRootAfterPublish,
      revision: (currentRootAfterPublish.revision as number) + 1,
      updated_at: serverTimestamp(),
      catalog_page_counts: [9, 0, 0, 0],
    }));
    expect(saved.profile.catalogPageCounts).toEqual([1, 0, 0, 0]);
  });

  it("rejects invalid aggregate counters and client timestamps in Firestore rules", async () => {
    await createPublicProfile();
    const root = ownerDb().doc("public_profiles/uid-1");
    const current = (await root.get()).data()!;
    const updateWithStats = (stats: unknown) => root.set({
      ...current,
      revision: current.revision + 1,
      updated_at: serverTimestamp(),
      shared_stats: stats,
    });
    await assertFails(updateWithStats({
      matches: 2, wins: 1, losses: 0, draws: 0,
      updated_at: serverTimestamp(),
    }));
    await assertFails(updateWithStats({
      matches: 2_147_483_648, wins: 2_147_483_648, losses: 0, draws: 0,
      updated_at: serverTimestamp(),
    }));
    await assertFails(updateWithStats({
      matches: 1, wins: 1, losses: 0, draws: 0,
      updated_at: new Date("2020-01-01T00:00:00.000Z"),
    }));
    await assertFails(updateWithStats({
      matches: 1, wins: 1, losses: 0, draws: 0,
      updated_at: "2026-10-22T12:00:00.000Z",
    }));
  });

  it("initializes a legacy catalog manifest from four pages with bounded transaction operations", async () => {
    await seedLegacyPublicProfile([[entry(1)], [], [entry(3)], []]);
    const { loadPublicProfile, saveCatalogPage } = await import(sharingPublicModule);
    await expect(loadPublicProfile("uid-1", { firestore: apiDb() })).resolves.toMatchObject({
      revision: 1,
      sharedStats: null,
      catalogPageCounts: null,
    });

    firestoreCallCounts.reset();
    const saved = await saveCatalogPage("uid-1", 1, [entry(2)], 1, 1, 1, { firestore: apiDb() });
    expect(saved).toMatchObject({
      profile: { revision: 2, catalogPageCounts: [1, 1, 1, 0] },
      page: { pageId: 1, revision: 2, entries: [entry(2)] },
    });
    expect(firestoreCallCounts.snapshot()).toMatchObject({ transactionGet: 5, transactionSet: 2 });
    await expect(loadPublicProfile("uid-1", { firestore: apiDb() })).resolves.toMatchObject({
      revision: 2,
      catalogPageCounts: [1, 1, 1, 0],
    });
  });

  it("retains empty markers and blocks stale publication after clear", async () => {
    const { clearSharingData } = await import(sharingLifecycleModule);
    const { saveCatalogPage, setPublicProfileStats } = await import(sharingPublicModule);
    const { savePrivateLibrary } = await import(sharingPrivateModule);
    await createPublicProfile();
    const published = await saveCatalogPage("uid-1", 0, [entry()], 1, 1, 1, { firestore: apiDb() });
    await setPublicProfileStats("uid-1", { matches: 1, wins: 1, losses: 0, draws: 0 }, published.profile.revision, { firestore: apiDb() });
    await savePrivateLibrary("uid-1", [privateEntry()], null, { firestore: apiDb() });

    const cleared = await clearSharingData("uid-1", { firestore: apiDb() });
    expect(cleared.publicProfile).toMatchObject({ enabled: false, displayName: "", generation: 2, revision: 4, sharedStats: null });
    expect((await ownerDb().doc("public_profiles/uid-1").get()).data()?.shared_stats).toBeNull();
    expect(cleared.pages).toHaveLength(4);
    const clearedPages = cleared.pages as Array<{ generation: number; revision: number; entries: unknown[] }>;
    expect(clearedPages.every((page) => page.generation === 2 && page.entries.length === 0)).toBe(true);
    expect(clearedPages.map((page) => page.revision)).toEqual([3, 2, 2, 2]);
    expect(cleared.privateLibrary).toMatchObject({ revision: 2, entries: [] });
    await expect(saveCatalogPage("uid-1", 0, [entry(2)], 1, 2, 2, { firestore: apiDb() })).rejects.toMatchObject({ name: "SharingConflictError" });
    await expect(savePrivateLibrary("uid-1", [privateEntry(2)], 1, { firestore: apiDb() })).rejects.toMatchObject({ name: "SharingConflictError" });
    await assertFails(ownerDb().doc("public_profiles/uid-1").delete());
    await assertFails(ownerDb().doc("public_profiles/uid-1/pages/0").delete());
    await assertFails(ownerDb().doc("replay_libraries/uid-1").delete());
    await expect(ownerDb().doc("public_profiles/uid-1").get()).resolves.toMatchObject({ exists: true });
  });

  it("conflicts a stats refresh with clear and prevents a stale refresh from restoring cleared stats", async () => {
    const { clearSharingData } = await import(sharingLifecycleModule);
    const { setPublicProfileStats } = await import(sharingPublicModule);
    const initial = await createPublicProfile();
    const shared = await setPublicProfileStats("uid-1", { matches: 1, wins: 1, losses: 0, draws: 0 }, initial.revision, { firestore: apiDb() });

    clearTransactionPause.arm();
    const clearing = clearSharingData("uid-1", { firestore: apiDb() });
    try {
      await clearTransactionPause.waitForReads();
      const refreshed = await setPublicProfileStats("uid-1", { matches: 2, wins: 2, losses: 0, draws: 0 }, shared.revision, { firestore: apiDb() });
      clearTransactionPause.release();

      await expect(clearing).rejects.toMatchObject({ code: "permission-denied" });
      const afterRefresh = (await ownerDb().doc("public_profiles/uid-1").get()).data()!;
      expect(afterRefresh).toMatchObject({ enabled: true, generation: 1, revision: refreshed.revision });
      expect(afterRefresh.shared_stats).toMatchObject({ matches: 2, wins: 2, losses: 0, draws: 0 });

      const cleared = await clearSharingData("uid-1", { firestore: apiDb() });
      expect(cleared.publicProfile).toMatchObject({ enabled: false, generation: 2, sharedStats: null });
      await expect(setPublicProfileStats("uid-1", { matches: 2, wins: 2, losses: 0, draws: 0 }, refreshed.revision, { firestore: apiDb() }))
        .rejects.toMatchObject({ name: "SharingConflictError" });
      const afterClear = (await ownerDb().doc("public_profiles/uid-1").get()).data()!;
      expect(afterClear).toMatchObject({ enabled: false, shared_stats: null });
    } finally {
      clearTransactionPause.release();
      clearTransactionPause.reset();
    }
  });

  it("preserves concurrent publication and library saves when clear loses the race", async () => {
    const { clearSharingData } = await import(sharingLifecycleModule);
    const { saveCatalogPage } = await import(sharingPublicModule);
    const { loadPrivateLibrary, savePrivateLibrary } = await import(sharingPrivateModule);
    await createPublicProfile();
    await saveCatalogPage("uid-1", 0, [entry()], 1, 1, 1, { firestore: apiDb() });
    await savePrivateLibrary("uid-1", [privateEntry()], null, { firestore: apiDb() });

    clearTransactionPause.arm();
    const clearing = clearSharingData("uid-1", { firestore: apiDb() });
    try {
      await clearTransactionPause.waitForReads();
      await saveCatalogPage("uid-1", 0, [entry(2)], 2, 1, 2, { firestore: apiDb() });
      await savePrivateLibrary("uid-1", [privateEntry(2)], 1, { firestore: apiDb() });
      clearTransactionPause.release();

      await expect(clearing).rejects.toMatchObject({ code: "permission-denied" });
      const publishedPage = await ownerDb().doc("public_profiles/uid-1/pages/0").get();
      expect(publishedPage.data()).toMatchObject({ generation: 1, revision: 3, entries: [entry(2)] });
      for (let pageId = 1; pageId < 4; pageId += 1) {
        const untouchedPage = await ownerDb().doc(`public_profiles/uid-1/pages/${pageId}`).get();
        expect(untouchedPage.data()).toMatchObject({ generation: 1, revision: 1, entries: [] });
      }
      await expect(loadPrivateLibrary("uid-1", { firestore: apiDb() }))
        .resolves.toMatchObject({ revision: 2, entries: [privateEntry(2)] });
      const unchangedRoot = await ownerDb().doc("public_profiles/uid-1").get();
      expect(unchangedRoot.data()).toMatchObject({ enabled: true, generation: 1, revision: 3 });
    } finally {
      clearTransactionPause.release();
      clearTransactionPause.reset();
    }
  });
});

describe("Firestore private replay library rules and APIs", () => {
  it("does not create a library on load and restricts reads to its owner", async () => {
    const { loadPrivateLibrary } = await import(sharingPrivateModule);
    await expect(loadPrivateLibrary("uid-1", { firestore: apiDb() })).resolves.toBeNull();
    await assertFails(ownerDb("uid-2").doc("replay_libraries/uid-1").get());
    await assertFails(testEnv.unauthenticatedContext().firestore().doc("replay_libraries/uid-1").get());
    await assertFails(ownerDb().collection("replay_libraries").get());
  });

  it("enforces snapshot bounds, exact fields, revisions, and retained deletion barriers", async () => {
    const { loadPrivateLibrary, savePrivateLibrary } = await import(sharingPrivateModule);
    await expect(savePrivateLibrary("uid-1", [privateEntry()], null, { firestore: apiDb() })).resolves.toMatchObject({ revision: 1 });
    await expect(loadPrivateLibrary("uid-1", { firestore: apiDb() })).resolves.toMatchObject({ revision: 1, entries: [privateEntry()] });
    await expect(savePrivateLibrary("uid-1", [], 1, { firestore: apiDb() })).resolves.toMatchObject({ revision: 2, entries: [] });
    await expect(savePrivateLibrary("uid-1", [privateEntry()], 1, { firestore: apiDb() })).rejects.toMatchObject({ name: "SharingConflictError" });
    await expect(savePrivateLibrary("uid-1", Array.from({ length: 65 }, (_, index) => privateEntry(index + 1)), 2, { firestore: apiDb() })).rejects.toThrow(/at most 64/);
    await expect(savePrivateLibrary("uid-1", [{ ...privateEntry(), unexpected: true }], 2, { firestore: apiDb() })).rejects.toThrow(/invalid shape/);

    const library = ownerDb().doc("replay_libraries/uid-1");
    const current = (await library.get()).data()!;
    await assertFails(library.set({ ...current, revision: 3, updated_at: serverTimestamp(), unexpected: "field" }));
    await assertFails(library.set({ ...current, revision: 3, updated_at: serverTimestamp(), entries_json: "x".repeat(600001) }));
    await assertFails(library.set({ ...current, revision: 3, updated_at: serverTimestamp(), entries_json: "[\"é\"]" }));
    await assertFails(library.set({ ...current, revision: 2, updated_at: serverTimestamp(), entries_json: "[]" }));
    await assertFails(library.delete());
  });

  it("allows a missing private marker to be initialized once and rejects stale recreation", async () => {
    const { savePrivateLibrary } = await import(sharingPrivateModule);
    await expect(savePrivateLibrary("uid-1", [], null, { firestore: apiDb() })).resolves.toMatchObject({ revision: 1 });
    await expect(savePrivateLibrary("uid-1", [], null, { firestore: apiDb() })).rejects.toMatchObject({ name: "SharingConflictError" });
  });

  it("accepts the largest valid private snapshot through the production API and rules", async () => {
    const { savePrivateLibrary } = await import(sharingPrivateModule);
    const payload = "A".repeat(8192);
    const entries = Array.from({ length: 64 }, (_, index) => ({
      ...privateEntry(index + 1),
      payload,
    }));
    await expect(savePrivateLibrary("uid-1", entries, null, { firestore: apiDb() }))
      .resolves.toMatchObject({ revision: 1, entries: expect.arrayContaining([expect.objectContaining({ payload })]) });
  });

  it("honors the literal public sharing kill switch while retaining owner cleanup and private writes", async () => {
    const rules = await readFile(new URL("../../firestore.rules", import.meta.url), "utf8");
    const openSwitch = "function publicSharingEnabled() {\n      return true;\n    }";
    expect(rules).toContain(openSwitch);
    const closedEnv = await initializeTestEnvironment({
      projectId: "demo-gomoku2d-sharing-closed-test",
      firestore: { rules: rules.replace(openSwitch, openSwitch.replace("return true;", "return false;")) },
    });
    try {
      await closedEnv.withSecurityRulesDisabled(async (context) => {
        const db = context.firestore();
        await db.doc("public_profiles/uid-1").set({
          schema_version: 1, uid: "uid-1", display_name: "Public Name", enabled: true,
          generation: 1, revision: 1, updated_at: new Date("2020-01-01T00:00:00.000Z"),
        });
        for (let pageId = 0; pageId < 4; pageId += 1) {
          await db.doc(`public_profiles/uid-1/pages/${pageId}`).set({
            schema_version: 1, generation: 1, revision: 1,
            updated_at: new Date("2020-01-01T00:00:00.000Z"), entries: [],
          });
        }
      });
      const closedOwner = closedEnv.authenticatedContext("uid-1").firestore() as unknown as Firestore;
      await assertFails(closedEnv.unauthenticatedContext().firestore().doc("public_profiles/uid-1").get());
      const { saveCatalogPage, setPublicProfile } = await import(sharingPublicModule);
      const { savePrivateLibrary } = await import(sharingPrivateModule);
      await expect(saveCatalogPage("uid-1", 0, [entry()], 1, 1, 1, { firestore: closedOwner })).rejects.toMatchObject({ code: "permission-denied" });
      await expect(setPublicProfile("uid-2", "Public Name", true, null, {
        firestore: closedEnv.authenticatedContext("uid-2").firestore() as unknown as Firestore,
      })).rejects.toMatchObject({ code: "permission-denied" });
      await expect(setPublicProfile("uid-1", "", false, 1, { firestore: closedOwner }))
        .resolves.toMatchObject({ enabled: false, generation: 2 });
      await expect(savePrivateLibrary("uid-1", [privateEntry()], null, { firestore: closedOwner }))
        .resolves.toMatchObject({ revision: 1, entries: [privateEntry()] });
    } finally {
      await closedEnv.cleanup();
    }
  });
});

describe("Firestore sharing operation and document shape evidence", () => {
  it("records API method counts and bounded JSON fixture sizes", async () => {
    const { loadCatalogPage, loadPublicProfile, saveCatalogPage, setPublicProfile, setPublicProfileStats } = await import("../src/cloud/sharing_public.js");
    const { loadPrivateLibrary, savePrivateLibrary } = await import("../src/cloud/sharing_private.js");
    const { clearSharingData } = await import("../src/cloud/sharing_lifecycle.js");
    const counts: Record<string, ReturnType<typeof firestoreCallCounts.snapshot>> = {};
    const measure = async <T>(name: string, operation: () => Promise<T>): Promise<T> => {
      firestoreCallCounts.reset();
      const result = await operation();
      counts[name] = firestoreCallCounts.snapshot();
      return result;
    };

    const enabled = await measure("public_profile_enable_create", () => (
      setPublicProfile("uid-1", "Public Name", true, null, { firestore: apiDb() })
    ));
    const sharedStats = await measure("public_stats_share", () => (
      setPublicProfileStats("uid-1", { matches: 1, wins: 1, losses: 0, draws: 0 }, enabled.revision, { firestore: apiDb() })
    ));
    const optedOutStats = await measure("public_stats_opt_out", () => (
      setPublicProfileStats("uid-1", null, sharedStats.revision, { firestore: apiDb() })
    ));
    await measure("public_profile_load", () => loadPublicProfile("uid-1", { firestore: apiDb() }));
    await measure("public_first_page_load", () => loadCatalogPage("uid-1", 0, { firestore: apiDb() }));
    await measure("public_profile_manager_load_all_four_pages", async () => {
      await loadPublicProfile("uid-1", { firestore: apiDb() });
      for (let pageId = 0; pageId < 4; pageId += 1) {
        await loadCatalogPage("uid-1", pageId, { firestore: apiDb() });
      }
    });
    const renamed = await measure("public_profile_rename", () => (
      setPublicProfile("uid-1", "Renamed Public Name", true, optedOutStats.revision, { firestore: apiDb() })
    ));
    const published = await measure("public_page_publish", () => (
      saveCatalogPage("uid-1", 0, [entry()], renamed.revision, enabled.generation, 1, { firestore: apiDb() })
    ));
    const removed = await measure("public_page_remove", () => (
      saveCatalogPage("uid-1", 0, [], published.profile.revision, enabled.generation, published.page.revision, { firestore: apiDb() })
    ));
    await measure("public_profile_disable", () => (
      setPublicProfile("uid-1", "", false, removed.profile.revision, { firestore: apiDb() })
    ));
    await measure("private_library_load_missing", () => loadPrivateLibrary("uid-1", { firestore: apiDb() }));
    await measure("private_library_save_create", () => (
      savePrivateLibrary("uid-1", [privateEntry()], null, { firestore: apiDb() })
    ));
    await measure("private_library_load_existing", () => loadPrivateLibrary("uid-1", { firestore: apiDb() }));
    await measure("sharing_clear_six_documents", () => clearSharingData("uid-1", { firestore: apiDb() }));

    expect(counts.public_profile_enable_create).toMatchObject({ getDocFromServer: 0, transactionGet: 5, transactionSet: 5 });
    expect(counts.public_profile_load).toMatchObject({ getDocFromServer: 1, transactionGet: 0, transactionSet: 0 });
    expect(counts.public_first_page_load).toMatchObject({ getDocFromServer: 1, transactionGet: 0, transactionSet: 0 });
    expect(counts.public_profile_manager_load_all_four_pages.getDocFromServer).toBe(5);
    expect(counts.public_stats_share).toMatchObject({ transactionGet: 1, transactionSet: 1 });
    expect(counts.public_stats_opt_out).toMatchObject({ transactionGet: 1, transactionSet: 1 });
    expect(counts.public_profile_rename).toMatchObject({ transactionGet: 1, transactionSet: 1 });
    expect(counts.public_page_publish).toMatchObject({ transactionGet: 2, transactionSet: 2 });
    expect(counts.public_page_remove).toMatchObject({ transactionGet: 2, transactionSet: 2 });
    expect(counts.public_profile_disable).toMatchObject({ transactionGet: 5, transactionSet: 5 });
    expect(counts.private_library_load_missing).toMatchObject({ getDocFromServer: 1, transactionGet: 0, transactionSet: 0 });
    expect(counts.private_library_save_create).toMatchObject({ transactionGet: 1, transactionSet: 1 });
    expect(counts.private_library_load_existing).toMatchObject({ getDocFromServer: 1, transactionGet: 0, transactionSet: 0 });
    expect(counts.sharing_clear_six_documents).toMatchObject({ transactionGet: 6, transactionSet: 6 });

    const fixtureTimestamp = "2026-10-21T12:00:00.000Z";
    const pageDocument = (entries: unknown[]) => ({
      schema_version: 1,
      generation: 1,
      revision: 1,
      updated_at: fixtureTimestamp,
      entries,
    });
    const privateEntries = Array.from({ length: 64 }, (_, index) => ({
      ...privateEntry(index + 1),
      payload: "A".repeat(8192),
    }));
    const fullPublicPage = pageDocument(Array.from({ length: 8 }, (_, index) => ({
      ...entry(index + 1),
      payload: "A".repeat(8192),
    })));
    const fullPrivateEntriesJson = JSON.stringify(privateEntries);
    const privateDocument = (entriesJson: string) => ({
      schema_version: 1,
      uid: "uid-1",
      revision: 1,
      updated_at: fixtureTimestamp,
      entries_json: entriesJson,
    });
    const jsonFixtureBytes = {
      public_page_empty: Buffer.byteLength(JSON.stringify(pageDocument([])), "utf8"),
      public_page_full_8_entries_8192_payload_each: Buffer.byteLength(JSON.stringify(fullPublicPage), "utf8"),
      private_library_empty: Buffer.byteLength(JSON.stringify(privateDocument("[]")), "utf8"),
      private_library_full_64_entries_8192_payload_each: Buffer.byteLength(JSON.stringify(privateDocument(fullPrivateEntriesJson)), "utf8"),
      private_library_full_entries_json_chars: fullPrivateEntriesJson.length,
    };
    expect(fullPrivateEntriesJson.length).toBeLessThanOrEqual(600_000);
    expect(jsonFixtureBytes.private_library_full_64_entries_8192_payload_each).toBeLessThan(600_000);

    const clearByPath = counts.sharing_clear_six_documents.byPath;
    const evidence = {
      sources: {
        operation_counts: "Logical SDK document operations captured at production Firebase Web SDK getDocFromServer and Transaction.get/set calls.",
        json_fixture_bytes: "UTF-8 byte lengths for persisted-schema JSON fixtures; excludes Firestore/index/network overhead.",
      },
      measurement_scope: "Production sharing API calls instrumented around Firebase Web SDK getDocFromServer and Transaction.get/set while running against the Firestore emulator.",
      caveat: "Method counts are not SDK transport request or billable-read estimates. Rules dependent reads, transaction retries, Firestore protocol encoding, indexes, and service overhead are excluded.",
      operation_counts: {
        ...counts,
        private_library_clear_with_six_document_clear: {
          transactionGet: clearByPath["replay_libraries/uid-1"].transactionGet,
          transactionSet: clearByPath["replay_libraries/uid-1"].transactionSet,
          included_in: "sharing_clear_six_documents",
        },
      },
      json_fixture_bytes: {
        definition: "UTF-8 byte length of JSON.stringify on representative persisted-schema fixtures; excludes Firestore/index/protocol overhead and billing.",
        ...jsonFixtureBytes,
      },
    };
    if (process.env.GOMOKU_WRITE_EVIDENCE === "1") {
      const evidenceDirectory = new URL("../../docs/working/v0_6_evidence/", import.meta.url);
      await mkdir(evidenceDirectory, { recursive: true });
      await writeFile(new URL("sharing_shape.json", evidenceDirectory), `${JSON.stringify(evidence, null, 2)}\n`);
    }
  });
});
