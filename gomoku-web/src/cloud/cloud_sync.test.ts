import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { CloudAuthUser } from "./auth_store";
import { emptyCloudMatchHistory, type CloudProfile } from "./cloud_profile";
import { cloudProfileStore } from "./cloud_profile_store";
import { cloudPromotionStore } from "./cloud_promotion_store";
import { flushCloudProfileSync } from "./cloud_sync";
import type { LocalProfilePromotionInput, LocalProfilePromotionResult } from "./cloud_promotion";
import { promoteLocalProfileToCloud } from "./cloud_promotion";
import { emptyLocalMatchHistory, localProfileStore, type LocalProfileIdentity } from "../profile/local_profile_store";
import { createDefaultProfileSettings } from "../profile/profile_settings";

vi.mock("./cloud_promotion", async (importOriginal) => ({
  ...await importOriginal<typeof import("./cloud_promotion")>(),
  promoteLocalProfileToCloud: vi.fn(),
}));

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

const defaultSettings = createDefaultProfileSettings();
const renjuSettings = {
  ...defaultSettings,
  gameConfig: {
    opening: "standard" as const,
    ruleset: "renju" as const,
  },
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
  settings: defaultSettings,
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
    cloudPromotionStore.getState().reset();
    vi.mocked(promoteLocalProfileToCloud).mockReset();
    localProfileStore.setState({
      matchHistory: emptyLocalMatchHistory(),
      profile: localProfile,
      settings: renjuSettings,
    });
  });

  it("flushes the current local profile/settings to cloud on demand", async () => {
    const promote = vi.fn(async (_input: LocalProfilePromotionInput) => {
      cloudPromotionStore.setState({
        errorMessage: null,
        result: promotionResult,
        status: "complete",
      });
      return promotionResult;
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
      cloudSettings: defaultSettings,
      localMatchHistory: emptyLocalMatchHistory(),
      localProfile,
      resetAt: null,
      settings: renjuSettings,
      user,
    });
    expect(result).toMatchObject({
      displayName: "ByeByeBryan",
      settings: {
        gameConfig: {
          ruleset: "renju",
        },
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
      return cloudProfile;
    });
    const promote = vi.fn(async () => {
      cloudPromotionStore.setState({
        errorMessage: null,
        result: { ...promotionResult, profileDisplayNamePromoted: false, promotedDisplayName: null },
        status: "complete",
      });
      return { ...promotionResult, profileDisplayNamePromoted: false, promotedDisplayName: null };
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

    expect(loadForUser).toHaveBeenCalledWith(user, renjuSettings);
    expect(promote).toHaveBeenCalledTimes(1);
  });

  it.each(["account switch", "profile reset"])("discards an old flush after %s, even when the next promotion completes", async (change) => {
    let finishOld!: (result: LocalProfilePromotionResult) => void;
    vi.mocked(promoteLocalProfileToCloud)
      .mockImplementationOnce(() => new Promise((resolve) => { finishOld = resolve; }))
      .mockResolvedValueOnce(promotionResult);
    cloudProfileStore.setState({ profile: cloudProfile, status: "ready" });
    const oldFlush = flushCloudProfileSync(user);

    cloudPromotionStore.getState().reset();
    const nextUser = change === "account switch" ? { ...user, uid: "uid-2" } : user;
    const nextProfile = {
      ...cloudProfile,
      resetAt: change === "profile reset" ? "2026-10-15T00:00:00.000Z" : null,
      uid: nextUser.uid,
    };
    cloudProfileStore.setState({ profile: nextProfile, status: "ready" });
    localProfileStore.setState({ settings: defaultSettings });
    const current = await flushCloudProfileSync(nextUser);
    expect(current?.settings.gameConfig.ruleset).toBe("freestyle");

    finishOld(promotionResult);
    expect(await oldFlush).toBeNull();
    expect(cloudProfileStore.getState().profile).toBe(current);
    expect(cloudPromotionStore.getState().status).toBe("complete");
  });

  it.each(["sign out", "local reset"])("discards an in-flight flush after %s", async (change) => {
    let finish!: (result: LocalProfilePromotionResult) => void;
    vi.mocked(promoteLocalProfileToCloud).mockImplementationOnce(() => new Promise((resolve) => { finish = resolve; }));
    cloudProfileStore.setState({ profile: cloudProfile, status: "ready" });
    const flush = flushCloudProfileSync(user);
    if (change === "sign out") {
      cloudProfileStore.getState().reset();
      cloudPromotionStore.getState().reset();
    } else {
      localProfileStore.getState().resetLocalProfile();
    }
    const current = cloudProfileStore.getState().profile;
    finish(promotionResult);
    expect(await flush).toBeNull();
    expect(cloudProfileStore.getState().profile).toBe(current);
  });

  it("keeps the newest settings when same-account flushes finish out of order", async () => {
    let finishOld!: (result: LocalProfilePromotionResult) => void;
    vi.mocked(promoteLocalProfileToCloud)
      .mockImplementationOnce(() => new Promise((resolve) => { finishOld = resolve; }))
      .mockResolvedValueOnce(promotionResult);
    cloudProfileStore.setState({ profile: cloudProfile, status: "ready" });
    const oldFlush = flushCloudProfileSync(user);
    localProfileStore.setState({ settings: defaultSettings });
    const current = await flushCloudProfileSync(user);
    finishOld(promotionResult);
    expect(await oldFlush).toBeNull();
    expect(cloudProfileStore.getState().profile).toBe(current);
    expect(current?.settings.gameConfig.ruleset).toBe("freestyle");
  });
});
