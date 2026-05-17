import { describe, expect, it } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";
import type { MatchMove } from "../game/types";
import type { ReplayFrameAnnotations, ReplayAnalysisStepResult } from "./replay_analysis_protocol";

import {
  analysisOverlaysForFrame,
  mergeReplayAnalysisAnnotations,
  nextReplayMove,
  replayAnalysisStatusSummary,
  replayTimelineAnalysis,
} from "./replay_analysis_overlays";

const MOVES: MatchMove[] = [
  { col: 5, moveNumber: 1, player: 1, row: 7 },
  { col: 0, moveNumber: 2, player: 2, row: 0 },
  { col: 6, moveNumber: 3, player: 1, row: 7 },
  { col: 1, moveNumber: 4, player: 2, row: 0 },
  { col: 7, moveNumber: 5, player: 1, row: 7 },
];

const TEN_MOVES: MatchMove[] = [
  ...MOVES,
  { col: 2, moveNumber: 6, player: 2, row: 0 },
  { col: 8, moveNumber: 7, player: 1, row: 7 },
  { col: 3, moveNumber: 8, player: 2, row: 0 },
  { col: 9, moveNumber: 9, player: 1, row: 7 },
  { col: 4, moveNumber: 10, player: 2, row: 0 },
];

function match(
  status: "black_won" | "white_won" | "draw" = "black_won",
  moves: MatchMove[] = MOVES,
) {
  return createLocalSavedMatch({
    id: "match-1",
    localProfileId: "local-1",
    moves,
    players: [
      { kind: "human", name: "Black", stone: "black" },
      { kind: "bot", name: "White", stone: "white" },
    ],
    ruleset: "renju",
    savedAt: "2026-05-16T12:00:00.000Z",
    status,
  });
}

function tenMoveMatch(status: "black_won" | "white_won" | "draw" = "black_won") {
  return match(status, TEN_MOVES);
}

function annotation(ply: number, sideToMove: "Black" | "White"): ReplayFrameAnnotations {
  return {
    highlights: [
      {
        mv: { col: 6, row: 7 },
        notation: "G8",
        role: "immediate_threat",
        side: "Black",
      },
    ],
    markers: [
      {
        mv: { col: 8, row: 7 },
        notation: "I8",
        role: "forced_loss",
        side: sideToMove,
      },
    ],
    ply,
    side_to_move: sideToMove,
  };
}

function step(...annotations: ReplayFrameAnnotations[]): ReplayAnalysisStepResult {
  return {
    analysis: null,
    annotations,
    counters: { branch_roots: 0, prefixes_analyzed: annotations.length, proof_nodes: 0 },
    current_ply: null,
    done: false,
    error: null,
    schema_version: 1,
    status: "running",
  };
}

describe("mergeReplayAnalysisAnnotations", () => {
  it("stores the latest annotation for each ply without duplicating stale frames", () => {
    const first = mergeReplayAnalysisAnnotations({}, step(annotation(4, "White")));
    const replacement = annotation(4, "White");
    replacement.markers = [{ ...replacement.markers[0], role: "confirmed_escape" }];

    const merged = mergeReplayAnalysisAnnotations(first, step(replacement, annotation(5, "Black")));

    expect(Object.keys(merged)).toEqual(["4", "5"]);
    expect(merged[4].markers).toEqual([expect.objectContaining({ role: "confirmed_escape" })]);
  });
});

describe("replayTimelineAnalysis", () => {
  it("marks the searched corridor range and escape point", () => {
    const escapeFrame = annotation(4, "White");
    escapeFrame.markers = [{ ...escapeFrame.markers[0], role: "possible_escape" }];
    const annotations = mergeReplayAnalysisAnnotations(
      {},
      step(annotation(8, "White"), escapeFrame, annotation(6, "White")),
    );

    expect(replayTimelineAnalysis(annotations, 10)).toEqual({
      corridorEndPercent: "100%",
      corridorEndPly: 10,
      corridorStartPercent: "40%",
      corridorStartPly: 4,
      escapePercent: "40%",
      escapePly: 4,
    });
  });

  it("omits timeline analysis when no frame annotations are available", () => {
    expect(replayTimelineAnalysis({}, 10)).toEqual({
      corridorEndPercent: null,
      corridorEndPly: null,
      corridorStartPercent: null,
      corridorStartPly: null,
      escapePercent: null,
      escapePly: null,
    });
  });
});

