import { describe, expect, it } from "vitest";

import { SharingConflictError } from "../cloud/sharing_common";
import type { PrivateLibrarySnapshot } from "../cloud/sharing_private";
import { createReplayLibraryStore, REPLAY_LIBRARY_STORAGE_KEY } from "./replay_library_store";
import { createReplayLibrarySync } from "./replay_library_sync";
import { validatePortableReplay, type PortableReplayV1 } from "./portable_replay";
import { encodePortableReplay, portableReplayFingerprint } from "./portable_replay_codec";

function memoryStorage() {
  const data = new Map<string, string>();
  return { getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => { data.set(key, value); },
    removeItem: (key: string) => { data.delete(key); } };
}
function replay(name = "Alice"): PortableReplayV1 {
  return { version: 1, ruleset: "freestyle", moves: [110, 0, 111, 1, 112, 2, 113, 3, 114],
    black: { kind: "human", name }, white: { kind: "human", name: "Bob" } };
}
function backend() {
  let snapshot: PrivateLibrarySnapshot | null = null;
  let reads = 0; let writes = 0;
  return {
    load: async () => { reads++; return snapshot; },
    save: async (uid: string, entries: unknown, revision: number | null) => {
      if (revision !== (snapshot?.revision ?? null)) throw new SharingConflictError();
      writes++;
      snapshot = { uid, entries: structuredClone(entries) as PrivateLibrarySnapshot["entries"],
        revision: (revision ?? 0) + 1, updatedAt: new Date().toISOString() };
      return snapshot;
    },
    counts: () => ({ reads, writes }),
  };
}

