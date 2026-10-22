import type { GameVariant } from "../core/bot_protocol";
import {
  isBotConfig,
  isBotWidthAllowed,
  type BotConfig,
} from "../core/bot_config";
import {
  BOT_CONFIG_VERSION,
  BOT_ENGINE,
  BOT_VERSION,
  isSavedMatchV2,
  type SavedMatchStatus,
  type SavedMatchV2,
} from "../match/saved_match";
import { applyWasmMove, createWasmBoard } from "../core/wasm_bridge";

export const PORTABLE_REPLAY_VERSION = 1 as const;
export const PORTABLE_REPLAY_MIN_MOVES = 9;
export const PORTABLE_REPLAY_MAX_MOVES = 225;
export const PORTABLE_REPLAY_MAX_LABEL_CODE_POINTS = 80;
export const PORTABLE_REPLAY_MAX_LABEL_UTF8_BYTES = 320;
export const PORTABLE_REPLAY_MAX_UID_LENGTH = 128;
export const PORTABLE_REPLAY_MIN_PRACTICE_PLY = 4;

export interface PortableBotIdentityV1 {
  config: BotConfig;
  configVersion: typeof BOT_CONFIG_VERSION;
  engine: typeof BOT_ENGINE;
  version: typeof BOT_VERSION;
}

export type PortableReplayParticipantV1 =
  | {
    kind: "human";
    name: string;
    profileUid?: string;
  }
  | {
    bot: PortableBotIdentityV1;
    kind: "bot";
    name: string;
  };

export interface PortableReplayPracticeOriginV1 {
  inheritedPly: number;
  sourceFingerprint: string;
}

/**
 * Portable client record. The move cells use row * 15 + col; color is implied
 * by ply, beginning with Black. Result, trust, and storage metadata are absent.
 */
export interface PortableReplayV1 {
  black: PortableReplayParticipantV1;
  moves: readonly number[];
  practice?: PortableReplayPracticeOriginV1;
  ruleset: GameVariant;
  version: typeof PORTABLE_REPLAY_VERSION;
  viewPly?: number;
  white: PortableReplayParticipantV1;
}

export interface ValidatedPortableReplayV1 {
  replay: PortableReplayV1;
  status: SavedMatchStatus;
}

const PROFILE_UID_PATTERN = /^[A-Za-z0-9_-]{1,128}$/;
// SHA-256 is 32 bytes, so its unpadded base64url form is 43 chars. The final
// symbol carries only four data bits and therefore must have zero pad bits.
const FINGERPRINT_PATTERN = /^[A-Za-z0-9_-]{42}[AEIMQUYcgkosw048]$/;
const CONTROL_CHARACTER_PATTERN = /[\u0000-\u001f\u007f-\u009f]/u;

function hasOwn(record: Record<string, unknown>, key: string): boolean {
  return Object.prototype.hasOwnProperty.call(record, key);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return false;
  }

  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function requireRecord(value: unknown, label: string): Record<string, unknown> {
  if (!isRecord(value)) {
    throw new Error(`${label} must be a plain object.`);
  }

  return value;
}

function assertKeys(
  record: Record<string, unknown>,
  required: readonly string[],
  optional: readonly string[],
  label: string,
): void {
  const allowed = new Set([...required, ...optional]);
  for (const key of required) {
    if (!hasOwn(record, key)) {
      throw new Error(`${label} is missing ${key}.`);
    }
  }
  for (const key of Object.keys(record)) {
    if (!allowed.has(key)) {
      throw new Error(`${label} contains unsupported field ${key}.`);
    }
  }
}

