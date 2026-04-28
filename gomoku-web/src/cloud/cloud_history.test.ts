import { describe, expect, it, vi } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";

import type { CloudAuthUser } from "./auth_store";
import { loadCloudHistory, saveCloudMatch, type CloudHistoryBackend } from "./cloud_history";

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

function createBackend(existingMatchIds: string[] = []) {
  const existing = new Set(existingMatchIds);
  const created = new Map<string, unknown>();
  const backend: CloudHistoryBackend = {
    createMatch: vi.fn(async (matchId, document) => {
      created.set(matchId, document);
      existing.add(matchId);
    }),
    loadMatches: vi.fn(async () => Array.from(created.values())),
    matchExists: vi.fn(async (matchId) => existing.has(matchId)),
  };

  return { backend, created, existing };
}

describe("cloud history", () => {
  it("saves a finished local match as cloud_saved", async () => {
    const { backend, created } = createBackend();

    const result = await saveCloudMatch(user, match, { backend });

    expect(result).toMatchObject({
      matchId: "match-1",
      skipped: false,
    });
    expect(result.match).toMatchObject({
      id: "match-1",
      player_black: {
        local_profile_id: null,
        profile_uid: "uid-1",
      },
      source: "cloud_saved",
      trust: "client_uploaded",
    });
    expect(backend.createMatch).toHaveBeenCalledTimes(1);
    expect(created.get("match-1")).toMatchObject({
      id: "match-1",
      source: "cloud_saved",
      trust: "client_uploaded",
    });
  });

  it("treats an existing direct cloud match as an idempotent save", async () => {
    const { backend } = createBackend(["match-1"]);

    const result = await saveCloudMatch(user, match, { backend });

    expect(result).toMatchObject({
      matchId: "match-1",
      skipped: true,
    });
    expect(backend.createMatch).not.toHaveBeenCalled();
  });

  it("treats a raced create as saved when the document now exists", async () => {
    const { backend, existing } = createBackend();
    vi.mocked(backend.createMatch).mockImplementationOnce(async (matchId) => {
      existing.add(matchId);
      throw new Error("permission denied");
    });

    const result = await saveCloudMatch(user, match, { backend });

    expect(result).toMatchObject({
      matchId: "match-1",
      skipped: true,
    });
  });

  it("loads only valid saved match documents", async () => {
    const { backend } = createBackend();
    vi.mocked(backend.loadMatches).mockResolvedValueOnce([
      {
        ...match,
        player_black: {
          ...match.player_black,
          local_profile_id: null,
          profile_uid: "uid-1",
        },
        source: "cloud_saved",
        trust: "client_uploaded",
      },
      { id: "bad-match" },
    ]);

    const history = await loadCloudHistory(user, { backend, limitCount: 10 });

    expect(backend.loadMatches).toHaveBeenCalledWith(10);
    expect(history).toHaveLength(1);
    expect(history[0]?.id).toBe("match-1");
  });
});
