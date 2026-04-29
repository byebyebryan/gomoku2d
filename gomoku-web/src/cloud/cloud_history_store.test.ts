import { describe, expect, it, vi } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";
import type { LocalProfileStorage } from "../profile/local_profile_store";

import type { CloudAuthUser } from "./auth_store";
import { emptyCloudMatchHistory, type CloudProfile } from "./cloud_profile";
import { createCloudHistoryStore } from "./cloud_history_store";

function createMemoryStorage(): LocalProfileStorage {
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
  localProfileId: "local-1",
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

function cloudProfile(overrides: Partial<CloudProfile> = {}): CloudProfile {
  return {
    auth: {
      providers: [
        {
          avatarUrl: null,
          displayName: "Bryan",
          provider: "google.com",
        },
      ],
    },
    createdAt: "2026-04-28T00:00:00.000Z",
    displayName: "Bryan",
    matchHistory: {
      ...emptyCloudMatchHistory(),
      replayMatches: [cloudMatch],
    },
    resetAt: null,
    settings: {
      defaultRules: {
        opening: "standard",
        ruleset: "freestyle",
      },
    },
    uid: "uid-1",
    updatedAt: "2026-04-28T00:00:00.000Z",
    username: null,
    ...overrides,
  };
}

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

  it("loads directly from a cloud profile snapshot", () => {
    const store = createCloudHistoryStore({
      now: () => "2026-04-28T02:00:00.000Z",
      storage: createMemoryStorage(),
    });

    store.getState().loadFromProfile(user, cloudProfile());

    expect(store.getState().users["uid-1"]).toMatchObject({
      cachedMatches: [cloudMatch],
      loadedAt: "2026-04-28T02:00:00.000Z",
    });
  });

  it("keeps a local match pending when the 5-minute profile sync gate is closed", async () => {
    const saveHistory = vi.fn().mockResolvedValue({ matches: [cloudMatch], profile: cloudProfile() });
    const store = createCloudHistoryStore({
      cloudProfileForUser: () => cloudProfile({
        matchHistory: emptyCloudMatchHistory(),
        updatedAt: "2999-01-01T00:00:00.000Z",
      }),
      saveHistory,
      storage: createMemoryStorage(),
    });

    await store.getState().syncMatchForUser(user, match);

    expect(saveHistory).not.toHaveBeenCalled();
    expect(store.getState().users["uid-1"]?.pendingMatches["match-1"]).toEqual(match);
  });

  it("refreshes a stale due profile before writing queued history", async () => {
    const staleProfile = cloudProfile({
      matchHistory: emptyCloudMatchHistory(),
      updatedAt: "2026-04-28T00:00:00.000Z",
    });
    const refreshedProfile = cloudProfile({
      matchHistory: emptyCloudMatchHistory(),
      updatedAt: "2999-01-01T00:00:00.000Z",
    });
    const refreshCloudProfileForUser = vi.fn().mockResolvedValue(refreshedProfile);
    const saveHistory = vi.fn().mockResolvedValue({ matches: [cloudMatch], profile: refreshedProfile });
    const store = createCloudHistoryStore({
      cloudProfileForUser: () => staleProfile,
      refreshCloudProfileForUser,
      saveHistory,
      storage: createMemoryStorage(),
    });

    await store.getState().syncMatchForUser(user, match);

    expect(refreshCloudProfileForUser).toHaveBeenCalledWith(user, "freestyle");
    expect(saveHistory).not.toHaveBeenCalled();
    expect(store.getState()).toMatchObject({
      errorMessage: null,
      syncStatus: "idle",
    });
    expect(store.getState().users["uid-1"]?.pendingMatches["match-1"]).toEqual(match);
  });

  it("clears a queued match when the refreshed profile already contains it", async () => {
    const staleProfile = cloudProfile({
      matchHistory: emptyCloudMatchHistory(),
      updatedAt: "2026-04-28T00:00:00.000Z",
    });
    const refreshedProfile = cloudProfile({ updatedAt: "2999-01-01T00:00:00.000Z" });
    const refreshCloudProfileForUser = vi.fn().mockResolvedValue(refreshedProfile);
    const saveHistory = vi.fn().mockResolvedValue({ matches: [cloudMatch], profile: refreshedProfile });
    const store = createCloudHistoryStore({
      cloudProfileForUser: () => staleProfile,
      refreshCloudProfileForUser,
      saveHistory,
      storage: createMemoryStorage(),
    });

    await store.getState().syncMatchForUser(user, match);

    expect(refreshCloudProfileForUser).toHaveBeenCalledWith(user, "freestyle");
    expect(saveHistory).not.toHaveBeenCalled();
    expect(store.getState()).toMatchObject({
      errorMessage: null,
      syncStatus: "idle",
    });
    expect(store.getState().users["uid-1"]?.pendingMatches).toEqual({});
    expect(store.getState().users["uid-1"]?.sync).toEqual({});
  });

  it("writes one merged profile snapshot when the sync gate is open", async () => {
    const profile = cloudProfile({ matchHistory: emptyCloudMatchHistory() });
    const saveHistory = vi.fn().mockResolvedValue({ matches: [cloudMatch], profile });
    const store = createCloudHistoryStore({
      cloudProfileForUser: () => profile,
      saveHistory,
      storage: createMemoryStorage(),
    });

    await store.getState().syncMatchForUser(user, match);

    expect(saveHistory).toHaveBeenCalledTimes(1);
    expect(saveHistory.mock.calls[0]?.[1].matches).toEqual([cloudMatch]);
    expect(store.getState().users["uid-1"]?.pendingMatches).toEqual({});
  });

  it("keeps failed snapshot sync records pending for retry", async () => {
    const profile = cloudProfile({ matchHistory: emptyCloudMatchHistory() });
    const saveHistory = vi.fn().mockRejectedValue(new Error("offline"));
    const store = createCloudHistoryStore({
      cloudProfileForUser: () => profile,
      saveHistory,
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

  it("clears stale pending errors when a loaded cloud profile already has the match", async () => {
    const profile = cloudProfile({ matchHistory: emptyCloudMatchHistory() });
    const saveHistory = vi.fn().mockRejectedValue(new Error("offline"));
    const store = createCloudHistoryStore({
      cloudProfileForUser: () => profile,
      saveHistory,
      storage: createMemoryStorage(),
    });

    await store.getState().syncMatchForUser(user, match);
    expect(store.getState().users["uid-1"]?.sync["match-1"]).toMatchObject({
      errorMessage: "offline",
      status: "error",
    });

    store.getState().loadFromProfile(user, cloudProfile());

    expect(store.getState()).toMatchObject({
      errorMessage: null,
      loadStatus: "ready",
      syncStatus: "idle",
    });
    expect(store.getState().users["uid-1"]?.cachedMatches).toEqual([cloudMatch]);
    expect(store.getState().users["uid-1"]?.pendingMatches).toEqual({});
    expect(store.getState().users["uid-1"]?.sync).toEqual({});
  });

  it("drops pending records older than the reset barrier without syncing them", async () => {
    const saveHistory = vi.fn().mockResolvedValue({ matches: [cloudMatch], profile: cloudProfile() });
    const store = createCloudHistoryStore({
      cloudProfileForUser: () => cloudProfile(),
      saveHistory,
      storage: createMemoryStorage(),
    });

    await store.getState().syncMatchForUser(user, match, "2026-04-28T02:00:00.000Z");

    expect(saveHistory).not.toHaveBeenCalled();
    expect(store.getState().users["uid-1"]?.pendingMatches).toEqual({});
  });

  it("clears only the local per-user cache after profile reset", async () => {
    const store = createCloudHistoryStore({
      loadHistory: vi.fn().mockResolvedValue([cloudMatch]),
      storage: createMemoryStorage(),
    });

    await store.getState().loadForUser(user);
    await store.getState().clearForUser(user);

    expect(store.getState()).toMatchObject({
      errorMessage: null,
      syncStatus: "idle",
      users: {},
    });
  });
});
