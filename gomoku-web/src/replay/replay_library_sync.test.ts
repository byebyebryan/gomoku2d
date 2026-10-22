import { describe, expect, it } from "vitest";

import { SharingConflictError } from "../cloud/sharing_common";
import type { PrivateLibrarySnapshot } from "../cloud/sharing_private";
import { createReplayLibraryStore } from "./replay_library_store";
import { createReplayLibrarySync } from "./replay_library_sync";
import type { PortableReplayV1 } from "./portable_replay";

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
    expect(localA.getState().owners["user:alice"]!.entries).toEqual([]);
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
    await b.useCloud("alice");
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

  it("preserves concurrent local additions and removals while an explicit merge commits", async () => {
    const cloud = backend();
    const local = createReplayLibraryStore({ storage: memoryStorage() });
    const first = await local.getState().saveReplay("user:alice", replay(), "saved");
    let release!: () => void;
    const barrier = new Promise<void>((resolve) => { release = resolve; });
    const sync = createReplayLibrarySync({ local, currentUid: () => "alice", load: cloud.load,
      save: async (...args) => { await barrier; return cloud.save(args[0], args[1], args[2]); } });
    await sync.reload("alice");
    const pending = sync.mergeCopies("alice");
    await local.getState().removeReplay("user:alice", first);
    const second = await local.getState().saveReplay("user:alice", replay("Carol"), "saved");
    release(); await pending;
    expect(local.getState().owners["user:alice"]?.entries.map((entry) => entry.fingerprint)).toEqual([second]);
    expect(local.getState().owners["user:alice"]?.dirty).toBe(true);
    expect(sync.store.getState().users.alice?.status).toBe("pending");
    await sync.sync("alice");
    expect((await cloud.load())?.entries.map((entry) => entry.fingerprint)).toEqual([second]);
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
