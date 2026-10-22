import { readFile } from "node:fs/promises";

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
  const runTransaction = ((firestore: Firestore, updateFunction: (transaction: Transaction) => Promise<unknown>, options?: unknown) => {
    if (!clearTransactionPause.consumePause()) {
      return actual.runTransaction(firestore, updateFunction, options as never);
    }

    return actual.runTransaction(firestore, async (transaction) => {
      let reads = 0;
      const pausedTransaction = new Proxy(transaction, {
        get(target, property) {
          if (property === "get") {
            return async (...args: unknown[]) => {
              const snapshot = await (target.get as (...getArgs: unknown[]) => Promise<unknown>)(...args);
              reads += 1;
              if (reads === 6) await clearTransactionPause.afterSixthRead();
              return snapshot;
            };
          }
          const value = Reflect.get(target, property, target) as unknown;
          return typeof value === "function" ? value.bind(target) : value;
        },
      });
      return updateFunction(pausedTransaction);
    }, options as never);
  }) as typeof actual.runTransaction;

  return { ...actual, runTransaction };
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

beforeAll(async () => {
  const rules = await readFile(new URL("../../firestore.rules", import.meta.url), "utf8");
  testEnv = await initializeTestEnvironment({ projectId, firestore: { rules } });
});

beforeEach(async () => {
  await testEnv.clearFirestore();
});

afterAll(async () => {
  await testEnv.cleanup();
});

