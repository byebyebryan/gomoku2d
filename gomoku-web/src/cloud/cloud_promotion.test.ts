import { describe, expect, it, vi } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";
import type { GuestProfileIdentity, GuestProfileSettings, GuestSavedMatch } from "../profile/guest_profile_store";

import type { CloudAuthUser } from "./auth_store";
import { cloudProfilePromotionUpdate, promoteGuestToCloud, type CloudPromotionBackend } from "./cloud_promotion";

const user: CloudAuthUser = {
  avatarUrl: "https://example.com/avatar.png",
  displayName: "Google Bryan",
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
  preferredVariant: "renju",
};

const match: GuestSavedMatch = createLocalSavedMatch({
  id: "match-1",
  localProfileId: "guest-1",
  moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
  players: [
    { kind: "human", name: "ByeByeBryan", stone: "black" },
    { kind: "bot", name: "Practice Bot", stone: "white" },
  ],
  savedAt: "2026-04-27T01:02:03.000Z",
  status: "draw",
  variant: "freestyle",
});

function createBackend() {
  const profileUpdates: Array<Record<string, unknown>> = [];
  const backend: CloudPromotionBackend = {
    updateProfile: vi.fn(async (patch) => {
      profileUpdates.push(patch);
    }),
  };

  return { backend, profileUpdates };
}

describe("cloudProfilePromotionUpdate", () => {
  it("promotes a user-chosen local display name", () => {
    expect(
      cloudProfilePromotionUpdate({
        cloudDisplayName: user.displayName,
        cloudHistory: [],
        guestHistory: [],
        guestProfile,
        settings,
        user,
      }),
    ).toMatchObject({
      display_name: "ByeByeBryan",
      preferred_variant: "renju",
      uid: "uid-1",
    });
  });

  it("keeps the provider display name when local profile is still Guest", () => {
    const update = cloudProfilePromotionUpdate({
      cloudHistory: [],
      guestHistory: [],
      guestProfile: { ...guestProfile, displayName: "Guest" },
      settings,
      user,
    });

    expect(update).not.toHaveProperty("display_name");
  });

  it("skips the profile update when loaded cloud fields already match", () => {
    expect(
      cloudProfilePromotionUpdate({
        cloudDisplayName: user.displayName,
        cloudHistory: [],
        cloudPreferredVariant: "renju",
        guestHistory: [],
        guestProfile: { ...guestProfile, displayName: "Guest" },
        settings,
        user,
      }),
    ).toBeNull();
  });

  it("writes only changed cloud fields when the loaded preferred rule is stale", () => {
    const update = cloudProfilePromotionUpdate({
      cloudDisplayName: user.displayName,
      cloudHistory: [],
      cloudPreferredVariant: "freestyle",
      guestHistory: [],
      guestProfile: { ...guestProfile, displayName: "Guest" },
      settings,
      user,
    });

    expect(update).toMatchObject({ preferred_variant: "renju" });
    expect(update).not.toHaveProperty("display_name");
    expect(update).not.toHaveProperty("uid");
  });

  it("does not overwrite a custom cloud display name set from another device", () => {
    const update = cloudProfilePromotionUpdate({
      cloudDisplayName: "AliceFromDeviceA",
      cloudHistory: [],
      guestHistory: [],
      guestProfile,
      settings,
      user,
    });

    expect(update).not.toHaveProperty("display_name");
  });
});

describe("promoteGuestToCloud", () => {
  it("updates the cloud profile with a single embedded history snapshot", async () => {
    const { backend, profileUpdates } = createBackend();

    const result = await promoteGuestToCloud(
      {
        cloudDisplayName: user.displayName,
        cloudHistory: [],
        guestHistory: [match],
        guestProfile,
        settings,
        user,
      },
      { backend },
    );

    expect(profileUpdates).toHaveLength(1);
    expect(profileUpdates[0]).toMatchObject({
      display_name: "ByeByeBryan",
      preferred_variant: "renju",
      recent_matches: {
        matches: [
          expect.objectContaining({
            id: "match-1",
            source: "cloud_saved",
            trust: "client_uploaded",
          }),
        ],
        schema_version: 1,
      },
    });
    expect(result).toEqual({
      localMatchesSynced: 1,
      profileDisplayNamePromoted: true,
      promotedDisplayName: "ByeByeBryan",
    });
  });

  it("does not write when profile and embedded history are already current", async () => {
    const { backend, profileUpdates } = createBackend();
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

    const result = await promoteGuestToCloud(
      {
        cloudDisplayName: user.displayName,
        cloudHistory: [cloudMatch],
        cloudPreferredVariant: "renju",
        guestHistory: [match],
        guestProfile: { ...guestProfile, displayName: "Guest" },
        settings,
        user,
      },
      { backend },
    );

    expect(profileUpdates).toHaveLength(0);
    expect(result).toMatchObject({
      localMatchesSynced: 1,
    });
  });

  it("does not include local matches at or before the reset barrier", async () => {
    const { backend, profileUpdates } = createBackend();

    const result = await promoteGuestToCloud(
      {
        cloudDisplayName: user.displayName,
        cloudHistory: [],
        guestHistory: [match],
        guestProfile,
        historyResetAt: "2026-04-28T00:00:00.000Z",
        settings,
        user,
      },
      { backend },
    );

    expect(profileUpdates[0]).toMatchObject({
      recent_matches: {
        matches: [],
      },
    });
    expect(result).toMatchObject({
      localMatchesSynced: 0,
    });
  });
});
