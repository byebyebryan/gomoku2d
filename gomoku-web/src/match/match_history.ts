import type { GameVariant } from "../core/bot_protocol";
import { DEFAULT_RULE_OPENING } from "../profile/profile_settings";
import {
  savedMatchIsAfterReset,
  savedMatchPlayers,
  savedMatchWinningSide,
  type SavedMatchPlayer,
  type SavedMatchSide,
  type SavedMatchV2,
} from "./saved_match";

export const REPLAY_MATCHES_LIMIT = 128;
export const SUMMARY_MATCHES_LIMIT = 1024;
export const MATCH_SUMMARY_SCHEMA_VERSION = 1;
export const ARCHIVED_MATCH_STATS_SCHEMA_VERSION = 1;

export type MatchSummaryOutcome = "draw" | "loss" | "win";
export type MatchSummaryOpponentKind = "bot" | "human";

export interface MatchSummaryOpponent {
  bot_key: string | null;
  display_name: string;
  kind: MatchSummaryOpponentKind;
  profile_uid: string | null;
}

export interface MatchSummaryV1 {
  id: string;
  match_kind: SavedMatchV2["match_kind"];
  move_count: number;
  opening: typeof DEFAULT_RULE_OPENING;
  opponent: MatchSummaryOpponent;
  outcome: MatchSummaryOutcome;
  ruleset: GameVariant;
  saved_at: string;
  schema_version: typeof MATCH_SUMMARY_SCHEMA_VERSION;
  side: SavedMatchSide;
  trust: SavedMatchV2["trust"];
}

export interface MatchStatsCounter {
  draws: number;
  losses: number;
  matches: number;
  moves: number;
  wins: number;
}

export interface ArchivedMatchStatsV1 {
  archived_before: string | null;
  archived_count: number;
  by_opponent_type: Record<MatchSummaryOpponentKind, MatchStatsCounter>;
  by_ruleset: Record<GameVariant, MatchStatsCounter>;
  by_side: Record<SavedMatchSide, MatchStatsCounter>;
  schema_version: typeof ARCHIVED_MATCH_STATS_SCHEMA_VERSION;
  totals: MatchStatsCounter;
}

export interface MatchHistory {
  archivedStats: ArchivedMatchStatsV1;
  replayMatches: SavedMatchV2[];
  summaryMatches: MatchSummaryV1[];
}

export interface MatchSummaryIdentity {
  localProfileId?: string | null;
  profileUid?: string | null;
}

function validVariant(value: unknown): GameVariant | null {
  return value === "freestyle" || value === "renju" ? value : null;
}

function stringOrNull(value: unknown): string | null {
  return typeof value === "string" && value.trim() ? value : null;
}

function isNullableString(value: unknown): value is string | null {
  return value === null || typeof value === "string";
}

function emptyStatsCounter(): MatchStatsCounter {
  return {
    draws: 0,
    losses: 0,
    matches: 0,
    moves: 0,
    wins: 0,
  };
}

function emptyStatsByRuleset(): Record<GameVariant, MatchStatsCounter> {
  return {
    freestyle: emptyStatsCounter(),
    renju: emptyStatsCounter(),
  };
}

function emptyStatsBySide(): Record<SavedMatchSide, MatchStatsCounter> {
  return {
    black: emptyStatsCounter(),
    white: emptyStatsCounter(),
  };
}

function emptyStatsByOpponentType(): Record<MatchSummaryOpponentKind, MatchStatsCounter> {
  return {
    bot: emptyStatsCounter(),
    human: emptyStatsCounter(),
  };
}

export function emptyArchivedMatchStats(): ArchivedMatchStatsV1 {
  return {
    archived_before: null,
    archived_count: 0,
    by_opponent_type: emptyStatsByOpponentType(),
    by_ruleset: emptyStatsByRuleset(),
    by_side: emptyStatsBySide(),
    schema_version: ARCHIVED_MATCH_STATS_SCHEMA_VERSION,
    totals: emptyStatsCounter(),
  };
}

function counterFromDocument(value: unknown): MatchStatsCounter | null {
  const candidate = value as Partial<MatchStatsCounter> | null;
  if (
    candidate === null
    || typeof candidate !== "object"
    || !Number.isFinite(candidate.draws)
    || !Number.isFinite(candidate.losses)
    || !Number.isFinite(candidate.matches)
    || !Number.isFinite(candidate.moves)
    || !Number.isFinite(candidate.wins)
  ) {
    return null;
  }

  return {
    draws: Math.max(0, Math.floor(Number(candidate.draws))),
    losses: Math.max(0, Math.floor(Number(candidate.losses))),
    matches: Math.max(0, Math.floor(Number(candidate.matches))),
    moves: Math.max(0, Math.floor(Number(candidate.moves))),
    wins: Math.max(0, Math.floor(Number(candidate.wins))),
  };
}

