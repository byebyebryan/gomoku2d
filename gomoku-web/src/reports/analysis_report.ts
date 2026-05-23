export interface MovePoint {
  row: number;
  col: number;
}

export interface PublishedAnalysisReport {
  schema_version: number;
  report_kind: "published_analysis";
  source_kind: string;
  source_report: string;
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
    ply: number;
    attacker: string;
    defender: string;
    kind?: string;
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
  if (!data || typeof data !== "object") {
    return false;
  }
  const report = data as Partial<PublishedAnalysisReport>;
  return (
    report.report_kind === "published_analysis" &&
    Array.isArray(report.sections) &&
    typeof report.total === "number"
  );
}
