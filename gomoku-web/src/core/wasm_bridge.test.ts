import { expect, it } from "vitest";

import { applyWasmMove, readWasmThreatSnapshot, wasmBoardFromFenWithVariant } from "./wasm_bridge";

it("keeps FEN-imported draw and undo semantics across the actual Wasm bridge", () => {
  const board = wasmBoardFromFenWithVariant("3/3/B/BWBBWWWB.", "freestyle");
  try {
    expect(applyWasmMove(board, 2, 2)).toEqual({ error: null, result: "draw" });
    expect(board.result()).toBe("draw");
    expect(readWasmThreatSnapshot(board).winningMoves).toEqual([]);
    board.undoLastMove();
    expect(board.result()).toBe("ongoing");
    expect(board.toFen()).toBe("3/3/B/BWBBWWWB.");
  } finally {
    board.free();
  }
});

it("rejects invalid FEN dimensions at the actual Wasm boundary", () => {
  expect(() => wasmBoardFromFenWithVariant("3/0/B/.........", "freestyle")).toThrow();
});
