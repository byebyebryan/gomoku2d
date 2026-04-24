import type { CellPosition, CellStone, MatchMove, MatchStatus } from "../game/types";

import { HOVER_ANIMS, SPRITE, WARNING_ANIMS } from "./constants";

const TOUCH_DRAG_SENSITIVITY = 1.0;
const DEFAULT_BOARD_SIZE = 15;
const SEQUENCE_FONT_DESKTOP_CELL_SIZE = 40;
const SEQUENCE_FONT_MOBILE_SIZE = 8;
const SEQUENCE_FONT_DESKTOP_SIZE = 16;

export type WarningOverlayRole = "forbidden" | "threatMove" | "winningMove" | "winningLine";
export type PointerCue = "blocked" | "hidden" | "normal" | "preferred";

export type SpriteFrameTarget = {
  frame: number;
  texture: string;
};

export type ResettableSprite = {
  active?: boolean;
  scene?: unknown;
  setFrame: (frame: number) => unknown;
  setTexture: (texture: string, frame: number) => unknown;
  stop: () => unknown;
  texture: { key: string };
};

export type BoardOverlayState = {
  cells: CellStone[][];
  forbiddenMoves: CellPosition[];
  moves: MatchMove[];
  showSequenceNumbers: boolean;
  status: MatchStatus;
  threatMoves: CellPosition[];
  winningCells: CellPosition[];
  winningMoves: CellPosition[];
};

function cellPositionsEqual(a: CellPosition[], b: CellPosition[]): boolean {
  if (a.length !== b.length) {
    return false;
  }

  return a.every((cell, index) => cell.row === b[index]?.row && cell.col === b[index]?.col);
}

function movesEqual(a: MatchMove[], b: MatchMove[]): boolean {
  if (a.length !== b.length) {
    return false;
  }

  return a.every((move, index) => {
    const other = b[index];
    return (
      move.row === other?.row &&
      move.col === other.col &&
      move.moveNumber === other.moveNumber &&
      move.player === other.player
    );
  });
}

function cellsEqual(a: CellStone[][], b: CellStone[][]): boolean {
  if (a.length !== b.length) {
    return false;
  }

  return a.every((row, rowIndex) => {
    const otherRow = b[rowIndex];
    return row.length === otherRow?.length && row.every((cell, colIndex) => cell === otherRow[colIndex]);
  });
}

export function warningAnimationForOverlay(role: WarningOverlayRole, isForbidden = false): string {
  switch (role) {
    case "forbidden":
      return WARNING_ANIMS.FORBIDDEN_OUT.key;
    case "threatMove":
      return isForbidden ? WARNING_ANIMS.WARNING_ON_FORBIDDEN.key : WARNING_ANIMS.WARNING.key;
    case "winningMove":
      return WARNING_ANIMS.WARNING.key;
    case "winningLine":
      return HOVER_ANIMS.HOVER.key;
  }
}

export function warningSpriteForOverlay(role: WarningOverlayRole): string {
  return role === "winningLine" ? SPRITE.HOVER : SPRITE.WARNING;
}

export function shouldRenderStandaloneForbiddenOverlay(
  forbiddenCell: CellPosition,
  threatMoves: CellPosition[],
): boolean {
  return !threatMoves.some((cell) => cell.row === forbiddenCell.row && cell.col === forbiddenCell.col);
}

export function sequenceNumberFontSize(cellSize: number): number {
  return cellSize >= SEQUENCE_FONT_DESKTOP_CELL_SIZE
    ? SEQUENCE_FONT_DESKTOP_SIZE
    : SEQUENCE_FONT_MOBILE_SIZE;
}

export function sequenceNumberPosition(x: number, y: number): { x: number; y: number } {
  return {
    x: Math.round(x),
    y: Math.round(y),
  };
}

export function shouldAnimatePlacedStone(
  isNewStone: boolean,
  animateNewStones: boolean,
  status: MatchStatus,
): boolean {
  return animateNewStones && isNewStone && status === "playing";
}

