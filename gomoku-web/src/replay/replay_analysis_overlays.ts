import type { BoardAnalysisOverlay } from "../board/board_scene_logic";
import type { CellPosition } from "../game/types";
import {
  movesFromMoveCells,
  savedMatchWinningSide,
  type SavedMatchV2,
} from "../match/saved_match";

import type {
  ReplayAnalysisSide,
  ReplayAnalysisStepResult,
  ReplayFrameAnnotations,
  ReplayFrameHighlightRole,
  ReplayFrameMarkerRole,
} from "./replay_analysis_protocol";

export type ReplayAnalysisAnnotationsByPly = Record<number, ReplayFrameAnnotations>;

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
      return "confirmedEscape";
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
