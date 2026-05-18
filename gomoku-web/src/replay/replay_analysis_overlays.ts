import type { BoardAnalysisOverlay } from "../board/board_scene_logic";
import type { CellPosition, MatchStatus } from "../game/types";
import {
  movesFromMoveCells,
  savedMatchWinningSide,
  type SavedMatchV2,
} from "../match/saved_match";

import type {
  ReplayAnalysisSide,
  ReplayAnalysisStepResult,
  ReplayAnalysisSummary,
  ReplayFrameAnnotations,
  ReplayFrameHighlightRole,
  ReplayFrameMarkerRole,
} from "./replay_analysis_protocol";

export type ReplayAnalysisAnnotationsByPly = Record<number, ReplayFrameAnnotations>;

export type ReplayTimelineAnalysis = {
  setupEndPercent: string | null;
  setupEndPly: number | null;
  setupStartPercent: string | null;
  setupStartPly: number | null;
  escapePercent: string | null;
  escapePly: number | null;
};

export type ReplayAnalysisStatusSummary = {
  detail: string;
  label: string;
};

export type ReplayAnalysisStatusFrame = {
  currentPlayer: 1 | 2;
  moveIndex: number;
  status: MatchStatus;
};

function loserSideToMove(match: Pick<SavedMatchV2, "status">): ReplayAnalysisSide | null {
  const winningSide = savedMatchWinningSide(match);

  if (winningSide === "black") return "White";
  if (winningSide === "white") return "Black";
  return null;
}

function overlaySide(side: ReplayAnalysisSide): BoardAnalysisOverlay["side"] {
  return side === "Black" ? "black" : "white";
}

function highlightRole(role: ReplayFrameHighlightRole): BoardAnalysisOverlay["highlight"] {
  switch (role) {
    case "corridor_entry":
      return "corridorEntry";
    case "counter_threat":
      return "counterThreat";
    case "immediate_threat":
      return "immediateThreat";
    case "immediate_win":
      return "immediateWin";
    case "imminent_threat":
      return "imminentThreat";
  }
}

function markerRole(role: ReplayFrameMarkerRole): BoardAnalysisOverlay["marker"] | null {
  switch (role) {
    case "confirmed_escape":
    case "possible_escape":
      return "escape";
    case "forbidden":
      return "forbidden";
    case "forced_loss":
      return "forcedLoss";
    case "immediate_loss":
      return "immediateLoss";
    case "unknown":
      return null;
  }
}

function percentForPly(ply: number, totalMoves: number): string {
  if (totalMoves <= 0) {
    return "0%";
  }

  const clamped = Math.max(0, Math.min(totalMoves, ply));
  return `${(clamped / totalMoves) * 100}%`;
}

function annotatedPlies(annotationsByPly: ReplayAnalysisAnnotationsByPly): number[] {
  return Object.keys(annotationsByPly)
    .map((ply) => Number(ply))
    .filter((ply) => Number.isFinite(ply));
}

function escapePlies(annotationsByPly: ReplayAnalysisAnnotationsByPly): number[] {
  return Object.values(annotationsByPly)
    .filter((annotation) => annotation.markers.some((marker) => (
      marker.role === "confirmed_escape" || marker.role === "possible_escape"
    )))
    .map((annotation) => annotation.ply)
    .filter((ply) => Number.isFinite(ply));
}

function nodeDetail(step: Pick<ReplayAnalysisStepResult, "counters">): string {
  return `${step.counters.proof_nodes} nodes`;
}

function sideNameForPlayer(player: 1 | 2): ReplayAnalysisSide {
  return player === 1 ? "Black" : "White";
}

function winningSideName(match: Pick<SavedMatchV2, "status">): ReplayAnalysisSide | null {
  const winningSide = savedMatchWinningSide(match);
  if (winningSide === "black") return "Black";
  if (winningSide === "white") return "White";
  return null;
}

function setupCorridorDetail(timeline: ReplayTimelineAnalysis, totalMoves: number): string {
  if (timeline.setupStartPly === null || timeline.setupEndPly === null) {
    return "No setup corridor found";
  }

  const startMove = Math.min(totalMoves, timeline.setupStartPly + 1);
  const endMove = Math.min(totalMoves, timeline.setupEndPly);
  return `Setup corridor: moves ${startMove}-${endMove}`;
}

function setupCorridorFromAnalysis(analysis: ReplayAnalysisSummary | null | undefined) {
  const setup = analysis?.setup_corridor ?? null;
  if (
    setup &&
    Number.isFinite(setup.start_ply) &&
    Number.isFinite(setup.end_ply) &&
    setup.end_ply >= setup.start_ply
  ) {
    return setup;
  }
  return null;
}

export function replayTimelineAnalysis(
  annotationsByPly: ReplayAnalysisAnnotationsByPly,
  totalMoves: number,
  analysis?: ReplayAnalysisSummary | null,
): ReplayTimelineAnalysis {
  const plies = annotatedPlies(annotationsByPly);
  const setupCorridor = setupCorridorFromAnalysis(analysis);
  if (plies.length === 0 && !setupCorridor) {
    return {
      setupEndPercent: null,
      setupEndPly: null,
      setupStartPercent: null,
      setupStartPly: null,
      escapePercent: null,
      escapePly: null,
    };
  }

  const setupStartPly = setupCorridor ? setupCorridor.start_ply : Math.min(...plies);
  const setupEndPly = setupCorridor ? setupCorridor.end_ply : Math.max(totalMoves, ...plies);
  const escapes = escapePlies(annotationsByPly);
  const escapePly = escapes.length > 0 ? Math.min(...escapes) : null;

  return {
    setupEndPercent: percentForPly(setupEndPly, totalMoves),
    setupEndPly,
    setupStartPercent: percentForPly(setupStartPly, totalMoves),
    setupStartPly,
    escapePercent: escapePly === null ? null : percentForPly(escapePly, totalMoves),
    escapePly,
  };
}

