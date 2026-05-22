import { WasmBoard, WasmBot, WasmReplayAnalyzer } from "gomoku-wasm";

import type { BotMove, BotSpec, GameVariant } from "./bot_protocol";

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

function parseBridgeJson<T>(json: string, label: string): T {
  try {
    return JSON.parse(json) as T;
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`Invalid WASM ${label} JSON: ${detail}`);
  }
}

export function createWasmBoard(variant: GameVariant): WasmBoard {
  return WasmBoard.createWithVariant(variant);
}

export function wasmBoardFromFenWithVariant(fen: string, variant: GameVariant): WasmBoard {
  return WasmBoard.fromFenWithVariant(fen, variant);
}

export function applyWasmMove(board: WasmBoard, row: number, col: number): WasmApplyMoveResult {
  return parseBridgeJson<WasmApplyMoveResult>(board.applyMove(row, col), "apply move result");
}

export function readWasmThreatSnapshot(board: WasmBoard): WasmThreatSnapshot {
  return parseBridgeJson<WasmThreatSnapshot>(board.threatSnapshot(), "threat snapshot");
}

export function readWasmWinningCells(board: WasmBoard): WasmMove[] {
  return parseBridgeJson<WasmMove[]>(board.winningCells(), "winning cells");
}

export function createWasmBotFromSpec(spec: BotSpec): WasmBot | null {
  return spec.kind === "human" ? null : WasmBot.createFromSpec(JSON.stringify(spec));
}

export function chooseWasmBotMove(bot: WasmBot, board: WasmBoard): BotMove | null {
  return parseBridgeJson<BotMove | null>(bot.chooseMove(board), "bot move");
}

export function parseWasmReplayAnalysisStep<T>(json: string): T {
  return parseBridgeJson<T>(json, "replay analysis step");
}

export { WasmBoard, WasmBot, WasmReplayAnalyzer };
