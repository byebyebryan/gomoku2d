import { describe, expect, it, vi } from "vitest";

import type { CloudAuthUser } from "./auth_store";
import type { CloudProfile } from "./cloud_profile";
import { createCloudProfileStore } from "./cloud_profile_store";

const authUser: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

const profile: CloudProfile = {
  authProviders: ["google.com"],
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  preferredVariant: "freestyle",
  uid: "uid-1",
  username: null,
};

describe("createCloudProfileStore", () => {
  it("loads a cloud profile for a signed-in user", async () => {
    const loadProfile = vi.fn().mockResolvedValue(profile);
    const store = createCloudProfileStore({ loadProfile });

    const promise = store.getState().loadForUser(authUser, "freestyle");
    expect(store.getState().status).toBe("loading");
    await promise;

    expect(loadProfile).toHaveBeenCalledWith(authUser, "freestyle");
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

    await store.getState().loadForUser(authUser, "renju");
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
});
