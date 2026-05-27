export interface TournamentRunReport {
  bots: string[];
  schedule: string;
  rules: {
    board_size: number;
    win_length: number;
    variant: string;
  };
  games_per_pair: number;
  seed: number;
  opening_plies: number;
  opening_policy: string;
  threads: number;
  search_time_ms?: number | null;
  search_cpu_time_ms?: number | null;
  search_budget_mode?: string;
  search_cpu_reserve_ms?: number | null;
  search_cpu_max_move_ms?: number | null;
  max_moves?: number | null;
  max_game_ms?: number | null;
  total_wall_time_ms?: number | null;
}

export interface ReportProvenance {
  generated_at_utc?: string | null;
  generated_at_local?: string | null;
  git_commit?: string | null;
  git_dirty?: boolean | null;
  command?: string[];
  host?: {
    os: string;
    arch: string;
    logical_cpus?: number | null;
    cpu_model?: string | null;
    cpu_mhz?: number | null;
  } | null;
}

export interface StandingReport {
  bot: string;
  wins: number;
  draws: number;
  losses: number;
  sequential_elo: number;
  shuffled_elo_avg: number;
  shuffled_elo_stddev: number;
  match_count: number;
  move_count: number;
  search_move_count: number;
  total_time_ms: number;
  avg_search_time_ms: number;
  total_nodes: number;
  avg_nodes: number;
  avg_depth: number;
  max_depth: number;
  avg_effective_depth?: number;
  max_effective_depth?: number;
  avg_child_moves_after?: number;
  avg_child_moves_before?: number;
  budget_exhausted_rate: number;
  pooled_budget_over_base_rate?: number;
  pooled_budget_reserve_exhausted_rate?: number;
  stage_move_gen_ns?: number;
  stage_ordering_ns?: number;
  stage_eval_ns?: number;
  stage_threat_ns?: number;
  stage_proof_ns?: number;
}

export interface PairwiseReport {
  bot_a: string;
  bot_b: string;
  wins_a: number;
  wins_b: number;
  draws: number;
  total: number;
  score_a: number;
  score_b: number;
}

export interface CountReport {
  key: string;
  count: number;
}

export interface PublishedMatchReport {
  match_index: number;
  black: string;
  white: string;
  result: string;
  winner?: string | null;
  end_reason: string;
  move_cells: number[];
  move_count: number;
}

export interface PublishedBotReport {
  schema_version: number;
  report_kind: "published_tournament";
  source_schema_version: number;
  board_size: number;
  move_codec: string;
  provenance?: ReportProvenance;
  run: TournamentRunReport;
  standings: StandingReport[];
  pairwise: PairwiseReport[];
  end_reasons: CountReport[];
  matches: PublishedMatchReport[];
}

const BOT_REPORT_URL = `${import.meta.env.BASE_URL}bot-report/report.json`;

export async function loadPublishedBotReport(): Promise<PublishedBotReport> {
  const response = await fetch(BOT_REPORT_URL, { cache: "no-cache" });
  if (!response.ok) {
    throw new Error(`Failed to load bot report (${response.status})`);
  }

  const data = (await response.json()) as unknown;
  if (!isPublishedBotReport(data)) {
    throw new Error("Published bot report has an unsupported schema.");
  }
  return data;
}

function isPublishedBotReport(data: unknown): data is PublishedBotReport {
  if (!isObject(data)) {
    return false;
  }
  const report = data as Partial<PublishedBotReport>;
  if (
    report.schema_version === 2 &&
    report.report_kind === "published_tournament" &&
    typeof report.source_schema_version === "number" &&
    isPositiveInteger(report.board_size) &&
    report.move_codec === "cell_index_v1" &&
    isTournamentRunReport(report.run) &&
    Array.isArray(report.standings) &&
    report.standings.every(isStandingReport) &&
    Array.isArray(report.pairwise) &&
    report.pairwise.every(isPairwiseReport) &&
    Array.isArray(report.end_reasons) &&
    report.end_reasons.every(isCountReport) &&
    Array.isArray(report.matches)
  ) {
    const boardSize = report.board_size;
    return report.matches.every((match) => isPublishedMatchReport(match, boardSize));
  }
  return false;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

function isNumberArray(value: unknown): value is number[] {
  return Array.isArray(value) && value.every((item) => Number.isInteger(item) && item >= 0);
}

function isPositiveInteger(value: unknown): value is number {
  return Number.isInteger(value) && (value as number) > 0;
}

function hasNumberFields(value: Record<string, unknown>, fields: string[]): boolean {
  return fields.every((field) => typeof value[field] === "number");
}

function isTournamentRunReport(value: unknown): value is TournamentRunReport {
  if (!isObject(value) || !isStringArray(value.bots) || !isObject(value.rules)) {
    return false;
  }
  return (
    isPositiveInteger(value.rules.board_size) &&
    typeof value.rules.variant === "string" &&
    typeof value.schedule === "string" &&
    typeof value.opening_policy === "string" &&
    hasNumberFields(value, ["games_per_pair", "seed", "opening_plies", "threads"])
  );
}

function isStandingReport(value: unknown): value is StandingReport {
  if (!isObject(value) || typeof value.bot !== "string") {
    return false;
  }
  return hasNumberFields(value, [
    "wins",
    "draws",
    "losses",
    "sequential_elo",
    "shuffled_elo_avg",
    "shuffled_elo_stddev",
    "match_count",
    "move_count",
    "search_move_count",
    "total_time_ms",
    "avg_search_time_ms",
    "total_nodes",
    "avg_nodes",
    "avg_depth",
    "max_depth",
    "budget_exhausted_rate",
  ]);
}

function isPairwiseReport(value: unknown): value is PairwiseReport {
  return (
    isObject(value) &&
    typeof value.bot_a === "string" &&
    typeof value.bot_b === "string" &&
    hasNumberFields(value, ["wins_a", "wins_b", "draws", "total", "score_a", "score_b"])
  );
}

function isCountReport(value: unknown): value is CountReport {
  return isObject(value) && typeof value.key === "string" && typeof value.count === "number";
}

function isPublishedMatchReport(value: unknown, boardSize: number): value is PublishedMatchReport {
  if (
    !isObject(value) ||
    typeof value.black !== "string" ||
    typeof value.white !== "string" ||
    typeof value.result !== "string" ||
    typeof value.end_reason !== "string" ||
    !hasNumberFields(value, ["match_index", "move_count"]) ||
    !isNumberArray(value.move_cells)
  ) {
    return false;
  }
  const maxCell = boardSize * boardSize;
  return value.move_cells.every((cell) => cell < maxCell);
}

export function displayBotSpec(spec: string): string {
  return spec
    .replace(/\+corridor-proof(?:-c\d+-d\d+-w\d+)?/g, "+proof")
    .replace(/\+pattern-eval/g, "+pattern")
    .replace(/\+tactical-cap-(\d+)/g, "+tcap$1");
}

export function scorePercent(wins: number, draws: number, total: number): number {
  if (total === 0) {
    return 0;
  }
  return ((wins + draws * 0.5) / total) * 100;
}
