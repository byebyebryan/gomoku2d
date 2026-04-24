import { describe, expect, it } from "vitest";

import type { CellStone } from "../game/types";

import {
  canPlaceTouchCandidate,
  moveTouchCandidateFromDrag,
  shouldAnimatePlacedStone,
  shouldRestartPointerCycle,
  shouldStopStoneIdleCycle,
  touchDragSteps,
  warningAnimationForOverlay,
} from "./board_scene_logic";

describe("shouldAnimatePlacedStone", () => {
  it("animates new stones while the match is still playing", () => {
    expect(shouldAnimatePlacedStone(true, true, "playing")).toBe(true);
  });

  it("keeps the final move static once the match has concluded", () => {
    expect(shouldAnimatePlacedStone(true, true, "black_won")).toBe(false);
    expect(shouldAnimatePlacedStone(true, true, "white_won")).toBe(false);
    expect(shouldAnimatePlacedStone(true, true, "draw")).toBe(false);
  });

  it("does not animate existing stones or non-animated updates", () => {
    expect(shouldAnimatePlacedStone(false, true, "playing")).toBe(false);
    expect(shouldAnimatePlacedStone(true, false, "playing")).toBe(false);
  });
});

describe("shouldStopStoneIdleCycle", () => {
  it("stops the active idle animation when the match concludes", () => {
    expect(shouldStopStoneIdleCycle("playing", "black_won")).toBe(true);
    expect(shouldStopStoneIdleCycle("playing", "white_won")).toBe(true);
    expect(shouldStopStoneIdleCycle("playing", "draw")).toBe(true);
  });

  it("keeps the idle cycle unchanged for non-terminal transitions", () => {
    expect(shouldStopStoneIdleCycle("playing", "playing")).toBe(false);
    expect(shouldStopStoneIdleCycle("white_won", "white_won")).toBe(false);
    expect(shouldStopStoneIdleCycle("black_won", "playing")).toBe(false);
  });
});

describe("shouldRestartPointerCycle", () => {
  it("starts the cycle when the pointer first becomes visible on a legal cell", () => {
    expect(shouldRestartPointerCycle(null, "7,7", false)).toBe(true);
  });

  it("restarts the cycle when the hovered cell changes", () => {
    expect(shouldRestartPointerCycle("7,7", "7,8", true)).toBe(true);
  });

  it("does not restart the cycle for pointer jitter within the same cell", () => {
    expect(shouldRestartPointerCycle("7,7", "7,7", true)).toBe(false);
  });

  it("does not restart the cycle when there is no valid hovered cell", () => {
    expect(shouldRestartPointerCycle("7,7", null, true)).toBe(false);
  });
});

describe("touchDragSteps", () => {
  it("requires a full cell of drag before moving a cell", () => {
    expect(touchDragSteps(39, 40)).toBe(0);
    expect(touchDragSteps(40, 40)).toBe(1);
    expect(touchDragSteps(-39, 40)).toBe(0);
    expect(touchDragSteps(-40, 40)).toBe(-1);
  });

  it("accumulates multiple cell steps for larger drags", () => {
    expect(touchDragSteps(89, 40)).toBe(2);
    expect(touchDragSteps(-90, 40)).toBe(-2);
  });
});

describe("moveTouchCandidateFromDrag", () => {
  it("moves the candidate relative to its starting cell", () => {
    expect(moveTouchCandidateFromDrag({ row: 7, col: 7 }, 90, -30, 40)).toEqual({
      row: 7,
      col: 9,
    });
  });

  it("clamps the candidate inside the board", () => {
    expect(moveTouchCandidateFromDrag({ row: 0, col: 0 }, -120, -120, 40)).toEqual({
      row: 0,
      col: 0,
    });
    expect(moveTouchCandidateFromDrag({ row: 14, col: 14 }, 120, 120, 40)).toEqual({
      row: 14,
      col: 14,
    });
  });
});

describe("canPlaceTouchCandidate", () => {
  it("allows empty, non-forbidden cells", () => {
    const cells: CellStone[][] = Array.from({ length: 15 }, () =>
      Array.from({ length: 15 }, () => null),
    );

    expect(canPlaceTouchCandidate(cells, [], { row: 7, col: 7 })).toBe(true);
  });

  it("blocks occupied or forbidden cells", () => {
    const cells: CellStone[][] = Array.from({ length: 15 }, () =>
      Array.from({ length: 15 }, () => null),
    );
    cells[7][7] = 1;

    expect(canPlaceTouchCandidate(cells, [], { row: 7, col: 7 })).toBe(false);
    expect(canPlaceTouchCandidate(cells, [{ row: 8, col: 8 }], { row: 8, col: 8 })).toBe(false);
  });
});

describe("warningAnimationForOverlay", () => {
  it("uses hover warnings for tactical hints and surface warnings for forbidden moves", () => {
    expect(warningAnimationForOverlay("tacticalHint")).toBe("warning-hover");
    expect(warningAnimationForOverlay("winningLine")).toBe("warning-hover");
    expect(warningAnimationForOverlay("forbidden")).toBe("warning-forbidden");
  });
});
