export type ReplayAnalysisStatus = "running" | "resolved" | "unclear" | "unsupported" | "error";
export type ReplayAnalysisSide = "Black" | "White";

export type ReplayFrameHighlightRole =
  | "immediate_win"
  | "immediate_threat"
  | "imminent_threat"
  | "counter_threat"
  | "corridor_entry";

export type ReplayFrameMarkerRole =
  | "confirmed_escape"
  | "possible_escape"
  | "forced_loss"
  | "immediate_loss"
  | "forbidden"
  | "unknown";

export interface ReplayAnalysisMove {
  col: number;
  row: number;
}

export interface ReplayFrameHighlight {
  mv: ReplayAnalysisMove;
  notation: string;
  role: ReplayFrameHighlightRole;
  side: ReplayAnalysisSide;
}

export interface ReplayFrameMarker {
  mv: ReplayAnalysisMove;
  notation: string;
  role: ReplayFrameMarkerRole;
  side: ReplayAnalysisSide;
}

export interface ReplayFrameAnnotations {
  highlights: ReplayFrameHighlight[];
  markers: ReplayFrameMarker[];
  ply: number;
  side_to_move: ReplayAnalysisSide;
}

export interface ReplayAnalysisCounters {
  branch_roots: number;
  prefixes_analyzed: number;
  proof_nodes: number;
}

export interface ReplayAnalysisStepResult {
  analysis: unknown | null;
  annotations: ReplayFrameAnnotations[];
  counters: ReplayAnalysisCounters;
  current_ply: number | null;
  done: boolean;
  error: string | null;
  schema_version: number;
  status: ReplayAnalysisStatus;
}

export type ReplayAnalysisWorkerRequest =
  | {
      optionsJson: string;
      replayJson: string;
      requestId: number;
      stepWorkUnits: number;
      type: "analyze";
    }
  | { requestId: number; type: "cancel" };

export type ReplayAnalysisWorkerResponse =
  | { type: "ready" }
  | { requestId: number; result: ReplayAnalysisStepResult; type: "progress" }
  | { requestId: number; result: ReplayAnalysisStepResult; type: "complete" }
  | { requestId: number; type: "cancelled" }
  | { message: string; requestId?: number; type: "error" };
