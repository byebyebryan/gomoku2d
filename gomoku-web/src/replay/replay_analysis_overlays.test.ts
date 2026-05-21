import { describe, expect, it } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";
import type { MatchMove } from "../game/types";
import type { ReplayFrameAnnotations, ReplayAnalysisStepResult } from "./replay_analysis_protocol";

import {
  analysisOverlaysForFrame,
  mergeReplayAnalysisAnnotations,
  nextReplayMove,
  replayAnalysisStatusSummary,
  replayFailureSummary,
  replayMistakePoint,
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
  it("marks the setup corridor range and escape point", () => {
    const escapeFrame = annotation(4, "White");
    escapeFrame.markers = [{ ...escapeFrame.markers[0], role: "possible_escape" }];
    const annotations = mergeReplayAnalysisAnnotations(
      {},
      step(annotation(8, "White"), escapeFrame, annotation(6, "White")),
    );

    expect(replayTimelineAnalysis(annotations, 10)).toEqual({
      analyzedEndPercent: "100%",
      analyzedStartPercent: "40%",
      lethalOnsetPercent: null,
      lethalOnsetPly: null,
      lethalTailEndPercent: null,
      lethalTailStartPercent: null,
      setupEndPercent: null,
      setupEndPly: null,
      setupStartPercent: null,
      setupStartPly: null,
      escapePercent: "40%",
      escapePly: 4,
    });
  });

  it("uses the analyzer setup corridor when resolved analysis is available", () => {
    const annotations = mergeReplayAnalysisAnnotations(
      {},
      step(annotation(8, "White"), annotation(10, "White")),
    );

    expect(replayTimelineAnalysis(annotations, 10, {
      lethal_onset: { prefix_ply: 7 },
      setup_corridor: { start_ply: 4, end_ply: 7 },
    })).toEqual({
      analyzedEndPercent: "100%",
      analyzedStartPercent: "80%",
      lethalOnsetPercent: "70%",
      lethalOnsetPly: 7,
      lethalTailEndPercent: "100%",
      lethalTailStartPercent: "70%",
      setupEndPercent: "70%",
      setupEndPly: 7,
      setupStartPercent: "40%",
      setupStartPly: 4,
      escapePercent: null,
      escapePly: null,
    });
  });

  it("extends the setup corridor fill back to the escape marker", () => {
    const escapeFrame = annotation(4, "Black");
    escapeFrame.markers = [{ ...escapeFrame.markers[0], role: "confirmed_escape" }];
    const annotations = mergeReplayAnalysisAnnotations({}, step(escapeFrame, annotation(6, "White"), annotation(8, "White")));

    expect(replayTimelineAnalysis(annotations, 10, {
      lethal_onset: { prefix_ply: 8 },
      setup_corridor: { start_ply: 6, end_ply: 8 },
    })).toEqual(expect.objectContaining({
      escapePercent: "40%",
      escapePly: 4,
      setupEndPercent: "80%",
      setupEndPly: 8,
      setupStartPercent: "40%",
      setupStartPly: 6,
    }));
  });

  it("omits timeline analysis when no frame annotations are available", () => {
    expect(replayTimelineAnalysis({}, 10)).toEqual({
      analyzedEndPercent: null,
      analyzedStartPercent: null,
      lethalOnsetPercent: null,
      lethalOnsetPly: null,
      lethalTailEndPercent: null,
      lethalTailStartPercent: null,
      setupEndPercent: null,
      setupEndPly: null,
      setupStartPercent: null,
      setupStartPly: null,
      escapePercent: null,
      escapePly: null,
    });
  });

  it("can render summary-only lethal onset data", () => {
    expect(replayTimelineAnalysis({}, 10, {
      lethal_onset: { prefix_ply: 8 },
    })).toEqual({
      analyzedEndPercent: null,
      analyzedStartPercent: null,
      lethalOnsetPercent: "80%",
      lethalOnsetPly: 8,
      lethalTailEndPercent: "100%",
      lethalTailStartPercent: "80%",
      setupEndPercent: null,
      setupEndPly: null,
      setupStartPercent: null,
      setupStartPly: null,
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

  it("summarizes the terminal frame with the winner", () => {
    const escapeFrame = annotation(4, "White");
    escapeFrame.markers = [{ ...escapeFrame.markers[0], role: "confirmed_escape" }];
    const annotations = mergeReplayAnalysisAnnotations({}, step(escapeFrame, annotation(8, "White")));
    const resolved = {
      ...step(escapeFrame),
      analysis: { setup_corridor: { start_ply: 4, end_ply: 8 } },
      counters: { branch_roots: 1, prefixes_analyzed: 6, proof_nodes: 2048 },
      done: true,
      status: "resolved" as const,
    };

    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 10,
      status: "black_won",
    })).toEqual({
      detail: "Lethal sequence",
      label: "Black has won",
    });
  });

  it("summarizes the current frame inside and outside the setup corridor", () => {
    const escapeFrame = annotation(4, "White");
    escapeFrame.markers = [{ ...escapeFrame.markers[0], role: "confirmed_escape" }];
    const annotations = mergeReplayAnalysisAnnotations({}, step(escapeFrame, annotation(6, "White"), annotation(8, "White")));
    const resolved = {
      ...step(escapeFrame),
      analysis: { setup_corridor: { start_ply: 4, end_ply: 8 } },
      counters: { branch_roots: 1, prefixes_analyzed: 6, proof_nodes: 2048 },
      done: true,
      status: "resolved" as const,
    };

    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 4,
      status: "playing",
    })).toEqual({
      detail: "Last chance to avoid loss",
      label: "White's last escape",
    });
    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 1,
      moveIndex: 7,
      status: "playing",
    })).toEqual({
      detail: "No viable escape",
      label: "Black can force a win",
    });
    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 8,
      status: "playing",
    })).toEqual({
      detail: "No viable escape",
      label: "White is locked in",
    });
    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 1,
      moveIndex: 2,
      status: "playing",
    })).toEqual({
      detail: "Normal play",
      label: "Black to move",
    });
    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 9,
      status: "playing",
    })).toEqual({
      detail: "Lethal sequence",
      label: "Black has won",
    });
  });

  it("uses lethal onset as the setup-corridor phase boundary", () => {
    const escapeFrame = annotation(4, "White");
    escapeFrame.markers = [{ ...escapeFrame.markers[0], role: "confirmed_escape" }];
    const annotations = mergeReplayAnalysisAnnotations({}, step(escapeFrame, annotation(6, "White"), annotation(8, "White")));
    const resolved = {
      ...step(escapeFrame),
      analysis: {
        lethal_onset: {
          prefix_ply: 8,
          shape: {
            components: [],
            label: "4x3",
            mechanisms: ["multi_route" as const],
          },
        },
        setup_corridor: { start_ply: 4, end_ply: 8 },
      },
      counters: { branch_roots: 1, prefixes_analyzed: 6, proof_nodes: 2048 },
      done: true,
      status: "resolved" as const,
    };

    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 1,
      moveIndex: 7,
      status: "playing",
    })).toEqual({
      detail: "No viable escape",
      label: "Black can force a win",
    });
    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 8,
      status: "playing",
    })).toEqual({
      detail: "by 4+3",
      label: "White has lost",
    });
    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 1,
      moveIndex: 9,
      status: "playing",
    })).toEqual({
      detail: "by 4+3",
      label: "Black has won",
    });
  });

  it("summarizes open-four onset as a local lethal shape", () => {
    const resolved = {
      ...step(annotation(8, "White")),
      analysis: {
        lethal_onset: {
          kind: "terminal_coverage" as const,
          prefix_ply: 8,
          shape: {
            components: [],
            label: "4",
            mechanisms: ["multi_route" as const],
          },
          terminal_targets: [{ row: 0, col: 2 }, { row: 0, col: 7 }],
        },
        setup_corridor: { start_ply: 6, end_ply: 8 },
      },
      done: true,
      status: "resolved" as const,
    };

    expect(replayAnalysisStatusSummary(resolved, {}, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 10,
      status: "black_won",
    })).toEqual({
      detail: "by open four",
      label: "Black has won",
    });
  });

  it("shows the last escape even when it sits before the setup corridor span", () => {
    const escapeFrame = annotation(4, "Black");
    escapeFrame.markers = [{ ...escapeFrame.markers[0], role: "confirmed_escape" }];
    const annotations = mergeReplayAnalysisAnnotations({}, step(escapeFrame, annotation(6, "White"), annotation(8, "White")));
    const resolved = {
      ...step(escapeFrame),
      analysis: {
        lethal_onset: { prefix_ply: 8 },
        setup_corridor: { start_ply: 6, end_ply: 8 },
      },
      counters: { branch_roots: 1, prefixes_analyzed: 6, proof_nodes: 2048 },
      done: true,
      status: "resolved" as const,
    };

    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("white_won"), {
      currentPlayer: 1,
      moveIndex: 4,
      status: "playing",
    })).toEqual({
      detail: "Last chance to avoid loss",
      label: "Black's last escape",
    });
  });

  it("keeps escape-frame copy concise when failure candidates exist", () => {
    const escapeFrame = annotation(5, "White");
    escapeFrame.markers = [{ ...escapeFrame.markers[0], role: "confirmed_escape" }];
    const annotations = mergeReplayAnalysisAnnotations({}, step(escapeFrame));
    const resolved = {
      ...step(escapeFrame),
      analysis: {
        failure: {
          actual_move: { row: 0, col: 1 },
          actual_notation: "B1",
          confidence: "confirmed" as const,
          missed_candidates: [
            {
              mv: { row: 7, col: 6 },
              notation: "G8",
              outcome: "confirmed_escape" as const,
              roles: ["imminent_defense" as const],
            },
          ],
          mode: "missed_imminent_response" as const,
          prefix_ply: 5,
          prevented_onset_ply: null,
          side: "White" as const,
        },
      },
      done: true,
      status: "resolved" as const,
    };

    expect(replayAnalysisStatusSummary(resolved, annotations, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 5,
      status: "playing",
    })).toEqual({
      detail: "Last chance to avoid loss",
      label: "White's last escape",
    });
  });

  it("summarizes the classified failure on its frame", () => {
    const resolved = {
      ...step(annotation(4, "White")),
      analysis: {
        failure: {
          actual_move: { row: 0, col: 1 },
          actual_notation: "B1",
          confidence: "confirmed" as const,
          missed_candidates: [
            {
              mv: { row: 7, col: 6 },
              notation: "G8",
              outcome: "confirmed_escape" as const,
              roles: ["imminent_defense" as const],
            },
          ],
          mode: "missed_imminent_response" as const,
          prefix_ply: 5,
          prevented_onset_ply: null,
          side: "White" as const,
        },
        setup_corridor: { start_ply: 5, end_ply: 8 },
      },
      done: true,
      status: "resolved" as const,
    };

    expect(replayAnalysisStatusSummary(resolved, {}, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 5,
      status: "playing",
    })).toEqual({
      detail: "Played B1 · Response: G8",
      label: "Missed 3",
    });
    expect(replayAnalysisStatusSummary(resolved, {}, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 10,
      status: "black_won",
    })).toEqual({
      detail: "Lethal sequence",
      label: "Black has won",
    });
  });

  it("does not show non-actionable failure copy on terminal frames", () => {
    const resolved = {
      ...step(annotation(4, "White")),
      analysis: {
        failure: {
          actual_move: null,
          actual_notation: null,
          confidence: "confirmed" as const,
          missed_candidates: [],
          mode: "missed_escape" as const,
          prefix_ply: 8,
          prevented_onset_ply: 8,
          side: "White" as const,
        },
      },
      done: true,
      status: "resolved" as const,
    };

    expect(replayAnalysisStatusSummary(resolved, {}, tenMoveMatch("black_won"), {
      currentPlayer: 2,
      moveIndex: 10,
      status: "black_won",
    })).toEqual({
      detail: "Lethal sequence",
      label: "Black has won",
    });
  });
});

