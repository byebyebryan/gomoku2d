import { describe, expect, it, vi } from "vitest";

import type { BrowserStorage } from "../profile/best_effort_storage";
import type { PrivateLibraryEntry } from "../cloud/sharing_private";
import type { PortableReplayV1 } from "./portable_replay";
import { decodePortableReplay } from "./portable_replay_codec";
import {
  REPLAY_LIBRARY_LIMIT,
  REPLAY_LIBRARY_STORAGE_KEY,
  createReplayLibraryStore,
  currentReplayLibraryOwner,
} from "./replay_library_store";

const WIN_MOVES = [110, 0, 111, 1, 112, 2, 113, 3, 114];
const BOT: PortableReplayV1["white"] = {
  bot: {
    config: { mode: "preset", preset: "normal", version: 1 },
    configVersion: 1,
    engine: "search_bot",
    version: 1,
  },
  kind: "bot",
  name: "Normal Bot",
};

function winningReplay(name = "Guest", viewPly?: number): PortableReplayV1 {
  return {
    black: { kind: "human", name },
    moves: WIN_MOVES,
    ruleset: "freestyle",
    version: 1,
    ...(viewPly === undefined ? {} : { viewPly }),
    white: BOT,
  };
}

function createMemoryStorage(): BrowserStorage & { backing: Map<string, string> } {
  const backing = new Map<string, string>();
  return {
    backing,
    getItem: (name) => backing.get(name) ?? null,
    removeItem: (name) => { backing.delete(name); },
    setItem: (name, value) => { backing.set(name, value); },
  };
}

function pauseFirstDigest() {
  let signalStarted!: () => void;
  let release!: () => void;
  const started = new Promise<void>((resolve) => { signalStarted = resolve; });
  const gate = new Promise<void>((resolve) => { release = resolve; });
  const nativeDigest = globalThis.crypto.subtle.digest.bind(globalThis.crypto.subtle);
  let calls = 0;
  const digest = vi.spyOn(globalThis.crypto.subtle, "digest").mockImplementation(async (algorithm, data) => {
    if (calls++ === 0) {
      signalStarted();
      await gate;
    }
    return nativeDigest(algorithm, data);
  });
  return { digest, release, started };
}

