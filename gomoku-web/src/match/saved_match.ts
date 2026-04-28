import { BOARD_SIZE } from "../board/constants";
import type { GameVariant } from "../core/bot_protocol";
import type { CellPosition, MatchMove, MatchPlayer } from "../game/types";

export const SAVED_MATCH_SCHEMA_VERSION = 1;
export const SAVED_MATCH_BOARD_SIZE = BOARD_SIZE;
export const PRACTICE_BOT_ID = "practice_bot";
export const PRACTICE_BOT_VERSION = 1;
export const PRACTICE_BOT_ENGINE = "baseline_search";
export const PRACTICE_BOT_CONFIG_VERSION = 1;
export const PRACTICE_BOT_DEPTH = 3;

export type SavedMatchSource = "local_history" | "guest_import" | "cloud_saved";
export type SavedMatchTrust = "local_only" | "client_uploaded" | "server_verified";
export type SavedMatchKind = "local_vs_bot" | "local_pvp" | "online_pvp" | "puzzle_challenge";
export type SavedMatchStatus = "black_won" | "white_won" | "draw";
export type SavedMatchSide = "black" | "white";

export interface SavedMatchBotIdentity {
  config: {
    depth: typeof PRACTICE_BOT_DEPTH;
    kind: "baseline";
  };
  config_version: typeof PRACTICE_BOT_CONFIG_VERSION;
  engine: typeof PRACTICE_BOT_ENGINE;
  id: typeof PRACTICE_BOT_ID;
  version: typeof PRACTICE_BOT_VERSION;
}

export interface SavedMatchPlayer {
  bot: SavedMatchBotIdentity | null;
  display_name: string;
  kind: "human" | "bot";
  local_profile_id: string | null;
  profile_uid: string | null;
}

export interface SavedMatchV1 {
  board_size: typeof SAVED_MATCH_BOARD_SIZE;
  id: string;
  match_kind: SavedMatchKind;
  move_cells: number[];
  move_count: number;
  player_black: SavedMatchPlayer;
  player_white: SavedMatchPlayer;
  saved_at: string;
  schema_version: typeof SAVED_MATCH_SCHEMA_VERSION;
  source: SavedMatchSource;
  status: SavedMatchStatus;
  trust: SavedMatchTrust;
  undo_floor: number;
  variant: GameVariant;
}

export type LocalSavedMatchV1 = SavedMatchV1 & {
  source: "local_history";
  trust: "local_only";
};

export interface CreateLocalSavedMatchInput {
  id: string;
  localProfileId: string;
  moves: MatchMove[];
  players: [MatchPlayer, MatchPlayer];
  savedAt: string;
  status: SavedMatchStatus;
  undoFloor?: number;
  variant: GameVariant;
}

export interface LegacyGuestSavedMatch {
  guestStone?: "black" | "white";
  id: string;
  mode: "bot";
  moves: MatchMove[];
  players: [MatchPlayer, MatchPlayer];
  savedAt: string;
  status: SavedMatchStatus;
  undoFloor?: number;
  variant: GameVariant;
  winningCells?: CellPosition[];
}

function isString(value: unknown): value is string {
  return typeof value === "string";
}

function isNullableString(value: unknown): value is string | null {
  return value === null || isString(value);
}

function isSavedMatchSource(value: unknown): value is SavedMatchSource {
  return value === "local_history" || value === "guest_import" || value === "cloud_saved";
}

function isSavedMatchTrust(value: unknown): value is SavedMatchTrust {
  return value === "local_only" || value === "client_uploaded" || value === "server_verified";
}

function isSavedMatchKind(value: unknown): value is SavedMatchKind {
  return (
    value === "local_vs_bot"
    || value === "local_pvp"
    || value === "online_pvp"
    || value === "puzzle_challenge"
  );
}

function isSavedMatchStatus(value: unknown): value is SavedMatchStatus {
  return value === "black_won" || value === "white_won" || value === "draw";
}

function isSavedMatchVariant(value: unknown): value is GameVariant {
  return value === "freestyle" || value === "renju";
}

function isSavedMatchBotIdentity(value: unknown): value is SavedMatchBotIdentity {
  const candidate = value as Partial<SavedMatchBotIdentity> | null;
  const config = candidate?.config as Partial<SavedMatchBotIdentity["config"]> | undefined;

  return (
    candidate !== null
    && typeof candidate === "object"
    && candidate.id === PRACTICE_BOT_ID
    && candidate.version === PRACTICE_BOT_VERSION
    && candidate.engine === PRACTICE_BOT_ENGINE
    && candidate.config_version === PRACTICE_BOT_CONFIG_VERSION
    && config?.kind === "baseline"
    && config.depth === PRACTICE_BOT_DEPTH
  );
}

