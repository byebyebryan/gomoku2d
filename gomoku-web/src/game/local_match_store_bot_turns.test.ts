import { describe, expect, it, vi } from "vitest";

import type { LocalMatchState } from "./local_match_store";
import { createLocalMatchStore } from "./local_match_store";

describe("createLocalMatchStore bot turns and undo", () => {
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
});
