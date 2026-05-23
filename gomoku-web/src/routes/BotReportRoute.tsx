import {
  Fragment,
  useEffect,
  useState,
  type CSSProperties,
  type KeyboardEvent,
} from "react";

import {
  displayBotSpec,
  loadPublishedBotReport,
  scorePercent,
  type PairwiseReport,
  type PublishedBotReport,
  type PublishedMatchReport,
  type StandingReport,
} from "../reports/bot_report";

import styles from "./ReportRoute.module.css";

const baseUrl = import.meta.env.BASE_URL;

type LoadState =
  | { status: "loading" }
  | { status: "loaded"; report: PublishedBotReport }
  | { status: "error"; message: string };

type ReportView = "ranking" | "search" | "pairwise";

const REPORT_VIEWS: Array<{ id: ReportView; label: string }> = [
  { id: "ranking", label: "Ranking" },
  { id: "search", label: "Search" },
  { id: "pairwise", label: "Games" },
];

export function BotReportRoute() {
  const [state, setState] = useState<LoadState>({ status: "loading" });

  useEffect(() => {
    document.title = "Gomoku2D Bot Lab Report";
    let cancelled = false;
    loadPublishedBotReport()
      .then((report) => {
        if (!cancelled) {
          setState({ status: "loaded", report });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setState({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          });
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  if (state.status === "loading") {
    return <ReportState title="Bot Lab Report" message="Loading report…" />;
  }
  if (state.status === "error") {
    return <ReportState title="Bot Lab Report" message={state.message} />;
  }

  return <BotReportPage report={state.report} />;
}

function BotReportPage({ report }: { report: PublishedBotReport }) {
  const [view, setView] = useState<ReportView>("ranking");
  const [openBots, setOpenBots] = useState<Set<string>>(() => new Set());

  const toggleBot = (bot: string) => {
    setOpenBots((current) => toggleSetValue(current, bot));
  };

  return (
    <main className={styles.page}>
      <div className={styles.shell}>
        <header className={styles.hero}>
          <div className={styles.headerRow}>
            <div>
              <p className="uiPageEyebrow">Gomoku2D lab</p>
              <h1 className={styles.title}>Bot Lab Report</h1>
            </div>
            <nav className={styles.links} aria-label="Report links">
              <a href={baseUrl}>Game</a>
              <a href={`${baseUrl}assets/`}>Assets</a>
              <a href={`${baseUrl}analysis-report/`}>Analysis</a>
            </nav>
          </div>
          <div className={styles.chips} aria-label="Run summary">
            <ReportChip label="Schedule" value={scheduleSummary(report)} />
            <ReportChip label="Rule" value={report.run.rules.variant} />
            <ReportChip label="Opening" value={openingSummary(report)} />
            <ReportChip label="Budget" value={budgetLabel(report)} />
            <ReportChip label="Wall" value={formatDurationMs(report.run.total_wall_time_ms)} />
            <ReportChip label="Finish" value={finishSummary(report)} />
          </div>
          {report.provenance?.git_dirty ? (
            <p className={styles.warning}>Development run: generated from a dirty git worktree.</p>
          ) : null}
        </header>

        <section className={`${styles.panel} ${styles.entrantWorkbench}`} data-view={view}>
          <div className={styles.headerRow}>
            <h2>Results</h2>
            <div className={styles.viewToggle} aria-label="Entrant table mode">
              {REPORT_VIEWS.map((option) => (
                <button
                  key={option.id}
                  type="button"
                  className={view === option.id ? styles.activeToggle : undefined}
                  onClick={() => setView(option.id)}
                >
                  {option.label}
                </button>
              ))}
            </div>
          </div>

          <div className={styles.entrantGrid}>
            <EntrantHeader />
            {report.standings.map((standing, index) => {
              const isOpen = openBots.has(standing.bot);
              return (
                <EntrantRow
                  key={standing.bot}
                  report={report}
                  standing={standing}
                  rank={index + 1}
                  view={view}
                  isOpen={isOpen}
                  onToggle={() => toggleBot(standing.bot)}
                />
              );
            })}
          </div>
        </section>

        <HowToReadSection />
        <ProvenanceSection report={report} />
      </div>
    </main>
  );
}

function EntrantHeader() {
  return (
    <div className={styles.entrantHead}>
      <span>Spec</span>
      {["Rank", "Score %", "W-D-L", "Shuffled Elo", "Depth", "Width", "Avg ms", "Budget hit"].map(
        (head) => (
          <span
            key={`result-${head}`}
            className={`${styles.metric} ${styles.metricResults} ${
              head === "W-D-L" ? styles.metricNowrap : ""
            }`}
          >
            {head}
          </span>
        ),
      )}
      {["Nodes", "Move gen", "Ordering", "Scoring", "Threat detection", "Proof", "Other"].map(
        (head) => (
          <span key={`search-${head}`} className={`${styles.metric} ${styles.metricSearch}`}>
            {head}
          </span>
        ),
      )}
      {["Score %", "W-D-L", "Shuffled Elo"].map((head) => (
        <span
          key={`pairwise-${head}`}
          className={`${styles.metric} ${styles.metricPairwise} ${
            head === "W-D-L" ? styles.metricNowrap : ""
          }`}
        >
          {head}
        </span>
      ))}
    </div>
  );
}

function EntrantRow({
  report,
  standing,
  rank,
  view,
  isOpen,
  onToggle,
}: {
  report: PublishedBotReport;
  standing: StandingReport;
  rank: number;
  view: ReportView;
  isOpen: boolean;
  onToggle: () => void;
}) {
  const score = scorePercent(standing.wins, standing.draws, standing.match_count);
  const pairwiseEntries = rankedPairsForBot(report, standing.bot);
  const canExpand = view !== "search";
  const expanded = canExpand && isOpen;

  return (
    <details className={styles.entrantRow} open={expanded}>
      <summary
        aria-expanded={expanded}
        className={canExpand ? undefined : styles.staticSummary}
        onClick={(event) => {
          event.preventDefault();
          if (canExpand) {
            onToggle();
          }
        }}
        onKeyDown={(event) => {
          if (canExpand) {
            handleToggleKey(event, onToggle);
          } else if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
          }
        }}
      >
        <BotLabel bot={standing.bot} />
        <MetricCell kind="results" label="Rank" primary={`#${rank}`} />
        <MetricCell kind="results" label="Score %" primary={formatPercent(score)} />
        <MetricCell
          kind="results"
          label="W-D-L"
          primary={`${standing.wins}-${standing.draws}-${standing.losses}`}
          nowrap
        />
        <MetricCell
          kind="results"
          label="Shuffled Elo"
          primary={formatNumber(standing.shuffled_elo_avg)}
          secondary={`+/- ${formatNumber(standing.shuffled_elo_stddev)}`}
        />
        <MetricCell
          kind="results"
          label="Depth"
          primary={formatNumber(standing.avg_depth)}
          secondary={
            (standing.avg_effective_depth ?? 0) > standing.avg_depth
              ? `eff ${formatNumber(standing.avg_effective_depth ?? 0)}`
              : undefined
          }
        />
        <MetricCell
          kind="results"
          label="Width"
          primary={widthPrimary(standing)}
          secondary={widthSecondary(standing)}
        />
        <MetricCell kind="results" label="Avg ms" primary={formatNumber(standing.avg_search_time_ms)} />
        <MetricCell
          kind="results"
          label="Budget hit"
          primary={formatPercent((standing.budget_exhausted_rate ?? 0) * 100)}
        />
        <MetricCell kind="search" label="Nodes" primary={formatCompact(standing.avg_nodes)} />
        <StageMetricCell label="Move gen" stageNs={standing.stage_move_gen_ns ?? 0} standing={standing} />
        <StageMetricCell label="Ordering" stageNs={standing.stage_ordering_ns ?? 0} standing={standing} />
        <StageMetricCell label="Scoring" stageNs={standing.stage_eval_ns ?? 0} standing={standing} />
        <StageMetricCell
          label="Threat detection"
          stageNs={standing.stage_threat_ns ?? 0}
          standing={standing}
        />
        <StageMetricCell label="Proof" stageNs={standing.stage_proof_ns ?? 0} standing={standing} />
        <StageMetricCell label="Other" stageNs={stageOtherNs(standing)} standing={standing} />
        <MetricCell kind="pairwise" label="Score %" primary={formatPercent(score)} />
        <MetricCell
          kind="pairwise"
          label="W-D-L"
          primary={`${standing.wins}-${standing.draws}-${standing.losses}`}
          nowrap
        />
        <MetricCell
          kind="pairwise"
          label="Shuffled Elo"
          primary={formatNumber(standing.shuffled_elo_avg)}
          secondary={`+/- ${formatNumber(standing.shuffled_elo_stddev)}`}
        />
      </summary>
      {view === "ranking" && expanded ? (
        <ResultComparisons bot={standing.bot} pairs={pairwiseEntries} />
      ) : null}
      {view === "pairwise" && expanded ? (
        <EntrantPairwise report={report} bot={standing.bot} pairs={pairwiseEntries} />
      ) : null}
    </details>
  );
}

function BotLabel({ bot, prefix = "" }: { bot: string; prefix?: string }) {
  const label = displayBotSpec(bot);
  const [primary, ...rest] = label.split("+");
  return (
    <strong className={styles.botLabel}>
      <span>{prefix}{primary}</span>
      {rest.length > 0 ? <span>{rest.join("+")}</span> : null}
    </strong>
  );
}

function MetricCell({
  kind,
  label,
  primary,
  secondary,
  nowrap,
}: {
  kind: "results" | "search" | "pairwise";
  label: string;
  primary: string;
  secondary?: string;
  nowrap?: boolean;
}) {
  const kindClass =
    kind === "results"
      ? styles.metricResults
      : kind === "search"
        ? styles.metricSearch
        : styles.metricPairwise;
  return (
    <span
      className={`${styles.metric} ${kindClass} ${nowrap ? styles.metricNowrap : ""}`}
      data-label={label}
    >
      <span>{primary}</span>
      {secondary ? <span>{secondary}</span> : null}
    </span>
  );
}

function StageMetricCell({
  label,
  stageNs,
  standing,
}: {
  label: string;
  stageNs: number;
  standing: StandingReport;
}) {
  const pct = stageSharePercent(stageNs, standing);
  return (
    <MetricCell
      kind="search"
      label={label}
      primary={formatPercent(pct)}
      secondary={stageAvgMsLabel(stageNs, standing.search_move_count)}
    />
  );
}

function ResultComparisons({ bot, pairs }: { bot: string; pairs: PairwiseReport[] }) {
  return (
    <div className={styles.entrantResultComparisons}>
      {pairs.map((pair) => {
        const opponent = opponentForPair(pair, bot);
        const score = pairScoreForBot(pair, bot);
        return (
          <div className={styles.comparisonRow} key={pairKey(pair)}>
            <span className={styles.comparisonOpponent} data-label="Opponent">
              <BotLabel bot={opponent} prefix="Vs " />
            </span>
            <span
              className={`${styles.comparisonValue} ${scoreToneClass(score)}`}
              data-label="Score"
            >
              {formatPercent(score)}
            </span>
            <span className={styles.comparisonValue} data-label="Record">{pairRecordForBot(pair, bot)} W-D-L</span>
            <span className={styles.comparisonValue} data-label="Points">{pairPointsForBot(pair, bot)} points</span>
          </div>
        );
      })}
    </div>
  );
}

function EntrantPairwise({
  report,
  bot,
  pairs,
}: {
  report: PublishedBotReport;
  bot: string;
  pairs: PairwiseReport[];
}) {
  return (
    <div className={styles.entrantPairs}>
      {pairs.map((pair) => (
        <OpponentDetails key={pairKey(pair)} report={report} bot={bot} pair={pair} />
      ))}
    </div>
  );
}

function OpponentDetails({
  report,
  bot,
  pair,
}: {
  report: PublishedBotReport;
  bot: string;
  pair: PairwiseReport;
}) {
  const [isOpen, setIsOpen] = useState(false);
  const opponent = opponentForPair(pair, bot);
  const score = pairScoreForBot(pair, bot);
  const matches = isOpen ? report.matches.filter((match) => matchIsPair(match, pair)) : [];

  return (
    <details className={styles.opponentRow} open={isOpen}>
      <summary
        aria-expanded={isOpen}
        onClick={(event) => {
          event.preventDefault();
          setIsOpen((current) => !current);
        }}
        onKeyDown={(event) => handleToggleKey(event, () => setIsOpen((current) => !current))}
      >
        <BotLabel bot={opponent} prefix="Vs " />
        <span className={scoreToneClass(score)} data-label="Score">{formatPercent(score)}</span>
        <span data-label="W-D-L">{pairRecordForBot(pair, bot)} W-D-L</span>
        <span data-label="Points">{pairPointsForBot(pair, bot)} points</span>
      </summary>
      {isOpen ? (
        <div className={styles.matchList}>
          {matches.map((match) => (
            <MatchDetails
              key={`${bot}-${match.match_index}`}
              report={report}
              bot={bot}
              match={match}
            />
          ))}
        </div>
      ) : null}
    </details>
  );
}

function MatchDetails({
  report,
  bot,
  match,
}: {
  report: PublishedBotReport;
  bot: string;
  match: PublishedMatchReport;
}) {
  const [isOpen, setIsOpen] = useState(false);
  const botSide = match.black === bot ? "Black" : "White";
  const opponentSide = match.black === bot ? "White" : "Black";
  const result = matchResultForBot(match, bot);

  return (
    <details className={styles.match} open={isOpen}>
      <summary
        aria-expanded={isOpen}
        onClick={(event) => {
          event.preventDefault();
          setIsOpen((current) => !current);
        }}
        onKeyDown={(event) => handleToggleKey(event, () => setIsOpen((current) => !current))}
      >
        <span className={styles.matchSides} data-label="Game">
          <span>{botSide}</span>
          <span>{opponentSide}</span>
        </span>
        <span className={resultToneClass(result)} data-label="Result">{result}</span>
        <span data-label="Moves">{match.move_count} moves</span>
        <span data-label="End">{matchEndLabel(match)}</span>
      </summary>
      {isOpen ? (
        <div className={styles.matchGrid}>
          {match.move_cells.length > 0 ? (
            <div className={styles.boardPanel}>
              <FinishedBoard moveCells={match.move_cells} boardSize={report.board_size} />
            </div>
          ) : (
            <p className={styles.muted}>Move cells were not captured for this match.</p>
          )}
        </div>
      ) : null}
    </details>
  );
}

function FinishedBoard({
  moveCells,
  boardSize,
}: {
  moveCells: number[];
  boardSize: number;
}) {
  const stones = new Map<number, "black" | "white">();
  moveCells.forEach((cell, index) => {
    stones.set(cell, index % 2 === 0 ? "black" : "white");
  });
  const lastCell = moveCells.length > 0 ? moveCells[moveCells.length - 1] : undefined;
  const gridSpan = Math.max(0, boardSize - 1) * 20 + 1;
  const style = {
    "--proof-grid-span": `${gridSpan}px`,
    gridTemplateColumns: `repeat(${boardSize}, var(--proof-cell-size))`,
    gridTemplateRows: `repeat(${boardSize}, var(--proof-cell-size))`,
  } as CSSProperties;

  return (
    <div className={styles.proofBoard} style={style}>
      {Array.from({ length: boardSize * boardSize }, (_, cell) => {
        const stone = stones.get(cell);
        const isLast = cell === lastCell;
        return (
          <div className={styles.proofCell} key={cell} data-move={cellNotation(cell, boardSize)}>
            {stone ? (
              <span
                className={`${styles.proofStone} ${
                  stone === "black" ? styles.proofStoneBlack : styles.proofStoneWhite
                }`}
              />
            ) : null}
            {stone && isLast ? (
              <span
                className={`${styles.proofActualStone} ${
                  stone === "black" ? styles.proofActualStoneBlack : styles.proofActualStoneWhite
                }`}
              />
            ) : null}
          </div>
        );
      })}
    </div>
  );
}

function HowToReadSection() {
  return (
    <section className={`${styles.panel} ${styles.howToRead}`}>
      <h2>How To Read This</h2>
      <dl className={styles.termList}>
        <TermRow
          title="Run Shape"
          body="Schedule shows the pairing count, games per pair, and total matches. Opening shows the seeded legal moves before bots take over."
        />
        <TermRow
          title="Elo"
          body="Relative rating within this report only. Shuffled Elo averages repeated Elo passes over randomized match order to reduce run-order noise."
        />
        <TermRow
          title="Score"
          body="Score % counts wins plus half draws. W-D-L is wins, draws, then losses. Comparisons above 50% are marked green."
        />
        <TermRow
          title="Budget Hit"
          body="Share of searched moves that hit the active CPU cap before search finished naturally."
        />
        <TermRow
          title="Search Cost"
          body="Width is the average number of moves searched. The Search tab splits measured time into move generation, ordering, scoring, threat detection, corridor proof, and uncategorized search overhead."
        />
      </dl>
    </section>
  );
}

function TermRow({ title, body }: { title: string; body: string }) {
  return (
    <Fragment>
      <dt>{title}</dt>
      <dd>{body}</dd>
    </Fragment>
  );
}

function ProvenanceSection({ report }: { report: PublishedBotReport }) {
  return (
    <section className={`${styles.panel} ${styles.provenance}`}>
      <h2>Provenance</h2>
      <dl>
        <dt>Generated local</dt>
        <dd>{report.provenance?.generated_at_local ?? "unknown"}</dd>
        <dt>Generated UTC</dt>
        <dd>{report.provenance?.generated_at_utc ?? "unknown"}</dd>
        <dt>Wall clock</dt>
        <dd>{formatDurationMs(report.run.total_wall_time_ms)}</dd>
        <dt>Git revision</dt>
        <dd>{revisionLabel(report)}</dd>
        <dt>Schema</dt>
        <dd>v{report.schema_version} / {report.move_codec}</dd>
      </dl>
    </section>
  );
}

function ReportChip({ label, value }: { label: string; value: string }) {
  return (
    <span className={styles.chip}>
      <span>{label}</span>
      <strong>{value}</strong>
    </span>
  );
}

function ReportState({ title, message }: { title: string; message: string }) {
  return (
    <main className={styles.page}>
      <section className={styles.state}>
        <h1>{title}</h1>
        <p>{message}</p>
      </section>
    </main>
  );
}

function scheduleSummary(report: PublishedBotReport): string {
  const pairs = report.pairwise.length;
  const pairWord = pairs === 1 ? "pair" : "pairs";
  return `${pairs} ${pairWord} x ${report.run.games_per_pair} games = ${report.matches.length} matches`;
}

function openingSummary(report: PublishedBotReport): string {
  return `${report.run.opening_policy}, seed ${report.run.seed}, ${report.run.opening_plies} plies`;
}

function budgetLabel(report: PublishedBotReport): string {
  const base = report.run.search_cpu_time_ms
    ? `CPU ${report.run.search_cpu_time_ms} ms/move`
    : report.run.search_time_ms
      ? `Wall ${report.run.search_time_ms} ms/move`
      : "no per-move budget";
  if (report.run.search_budget_mode === "pooled") {
    const reserve = report.run.search_cpu_reserve_ms;
    const maxMove = report.run.search_cpu_max_move_ms;
    if (reserve && maxMove) {
      return `${base}, reserve ${reserve} ms, max ${maxMove} ms`;
    }
    if (reserve) {
      return `${base}, reserve ${reserve} ms`;
    }
  }
  return base;
}

function finishSummary(report: PublishedBotReport): string {
  const natural = countEndReason(report, "natural");
  const maxMoves = countEndReason(report, "max_moves");
  const parts = [];
  if (natural > 0) {
    parts.push(`${natural} finished`);
  }
  if (maxMoves > 0) {
    parts.push(`${maxMoves} max moves`);
  }
  return parts.length > 0 ? parts.join(" / ") : "none";
}

function countEndReason(report: PublishedBotReport, key: string): number {
  return report.end_reasons.find((reason) => reason.key === key)?.count ?? 0;
}

function revisionLabel(report: PublishedBotReport): string {
  const commit = report.provenance?.git_commit ?? "unknown";
  return report.provenance?.git_dirty ? `${commit} dirty` : commit;
}

function widthPrimary(standing: StandingReport): string {
  if ((standing.avg_child_moves_after ?? 0) > 0) {
    return formatNumber(standing.avg_child_moves_after ?? 0);
  }
  return formatNumber(standing.avg_child_moves_before ?? 0);
}

function widthSecondary(standing: StandingReport): string | undefined {
  if ((standing.avg_child_moves_after ?? 0) > 0 && (standing.avg_child_moves_before ?? 0) > 0) {
    return `pre ${formatNumber(standing.avg_child_moves_before ?? 0)}`;
  }
  return undefined;
}

function stageKnownNs(standing: StandingReport): number {
  return (
    (standing.stage_move_gen_ns ?? 0) +
    (standing.stage_ordering_ns ?? 0) +
    (standing.stage_eval_ns ?? 0) +
    (standing.stage_threat_ns ?? 0) +
    (standing.stage_proof_ns ?? 0)
  );
}

function stageDenominatorNs(standing: StandingReport): number {
  return Math.max(standing.total_time_ms * 1_000_000, stageKnownNs(standing));
}

function stageOtherNs(standing: StandingReport): number {
  return Math.max(0, stageDenominatorNs(standing) - stageKnownNs(standing));
}

function stageSharePercent(stageNs: number, standing: StandingReport): number {
  const denominator = stageDenominatorNs(standing);
  return denominator === 0 ? 0 : (stageNs * 100) / denominator;
}

function stageAvgMsLabel(stageNs: number, searchMoveCount: number): string {
  const avgMs = searchMoveCount === 0 ? 0 : stageNs / 1_000_000 / searchMoveCount;
  return avgMs < 0.05 ? "0 ms" : `${formatNumber(avgMs)} ms`;
}

function rankedPairsForBot(report: PublishedBotReport, bot: string): PairwiseReport[] {
  const ranking = new Map(report.standings.map((standing, index) => [standing.bot, index]));
  return report.pairwise
    .filter((pair) => pair.bot_a === bot || pair.bot_b === bot)
    .sort((a, b) => {
      const rankA = ranking.get(opponentForPair(a, bot)) ?? Number.MAX_SAFE_INTEGER;
      const rankB = ranking.get(opponentForPair(b, bot)) ?? Number.MAX_SAFE_INTEGER;
      return rankA - rankB;
    });
}

function pairKey(pair: PairwiseReport): string {
  return `${pair.bot_a}|${pair.bot_b}`;
}

function opponentForPair(pair: PairwiseReport, bot: string): string {
  return pair.bot_a === bot ? pair.bot_b : pair.bot_a;
}

function pairScoreForBot(pair: PairwiseReport, bot: string): number {
  const score = pair.bot_a === bot ? pair.score_a : pair.score_b;
  return pair.total === 0 ? 0 : (score / pair.total) * 100;
}

function pairRecordForBot(pair: PairwiseReport, bot: string): string {
  if (pair.bot_a === bot) {
    return `${pair.wins_a}-${pair.draws}-${pair.wins_b}`;
  }
  return `${pair.wins_b}-${pair.draws}-${pair.wins_a}`;
}

function pairPointsForBot(pair: PairwiseReport, bot: string): string {
  if (pair.bot_a === bot) {
    return `${formatNumber(pair.score_a)}-${formatNumber(pair.score_b)}`;
  }
  return `${formatNumber(pair.score_b)}-${formatNumber(pair.score_a)}`;
}

function matchIsPair(match: PublishedMatchReport, pair: PairwiseReport): boolean {
  return (
    (match.black === pair.bot_a && match.white === pair.bot_b) ||
    (match.black === pair.bot_b && match.white === pair.bot_a)
  );
}

function matchResultForBot(match: PublishedMatchReport, bot: string): string {
  if (!match.winner) {
    return "draw";
  }
  return match.winner === bot ? "win" : "lose";
}

function scoreToneClass(score: number): string {
  if (score > 50) {
    return styles.scoreGood;
  }
  if (score < 50) {
    return styles.scoreBad;
  }
  return "";
}

function resultToneClass(result: string): string {
  if (result === "win") {
    return styles.scoreGood;
  }
  if (result === "lose") {
    return styles.scoreBad;
  }
  return "";
}

function matchEndLabel(match: PublishedMatchReport): string {
  if (match.end_reason === "max_moves") {
    return "max moves";
  }
  if (match.end_reason === "natural") {
    return "finished";
  }
  return match.end_reason;
}

function cellNotation(cell: number, boardSize: number): string {
  const row = Math.floor(cell / boardSize);
  const col = cell % boardSize;
  return `${String.fromCharCode("A".charCodeAt(0) + col)}${row + 1}`;
}

function formatNumber(value: number): string {
  return value.toFixed(1);
}

function formatDurationMs(value: number | null | undefined): string {
  const ms = value ?? 0;
  if (ms < 1_000) {
    return `${ms} ms`;
  }
  if (ms < 60_000) {
    return `${(ms / 1_000).toFixed(2)} s`;
  }
  const minutes = Math.floor(ms / 60_000);
  const seconds = ((ms % 60_000) / 1_000).toFixed(1);
  return `${minutes}m ${seconds}s`;
}

function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}

function formatCompact(value: number): string {
  if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(1)}m`;
  }
  if (value >= 1_000) {
    return `${(value / 1_000).toFixed(1)}k`;
  }
  return value.toFixed(0);
}

function toggleSetValue<T>(current: Set<T>, value: T): Set<T> {
  const next = new Set(current);
  if (next.has(value)) {
    next.delete(value);
  } else {
    next.add(value);
  }
  return next;
}

function handleToggleKey(event: KeyboardEvent, onToggle: () => void) {
  if (event.key !== "Enter" && event.key !== " ") {
    return;
  }
  event.preventDefault();
  onToggle();
}
