import { describe, expect, it, vi } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";

import type { CloudAuthUser } from "./auth_store";
import {
  clearCloudHistory,
  cloudHistoryFromProfile,
  saveCloudHistorySnapshot,
  type CloudHistoryBackend,
} from "./cloud_history";
import type { CloudProfile } from "./cloud_profile";

const user: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

const match = createLocalSavedMatch({
  id: "match-1",
  localProfileId: "guest-1",
  moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
  players: [
    { kind: "human", name: "Bryan", stone: "black" },
    { kind: "bot", name: "Practice Bot", stone: "white" },
  ],
  savedAt: "2026-04-28T01:02:03.000Z",
  status: "draw",
  variant: "freestyle",
});

function cloudProfile(overrides: Partial<CloudProfile> = {}): CloudProfile {
  return {
    authProviders: ["google.com"],
    avatarUrl: null,
    createdAt: "2026-04-28T00:00:00.000Z",
    displayName: "Bryan",
    email: "bryan@example.com",
    historyResetAt: null,
    preferredVariant: "freestyle",
    recentMatches: {
      matches: [],
      schemaVersion: 1,
      updatedAt: null,
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
      historyResetAt: "2026-04-28T00:00:00.000Z",
      recentMatches: {
        matches: [
          { ...match, source: "cloud_saved", trust: "client_uploaded" },
          {
            ...match,
            id: "old-match",
            saved_at: "2026-04-27T01:02:03.000Z",
            source: "cloud_saved",
            trust: "client_uploaded",
          },
        ],
        schemaVersion: 1,
        updatedAt: null,
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
    expect(result.matches).toHaveLength(1);
    expect(result.matches[0]?.player_black.profile_uid).toBe("uid-1");
  });

  it("clears embedded cloud history with one profile snapshot write", async () => {
    const { backend, updates } = createBackend();
    const profile = cloudProfile({
      recentMatches: {
        matches: [{ ...match, source: "cloud_saved", trust: "client_uploaded" }],
        schemaVersion: 1,
        updatedAt: null,
      },
    });

    const result = await clearCloudHistory(
      user,
      {
        cloudProfile: profile,
        displayName: "Bryan",
        preferredVariant: "freestyle",
      },
      { backend },
    );

    expect(backend.updateProfile).toHaveBeenCalledTimes(1);
    expect(updates[0]).toMatchObject({
      recent_matches: {
        matches: [],
      },
    });
    expect(result.matches).toEqual([]);
  });
});
