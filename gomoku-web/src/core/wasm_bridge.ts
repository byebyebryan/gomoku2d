import { WasmBoard, WasmBot, WasmReplayAnalyzer } from "gomoku-wasm";

import type { BotMove, BotSpec, GameVariant } from "./bot_protocol";
import type {
  ReplayAnalysisCounters,
  ReplayAnalysisMove,
  ReplayAnalysisStepResult,
} from "../replay/replay_analysis_protocol";

export type WasmGameResult = "black" | "draw" | "ongoing" | "white";

export interface WasmApplyMoveResult {
  error: string | null;
  result: WasmGameResult | null;
}

export interface WasmMove {
  row: number;
  col: number;
}

export interface WasmThreatSnapshot {
  counterThreatEvidenceCells: WasmMove[];
  counterThreatMoves: WasmMove[];
  forbiddenMoves: WasmMove[];
  immediateThreatEvidenceCells: WasmMove[];
  immediateThreatMoves: WasmMove[];
  imminentThreatEvidenceCells: WasmMove[];
  imminentThreatMoves: WasmMove[];
  winningEvidenceCells: WasmMove[];
  winningMoves: WasmMove[];
}

type BridgeValidator<T> = (value: unknown) => T;

function parseBridgeJson<T>(json: string, label: string, validate?: BridgeValidator<T>): T {
  let parsed: unknown;
  try {
    parsed = JSON.parse(json);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`Invalid WASM ${label} JSON: ${detail}`);
  }

  if (!validate) {
    return parsed as T;
  }

  try {
    return validate(parsed);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`Invalid WASM ${label} JSON: ${detail}`);
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function requireRecord(value: unknown, label: string): Record<string, unknown> {
  if (!isRecord(value)) {
    throw new Error(`${label} must be an object`);
  }

  return value;
}

function validateWasmMove(value: unknown, label = "move"): WasmMove {
  const record = requireRecord(value, label);
  const row = record.row;
  const col = record.col;

  if (typeof row !== "number" || typeof col !== "number" || !Number.isInteger(row) || !Number.isInteger(col)) {
    throw new Error(`${label} must include integer row and col`);
  }

  return { row, col };
}

function validateReplayAnalysisMove(value: unknown, label = "move"): ReplayAnalysisMove {
  return validateWasmMove(value, label);
}

function validateWasmMoveList(value: unknown, label: string): WasmMove[] {
  if (!Array.isArray(value)) {
    throw new Error(`${label} must be an array`);
  }

  return value.map((move, index) => validateWasmMove(move, `${label}[${index}]`));
}

function validateApplyMoveResult(value: unknown): WasmApplyMoveResult {
  const record = requireRecord(value, "apply move result");
  const error = record.error;
  const result = record.result;

  if (error !== null && typeof error !== "string") {
    throw new Error("apply move result.error must be a string or null");
  }

  if (result !== null && result !== "black" && result !== "draw" && result !== "ongoing" && result !== "white") {
    throw new Error("apply move result.result has an unknown game result");
  }

  return { error, result };
}

function validateThreatSnapshot(value: unknown): WasmThreatSnapshot {
  const record = requireRecord(value, "threat snapshot");

  return {
    counterThreatEvidenceCells: validateWasmMoveList(record.counterThreatEvidenceCells, "counterThreatEvidenceCells"),
    counterThreatMoves: validateWasmMoveList(record.counterThreatMoves, "counterThreatMoves"),
    forbiddenMoves: validateWasmMoveList(record.forbiddenMoves, "forbiddenMoves"),
    immediateThreatEvidenceCells: validateWasmMoveList(
      record.immediateThreatEvidenceCells,
      "immediateThreatEvidenceCells",
    ),
    immediateThreatMoves: validateWasmMoveList(record.immediateThreatMoves, "immediateThreatMoves"),
    imminentThreatEvidenceCells: validateWasmMoveList(
      record.imminentThreatEvidenceCells,
      "imminentThreatEvidenceCells",
    ),
    imminentThreatMoves: validateWasmMoveList(record.imminentThreatMoves, "imminentThreatMoves"),
    winningEvidenceCells: validateWasmMoveList(record.winningEvidenceCells, "winningEvidenceCells"),
    winningMoves: validateWasmMoveList(record.winningMoves, "winningMoves"),
  };
}

function validateBotMove(value: unknown): BotMove | null {
  if (value === null) {
    return null;
  }

  return validateWasmMove(value, "bot move");
}

