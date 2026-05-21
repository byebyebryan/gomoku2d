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

export type ReplayAnalysisDefenderReplyRole =
  | "actual"
  | "immediate_defense"
  | "imminent_defense"
  | "offensive_counter";

export type ReplayAnalysisFailureMode =
  | "missed_immediate_win"
  | "missed_immediate_response"
  | "missed_imminent_response"
  | "missed_escape"
  | "missed_lethal_prevention"
  | "unclear";

export type ReplayAnalysisFailureConfidence = "confirmed" | "possible" | "unclear";

export type ReplayAnalysisMissedCandidateOutcome =
  | "confirmed_escape"
  | "possible_escape"
  | "prevents_lethal_onset"
  | "prevents_corridor_entry";

export interface ReplayAnalysisMissedCandidate {
  mv: ReplayAnalysisMove;
  notation: string;
  outcome: ReplayAnalysisMissedCandidateOutcome;
  roles: ReplayAnalysisDefenderReplyRole[];
}

export interface ReplayAnalysisFailure {
  actual_move?: ReplayAnalysisMove | null;
  actual_notation?: string | null;
  confidence: ReplayAnalysisFailureConfidence;
  missed_candidates: ReplayAnalysisMissedCandidate[];
  mode: ReplayAnalysisFailureMode;
  prefix_ply?: number | null;
  prevented_onset_ply?: number | null;
  side: ReplayAnalysisSide;
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

export interface ReplayAnalysisInterval {
  end_ply: number;
  start_ply: number;
}

export type ReplayAnalysisLethalOnsetKind = "terminal_coverage" | "one_step_coverage";
export type ReplayAnalysisLethalOnsetComponentTier = "four" | "three";
export type ReplayAnalysisLethalOnsetMechanism = "multi_route" | "forbidden_cover";

export interface ReplayAnalysisLethalOnsetComponent {
  mv: ReplayAnalysisMove;
  tier: ReplayAnalysisLethalOnsetComponentTier;
}

export interface ReplayAnalysisLethalOnsetShape {
  components: ReplayAnalysisLethalOnsetComponent[];
  label: string;
  mechanisms: ReplayAnalysisLethalOnsetMechanism[];
}

export interface ReplayAnalysisLethalOnsetEntry {
  mv: ReplayAnalysisMove;
  terminal_targets: ReplayAnalysisMove[];
}

export interface ReplayAnalysisLethalOnsetReply {
  lethal_entries: ReplayAnalysisLethalOnsetEntry[];
  reply: ReplayAnalysisMove;
}

export interface ReplayAnalysisLethalOnset {
  attacker?: ReplayAnalysisSide;
  covering_replies?: ReplayAnalysisMove[];
  defender?: ReplayAnalysisSide;
  kind?: ReplayAnalysisLethalOnsetKind;
  prefix_ply: number;
  shape?: ReplayAnalysisLethalOnsetShape;
  one_step_replies?: ReplayAnalysisLethalOnsetReply[];
  terminal_targets?: ReplayAnalysisMove[];
}

export interface ReplayAnalysisSummary {
  failure?: ReplayAnalysisFailure | null;
  final_forced_interval?: ReplayAnalysisInterval | null;
  lethal_onset?: ReplayAnalysisLethalOnset | null;
  schema_version?: number;
  setup_corridor?: ReplayAnalysisInterval | null;
}

export interface ReplayAnalysisStepResult {
  analysis: ReplayAnalysisSummary | null;
  annotations: ReplayFrameAnnotations[];
  counters: ReplayAnalysisCounters;
  current_ply: number | null;
  done: boolean;
  error: string | null;
  schema_version: number;
  status: ReplayAnalysisStatus;
}

export function replayAnalysisErrorResult(error: string): ReplayAnalysisStepResult {
  return {
    analysis: null,
    annotations: [],
    counters: { branch_roots: 0, prefixes_analyzed: 0, proof_nodes: 0 },
    current_ply: null,
    done: true,
    error,
    schema_version: 1,
    status: "error",
  };
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
