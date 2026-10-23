import { describe, expect, it } from "vitest";
import { emptyMatchHistory, matchSummaryForMatch } from "../match/match_history";
import { createLocalSavedMatch, movesFromMoveCells } from "../match/saved_match";
import { playerResultForMatch, playerStatsFromHistory, playerWinRate } from "./player_stats";

const identity = { localProfileId: "local-player", profileUid: "current-user" };

function game(id: string, status: "black_won" | "white_won" | "draw" = "black_won") {
  return createLocalSavedMatch({
    id, localProfileId: identity.localProfileId,
    moves: movesFromMoveCells([110, 0, 111, 1, 112, 2, 113, 3, 114]),
    players: [{ kind: "human", name: "Player", stone: "black" }, { kind: "bot", name: "Normal Bot", stone: "white" }],
    savedAt: "2026-10-22T10:00:00.000Z", status, ruleset: "freestyle",
  });
}

describe("player statistics", () => {
  it("combines deduplicated replay, summary and archived records, including continuations", () => {
    const win = game("win");
    const continuation = { ...game("continuation", "white_won"), undo_floor: 8,
      replay_origin: { source_fingerprint: "A".repeat(43), inherited_ply: 8 } };
    const draw = game("summary-draw", "draw");
    const sourceHistory = emptyMatchHistory();
    sourceHistory.summaryMatches = [
      matchSummaryForMatch(identity, win),
      matchSummaryForMatch(identity, draw),
      matchSummaryForMatch(identity, draw),
    ];
    sourceHistory.archivedStats.totals = { matches: 4, wins: 2, losses: 1, draws: 1, moves: 40 };
    const totals = playerStatsFromHistory({ identity, replayHistory: [win, continuation, win], sourceHistory });
    expect(totals).toEqual({ matches: 7, wins: 3, losses: 2, draws: 2 });
    expect(playerWinRate(totals)).toBe(3 / 7);
  });

  it("uses cloud identity before local identity and supports a human playing White", () => {
    const base = game("cross-device", "white_won");
    const cloudMatch = { ...base, player_black: { ...base.player_white },
      player_white: { ...base.player_black, local_profile_id: null, profile_uid: identity.profileUid } };
    expect(playerResultForMatch(cloudMatch, identity)).toBe("Win");
    expect(playerStatsFromHistory({ identity, replayHistory: [cloudMatch], sourceHistory: null }))
      .toEqual({ matches: 1, wins: 1, losses: 0, draws: 0 });
  });

  it("keeps zero records valid without inventing a win rate", () => {
    const totals = playerStatsFromHistory({ identity, replayHistory: [], sourceHistory: emptyMatchHistory() });
    expect(totals).toEqual({ matches: 0, wins: 0, losses: 0, draws: 0 });
    expect(playerWinRate(totals)).toBeNull();
  });
});
