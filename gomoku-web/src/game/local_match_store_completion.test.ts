import { describe, expect, it, vi } from "vitest";

import { BOARD_SIZE } from "../board/constants";
import { DEFAULT_BOT_CONFIG } from "../core/bot_config";
import { applyWasmMove, createWasmBoard } from "../core/wasm_bridge";
import { noOpBotRunner } from "../test/local_match_fixtures";

import { canUndoLocalMatch, createLocalMatchStore, type LocalMatchState } from "./local_match_store";

describe("createLocalMatchStore completion", () => {
  it("reports finished matches through the completion callback with the active variant", () => {
    const board = createWasmBoard("renju");
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
      botRunner: noOpBotRunner,
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

    const readCell = vi.spyOn(board, "cell");
    expect(store.getState().placeHumanMove(7, 11)).toBe(true);
    expect(readCell).toHaveBeenCalledTimes(BOARD_SIZE * BOARD_SIZE);

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

    const won = store.getState();
    expect(finishedMatches[0].winningCells).toEqual(won.winningCells);
    expect(finishedMatches[0].winningCells).not.toBe(won.winningCells);
    expect(finishedMatches[0].winningCells[0]).not.toBe(won.winningCells[0]);
    expect(canUndoLocalMatch(won)).toBe(false);
    expect(won.undoLastTurn()).toBe(false);
    expect(store.getState()).toBe(won);
    expect(won.placeHumanMove(7, 11)).toBe(false);
    expect(finishedMatches).toHaveLength(1);
    store.getState().dispose();
  });

  it("keeps a full-board draw final", () => {
    const black: Array<[number, number]> = [];
    const white: Array<[number, number]> = [];
    // Alternating columns with a two-row phase shift avoid a five-stone line.
    for (let row = 0; row < 15; row += 1) {
      const blackParity = row % 4 === 1 || row % 4 === 2 ? 0 : 1;
      for (let col = 0; col < 15; col += 1) {
        (col % 2 === blackParity ? black : white).push([row, col]);
      }
    }
    const opening = white.flatMap((cell, index) => [black[index]!, cell]);
    const [lastRow, lastCol] = black[black.length - 1]!;
    const results: LocalMatchState["status"][] = [];
    const store = createLocalMatchStore({
      botRunner: noOpBotRunner,
      onMatchFinished: ({ status }) => { results.push(status); },
      resumeState: {
        currentPlayer: 1,
        moves: opening.map(([row, col], index) => ({
          row,
          col,
          player: index % 2 === 0 ? 1 : 2,
          moveNumber: index + 1,
        })),
        variant: "freestyle",
      },
    });

    expect(store.getState().moves).toHaveLength(224);
    expect(store.getState().placeHumanMove(lastRow, lastCol)).toBe(true);
    const drawn = store.getState();
    expect(drawn.moves).toHaveLength(225);
    expect(drawn.status).toBe("draw");
    expect(canUndoLocalMatch(drawn)).toBe(false);
    expect(drawn.undoLastTurn()).toBe(false);
    expect(store.getState()).toBe(drawn);
    expect(results).toEqual(["draw"]);
    store.getState().dispose();
  });
});
