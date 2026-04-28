import { describe, expect, it, vi } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";
import type { GuestProfileStorage } from "../profile/guest_profile_store";

import type { CloudAuthUser } from "./auth_store";
import { createCloudHistoryStore } from "./cloud_history_store";

function createMemoryStorage(): GuestProfileStorage {
  const backing = new Map<string, string>();

  return {
    getItem: (name) => backing.get(name) ?? null,
    removeItem: (name) => {
      backing.delete(name);
    },
    setItem: (name, value) => {
      backing.set(name, value);
    },
  };
}

const user: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

const match = createLocalSavedMatch({
  id: "match-1",
  localProfileId: "guest-1",
  moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
  players: [
    { kind: "human", name: "Bryan", stone: "black" },
    { kind: "bot", name: "Practice Bot", stone: "white" },
  ],
  savedAt: "2026-04-28T01:02:03.000Z",
  status: "draw",
  variant: "freestyle",
});

const cloudMatch = {
  ...match,
  player_black: {
    ...match.player_black,
    local_profile_id: null,
    profile_uid: "uid-1",
  },
  source: "cloud_saved" as const,
  trust: "client_uploaded" as const,
};

describe("createCloudHistoryStore", () => {
  it("loads and persists a per-user cloud history cache", async () => {
    const storage = createMemoryStorage();
    const loadHistory = vi.fn().mockResolvedValue([cloudMatch]);
    const store = createCloudHistoryStore({
      loadHistory,
      now: () => "2026-04-28T02:00:00.000Z",
      storage,
    });

    await store.getState().loadForUser(user);

    expect(store.getState().loadStatus).toBe("ready");
    expect(loadHistory).toHaveBeenCalledWith(user, null);
    expect(store.getState().users["uid-1"]).toMatchObject({
      cachedMatches: [cloudMatch],
      loadedAt: "2026-04-28T02:00:00.000Z",
    });

    const reloaded = createCloudHistoryStore({ storage });
    expect(reloaded.getState().users["uid-1"]?.cachedMatches).toHaveLength(1);
  });

  it("syncs a pending local match and caches the cloud version", async () => {
    const saveMatch = vi.fn().mockResolvedValue({ match: cloudMatch });
    const store = createCloudHistoryStore({
      now: () => "2026-04-28T02:00:00.000Z",
      saveMatch,
      storage: createMemoryStorage(),
    });

    await store.getState().syncMatchForUser(user, match);

    const cache = store.getState().users["uid-1"];
    expect(saveMatch).toHaveBeenCalledWith(user, match);
    expect(cache?.cachedMatches).toEqual([cloudMatch]);
    expect(cache?.pendingMatches).toEqual({});
    expect(cache?.sync["match-1"]).toMatchObject({
      errorMessage: null,
      status: "synced",
    });
  });

  it("keeps failed sync records pending for retry", async () => {
    const saveMatch = vi.fn().mockRejectedValue(new Error("offline"));
    const store = createCloudHistoryStore({
      now: () => "2026-04-28T02:00:00.000Z",
      saveMatch,
      storage: createMemoryStorage(),
    });

    await store.getState().syncMatchForUser(user, match);

    const cache = store.getState().users["uid-1"];
    expect(store.getState()).toMatchObject({
      errorMessage: "offline",
      syncStatus: "error",
    });
    expect(cache?.pendingMatches["match-1"]).toEqual(match);
    expect(cache?.sync["match-1"]).toMatchObject({
      errorMessage: "offline",
      status: "error",
    });
  });

  it("retries all pending records for a user", async () => {
    const saveMatch = vi
      .fn()
      .mockRejectedValueOnce(new Error("offline"))
      .mockResolvedValueOnce({ match: cloudMatch });
    const store = createCloudHistoryStore({
      saveMatch,
      storage: createMemoryStorage(),
    });

    await store.getState().syncMatchForUser(user, match);
    await store.getState().syncPendingForUser(user);

    expect(saveMatch).toHaveBeenCalledTimes(2);
    expect(store.getState().users["uid-1"]?.pendingMatches).toEqual({});
  });

  it("drops pending records older than the reset barrier without syncing them", async () => {
    const saveMatch = vi.fn().mockResolvedValue({ match: cloudMatch });
    const store = createCloudHistoryStore({
      saveMatch,
      storage: createMemoryStorage(),
    });

    await store.getState().syncMatchForUser(user, match, "2026-04-28T02:00:00.000Z");

    expect(saveMatch).not.toHaveBeenCalled();
    expect(store.getState().users["uid-1"]?.pendingMatches).toEqual({});
  });

  it("drops a cloud save result when the reset barrier appears during sync", async () => {
    let historyResetAt: string | null = null;
    const saveMatch = vi.fn(async () => {
      historyResetAt = "2026-04-28T02:00:00.000Z";
      return { match: cloudMatch };
    });
    const store = createCloudHistoryStore({
      historyResetAtForUser: () => historyResetAt,
      saveMatch,
      storage: createMemoryStorage(),
    });

    await store.getState().syncMatchForUser(user, match);

    const cache = store.getState().users["uid-1"];
    expect(saveMatch).toHaveBeenCalledWith(user, match);
    expect(cache?.cachedMatches).toEqual([]);
    expect(cache?.pendingMatches).toEqual({});
    expect(cache?.sync).toEqual({});
    expect(store.getState().syncStatus).toBe("idle");
  });

  it("clears remote history and the local per-user cache", async () => {
    const clearHistory = vi.fn().mockResolvedValue(1);
    const store = createCloudHistoryStore({
      clearHistory,
      loadHistory: vi.fn().mockResolvedValue([cloudMatch]),
      storage: createMemoryStorage(),
    });

    await store.getState().loadForUser(user);
    await store.getState().clearForUser(user);

    expect(clearHistory).toHaveBeenCalledWith(user);
    expect(store.getState()).toMatchObject({
      errorMessage: null,
      syncStatus: "idle",
      users: {},
    });
  });

  it("waits for active syncs before clearing remote history", async () => {
    const events: string[] = [];
    let resolveSave!: (value: { match: typeof cloudMatch }) => void;
    const saveMatch = vi.fn(
      () =>
        new Promise<{ match: typeof cloudMatch }>((resolve) => {
          resolveSave = resolve;
        }),
    );
    const clearHistory = vi.fn(async () => {
      events.push("clear");
      return 1;
    });
    const store = createCloudHistoryStore({
      clearHistory,
      saveMatch,
      storage: createMemoryStorage(),
    });

    const syncPromise = store.getState().syncMatchForUser(user, match);
    expect(saveMatch).toHaveBeenCalled();
    const clearPromise = store.getState().clearForUser(user);
    expect(clearHistory).not.toHaveBeenCalled();

    events.push("save");
    resolveSave({ match: cloudMatch });
    await syncPromise;
    await clearPromise;

    expect(events).toEqual(["save", "clear"]);
    expect(store.getState().users["uid-1"]).toBeUndefined();
  });

  it("does not clear the local cache when remote history clear fails", async () => {
    const clearHistory = vi.fn().mockRejectedValue(new Error("permission denied"));
    const store = createCloudHistoryStore({
      clearHistory,
      loadHistory: vi.fn().mockResolvedValue([cloudMatch]),
      storage: createMemoryStorage(),
    });

    await store.getState().loadForUser(user);
    await expect(store.getState().clearForUser(user)).rejects.toThrow("permission denied");

    expect(clearHistory).toHaveBeenCalledWith(user);
    expect(store.getState()).toMatchObject({
      errorMessage: "permission denied",
      syncStatus: "error",
    });
    expect(store.getState().users["uid-1"]?.cachedMatches).toEqual([cloudMatch]);
  });
});
