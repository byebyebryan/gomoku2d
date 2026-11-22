import { describe, expect, it, vi } from "vitest";

import { createDefaultProfileSettings } from "../profile/profile_settings";

import type { CloudAuthUser } from "./auth_store";
import { emptyCloudMatchHistory, type CloudProfile } from "./cloud_profile";
import { createCloudProfileStore } from "./cloud_profile_store";

const authUser: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

const defaultSettings = createDefaultProfileSettings();

const profile: CloudProfile = {
  auth: {
    providers: [
      {
        avatarUrl: null,
        displayName: "Bryan",
        provider: "google.com",
      },
    ],
  },
  createdAt: null,
  displayName: "Bryan",
  matchHistory: emptyCloudMatchHistory(),
  resetAt: null,
  settings: defaultSettings,
  uid: "uid-1",
  updatedAt: null,
  username: null,
};

describe("createCloudProfileStore", () => {
  it("returns null for a cancelled load instead of a later account's profile", async () => {
    let finishOld!: (value: CloudProfile) => void;
    const replacement = { ...profile, uid: "uid-2" };
    const store = createCloudProfileStore({
      loadProfile: vi.fn()
        .mockImplementationOnce(() => new Promise((resolve) => { finishOld = resolve; }))
        .mockResolvedValueOnce(replacement),
    });
    const oldLoad = store.getState().loadForUser(authUser, defaultSettings);
    store.getState().reset();
    expect(await store.getState().loadForUser({ ...authUser, uid: "uid-2" }, defaultSettings)).toBe(replacement);
    finishOld(profile);
    expect(await oldLoad).toBeNull();
    expect(store.getState().profile).toBe(replacement);
  });

  it("loads a cloud profile for a signed-in user", async () => {
    const loadProfile = vi.fn().mockResolvedValue(profile);
    const store = createCloudProfileStore({ loadProfile });

    const promise = store.getState().loadForUser(authUser, defaultSettings);
    expect(store.getState().status).toBe("loading");
    await promise;

    expect(loadProfile).toHaveBeenCalledWith(authUser, defaultSettings);
    expect(store.getState()).toMatchObject({
      errorMessage: null,
      profile,
      status: "ready",
    });
  });

  it("surfaces load errors and resets state", async () => {
    const store = createCloudProfileStore({
      loadProfile: vi.fn().mockRejectedValue(new Error("permission denied")),
    });

    await store.getState().loadForUser(authUser, {
      ...defaultSettings,
      gameConfig: {
        opening: "standard",
        ruleset: "renju",
      },
    });
    expect(store.getState()).toMatchObject({
      errorMessage: "permission denied",
      status: "error",
    });

    store.getState().reset();
    expect(store.getState()).toMatchObject({
      errorMessage: null,
      profile: null,
      status: "idle",
    });
  });

  it("rejects patches from a profile snapshot invalidated by reset or a new account", async () => {
    const store = createCloudProfileStore({ loadProfile: vi.fn().mockResolvedValue(profile) });
    await store.getState().loadForUser(authUser, defaultSettings);
    store.getState().reset();
    expect(store.getState().applyLocalPatch(profile, { displayName: "Stale" })).toBe(false);
    const replacement = { ...profile, uid: "other-account" };
    store.setState({ profile: replacement, status: "ready" });
    expect(store.getState().applyLocalPatch(profile, { displayName: "Stale" })).toBe(false);
    expect(store.getState().profile).toBe(replacement);
  });

  it("applies local cloud profile patches after background sync", async () => {
    const store = createCloudProfileStore({
      loadProfile: vi.fn().mockResolvedValue(profile),
    });

    await store.getState().loadForUser(authUser, defaultSettings);
    store.getState().applyLocalPatch(profile, {
      displayName: "ByeByeBryan",
      settings: {
        ...defaultSettings,
        gameConfig: {
          opening: "standard",
          ruleset: "renju",
        },
      },
    });

    expect(store.getState().profile).toMatchObject({
      displayName: "ByeByeBryan",
      settings: {
        gameConfig: {
          ruleset: "renju",
        },
      },
    });
  });

  it("resets a cloud profile for a signed-in user", async () => {
    const resetProfile = vi.fn().mockResolvedValue({
      ...profile,
      resetAt: "2026-04-28T00:00:00.000Z",
    });
    const store = createCloudProfileStore({ resetProfile });

    const promise = store.getState().resetForUser(authUser, defaultSettings);
    expect(store.getState().status).toBe("loading");
    await promise;

    expect(resetProfile).toHaveBeenCalledWith(authUser, defaultSettings);
    expect(store.getState()).toMatchObject({
      errorMessage: null,
      profile: {
        resetAt: "2026-04-28T00:00:00.000Z",
      },
      status: "ready",
    });
  });

  it("rejects reset failures after surfacing the error", async () => {
    const store = createCloudProfileStore({
      resetProfile: vi.fn().mockRejectedValue(new Error("permission denied")),
    });

    await expect(store.getState().resetForUser(authUser, defaultSettings)).rejects.toThrow("permission denied");
    expect(store.getState()).toMatchObject({
      errorMessage: "permission denied",
      status: "error",
    });
  });

  it.each(["reset", "delete"] as const)("rejects a stale %s failure without replacing refreshed state", async (action) => {
    const mutationError = new Error(`${action} permission denied`);
    let rejectMutation!: (error: Error) => void;
    const pendingMutation = () => new Promise<CloudProfile>((_resolve, reject) => {
      rejectMutation = reject;
    });
    const store = createCloudProfileStore({
      deleteProfile: action === "delete" ? vi.fn(() => pendingMutation().then(() => undefined)) : undefined,
      loadProfile: vi.fn().mockResolvedValue(profile),
      resetProfile: action === "reset" ? vi.fn(pendingMutation) : undefined,
    });

    const mutation = action === "reset"
      ? store.getState().resetForUser(authUser, defaultSettings)
      : store.getState().deleteForUser(authUser);
    const rejection = expect(mutation).rejects.toBe(mutationError);

    await expect(store.getState().loadForUser(authUser, defaultSettings)).resolves.toBe(profile);
    rejectMutation(mutationError);

    await rejection;
    expect(store.getState()).toMatchObject({
      errorMessage: null,
      profile,
      status: "ready",
    });
  });

  it("keeps a successful reset barrier ahead of an in-flight same-user refresh", async () => {
    const resetSnapshot: CloudProfile = { ...profile, resetAt: "2026-04-28T00:00:00.000Z" };
    let finishReset!: (value: CloudProfile) => void;
    let finishLoad!: (value: CloudProfile) => void;
    const store = createCloudProfileStore({
      loadProfile: vi.fn(() => new Promise<CloudProfile>((resolve) => { finishLoad = resolve; })),
      resetProfile: vi.fn(() => new Promise<CloudProfile>((resolve) => { finishReset = resolve; })),
    });

    const reset = store.getState().resetForUser(authUser, defaultSettings);
    const refresh = store.getState().loadForUser(authUser, defaultSettings);
    finishReset(resetSnapshot);
    await reset;
    finishLoad(profile);

    await expect(refresh).resolves.toBeNull();
    expect(store.getState()).toMatchObject({
      errorMessage: null,
      profile: resetSnapshot,
      status: "ready",
    });
  });

  it("preserves a strictly newer same-user reset barrier over a stale reset response", async () => {
    const newerSnapshot: CloudProfile = { ...profile, resetAt: "2026-04-29T00:00:00.000Z" };
    const olderSnapshot: CloudProfile = { ...profile, resetAt: "2026-04-28T00:00:00.000Z" };
    let finishReset!: (value: CloudProfile) => void;
    const store = createCloudProfileStore({
      loadProfile: vi.fn().mockResolvedValue(newerSnapshot),
      resetProfile: vi.fn(() => new Promise<CloudProfile>((resolve) => { finishReset = resolve; })),
    });

    const reset = store.getState().resetForUser(authUser, defaultSettings);
    await store.getState().loadForUser(authUser, defaultSettings);
    finishReset(olderSnapshot);
    await reset;

    expect(store.getState()).toMatchObject({ profile: newerSnapshot, status: "ready" });
  });

  it("does not let an older reset supersede a load after a newer mutation starts", async () => {
    const loadedSnapshot: CloudProfile = { ...profile, displayName: "After newer mutation" };
    let finishReset!: (value: CloudProfile) => void;
    let finishDelete!: () => void;
    const store = createCloudProfileStore({
      deleteProfile: vi.fn(() => new Promise<void>((resolve) => { finishDelete = resolve; })),
      loadProfile: vi.fn().mockResolvedValue(loadedSnapshot),
      resetProfile: vi.fn(() => new Promise<CloudProfile>((resolve) => { finishReset = resolve; })),
    });

    const olderReset = store.getState().resetForUser(authUser, defaultSettings);
    const newerDelete = store.getState().deleteForUser(authUser);
    const refresh = store.getState().loadForUser(authUser, defaultSettings);
    await refresh;
    finishReset({ ...profile, resetAt: "2026-04-28T00:00:00.000Z" });
    await olderReset;

    expect(store.getState()).toMatchObject({ profile: loadedSnapshot, status: "ready" });
    finishDelete();
    await newerDelete;
  });

  it("does not carry a stale reset across an A-to-B-to-A account sequence", async () => {
    const nextUser: CloudAuthUser = { ...authUser, uid: "uid-2" };
    const nextProfile: CloudProfile = { ...profile, uid: nextUser.uid };
    const laterOriginalProfile: CloudProfile = { ...profile, displayName: "Later A snapshot" };
    const resetSnapshot: CloudProfile = { ...profile, resetAt: "2026-04-28T00:00:00.000Z" };
    let finishReset!: (value: CloudProfile) => void;
    const store = createCloudProfileStore({
      loadProfile: vi.fn()
        .mockResolvedValueOnce(nextProfile)
        .mockResolvedValueOnce(laterOriginalProfile),
      resetProfile: vi.fn(() => new Promise<CloudProfile>((resolve) => { finishReset = resolve; })),
    });

    const reset = store.getState().resetForUser(authUser, defaultSettings);
    await store.getState().loadForUser(nextUser, defaultSettings);
    await store.getState().loadForUser(authUser, defaultSettings);
    finishReset(resetSnapshot);
    await reset;

    expect(store.getState()).toMatchObject({ profile: laterOriginalProfile, status: "ready" });
  });

  it("deletes a cloud profile for a signed-in user", async () => {
    const deleteProfile = vi.fn().mockResolvedValue(undefined);
    const store = createCloudProfileStore({
      deleteProfile,
      loadProfile: vi.fn().mockResolvedValue(profile),
    });

    await store.getState().loadForUser(authUser, defaultSettings);
    const promise = store.getState().deleteForUser(authUser);
    expect(store.getState().status).toBe("loading");
    await promise;

    expect(deleteProfile).toHaveBeenCalledWith(authUser);
    expect(store.getState()).toMatchObject({
      errorMessage: null,
      profile: null,
      status: "idle",
    });
  });

  it("rejects delete failures after surfacing the error", async () => {
    const store = createCloudProfileStore({
      deleteProfile: vi.fn().mockRejectedValue(new Error("permission denied")),
    });

    await expect(store.getState().deleteForUser(authUser)).rejects.toThrow("permission denied");
    expect(store.getState()).toMatchObject({
      errorMessage: "permission denied",
      status: "error",
    });
  });
});
