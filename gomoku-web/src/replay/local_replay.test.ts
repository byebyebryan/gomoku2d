import { describe, expect, it } from "vitest";

import type { GuestSavedMatch } from "../profile/guest_profile_store";

import {
  buildLocalReplayFrame,
  canResumeReplay,
  defaultReplayMoveIndex,
  replayStartMoveIndex,
  shouldShowReplaySequenceNumbers,
} from "./local_replay";

const SAMPLE_MATCH: GuestSavedMatch = {
  guestStone: "black",
  id: "match-1",
  mode: "bot",
  moves: [
    { col: 7, moveNumber: 1, player: 1, row: 7 },
    { col: 0, moveNumber: 2, player: 2, row: 0 },
    { col: 8, moveNumber: 3, player: 1, row: 7 },
  ],
  players: [
    { kind: "human", name: "Guest", stone: "black" },
    { kind: "bot", name: "Classic Bot", stone: "white" },
  ],
  savedAt: "2026-04-21T12:00:00.000Z",
  status: "black_won",
  variant: "freestyle",
  winningCells: [
    { row: 7, col: 5 },
    { row: 7, col: 6 },
    { row: 7, col: 7 },
    { row: 7, col: 8 },
    { row: 7, col: 9 },
  ],
};

describe("buildLocalReplayFrame", () => {
  it("derives an in-progress board position for intermediate move indexes", () => {
    const frame = buildLocalReplayFrame(SAMPLE_MATCH, 2);

    expect(frame.moveIndex).toBe(2);
    expect(frame.moves).toEqual(SAMPLE_MATCH.moves.slice(0, 2));
    expect(frame.currentPlayer).toBe(1);
    expect(frame.lastMove).toEqual(SAMPLE_MATCH.moves[1]);
    expect(frame.status).toBe("playing");
    expect(frame.winningCells).toEqual([]);
    expect(frame.cells[7][7]).toBe(0);
    expect(frame.cells[0][0]).toBe(1);
    expect(frame.cells[7][8]).toBeNull();
  });

  it("returns the finished match state at the final move index", () => {
    const frame = buildLocalReplayFrame(SAMPLE_MATCH, SAMPLE_MATCH.moves.length);

    expect(frame.moveIndex).toBe(3);
    expect(frame.moves).toEqual(SAMPLE_MATCH.moves);
    expect(frame.status).toBe("black_won");
    expect(frame.winningCells).toEqual(SAMPLE_MATCH.winningCells);
    expect(frame.lastMove).toEqual(SAMPLE_MATCH.moves[2]);
    expect(frame.cells[7][8]).toBe(0);
  });

  it("shows sequence numbers only when the replay is at a terminal frame", () => {
    expect(shouldShowReplaySequenceNumbers(buildLocalReplayFrame(SAMPLE_MATCH, 1))).toBe(false);
    expect(shouldShowReplaySequenceNumbers(buildLocalReplayFrame(SAMPLE_MATCH, SAMPLE_MATCH.moves.length))).toBe(
      true,
    );
  });

  it("starts replay a few moves in but keeps Start on the first move", () => {
    expect(defaultReplayMoveIndex(0)).toBe(0);
    expect(defaultReplayMoveIndex(2)).toBe(2);
    expect(defaultReplayMoveIndex(8)).toBe(4);
    expect(replayStartMoveIndex(0)).toBe(0);
    expect(replayStartMoveIndex(8)).toBe(1);
  });

  it("only enables replay resume once the frame is deep enough and still playing", () => {
    expect(canResumeReplay(buildLocalReplayFrame(SAMPLE_MATCH, 3))).toBe(false);
    expect(canResumeReplay(buildLocalReplayFrame(SAMPLE_MATCH, SAMPLE_MATCH.moves.length))).toBe(false);

    const longerReplay = {
      ...SAMPLE_MATCH,
      moves: [
        ...SAMPLE_MATCH.moves,
        { col: 1, moveNumber: 4, player: 2 as const, row: 1 },
        { col: 9, moveNumber: 5, player: 1 as const, row: 7 },
      ],
      status: "playing" as const,
      winningCells: [],
    };

    expect(canResumeReplay(buildLocalReplayFrame(longerReplay, 4))).toBe(true);
  });
});
