import type { MatchHistory, MatchSummaryIdentity } from "../match/match_history";
import {
  matchUserSide,
  savedMatchPlayers,
  savedMatchWinningSide,
  type SavedMatchSide,
  type SavedMatchV2,
} from "../match/saved_match";

export interface PlayerStats {
  draws: number;
  losses: number;
  matches: number;
  wins: number;
}

export function playerSideForMatch(match: SavedMatchV2, identity: MatchSummaryIdentity): SavedMatchSide | null {
  return matchUserSide(match, identity)
    ?? savedMatchPlayers(match).find(({ player }) => player.kind === "human")?.side
    ?? null;
}

export function playerResultForMatch(match: SavedMatchV2, identity: MatchSummaryIdentity): "Win" | "Loss" | "Draw" {
  if (match.status === "draw") return "Draw";
  return savedMatchWinningSide(match) === playerSideForMatch(match, identity) ? "Win" : "Loss";
}

/** The same aggregate powers My games and an explicitly shared public snapshot. */
export function playerStatsFromHistory(input: {
  identity: MatchSummaryIdentity;
  replayHistory: SavedMatchV2[];
  sourceHistory: Pick<MatchHistory, "summaryMatches" | "archivedStats"> | null;
}): PlayerStats {
  const totals: PlayerStats = { draws: 0, losses: 0, matches: 0, wins: 0 };
  const replayIds = new Set<string>();
  for (const match of input.replayHistory) {
    if (replayIds.has(match.id)) continue;
    replayIds.add(match.id);
    totals.matches += 1;
    const result = playerResultForMatch(match, input.identity);
    if (result === "Win") totals.wins += 1;
    else if (result === "Loss") totals.losses += 1;
    else totals.draws += 1;
  }
  if (!input.sourceHistory) return totals;
  const countedIds = new Set(replayIds);
  for (const summary of input.sourceHistory.summaryMatches) {
    if (countedIds.has(summary.id)) continue;
    countedIds.add(summary.id);
    totals.matches += 1;
    if (summary.outcome === "win") totals.wins += 1;
    else if (summary.outcome === "loss") totals.losses += 1;
    else totals.draws += 1;
  }
  const archive = input.sourceHistory.archivedStats.totals;
  totals.draws += archive.draws;
  totals.losses += archive.losses;
  totals.matches += archive.matches;
  totals.wins += archive.wins;
  return totals;
}

export function playerWinRate(stats: Pick<PlayerStats, "matches" | "wins">): number | null {
  return stats.matches > 0 ? stats.wins / stats.matches : null;
}