describe("replay library store", () => {
  it("persists complete payloads, deduplicates across viewing plies, and restores dirty state", async () => {
    const storage = createMemoryStorage();
    const first = createReplayLibraryStore({ storage, now: () => "2026-10-21T00:00:00.000Z" });
    const fingerprint = await first.getState().saveReplay("guest", winningReplay("Guest", 2), "saved");
    const duplicate = await first.getState().saveReplay("guest", winningReplay("Guest", 8), "saved");

    expect(duplicate).toBe(fingerprint);
    expect(first.getState().owners.guest).toMatchObject({ dirty: true, entries: [{ fingerprint, kind: "saved" }] });
    expect(first.getState().owners.guest.entries).toHaveLength(1);
    const decoded = decodePortableReplay(first.getState().owners.guest.entries[0]!.payload);
    expect(decoded.replay.moves).toEqual(WIN_MOVES);
    expect(decoded.replay).not.toHaveProperty("viewPly");

    const restored = createReplayLibraryStore({ storage });
    expect(restored.getState().owners.guest).toMatchObject({ dirty: true, entries: first.getState().owners.guest.entries });
    restored.getState().markSynced("guest");
    expect(createReplayLibraryStore({ storage }).getState().owners.guest?.dirty).toBe(false);
    restored.getState().markDirty("guest");
    expect(createReplayLibraryStore({ storage }).getState().owners.guest?.dirty).toBe(true);
  });

  it("keeps different attribution as a separate entry and isolates owner changes", async () => {
    const store = createReplayLibraryStore({ storage: createMemoryStorage() });
    const guestFingerprint = await store.getState().saveReplay("guest", winningReplay("Guest"), "saved");
    const accountFingerprint = await store.getState().saveReplay("user:account-1", winningReplay("Guest 2"), "saved");

    expect(accountFingerprint).not.toBe(guestFingerprint);
    expect(store.getState().owners.guest?.entries).toHaveLength(1);
    expect(store.getState().owners["user:account-1"]?.entries).toHaveLength(1);
    await store.getState().removeReplay("guest", guestFingerprint);
    expect(store.getState().owners.guest?.entries).toEqual([]);
    expect(store.getState().owners["user:account-1"]?.entries[0]?.fingerprint).toBe(accountFingerprint);
  });

  it("merges independent saves that finish hashing concurrently", async () => {
    const store = createReplayLibraryStore({ storage: createMemoryStorage() });
    const fingerprints = await Promise.all([
      store.getState().saveReplay("guest", winningReplay("Concurrent A"), "saved"),
      store.getState().saveReplay("guest", winningReplay("Concurrent B"), "saved"),
    ]);

    expect(new Set(store.getState().owners.guest?.entries.map((entry) => entry.fingerprint)))
      .toEqual(new Set(fingerprints));
    expect(store.getState().owners.guest?.entries).toHaveLength(2);
  });

  it("rejects a stale replacement after a remove and concurrent save instead of restoring or dropping entries", async () => {
    const storage = createMemoryStorage();
    const source = createReplayLibraryStore({ storage: createMemoryStorage() });
    await source.getState().saveReplay("guest", winningReplay("Replacement snapshot"), "saved");
    const replacement = source.getState().owners.guest!.entries;
    const store = createReplayLibraryStore({ storage });
    const removedFingerprint = await store.getState().saveReplay("guest", winningReplay("Remove me"), "saved");
    const digestGate = pauseFirstDigest();

    try {
      const replacing = store.getState().replaceEntries("guest", replacement);
      await digestGate.started;
      await store.getState().removeReplay("guest", removedFingerprint);
      const concurrentFingerprint = await store.getState().saveReplay("guest", winningReplay("Keep me"), "saved");
      digestGate.release();

      await expect(replacing).rejects.toThrow(/changed while validating replacement/i);
      expect(store.getState().owners.guest?.entries.map((entry) => entry.fingerprint))
        .toEqual([concurrentFingerprint]);
      expect(store.getState().owners.guest?.entries).not.toEqual(replacement);
    } finally {
      digestGate.release();
      digestGate.digest.mockRestore();
    }
  });

  it("does not restore a save whose fingerprint finishes after clearOwner", async () => {
    const store = createReplayLibraryStore({ storage: createMemoryStorage() });
    await store.getState().saveReplay("guest", winningReplay("Before clear"), "saved");
    const digestGate = pauseFirstDigest();

    try {
      const pendingSave = store.getState().saveReplay("guest", winningReplay("Late save"), "saved");
      await digestGate.started;
      await store.getState().clearOwner("guest");
      digestGate.release();

      await expect(pendingSave).rejects.toThrow(/cleared or replaced/i);
      expect(store.getState().owners.guest).toMatchObject({ dirty: true, entries: [] });
      expect(store.getState().owners.guest?.error).toMatch(/cleared or replaced/i);
    } finally {
      digestGate.release();
      digestGate.digest.mockRestore();
    }
  });

  it("invalidates a pending save when a replacement snapshot commits first", async () => {
    const source = createReplayLibraryStore({ storage: createMemoryStorage() });
    await source.getState().saveReplay("guest", winningReplay("Reviewed snapshot"), "saved");
    const replacement = source.getState().owners.guest!.entries;
    const store = createReplayLibraryStore({ storage: createMemoryStorage() });
    const digestGate = pauseFirstDigest();

    try {
      const pendingSave = store.getState().saveReplay("guest", winningReplay("Late save"), "saved");
      await digestGate.started;
      await store.getState().replaceEntries("guest", replacement);
      digestGate.release();

      await expect(pendingSave).rejects.toThrow(/cleared or replaced/i);
      expect(store.getState().owners.guest?.entries).toEqual(replacement);
    } finally {
      digestGate.release();
      digestGate.digest.mockRestore();
    }
  });

  it("reconciles a validated snapshot and enforces the hard capacity without eviction", async () => {
    const store = createReplayLibraryStore({ storage: createMemoryStorage() });
    const entries: PrivateLibraryEntry[] = [];
    for (let index = 0; index < REPLAY_LIBRARY_LIMIT; index += 1) {
      const fingerprint = await store.getState().saveReplay("user:account-2", winningReplay(`Player ${index}`), "saved");
      entries.push(store.getState().owners["user:account-2"]!.entries.find((entry) => entry.fingerprint === fingerprint)!);
    }
    await store.getState().replaceEntries("user:account-3", entries);
    expect(store.getState().owners["user:account-3"]).toMatchObject({ dirty: false, entries });
    await expect(store.getState().replaceEntries("user:account-3", [
      { ...entries[0]!, fingerprint: "B".repeat(43) },
    ])).rejects.toThrow("does not match fingerprint");
    await expect(store.getState().replaceEntries("user:account-3", [
      { ...entries[0]!, payload: "A".repeat(43) },
    ])).rejects.toThrow();
    expect(store.getState().owners["user:account-3"]?.entries).toEqual(entries);
    const originalEntries = store.getState().owners["user:account-2"]!.entries;
    const firstFingerprint = originalEntries[originalEntries.length - 1]!.fingerprint;

    await expect(store.getState().saveReplay("user:account-2", winningReplay("Overflow"), "saved"))
      .rejects.toThrow("remove a replay before saving another");
    expect(store.getState().owners["user:account-2"]!.entries).toHaveLength(REPLAY_LIBRARY_LIMIT);
    const entriesAfterOverflow = store.getState().owners["user:account-2"]!.entries;
    expect(entriesAfterOverflow[entriesAfterOverflow.length - 1]!.fingerprint).toBe(firstFingerprint);
  });

  it.each([
    ["malformed JSON", "{not-json"],
    ["unsupported schema", JSON.stringify({ schema_version: 99, owners: {} })],
    ["illegal portable replay", JSON.stringify({
      schema_version: 1,
      owners: {
        guest: {
          dirty: true,
          entries: [{
            fingerprint: "A".repeat(43),
            payload: "A".repeat(43),
            kind: "saved",
            saved_at: "2026-10-21T00:00:00.000Z",
          }],
        },
      },
    })],
    ["oversized cache", "x".repeat(600_001)],
  ])("keeps an unusable %s cache out of the live store without overwriting it", async (_label, raw) => {
    const storage = createMemoryStorage();
    storage.setItem(REPLAY_LIBRARY_STORAGE_KEY, raw);
    storage.setItem("gomoku2d.local-profile.v5", "preserve-old-profile");
    const onPersistenceUnavailable = vi.fn();
    const store = createReplayLibraryStore({ storage, onPersistenceUnavailable });

    expect(store.getState().owners.guest).toBeUndefined();
    await store.getState().saveReplay("guest", winningReplay(), "saved");
    expect(storage.getItem(REPLAY_LIBRARY_STORAGE_KEY)).toBe(raw);
    expect(storage.getItem("gomoku2d.local-profile.v5")).toBe("preserve-old-profile");
    expect(store.getState().storageError).toBeTruthy();
    expect(onPersistenceUnavailable).toHaveBeenCalledOnce();
  });

  it("keeps saves usable in memory and reports denied persistence", async () => {
    const onPersistenceUnavailable = vi.fn();
    const store = createReplayLibraryStore({
      onPersistenceUnavailable,
      storage: {
        getItem: () => null,
        removeItem: () => undefined,
        setItem: () => { throw new DOMException("Storage full", "QuotaExceededError"); },
      },
    });

    const fingerprint = await store.getState().saveReplay("guest", winningReplay(), "saved");
    expect(store.getState().owners.guest?.entries[0]?.fingerprint).toBe(fingerprint);
    expect(store.getState().owners.guest?.error).toContain("Storage full");
    expect(store.getState().storageError).toContain("Storage full");
    expect(onPersistenceUnavailable).toHaveBeenCalledOnce();
  });

  it("derives an account-specific owner from the active Firebase UID", async () => {
    const { cloudAuthStore } = await import("../cloud/auth_store");
    const initial = cloudAuthStore.getState();
    try {
      cloudAuthStore.setState({
        ...initial,
        status: "signed_in",
        user: {
          avatarUrl: null,
          displayName: "Player",
          email: null,
          providerIds: [],
          uid: "auth_uid-42",
        },
      });
      expect(currentReplayLibraryOwner()).toBe("user:auth_uid-42");
    } finally {
      cloudAuthStore.setState(initial, true);
    }
  });
});