function validateReplayAnalysisCounters(value: unknown): ReplayAnalysisCounters {
  const record = requireRecord(value, "replay analysis counters");
  for (const field of ["branch_roots", "prefixes_analyzed", "proof_nodes"]) {
    if (typeof record[field] !== "number") {
      throw new Error(`replay analysis counters.${field} must be a number`);
    }
  }
  return record as unknown as ReplayAnalysisCounters;
}

function validateReplayAnalysisStep(value: unknown): ReplayAnalysisStepResult {
  const record = requireRecord(value, "replay analysis step");
  const status = record.status;
  if (
    status !== "running" &&
    status !== "resolved" &&
    status !== "unclear" &&
    status !== "unsupported" &&
    status !== "error"
  ) {
    throw new Error("replay analysis step.status has an unknown value");
  }
  if (typeof record.schema_version !== "number") {
    throw new Error("replay analysis step.schema_version must be a number");
  }
  if (typeof record.done !== "boolean") {
    throw new Error("replay analysis step.done must be a boolean");
  }
  if (record.current_ply !== null && typeof record.current_ply !== "number") {
    throw new Error("replay analysis step.current_ply must be a number or null");
  }
  if (record.error !== null && typeof record.error !== "string") {
    throw new Error("replay analysis step.error must be a string or null");
  }
  if (!Array.isArray(record.annotations)) {
    throw new Error("replay analysis step.annotations must be an array");
  }
  for (const [index, annotation] of record.annotations.entries()) {
    validateReplayFrameAnnotations(annotation, `replay analysis step.annotations[${index}]`);
  }
  validateReplayAnalysisCounters(record.counters);
  if (record.analysis !== null) {
    requireRecord(record.analysis, "replay analysis step.analysis");
  }
  return record as unknown as ReplayAnalysisStepResult;
}

function validateReplayFrameAnnotations(value: unknown, label: string): void {
  const record = requireRecord(value, label);
  if (typeof record.ply !== "number") {
    throw new Error(`${label}.ply must be a number`);
  }
  if (record.side_to_move !== "Black" && record.side_to_move !== "White") {
    throw new Error(`${label}.side_to_move has an unknown value`);
  }
  validateReplayFrameHintList(record.highlights, `${label}.highlights`);
  validateReplayFrameHintList(record.markers, `${label}.markers`);
  if (record.evidence !== undefined) {
    validateReplayFrameHintList(record.evidence, `${label}.evidence`);
  }
}

function validateReplayFrameHintList(value: unknown, label: string): void {
  if (!Array.isArray(value)) {
    throw new Error(`${label} must be an array`);
  }
  for (const [index, hint] of value.entries()) {
    const record = requireRecord(hint, `${label}[${index}]`);
    validateReplayAnalysisMove(record.mv, `${label}[${index}].mv`);
    if (typeof record.notation !== "string") {
      throw new Error(`${label}[${index}].notation must be a string`);
    }
    if (typeof record.role !== "string") {
      throw new Error(`${label}[${index}].role must be a string`);
    }
    if (record.side !== "Black" && record.side !== "White") {
      throw new Error(`${label}[${index}].side has an unknown value`);
    }
  }
}

export function createWasmBoard(variant: GameVariant): WasmBoard {
  return WasmBoard.createWithVariant(variant);
}

export function wasmBoardFromFenWithVariant(fen: string, variant: GameVariant): WasmBoard {
  return WasmBoard.fromFenWithVariant(fen, variant);
}

export function applyWasmMove(board: WasmBoard, row: number, col: number): WasmApplyMoveResult {
  return parseBridgeJson(board.applyMove(row, col), "apply move result", validateApplyMoveResult);
}

export function readWasmThreatSnapshot(board: WasmBoard): WasmThreatSnapshot {
  return parseBridgeJson(board.threatSnapshot(), "threat snapshot", validateThreatSnapshot);
}

export function readWasmWinningCells(board: WasmBoard): WasmMove[] {
  return parseBridgeJson(board.winningCells(), "winning cells", (value) => validateWasmMoveList(value, "winning cells"));
}

export function createWasmBotFromSpec(spec: BotSpec): WasmBot | null {
  return spec.kind === "human" ? null : WasmBot.createFromSpec(JSON.stringify(spec));
}

export function chooseWasmBotMove(bot: WasmBot, board: WasmBoard): BotMove | null {
  return parseBridgeJson(bot.chooseMove(board), "bot move", validateBotMove);
}

export function parseWasmReplayAnalysisStep(json: string): ReplayAnalysisStepResult {
  return parseBridgeJson(json, "replay analysis step", validateReplayAnalysisStep);
}

export { WasmBoard, WasmBot, WasmReplayAnalyzer };
