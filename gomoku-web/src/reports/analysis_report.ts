export interface MovePoint {
  row: number;
  col: number;
}

export interface AnalysisBatchReport {
  schema_version: number;
  source_kind: string;
  source: string;
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
  entries: AnalysisEntry[];
}

export interface AnalysisEntry {
  path: string;
  status: string;
  winner?: string | null;
  move_count?: number | null;
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
    principal_line_notation?: string[];
  }>;
}

const ANALYSIS_REPORT_URL = `${import.meta.env.BASE_URL}analysis-report/report.json`;

export async function loadAnalysisReport(): Promise<AnalysisBatchReport> {
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

function isAnalysisReport(data: unknown): data is AnalysisBatchReport {
  if (!data || typeof data !== "object") {
    return false;
  }
  const report = data as Partial<AnalysisBatchReport>;
  return Array.isArray(report.entries) && typeof report.total === "number";
}

export function splitAnalysisSource(source: string): {
  sourcePath: string;
  selector: string;
} {
  const [sourcePath, selector] = source.split(":", 2);
  return {
    sourcePath: sourcePath || source,
    selector: selector || "Top 2 entrants",
  };
}

