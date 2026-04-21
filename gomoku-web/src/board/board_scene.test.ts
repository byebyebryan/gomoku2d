import { describe, expect, it } from "vitest";

import { shouldAnimatePlacedStone, shouldStopStoneIdleCycle } from "./board_scene_logic";

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