describe("explicit account replay-library sync", () => {
  it("does no backend work on construction and propagates saves/removals between clients", async () => {
    const cloud = backend();
    const localA = createReplayLibraryStore({ storage: memoryStorage() });
    const localB = createReplayLibraryStore({ storage: memoryStorage() });
    const a = createReplayLibrarySync({ local: localA, currentUid: () => "alice", ...cloud });
    const b = createReplayLibrarySync({ local: localB, currentUid: () => "alice", ...cloud });
    expect(cloud.counts()).toEqual({ reads: 0, writes: 0 });
    await a.reload("alice");
    const fp = await localA.getState().saveReplay("user:alice", replay(), "saved");
    await a.sync("alice");
    await b.reload("alice");
    expect(localB.getState().owners["user:alice"]!.entries[0]!.fingerprint).toBe(fp);
    await localB.getState().removeReplay("user:alice", fp);
    await b.sync("alice");
    await a.reload("alice");
    expect(a.store.getState().users.alice).toMatchObject({ status: "conflict", hasIncoming: true });
    await a.useCloud("alice", true);
    expect(localA.getState().owners["user:alice"]!.entries).toEqual([]);
  });

  it("keeps verified cloud reads available when the device lock is unavailable", async () => {
    const source = createReplayLibraryStore({ storage: memoryStorage() });
    const fingerprint = await source.getState().saveReplay("user:alice", replay(), "saved");
    const entry = source.getState().owners["user:alice"]!.entries[0]!;
    let reads = 0;
    let writes = 0;
    const local = createReplayLibraryStore({
      storage: memoryStorage(),
      lockManager: { request: async () => { throw new Error("origin lock denied"); } },
    });
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice",
      load: async () => { reads += 1; return { uid: "alice", revision: 7, entries: [entry], updatedAt: "2026-10-21T00:00:00.000Z" }; },
      save: async (_uid, entries, revision) => {
        writes += 1;
        return { uid: "alice", revision: (revision ?? 0) + 1, entries: entries as PrivateLibrarySnapshot["entries"],
          updatedAt: "2026-10-21T00:00:00.000Z" };
      },
    });

    expect(await local.getState().refreshFromStorage()).toBe(false);
    await sync.reload("alice");
    expect(reads).toBe(1);
    expect(writes).toBe(0);
    expect(local.getState().owners["user:alice"]?.entries[0]?.fingerprint).toBe(fingerprint);
    expect(local.getState().owners["user:alice"]?.durable).toBe(false);
    expect(local.getState().sessionOnly).toBe(true);
    expect(sync.store.getState().users.alice?.status).toBe("synced");
  });

  it("can sync session-only edits when their missing-marker base was verified first", async () => {
    const cloud = backend();
    const local = createReplayLibraryStore({ storage: memoryStorage(), lockManager: null });
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice", ...cloud });
    await sync.reload("alice");
    await local.getState().saveReplay("user:alice", replay(), "saved");
    expect(local.getState().owners["user:alice"]?.durable).toBe(false);

    await sync.sync("alice");
    expect(cloud.counts().writes).toBe(1);
    expect(sync.store.getState().users.alice?.status).toBe("synced");
    expect(local.getState().owners["user:alice"]).toMatchObject({ dirty: false, baseRevision: 1, durable: false });
  });

  it("requires review instead of silently replaying an offline stale snapshot over deletion", async () => {
    const cloud = backend();
    const localA = createReplayLibraryStore({ storage: memoryStorage() });
    const localB = createReplayLibraryStore({ storage: memoryStorage() });
    const a = createReplayLibrarySync({ local: localA, currentUid: () => "alice", ...cloud });
    const b = createReplayLibrarySync({ local: localB, currentUid: () => "alice", ...cloud });
    await a.reload("alice");
    const fp = await localA.getState().saveReplay("user:alice", replay(), "saved");
    await a.sync("alice"); await b.reload("alice");
    await localA.getState().removeReplay("user:alice", fp); await a.sync("alice");
    await localB.getState().saveReplay("user:alice", replay("Carol"), "saved");
    await expect(b.sync("alice")).rejects.toThrow(/changed/i);
    await b.reload("alice");
    expect(b.store.getState().users.alice?.status).toBe("conflict");
    expect((await cloud.load())?.entries).toEqual([]);
    await b.useCloud("alice", true);
    expect(localB.getState().owners["user:alice"]!.entries).toEqual([]);
  });

  it("treats persisted unsynced copies as requiring an explicit first-load merge", async () => {
    const cloud = backend();
    const local = createReplayLibraryStore({ storage: memoryStorage() });
    await local.getState().saveReplay("user:alice", replay(), "saved");
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice", ...cloud });
    await sync.reload("alice");
    expect(sync.store.getState().users.alice?.status).toBe("conflict");
    expect(cloud.counts().writes).toBe(0);
    await sync.mergeCopies("alice");
    expect((await cloud.load())?.entries).toHaveLength(1);
    expect(local.getState().owners["user:alice"]!.dirty).toBe(false);
  });

  it("keeps the original device details on disk when the reviewed cloud revision is rejected", async () => {
    const owner = "user:alice";
    const cloud = backend();
    const storage = memoryStorage();
    const local = createReplayLibraryStore({ storage });
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice", ...cloud });
    await sync.reload("alice");
    const fingerprint = await local.getState().saveReplay(owner, replay(), "saved");
    await sync.sync("alice");

    const device = { title: "Device", note: "Device note", opening_ply: 0 };
    const firstCloud = { title: "First cloud", note: "Reviewed cloud note", opening_ply: 2 };
    const newestCloud = { title: "Newest cloud", note: "New cloud note", opening_ply: 4 };
    const entry = (await cloud.load())!.entries[0]!;
    await local.getState().updateReplayMetadata(owner, fingerprint, device);
    await cloud.save("alice", [{ ...entry, metadata: firstCloud }], 1);
    await sync.reload("alice");
    const plan = await sync.planReplayLibraryMerge("alice");
    const originalBytes = storage.getItem(REPLAY_LIBRARY_STORAGE_KEY);
    const originalLocal = local.getState().owners[owner];
    await cloud.save("alice", [{ ...entry, metadata: newestCloud }], 2);

    await expect(sync.commitReplayLibraryMerge(plan, { [fingerprint]: "cloud" })).rejects.toThrow(SharingConflictError);
    expect(storage.getItem(REPLAY_LIBRARY_STORAGE_KEY)).toBe(originalBytes);
    expect(local.getState().owners[owner]).toBe(originalLocal);
    expect((await cloud.load())?.entries[0]?.metadata).toEqual(newestCloud);

    await sync.reload("alice");
    const fresh = await sync.planReplayLibraryMerge("alice");
    expect(fresh.rows[0]).toMatchObject({
      status: "metadata-conflict", local: { metadata: device }, cloud: { metadata: newestCloud },
    });
    await sync.commitReplayLibraryMerge(fresh, { [fingerprint]: "local" });
    expect((await cloud.load())?.entries[0]?.metadata).toEqual(device);
    expect(local.getState().owners[owner]).toMatchObject({ baseRevision: 4, dirty: false });
  });

  it("rejects a merge before any cloud write if another tab changed the local cache", async () => {
    const cloud = backend();
    const storage = memoryStorage();
    const local = createReplayLibraryStore({ storage });
    await local.getState().saveReplay("user:alice", replay(), "saved");
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice", ...cloud });
    await sync.reload("alice");
    const plan = await sync.planReplayLibraryMerge("alice");
    const otherTab = createReplayLibraryStore({ storage });
    await otherTab.getState().saveReplay("guest", replay("Guest tab"), "saved");
    const newerBytes = storage.getItem(REPLAY_LIBRARY_STORAGE_KEY);

    await expect(sync.commitReplayLibraryMerge(plan, {})).rejects.toThrow(/changed in another tab/);
    expect(storage.getItem(REPLAY_LIBRARY_STORAGE_KEY)).toBe(newerBytes);
    expect(cloud.counts().writes).toBe(0);
  });

  it("checks the combined device cache budget before attempting a reviewed cloud write", async () => {
    const metadata = { title: "🟡".repeat(80), note: "🟡".repeat(280), opening_ply: 0 };
    const saved_at = "2026-11-18T00:00:00.000Z";
    const entries = await Promise.all(Array.from({ length: 64 }, async (_, index) => {
      const game = replay(`Player ${index}`);
      return { fingerprint: await portableReplayFingerprint(validatePortableReplay(game)),
        payload: encodePortableReplay(game), kind: "saved" as const, saved_at, metadata };
    }));
    const packed = { entries, dirty: true, base_revision: "unknown", restore_review: false };
    const storage = memoryStorage();
    const original = JSON.stringify({ schema_version: 2, revision: 1, owners: {
      guest: packed, "user:b": packed, "user:c": packed, "user:d": packed, "user:e": packed,
      "user:alice": { ...packed, entries: entries.slice(0, 1) },
    } });
    storage.setItem(REPLAY_LIBRARY_STORAGE_KEY, original);
    const cloud = backend();
    await cloud.save("alice", entries, null);
    const local = createReplayLibraryStore({ storage });
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice", ...cloud });
    await sync.reload("alice");

    await expect(sync.mergeCopies("alice")).rejects.toThrow(/cache exceeds 600000 bytes/i);
    expect(storage.getItem(REPLAY_LIBRARY_STORAGE_KEY)).toBe(original);
    expect(cloud.counts().writes).toBe(1);
  });

  it("preserves concurrent local additions and removals while an explicit merge commits", async () => {
    const cloud = backend();
    const local = createReplayLibraryStore({ storage: memoryStorage() });
    const first = await local.getState().saveReplay("user:alice", replay(), "saved");
    let release!: () => void;
    let started!: () => void;
    const barrier = new Promise<void>((resolve) => { release = resolve; });
    const saving = new Promise<void>((resolve) => { started = resolve; });
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice", load: cloud.load,
      save: async (...args) => { started(); await barrier; return cloud.save(args[0], args[1], args[2]); } });
    await sync.reload("alice");
    const pending = sync.mergeCopies("alice");
    await saving;
    await local.getState().removeReplay("user:alice", first);
    const second = await local.getState().saveReplay("user:alice", replay("Carol"), "saved");
    release(); await pending;
    expect(local.getState().owners["user:alice"]?.entries.map((entry) => entry.fingerprint)).toEqual([second]);
    expect(local.getState().owners["user:alice"]?.dirty).toBe(true);
    expect(sync.store.getState().users.alice).toMatchObject({ status: "conflict", hasIncoming: true });
    const review = await sync.planReplayLibraryMerge("alice");
    await sync.commitReplayLibraryMerge(review, {});
    expect((await cloud.load())?.entries.map((entry) => entry.fingerprint)).toEqual([first, second]);
  });

  it("does not let a delayed old save undo cloud-clear recovery state", async () => {
    const cloud = backend();
    const local = createReplayLibraryStore({ storage: memoryStorage() });
    let release!: () => void;
    const barrier = new Promise<void>((resolve) => { release = resolve; });
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice", load: cloud.load,
      save: async (...args) => { const result = await cloud.save(args[0], args[1], args[2]); await barrier; return result; } });
    await sync.reload("alice");
    await local.getState().saveReplay("user:alice", replay(), "saved");
    const pending = sync.sync("alice");
    const cleared = await cloud.save("alice", [], 1);
    sync.acceptCleared("alice", cleared);
    release(); await pending;
    expect(sync.store.getState().users.alice?.revision).toBe(2);
    expect(sync.store.getState().users.alice?.status).toBe("conflict");
    expect(local.getState().owners["user:alice"]?.dirty).toBe(true);
    expect((await cloud.load())?.entries).toEqual([]);
  });

  it.each(["sync", "merge"] as const)("ignores a late failed %s after cloud clear supersedes it", async (operation) => {
    const cloud = backend();
    const local = createReplayLibraryStore({ storage: memoryStorage() });
    let rejectSave!: (error: Error) => void;
    let started!: () => void;
    let saves = 0;
    const saving = new Promise<void>((resolve) => { started = resolve; });
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice", load: cloud.load,
      save: (...args) => saves++ === 0
        ? new Promise((_resolve, reject) => { rejectSave = reject; started(); })
        : cloud.save(args[0], args[1], args[2]) });
    if (operation === "sync") await sync.reload("alice");
    const kept = await local.getState().saveReplay("user:alice", replay(), "saved");
    if (operation === "merge") await sync.reload("alice");
    const pending = operation === "sync" ? sync.sync("alice") : sync.mergeCopies("alice");
    const rejection = expect(pending).rejects.toThrow("Old request failed");
    await saving;

    const cleared = await cloud.save("alice", [], null);
    sync.acceptCleared("alice", cleared);
    const recovery = sync.store.getState().users.alice;
    rejectSave(new Error("Old request failed"));
    await rejection;

    await expect(sync.sync("alice")).rejects.toThrow();
    expect(sync.store.getState().users.alice).toEqual(recovery);
    expect(sync.store.getState().users.alice).toMatchObject({ revision: 1, status: "conflict", hasIncoming: true });
    expect(local.getState().owners["user:alice"]?.entries.map((entry) => entry.fingerprint)).toEqual([kept]);
    expect((await cloud.load())?.entries).toEqual([]);
    expect(cloud.counts().writes).toBe(1);
  });

  it("keeps retained copies behind explicit review after clear and same-revision reload", async () => {
    const cloud = backend();
    const local = createReplayLibraryStore({ storage: memoryStorage() });
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice", ...cloud });
    await sync.reload("alice");
    await local.getState().saveReplay("user:alice", replay(), "saved");
    await sync.sync("alice");
    const cleared = await cloud.save("alice", [], 1);
    local.getState().markDirty("user:alice");
    sync.acceptCleared("alice", cleared);
    await sync.reload("alice");
    expect(sync.store.getState().users.alice?.status).toBe("conflict");
    expect(sync.store.getState().users.alice?.hasIncoming).toBe(true);
    await expect(sync.sync("alice")).rejects.toThrow();
    expect((await cloud.load())?.entries).toEqual([]);
    await sync.mergeCopies("alice");
    expect((await cloud.load())?.entries).toHaveLength(1);
  });

  it("keeps the review barrier when a reload fails after cloud clear", async () => {
    const cloud = backend();
    const local = createReplayLibraryStore({ storage: memoryStorage() });
    let offline = false;
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice", save: cloud.save,
      load: async () => { if (offline) throw new Error("Offline"); return cloud.load(); } });
    await sync.reload("alice");
    await local.getState().saveReplay("user:alice", replay(), "saved");
    const cleared = await cloud.save("alice", [], null);
    sync.acceptCleared("alice", cleared);

    offline = true;
    await expect(sync.reload("alice")).rejects.toThrow("Offline");
    offline = false;
    await expect(sync.sync("alice")).rejects.toThrow(/review/i);
    expect(sync.store.getState().users.alice?.hasIncoming).toBe(true);
    expect((await cloud.load())?.entries).toEqual([]);
    expect(cloud.counts().writes).toBe(1);
  });

  it("cancels a reviewed replacement invalidated by a later cloud clear", async () => {
    const cloud = backend();
    const seed = createReplayLibraryStore({ storage: memoryStorage() });
    await seed.getState().saveReplay("user:alice", replay(), "saved");
    await cloud.save("alice", seed.getState().owners["user:alice"]!.entries, null);
    const local = createReplayLibraryStore({ storage: memoryStorage() });
    const kept = await local.getState().saveReplay("user:alice", replay("Carol"), "saved");
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice", ...cloud });
    await sync.reload("alice");
    let release!: () => void;
    const barrier = new Promise<void>((resolve) => { release = resolve; });
    const replace = local.getState().replaceEntries;
    local.setState({ replaceEntries: async (...args) => { await barrier; return replace(...args); } });
    const pending = sync.useCloud("alice", true);
    const cleared = await cloud.save("alice", [], 1);
    sync.acceptCleared("alice", cleared);
    release();
    await expect(pending).rejects.toThrow(/superseded|changed after review/i);
    expect(local.getState().owners["user:alice"]?.entries.map((entry) => entry.fingerprint)).toEqual([kept]);
    expect(sync.store.getState().users.alice).toMatchObject({ revision: 2, status: "conflict", hasIncoming: true });
  });

  it("ignores an old-account read after an account switch", async () => {
    const local = createReplayLibraryStore({ storage: memoryStorage() });
    let uid: string | null = "alice";
    let finish!: (value: PrivateLibrarySnapshot | null) => void;
    const sync = createReplayLibrarySync({ local, currentUid: () => uid,
      load: () => new Promise((resolve) => { finish = resolve; }) });
    const pending = sync.reload("alice");
    uid = "bob"; finish(null);
    await expect(pending).rejects.toThrow(/account changed/i);
    expect(local.getState().owners["user:bob"]).toBeUndefined();
    expect(local.getState().owners["user:alice"]).toBeUndefined();
  });
});
