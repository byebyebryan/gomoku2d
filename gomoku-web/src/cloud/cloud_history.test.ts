import { describe, expect, it, vi } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";
import { DEFAULT_PRACTICE_BOT_CONFIG } from "../core/practice_bot_config";

import type { CloudAuthUser } from "./auth_store";
import {
  cloudHistoryFromProfile,
  saveCloudHistorySnapshot,
  type CloudHistoryBackend,
} from "./cloud_history";
import {
  CLOUD_REPLAY_MATCHES_LIMIT,
  emptyCloudArchivedMatchStats,
  emptyCloudMatchHistory,
  type CloudProfile,
} from "./cloud_profile";

const user: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

const match = createLocalSavedMatch({
  id: "match-1",
  localProfileId: "local-1",
  moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
  players: [
    { kind: "human", name: "Bryan", stone: "black" },
    { kind: "bot", name: "Practice Bot", stone: "white" },
  ],
  savedAt: "2026-04-28T01:02:03.000Z",
  status: "draw",
  variant: "freestyle",
});

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

function cloudProfile(overrides: Partial<CloudProfile> = {}): CloudProfile {
  return {
    auth: {
      providers: [
        {
          avatarUrl: null,
          displayName: "Bryan",
          provider: "google.com",
        },
      ],
    },
    createdAt: "2026-04-28T00:00:00.000Z",
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
    updatedAt: "2026-04-28T00:00:00.000Z",
    username: null,
    ...overrides,
  };
}

function createBackend() {
  const updates: Array<Record<string, unknown>> = [];
  const backend: CloudHistoryBackend = {
    loadProfile: vi.fn(async () => null),
    updateProfile: vi.fn(async (patch) => {
      updates.push(patch);
    }),
  };

  return { backend, updates };
}

describe("cloud history", () => {
  it("filters embedded cloud history after the reset barrier", () => {
    const profile = cloudProfile({
      resetAt: "2026-04-28T00:00:00.000Z",
      matchHistory: {
        ...emptyCloudMatchHistory(),
        replayMatches: [
          { ...match, source: "cloud_saved", trust: "client_uploaded" },
          {
            ...match,
            id: "old-match",
            saved_at: "2026-04-27T01:02:03.000Z",
            source: "cloud_saved",
            trust: "client_uploaded",
          },
        ],
      },
    });

    expect(cloudHistoryFromProfile(profile)).toHaveLength(1);
    expect(cloudHistoryFromProfile(profile)[0]?.id).toBe("match-1");
  });

  it("writes one profile snapshot for a merged history save", async () => {
    const { backend, updates } = createBackend();
    const profile = cloudProfile();

    const result = await saveCloudHistorySnapshot(
      user,
      {
        cloudProfile: profile,
        displayName: "Bryan",
        matches: [match],
        preferredVariant: "renju",
      },
      { backend },
    );

    expect(backend.updateProfile).toHaveBeenCalledTimes(1);
    expect(updates[0]).toMatchObject({
      display_name: "Bryan",
      match_history: {
        archived_stats: emptyCloudArchivedMatchStats(),
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
        default_rules: {
          opening: "standard",
          ruleset: "renju",
        },
      },
    });
    expect(result.matches).toHaveLength(1);
    expect(result.matches[0]?.player_black.profile_uid).toBe("uid-1");
  });

  it("moves replay overflow into the summary tier", async () => {
    const { backend, updates } = createBackend();
    const matches = Array.from({ length: CLOUD_REPLAY_MATCHES_LIMIT + 1 }, (_, index) =>
      localMatch(
        `match-${index}`,
        new Date(Date.parse("2026-04-28T00:00:00.000Z") + index * 1000).toISOString(),
      )
    );

    await saveCloudHistorySnapshot(
      user,
      {
        cloudProfile: cloudProfile(),
        displayName: "Bryan",
        matches,
        preferredVariant: "freestyle",
      },
      { backend },
    );

    expect(updates[0]).toMatchObject({
      match_history: {
        replay_matches: expect.arrayContaining([
          expect.objectContaining({ id: `match-${CLOUD_REPLAY_MATCHES_LIMIT}` }),
        ]),
        summary_matches: [
          expect.objectContaining({
            id: "match-0",
          }),
        ],
      },
    });
  });

  it("clears embedded cloud history with one profile snapshot write", async () => {
    const { backend, updates } = createBackend();
    const profile = cloudProfile({
      matchHistory: {
        ...emptyCloudMatchHistory(),
        replayMatches: [{ ...match, source: "cloud_saved", trust: "client_uploaded" }],
      },
    });

    const result = await saveCloudHistorySnapshot(
      user,
      {
        cloudProfile: profile,
        displayName: "Bryan",
        matches: [],
        preferredVariant: "freestyle",
      },
      { backend },
    );

    expect(backend.updateProfile).toHaveBeenCalledTimes(1);
    expect(updates[0]).toMatchObject({
      match_history: {
        replay_matches: [],
        summary_matches: [],
      },
    });
    expect(result.matches).toEqual([]);
  });
});