export function replayAnalysisStatusSummary(
  step: ReplayAnalysisStepResult | null,
  annotationsByPly: ReplayAnalysisAnnotationsByPly,
  match: Pick<SavedMatchV2, "move_count" | "status">,
  frame: ReplayAnalysisStatusFrame,
): ReplayAnalysisStatusSummary {
  const totalMoves = match.move_count;
  if (!step) {
    return {
      detail: "Waiting for analyzer",
      label: "Analyzing replay",
    };
  }

  if (step.status === "running") {
    return {
      detail: step.current_ply === null
        ? nodeDetail(step)
        : `Move ${step.current_ply} · ${nodeDetail(step)}`,
      label: "Analyzing replay",
    };
  }

  if (step.status === "resolved") {
    const timeline = replayTimelineAnalysis(annotationsByPly, totalMoves, step.analysis);
    const winner = winningSideName(match);
    const loser = loserSideToMove(match);
    const currentSide = sideNameForPlayer(frame.currentPlayer);
    const insideSetupCorridor = (
      timeline.setupStartPly !== null &&
      timeline.setupEndPly !== null &&
      frame.moveIndex >= timeline.setupStartPly &&
      frame.moveIndex <= timeline.setupEndPly
    );
    const afterSetupCorridor = (
      timeline.setupEndPly !== null &&
      frame.moveIndex > timeline.setupEndPly
    );

    if (winner && frame.status !== "playing") {
      if (timeline.setupStartPly !== null && timeline.setupEndPly !== null) {
        return {
          detail: `${Math.max(1, timeline.setupEndPly - timeline.setupStartPly + 1)}-ply setup corridor`,
          label: `${winner} won`,
        };
      }

      return {
        detail: nodeDetail(step),
        label: `${winner} won`,
      };
    }

    if (!insideSetupCorridor) {
      if (winner && afterSetupCorridor) {
        return {
          detail: "After lethal onset",
          label: `${winner} has a guaranteed win`,
        };
      }

      return {
        detail: "Outside the setup corridor",
        label: `${currentSide} to move`,
      };
    }

    if (timeline.escapePly !== null) {
      if (frame.moveIndex === timeline.escapePly && loser) {
        return {
          detail: `Last chance before move ${Math.min(totalMoves, timeline.escapePly + 1)}`,
          label: `${loser}'s last escape`,
        };
      }

      if (winner && currentSide === winner) {
        return {
          detail: setupCorridorDetail(timeline, totalMoves),
          label: `${winner} can force a win`,
        };
      }

      if (loser && currentSide === loser) {
        return {
          detail: "No viable escape found",
          label: `${loser} is locked in`,
        };
      }

      return {
        detail: `Before move ${Math.min(totalMoves, timeline.escapePly + 1)} · ${nodeDetail(step)}`,
        label: "Last escape found",
      };
    }

    if (timeline.setupStartPly !== null) {
      return {
        detail: `From move ${Math.min(totalMoves, timeline.setupStartPly + 1)} · ${nodeDetail(step)}`,
        label: "Setup corridor found",
      };
    }

    return {
      detail: "Outside the setup corridor",
      label: `${currentSide} to move`,
    };
  }

  if (step.status === "unclear") {
    return {
      detail: `Traceback unresolved · ${nodeDetail(step)}`,
      label: "Analysis unclear",
    };
  }

  if (step.status === "unsupported") {
    return {
      detail: step.error ?? "This replay cannot be analyzed",
      label: "Analysis unavailable",
    };
  }

  return {
    detail: step.error ?? "Analyzer failed",
    label: "Analysis error",
  };
}

export function mergeReplayAnalysisAnnotations(
  current: ReplayAnalysisAnnotationsByPly,
  step: Pick<ReplayAnalysisStepResult, "annotations">,
): ReplayAnalysisAnnotationsByPly {
  if (step.annotations.length === 0) {
    return current;
  }

  const next = { ...current };
  for (const annotation of step.annotations) {
    next[annotation.ply] = annotation;
  }
  return next;
}

export function analysisOverlaysForFrame(
  annotationsByPly: ReplayAnalysisAnnotationsByPly,
  match: SavedMatchV2,
  moveIndex: number,
): BoardAnalysisOverlay[] {
  const annotation = annotationsByPly[moveIndex];
  const loserSide = loserSideToMove(match);

  if (!annotation || !loserSide || annotation.side_to_move !== loserSide) {
    return [];
  }

  return [
    ...annotation.highlights.map((highlight) => ({
      col: highlight.mv.col,
      highlight: highlightRole(highlight.role),
      row: highlight.mv.row,
      side: overlaySide(highlight.side),
    })),
    ...annotation.markers.flatMap((marker) => {
      const role = markerRole(marker.role);
      if (!role) {
        return [];
      }

      return [{
        col: marker.mv.col,
        marker: role,
        row: marker.mv.row,
      }];
    }),
  ];
}

export function nextReplayMove(match: SavedMatchV2, moveIndex: number): CellPosition | null {
  const move = movesFromMoveCells(match.move_cells)[moveIndex];
  return move ? { row: move.row, col: move.col } : null;
}
