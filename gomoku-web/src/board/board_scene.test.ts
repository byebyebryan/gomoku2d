import { describe, expect, it } from "vitest";

import type { CellStone } from "../game/types";

import {
  BOARD_RENDER_DEPTHS,
  BOARD_RENDER_LAYER_ORDER,
  HOVER_ANIMS,
  POINTER_ANIMS,
  SPRITE,
  SPRITESHEET_CONFIG,
  STONE_ANIMS,
  TRANSFORM_ANIMS,
  WARNING_ANIMS,
} from "./constants";
import {
  canPlaceTouchCandidate,
  moveTouchCandidateFromDrag,
  pointerCueForCandidate,
  resetSpriteToFrame,
  sequenceNumberFontSize,
  sequenceNumberPosition,
  shouldAnimatePlacedStone,
  shouldSyncOverlaySprites,
  shouldStopStoneCycleBeforeStoneRemoval,
  shouldRestartPointerCycle,
  shouldStopStoneIdleCycle,
  touchDragSteps,
  warningAnimationForOverlay,
  warningSpriteForOverlay,
} from "./board_scene_logic";

const emptyCells = (): CellStone[][] => Array.from({ length: 15 }, () =>
  Array.from({ length: 15 }, () => null),
);

const overlayState = (overrides: Partial<Parameters<typeof shouldSyncOverlaySprites>[1]> = {}) => ({
  cells: emptyCells(),
  forbiddenMoves: [{ row: 8, col: 8 }],
  moves: [{ row: 7, col: 7, moveNumber: 1, player: 1 as const }],
  showSequenceNumbers: true,
  status: "playing" as const,
  threatMoves: [{ row: 9, col: 9 }],
  winningCells: [],
  winningMoves: [{ row: 10, col: 10 }],
  ...overrides,
});

describe("animation sheet inventory", () => {
  it("maps the row-based sprite sheets to frame ranges", () => {
    expect(SPRITESHEET_CONFIG).toEqual({
      [SPRITE.STONE]: { url: "assets/sprites/stone.png", end: 23 },
      [SPRITE.POINTER]: { url: "assets/sprites/pointer.png", end: 19 },
      [SPRITE.HOVER]: { url: "assets/sprites/hover.png", end: 5 },
      [SPRITE.WARNING]: { url: "assets/sprites/warning.png", end: 29 },
      [SPRITE.TRANSFORM]: { url: "assets/sprites/transform.png", end: 9 },
    });

    expect(STONE_ANIMS).toMatchObject({
      DESTROY: { start: 0, end: 3, frameRate: 12, key: "stone-destroy" },
      IDLE_1: { start: 0, end: 5, frameRate: 6, key: "stone-idle-1" },
      IDLE_2: { start: 6, end: 11, frameRate: 6, key: "stone-idle-2" },
      IDLE_3: { start: 12, end: 17, frameRate: 6, key: "stone-idle-3" },
      IDLE_4: { start: 18, end: 23, frameRate: 6, key: "stone-idle-4" },
    });

    expect(POINTER_ANIMS).toMatchObject({
      IDLE_1: { start: 0, end: 5, frameRate: 12, key: "pointer-idle-1" },
      IDLE_2: { start: 6, end: 9, frameRate: 12, key: "pointer-idle-2" },
      IDLE_LONG: { start: 10, end: 19, frameRate: 12, key: "pointer-idle-long" },
    });

    expect(HOVER_ANIMS).toMatchObject({
      HOVER: { start: 0, end: 5, frameRate: 12, key: "warning-hover" },
    });

    expect(WARNING_ANIMS).toMatchObject({
      WARNING: { start: 0, end: 5, frameRate: 12, key: "warning" },
      WARNING_ON_FORBIDDEN: { start: 6, end: 11, frameRate: 12, key: "warning-on-forbidden" },
      FORBIDDEN_OUT: { start: 12, end: 17, frameRate: 12, key: "forbidden-out" },
      FORBIDDEN_IN: { start: 18, end: 23, frameRate: 12, key: "forbidden-in" },
      HIGHLIGHT: { start: 24, end: 29, frameRate: 12, key: "warning-highlight" },
    });

    expect(TRANSFORM_ANIMS).toMatchObject({
      FORM: { start: 0, end: 9, frameRate: 18, key: "transform-form" },
    });
    expect(TRANSFORM_ANIMS).not.toHaveProperty("DEFORM");
  });
});

