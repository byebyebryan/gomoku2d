import { expect, it } from "vitest";

import { applyWasmMove, createWasmBoard, readWasmThreatSnapshot, wasmBoardFromFenWithVariant } from "./wasm_bridge";

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

it("keeps directional Renju four legality consistent in board and hint exports", () => {
  const board = createWasmBoard("renju");
  try {
    for (const notation of [
      "E8", "A1", "F8", "A3", "G8", "A5", "J8", "A7",
      "I4", "C1", "I5", "C3", "I6", "C5", "I7", "C7",
    ]) {
      const col = notation.charCodeAt(0) - 65;
      const row = Number(notation.slice(1)) - 1;
      expect(applyWasmMove(board, row, col).error).toBeNull();
    }
    expect(board.isLegal(7, 7)).toBe(true);
    expect(readWasmThreatSnapshot(board).forbiddenMoves).not.toContainEqual({ row: 7, col: 7 });
    expect(applyWasmMove(board, 7, 7)).toEqual({ error: null, result: "ongoing" });
  } finally {
    board.free();
  }
});
