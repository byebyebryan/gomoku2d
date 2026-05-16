import type { CellPosition, CellStone, MatchMove, MatchStatus } from "../game/types";

import { CAUTION_ANIMS, HOVER_ANIMS, MARKER_ANIMS, SPRITE } from "./constants";

const TOUCH_DRAG_SENSITIVITY = 1.0;
const DEFAULT_BOARD_SIZE = 15;
const SEQUENCE_FONT_DESKTOP_CELL_SIZE = 40;
const SEQUENCE_FONT_MOBILE_SIZE = 8;
const SEQUENCE_FONT_DESKTOP_SIZE = 16;

export type BoardOverlayRole =
  | "counterThreatMove"
  | "forbidden"
  | "imminentThreatMove"
  | "threatMove"
  | "winningMove"
  | "winningLine";
export type PointerCue = "blocked" | "hidden" | "normal" | "preferred";
export type BoardTouchControlMode = "none" | "pointer" | "touchpad";

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
  counterThreatMoves: CellPosition[];
  forbiddenMoves: CellPosition[];
  imminentThreatMoves: CellPosition[];
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

export function overlayAnimationForRole(role: BoardOverlayRole, isForbidden = false): string {
  switch (role) {
    case "counterThreatMove":
      return MARKER_ANIMS.WARNING.key;
    case "forbidden":
      return CAUTION_ANIMS.FORBIDDEN_OUT.key;
    case "imminentThreatMove":
      return MARKER_ANIMS.WARNING.key;
    case "threatMove":
      return isForbidden ? CAUTION_ANIMS.FORBIDDEN_WARNING.key : MARKER_ANIMS.WARNING.key;
    case "winningMove":
      return MARKER_ANIMS.WARNING.key;
    case "winningLine":
      return HOVER_ANIMS.HOVER.key;
  }
}

export function overlaySpriteForRole(role: BoardOverlayRole, isForbidden = false): string {
  if (role === "winningLine") {
    return SPRITE.HOVER;
  }

  if (role === "forbidden" || (role === "threatMove" && isForbidden)) {
    return SPRITE.CAUTION;
  }

  return SPRITE.MARKER;
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
    !cellPositionsEqual(previous.counterThreatMoves, next.counterThreatMoves) ||
    !movesEqual(previous.moves, next.moves) ||
    !cellPositionsEqual(previous.forbiddenMoves, next.forbiddenMoves) ||
    !cellPositionsEqual(previous.imminentThreatMoves, next.imminentThreatMoves) ||
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

export function moveTouchCandidateFromPointerMove(
  mode: BoardTouchControlMode,
  origin: CellPosition,
  pointerCell: CellPosition | null,
  deltaX: number,
  deltaY: number,
  cellSize: number,
  boardSize = DEFAULT_BOARD_SIZE,
): CellPosition | null {
  if (mode === "pointer") {
    return pointerCell;
  }

  if (mode === "touchpad") {
    return moveTouchCandidateFromDrag(origin, deltaX, deltaY, cellSize, boardSize);
  }

  return null;
}

export function usesTouchCandidate(mode: BoardTouchControlMode): boolean {
  return mode !== "none";
}

export function usesTouchpadDrag(mode: BoardTouchControlMode): boolean {
  return mode === "touchpad";
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