describe("replayFailureSummary", () => {
  it("returns null for unclear or missing failures", () => {
    expect(replayFailureSummary(null)).toBeNull();
    expect(replayFailureSummary({
      failure: {
        actual_move: null,
        actual_notation: null,
        confidence: "unclear",
        missed_candidates: [],
        mode: "unclear",
        prefix_ply: 4,
        prevented_onset_ply: null,
        side: "White",
      },
    })).toBeNull();
  });

  it("separates non-actionable failure summaries from mistake points", () => {
    const analysis = {
      failure: {
        actual_move: null,
        actual_notation: null,
        confidence: "confirmed" as const,
        missed_candidates: [],
        mode: "missed_escape" as const,
        prefix_ply: 8,
        prevented_onset_ply: 8,
        side: "White" as const,
      },
    };

    expect(replayFailureSummary(analysis)).toEqual(expect.objectContaining({
      actualMove: null,
      criticalPly: 8,
      label: "Missed escape",
    }));
    expect(replayMistakePoint(analysis)).toBeNull();
  });

  it("uses lethal onset shape copy for missed lethal prevention", () => {
    expect(replayFailureSummary({
      failure: {
        actual_move: { row: 0, col: 1 },
        actual_notation: "B1",
        confidence: "confirmed" as const,
        missed_candidates: [],
        mode: "missed_lethal_prevention" as const,
        prefix_ply: 8,
        prevented_onset_ply: 8,
        side: "White" as const,
      },
      lethal_onset: {
        prefix_ply: 8,
        shape: {
          components: [],
          label: "4x3",
          mechanisms: ["multi_route" as const],
        },
      },
    })).toEqual(expect.objectContaining({
      label: "Missed 4+3",
    }));

    expect(replayFailureSummary({
      failure: {
        actual_move: { row: 0, col: 1 },
        actual_notation: "B1",
        confidence: "confirmed" as const,
        missed_candidates: [],
        mode: "missed_lethal_prevention" as const,
        prefix_ply: 8,
        prevented_onset_ply: 8,
        side: "Black" as const,
      },
      lethal_onset: {
        prefix_ply: 8,
        shape: {
          components: [],
          label: "4",
          mechanisms: ["forbidden_cover" as const],
        },
      },
    })).toEqual(expect.objectContaining({
      label: "Missed forbidden 4",
    }));

    expect(replayFailureSummary({
      failure: {
        actual_move: { row: 0, col: 1 },
        actual_notation: "B1",
        confidence: "confirmed" as const,
        missed_candidates: [],
        mode: "missed_lethal_prevention" as const,
        prefix_ply: 8,
        prevented_onset_ply: 8,
        side: "White" as const,
      },
      lethal_onset: {
        kind: "terminal_coverage" as const,
        prefix_ply: 8,
        shape: {
          components: [],
          label: "4",
          mechanisms: ["multi_route" as const],
        },
        terminal_targets: [{ row: 0, col: 2 }, { row: 0, col: 7 }],
      },
    })).toEqual(expect.objectContaining({
      label: "Missed open four",
    }));
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
      expect.objectContaining({ marker: "escape", row: 7, col: 8 }),
      expect.objectContaining({ marker: "forbidden", row: 7, col: 9 }),
    ]);
  });

  it("marks only the classified actual mistake move", () => {
    expect(analysisOverlaysForFrame({}, match("black_won"), 5, {
      failure: {
        actual_move: { row: 0, col: 1 },
        actual_notation: "B1",
        confidence: "confirmed",
        missed_candidates: [
          {
            mv: { row: 7, col: 6 },
            notation: "G8",
            outcome: "confirmed_escape",
            roles: ["imminent_defense"],
          },
        ],
        mode: "missed_imminent_response",
        prefix_ply: 5,
        prevented_onset_ply: null,
        side: "White",
      },
    })).toEqual([
      expect.objectContaining({ marker: "mistake", row: 0, col: 1 }),
    ]);
    expect(analysisOverlaysForFrame({}, match("black_won"), 4, {
      failure: {
        actual_move: { row: 0, col: 1 },
        actual_notation: "B1",
        confidence: "confirmed",
        missed_candidates: [],
        mode: "missed_imminent_response",
        prefix_ply: 5,
        prevented_onset_ply: null,
        side: "White",
      },
    })).toEqual([]);
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
