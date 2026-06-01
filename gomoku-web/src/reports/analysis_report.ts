import type { ReportProvenance } from "./bot_report";

export interface MovePoint {
  row: number;
  col: number;
}

export interface PublishedAnalysisReport {
  schema_version: number;
  report_kind: "published_analysis";
  source_kind: string;
  source_report: string;
  provenance?: ReportProvenance;
  source_report_provenance?: ReportProvenance | null;
  selector: string;
  total: number;
  analyzed: number;
  failed: number;
  elapsed_ms: number;
  total_elapsed_ms: number;
  model: {
    max_depth: number;
    max_scan_plies?: number | null;
  };
  summary: {
    unclear: number;
    ongoing_or_draw: number;
    analysis_error: number;
  };
  sections: AnalysisSection[];
}

export interface AnalysisSection {
  label: string;
  entrant_a: string;
  entrant_b: string;
  total: number;
  analyzed: number;
  failed: number;
  summary: {
    unclear: number;
    ongoing_or_draw: number;
    analysis_error: number;
  };
  entries: AnalysisEntry[];
}

export interface AnalysisEntry {
  path: string;
  match_report: {
    match_index: number;
    black: string;
    white: string;
    result: string;
    winner?: string | null;
    end_reason: string;
    move_cells: number[];
    move_count: number;
  };
  status: string;
  root_cause?: string | null;
  unclear_reason?: string | null;
  lethal_onset?: {
    prefix_ply: number;
    attacker: string;
    defender: string;
    kind?: string;
    shape?: {
      label?: string;
      mechanisms?: string[];
    };
    terminal_targets?: MovePoint[];
  } | null;
  setup_corridor?: {
    start_ply: number;
    end_ply: number;
  } | null;
  last_chance_ply?: number | null;
  critical_loser_ply?: number | null;
  failure?: {
    ply: number;
    side: string;
    mode: string;
    actual_move?: MovePoint | null;
    actual_notation?: string | null;
    summary?: string | null;
    missed_candidates?: Array<{
      mv: MovePoint;
      notation: string;
      role: string;
    }>;
  } | null;
  proof_details?: {
    proof_frames: ProofFrame[];
  } | null;
  search_details?: {
    search_nodes: number;
    branch_probes: number;
    max_depth_reached: number;
  } | null;
  elapsed_ms: number;
  error?: string | null;
}

export interface ProofFrame {
  label: string;
  ply: number;
  side_to_move: string;
  status: string;
  move_played_notation?: string | null;
  lethal_onset_reached: boolean;
  markers: Array<{
    notation: string;
    kinds: string[];
  }>;
  reply_outcomes: Array<{
    notation: string;
    roles: string[];
    outcome: string;
  }>;
}

const ANALYSIS_REPORT_URL = `${import.meta.env.BASE_URL}analysis-report/report.json`;

export async function loadAnalysisReport(): Promise<PublishedAnalysisReport> {
  const response = await fetch(ANALYSIS_REPORT_URL, { cache: "no-cache" });
  if (!response.ok) {
    throw new Error(`Failed to load analysis report (${response.status})`);
  }

  const data = (await response.json()) as unknown;
  if (!isAnalysisReport(data)) {
    throw new Error("Analysis report has an unsupported schema.");
  }
  return data;
}

function isAnalysisReport(data: unknown): data is PublishedAnalysisReport {
  if (!isObject(data)) {
    return false;
  }
  const report = data as Partial<PublishedAnalysisReport>;
  return (
    (report.schema_version === 2 || report.schema_version === 3 || report.schema_version === 4) &&
    report.report_kind === "published_analysis" &&
    typeof report.source_kind === "string" &&
    typeof report.source_report === "string" &&
    typeof report.selector === "string" &&
    typeof report.analyzed === "number" &&
    typeof report.failed === "number" &&
    typeof report.elapsed_ms === "number" &&
    typeof report.total_elapsed_ms === "number" &&
    isAnalysisModel(report.model) &&
    isAnalysisSummary(report.summary) &&
    Array.isArray(report.sections) &&
    report.sections.every(isAnalysisSection) &&
    typeof report.total === "number"
  );
}

function isObject(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

function isMoveCellArray(value: unknown): value is number[] {
  return Array.isArray(value) && value.every((item) => Number.isInteger(item) && item >= 0);
}

function hasNumberFields(value: Record<string, unknown>, fields: string[]): boolean {
  return fields.every((field) => typeof value[field] === "number");
}

function isAnalysisModel(value: unknown): value is PublishedAnalysisReport["model"] {
  return isObject(value) && typeof value.max_depth === "number";
}

function isAnalysisSummary(value: unknown): value is PublishedAnalysisReport["summary"] {
  return (
    isObject(value) &&
    hasNumberFields(value, ["unclear", "ongoing_or_draw", "analysis_error"])
  );
}

function isAnalysisSection(value: unknown): value is AnalysisSection {
  return (
    isObject(value) &&
    typeof value.label === "string" &&
    typeof value.entrant_a === "string" &&
    typeof value.entrant_b === "string" &&
    hasNumberFields(value, ["total", "analyzed", "failed"]) &&
    isAnalysisSummary(value.summary) &&
    Array.isArray(value.entries) &&
    value.entries.every(isAnalysisEntry)
  );
}

function isAnalysisEntry(value: unknown): value is AnalysisEntry {
  return (
    isObject(value) &&
    typeof value.path === "string" &&
    isAnalysisMatchSummary(value.match_report) &&
    typeof value.status === "string" &&
    typeof value.elapsed_ms === "number" &&
    (value.proof_details == null || isProofDetails(value.proof_details)) &&
    (value.search_details == null || isSearchDetails(value.search_details))
  );
}

function isAnalysisMatchSummary(value: unknown): value is AnalysisEntry["match_report"] {
  return (
    isObject(value) &&
    typeof value.black === "string" &&
    typeof value.white === "string" &&
    typeof value.result === "string" &&
    typeof value.end_reason === "string" &&
    hasNumberFields(value, ["match_index", "move_count"]) &&
    isMoveCellArray(value.move_cells)
  );
}

function isProofDetails(value: unknown): value is NonNullable<AnalysisEntry["proof_details"]> {
  return isObject(value) && Array.isArray(value.proof_frames) && value.proof_frames.every(isProofFrame);
}

function isProofFrame(value: unknown): value is ProofFrame {
  return (
    isObject(value) &&
    typeof value.label === "string" &&
    typeof value.ply === "number" &&
    typeof value.side_to_move === "string" &&
    typeof value.status === "string" &&
    typeof value.lethal_onset_reached === "boolean" &&
    Array.isArray(value.markers) &&
    value.markers.every(isProofMarker) &&
    Array.isArray(value.reply_outcomes) &&
    value.reply_outcomes.every(isReplyOutcome)
  );
}

function isProofMarker(value: unknown): value is ProofFrame["markers"][number] {
  return isObject(value) && typeof value.notation === "string" && isStringArray(value.kinds);
}

function isReplyOutcome(value: unknown): value is ProofFrame["reply_outcomes"][number] {
  return (
    isObject(value) &&
    typeof value.notation === "string" &&
    isStringArray(value.roles) &&
    typeof value.outcome === "string"
  );
}

function isSearchDetails(value: unknown): value is NonNullable<AnalysisEntry["search_details"]> {
  return isObject(value) && hasNumberFields(value, ["search_nodes", "branch_probes", "max_depth_reached"]);
}
