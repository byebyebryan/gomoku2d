import { describe, expect, it } from "vitest";

import { applyWasmMove, createWasmBoard, readWasmWinningCells } from "../core/wasm_bridge";

import { createLocalMatchStore } from "./local_match_store";

describe("createLocalMatchStore tactical hints", () => {
  it("derives human-turn warning cues from the wasm board", () => {
    const board = createWasmBoard("freestyle");
    const moves: Array<[number, number]> = [
      [7, 7],
      [0, 0],
      [7, 8],
      [0, 1],
      [7, 9],
      [0, 2],
      [7, 10],
      [0, 3],
    ];

    for (const [row, col] of moves) {
      applyWasmMove(board, row, col);
    }

    const store = createLocalMatchStore({
      boardFactory: () => board,
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    const state = store.getState();

    expect(state.currentPlayer).toBe(1);
    expect(state.winningMoves).toEqual(
      expect.arrayContaining([
        { row: 7, col: 6 },
        { row: 7, col: 11 },
      ]),
    );
    expect(state.winningEvidenceCells).toEqual([
      { row: 7, col: 7 },
      { row: 7, col: 8 },
      { row: 7, col: 9 },
      { row: 7, col: 10 },
    ]);
    expect(state.threatMoves).toEqual([{ row: 0, col: 4 }]);
    expect(state.immediateThreatEvidenceCells).toEqual([
      { row: 0, col: 0 },
      { row: 0, col: 1 },
      { row: 0, col: 2 },
      { row: 0, col: 3 },
    ]);
    expect(state.imminentThreatMoves).toEqual([]);
  });

  it("derives defensive replies to opponent imminent threats from the wasm board", () => {
    const board = createWasmBoard("freestyle");
    const moves: Array<[number, number]> = [
      [0, 0],
      [7, 7],
      [0, 2],
      [7, 8],
      [0, 4],
      [7, 9],
    ];

    for (const [row, col] of moves) {
      applyWasmMove(board, row, col);
    }

    const store = createLocalMatchStore({
      boardFactory: () => board,
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    const state = store.getState();

    expect(state.currentPlayer).toBe(1);
    expect(state.winningMoves).toEqual([]);
    expect(state.threatMoves).toEqual([]);
    expect(state.imminentThreatMoves).toEqual([
      { row: 7, col: 6 },
      { row: 7, col: 10 },
    ]);
    expect(state.imminentThreatEvidenceCells).toEqual([
      { row: 7, col: 7 },
      { row: 7, col: 8 },
      { row: 7, col: 9 },
    ]);
    expect(state.counterThreatMoves).toEqual([
      { row: 0, col: 1 },
      { row: 0, col: 3 },
    ]);
    expect(state.counterThreatEvidenceCells).toEqual([
      { row: 0, col: 0 },
      { row: 0, col: 2 },
      { row: 0, col: 4 },
    ]);
  });

  it("does not show counter-threat hints without an opponent imminent threat", () => {
    const board = createWasmBoard("freestyle");
    const moves: Array<[number, number]> = [
      [7, 7],
      [0, 0],
      [7, 8],
      [0, 14],
      [7, 9],
      [14, 0],
    ];

    for (const [row, col] of moves) {
      applyWasmMove(board, row, col);
    }

    const store = createLocalMatchStore({
      boardFactory: () => board,
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    const state = store.getState();

    expect(state.currentPlayer).toBe(1);
    expect(state.imminentThreatMoves).toEqual([]);
    expect(state.counterThreatMoves).toEqual([]);
  });

  it("prioritizes immediate replies over imminent replies for opponent combo threats", () => {
    const board = createWasmBoard("freestyle");
    const moves: Array<[number, number]> = [
      [0, 0],
      [7, 7],
      [1, 2],
      [7, 8],
      [2, 4],
      [7, 9],
      [3, 6],
      [7, 10],
      [4, 8],
      [5, 5],
      [5, 10],
      [5, 6],
      [6, 12],
      [5, 7],
    ];

    for (const [row, col] of moves) {
      applyWasmMove(board, row, col);
    }

    const store = createLocalMatchStore({
      boardFactory: () => board,
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    const state = store.getState();

    expect(state.currentPlayer).toBe(1);
    expect(state.threatMoves).toEqual(
      expect.arrayContaining([
        { row: 7, col: 6 },
        { row: 7, col: 11 },
      ]),
    );
    expect(state.imminentThreatMoves).toEqual([]);
  });

  it("does not show closed broken threes as imminent threat hints", () => {
    const board = createWasmBoard("freestyle");
    const moves: Array<[number, number]> = [
      [0, 0],
      [7, 7],
      [7, 11],
      [7, 9],
      [2, 0],
      [7, 10],
    ];

    for (const [row, col] of moves) {
      applyWasmMove(board, row, col);
    }

    const store = createLocalMatchStore({
      boardFactory: () => board,
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    const state = store.getState();

    expect(state.currentPlayer).toBe(1);
    expect(state.imminentThreatMoves).toEqual([]);
  });

  it("derives counter-threat replies from the unified wasm threat snapshot", () => {
    const moves = [
      [7, 7],
      [7, 8],
      [6, 7],
      [6, 8],
      [5, 7],
      [4, 7],
      [5, 8],
      [8, 8],
      [5, 6],
      [5, 9],
      [7, 6],
      [4, 9],
      [4, 6],
    ].map(([row, col], index) => ({
      col,
      moveNumber: index + 1,
      player: (index % 2 === 0 ? 1 : 2) as 1 | 2,
      row,
    }));

    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
      resumeState: {
        currentPlayer: 2,
        moves,
        variant: "renju",
      },
    });

    const state = store.getState();

    expect(state.currentPlayer).toBe(2);
    expect(state.imminentThreatMoves).toEqual(
      expect.arrayContaining([
        { row: 3, col: 6 },
        { row: 6, col: 6 },
        { row: 8, col: 6 },
      ]),
    );
    expect(state.counterThreatMoves).toEqual(
      expect.arrayContaining([
        { row: 9, col: 8 },
        { row: 10, col: 8 },
      ]),
    );
  });

  it("exposes canonical winning cells from the wasm board", () => {
    const board = createWasmBoard("freestyle");
    const moves: Array<[number, number]> = [
      [0, 0],
      [14, 0],
      [0, 1],
      [14, 2],
      [0, 2],
      [14, 4],
      [0, 3],
      [14, 6],
      [0, 5],
      [14, 8],
    ];

    for (const [row, col] of moves) {
      applyWasmMove(board, row, col);
    }

    expect(applyWasmMove(board, 0, 4)).toMatchObject({ result: "black" });
    expect(readWasmWinningCells(board)).toEqual([
      { row: 0, col: 0 },
      { row: 0, col: 1 },
      { row: 0, col: 2 },
      { row: 0, col: 3 },
      { row: 0, col: 4 },
      { row: 0, col: 5 },
    ]);
  });
});
