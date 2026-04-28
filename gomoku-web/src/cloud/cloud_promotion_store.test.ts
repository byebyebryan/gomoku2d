import { describe, expect, it, vi } from "vitest";

import type { CloudAuthUser } from "./auth_store";
import { createCloudPromotionStore } from "./cloud_promotion_store";
import type { GuestProfileIdentity, GuestProfileSettings } from "../profile/guest_profile_store";

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

const settings: GuestProfileSettings = {
  preferredVariant: "freestyle",
};

describe("createCloudPromotionStore", () => {
  const promotionResult = {
    importedMatches: 0,
    profileDisplayNamePromoted: true,
    promotedDisplayName: "ByeByeBryan",
    skippedMatches: 0,
    totalMatches: 0,
  };

  it("promotes once for the same input signature", async () => {
    const promoteGuest = vi.fn().mockResolvedValue(promotionResult);
    const store = createCloudPromotionStore({ promoteGuest });
    const input = { guestHistory: [], guestProfile, settings, user };

    const first = store.getState().promote(input);
    expect(store.getState().status).toBe("promoting");
    await first;

    expect(store.getState()).toMatchObject({
      errorMessage: null,
      status: "complete",
    });

    await store.getState().promote(input);
    expect(promoteGuest).toHaveBeenCalledTimes(1);
  });

  it("deduplicates identical promotion requests while one is in flight", async () => {
    let resolvePromotion: (result: typeof promotionResult) => void = () => {};
    const promoteGuest = vi.fn(
      () =>
        new Promise<typeof promotionResult>((resolve) => {
          resolvePromotion = resolve;
        }),
    );
    const store = createCloudPromotionStore({ promoteGuest });
    const input = { guestHistory: [], guestProfile, settings, user };

    const first = store.getState().promote(input);
    const second = store.getState().promote(input);
    expect(promoteGuest).toHaveBeenCalledTimes(1);

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
      promoteGuest: vi.fn().mockRejectedValue(new Error("permission denied")),
    });

    await store.getState().promote({ guestHistory: [], guestProfile, settings, user });
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
