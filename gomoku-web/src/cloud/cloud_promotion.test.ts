import { describe, expect, it, vi } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";
import { DEFAULT_BOT_CONFIG } from "../core/bot_config";
import { createDefaultProfileSettings } from "../profile/profile_settings";
import {
  emptyLocalMatchHistory,
  type LocalProfileIdentity,
  type LocalProfileSettings,
  type LocalProfileSavedMatch,
} from "../profile/local_profile_store";

import type { CloudAuthUser } from "./auth_store";
import { cloudProfilePromotionUpdate, promoteLocalProfileToCloud, type CloudPromotionBackend } from "./cloud_promotion";
import {
  cloudMatchSummaryForMatch,
  emptyCloudArchivedMatchStats,
  emptyCloudMatchHistory,
  mergeCloudMatchSummaryState,
} from "./cloud_profile";

const user: CloudAuthUser = {
  avatarUrl: "https://example.com/avatar.png",
  displayName: "Google Bryan",
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
  ...createDefaultProfileSettings(),
  gameConfig: {
    opening: "standard",
    ruleset: "renju",
  },
};

const match: LocalProfileSavedMatch = createLocalSavedMatch({
  id: "match-1",
  localProfileId: "local-1",
  moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
  players: [
    { kind: "human", name: "ByeByeBryan", stone: "black" },
    { kind: "bot", name: "Practice Bot", stone: "white" },
  ],
  savedAt: "2026-04-27T01:02:03.000Z",
  status: "draw",
  ruleset: "freestyle",
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

function localMatchHistory(matches: LocalProfileSavedMatch[] = []) {
  return {
    ...emptyLocalMatchHistory(),
    replayMatches: matches,
  };
}

describe("cloudProfilePromotionUpdate", () => {
  it("promotes a user-chosen local display name", () => {
    expect(
      cloudProfilePromotionUpdate({
        cloudDisplayName: user.displayName,
        cloudMatchHistory: emptyCloudMatchHistory(),
        localMatchHistory: localMatchHistory(),
        localProfile,
        settings,
        user,
      }),
    ).toMatchObject({
      display_name: "ByeByeBryan",
      settings: {
        game_config: {
          opening: "standard",
          ruleset: "renju",
        },
        bot_config: DEFAULT_BOT_CONFIG,
      },
      uid: "uid-1",
    });
  });

  it("keeps the provider display name when local profile is still Guest", () => {
    const update = cloudProfilePromotionUpdate({
      cloudMatchHistory: emptyCloudMatchHistory(),
      localMatchHistory: localMatchHistory(),
      localProfile: { ...localProfile, displayName: "Guest" },
      settings,
      user,
    });

    expect(update).not.toHaveProperty("display_name");
  });

  it("skips the profile update when loaded cloud fields already match", () => {
    expect(
      cloudProfilePromotionUpdate({
        cloudDisplayName: user.displayName,
        cloudMatchHistory: emptyCloudMatchHistory(),
        cloudSettings: {
          ...settings,
        },
        localMatchHistory: localMatchHistory(),
        localProfile: { ...localProfile, displayName: "Guest" },
        settings,
        user,
      }),
    ).toBeNull();
  });

  it("writes only changed cloud fields when the loaded preferred rule is stale", () => {
    const update = cloudProfilePromotionUpdate({
      cloudDisplayName: user.displayName,
      cloudMatchHistory: emptyCloudMatchHistory(),
      cloudSettings: {
        ...settings,
        gameConfig: {
          opening: "standard",
          ruleset: "freestyle",
        },
      },
      localMatchHistory: localMatchHistory(),
      localProfile: { ...localProfile, displayName: "Guest" },
      settings,
      user,
    });

    expect(update).toMatchObject({
      settings: {
        game_config: {
          opening: "standard",
          ruleset: "renju",
        },
        bot_config: DEFAULT_BOT_CONFIG,
      },
    });
    expect(update).not.toHaveProperty("display_name");
    expect(update).not.toHaveProperty("uid");
  });

  it("does not overwrite a custom cloud display name set from another device", () => {
    const update = cloudProfilePromotionUpdate({
      cloudDisplayName: "AliceFromDeviceA",
      cloudMatchHistory: emptyCloudMatchHistory(),
      localMatchHistory: localMatchHistory(),
      localProfile,
      settings,
      user,
    });

    expect(update).not.toHaveProperty("display_name");
  });
});

describe("promoteLocalProfileToCloud", () => {
  it("updates the cloud profile with a single embedded history snapshot", async () => {
    const { backend, profileUpdates } = createBackend();

    const result = await promoteLocalProfileToCloud(
      {
        cloudDisplayName: user.displayName,
        cloudMatchHistory: emptyCloudMatchHistory(),
        localMatchHistory: localMatchHistory([match]),
        localProfile,
        settings,
        user,
      },
      { backend },
    );

    expect(profileUpdates).toHaveLength(1);
    expect(profileUpdates[0]).toMatchObject({
      display_name: "ByeByeBryan",
      match_history: {
        replay_matches: [
          expect.objectContaining({
            id: "match-1",
            source: "cloud_saved",
            trust: "client_uploaded",
          }),
        ],
        summary_matches: [],
      },
      settings: {
        game_config: {
          opening: "standard",
          ruleset: "renju",
        },
        bot_config: DEFAULT_BOT_CONFIG,
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
    const recordState = mergeCloudMatchSummaryState({
      archivedStats: emptyCloudArchivedMatchStats(),
      matches: [cloudMatch],
      replayMatches: [cloudMatch],
      summaries: [],
      user,
    });
    const cloudMatchHistory = {
      archivedStats: recordState.archivedStats,
      replayMatches: [cloudMatch],
      summaryMatches: recordState.summaryMatches,
    };

    const result = await promoteLocalProfileToCloud(
      {
        cloudDisplayName: user.displayName,
        cloudMatchHistory,
        cloudSettings: {
          ...settings,
        },
        localMatchHistory: localMatchHistory([match]),
        localProfile: { ...localProfile, displayName: "Guest" },
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

    const result = await promoteLocalProfileToCloud(
      {
        cloudDisplayName: user.displayName,
        cloudMatchHistory: emptyCloudMatchHistory(),
        localMatchHistory: localMatchHistory([match]),
        localProfile,
        resetAt: "2026-04-28T00:00:00.000Z",
        settings,
        user,
      },
      { backend },
    );

    expect(profileUpdates[0]).toMatchObject({
      match_history: {
        replay_matches: [],
        summary_matches: [],
      },
    });
    expect(result).toMatchObject({
      localMatchesSynced: 0,
    });
  });

  it("promotes local summary-tier records when full replay records have already rolled off", async () => {
    const { backend, profileUpdates } = createBackend();
    const summary = cloudMatchSummaryForMatch({ localProfileId: localProfile.id }, match);

    await promoteLocalProfileToCloud(
      {
        cloudDisplayName: user.displayName,
        cloudMatchHistory: emptyCloudMatchHistory(),
        localMatchHistory: {
          ...emptyLocalMatchHistory(),
          summaryMatches: [summary],
        },
        localProfile,
        settings,
        user,
      },
      { backend },
    );

    expect(profileUpdates[0]).toMatchObject({
      match_history: {
        replay_matches: [],
        summary_matches: [
          expect.objectContaining({
            id: "match-1",
            outcome: "draw",
            side: "black",
            trust: "client_uploaded",
          }),
        ],
      },
    });
  });
});
