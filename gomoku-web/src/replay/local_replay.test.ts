import { describe, expect, it } from "vitest";

import { createLocalSavedMatch, movesFromMoveCells } from "../match/saved_match";
import type { LocalProfileSavedMatch } from "../profile/local_profile_store";

import {
  buildLocalReplayFrame,
  canResumeReplay,
  defaultReplayMoveIndex,
  replayResumeUndoFloor,
  replayStartMoveIndex,
  shouldShowReplaySequenceNumbers,
} from "./local_replay";
import { winningCellsFromCore } from "./local_replay_core";

const SAMPLE_MOVES = [
  { col: 5, moveNumber: 1, player: 1 as const, row: 7 },
  { col: 0, moveNumber: 2, player: 2 as const, row: 0 },
  { col: 6, moveNumber: 3, player: 1 as const, row: 7 },
  { col: 1, moveNumber: 4, player: 2 as const, row: 0 },
  { col: 7, moveNumber: 5, player: 1 as const, row: 7 },
  { col: 2, moveNumber: 6, player: 2 as const, row: 0 },
  { col: 8, moveNumber: 7, player: 1 as const, row: 7 },
  { col: 3, moveNumber: 8, player: 2 as const, row: 0 },
  { col: 9, moveNumber: 9, player: 1 as const, row: 7 },
];

function savedMatchWithMoves(
  moves: typeof SAMPLE_MOVES,
  patch: Partial<Pick<LocalProfileSavedMatch, "status" | "undo_floor">> = {},
): LocalProfileSavedMatch {
  return {
    ...createLocalSavedMatch({
      id: "match-1",
      localProfileId: "local-1",
      moves,
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Practice Bot", stone: "white" },
      ],
      savedAt: "2026-04-21T12:00:00.000Z",
      status: "black_won",
      ruleset: "freestyle",
    }),
    ...patch,
  };
}

const SAMPLE_MATCH = savedMatchWithMoves(SAMPLE_MOVES);

describe("buildLocalReplayFrame", () => {
  it("derives an in-progress board position for intermediate move indexes", () => {
    const frame = buildLocalReplayFrame(SAMPLE_MATCH, 2);

    expect(frame.moveIndex).toBe(2);
    expect(frame.moves).toEqual(movesFromMoveCells(SAMPLE_MATCH.move_cells).slice(0, 2));
    expect(frame.currentPlayer).toBe(1);
    expect(frame.lastMove).toEqual(SAMPLE_MOVES[1]);
    expect(frame.status).toBe("playing");
    expect(frame.winningCells).toEqual([]);
    expect(frame.cells[7][5]).toBe(0);
    expect(frame.cells[0][0]).toBe(1);
    expect(frame.cells[7][6]).toBeNull();
  });

  it("returns the finished match state at the final move index", () => {
    const frame = buildLocalReplayFrame(SAMPLE_MATCH, SAMPLE_MATCH.move_count, winningCellsFromCore);

    expect(frame.moveIndex).toBe(9);
    expect(frame.moves).toEqual(movesFromMoveCells(SAMPLE_MATCH.move_cells));
    expect(frame.status).toBe("black_won");
    expect(frame.winningCells).toEqual([
      { row: 7, col: 5 },
      { row: 7, col: 6 },
      { row: 7, col: 7 },
      { row: 7, col: 8 },
      { row: 7, col: 9 },
    ]);
    expect(frame.lastMove).toEqual(SAMPLE_MOVES[8]);
    expect(frame.cells[7][9]).toBe(0);
  });

  it("shows sequence numbers only when the replay is at a terminal frame", () => {
    expect(shouldShowReplaySequenceNumbers(buildLocalReplayFrame(SAMPLE_MATCH, 1))).toBe(false);
    expect(shouldShowReplaySequenceNumbers(buildLocalReplayFrame(SAMPLE_MATCH, SAMPLE_MATCH.move_count))).toBe(
      true,
    );
  });

  it("starts replay a few moves in but keeps Start on the first move", () => {
    expect(defaultReplayMoveIndex(0)).toBe(0);
    expect(defaultReplayMoveIndex(2)).toBe(2);
    expect(defaultReplayMoveIndex(8)).toBe(4);
    expect(defaultReplayMoveIndex(8, 5)).toBe(5);
    expect(replayStartMoveIndex(0)).toBe(0);
    expect(replayStartMoveIndex(8)).toBe(1);
  });

  it("only enables replay resume once the frame is deep enough and still playing", () => {
    expect(canResumeReplay(buildLocalReplayFrame(SAMPLE_MATCH, 3))).toBe(false);
    expect(canResumeReplay(buildLocalReplayFrame(SAMPLE_MATCH, SAMPLE_MATCH.move_count))).toBe(false);

    expect(canResumeReplay(buildLocalReplayFrame(SAMPLE_MATCH, 4))).toBe(true);
    expect(canResumeReplay(buildLocalReplayFrame(SAMPLE_MATCH, 4), 5)).toBe(false);
    expect(canResumeReplay(buildLocalReplayFrame(SAMPLE_MATCH, 5), 5)).toBe(true);
  });

  it("only raises the replay undo floor when branching again", () => {
    const branchedReplay = savedMatchWithMoves(SAMPLE_MOVES, { undo_floor: 5 });

    expect(replayResumeUndoFloor(branchedReplay, buildLocalReplayFrame(branchedReplay, 5))).toBe(5);
    expect(replayResumeUndoFloor(branchedReplay, buildLocalReplayFrame(branchedReplay, 6))).toBe(6);
  });
});
