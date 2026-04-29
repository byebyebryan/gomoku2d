import { describe, expect, it } from "vitest";

import type { CloudAuthUser } from "./auth_store";
import {
  CLOUD_PROFILE_SCHEMA_VERSION,
  CLOUD_REPLAY_MATCHES_LIMIT,
  CLOUD_SUMMARY_MATCHES_LIMIT,
  cloudProfileFromDocument,
  cloudProfileSyncDue,
  emptyCloudArchivedMatchStats,
  emptyCloudMatchHistory,
  existingCloudProfileUpdate,
  mergeCloudMatchSummaryState,
  mergeCloudReplayMatches,
  newCloudProfileWrite,
  resetCloudProfileUpdate,
} from "./cloud_profile";
import { createLocalSavedMatch } from "../match/saved_match";

const authUser: CloudAuthUser = {
  avatarUrl: "https://example.com/avatar.png",
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

function emptyMatchHistoryDocument() {
  return {
    archived_stats: emptyCloudArchivedMatchStats(),
    replay_matches: [],
    summary_matches: [],
  };
}

function localMatch(id: string, savedAt: string) {
  return createLocalSavedMatch({
    id,
    localProfileId: "local-1",
    moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
    players: [
      { kind: "human", name: "Bryan", stone: "black" },
      { kind: "bot", name: "Practice Bot", stone: "white" },
    ],
    savedAt,
    status: "draw",
    variant: "freestyle",
  });
}

describe("cloudProfileFromDocument", () => {
  it("maps existing Firestore profile data and preserves app-owned fields", () => {
    expect(
      cloudProfileFromDocument(authUser, "freestyle", {
        auth: {
          providers: [
            {
              avatar_url: "https://example.com/cloud.png",
              display_name: "Google Bryan",
              provider: "google.com",
            },
            {
              avatar_url: null,
              display_name: "GitHub Bryan",
              provider: "github.com",
            },
          ],
        },
        display_name: "ByeByeBryan",
        match_history: emptyMatchHistoryDocument(),
        settings: {
          default_rules: {
            opening: "standard",
            ruleset: "renju",
          },
        },
        username: "byebyebryan",
      }),
    ).toEqual({
      auth: {
        providers: [
          {
            avatarUrl: "https://example.com/cloud.png",
            displayName: "Google Bryan",
            provider: "google.com",
          },
          {
            avatarUrl: null,
            displayName: "GitHub Bryan",
            provider: "github.com",
          },
        ],
      },
      createdAt: null,
      displayName: "ByeByeBryan",
      matchHistory: emptyCloudMatchHistory(),
      resetAt: null,
      settings: {
        defaultRules: {
          opening: "standard",
          ruleset: "renju",
        },
      },
      uid: "uid-1",
      updatedAt: null,
      username: "byebyebryan",
    });
  });

  it("falls back to auth user data for missing or invalid fields", () => {
    expect(
      cloudProfileFromDocument(authUser, "freestyle", {
        display_name: "",
        settings: {
          default_rules: {
            opening: "standard",
            ruleset: "unknown",
          },
        },
      }),
    ).toMatchObject({
      auth: {
        providers: [
          {
            avatarUrl: authUser.avatarUrl,
            displayName: authUser.displayName,
            provider: "google.com",
          },
        ],
      },
      displayName: authUser.displayName,
      matchHistory: emptyCloudMatchHistory(),
      resetAt: null,
      settings: {
        defaultRules: {
          opening: "standard",
          ruleset: "freestyle",
        },
      },
      username: null,
    });
  });

  it("maps Firestore reset timestamps to stable ISO strings", () => {
    expect(
      cloudProfileFromDocument(authUser, "freestyle", {
        reset_at: {
          nanoseconds: 123_000_000,
          seconds: 1_777_363_200,
        },
      }),
    ).toMatchObject({
      resetAt: "2026-04-28T08:00:00.123Z",
    });
  });
});

describe("cloud profile writes", () => {
  it("creates a complete profile document for first sign-in", () => {
    expect(newCloudProfileWrite(authUser, "renju")).toMatchObject({
      auth: {
        providers: [
          {
            avatar_url: authUser.avatarUrl,
            display_name: authUser.displayName,
            provider: "google.com",
          },
        ],
      },
      display_name: "Bryan",
      match_history: emptyMatchHistoryDocument(),
      reset_at: null,
      schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
      settings: {
        default_rules: {
          opening: "standard",
          ruleset: "renju",
        },
      },
      uid: "uid-1",
      username: null,
    });
  });

  it("updates auth-owned fields without overwriting app-owned display name", () => {
    expect(existingCloudProfileUpdate(authUser)).toMatchObject({
      auth: {
        providers: [
          {
            avatar_url: authUser.avatarUrl,
            display_name: authUser.displayName,
            provider: "google.com",
          },
        ],
      },
      match_history: emptyMatchHistoryDocument(),
      reset_at: null,
      schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
      settings: {
        default_rules: {
          opening: "standard",
          ruleset: "freestyle",
        },
      },
      uid: "uid-1",
    });
    expect(existingCloudProfileUpdate(authUser)).not.toHaveProperty("display_name");
    expect(existingCloudProfileUpdate(authUser)).not.toHaveProperty("email");
    expect(existingCloudProfileUpdate(authUser)).not.toHaveProperty("username");
  });

  it("skips existing profile writes when cloud fields are already current", () => {
    expect(
      existingCloudProfileUpdate(authUser, {
        auth: {
          providers: [
            {
              avatar_url: authUser.avatarUrl,
              display_name: authUser.displayName,
              provider: "google.com",
            },
          ],
        },
        match_history: emptyMatchHistoryDocument(),
        reset_at: null,
        schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
        settings: {
          default_rules: {
            opening: "standard",
            ruleset: "freestyle",
          },
        },
        uid: "uid-1",
      }),
    ).toBeNull();
  });

  it("does not sync preferred rule during existing profile load", () => {
    expect(
      existingCloudProfileUpdate(authUser, {
        auth: {
          providers: [
            {
              avatar_url: authUser.avatarUrl,
              display_name: authUser.displayName,
              provider: "google.com",
            },
          ],
        },
        match_history: emptyMatchHistoryDocument(),
        reset_at: null,
        schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
        settings: {
          default_rules: {
            opening: "standard",
            ruleset: "freestyle",
          },
        },
        uid: "uid-1",
      }, "renju"),
    ).toBeNull();
  });

  it("keeps existing profile updates narrow when one auth field changes", () => {
    const update = existingCloudProfileUpdate(authUser, {
      auth: {
        providers: [
          {
            avatar_url: "https://example.com/old.png",
            display_name: authUser.displayName,
            provider: "google.com",
          },
        ],
      },
      match_history: emptyMatchHistoryDocument(),
      reset_at: null,
      schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
      settings: {
        default_rules: {
          opening: "standard",
          ruleset: "freestyle",
        },
      },
      uid: "uid-1",
    });

    expect(update).toMatchObject({
      auth: {
        providers: [
          expect.objectContaining({
            avatar_url: authUser.avatarUrl,
          }),
        ],
      },
    });
    expect(update).not.toHaveProperty("settings");
  });

  it("resets profile-owned fields and writes a history reset barrier", () => {
    expect(resetCloudProfileUpdate(authUser, "freestyle")).toMatchObject({
      auth: {
        providers: [
          {
            avatar_url: authUser.avatarUrl,
            display_name: authUser.displayName,
            provider: "google.com",
          },
        ],
      },
      display_name: authUser.displayName,
      match_history: emptyMatchHistoryDocument(),
      schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
      settings: {
        default_rules: {
          opening: "standard",
          ruleset: "freestyle",
        },
      },
      uid: "uid-1",
    });
    expect(resetCloudProfileUpdate(authUser, "freestyle")).toHaveProperty("reset_at");
    expect(resetCloudProfileUpdate(authUser, "freestyle")).not.toHaveProperty("email");
    expect(resetCloudProfileUpdate(authUser, "freestyle")).not.toHaveProperty("username");
  });

  it("uses a 5-minute sync interval for settled profile snapshots", () => {
    const syncedMatch = mergeCloudReplayMatches(authUser, [
      localMatch("match-sync-test", "2026-04-28T07:59:00.000Z"),
    ])[0]!;
    const profile = {
      createdAt: "2026-04-28T08:00:00.000Z",
      matchHistory: {
        ...emptyCloudMatchHistory(),
        replayMatches: [syncedMatch],
      },
      updatedAt: "2026-04-28T08:00:00.000Z",
    };

    expect(cloudProfileSyncDue(profile, Date.parse("2026-04-28T08:04:59.999Z"))).toBe(false);
    expect(cloudProfileSyncDue(profile, Date.parse("2026-04-28T08:05:00.000Z"))).toBe(true);
  });

  it("merges local matches into a capped cloud replay tier", () => {
    const matches = Array.from({ length: CLOUD_REPLAY_MATCHES_LIMIT + 1 }, (_, index) =>
      localMatch(
        `match-${index}`,
        new Date(Date.parse("2026-04-28T00:00:00.000Z") + index * 1000).toISOString(),
      )
    );

    const merged = mergeCloudReplayMatches(authUser, matches);
    expect(merged).toHaveLength(CLOUD_REPLAY_MATCHES_LIMIT);
    expect(merged[0]?.id).toBe(`match-${CLOUD_REPLAY_MATCHES_LIMIT}`);
    expect(merged[0]?.source).toBe("cloud_saved");
    expect(merged[0]?.player_black.profile_uid).toBe("uid-1");
  });

  it("keeps a longer summary tier and archives evicted stats", () => {
    const matches = Array.from({ length: CLOUD_SUMMARY_MATCHES_LIMIT + 1 }, (_, index) =>
      localMatch(
        `record-${index}`,
        new Date(Date.parse("2026-04-28T00:00:00.000Z") + index * 1000).toISOString(),
      )
    );

    const state = mergeCloudMatchSummaryState({
      archivedStats: emptyCloudArchivedMatchStats(),
      matches,
      replayMatches: [],
      summaries: [],
      user: authUser,
    });

    expect(state.summaryMatches).toHaveLength(CLOUD_SUMMARY_MATCHES_LIMIT);
    expect(state.summaryMatches[0]?.id).toBe(`record-${CLOUD_SUMMARY_MATCHES_LIMIT}`);
    expect(state.summaryMatches[state.summaryMatches.length - 1]?.id).toBe("record-1");
    expect(state.archivedStats.archived_count).toBe(1);
    expect(state.archivedStats.archived_before).toBe("2026-04-28T00:00:00.000Z");
    expect(state.archivedStats.totals).toMatchObject({
      draws: 1,
      matches: 1,
      moves: 1,
    });
    expect(state.archivedStats.by_opponent_type.bot.matches).toBe(1);
    expect(state.archivedStats.by_ruleset.freestyle.matches).toBe(1);
    expect(state.archivedStats.by_side.black.matches).toBe(1);
  });

  it("keeps full replay records out of the summary tier", () => {
    const newerReplayMatch = localMatch("replay-match", "2026-04-28T00:01:00.000Z");
    const olderSummaryMatch = localMatch("summary-match", "2026-04-28T00:00:00.000Z");
    const replayMatches = mergeCloudReplayMatches(authUser, [newerReplayMatch]);

    const state = mergeCloudMatchSummaryState({
      archivedStats: emptyCloudArchivedMatchStats(),
      matches: [newerReplayMatch, olderSummaryMatch],
      replayMatches,
      summaries: [],
      user: authUser,
    });

    expect(state.summaryMatches).toHaveLength(1);
    expect(state.summaryMatches[0]?.id).toBe("summary-match");
  });
});