function validateName(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${label} must be a non-empty string.`);
  }
  if (CONTROL_CHARACTER_PATTERN.test(value)) {
    throw new Error(`${label} cannot contain control characters.`);
  }

  const codePoints = Array.from(value).length;
  const bytes = new TextEncoder().encode(value);
  if (codePoints > PORTABLE_REPLAY_MAX_LABEL_CODE_POINTS) {
    throw new Error(`${label} exceeds ${PORTABLE_REPLAY_MAX_LABEL_CODE_POINTS} Unicode code points.`);
  }
  if (bytes.byteLength > PORTABLE_REPLAY_MAX_LABEL_UTF8_BYTES) {
    throw new Error(`${label} exceeds ${PORTABLE_REPLAY_MAX_LABEL_UTF8_BYTES} UTF-8 bytes.`);
  }
  if (new TextDecoder("utf-8", { fatal: true }).decode(bytes) !== value) {
    throw new Error(`${label} is not valid Unicode text.`);
  }

  return value;
}

function validateBotConfig(value: unknown): BotConfig {
  if (!isRecord(value) || !isBotConfig(value)) {
    throw new Error("Bot config has an unsupported shape or version.");
  }
  if (value.mode === "custom" && !isBotWidthAllowed(value.depth, value.width)) {
    throw new Error("Custom bot config uses an unsupported depth and width combination.");
  }

  return value.mode === "preset"
    ? { mode: "preset", preset: value.preset, version: 1 }
    : {
      mode: "custom",
      depth: value.depth,
      extraPass: value.extraPass,
      scoring: value.scoring,
      version: 1,
      width: value.width,
    };
}

function validateParticipant(value: unknown, label: string): PortableReplayParticipantV1 {
  const record = requireRecord(value, label);
  if (record.kind === "human") {
    assertKeys(record, ["kind", "name"], ["profileUid"], label);
    const name = validateName(record.name, `${label}.name`);
    if (hasOwn(record, "profileUid")) {
      if (typeof record.profileUid !== "string" || !PROFILE_UID_PATTERN.test(record.profileUid)) {
        throw new Error(`${label}.profileUid must be 1 to ${PORTABLE_REPLAY_MAX_UID_LENGTH} URL-safe characters.`);
      }
      return { kind: "human", name, profileUid: record.profileUid };
    }
    return { kind: "human", name };
  }

  assertKeys(record, ["kind", "name", "bot"], [], label);
  if (record.kind !== "bot") {
    throw new Error(`${label}.kind must be human or bot.`);
  }
  const bot = requireRecord(record.bot, `${label}.bot`);
  assertKeys(bot, ["engine", "version", "configVersion", "config"], [], `${label}.bot`);
  if (
    bot.engine !== BOT_ENGINE
    || bot.version !== BOT_VERSION
    || bot.configVersion !== BOT_CONFIG_VERSION
  ) {
    throw new Error(`${label}.bot uses an unsupported engine or version.`);
  }

  return {
    bot: {
      config: validateBotConfig(bot.config),
      configVersion: BOT_CONFIG_VERSION,
      engine: BOT_ENGINE,
      version: BOT_VERSION,
    },
    kind: "bot",
    name: validateName(record.name, `${label}.name`),
  };
}

function validatePractice(value: unknown, moveCount: number): PortableReplayPracticeOriginV1 {
  const record = requireRecord(value, "practice");
  assertKeys(record, ["sourceFingerprint", "inheritedPly"], [], "practice");
  if (typeof record.sourceFingerprint !== "string" || !FINGERPRINT_PATTERN.test(record.sourceFingerprint)) {
    throw new Error("practice.sourceFingerprint must be a 43-character SHA-256 base64url fingerprint.");
  }
  if (
    typeof record.inheritedPly !== "number"
    || !Number.isInteger(record.inheritedPly)
    || record.inheritedPly < PORTABLE_REPLAY_MIN_PRACTICE_PLY
    || record.inheritedPly >= moveCount
  ) {
    throw new Error(
      `practice.inheritedPly must be at least ${PORTABLE_REPLAY_MIN_PRACTICE_PLY} and before the final move.`,
    );
  }

  return {
    sourceFingerprint: record.sourceFingerprint,
    inheritedPly: record.inheritedPly,
  };
}

function normalizePortableReplay(value: unknown): PortableReplayV1 {
  const record = requireRecord(value, "portable replay");
  assertKeys(
    record,
    ["version", "ruleset", "moves", "black", "white"],
    ["practice", "viewPly"],
    "portable replay",
  );
  if (record.version !== PORTABLE_REPLAY_VERSION) {
    throw new Error(`Unsupported portable replay version: ${String(record.version)}.`);
  }
  if (record.ruleset !== "freestyle" && record.ruleset !== "renju") {
    throw new Error("Portable replay ruleset must be freestyle or renju.");
  }
  if (!Array.isArray(record.moves)) {
    throw new Error("Portable replay moves must be an array of cell bytes.");
  }
  if (record.moves.length < PORTABLE_REPLAY_MIN_MOVES || record.moves.length > PORTABLE_REPLAY_MAX_MOVES) {
    throw new Error(
      `Portable replay must contain ${PORTABLE_REPLAY_MIN_MOVES} to ${PORTABLE_REPLAY_MAX_MOVES} moves.`,
    );
  }

  const seen = new Set<number>();
  const moves = record.moves.map((cell, index) => {
    if (typeof cell !== "number" || !Number.isInteger(cell) || cell < 0 || cell >= 225) {
      throw new Error(`Portable replay move ${index + 1} must be a cell from 0 through 224.`);
    }
    if (seen.has(cell)) {
      throw new Error(`Portable replay repeats cell ${cell} at ply ${index + 1}.`);
    }
    seen.add(cell);
    return cell;
  });

  const black = validateParticipant(record.black, "black");
  const white = validateParticipant(record.white, "white");
  const practice = hasOwn(record, "practice")
    ? validatePractice(record.practice, moves.length)
    : undefined;
  let viewPly: number | undefined;
  if (hasOwn(record, "viewPly")) {
    if (
      typeof record.viewPly !== "number"
      || !Number.isInteger(record.viewPly)
      || record.viewPly < 0
      || record.viewPly > moves.length
    ) {
      throw new Error("Portable replay viewPly must be an integer between 0 and the move count.");
    }
    viewPly = record.viewPly;
  }

  return {
    black,
    moves,
    ...(practice ? { practice } : {}),
    ruleset: record.ruleset,
    version: PORTABLE_REPLAY_VERSION,
    ...(viewPly === undefined ? {} : { viewPly }),
    white,
  };
}

function savedMatchStatus(result: string): SavedMatchStatus | null {
  switch (result) {
    case "black":
      return "black_won";
    case "white":
      return "white_won";
    case "draw":
      return "draw";
    default:
      return null;
  }
}

/** Validates strict metadata first, then applies every move through Rust/Wasm. */
export function validatePortableReplay(value: unknown): ValidatedPortableReplayV1 {
  const replay = normalizePortableReplay(value);
  const board = createWasmBoard(replay.ruleset);

  try {
    const inheritedPly = replay.practice?.inheritedPly;
    for (let index = 0; index < replay.moves.length; index += 1) {
      const cell = replay.moves[index]!;
      const result = applyWasmMove(board, Math.floor(cell / 15), cell % 15);
      if (result.error) {
        throw new Error(`Portable replay has an illegal move at ply ${index + 1}: ${result.error}`);
      }
      if (result.result === null) {
        throw new Error(`Portable replay has no core result at ply ${index + 1}.`);
      }
      if (inheritedPly === index + 1 && result.result !== "ongoing") {
        throw new Error("Practice inheritedPly must point to a legal nonterminal position.");
      }
    }

    const status = savedMatchStatus(board.result());
    if (!status) {
      throw new Error("Portable replay must end with a win or a full-board draw.");
    }
    return { replay, status };
  } finally {
    board.free();
  }
}

function participantFromSavedMatch(
  player: SavedMatchV2["player_black"],
): PortableReplayParticipantV1 {
  if (player.kind === "human") {
    return {
      kind: "human",
      name: player.display_name,
      ...(player.profile_uid === null ? {} : { profileUid: player.profile_uid }),
    };
  }

  if (!player.bot) {
    throw new Error("Saved match bot is missing its identity snapshot.");
  }
  return {
    bot: {
      config: player.bot.config,
      configVersion: player.bot.config_version,
      engine: player.bot.engine,
      version: player.bot.version,
    },
    kind: "bot",
    name: player.display_name,
  };
}

/** Whitelists portable fields from a saved match and verifies its stored result. */
export function portableReplayFromSavedMatch(
  value: unknown,
  viewPly?: number,
): ValidatedPortableReplayV1 {
  if (!isSavedMatchV2(value)) {
    throw new Error("Saved match has an unsupported or invalid schema.");
  }

  const replay: PortableReplayV1 = {
    black: participantFromSavedMatch(value.player_black),
    moves: value.move_cells,
    ruleset: value.ruleset,
    version: PORTABLE_REPLAY_VERSION,
    ...(viewPly === undefined ? {} : { viewPly }),
    white: participantFromSavedMatch(value.player_white),
  };
  const validated = validatePortableReplay(replay);
  if (validated.status !== value.status) {
    throw new Error(
      `Saved match status ${value.status} does not match the core result ${validated.status}.`,
    );
  }
  return validated;
}
