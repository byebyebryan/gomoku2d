import { describe, expect, it, vi } from "vitest";

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

  it("seeds a resumed local match and keeps the replay variant", () => {
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
      humanDisplayName: "Bryan Guest",
      resumeState: {
        currentPlayer: 1,
        moves: [
          { col: 7, moveNumber: 1, player: 1, row: 7 },
          { col: 8, moveNumber: 2, player: 2, row: 7 },
          { col: 9, moveNumber: 3, player: 1, row: 7 },
          { col: 10, moveNumber: 4, player: 2, row: 7 },
        ],
        variant: "renju",
      },
    });

    expect(store.getState()).toMatchObject({
      currentPlayer: 1,
      currentVariant: "renju",
      moves: [
        expect.objectContaining({ moveNumber: 1, player: 1, row: 7, col: 7 }),
        expect.objectContaining({ moveNumber: 2, player: 2, row: 7, col: 8 }),
        expect.objectContaining({ moveNumber: 3, player: 1, row: 7, col: 9 }),
        expect.objectContaining({ moveNumber: 4, player: 2, row: 7, col: 10 }),
      ],
      pendingBotMove: false,
      selectedVariant: "renju",
      status: "playing",
    });
    expect(store.getState().players[0]).toMatchObject({ kind: "human", stone: "black", name: "Bryan Guest" });
    expect(store.getState().players[1]).toMatchObject({ kind: "bot", stone: "white", name: "Practice Bot" });
  });

  it("remaps the replay side to move to the human when resuming as white", () => {
    let chooseMoveCalls = 0;
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => {
          chooseMoveCalls += 1;
          return null;
        },
        configure: () => undefined,
        dispose: () => undefined,
      },
      humanDisplayName: "Bryan Guest",
      resumeState: {
        currentPlayer: 2,
        moves: [
          { col: 7, moveNumber: 1, player: 1, row: 7 },
          { col: 8, moveNumber: 2, player: 2, row: 7 },
          { col: 9, moveNumber: 3, player: 1, row: 7 },
        ],
        variant: "freestyle",
      },
    });

    expect(store.getState().currentPlayer).toBe(2);
    expect(store.getState().players[0]).toMatchObject({ kind: "bot", stone: "black", name: "Practice Bot" });
    expect(store.getState().players[1]).toMatchObject({ kind: "human", stone: "white", name: "Bryan Guest" });
    expect(store.getState().pendingBotMove).toBe(false);
    expect(chooseMoveCalls).toBe(0);

    expect(store.getState().placeHumanMove(8, 8)).toBe(true);
    expect(store.getState().pendingBotMove).toBe(true);
    expect(chooseMoveCalls).toBe(1);
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

  it("exposes canonical winning cells from the wasm board", () => {
    const board = WasmBoard.createWithVariant("freestyle");
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
      board.applyMove(row, col);
    }

    expect(board.applyMove(0, 4)).toMatchObject({ result: "black" });
    expect(board.winningCells()).toEqual([
      { row: 0, col: 0 },
      { row: 0, col: 1 },
      { row: 0, col: 2 },
      { row: 0, col: 3 },
      { row: 0, col: 4 },
      { row: 0, col: 5 },
    ]);
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

  it("undoes the pending human move before the bot replies", () => {
    let chooseMoveCalls = 0;
    let cancelPendingCalls = 0;
    const store = createLocalMatchStore({
      botRunner: {
        cancelPending: () => {
          cancelPendingCalls += 1;
        },
        chooseMove: async () => {
          chooseMoveCalls += 1;
          return new Promise(() => undefined);
        },
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    expect(store.getState().placeHumanMove(7, 7)).toBe(true);
    expect(store.getState().pendingBotMove).toBe(true);
    expect(store.getState().moves).toHaveLength(1);
    expect(chooseMoveCalls).toBe(1);

    expect(store.getState().undoLastTurn()).toBe(true);

    expect(cancelPendingCalls).toBe(1);
    expect(store.getState()).toMatchObject({
      currentPlayer: 1,
      moves: [],
      pendingBotMove: false,
      status: "playing",
    });
    expect(store.getState().cells.flat().every((cell) => cell === null)).toBe(true);
  });

  it("does not log an error when undo cancels the pending bot move", async () => {
    let rejectPendingMove!: (error: Error) => void;
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const store = createLocalMatchStore({
      botRunner: {
        cancelPending: () => {
          rejectPendingMove(new Error("cancelled"));
        },
        chooseMove: async () =>
          new Promise<null>((_, reject) => {
            rejectPendingMove = reject;
          }),
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    expect(store.getState().placeHumanMove(7, 7)).toBe(true);
    expect(store.getState().pendingBotMove).toBe(true);

    expect(store.getState().undoLastTurn()).toBe(true);
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(consoleError).not.toHaveBeenCalled();
    consoleError.mockRestore();
  });

  it("undoes the last full turn after the bot replies", async () => {
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => ({ row: 7, col: 8 }),
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    expect(store.getState().placeHumanMove(7, 7)).toBe(true);
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(store.getState().moves).toEqual([
      expect.objectContaining({ moveNumber: 1, player: 1, row: 7, col: 7 }),
      expect.objectContaining({ moveNumber: 2, player: 2, row: 7, col: 8 }),
    ]);
    expect(store.getState().currentPlayer).toBe(1);

    expect(store.getState().undoLastTurn()).toBe(true);

    expect(store.getState()).toMatchObject({
      currentPlayer: 1,
      moves: [],
      pendingBotMove: false,
      status: "playing",
    });
    expect(store.getState().cells.flat().every((cell) => cell === null)).toBe(true);
  });

  it("keeps the opening bot stone when undoing as white", async () => {
    let resolveOpeningMove!: (move: { row: number; col: number }) => void;
    let resolveReplyMove!: (move: { row: number; col: number }) => void;
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: () =>
          new Promise<{ row: number; col: number }>((resolve) => {
            if (!resolveOpeningMove) {
              resolveOpeningMove = resolve;
            } else {
              resolveReplyMove = resolve;
            }
          }),
        configure: () => undefined,
        dispose: () => undefined,
      },
    });

    store.getState().startNextRound();
    resolveOpeningMove({ row: 7, col: 7 });
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(store.getState().moves).toEqual([
      expect.objectContaining({ moveNumber: 1, player: 1, row: 7, col: 7 }),
    ]);
    expect(store.getState().undoLastTurn()).toBe(false);

    expect(store.getState().placeHumanMove(7, 8)).toBe(true);
    expect(store.getState().pendingBotMove).toBe(true);

    expect(store.getState().undoLastTurn()).toBe(true);
    expect(store.getState()).toMatchObject({
      currentPlayer: 2,
      moves: [expect.objectContaining({ moveNumber: 1, player: 1, row: 7, col: 7 })],
      pendingBotMove: false,
      status: "playing",
    });

    expect(store.getState().placeHumanMove(8, 8)).toBe(true);
    resolveReplyMove({ row: 7, col: 9 });
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(store.getState().moves).toEqual([
      expect.objectContaining({ moveNumber: 1, player: 1, row: 7, col: 7 }),
      expect.objectContaining({ moveNumber: 2, player: 2, row: 8, col: 8 }),
      expect.objectContaining({ moveNumber: 3, player: 1, row: 7, col: 9 }),
    ]);

    expect(store.getState().undoLastTurn()).toBe(true);
    expect(store.getState()).toMatchObject({
      currentPlayer: 2,
      moves: [expect.objectContaining({ moveNumber: 1, player: 1, row: 7, col: 7 })],
      pendingBotMove: false,
      status: "playing",
    });

    expect(store.getState().placeHumanMove(7, 8)).toBe(true);
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(store.getState().moves).toEqual([
      expect.objectContaining({ moveNumber: 1, player: 1, row: 7, col: 7 }),
      expect.objectContaining({ moveNumber: 2, player: 2, row: 7, col: 8 }),
    ]);
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