function isSavedMatchPlayer(value: unknown): value is SavedMatchPlayer {
  const candidate = value as Partial<SavedMatchPlayer> | null;
  if (
    candidate === null
    || typeof candidate !== "object"
    || !isString(candidate.display_name)
    || candidate.display_name.trim().length === 0
    || !isNullableString(candidate.local_profile_id)
    || !isNullableString(candidate.profile_uid)
  ) {
    return false;
  }

  if (candidate.kind === "human") {
    return candidate.bot === null;
  }

  return candidate.kind === "bot" && isSavedMatchBotIdentity(candidate.bot);
}

function isValidMoveCell(cell: unknown): cell is number {
  return (
    typeof cell === "number"
    && Number.isInteger(cell)
    && cell >= 0
    && cell < SAVED_MATCH_BOARD_SIZE * SAVED_MATCH_BOARD_SIZE
  );
}

export function isSavedMatchV1(value: unknown): value is SavedMatchV1 {
  const candidate = value as Partial<SavedMatchV1> | null;
  return (
    candidate !== null
    && typeof candidate === "object"
    && candidate.board_size === SAVED_MATCH_BOARD_SIZE
    && isString(candidate.id)
    && candidate.id.length > 0
    && isSavedMatchKind(candidate.match_kind)
    && Array.isArray(candidate.move_cells)
    && candidate.move_cells.every(isValidMoveCell)
    && typeof candidate.move_count === "number"
    && candidate.move_count === candidate.move_cells.length
    && isSavedMatchPlayer(candidate.player_black)
    && isSavedMatchPlayer(candidate.player_white)
    && isString(candidate.saved_at)
    && candidate.saved_at.length > 0
    && candidate.schema_version === SAVED_MATCH_SCHEMA_VERSION
    && isSavedMatchSource(candidate.source)
    && isSavedMatchStatus(candidate.status)
    && isSavedMatchTrust(candidate.trust)
    && typeof candidate.undo_floor === "number"
    && Number.isInteger(candidate.undo_floor)
    && candidate.undo_floor >= 0
    && candidate.undo_floor <= candidate.move_count
    && isSavedMatchVariant(candidate.variant)
  );
}

export function isLocalSavedMatchV1(value: unknown): value is LocalSavedMatchV1 {
  return isSavedMatchV1(value) && value.source === "local_history" && value.trust === "local_only";
}

export function practiceBotIdentity(): SavedMatchBotIdentity {
  return {
    config: {
      depth: PRACTICE_BOT_DEPTH,
      kind: "baseline",
    },
    config_version: PRACTICE_BOT_CONFIG_VERSION,
    engine: PRACTICE_BOT_ENGINE,
    id: PRACTICE_BOT_ID,
    version: PRACTICE_BOT_VERSION,
  };
}

function normalizeUndoFloor(undoFloor: number | undefined, moveCount: number): number {
  if (undoFloor === undefined || !Number.isFinite(undoFloor)) {
    return 0;
  }

  return Math.max(0, Math.min(moveCount, Math.floor(undoFloor)));
}

function normalizeSavedMatchStatus(status: string): SavedMatchStatus {
  if (status === "black_won" || status === "white_won" || status === "draw") {
    return status;
  }

  throw new Error("Saved match only supports finished matches.");
}

function sideForPlayer(player: MatchPlayer): SavedMatchSide {
  return player.stone;
}

export function encodeMoveCell(move: Pick<MatchMove, "row" | "col">): number {
  if (
    !Number.isInteger(move.row)
    || !Number.isInteger(move.col)
    || move.row < 0
    || move.row >= SAVED_MATCH_BOARD_SIZE
    || move.col < 0
    || move.col >= SAVED_MATCH_BOARD_SIZE
  ) {
    throw new Error("Saved match only supports moves inside the board.");
  }

  return move.row * SAVED_MATCH_BOARD_SIZE + move.col;
}

export function decodeMoveCell(cell: number): CellPosition {
  if (!Number.isInteger(cell) || cell < 0 || cell >= SAVED_MATCH_BOARD_SIZE * SAVED_MATCH_BOARD_SIZE) {
    throw new Error("Saved match contains a move outside the board.");
  }

  return {
    col: cell % SAVED_MATCH_BOARD_SIZE,
    row: Math.floor(cell / SAVED_MATCH_BOARD_SIZE),
  };
}

