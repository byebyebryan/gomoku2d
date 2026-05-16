import {
  deleteDoc,
  doc,
  getDoc,
  serverTimestamp,
  setDoc,
  type Firestore,
} from "firebase/firestore";

import type { GameVariant } from "../core/bot_protocol";
import {
  isBotConfig,
  sanitizeBotConfig,
  type BotConfig,
} from "../core/bot_config";
import {
  isSavedMatchV2,
  savedMatchIsAfterReset,
  savedMatchPlayers,
  savedMatchWinningSide,
  type SavedMatchPlayer,
  type SavedMatchSide,
  type SavedMatchV2,
} from "../match/saved_match";
import {
  DEFAULT_RULE_OPENING,
  createDefaultProfileSettings,
  sanitizeBoardHints,
  sanitizeGameConfig,
  sanitizeProfileSettings,
  sanitizeTouchControl,
  type BoardHintSettings,
  type GameConfig,
  type ProfileSettings,
  type TouchControlMode,
} from "../profile/profile_settings";

import type { CloudAuthUser } from "./auth_store";
import { createCloudSavedMatch } from "./cloud_match";
import { getFirebaseClients } from "./firebase";

export const CLOUD_PROFILE_SCHEMA_VERSION = 5;
export const CLOUD_REPLAY_MATCHES_LIMIT = 128;
export const CLOUD_SUMMARY_MATCHES_LIMIT = 1024;
export const CLOUD_MATCH_SUMMARY_SCHEMA_VERSION = 1;
export const CLOUD_ARCHIVED_MATCH_STATS_SCHEMA_VERSION = 1;
export const CLOUD_PROFILE_SYNC_INTERVAL_MS = 5 * 60 * 1000;
export const CLOUD_DEFAULT_RULE_OPENING = DEFAULT_RULE_OPENING;

export type CloudAuthProviderId = "github.com" | "google.com";

export interface CloudAuthProvider {
  avatarUrl: string | null;
  displayName: string | null;
  provider: CloudAuthProviderId;
}

export interface CloudProfileAuth {
  providers: CloudAuthProvider[];
}

export type CloudGameConfig = GameConfig;
export type CloudProfileSettings = ProfileSettings;

export type CloudMatchSummaryOutcome = "draw" | "loss" | "win";
export type CloudMatchSummaryOpponentKind = "bot" | "human";

export interface CloudMatchSummaryOpponent {
  bot_key: string | null;
  display_name: string;
  kind: CloudMatchSummaryOpponentKind;
  profile_uid: string | null;
}

export interface CloudMatchSummaryV1 {
  id: string;
  match_kind: SavedMatchV2["match_kind"];
  move_count: number;
  opening: typeof CLOUD_DEFAULT_RULE_OPENING;
  opponent: CloudMatchSummaryOpponent;
  outcome: CloudMatchSummaryOutcome;
  ruleset: GameVariant;
  saved_at: string;
  schema_version: typeof CLOUD_MATCH_SUMMARY_SCHEMA_VERSION;
  side: SavedMatchSide;
  trust: SavedMatchV2["trust"];
}

export interface CloudMatchStatsCounter {
  draws: number;
  losses: number;
  matches: number;
  moves: number;
  wins: number;
}

export interface CloudArchivedMatchStatsV1 {
  archived_before: string | null;
  archived_count: number;
  by_opponent_type: Record<CloudMatchSummaryOpponentKind, CloudMatchStatsCounter>;
  by_ruleset: Record<GameVariant, CloudMatchStatsCounter>;
  by_side: Record<SavedMatchSide, CloudMatchStatsCounter>;
  schema_version: typeof CLOUD_ARCHIVED_MATCH_STATS_SCHEMA_VERSION;
  totals: CloudMatchStatsCounter;
}

export interface CloudMatchHistory {
  archivedStats: CloudArchivedMatchStatsV1;
  replayMatches: SavedMatchV2[];
  summaryMatches: CloudMatchSummaryV1[];
}

export interface CloudMatchSummaryIdentity {
  localProfileId?: string | null;
  profileUid?: string | null;
}

