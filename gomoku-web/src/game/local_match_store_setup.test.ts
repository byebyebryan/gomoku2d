import { describe, expect, it } from "vitest";

import {
  DEFAULT_BOT_CONFIG,
  type BotConfig,
} from "../core/bot_config";

import type { LocalMatchState } from "./local_match_store";
import { createLocalMatchStore } from "./local_match_store";

describe("createLocalMatchStore setup", () => {
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
    expect(state.imminentThreatMoves).toEqual([]);
  });

  it("uses the provided human display name for the local player", () => {
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
      humanDisplayName: "Bryan Local",
    });

    expect(store.getState().players[0]).toMatchObject({
      kind: "human",
      name: "Bryan Local",
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

  it("tracks the active and selected bot config", () => {
    const botConfig: BotConfig = { mode: "preset", preset: "hard", version: 1 };
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
      botConfig,
    });

    expect(store.getState().currentBotConfig).toEqual(botConfig);
    expect(store.getState().selectedBotConfig).toEqual(botConfig);
    expect(store.getState().players[1]).toMatchObject({
      kind: "bot",
      name: "Hard Bot",
      stone: "white",
    });
  });

  it("configures bot runners from the selected bot config", () => {
    const configureCalls: unknown[] = [];
    createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: (specs) => {
          configureCalls.push(specs);
        },
        dispose: () => undefined,
      },
      botConfig: { mode: "preset", preset: "hard", version: 1 },
    });

    expect(configureCalls[0]).toEqual([
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

  it("applies a bot change immediately before the first move", () => {
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

    store.getState().selectBotConfig(hard);

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

  it("defers a bot change until the next game once moves exist", () => {
    const store = createLocalMatchStore({
      botRunner: {
        chooseMove: async () => null,
        configure: () => undefined,
        dispose: () => undefined,
      },
    });
    const hard: BotConfig = { mode: "preset", preset: "hard", version: 1 };

    expect(store.getState().placeHumanMove(7, 7)).toBe(true);
    store.getState().selectBotConfig(hard);

    expect(store.getState()).toMatchObject({
      currentBotConfig: DEFAULT_BOT_CONFIG,
      selectedBotConfig: hard,
    });
    expect(store.getState().moves).toHaveLength(1);
  });
});