describe("board render depths", () => {
  it("keeps render containers ordered from board surface up to hover warnings", () => {
    expect(BOARD_RENDER_LAYER_ORDER).toEqual([
      "BOARD",
      "WARNING",
      "POINTER",
      "STONE",
      "SEQUENCE_NUMBER",
      "HOVER",
    ]);
  });

  it("keeps warning surfaces below the pointer and hover warnings above result labels", () => {
    const depths = BOARD_RENDER_DEPTHS as Record<string, number>;

    expect(depths.BOARD).toBeLessThan(depths.WARNING_SURFACE);
    expect(depths.BOARD).toBeLessThan(depths.WARNING_BLOCKED);
    expect(depths.WARNING_SURFACE).toBeLessThan(depths.POINTER);
    expect(depths.WARNING_BLOCKED).toBeLessThan(depths.POINTER);
    expect(depths.POINTER).toBeLessThan(depths.STONE);
    expect(depths.STONE).toBeLessThan(depths.SEQUENCE_NUMBER);
    expect(depths.SEQUENCE_NUMBER).toBeLessThan(depths.WARNING_HOVER);
  });
});

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

describe("shouldStopStoneCycleBeforeStoneRemoval", () => {
  it("stops the idle owner before removed stones are destroyed", () => {
    expect(shouldStopStoneCycleBeforeStoneRemoval(1)).toBe(true);
    expect(shouldStopStoneCycleBeforeStoneRemoval(12)).toBe(true);
  });

  it("keeps the idle owner when syncing only added or unchanged stones", () => {
    expect(shouldStopStoneCycleBeforeStoneRemoval(0)).toBe(false);
  });
});

