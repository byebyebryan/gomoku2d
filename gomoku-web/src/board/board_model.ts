import type {
  BoardAnalysisHighlightRole,
  BoardAnalysisMarkerRole,
  BoardAnalysisOverlay,
  BoardTouchControlMode,
} from "./board_scene_logic";
import { BOARD_SIZE } from "./constants";
import type { CellPosition, CellStone, MatchMove, MatchStatus } from "../game/types";

export type BoardEvidenceRole = "counterThreat" | "immediateThreat" | "imminentThreat" | "winning";
export type BoardHintRole = "counterThreat" | "immediateThreat" | "imminentThreat" | "winning";

export interface BoardPosition {
  cells: CellStone[][];
  currentPlayer: 1 | 2;
  lastMove: CellPosition | null;
  moves: MatchMove[];
  showSequenceNumbers: boolean;
  status: MatchStatus;
}

export type BoardOverlay =
  | {
      cell: CellPosition;
      kind: "analysis";
      highlight?: BoardAnalysisHighlightRole;
      marker?: BoardAnalysisMarkerRole;
      side?: BoardAnalysisOverlay["side"];
    }
  | { cell: CellPosition; kind: "evidence"; role: BoardEvidenceRole }
  | { cell: CellPosition; kind: "hint"; role: BoardHintRole }
  | { cell: CellPosition; kind: "nextReplayMove" }
  | { cell: CellPosition; kind: "winningLine" };

export type BoardInteraction =
  | {
      interactive: boolean;
      kind: "play";
      onAdvanceRound: () => void;
      onPlace: (row: number, col: number) => void;
      onTouchCandidateChange: (candidate: CellPosition | null, canPlace: boolean) => void;
      touchCandidateResetVersion: number;
      touchControlMode: BoardTouchControlMode;
    }
  | { kind: "readonly" }
  | { kind: "replay" };

export interface BoardViewModel {
  boardSize: number;
  forbiddenMoves: CellPosition[];
  interaction: BoardInteraction;
  overlays: BoardOverlay[];
  position: BoardPosition;
}

export interface LocalMatchBoardHints {
  counterThreatEvidenceCells: CellPosition[];
  counterThreatMoves: CellPosition[];
  immediateThreatEvidenceCells: CellPosition[];
  imminentThreatEvidenceCells: CellPosition[];
  imminentThreatMoves: CellPosition[];
  threatMoves: CellPosition[];
  winningEvidenceCells: CellPosition[];
  winningMoves: CellPosition[];
}

export interface BuildLocalMatchBoardModelInput {
  forbiddenMoves: CellPosition[];
  hints: LocalMatchBoardHints;
  interaction: Extract<BoardInteraction, { kind: "play" }>;
  position: BoardPosition;
  winningCells: CellPosition[];
}

export interface BuildReplayBoardModelInput {
  analysisOverlays: BoardAnalysisOverlay[];
  nextReplayMove: CellPosition | null;
  position: BoardPosition;
  winningCells: CellPosition[];
}

export function buildLocalMatchBoardModel(input: BuildLocalMatchBoardModelInput): BoardViewModel {
  return {
    boardSize: BOARD_SIZE,
    forbiddenMoves: input.forbiddenMoves,
    interaction: input.interaction,
    overlays: [
      ...input.hints.winningEvidenceCells.map((cell) => evidenceOverlay(cell, "winning")),
      ...input.hints.immediateThreatEvidenceCells.map((cell) => evidenceOverlay(cell, "immediateThreat")),
      ...input.hints.imminentThreatEvidenceCells.map((cell) => evidenceOverlay(cell, "imminentThreat")),
      ...input.hints.counterThreatEvidenceCells.map((cell) => evidenceOverlay(cell, "counterThreat")),
      ...input.hints.winningMoves.map((cell) => hintOverlay(cell, "winning")),
      ...input.hints.threatMoves.map((cell) => hintOverlay(cell, "immediateThreat")),
      ...input.hints.imminentThreatMoves.map((cell) => hintOverlay(cell, "imminentThreat")),
      ...input.hints.counterThreatMoves.map((cell) => hintOverlay(cell, "counterThreat")),
      ...input.winningCells.map((cell) => winningLineOverlay(cell)),
    ],
    position: input.position,
  };
}

export function buildReplayBoardModel(input: BuildReplayBoardModelInput): BoardViewModel {
  return {
    boardSize: BOARD_SIZE,
    forbiddenMoves: [],
    interaction: { kind: "replay" },
    overlays: [
      ...input.analysisOverlays.map((overlay) => ({
        cell: { row: overlay.row, col: overlay.col },
        highlight: overlay.highlight,
        kind: "analysis" as const,
        marker: overlay.marker,
        side: overlay.side,
      })),
      ...(input.nextReplayMove ? [{ cell: input.nextReplayMove, kind: "nextReplayMove" as const }] : []),
      ...input.winningCells.map((cell) => winningLineOverlay(cell)),
    ],
    position: input.position,
  };
}

function evidenceOverlay(cell: CellPosition, role: BoardEvidenceRole): BoardOverlay {
  return { cell, kind: "evidence", role };
}

function hintOverlay(cell: CellPosition, role: BoardHintRole): BoardOverlay {
  return { cell, kind: "hint", role };
}

function winningLineOverlay(cell: CellPosition): BoardOverlay {
  return { cell, kind: "winningLine" };
}
