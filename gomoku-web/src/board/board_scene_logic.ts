import type { CellPosition, CellStone, MatchStatus } from "../game/types";

import { WARNING_ANIMS } from "./constants";

const TOUCH_DRAG_SENSITIVITY = 1.0;
const DEFAULT_BOARD_SIZE = 15;

export type WarningOverlayRole = "forbidden" | "tacticalHint" | "winningLine";

export function warningAnimationForOverlay(role: WarningOverlayRole): string {
  switch (role) {
    case "forbidden":
      return WARNING_ANIMS.FORBIDDEN.key;
    case "tacticalHint":
    case "winningLine":
      return WARNING_ANIMS.HOVER.key;
  }
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
