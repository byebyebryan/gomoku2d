import { describe, expect, it, vi } from "vitest";

import { DEFAULT_PRACTICE_BOT_CONFIG } from "../core/practice_bot_config";
import type { CloudAuthUser } from "./auth_store";
import { createCloudPromotionStore } from "./cloud_promotion_store";
import {
  emptyLocalMatchHistory,
  type LocalProfileIdentity,
  type LocalProfileSettings,
} from "../profile/local_profile_store";

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

const settings: LocalProfileSettings = {
  practiceBot: DEFAULT_PRACTICE_BOT_CONFIG,
  preferredVariant: "freestyle",
};

describe("createCloudPromotionStore", () => {
  const promotionResult = {
    localMatchesSynced: 0,
    profileDisplayNamePromoted: true,
    promotedDisplayName: "ByeByeBryan",
  };

  it("promotes once for the same input signature", async () => {
    const promoteLocalProfile = vi.fn().mockResolvedValue(promotionResult);
    const store = createCloudPromotionStore({ promoteLocalProfile });
    const input = { localMatchHistory: emptyLocalMatchHistory(), localProfile, settings, user };

    const first = store.getState().promote(input);
    expect(store.getState().status).toBe("promoting");
    await first;

    expect(store.getState()).toMatchObject({
      errorMessage: null,
      status: "complete",
    });

    await store.getState().promote(input);
    expect(promoteLocalProfile).toHaveBeenCalledTimes(1);
  });

  it("reruns promotion when only the loaded cloud display name changes", async () => {
    const promoteLocalProfile = vi.fn().mockResolvedValue(promotionResult);
    const store = createCloudPromotionStore({ promoteLocalProfile });
    const input = { localMatchHistory: emptyLocalMatchHistory(), localProfile, settings, user };

    await store.getState().promote({ ...input, cloudDisplayName: user.displayName });
    await store.getState().promote({ ...input, cloudDisplayName: "Cloud Custom" });

    expect(promoteLocalProfile).toHaveBeenCalledTimes(2);
  });

  it("reruns promotion when only the loaded cloud settings change", async () => {
    const promoteLocalProfile = vi.fn().mockResolvedValue(promotionResult);
    const store = createCloudPromotionStore({ promoteLocalProfile });
    const input = { localMatchHistory: emptyLocalMatchHistory(), localProfile, settings, user };

    await store.getState().promote({
      ...input,
      cloudSettings: {
        defaultRules: {
          opening: "standard",
          ruleset: "freestyle",
        },
        practiceBot: DEFAULT_PRACTICE_BOT_CONFIG,
      },
    });
    await store.getState().promote({
      ...input,
      cloudSettings: {
        defaultRules: {
          opening: "standard",
          ruleset: "renju",
        },
        practiceBot: DEFAULT_PRACTICE_BOT_CONFIG,
      },
    });

    expect(promoteLocalProfile).toHaveBeenCalledTimes(2);
  });

  it("deduplicates identical promotion requests while one is in flight", async () => {
    let resolvePromotion: (result: typeof promotionResult) => void = () => {};
    const promoteLocalProfile = vi.fn(
      () =>
        new Promise<typeof promotionResult>((resolve) => {
          resolvePromotion = resolve;
        }),
    );
    const store = createCloudPromotionStore({ promoteLocalProfile });
    const input = { localMatchHistory: emptyLocalMatchHistory(), localProfile, settings, user };

    const first = store.getState().promote(input);
    const second = store.getState().promote(input);
    expect(promoteLocalProfile).toHaveBeenCalledTimes(1);

    resolvePromotion(promotionResult);
    await Promise.all([first, second]);
    expect(store.getState()).toMatchObject({
      errorMessage: null,
      result: promotionResult,
      status: "complete",
    });
  });

  it("surfaces errors and resets state", async () => {
    const store = createCloudPromotionStore({
      promoteLocalProfile: vi.fn().mockRejectedValue(new Error("permission denied")),
    });

    await store.getState().promote({ localMatchHistory: emptyLocalMatchHistory(), localProfile, settings, user });
    expect(store.getState()).toMatchObject({
      errorMessage: "permission denied",
      result: null,
      status: "error",
    });

    store.getState().reset();
    expect(store.getState()).toMatchObject({
      errorMessage: null,
      result: null,
      status: "idle",
    });
  });
});
