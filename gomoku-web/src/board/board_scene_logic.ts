import type { MatchStatus } from "../game/types";

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
