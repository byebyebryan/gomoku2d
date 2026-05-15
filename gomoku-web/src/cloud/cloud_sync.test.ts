import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { CloudAuthUser } from "./auth_store";
import { DEFAULT_PRACTICE_BOT_CONFIG } from "../core/practice_bot_config";
import { emptyCloudMatchHistory, type CloudProfile } from "./cloud_profile";
import { cloudProfileStore } from "./cloud_profile_store";
import { cloudPromotionStore } from "./cloud_promotion_store";
import { flushCloudProfileSync } from "./cloud_sync";
import type { LocalProfilePromotionInput, LocalProfilePromotionResult } from "./cloud_promotion";
import { emptyLocalMatchHistory, localProfileStore, type LocalProfileIdentity } from "../profile/local_profile_store";

const user: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

const localProfile: LocalProfileIdentity = {
  avatarUrl: null,
  createdAt: "2026-04-27T00:00:00.000Z",
  displayName: "ByeByeBryan",
  id: "local-1",
  kind: "local",
  updatedAt: "2026-04-27T00:00:00.000Z",
  username: null,
};

const cloudProfile: CloudProfile = {
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
  settings: {
    defaultRules: {
      opening: "standard",
      ruleset: "freestyle",
    },
    practiceBot: DEFAULT_PRACTICE_BOT_CONFIG,
  },
  uid: "uid-1",
  updatedAt: null,
  username: null,
};

const promotionResult: LocalProfilePromotionResult = {
  localMatchesSynced: 0,
  profileDisplayNamePromoted: true,
  promotedDisplayName: "ByeByeBryan",
};

const initialCloudProfileState = cloudProfileStore.getState();
const initialCloudPromotionState = cloudPromotionStore.getState();
const initialLocalProfileState = localProfileStore.getState();

describe("flushCloudProfileSync", () => {
  afterEach(() => {
    cloudProfileStore.setState(initialCloudProfileState, true);
    cloudPromotionStore.setState(initialCloudPromotionState, true);
    localProfileStore.setState(initialLocalProfileState, true);
  });

  beforeEach(() => {
    localProfileStore.setState({
      matchHistory: emptyLocalMatchHistory(),
      profile: localProfile,
      settings: { practiceBot: DEFAULT_PRACTICE_BOT_CONFIG, preferredVariant: "renju" },
    });
  });

  it("flushes the current local profile/settings to cloud on demand", async () => {
    const promote = vi.fn(async (_input: LocalProfilePromotionInput) => {
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
      cloudMatchHistory: emptyCloudMatchHistory(),
      cloudSettings: {
        defaultRules: {
          opening: "standard",
          ruleset: "freestyle",
        },
        practiceBot: DEFAULT_PRACTICE_BOT_CONFIG,
      },
      localMatchHistory: emptyLocalMatchHistory(),
      localProfile,
      resetAt: null,
      settings: { practiceBot: DEFAULT_PRACTICE_BOT_CONFIG, preferredVariant: "renju" },
      user,
    });
    expect(result).toMatchObject({
      displayName: "ByeByeBryan",
      settings: {
        defaultRules: {
          ruleset: "renju",
        },
        practiceBot: DEFAULT_PRACTICE_BOT_CONFIG,
      },
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