describe("shouldSyncOverlaySprites", () => {
  it("syncs overlays on the initial board render", () => {
    expect(shouldSyncOverlaySprites(undefined, overlayState())).toBe(true);
  });

  it("keeps warning animations alive when overlay data is unchanged across pointer-only rerenders", () => {
    const previous = overlayState();
    const next = overlayState({
      cells: emptyCells(),
      forbiddenMoves: [{ row: 8, col: 8 }],
      moves: [{ row: 7, col: 7, moveNumber: 1, player: 1 }],
      threatMoves: [{ row: 9, col: 9 }],
      winningMoves: [{ row: 10, col: 10 }],
    });

    expect(shouldSyncOverlaySprites(previous, next)).toBe(false);
  });

  it("syncs overlays when tactical warning cells change", () => {
    expect(shouldSyncOverlaySprites(
      overlayState(),
      overlayState({ threatMoves: [{ row: 9, col: 10 }] }),
    )).toBe(true);
  });

  it("syncs overlays when result sequence labels can change", () => {
    expect(shouldSyncOverlaySprites(
      overlayState({ status: "playing" }),
      overlayState({ status: "black_won" }),
    )).toBe(true);
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
  it("maps board warnings to the intended overlay language", () => {
    expect(warningAnimationForOverlay("winningLine")).toBe("warning-hover");
    expect(warningAnimationForOverlay("winningMove")).toBe("warning");
    expect(warningAnimationForOverlay("threatMove")).toBe("warning");
    expect(warningAnimationForOverlay("threatMove", true)).toBe("warning-on-forbidden");
    expect(warningAnimationForOverlay("forbidden")).toBe("forbidden-out");
  });
});

describe("warningSpriteForOverlay", () => {
  it("uses the dedicated hover sheet for winning-line hover warnings", () => {
    expect(warningSpriteForOverlay("winningLine")).toBe(SPRITE.HOVER);
  });

  it("uses the warning sheet for tactical and forbidden warnings", () => {
    expect(warningSpriteForOverlay("winningMove")).toBe(SPRITE.WARNING);
    expect(warningSpriteForOverlay("threatMove")).toBe(SPRITE.WARNING);
    expect(warningSpriteForOverlay("forbidden")).toBe(SPRITE.WARNING);
  });
});

describe("pointerCueForCandidate", () => {
  it("shows a normal cue on empty non-forbidden cells", () => {
    const cells: CellStone[][] = Array.from({ length: 15 }, () =>
      Array.from({ length: 15 }, () => null),
    );

    expect(pointerCueForCandidate(cells, [], [], { row: 7, col: 7 }, false)).toBe("normal");
  });

  it("shows a preferred cue on empty winning or threat cells", () => {
    const cells: CellStone[][] = Array.from({ length: 15 }, () =>
      Array.from({ length: 15 }, () => null),
    );

    expect(pointerCueForCandidate(
      cells,
      [],
      [{ row: 7, col: 7 }],
      { row: 7, col: 7 },
      false,
    )).toBe("preferred");
  });

  it("shows a blocked cue on forbidden cells", () => {
    const cells: CellStone[][] = Array.from({ length: 15 }, () =>
      Array.from({ length: 15 }, () => null),
    );

    expect(pointerCueForCandidate(cells, [{ row: 7, col: 7 }], [], { row: 7, col: 7 }, false)).toBe("blocked");
  });

  it("blocks forbidden preferred cells before applying the preferred cue", () => {
    const cells: CellStone[][] = Array.from({ length: 15 }, () =>
      Array.from({ length: 15 }, () => null),
    );

    expect(pointerCueForCandidate(
      cells,
      [{ row: 7, col: 7 }],
      [{ row: 7, col: 7 }],
      { row: 7, col: 7 },
      false,
    )).toBe("blocked");
  });

  it("hides occupied desktop hover cells but blocks occupied mobile touch candidates", () => {
    const cells: CellStone[][] = Array.from({ length: 15 }, () =>
      Array.from({ length: 15 }, () => null),
    );
    cells[7][7] = 1;

    expect(pointerCueForCandidate(cells, [], [], { row: 7, col: 7 }, false)).toBe("hidden");
    expect(pointerCueForCandidate(cells, [], [], { row: 7, col: 7 }, true)).toBe("blocked");
  });
});

describe("resetSpriteToFrame", () => {
  it("stops any active animation before resetting to the static frame", () => {
    const calls: string[] = [];
    const sprite = {
      texture: { key: SPRITE.STONE },
      setFrame: (frame: number) => {
        calls.push(`setFrame:${frame}`);
      },
      setTexture: (texture: string, frame: number) => {
        calls.push(`setTexture:${texture}:${frame}`);
      },
      stop: () => {
        calls.push("stop");
      },
    };

    resetSpriteToFrame(sprite, { texture: SPRITE.STONE, frame: STONE_ANIMS.STATIC.frame });

    expect(calls).toEqual(["stop", "setFrame:0"]);
  });

  it("also stops before swapping textures", () => {
    const calls: string[] = [];
    const sprite = {
      texture: { key: SPRITE.POINTER },
      setFrame: (frame: number) => {
        calls.push(`setFrame:${frame}`);
      },
      setTexture: (texture: string, frame: number) => {
        calls.push(`setTexture:${texture}:${frame}`);
      },
      stop: () => {
        calls.push("stop");
      },
    };

    resetSpriteToFrame(sprite, { texture: SPRITE.TRANSFORM, frame: 0 });

    expect(calls).toEqual(["stop", "setTexture:transform:0"]);
  });

  it("does not reset destroyed sprites", () => {
    const calls: string[] = [];
    const sprite = {
      active: false,
      scene: null,
      texture: { key: SPRITE.STONE },
      setFrame: (frame: number) => {
        calls.push(`setFrame:${frame}`);
      },
      setTexture: (texture: string, frame: number) => {
        calls.push(`setTexture:${texture}:${frame}`);
      },
      stop: () => {
        throw new Error("destroyed sprite stop should not be called");
      },
    };

    resetSpriteToFrame(sprite, { texture: SPRITE.STONE, frame: STONE_ANIMS.STATIC.frame });

    expect(calls).toEqual([]);
  });
});

describe("sequenceNumberFontSize", () => {
  it("uses small pixel-font labels on mobile-sized boards and larger labels on desktop-sized boards", () => {
    expect(sequenceNumberFontSize(28)).toBe(8);
    expect(sequenceNumberFontSize(39)).toBe(8);
    expect(sequenceNumberFontSize(40)).toBe(16);
    expect(sequenceNumberFontSize(56)).toBe(16);
  });
});

describe("sequenceNumberPosition", () => {
  it("snaps canvas text labels to whole pixels", () => {
    expect(sequenceNumberPosition(12.4, 20.6)).toEqual({ x: 12, y: 21 });
  });
});
