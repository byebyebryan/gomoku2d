import { describe, expect, it } from "vitest";

import {
  createLocalSavedMatch,
  isLocalSavedMatchV1,
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
  variant: "renju" as const,
};

describe("saved match bot identity", () => {
  it("snapshots the selected practice bot config into new local matches", () => {
    const match = createLocalSavedMatch({
      ...baseInput,
      practiceBot: { mode: "preset", preset: "hard", version: 1 },
    });

    expect(match.player_white.bot).toEqual({
      config: {
        mode: "preset",
        preset: "hard",
        version: 1,
      },
      config_version: 2,
      engine: "search_bot",
      id: "practice_bot",
      label: "Hard",
      lab_spec: "search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4",
      version: 2,
    });
    expect(isLocalSavedMatchV1(match)).toBe(true);
  });

  it("continues to read legacy baseline depth-3 bot identity records", () => {
    const legacyMatch = {
      ...createLocalSavedMatch(baseInput),
      player_white: {
        bot: {
          config: {
            depth: 3,
            kind: "baseline",
          },
          config_version: 1,
          engine: "baseline_search",
          id: "practice_bot",
          version: 1,
        },
        display_name: "Practice Bot",
        kind: "bot",
        local_profile_id: null,
        profile_uid: null,
      },
    };

    expect(isLocalSavedMatchV1(legacyMatch)).toBe(true);
  });

  it("keeps historical practice bot snapshots readable if labels or lab specs change", () => {
    const baseMatch = createLocalSavedMatch({
      ...baseInput,
      practiceBot: { mode: "preset", preset: "hard", version: 1 },
    });
    const historicalMatch = {
      ...baseMatch,
      player_white: {
        ...baseMatch.player_white,
        bot: {
          ...baseMatch.player_white.bot,
          lab_spec: "older-hard-spec",
          label: "Older Hard",
        },
      },
    };

    expect(isLocalSavedMatchV1(historicalMatch)).toBe(true);
  });
});