export function archivedStatsFromDocument(value: unknown): ArchivedMatchStatsV1 {
  const candidate = value as Partial<ArchivedMatchStatsV1> | null;
  const totals = counterFromDocument(candidate?.totals);
  const freestyle = counterFromDocument(candidate?.by_ruleset?.freestyle);
  const renju = counterFromDocument(candidate?.by_ruleset?.renju);
  const black = counterFromDocument(candidate?.by_side?.black);
  const white = counterFromDocument(candidate?.by_side?.white);
  const bot = counterFromDocument(candidate?.by_opponent_type?.bot);
  const human = counterFromDocument(candidate?.by_opponent_type?.human);

  if (
    candidate === null
    || typeof candidate !== "object"
    || candidate.schema_version !== ARCHIVED_MATCH_STATS_SCHEMA_VERSION
    || !totals
    || !freestyle
    || !renju
    || !black
    || !white
    || !bot
    || !human
  ) {
    return emptyArchivedMatchStats();
  }

  return {
    archived_before: stringOrNull(candidate.archived_before),
    archived_count: Math.max(0, Math.floor(Number(candidate.archived_count) || 0)),
    by_opponent_type: { bot, human },
    by_ruleset: { freestyle, renju },
    by_side: { black, white },
    schema_version: ARCHIVED_MATCH_STATS_SCHEMA_VERSION,
    totals,
  };
}

function incrementCounter(counter: MatchStatsCounter, summary: MatchSummaryV1): void {
  counter.matches += 1;
  counter.moves += summary.move_count;

  if (summary.outcome === "win") {
    counter.wins += 1;
  } else if (summary.outcome === "loss") {
    counter.losses += 1;
  } else {
    counter.draws += 1;
  }
}

function addStatsCounter(target: MatchStatsCounter, source: MatchStatsCounter): void {
  target.draws += source.draws;
  target.losses += source.losses;
  target.matches += source.matches;
  target.moves += source.moves;
  target.wins += source.wins;
}

export function mergeArchivedMatchStats(
  left: ArchivedMatchStatsV1,
  right: ArchivedMatchStatsV1,
): ArchivedMatchStatsV1 {
  if (right.archived_count === 0) {
    return left;
  }

  const next: ArchivedMatchStatsV1 = JSON.parse(JSON.stringify(left)) as ArchivedMatchStatsV1;
  next.archived_count += right.archived_count;
  const archivedBefore = [next.archived_before, right.archived_before]
    .filter((value): value is string => Boolean(value))
    .sort();
  next.archived_before = archivedBefore.length > 0 ? archivedBefore[archivedBefore.length - 1]! : null;

  addStatsCounter(next.totals, right.totals);
  addStatsCounter(next.by_ruleset.freestyle, right.by_ruleset.freestyle);
  addStatsCounter(next.by_ruleset.renju, right.by_ruleset.renju);
  addStatsCounter(next.by_side.black, right.by_side.black);
  addStatsCounter(next.by_side.white, right.by_side.white);
  addStatsCounter(next.by_opponent_type.bot, right.by_opponent_type.bot);
  addStatsCounter(next.by_opponent_type.human, right.by_opponent_type.human);

  return next;
}

function archiveWithSummaries(
  archivedStats: ArchivedMatchStatsV1,
  summaries: MatchSummaryV1[],
): ArchivedMatchStatsV1 {
  if (summaries.length === 0) {
    return archivedStats;
  }

  const next: ArchivedMatchStatsV1 = JSON.parse(JSON.stringify(archivedStats)) as ArchivedMatchStatsV1;
  for (const summary of summaries) {
    incrementCounter(next.totals, summary);
    incrementCounter(next.by_ruleset[summary.ruleset], summary);
    incrementCounter(next.by_side[summary.side], summary);
    incrementCounter(next.by_opponent_type[summary.opponent.kind], summary);
  }

  next.archived_count += summaries.length;
  next.archived_before = summaries
    .map((summary) => summary.saved_at)
    .reduce((latest, savedAt) => (latest && latest > savedAt ? latest : savedAt), next.archived_before);

  return next;
}

function botKeyForPlayer(player: SavedMatchPlayer): string | null {
  if (player.kind !== "bot" || !player.bot) {
    return null;
  }

  return [
    `${player.bot.id}@${player.bot.version}`,
    `${player.bot.engine}/${player.bot.lab_spec}`,
  ].join(":");
}

