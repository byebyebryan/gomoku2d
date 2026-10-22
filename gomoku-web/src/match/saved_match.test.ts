import { describe, expect, it } from "vitest";

import {
  createLocalSavedMatch,
  isLocalSavedMatchV2,
  movesFromMoveCells,
  type CreateLocalSavedMatchInput,
} from "./saved_match";

const baseInput: CreateLocalSavedMatchInput = {
  id: "match-1",
  localProfileId: "local-1",
  moves: [
    { col: 7, moveNumber: 1, player: 1, row: 7 },
    { col: 8, moveNumber: 2, player: 2, row: 7 },
  ],
  players: [
    { kind: "human" as const, name: "Bryan", stone: "black" as const },
    { kind: "bot" as const, name: "Practice Bot", stone: "white" as const },
  ],
  savedAt: "2026-05-15T12:00:00.000Z",
  status: "black_won" as const,
  ruleset: "renju" as const,
};

describe("saved match bot identity", () => {
  it("validates optional continuation origins while retaining older records without one", () => {
    const match = createLocalSavedMatch({
      ...baseInput,
      moves: movesFromMoveCells([110, 0, 111, 1, 112, 2, 113, 3, 114]),
      undoFloor: 8,
      replayOrigin: { source_fingerprint: "A".repeat(43), inherited_ply: 8 },
    });
    expect(isLocalSavedMatchV2(JSON.parse(JSON.stringify(match)))).toBe(true);
    for (const origin of [
      { source_fingerprint: "private-id", inherited_ply: 8 },
      { source_fingerprint: "B".repeat(43), inherited_ply: 8 },
      { source_fingerprint: "A".repeat(43), inherited_ply: 3 },
      { source_fingerprint: "A".repeat(43), inherited_ply: 9 },
      { source_fingerprint: "A".repeat(43), inherited_ply: 6 },
    ]) {
      expect(isLocalSavedMatchV2({ ...match, replay_origin: origin })).toBe(false);
    }
    const { replay_origin: _origin, ...olderMatch } = match;
    expect(isLocalSavedMatchV2(olderMatch)).toBe(true);
  });

  it("snapshots the selected bot config into new local matches", () => {
    const match = createLocalSavedMatch({
      ...baseInput,
      botConfig: { mode: "preset", preset: "hard", version: 1 },
    });

    expect(match.player_white.display_name).toBe("Hard Bot");
    expect(match.player_white.bot).toEqual({
      config: {
        mode: "preset",
        preset: "hard",
        version: 1,
      },
      config_version: 1,
      engine: "search_bot",
      id: "bot",
      label: "Hard",
      lab_spec: "search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4",
      version: 1,
    });
    expect(isLocalSavedMatchV2(match)).toBe(true);
  });

  it("keeps historical bot snapshots readable if labels or lab specs change", () => {
    const baseMatch = createLocalSavedMatch({
      ...baseInput,
      botConfig: { mode: "preset", preset: "hard", version: 1 },
    });
    const historicalMatch = {
      ...baseMatch,
      player_white: {
        ...baseMatch.player_white,
        bot: {
          ...baseMatch.player_white.bot!,
          lab_spec: "older-hard-spec",
          label: "Older Hard",
        },
      },
    };

    expect(isLocalSavedMatchV2(historicalMatch)).toBe(true);
  });
});
