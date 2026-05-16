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

  it("applies local cloud profile patches after background sync", async () => {
    const store = createCloudProfileStore({
      loadProfile: vi.fn().mockResolvedValue(profile),
    });

    await store.getState().loadForUser(authUser, defaultSettings);
    store.getState().applyLocalPatch({
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