describe("replayAnalysisStatusSummary", () => {
  it("summarizes running analysis with search counters", () => {
    const running = {
      ...step(annotation(8, "White")),
      counters: { branch_roots: 3, prefixes_analyzed: 2, proof_nodes: 144 },
      current_ply: 8,
      status: "running" as const,
    };

    expect(replayAnalysisStatusSummary(running, {}, match(), {
      currentPlayer: 1,
      moveIndex: 10,
      status: "black_won",
    })).toEqual({
      detail: "Move 8 · 144 nodes",
      label: "Analyzing replay",
    });
  });

  it("summarizes the terminal frame with the winner and corridor length", () => {
    const escapeFrame = annotation(4, "White");
    escapeFrame.markers = [{ ...escapeFrame.markers[0], role: "confirmed_escape" }];
    const annotations = mergeReplayAnalysisAnnotations({}, step(escapeFrame, annotation(8, "White")));
    const resolved = {
      ...step(escapeFrame),
      counters: { branch_roots: 1, prefixes_analyzed: 6, proof_nodes: 2048 },
      done: true,
      status: "resolved" as const,
    };

    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 10,
      status: "black_won",
    })).toEqual({
      detail: "6-ply forced corridor",
      label: "Black won",
    });
  });

  it("summarizes the current frame inside and outside the forced corridor", () => {
    const escapeFrame = annotation(4, "White");
    escapeFrame.markers = [{ ...escapeFrame.markers[0], role: "confirmed_escape" }];
    const annotations = mergeReplayAnalysisAnnotations({}, step(escapeFrame, annotation(6, "White"), annotation(8, "White")));
    const resolved = {
      ...step(escapeFrame),
      counters: { branch_roots: 1, prefixes_analyzed: 6, proof_nodes: 2048 },
      done: true,
      status: "resolved" as const,
    };

    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 4,
      status: "playing",
    })).toEqual({
      detail: "Last chance before move 5",
      label: "White's last escape",
    });
    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 1,
      moveIndex: 7,
      status: "playing",
    })).toEqual({
      detail: "Corridor: moves 5-10",
      label: "Black can force a win",
    });
    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 8,
      status: "playing",
    })).toEqual({
      detail: "No viable escape found",
      label: "White is locked in",
    });
    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 1,
      moveIndex: 2,
      status: "playing",
    })).toEqual({
      detail: "Outside the forced corridor",
      label: "Black to move",
    });
  });
});

describe("analysisOverlaysForFrame", () => {
  it("renders only loser-side frame annotations", () => {
    const annotations = mergeReplayAnalysisAnnotations(
      {},
      step(annotation(4, "White"), annotation(5, "Black")),
    );

    expect(analysisOverlaysForFrame(annotations, match("black_won"), 4)).toEqual([
      expect.objectContaining({ highlight: "immediateThreat", row: 7, col: 6 }),
      expect.objectContaining({ marker: "forcedLoss", row: 7, col: 8 }),
    ]);
    expect(analysisOverlaysForFrame(annotations, match("black_won"), 5)).toEqual([]);
  });

  it("uses the opposite loser-side filter for white wins and suppresses draws", () => {
    const annotations = mergeReplayAnalysisAnnotations(
      {},
      step(annotation(4, "White"), annotation(5, "Black")),
    );

    expect(analysisOverlaysForFrame(annotations, match("white_won"), 5)).toEqual([
      expect.objectContaining({ highlight: "immediateThreat", row: 7, col: 6 }),
      expect.objectContaining({ marker: "forcedLoss", row: 7, col: 8 }),
    ]);
    expect(analysisOverlaysForFrame(annotations, match("draw"), 4)).toEqual([]);
  });

  it("renders corridor-entry annotations in the replay board overlay", () => {
    const frame = annotation(4, "White");
    frame.highlights = [
      {
        mv: { col: 7, row: 7 },
        notation: "H8",
        role: "corridor_entry",
        side: "Black",
      },
    ];

    const annotations = mergeReplayAnalysisAnnotations({}, step(frame));

    expect(analysisOverlaysForFrame(annotations, match("black_won"), 4)).toEqual([
      expect.objectContaining({ highlight: "corridorEntry", row: 7, col: 7, side: "black" }),
      expect.objectContaining({ marker: "forcedLoss", row: 7, col: 8 }),
    ]);
  });

  it("simplifies proof markers for the replay UI", () => {
    const frame = annotation(4, "White");
    frame.highlights = [];
    frame.markers = [
      {
        mv: { col: 8, row: 7 },
        notation: "I8",
        role: "possible_escape",
        side: "White",
      },
      {
        mv: { col: 9, row: 7 },
        notation: "J8",
        role: "forbidden",
        side: "Black",
      },
      {
        mv: { col: 10, row: 7 },
        notation: "K8",
        role: "unknown",
        side: "White",
      },
    ];

    const annotations = mergeReplayAnalysisAnnotations({}, step(frame));

    expect(analysisOverlaysForFrame(annotations, match("black_won"), 4)).toEqual([
      expect.objectContaining({ marker: "confirmedEscape", row: 7, col: 8 }),
      expect.objectContaining({ marker: "forbidden", row: 7, col: 9 }),
    ]);
  });
});

describe("nextReplayMove", () => {
  it("returns the next actual replay move for the current frame", () => {
    const replay = match();

    expect(nextReplayMove(replay, 0)).toEqual({ row: 7, col: 5 });
    expect(nextReplayMove(replay, 2)).toEqual({ row: 7, col: 6 });
    expect(nextReplayMove(replay, replay.move_count)).toBeNull();
  });
});
