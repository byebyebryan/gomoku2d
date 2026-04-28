import { describe, expect, it, vi } from "vitest";

import type { CloudAuthUser } from "./auth_store";
import { cloudProfilePromotionUpdate, promoteGuestToCloud, type CloudPromotionBackend } from "./cloud_promotion";
import { createLocalSavedMatch } from "../match/saved_match";
import type { GuestProfileIdentity, GuestProfileSettings, GuestSavedMatch } from "../profile/guest_profile_store";

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

function createBackend(existingMatchIds: string[] = []) {
  const existing = new Set(existingMatchIds);
  const created = new Map<string, unknown>();
  const profileUpdates: Array<Record<string, unknown>> = [];
  const backend: CloudPromotionBackend = {
    createMatch: vi.fn(async (matchId, document) => {
      created.set(matchId, document);
      existing.add(matchId);
    }),
    matchExists: vi.fn(async (matchId) => existing.has(matchId)),
    updateProfile: vi.fn(async (patch) => {
      profileUpdates.push(patch);
    }),
  };

  return { backend, created, profileUpdates };
}

describe("cloudProfilePromotionUpdate", () => {
  it("promotes a user-chosen local display name", () => {
    expect(
      cloudProfilePromotionUpdate({
        cloudDisplayName: user.displayName,
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
      guestHistory: [],
      guestProfile: { ...guestProfile, displayName: "Guest" },
      settings,
      user,
    });

    expect(update).not.toHaveProperty("display_name");
  });

  it("does not promote a custom local display name before the cloud name is loaded", () => {
    const update = cloudProfilePromotionUpdate({
      guestHistory: [],
      guestProfile,
      settings,
      user,
    });

    expect(update).not.toHaveProperty("display_name");
  });

  it("promotes local display name when cloud still holds the provider default", () => {
    const update = cloudProfilePromotionUpdate({
      cloudDisplayName: user.displayName,
      guestHistory: [],
      guestProfile,
      settings,
      user,
    });

    expect(update).toMatchObject({ display_name: "ByeByeBryan" });
  });

  it("does not overwrite a custom cloud display name set from another device", () => {
    const update = cloudProfilePromotionUpdate({
      cloudDisplayName: "AliceFromDeviceA",
      guestHistory: [],
      guestProfile,
      settings,
      user,
    });

    expect(update).not.toHaveProperty("display_name");
  });

  it("promotes when cloud display name is null (fresh account)", () => {
    const update = cloudProfilePromotionUpdate({
      cloudDisplayName: null,
      guestHistory: [],
      guestProfile,
      settings,
      user,
    });

    expect(update).toMatchObject({ display_name: "ByeByeBryan" });
  });
});

describe("promoteGuestToCloud", () => {
  it("updates the cloud profile and imports missing local matches", async () => {
    const { backend, created, profileUpdates } = createBackend();

    const result = await promoteGuestToCloud(
      {
        cloudDisplayName: user.displayName,
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
    });
    expect(backend.matchExists).toHaveBeenCalledWith("local-match-1");
    expect(backend.createMatch).toHaveBeenCalledTimes(1);
    const createdDocument = created.get("local-match-1") as { match_saved_at: { toDate: () => Date } };
    expect(createdDocument).toMatchObject({
      id: "local-match-1",
      local_match_id: "match-1",
      match_kind: "local_vs_bot",
      match_saved_at: expect.anything(),
      player_black: expect.objectContaining({
        local_profile_id: "guest-1",
        profile_uid: "uid-1",
      }),
    });
    expect(createdDocument.match_saved_at.toDate().toISOString()).toBe(match.saved_at);
    expect(result).toEqual({
      importedMatches: 1,
      profileDisplayNamePromoted: true,
      promotedDisplayName: "ByeByeBryan",
      skippedMatches: 0,
      totalMatches: 1,
    });
  });

  it("skips previously imported matches by deterministic doc id", async () => {
    const { backend } = createBackend(["local-match-1"]);

    const result = await promoteGuestToCloud(
      {
        cloudDisplayName: user.displayName,
        guestHistory: [match],
        guestProfile,
        settings,
        user,
      },
      { backend },
    );

    expect(backend.createMatch).not.toHaveBeenCalled();
    expect(result).toMatchObject({
      importedMatches: 0,
      skippedMatches: 1,
      totalMatches: 1,
    });
  });

  it("skips local history that was already saved directly while signed in", async () => {
    const { backend } = createBackend(["match-1"]);

    const result = await promoteGuestToCloud(
      {
        cloudDisplayName: user.displayName,
        guestHistory: [match],
        guestProfile,
        settings,
        user,
      },
      { backend },
    );

    expect(backend.createMatch).not.toHaveBeenCalled();
    expect(result).toMatchObject({
      importedMatches: 0,
      skippedMatches: 1,
      totalMatches: 1,
    });
  });

  it("does not import local matches at or before the reset barrier", async () => {
    const { backend } = createBackend();

    const result = await promoteGuestToCloud(
      {
        cloudDisplayName: user.displayName,
        guestHistory: [match],
        guestProfile,
        historyResetAt: "2026-04-28T00:00:00.000Z",
        settings,
        user,
      },
      { backend },
    );

    expect(backend.matchExists).not.toHaveBeenCalled();
    expect(backend.createMatch).not.toHaveBeenCalled();
    expect(result).toMatchObject({
      importedMatches: 0,
      skippedMatches: 0,
      totalMatches: 0,
    });
  });

  it("treats a raced create as skipped if the match now exists", async () => {
    const { backend } = createBackend();
    vi.mocked(backend.createMatch).mockImplementationOnce(async () => {
      vi.mocked(backend.matchExists).mockResolvedValueOnce(true);
      throw new Error("permission denied");
    });

    const result = await promoteGuestToCloud(
      {
        cloudDisplayName: user.displayName,
        guestHistory: [match],
        guestProfile,
        settings,
        user,
      },
      { backend },
    );

    expect(result).toMatchObject({
      importedMatches: 0,
      skippedMatches: 1,
      totalMatches: 1,
    });
  });
});
