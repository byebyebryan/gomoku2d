import { describe, expect, it } from "vitest";

import { WasmBoard } from "../core/wasm_bridge";

import type { LocalMatchState } from "./local_match_store";
import { createLocalMatchStore } from "./local_match_store";

describe("createLocalMatchStore", () => {
  it("creates a fresh human-vs-bot match with black to move", () => {
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
    });
    const state = store.getState();

    expect(state.status).toBe("playing");
    expect(state.currentPlayer).toBe(1);
    expect(state.players[0].kind).toBe("human");
    expect(state.players[1].kind).toBe("bot");
    expect(state.moves).toEqual([]);
    expect(state.forbiddenMoves).toEqual([]);
    expect(state.winningMoves).toEqual([]);
    expect(state.threatMoves).toEqual([]);
  });

  it("derives human-turn warning cues from the wasm board", () => {
    const board = WasmBoard.createWithVariant("freestyle");
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
      board.applyMove(row, col);
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
    expect(state.threatMoves).toEqual([{ row: 0, col: 4 }]);
  });

  it("swaps colors for the next round and lets the black bot open", async () => {
    let resolveMove!: (move: { row: number; col: number }) => void;
    let hasQueuedMove = false;
    const chooseMoveCalls: Array<0 | 1> = [];
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: (slot) => {
          chooseMoveCalls.push(slot);
          return new Promise<{ row: number; col: number }>((resolve) => {
            hasQueuedMove = true;
            resolveMove = resolve;
          });
        },
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    expect("startNextRound" in store.getState()).toBe(true);
    (store.getState() as LocalMatchState & { startNextRound: () => void }).startNextRound();

    expect(store.getState().players[0]).toMatchObject({ kind: "bot", stone: "black" });
    expect(store.getState().players[1]).toMatchObject({ kind: "human", stone: "white" });
    expect(store.getState().pendingBotMove).toBe(true);
    expect(chooseMoveCalls).toEqual([0]);
    expect(hasQueuedMove).toBe(true);
    resolveMove({ row: 7, col: 7 });
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(store.getState().moves).toEqual([
      expect.objectContaining({ moveNumber: 1, player: 1, row: 7, col: 7 }),
    ]);
    expect(store.getState().currentPlayer).toBe(2);
    expect(store.getState().placeHumanMove(7, 8)).toBe(true);
  });
});
