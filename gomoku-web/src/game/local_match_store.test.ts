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

  it("uses the provided human display name for the guest player", () => {
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
      humanDisplayName: "Bryan Guest",
    });

    expect(store.getState().players[0]).toMatchObject({
      kind: "human",
      name: "Bryan Guest",
      stone: "black",
    });
  });

  it("tracks the active and selected rules variant", () => {
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
      variant: "renju",
    });

    const state = store.getState() as LocalMatchState & {
      currentVariant: "freestyle" | "renju";
      selectedVariant: "freestyle" | "renju";
    };

    expect(state.currentVariant).toBe("renju");
    expect(state.selectedVariant).toBe("renju");
  });

  it("applies a rules change immediately before the first move", () => {
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    const state = store.getState() as LocalMatchState & {
      currentVariant: "freestyle" | "renju";
      selectedVariant: "freestyle" | "renju";
      selectVariant: (variant: "freestyle" | "renju") => void;
    };

    state.selectVariant("renju");

    expect(store.getState()).toMatchObject({
      currentPlayer: 1,
      currentVariant: "renju",
      moves: [],
      selectedVariant: "renju",
      status: "playing",
    });
  });

  it("defers a rules change until the next game once moves exist", () => {
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    expect(store.getState().placeHumanMove(7, 7)).toBe(true);

    const state = store.getState() as LocalMatchState & {
      currentVariant: "freestyle" | "renju";
      selectedVariant: "freestyle" | "renju";
      selectVariant: (variant: "freestyle" | "renju") => void;
    };

    state.selectVariant("renju");

    expect(store.getState()).toMatchObject({
      currentVariant: "freestyle",
      selectedVariant: "renju",
    });
    expect(store.getState().moves).toHaveLength(1);
  });

  it("starts a new match with the selected rules variant", () => {
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    expect(store.getState().placeHumanMove(7, 7)).toBe(true);

    const state = store.getState() as LocalMatchState & {
      currentVariant: "freestyle" | "renju";
      selectedVariant: "freestyle" | "renju";
      selectVariant: (variant: "freestyle" | "renju") => void;
    };

    state.selectVariant("renju");
    store.getState().startNewMatch();

    expect(store.getState()).toMatchObject({
      currentPlayer: 1,
      currentVariant: "renju",
      moves: [],
      selectedVariant: "renju",
      status: "playing",
    });
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

    (store.getState() as LocalMatchState & {
      selectVariant: (variant: "freestyle" | "renju") => void;
    }).selectVariant("renju");

    expect("startNextRound" in store.getState()).toBe(true);
    (store.getState() as LocalMatchState & { startNextRound: () => void }).startNextRound();

    expect((store.getState() as LocalMatchState & {
      currentVariant: "freestyle" | "renju";
      selectedVariant: "freestyle" | "renju";
    }).currentVariant).toBe("renju");
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

  it("reports finished matches through the completion callback with the active variant", () => {
    const board = WasmBoard.createWithVariant("renju");
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

    const finishedMatches: Array<{
      players: LocalMatchState["players"];
      status: LocalMatchState["status"];
      variant: "freestyle" | "renju";
      winningCells: LocalMatchState["winningCells"];
    }> = [];

    const store = createLocalMatchStore({
      boardFactory: () => board,
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
      onMatchFinished: (match) => {
        finishedMatches.push({
          players: match.players,
          status: match.status,
          variant: match.variant,
          winningCells: match.winningCells,
        });
      },
      variant: "renju",
    });

    expect(store.getState().placeHumanMove(7, 11)).toBe(true);

    expect(finishedMatches).toEqual([
      expect.objectContaining({
        players: [
          expect.objectContaining({ kind: "human", stone: "black" }),
          expect.objectContaining({ kind: "bot", stone: "white" }),
        ],
        status: "black_won",
        variant: "renju",
      }),
    ]);
    expect(finishedMatches[0].winningCells).toEqual(
      expect.arrayContaining([
        { row: 7, col: 7 },
        { row: 7, col: 8 },
        { row: 7, col: 9 },
        { row: 7, col: 10 },
        { row: 7, col: 11 },
      ]),
    );
  });
});