describe("Firestore public replay sharing rules and APIs", () => {
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
    await saveCatalogPage("uid-1", 0, [entry()], 1, 1, { firestore: apiDb() });
    await expect(saveCatalogPage("uid-1", 0, [], 1, 1, { firestore: apiDb() })).rejects.toMatchObject({ name: "SharingConflictError" });
    await setPublicProfile("uid-1", "Other", true, 1, { firestore: apiDb() });
    await expect(setPublicProfile("uid-1", "Third", true, 1, { firestore: apiDb() })).rejects.toMatchObject({ name: "SharingConflictError" });
    await expect(saveCatalogPage("uid-1", 4, [], 1, 1, { firestore: apiDb() })).rejects.toThrow(/page/);
    await expect(saveCatalogPage("uid-1", 0, Array.from({ length: 9 }, (_, index) => entry(index + 1)), 1, 2, { firestore: apiDb() })).rejects.toThrow(/at most 8/);
    await expect(saveCatalogPage("uid-1", 0, [entry(), entry()], 1, 2, { firestore: apiDb() })).rejects.toThrow(/repeat a fingerprint/);
    await expect(setPublicProfile("uid-1", "", true, 1, { firestore: apiDb() })).rejects.toThrow(/Public name/);
  });

  it("accepts Unicode names and titles at both client size limits", async () => {
    const { saveCatalogPage, setPublicProfile } = await import(sharingPublicModule);
    const name = "🙂".repeat(80);
    const title = "🙂".repeat(80);
    await setPublicProfile("uid-1", name, true, null, { firestore: apiDb() });
    await expect(saveCatalogPage("uid-1", 0, [{ ...entry(), title }], 1, 1, { firestore: apiDb() }))
      .resolves.toMatchObject({ revision: 2, entries: [{ title }] });
    const payload = "A".repeat(8192);
    await expect(saveCatalogPage("uid-1", 1, [{ ...entry(2), payload }], 1, 1, { firestore: apiDb() }))
      .resolves.toMatchObject({ revision: 2, entries: [{ payload }] });
    await expect(setPublicProfile("uid-1", "🙂".repeat(81), true, 1, { firestore: apiDb() })).rejects.toThrow(/Public name/);
  });

  it("supports eight entries and rejects ninth entries in Firestore rules", async () => {
    const { saveCatalogPage } = await import(sharingPublicModule);
    await createPublicProfile();
    const entries = Array.from({ length: 8 }, (_, index) => entry(index + 1));
    await expect(saveCatalogPage("uid-1", 0, entries, 1, 1, { firestore: apiDb() }))
      .resolves.toMatchObject({ revision: 2, entries });
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
    await saveCatalogPage("uid-1", 0, [entry()], 1, 1, { firestore: apiDb() });
    const disabled = await setPublicProfile("uid-1", "", false, 1, { firestore: apiDb() });
    expect(disabled).toMatchObject({ enabled: false, displayName: "", generation: 2, revision: 2 });
    for (let pageId = 0; pageId < 4; pageId += 1) {
      const page = (await ownerDb().doc(`public_profiles/uid-1/pages/${pageId}`).get()).data();
      expect(page).toMatchObject({ generation: 2, revision: pageId === 0 ? 3 : 2, entries: [] });
    }
    const enabledAgain = await setPublicProfile("uid-1", "Fresh Name", true, 2, { firestore: apiDb() });
    expect(enabledAgain).toMatchObject({ enabled: true, generation: 3, revision: 3 });
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

  it("retains empty markers and blocks stale publication after clear", async () => {
    const { clearSharingData } = await import(sharingLifecycleModule);
    const { saveCatalogPage } = await import(sharingPublicModule);
    const { savePrivateLibrary } = await import(sharingPrivateModule);
    await createPublicProfile();
    await saveCatalogPage("uid-1", 0, [entry()], 1, 1, { firestore: apiDb() });
    await savePrivateLibrary("uid-1", [privateEntry()], null, { firestore: apiDb() });

    const cleared = await clearSharingData("uid-1", { firestore: apiDb() });
    expect(cleared.publicProfile).toMatchObject({ enabled: false, displayName: "", generation: 2, revision: 2 });
    expect(cleared.pages).toHaveLength(4);
    const clearedPages = cleared.pages as Array<{ generation: number; revision: number; entries: unknown[] }>;
    expect(clearedPages.every((page) => page.generation === 2 && page.entries.length === 0)).toBe(true);
    expect(clearedPages.map((page) => page.revision)).toEqual([3, 2, 2, 2]);
    expect(cleared.privateLibrary).toMatchObject({ revision: 2, entries: [] });
    await expect(saveCatalogPage("uid-1", 0, [entry(2)], 1, 2, { firestore: apiDb() })).rejects.toMatchObject({ name: "SharingDataError" });
    await expect(savePrivateLibrary("uid-1", [privateEntry(2)], 1, { firestore: apiDb() })).rejects.toMatchObject({ name: "SharingConflictError" });
    await assertFails(ownerDb().doc("public_profiles/uid-1").delete());
    await assertFails(ownerDb().doc("public_profiles/uid-1/pages/0").delete());
    await assertFails(ownerDb().doc("replay_libraries/uid-1").delete());
    await expect(ownerDb().doc("public_profiles/uid-1").get()).resolves.toMatchObject({ exists: true });
  });

  it("preserves concurrent publication and library saves when clear loses the race", async () => {
    const { clearSharingData } = await import(sharingLifecycleModule);
    const { saveCatalogPage } = await import(sharingPublicModule);
    const { loadPrivateLibrary, savePrivateLibrary } = await import(sharingPrivateModule);
    await createPublicProfile();
    await saveCatalogPage("uid-1", 0, [entry()], 1, 1, { firestore: apiDb() });
    await savePrivateLibrary("uid-1", [privateEntry()], null, { firestore: apiDb() });

    clearTransactionPause.arm();
    const clearing = clearSharingData("uid-1", { firestore: apiDb() });
    try {
      await clearTransactionPause.waitForReads();
      await saveCatalogPage("uid-1", 0, [entry(2)], 1, 2, { firestore: apiDb() });
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
      expect(unchangedRoot.data()).toMatchObject({ enabled: true, generation: 1, revision: 1 });
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
      await expect(saveCatalogPage("uid-1", 0, [entry()], 1, 1, { firestore: closedOwner })).rejects.toMatchObject({ code: "permission-denied" });
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
