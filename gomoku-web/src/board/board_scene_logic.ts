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