export function shouldStopStoneIdleCycle(
  previousStatus: MatchStatus,
  nextStatus: MatchStatus,
): boolean {
  return previousStatus === "playing" && nextStatus !== "playing";
}

export function shouldStopStoneCycleBeforeStoneRemoval(removedStoneCount: number): boolean {
  return removedStoneCount > 0;
}

export function shouldSyncOverlaySprites(
  previous: BoardOverlayState | undefined,
  next: BoardOverlayState,
): boolean {
  if (!previous) {
    return true;
  }

  return (
    previous.showSequenceNumbers !== next.showSequenceNumbers ||
    previous.status !== next.status ||
    !cellsEqual(previous.cells, next.cells) ||
    !movesEqual(previous.moves, next.moves) ||
    !cellPositionsEqual(previous.forbiddenMoves, next.forbiddenMoves) ||
    !cellPositionsEqual(previous.threatMoves, next.threatMoves) ||
    !cellPositionsEqual(previous.winningMoves, next.winningMoves) ||
    !cellPositionsEqual(previous.winningCells, next.winningCells)
  );
}

export function shouldRestartPointerCycle(
  previousCellKey: string | null,
  nextCellKey: string | null,
  pointerVisible: boolean,
): boolean {
  if (nextCellKey === null) {
    return false;
  }

  return !pointerVisible || previousCellKey !== nextCellKey;
}

export function touchDragSteps(delta: number, cellSize: number): number {
  const safeCellSize = Math.max(1, cellSize);
  const threshold = safeCellSize * TOUCH_DRAG_SENSITIVITY;
  const steps = Math.trunc(delta / threshold);
  return Object.is(steps, -0) ? 0 : steps;
}

export function moveTouchCandidateFromDrag(
  origin: CellPosition,
  deltaX: number,
  deltaY: number,
  cellSize: number,
  boardSize = DEFAULT_BOARD_SIZE,
): CellPosition {
  const clamp = (value: number) => Math.max(0, Math.min(boardSize - 1, value));

  return {
    row: clamp(origin.row + touchDragSteps(deltaY, cellSize)),
    col: clamp(origin.col + touchDragSteps(deltaX, cellSize)),
  };
}

export function canPlaceTouchCandidate(
  cells: CellStone[][],
  forbiddenMoves: CellPosition[],
  candidate: CellPosition | null,
): boolean {
  if (!candidate) {
    return false;
  }

  if (cells[candidate.row]?.[candidate.col] !== null) {
    return false;
  }

  return !forbiddenMoves.some((cell) => cell.row === candidate.row && cell.col === candidate.col);
}

export function pointerCueForCandidate(
  cells: CellStone[][],
  forbiddenMoves: CellPosition[],
  preferredMoves: CellPosition[],
  candidate: CellPosition | null,
  showBlockedOccupied: boolean,
): PointerCue {
  if (!candidate) {
    return "hidden";
  }

  const isForbidden = forbiddenMoves.some((cell) => (
    cell.row === candidate.row && cell.col === candidate.col
  ));

  if (isForbidden) {
    return "blocked";
  }

  if (cells[candidate.row]?.[candidate.col] !== null) {
    return showBlockedOccupied ? "blocked" : "hidden";
  }

  const isPreferred = preferredMoves.some((cell) => (
    cell.row === candidate.row && cell.col === candidate.col
  ));

  return isPreferred ? "preferred" : "normal";
}

export function resetSpriteToFrame(sprite: ResettableSprite, frame: SpriteFrameTarget): void {
  if (sprite.active === false || ("scene" in sprite && !sprite.scene)) {
    return;
  }

  sprite.stop();

  if (sprite.texture.key !== frame.texture) {
    sprite.setTexture(frame.texture, frame.frame);
    return;
  }

  sprite.setFrame(frame.frame);
}