export function encodeMoveCells(moves: MatchMove[]): number[] {
  return moves.map(encodeMoveCell);
}

export function movesFromMoveCells(moveCells: number[]): MatchMove[] {
  return moveCells.map((cell, index) => {
    const position = decodeMoveCell(cell);
    return {
      ...position,
      moveNumber: index + 1,
      player: index % 2 === 0 ? 1 : 2,
    };
  });
}

function savedMatchPlayer(
  player: MatchPlayer,
  localProfileId: string,
): SavedMatchPlayer {
  if (player.kind === "human") {
    return {
      bot: null,
      display_name: player.name,
      kind: "human",
      local_profile_id: localProfileId,
      profile_uid: null,
    };
  }

  return {
    bot: practiceBotIdentity(),
    display_name: player.name,
    kind: "bot",
    local_profile_id: null,
    profile_uid: null,
  };
}

function sidePlayers(
  players: [MatchPlayer, MatchPlayer],
  localProfileId: string,
): Pick<SavedMatchV1, "player_black" | "player_white"> {
  const black = players.find((player) => sideForPlayer(player) === "black");
  const white = players.find((player) => sideForPlayer(player) === "white");

  if (!black || !white) {
    throw new Error("Saved match requires one black player and one white player.");
  }

  return {
    player_black: savedMatchPlayer(black, localProfileId),
    player_white: savedMatchPlayer(white, localProfileId),
  };
}

export function createLocalSavedMatch(input: CreateLocalSavedMatchInput): LocalSavedMatchV1 {
  const players = sidePlayers(input.players, input.localProfileId);
  const moveCells = encodeMoveCells(input.moves);

  return {
    board_size: SAVED_MATCH_BOARD_SIZE,
    id: input.id,
    match_kind: "local_vs_bot",
    move_cells: moveCells,
    move_count: moveCells.length,
    player_black: players.player_black,
    player_white: players.player_white,
    saved_at: input.savedAt,
    schema_version: SAVED_MATCH_SCHEMA_VERSION,
    source: "local_history",
    status: input.status,
    trust: "local_only",
    undo_floor: normalizeUndoFloor(input.undoFloor, moveCells.length),
    variant: input.variant,
  };
}

export function migrateLegacyGuestSavedMatch(
  match: LegacyGuestSavedMatch,
  localProfileId: string,
): LocalSavedMatchV1 {
  return createLocalSavedMatch({
    id: match.id,
    localProfileId,
    moves: match.moves,
    players: match.players,
    savedAt: match.savedAt,
    status: normalizeSavedMatchStatus(match.status),
    undoFloor: match.undoFloor,
    variant: match.variant,
  });
}

export function savedMatchPlayerForSide(match: SavedMatchV1, side: SavedMatchSide): SavedMatchPlayer {
  return side === "black" ? match.player_black : match.player_white;
}

export function savedMatchPlayers(
  match: SavedMatchV1,
): Array<{ player: SavedMatchPlayer; side: SavedMatchSide }> {
  return [
    { player: match.player_black, side: "black" },
    { player: match.player_white, side: "white" },
  ];
}

export function savedMatchLocalSide(
  match: SavedMatchV1,
  localProfileId: string | null | undefined,
): SavedMatchSide | null {
  if (!localProfileId) {
    return null;
  }

  if (match.player_black.local_profile_id === localProfileId) {
    return "black";
  }

  if (match.player_white.local_profile_id === localProfileId) {
    return "white";
  }

  return null;
}

/**
 * Resolves the local user's side for a match using cloud or local identity.
 * Prefers profile_uid (works cross-device for cloud matches), falls back to
 * local_profile_id (works for local-only and guest-imported records).
 */
export function matchUserSide(
  match: SavedMatchV1,
  opts: { localProfileId?: string | null; profileUid?: string | null },
): SavedMatchSide | null {
  const { localProfileId, profileUid } = opts;

  if (profileUid) {
    if (match.player_black.profile_uid === profileUid) {
      return "black";
    }
    if (match.player_white.profile_uid === profileUid) {
      return "white";
    }
  }

  return savedMatchLocalSide(match, localProfileId);
}

export function savedMatchWinningSide(match: Pick<SavedMatchV1, "status">): SavedMatchSide | null {
  if (match.status === "black_won") {
    return "black";
  }

  if (match.status === "white_won") {
    return "white";
  }

  return null;
}
