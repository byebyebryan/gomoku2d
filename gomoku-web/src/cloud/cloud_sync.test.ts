import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { CloudAuthUser } from "./auth_store";
import type { CloudProfile } from "./cloud_profile";
import { cloudProfileStore } from "./cloud_profile_store";
import { cloudPromotionStore } from "./cloud_promotion_store";
import { flushCloudProfileSync } from "./cloud_sync";
import type { GuestPromotionInput, GuestPromotionResult } from "./cloud_promotion";
import { guestProfileStore, type GuestProfileIdentity } from "../profile/guest_profile_store";

const user: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

const guestProfile: GuestProfileIdentity = {
  avatarUrl: null,
  createdAt: "2026-04-27T00:00:00.000Z",
  displayName: "ByeByeBryan",
  id: "guest-1",
  kind: "guest",
  updatedAt: "2026-04-27T00:00:00.000Z",
  username: null,
};

const cloudProfile: CloudProfile = {
  authProviders: ["google.com"],
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  historyResetAt: null,
  preferredVariant: "freestyle",
  uid: "uid-1",
  username: null,
};

const promotionResult: GuestPromotionResult = {
  importedMatches: 0,
  profileDisplayNamePromoted: true,
  promotedDisplayName: "ByeByeBryan",
  skippedMatches: 0,
  totalMatches: 0,
};

const initialCloudProfileState = cloudProfileStore.getState();
const initialCloudPromotionState = cloudPromotionStore.getState();
const initialGuestProfileState = guestProfileStore.getState();

describe("flushCloudProfileSync", () => {
  afterEach(() => {
    cloudProfileStore.setState(initialCloudProfileState, true);
    cloudPromotionStore.setState(initialCloudPromotionState, true);
    guestProfileStore.setState(initialGuestProfileState, true);
  });

  beforeEach(() => {
    guestProfileStore.setState({
      history: [],
      profile: guestProfile,
      settings: { preferredVariant: "renju" },
    });
  });

  it("flushes the current local profile/settings to cloud on demand", async () => {
    const promote = vi.fn(async (_input: GuestPromotionInput) => {
      cloudPromotionStore.setState({
        errorMessage: null,
        result: promotionResult,
        status: "complete",
      });
    });
    cloudProfileStore.setState({
      errorMessage: null,
      profile: cloudProfile,
      status: "ready",
    });
    cloudPromotionStore.setState({
      promote,
    });

    const result = await flushCloudProfileSync(user);

    expect(promote).toHaveBeenCalledWith({
      cloudDisplayName: "Bryan",
      cloudPreferredVariant: "freestyle",
      guestHistory: [],
      guestProfile,
      historyResetAt: null,
      settings: { preferredVariant: "renju" },
      user,
    });
    expect(result).toMatchObject({
      displayName: "ByeByeBryan",
      preferredVariant: "renju",
    });
  });

  it("loads the cloud profile before flushing when the cache is cold", async () => {
    const loadForUser = vi.fn(async () => {
      cloudProfileStore.setState({
        errorMessage: null,
        profile: cloudProfile,
        status: "ready",
      });
    });
    const promote = vi.fn(async () => {
      cloudPromotionStore.setState({
        errorMessage: null,
        result: { ...promotionResult, profileDisplayNamePromoted: false, promotedDisplayName: null },
        status: "complete",
      });
    });
    cloudProfileStore.setState({
      errorMessage: null,
      loadForUser,
      profile: null,
      status: "idle",
    });
    cloudPromotionStore.setState({
      promote,
    });

    await flushCloudProfileSync(user);

    expect(loadForUser).toHaveBeenCalledWith(user, "renju");
    expect(promote).toHaveBeenCalledTimes(1);
  });
});
