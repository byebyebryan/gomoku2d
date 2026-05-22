import { describe, expect, it } from "vitest";

import type { BotConfig } from "../core/bot_config";

import type { LocalMatchState } from "./local_match_store";
import { createLocalMatchStore } from "./local_match_store";

describe("createLocalMatchStore clocks and resume", () => {
  it("tracks settled player time and the active turn start", async () => {
    let now = 1_000;
    let resolveMove!: (move: { row: number; col: number }) => void;
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: () => new Promise((resolve) => {
          resolveMove = resolve;
        }),
        configure: () => undefined,
        dispose: () => undefined,
      },
      nowMs: () => now,
    });

    now = 2_500;
    expect(store.getState().placeHumanMove(7, 7)).toBe(true);

    expect(store.getState()).toMatchObject({
      playerClockMs: [1_500, 0],
      turnStartedAtMs: 2_500,
    });

    now = 4_800;
    resolveMove({ row: 7, col: 8 });
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(store.getState()).toMatchObject({
      currentPlayer: 1,
      playerClockMs: [1_500, 2_300],
      turnStartedAtMs: 4_800,
    });
  });

  it("resets player clocks for new games and removes undone move time", async () => {
    let now = 0;
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => {
          now = 2_300;
          return { row: 7, col: 8 };
        },
        configure: () => undefined,
        dispose: () => undefined,
      },
      nowMs: () => now,
    });

    now = 1_200;
    expect(store.getState().placeHumanMove(7, 7)).toBe(true);
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(store.getState().playerClockMs).toEqual([1_200, 1_100]);

    now = 3_000;
    expect(store.getState().undoLastTurn()).toBe(true);

    expect(store.getState()).toMatchObject({
      moves: [],
      playerClockMs: [0, 0],
      turnStartedAtMs: 3_000,
    });

    now = 4_000;
    store.getState().startNewMatch();

    expect(store.getState()).toMatchObject({
      playerClockMs: [0, 0],
      turnStartedAtMs: 4_000,
    });
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

  it("starts a new match with the selected bot config", () => {
    const configureCalls: unknown[] = [];
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: (specs) => {
          configureCalls.push(specs);
        },
        dispose: () => undefined,
      },
    });
    const hard: BotConfig = { mode: "preset", preset: "hard", version: 1 };

    expect(store.getState().placeHumanMove(7, 7)).toBe(true);
    store.getState().selectBotConfig(hard);
    store.getState().startNewMatch();

    expect(store.getState()).toMatchObject({
      currentBotConfig: hard,
      moves: [],
      selectedBotConfig: hard,
      status: "playing",
    });
    expect(store.getState().players[1]).toMatchObject({
      kind: "bot",
      name: "Hard Bot",
      stone: "white",
    });
    expect(configureCalls[configureCalls.length - 1]).toEqual([
      { kind: "human" },
      {
        childLimit: 8,
        corridorProof: {
          candidateLimit: 16,
          depth: 8,
          width: 4,
        },
        depth: 7,
        kind: "search",
        maxTtEntries: 500_000,
        patternEval: true,
      },
    ]);
  });

  it("seeds a resumed local match and keeps the replay variant", () => {
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
      humanDisplayName: "Bryan Local",
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
    expect(store.getState().players[0]).toMatchObject({ kind: "human", stone: "black", name: "Bryan Local" });
    expect(store.getState().players[1]).toMatchObject({ kind: "bot", stone: "white", name: "Normal Bot" });
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
      humanDisplayName: "Bryan Local",
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
    expect(store.getState().players[0]).toMatchObject({ kind: "bot", stone: "black", name: "Normal Bot" });
    expect(store.getState().players[1]).toMatchObject({ kind: "human", stone: "white", name: "Bryan Local" });
    expect(store.getState().pendingBotMove).toBe(false);
    expect(chooseMoveCalls).toBe(0);

    expect(store.getState().placeHumanMove(8, 8)).toBe(true);
    expect(store.getState().pendingBotMove).toBe(true);
    expect(chooseMoveCalls).toBe(1);
  });

  it("does not undo past the replay resume floor", async () => {
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => ({ row: 0, col: 2 }),
        configure: () => undefined,
        dispose: () => undefined,
      },
      resumeState: {
        currentPlayer: 1,
        moves: [
          { col: 7, moveNumber: 1, player: 1, row: 7 },
          { col: 0, moveNumber: 2, player: 2, row: 0 },
          { col: 8, moveNumber: 3, player: 1, row: 7 },
          { col: 1, moveNumber: 4, player: 2, row: 0 },
        ],
        variant: "freestyle",
      },
    });

    expect(store.getState()).toMatchObject({
      moves: [
        expect.objectContaining({ moveNumber: 1, player: 1, row: 7, col: 7 }),
        expect.objectContaining({ moveNumber: 2, player: 2, row: 0, col: 0 }),
        expect.objectContaining({ moveNumber: 3, player: 1, row: 7, col: 8 }),
        expect.objectContaining({ moveNumber: 4, player: 2, row: 0, col: 1 }),
      ],
      undoFloor: 4,
    });
    expect(store.getState().undoLastTurn()).toBe(false);

    expect(store.getState().placeHumanMove(7, 9)).toBe(true);
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(store.getState().moves).toEqual([
      expect.objectContaining({ moveNumber: 1, player: 1, row: 7, col: 7 }),
      expect.objectContaining({ moveNumber: 2, player: 2, row: 0, col: 0 }),
      expect.objectContaining({ moveNumber: 3, player: 1, row: 7, col: 8 }),
      expect.objectContaining({ moveNumber: 4, player: 2, row: 0, col: 1 }),
      expect.objectContaining({ moveNumber: 5, player: 1, row: 7, col: 9 }),
      expect.objectContaining({ moveNumber: 6, player: 2, row: 0, col: 2 }),
    ]);

    expect(store.getState().undoLastTurn()).toBe(true);
    expect(store.getState()).toMatchObject({
      currentPlayer: 1,
      moves: [
        expect.objectContaining({ moveNumber: 1, player: 1, row: 7, col: 7 }),
        expect.objectContaining({ moveNumber: 2, player: 2, row: 0, col: 0 }),
        expect.objectContaining({ moveNumber: 3, player: 1, row: 7, col: 8 }),
        expect.objectContaining({ moveNumber: 4, player: 2, row: 0, col: 1 }),
      ],
      pendingBotMove: false,
      undoFloor: 4,
    });
    expect(store.getState().undoLastTurn()).toBe(false);
  });
});