export interface CloudProfile {
  auth: CloudProfileAuth;
  createdAt: string | null;
  displayName: string;
  matchHistory: CloudMatchHistory;
  resetAt: string | null;
  settings: CloudProfileSettings;
  uid: string;
  updatedAt: string | null;
  username: string | null;
}

export interface CloudProfileDocument {
  auth?: unknown;
  created_at?: unknown;
  display_name?: unknown;
  match_history?: unknown;
  reset_at?: unknown;
  schema_version?: unknown;
  settings?: unknown;
  uid?: unknown;
  updated_at?: unknown;
  username?: unknown;
}

export interface EnsureCloudProfileOptions {
  firestore?: Firestore;
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

function timestampIsoOrNull(value: unknown): string | null {
  if (value instanceof Date && Number.isFinite(value.getTime())) {
    return value.toISOString();
  }

  const candidate = value as { nanoseconds?: unknown; seconds?: unknown; toDate?: unknown } | null;
  if (candidate && typeof candidate === "object") {
    if (typeof candidate.toDate === "function") {
      const date = candidate.toDate() as Date;
      return Number.isFinite(date.getTime()) ? date.toISOString() : null;
    }

    if (typeof candidate.seconds === "number") {
      const nanoseconds = typeof candidate.nanoseconds === "number" ? candidate.nanoseconds : 0;
      return new Date((candidate.seconds * 1000) + Math.floor(nanoseconds / 1_000_000)).toISOString();
    }
  }

  return null;
}

function normalizeProvider(value: unknown): CloudAuthProviderId | null {
  if (value === "google.com" || value === "google") {
    return "google.com";
  }

  if (value === "github.com" || value === "github") {
    return "github.com";
  }

  return null;
}

function authProviderForUser(user: CloudAuthUser): CloudAuthProvider[] {
  const rawProviders = user.providers?.length
    ? user.providers
    : user.providerIds.map((providerId) => ({
      avatarUrl: user.avatarUrl,
      displayName: user.displayName,
      provider: providerId,
    }));
  const byId = new Map<CloudAuthProviderId, CloudAuthProvider>();

  for (const rawProvider of rawProviders) {
    const provider = normalizeProvider(rawProvider.provider);
    if (!provider) {
      continue;
    }

    byId.set(provider, {
      avatarUrl: stringOrNull(rawProvider.avatarUrl),
      displayName: stringOrNull(rawProvider.displayName),
      provider,
    });
  }

  return Array.from(byId.values()).sort((left, right) => left.provider.localeCompare(right.provider));
}

function authForUser(user: CloudAuthUser): CloudProfileAuth {
  const providers = authProviderForUser(user);

  return {
    providers: providers.length > 0
      ? providers
      : [{
        avatarUrl: stringOrNull(user.avatarUrl),
        displayName: stringOrNull(user.displayName),
        provider: "google.com",
      }],
  };
}

function authProviderFromDocument(value: unknown): CloudAuthProvider | null {
  const candidate = value as {
    avatar_url?: unknown;
    display_name?: unknown;
    provider?: unknown;
    provider_id?: unknown;
  } | null;
  const provider = normalizeProvider(candidate?.provider) ?? normalizeProvider(candidate?.provider_id);

  if (!provider) {
    return null;
  }

  return {
    avatarUrl: stringOrNull(candidate?.avatar_url),
    displayName: stringOrNull(candidate?.display_name),
    provider,
  };
}

function authFromDocument(value: unknown, fallback: CloudProfileAuth): CloudProfileAuth {
  const candidate = value as { providers?: unknown } | null;
  if (!Array.isArray(candidate?.providers)) {
    return fallback;
  }

  const providers = candidate.providers.flatMap((entry) => {
    const provider = authProviderFromDocument(entry);
    return provider ? [provider] : [];
  });

  return providers.length > 0 ? { providers } : fallback;
}

function authDocument(auth: CloudProfileAuth) {
  return {
    providers: auth.providers.map((provider) => ({
      avatar_url: provider.avatarUrl,
      display_name: provider.displayName,
      provider: provider.provider,
    })),
  };
}

function authEqual(left: CloudProfileAuth, right: CloudProfileAuth): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function botConfigFromDocument(value: unknown): BotConfig {
  const candidate = value as Record<string, unknown> | null;
  if (candidate?.mode === "custom") {
    return sanitizeBotConfig({
      depth: candidate.depth,
      extraPass: candidate.extra_pass,
      mode: candidate.mode,
      scoring: candidate.scoring,
      version: candidate.version,
      width: candidate.width,
    });
  }

  return sanitizeBotConfig(value);
}

function isBotConfigDocument(value: unknown): boolean {
  const candidate = value as Record<string, unknown> | null;
  if (!candidate || typeof candidate !== "object") {
    return false;
  }

  if (candidate.mode === "preset") {
    return isBotConfig(candidate);
  }

  return isBotConfig({
    depth: candidate.depth,
    extraPass: candidate.extra_pass,
    mode: candidate.mode,
    scoring: candidate.scoring,
    version: candidate.version,
    width: candidate.width,
  });
}

function botConfigDocument(botConfig: BotConfig) {
  if (botConfig.mode === "preset") {
    return botConfig;
  }

  return {
    depth: botConfig.depth,
    extra_pass: botConfig.extraPass,
    mode: botConfig.mode,
    scoring: botConfig.scoring,
    version: botConfig.version,
    width: botConfig.width,
  };
}

function settingsFromDocument(value: unknown): CloudProfileSettings {
  const candidate = value as {
    board_hints?: unknown;
    bot_config?: unknown;
    game_config?: unknown;
    touch_control?: unknown;
  } | null;

  return sanitizeProfileSettings({
    boardHints: sanitizeBoardHints(candidate?.board_hints),
    botConfig: botConfigFromDocument(candidate?.bot_config),
    gameConfig: sanitizeGameConfig(candidate?.game_config),
    touchControl: sanitizeTouchControl(candidate?.touch_control),
  });
}

export function cloudSettingsFromProfileSettings(settings: ProfileSettings): CloudProfileSettings {
  return sanitizeProfileSettings(settings);
}

export function cloudSettingsDocument(settings: CloudProfileSettings) {
  return {
    board_hints: {
      immediate: settings.boardHints.immediate,
      imminent: settings.boardHints.imminent,
    },
    bot_config: botConfigDocument(settings.botConfig),
    game_config: {
      opening: settings.gameConfig.opening,
      ruleset: settings.gameConfig.ruleset,
    },
    touch_control: settings.touchControl,
  };
}

function settingsEqual(left: CloudProfileSettings, right: CloudProfileSettings): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function isSettingsDocument(value: unknown): boolean {
  const candidate = value as {
    board_hints?: Partial<BoardHintSettings>;
    bot_config?: unknown;
    game_config?: {
      opening?: unknown;
      ruleset?: unknown;
    };
    touch_control?: unknown;
  } | null;

  return Boolean(candidate)
    && typeof candidate === "object"
    && candidate?.game_config?.opening === CLOUD_DEFAULT_RULE_OPENING
    && validVariant(candidate?.game_config?.ruleset) !== null
    && isBotConfigDocument(candidate?.bot_config)
    && (candidate?.touch_control === "pointer" || candidate?.touch_control === "touchpad")
    && (
      candidate?.board_hints?.immediate === "off"
      || candidate?.board_hints?.immediate === "win"
      || candidate?.board_hints?.immediate === "win_threat"
    )
    && (
      candidate?.board_hints?.imminent === "off"
      || candidate?.board_hints?.imminent === "threat"
      || candidate?.board_hints?.imminent === "threat_counter"
    );
}

function sortReplayMatches(matches: SavedMatchV2[]): SavedMatchV2[] {
  return [...matches].sort((left, right) => right.saved_at.localeCompare(left.saved_at));
}

function replayMatchesFromDocument(
  matchHistoryValue: unknown,
  resetAt: string | null,
): SavedMatchV2[] {
  const candidate = matchHistoryValue as { replay_matches?: unknown } | null;
  const rawMatches = Array.isArray(candidate?.replay_matches) ? candidate.replay_matches : [];
  const matches = rawMatches
    .filter(isSavedMatchV2)
    .filter((match) => savedMatchIsAfterReset(match, resetAt));

  return sortReplayMatches(matches).slice(0, CLOUD_REPLAY_MATCHES_LIMIT);
}

export function mergeCloudSavedMatches(
  user: Pick<CloudAuthUser, "uid">,
  matches: SavedMatchV2[],
  resetAt: string | null | undefined = null,
): SavedMatchV2[] {
  const byId = new Map<string, SavedMatchV2>();

  for (const match of matches) {
    if (!savedMatchIsAfterReset(match, resetAt)) {
      continue;
    }

    const cloudMatch = match.source === "local_history"
      ? createCloudSavedMatch(user, match)
      : match;
    const existing = byId.get(cloudMatch.id);

    if (!existing || cloudMatch.saved_at >= existing.saved_at) {
      byId.set(cloudMatch.id, cloudMatch);
    }
  }

  return sortReplayMatches(Array.from(byId.values()));
}

export function mergeCloudReplayMatches(
  user: Pick<CloudAuthUser, "uid">,
  matches: SavedMatchV2[],
  resetAt: string | null | undefined = null,
): SavedMatchV2[] {
  return mergeCloudSavedMatches(user, matches, resetAt).slice(0, CLOUD_REPLAY_MATCHES_LIMIT);
}

function replayMatchesEqual(left: SavedMatchV2[], right: SavedMatchV2[]): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function emptyStatsCounter(): CloudMatchStatsCounter {
  return {
    draws: 0,
    losses: 0,
    matches: 0,
    moves: 0,
    wins: 0,
  };
}

function emptyStatsByRuleset(): Record<GameVariant, CloudMatchStatsCounter> {
  return {
    freestyle: emptyStatsCounter(),
    renju: emptyStatsCounter(),
  };
}

function emptyStatsBySide(): Record<SavedMatchSide, CloudMatchStatsCounter> {
  return {
    black: emptyStatsCounter(),
    white: emptyStatsCounter(),
  };
}

function emptyStatsByOpponentType(): Record<CloudMatchSummaryOpponentKind, CloudMatchStatsCounter> {
  return {
    bot: emptyStatsCounter(),
    human: emptyStatsCounter(),
  };
}

export function emptyCloudArchivedMatchStats(): CloudArchivedMatchStatsV1 {
  return {
    archived_before: null,
    archived_count: 0,
    by_opponent_type: emptyStatsByOpponentType(),
    by_ruleset: emptyStatsByRuleset(),
    by_side: emptyStatsBySide(),
    schema_version: CLOUD_ARCHIVED_MATCH_STATS_SCHEMA_VERSION,
    totals: emptyStatsCounter(),
  };
}

function counterFromDocument(value: unknown): CloudMatchStatsCounter | null {
  const candidate = value as Partial<CloudMatchStatsCounter> | null;
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

export function archivedStatsFromDocument(value: unknown): CloudArchivedMatchStatsV1 {
  const candidate = value as Partial<CloudArchivedMatchStatsV1> | null;
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
    || candidate.schema_version !== CLOUD_ARCHIVED_MATCH_STATS_SCHEMA_VERSION
    || !totals
    || !freestyle
    || !renju
    || !black
    || !white
    || !bot
    || !human
  ) {
    return emptyCloudArchivedMatchStats();
  }

  return {
    archived_before: stringOrNull(candidate.archived_before),
    archived_count: Math.max(0, Math.floor(Number(candidate.archived_count) || 0)),
    by_opponent_type: { bot, human },
    by_ruleset: { freestyle, renju },
    by_side: { black, white },
    schema_version: CLOUD_ARCHIVED_MATCH_STATS_SCHEMA_VERSION,
    totals,
  };
}

function incrementCounter(counter: CloudMatchStatsCounter, summary: CloudMatchSummaryV1): void {
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

function addStatsCounter(target: CloudMatchStatsCounter, source: CloudMatchStatsCounter): void {
  target.draws += source.draws;
  target.losses += source.losses;
  target.matches += source.matches;
  target.moves += source.moves;
  target.wins += source.wins;
}

export function mergeCloudArchivedMatchStats(
  left: CloudArchivedMatchStatsV1,
  right: CloudArchivedMatchStatsV1,
): CloudArchivedMatchStatsV1 {
  if (right.archived_count === 0) {
    return left;
  }

  const next: CloudArchivedMatchStatsV1 = JSON.parse(JSON.stringify(left)) as CloudArchivedMatchStatsV1;
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
  archivedStats: CloudArchivedMatchStatsV1,
  summaries: CloudMatchSummaryV1[],
): CloudArchivedMatchStatsV1 {
  if (summaries.length === 0) {
    return archivedStats;
  }

  const next: CloudArchivedMatchStatsV1 = JSON.parse(JSON.stringify(archivedStats)) as CloudArchivedMatchStatsV1;
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

export function cloudMatchSummaryForMatch(
  identity: CloudMatchSummaryIdentity,
  match: SavedMatchV2,
): CloudMatchSummaryV1 {
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
    opening: CLOUD_DEFAULT_RULE_OPENING,
    opponent: {
      bot_key: botKeyForPlayer(opponent),
      display_name: opponent.display_name,
      kind: opponent.kind === "bot" ? "bot" : "human",
      profile_uid: opponent.profile_uid,
    },
    outcome: winningSide === null ? "draw" : winningSide === local.side ? "win" : "loss",
    ruleset: match.ruleset,
    saved_at: match.saved_at,
    schema_version: CLOUD_MATCH_SUMMARY_SCHEMA_VERSION,
    side: local.side,
    trust: match.trust,
  };
}

export function isCloudMatchSummaryV1(value: unknown): value is CloudMatchSummaryV1 {
  const candidate = value as Partial<CloudMatchSummaryV1> | null;
  const opponent = candidate?.opponent as Partial<CloudMatchSummaryOpponent> | undefined;

  return (
    candidate !== null
    && typeof candidate === "object"
    && typeof candidate.id === "string"
    && candidate.id.length > 0
    && candidate.schema_version === CLOUD_MATCH_SUMMARY_SCHEMA_VERSION
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
    && candidate.opening === CLOUD_DEFAULT_RULE_OPENING
    && (candidate.side === "black" || candidate.side === "white")
    && (candidate.outcome === "win" || candidate.outcome === "loss" || candidate.outcome === "draw")
    && (candidate.trust === "local_only"
      || candidate.trust === "client_uploaded"
      || candidate.trust === "server_verified")
    && opponent !== undefined
    && (opponent.kind === "bot" || opponent.kind === "human")
    && typeof opponent.display_name === "string"
    && opponent.display_name.length > 0
    && isNullableString(opponent.profile_uid)
    && isNullableString(opponent.bot_key)
  );
}

function summaryMatchesFromDocument(
  matchHistoryValue: unknown,
  resetAt: string | null,
): CloudMatchSummaryV1[] {
  const candidate = matchHistoryValue as { summary_matches?: unknown } | null;
  const rawSummaries = Array.isArray(candidate?.summary_matches) ? candidate.summary_matches : [];
  const summaries = Array.isArray(rawSummaries)
    ? rawSummaries
      .filter(isCloudMatchSummaryV1)
      .filter((summary) => savedMatchIsAfterReset({ saved_at: summary.saved_at }, resetAt))
    : [];

  return summaries
    .sort((left, right) => right.saved_at.localeCompare(left.saved_at))
    .slice(0, CLOUD_SUMMARY_MATCHES_LIMIT);
}

export function mergeCloudMatchSummaryState(input: {
  archivedStats: CloudArchivedMatchStatsV1;
  convertLocalMatches?: boolean;
  identity?: CloudMatchSummaryIdentity;
  matches: SavedMatchV2[];
  replayMatches: SavedMatchV2[];
  resetAt?: string | null;
  summaries: CloudMatchSummaryV1[];
  user?: Pick<CloudAuthUser, "uid">;
}): Pick<CloudMatchHistory, "archivedStats" | "summaryMatches"> {
  const byId = new Map<string, CloudMatchSummaryV1>();
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

    const shouldConvertLocal = input.convertLocalMatches !== false && input.user;
    const cloudMatch = shouldConvertLocal && match.source === "local_history"
      ? createCloudSavedMatch(input.user!, match)
      : match;
    const summary = cloudMatchSummaryForMatch(input.identity ?? { profileUid: input.user?.uid ?? null }, cloudMatch);
    if (replayMatchIds.has(summary.id)) {
      continue;
    }

    const existing = byId.get(summary.id);
    if (!existing || summary.saved_at >= existing.saved_at) {
      byId.set(summary.id, summary);
    }
  }

  const sorted = Array.from(byId.values()).sort((left, right) => right.saved_at.localeCompare(left.saved_at));
  const summaryMatches = sorted.slice(0, CLOUD_SUMMARY_MATCHES_LIMIT);
  const evicted = sorted.slice(CLOUD_SUMMARY_MATCHES_LIMIT);

  return {
    archivedStats: archiveWithSummaries(input.archivedStats, evicted),
    summaryMatches,
  };
}

function matchHistoryFromDocument(
  document: CloudProfileDocument | null | undefined,
  resetAt: string | null,
  user: Pick<CloudAuthUser, "uid">,
): CloudMatchHistory {
  const candidate = document?.match_history as { archived_stats?: unknown } | null;
  const replayMatches = replayMatchesFromDocument(document?.match_history, resetAt);
  const summaryMatches = summaryMatchesFromDocument(document?.match_history, resetAt);
  const archivedStats = archivedStatsFromDocument(candidate?.archived_stats);
  const summaryState = mergeCloudMatchSummaryState({
    archivedStats,
    matches: [],
    replayMatches,
    resetAt,
    summaries: summaryMatches,
    user,
  });

  return {
    archivedStats: summaryState.archivedStats,
    replayMatches,
    summaryMatches: summaryState.summaryMatches,
  };
}

function matchHistoryDocument(history: CloudMatchHistory): Record<string, unknown> {
  return {
    archived_stats: history.archivedStats,
    replay_matches: history.replayMatches,
    summary_matches: history.summaryMatches,
  };
}

export function emptyCloudMatchHistory(): CloudMatchHistory {
  return {
    archivedStats: emptyCloudArchivedMatchStats(),
    replayMatches: [],
    summaryMatches: [],
  };
}

function isGroupedMatchHistoryDocument(value: unknown): boolean {
  const candidate = value as { archived_stats?: unknown; replay_matches?: unknown; summary_matches?: unknown } | null;
  return candidate !== null
    && typeof candidate === "object"
    && Array.isArray(candidate.replay_matches)
    && Array.isArray(candidate.summary_matches)
    && candidate.archived_stats !== undefined;
}

function matchHistoryEqual(left: CloudMatchHistory, right: CloudMatchHistory): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function cloudMatchHistoryHasMatch(history: CloudMatchHistory, matchId: string): boolean {
  return history.replayMatches.some((match) => match.id === matchId)
    || history.summaryMatches.some((match) => match.id === matchId);
}

export function cloudMatchHistoryIsEmpty(history: CloudMatchHistory): boolean {
  return history.replayMatches.length === 0
    && history.summaryMatches.length === 0
    && history.archivedStats.archived_count === 0;
}

function hasOwnField(document: CloudProfileDocument, field: keyof CloudProfileDocument): boolean {
  return Object.prototype.hasOwnProperty.call(document, field);
}

export function cloudProfileFromDocument(
  user: CloudAuthUser,
  fallbackSettings: ProfileSettings = createDefaultProfileSettings(),
  document: CloudProfileDocument | null,
): CloudProfile {
  const fallbackAuth = authForUser(user);
  const resetAt = timestampIsoOrNull(document?.reset_at);
  const settings = hasOwnField(document ?? {}, "settings")
    ? settingsFromDocument(document?.settings)
    : cloudSettingsFromProfileSettings(fallbackSettings);
  const matchHistory = matchHistoryFromDocument(document, resetAt, user);

  return {
    auth: authFromDocument(document?.auth, fallbackAuth),
    createdAt: timestampIsoOrNull(document?.created_at),
    displayName: stringOrNull(document?.display_name) ?? user.displayName,
    matchHistory,
    resetAt,
    settings,
    uid: user.uid,
    updatedAt: timestampIsoOrNull(document?.updated_at),
    username: stringOrNull(document?.username),
  };
}

export function newCloudProfileWrite(
  user: CloudAuthUser,
  settings: ProfileSettings = createDefaultProfileSettings(),
) {
  const now = serverTimestamp();
  const nextSettings = cloudSettingsFromProfileSettings(settings);

  return {
    auth: authDocument(authForUser(user)),
    created_at: now,
    display_name: user.displayName,
    match_history: matchHistoryDocument(emptyCloudMatchHistory()),
    reset_at: null,
    schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
    settings: cloudSettingsDocument(nextSettings),
    uid: user.uid,
    updated_at: now,
    username: null,
  };
}

export function existingCloudProfileUpdate(
  user: CloudAuthUser,
): Record<string, unknown>;
export function existingCloudProfileUpdate(
  user: CloudAuthUser,
  document: CloudProfileDocument,
  fallbackSettings?: ProfileSettings,
): Record<string, unknown> | null;
export function existingCloudProfileUpdate(
  user: CloudAuthUser,
  document?: CloudProfileDocument,
  fallbackSettings: ProfileSettings = createDefaultProfileSettings(),
): Record<string, unknown> | null {
  const patch: Record<string, unknown> = {};
  const expectedAuth = authForUser(user);

  if (!document || !authEqual(authFromDocument(document.auth, expectedAuth), expectedAuth)) {
    patch.auth = authDocument(expectedAuth);
  }

  if (!document || document.schema_version !== CLOUD_PROFILE_SCHEMA_VERSION) {
    patch.schema_version = CLOUD_PROFILE_SCHEMA_VERSION;
  }

  if (!document || !hasOwnField(document, "settings") || !isSettingsDocument(document.settings)) {
    patch.settings = cloudSettingsDocument(cloudSettingsFromProfileSettings(fallbackSettings));
  }

  if (!document || !hasOwnField(document, "reset_at")) {
    patch.reset_at = null;
  }

  const resetAt = timestampIsoOrNull(document?.reset_at);
  const matchHistory = matchHistoryFromDocument(document, resetAt, user);

  if (!document || !isGroupedMatchHistoryDocument(document.match_history)) {
    patch.match_history = matchHistoryDocument(matchHistory);
  }

  if (!document || document.uid !== user.uid) {
    patch.uid = user.uid;
  }

  if (document && Object.keys(patch).length === 0) {
    return null;
  }

  return {
    ...patch,
    updated_at: serverTimestamp(),
  };
}

export function existingCloudProfileLoadUpdate(
  user: CloudAuthUser,
  document: CloudProfileDocument,
  fallbackSettings: ProfileSettings = createDefaultProfileSettings(),
): Record<string, unknown> | null {
  if (
    document.schema_version !== CLOUD_PROFILE_SCHEMA_VERSION
    || !hasOwnField(document, "settings")
    || !isSettingsDocument(document.settings)
  ) {
    return null;
  }

  return existingCloudProfileUpdate(user, document, fallbackSettings);
}

export function resetCloudProfileUpdate(
  user: CloudAuthUser,
  settings: ProfileSettings = createDefaultProfileSettings(),
) {
  const now = serverTimestamp();
  const nextSettings = cloudSettingsFromProfileSettings(settings);
  const update: Record<string, unknown> = {
    auth: authDocument(authForUser(user)),
    display_name: user.displayName,
    match_history: matchHistoryDocument(emptyCloudMatchHistory()),
    reset_at: now,
    schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
    settings: cloudSettingsDocument(nextSettings),
    uid: user.uid,
    updated_at: now,
  };

  return update;
}

export function cloudProfileSnapshotUpdate(input: {
  displayName: string;
  matchHistory: CloudMatchHistory;
  settings: ProfileSettings;
  user: CloudAuthUser;
}): Record<string, unknown> {
  const now = serverTimestamp();
  const settings = cloudSettingsFromProfileSettings(input.settings);
  const update: Record<string, unknown> = {
    auth: authDocument(authForUser(input.user)),
    display_name: input.displayName,
    match_history: matchHistoryDocument(input.matchHistory),
    schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
    settings: cloudSettingsDocument(settings),
    uid: input.user.uid,
    updated_at: now,
  };

  return update;
}

export function cloudProfileSyncDue(
  profile: Pick<CloudProfile, "createdAt" | "matchHistory" | "updatedAt">,
  nowMs = Date.now(),
): boolean {
  if (!profile.updatedAt) {
    return true;
  }

  if (
    profile.createdAt === profile.updatedAt
    && profile.matchHistory.replayMatches.length === 0
    && profile.matchHistory.summaryMatches.length === 0
  ) {
    return true;
  }

  const updatedAtMs = Date.parse(profile.updatedAt);
  return !Number.isFinite(updatedAtMs) || nowMs >= updatedAtMs + CLOUD_PROFILE_SYNC_INTERVAL_MS;
}

export function cloudProfileNeedsSnapshotSync(input: {
  cloudDisplayName: string;
  cloudMatchHistory: CloudMatchHistory;
  cloudSettings: CloudProfileSettings;
  displayName: string;
  matchHistory: CloudMatchHistory;
  settings: ProfileSettings;
}): boolean {
  const settings = cloudSettingsFromProfileSettings(input.settings);

  return input.cloudDisplayName !== input.displayName
    || !settingsEqual(input.cloudSettings, settings)
    || !matchHistoryEqual(input.cloudMatchHistory, input.matchHistory);
}

export async function ensureCloudProfile(
  user: CloudAuthUser,
  settings: ProfileSettings = createDefaultProfileSettings(),
  options: EnsureCloudProfileOptions = {},
): Promise<CloudProfile> {
  const firestore = options.firestore ?? getFirebaseClients()?.firestore;
  if (!firestore) {
    throw new Error("Cloud profile is not configured for this build.");
  }

  const profileRef = doc(firestore, "profiles", user.uid);
  const snapshot = await getDoc(profileRef);

  if (snapshot.exists()) {
    const data = snapshot.data() as CloudProfileDocument;
    const update = existingCloudProfileLoadUpdate(user, data, settings);
    if (!update) {
      return cloudProfileFromDocument(user, settings, data);
    }

    await setDoc(profileRef, update, { merge: true });
    return cloudProfileFromDocument(user, settings, { ...data, ...update });
  }

  const profile = newCloudProfileWrite(user, settings);
  await setDoc(profileRef, profile);
  return cloudProfileFromDocument(user, settings, profile);
}

export async function resetCloudProfile(
  user: CloudAuthUser,
  settings: ProfileSettings = createDefaultProfileSettings(),
  options: EnsureCloudProfileOptions = {},
): Promise<CloudProfile> {
  const firestore = options.firestore ?? getFirebaseClients()?.firestore;
  if (!firestore) {
    throw new Error("Cloud profile reset is not configured for this build.");
  }

  const profileRef = doc(firestore, "profiles", user.uid);
  const snapshot = await getDoc(profileRef);
  if (!snapshot.exists()) {
    const profile = {
      ...newCloudProfileWrite(user, settings),
      reset_at: serverTimestamp(),
    };
    await setDoc(profileRef, profile);
    const refreshed = await getDoc(profileRef);
    return cloudProfileFromDocument(
      user,
      settings,
      refreshed.exists() ? (refreshed.data() as CloudProfileDocument) : profile,
    );
  }

  const update = resetCloudProfileUpdate(user, settings);
  await setDoc(profileRef, update, { merge: true });
  const refreshed = await getDoc(profileRef);
  const fallback = {
    ...(snapshot.data() as CloudProfileDocument),
    ...update,
  };

  return cloudProfileFromDocument(
    user,
    settings,
    refreshed.exists() ? (refreshed.data() as CloudProfileDocument) : fallback,
  );
}

export async function deleteCloudProfile(
  user: CloudAuthUser,
  options: EnsureCloudProfileOptions = {},
): Promise<void> {
  const firestore = options.firestore ?? getFirebaseClients()?.firestore;
  if (!firestore) {
    throw new Error("Cloud profile deletion is not configured for this build.");
  }

  await deleteDoc(doc(firestore, "profiles", user.uid));
}
