import { isReportProvenance, type ReportProvenance } from "./bot_report";
import type { ReplayAnalysisFailure } from "../replay/replay_analysis_protocol";
import { validateReplayAnalysisFailure } from "../replay/replay_analysis_validation";

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
  failure?: (Omit<ReplayAnalysisFailure, "confidence" | "missed_candidates"> & {
    // Older published projections can omit these fields.
    confidence?: ReplayAnalysisFailure["confidence"];
    missed_candidates?: ReplayAnalysisFailure["missed_candidates"];
  }) | null;
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
    isReportProvenance(report.provenance) &&
    (report.source_report_provenance == null || isReportProvenance(report.source_report_provenance)) &&
    typeof report.source_kind === "string" &&
    typeof report.source_report === "string" &&
    typeof report.selector === "string" &&
    hasNumberFields(data, ["analyzed", "failed", "elapsed_ms", "total_elapsed_ms", "total"]) &&
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
  return Array.isArray(value) && value.length <= 225
    && value.every((item) => Number.isInteger(item) && item >= 0 && item < 225)
    && new Set(value).size === value.length;
}

function hasNumberFields(value: Record<string, unknown>, fields: string[]): boolean {
  return fields.every((field) => typeof value[field] === "number" && Number.isFinite(value[field]));
}

function isAnalysisModel(value: unknown): value is PublishedAnalysisReport["model"] {
  return isObject(value) && hasNumberFields(value, ["max_depth"])
    && nullable(value.max_scan_plies, isPly);
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
    hasNumberFields(value, ["elapsed_ms"]) &&
    nullable(value.root_cause, isString) &&
    nullable(value.unclear_reason, isString) &&
    nullable(value.error, isString) &&
    nullable(value.last_chance_ply, isPly) &&
    nullable(value.critical_loser_ply, isPly) &&
    nullable(value.lethal_onset, isLethalOnset) &&
    nullable(value.setup_corridor, isInterval) &&
    nullable(value.failure, isFailure) &&
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
    isMoveCellArray(value.move_cells) &&
    value.move_count === value.move_cells.length &&
    nullable(value.winner, isString)
  );
}

function isProofDetails(value: unknown): value is NonNullable<AnalysisEntry["proof_details"]> {
  return isObject(value) && Array.isArray(value.proof_frames) && value.proof_frames.every(isProofFrame);
}

function isProofFrame(value: unknown): value is ProofFrame {
  return (
    isObject(value) &&
    typeof value.label === "string" &&
    isPly(value.ply) &&
    typeof value.side_to_move === "string" &&
    typeof value.status === "string" &&
    nullable(value.move_played_notation, isString) &&
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

function isString(value: unknown): value is string { return typeof value === "string"; }
function isPly(value: unknown): value is number {
  return Number.isInteger(value) && (value as number) >= 0 && (value as number) <= 225;
}
function nullable(value: unknown, validate: (value: unknown) => boolean): boolean {
  return value == null || validate(value);
}
function isMove(value: unknown): boolean {
  return isObject(value) && Number.isInteger(value.row) && Number.isInteger(value.col)
    && (value.row as number) >= 0 && (value.row as number) < 15
    && (value.col as number) >= 0 && (value.col as number) < 15;
}
function isInterval(value: unknown): boolean {
  return isObject(value) && isPly(value.start_ply) && isPly(value.end_ply) && value.end_ply >= value.start_ply;
}
function isLethalOnset(value: unknown): boolean {
  if (!isObject(value)) return false;
  return isPly(value.prefix_ply) && isString(value.attacker) && isString(value.defender)
    && nullable(value.kind, isString)
    && nullable(value.terminal_targets, (items) => Array.isArray(items) && items.every(isMove))
    && nullable(value.shape, (shape) => isObject(shape)
      && nullable(shape.label, isString) && nullable(shape.mechanisms, isStringArray));
}
function isFailure(value: unknown): boolean {
  if (!isObject(value)) return false;
  try {
    validateReplayAnalysisFailure({
      ...value,
      confidence: value.confidence ?? "unclear",
      missed_candidates: value.missed_candidates ?? [],
    });
    return true;
  } catch {
    return false;
  }
}
