import { describe, expect, it } from "vitest";

import { DEFAULT_BOT_CONFIG } from "../core/bot_config";
import { WasmBoard } from "../core/wasm_bridge";

import type { LocalMatchState } from "./local_match_store";
import { createLocalMatchStore } from "./local_match_store";

describe("createLocalMatchStore completion", () => {
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
      botConfig: unknown;
      status: LocalMatchState["status"];
      undoFloor: number;
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
          botConfig: match.botConfig,
          status: match.status,
          undoFloor: match.undoFloor,
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
        botConfig: DEFAULT_BOT_CONFIG,
        status: "black_won",
        undoFloor: 0,
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