export function matchSummaryForMatch(
  identity: MatchSummaryIdentity,
  match: SavedMatchV2,
): MatchSummaryV1 {
  const players = savedMatchPlayers(match);
  const local = players.find(({ player }) => identity.profileUid && player.profile_uid === identity.profileUid)
    ?? players.find(({ player }) => identity.localProfileId && player.local_profile_id === identity.localProfileId)
    ?? players.find(({ player }) => player.kind === "human")
    ?? players[0]!;
  const opponent = players.find(({ side }) => side !== local.side)?.player ?? players[1]!.player;
  const winningSide = savedMatchWinningSide(match);

  return {
    id: match.id,
    match_kind: match.match_kind,
    move_count: match.move_count,
    opening: DEFAULT_RULE_OPENING,
    opponent: {
      bot_key: botKeyForPlayer(opponent),
      display_name: opponent.display_name,
      kind: opponent.kind === "bot" ? "bot" : "human",
      profile_uid: opponent.profile_uid,
    },
    outcome: winningSide === null ? "draw" : winningSide === local.side ? "win" : "loss",
    ruleset: match.ruleset,
    saved_at: match.saved_at,
    schema_version: MATCH_SUMMARY_SCHEMA_VERSION,
    side: local.side,
    trust: match.trust,
  };
}

export function isMatchSummaryV1(value: unknown): value is MatchSummaryV1 {
  const candidate = value as Partial<MatchSummaryV1> | null;
  const opponent = candidate?.opponent as Partial<MatchSummaryOpponent> | null | undefined;

  return (
    candidate !== null
    && typeof candidate === "object"
    && typeof candidate.id === "string"
    && candidate.id.length > 0
    && candidate.schema_version === MATCH_SUMMARY_SCHEMA_VERSION
    && typeof candidate.saved_at === "string"
    && candidate.saved_at.length > 0
    && typeof candidate.move_count === "number"
    && Number.isInteger(candidate.move_count)
    && candidate.move_count >= 0
    && (candidate.match_kind === "local_vs_bot"
      || candidate.match_kind === "local_pvp"
      || candidate.match_kind === "online_pvp"
      || candidate.match_kind === "puzzle_challenge")
    && validVariant(candidate.ruleset) !== null
    && candidate.opening === DEFAULT_RULE_OPENING
    && (candidate.side === "black" || candidate.side === "white")
    && (candidate.outcome === "win" || candidate.outcome === "loss" || candidate.outcome === "draw")
    && (candidate.trust === "local_only"
      || candidate.trust === "client_uploaded"
      || candidate.trust === "server_verified")
    && opponent != null
    && (opponent.kind === "bot" || opponent.kind === "human")
    && typeof opponent.display_name === "string"
    && opponent.display_name.length > 0
    && isNullableString(opponent.profile_uid)
    && isNullableString(opponent.bot_key)
  );
}

export function mergeMatchSummaryState(input: {
  archivedStats: ArchivedMatchStatsV1;
  identity?: MatchSummaryIdentity;
  matches: SavedMatchV2[];
  replayMatches: SavedMatchV2[];
  resetAt?: string | null;
  summaries: MatchSummaryV1[];
}): Pick<MatchHistory, "archivedStats" | "summaryMatches"> {
  const byId = new Map<string, MatchSummaryV1>();
  const replayMatchIds = new Set(input.replayMatches.map((match) => match.id));

  for (const summary of input.summaries) {
    if (!savedMatchIsAfterReset({ saved_at: summary.saved_at }, input.resetAt)) {
      continue;
    }
    if (replayMatchIds.has(summary.id)) {
      continue;
    }
    byId.set(summary.id, summary);
  }

  for (const match of input.matches) {
    if (!savedMatchIsAfterReset(match, input.resetAt)) {
      continue;
    }

    const summary = matchSummaryForMatch(input.identity ?? {}, match);
    if (replayMatchIds.has(summary.id)) {
      continue;
    }

    const existing = byId.get(summary.id);
    if (!existing || summary.saved_at >= existing.saved_at) {
      byId.set(summary.id, summary);
    }
  }

  const sorted = Array.from(byId.values()).sort((left, right) => right.saved_at.localeCompare(left.saved_at));
  const summaryMatches = sorted.slice(0, SUMMARY_MATCHES_LIMIT);
  const evicted = sorted.slice(SUMMARY_MATCHES_LIMIT);

  return {
    archivedStats: archiveWithSummaries(input.archivedStats, evicted),
    summaryMatches,
  };
}

export function emptyMatchHistory(): MatchHistory {
  return {
    archivedStats: emptyArchivedMatchStats(),
    replayMatches: [],
    summaryMatches: [],
  };
}

export function matchHistoryHasMatch(history: MatchHistory, matchId: string): boolean {
  return history.replayMatches.some((match) => match.id === matchId)
    || history.summaryMatches.some((match) => match.id === matchId);
}

export function matchHistoryIsEmpty(history: MatchHistory): boolean {
  return history.replayMatches.length === 0
    && history.summaryMatches.length === 0
    && history.archivedStats.archived_count === 0;
}

