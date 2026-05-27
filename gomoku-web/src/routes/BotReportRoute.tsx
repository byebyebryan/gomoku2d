import {
  Fragment,
  useEffect,
  useState,
  type CSSProperties,
  type KeyboardEvent,
  type ReactNode,
} from "react";
import { useSearchParams } from "react-router-dom";

import { presetForLabSpec, type BotPresetId } from "../core/bot_config";
import {
  displayBotSpec,
  loadPublishedBotReport,
  scorePercent,
  type PairwiseReport,
  type PublishedBotReport,
  type PublishedMatchReport,
  type StandingReport,
} from "../reports/bot_report";
import {
  loadAnalysisReport,
  type PublishedAnalysisReport,
} from "../reports/analysis_report";
import { AnalysisReportContent } from "./AnalysisReportRoute";

import styles from "./ReportRoute.module.css";

const baseUrl = import.meta.env.BASE_URL;

type LoadState<T> =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "loaded"; report: T }
  | { status: "error"; message: string };

type ReportView = "ranking" | "search" | "analysis";
type BotReportView = Exclude<ReportView, "analysis">;

const REPORT_VIEWS: Array<{ id: ReportView; label: string }> = [
  { id: "ranking", label: "Ranking" },
  { id: "search", label: "Search" },
  { id: "analysis", label: "Analysis" },
];

export function LabReportRoute() {
  const [searchParams, setSearchParams] = useSearchParams();
  const view = parseReportView(searchParams.get("tab"));
  const [analysisState, setAnalysisState] = useState<LoadState<PublishedAnalysisReport>>({
    status: "idle",
  });
  const [botState, setBotState] = useState<LoadState<PublishedBotReport>>({ status: "idle" });

  useEffect(() => {
    document.title = "Gomoku2D Lab Report";
  }, []);

  useEffect(() => {
    if (view === "analysis" || botState.status !== "idle") {
      return;
    }

    let cancelled = false;
    setBotState({ status: "loading" });
    loadPublishedBotReport()
      .then((report) => {
        if (!cancelled) {
          setBotState({ status: "loaded", report });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setBotState({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          });
        }
      });
    return () => {
      cancelled = true;
    };
  }, [view]);

  useEffect(() => {
    if (view !== "analysis" || analysisState.status !== "idle") {
      return;
    }

    let cancelled = false;
    setAnalysisState({ status: "loading" });
    loadAnalysisReport()
      .then((report) => {
        if (!cancelled) {
          setAnalysisState({ status: "loaded", report });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setAnalysisState({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          });
        }
      });
    return () => {
      cancelled = true;
    };
  }, [view]);

  const setView = (nextView: ReportView) => {
    setSearchParams(nextView === "ranking" ? {} : { tab: nextView });
  };

  let content: ReactNode;

  if (view === "analysis") {
    if (analysisState.status === "error") {
      content = <ReportStatePanel title="Lab Report" message={analysisState.message} />;
    } else if (analysisState.status !== "loaded") {
      content = <ReportStatePanel title="Lab Report" message="Loading report…" />;
    } else {
      content = (
        <>
          <AnalysisReportContent report={analysisState.report} />
          <AnalysisHowToReadSection />
          <AnalysisProvenanceSection report={analysisState.report} />
        </>
      );
    }
  } else if (botState.status === "error") {
    content = <ReportStatePanel title="Lab Report" message={botState.message} />;
  } else if (botState.status !== "loaded") {
    content = <ReportStatePanel title="Lab Report" message="Loading report…" />;
  } else {
    content = (
      <>
        <BotReportPanel report={botState.report} view={view} />
        <BotHowToReadSection view={view} />
        <BotProvenanceSection report={botState.report} view={view} />
      </>
    );
  }

  return (
    <main className={styles.page}>
      <div className={styles.shell}>
        <header className={styles.hero}>
          <div className={styles.headerRow}>
            <div className={styles.headerCopy}>
              <p className="uiPageEyebrow">Gomoku2D lab</p>
              <h1 className={styles.title}>Lab Report</h1>
              <p className={styles.subtitle}>
                Inner workings of a Gomoku bot: rankings, search telemetry, and replay
                analysis.
              </p>
            </div>
            <nav className={styles.links} aria-label="Report links">
              <a className="uiAction uiActionNeutral" href={baseUrl}>
                <span className="uiActionLabel">Home</span>
              </a>
              <a className="uiAction uiActionNeutral" href={`${baseUrl}visuals/`}>
                <span className="uiActionLabel">Visuals</span>
              </a>
            </nav>
          </div>
          <ReportTabs value={view} onChange={setView} />
          {botState.status === "loaded" && botState.report.provenance?.git_dirty ? (
            <p className={styles.warning}>Development run: generated from a dirty git worktree.</p>
          ) : null}
        </header>

        {content}
      </div>
    </main>
  );
}

function ReportTabs({
  value,
  onChange,
}: {
  value: ReportView;
  onChange: (view: ReportView) => void;
}) {
  return (
    <div className={styles.viewToggle} aria-label="Lab report sections" role="tablist">
      {REPORT_VIEWS.map((option) => (
        <button
          key={option.id}
          type="button"
          id={`lab-report-tab-${option.id}`}
          aria-controls={`lab-report-panel-${option.id}`}
          aria-selected={value === option.id}
          className={value === option.id ? styles.activeToggle : undefined}
          onClick={() => onChange(option.id)}
          role="tab"
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

function parseReportView(value: string | null): ReportView {
  return value === "search" || value === "analysis" ? value : "ranking";
}

function BotReportPanel({
  report,
  view,
}: {
  report: PublishedBotReport;
  view: BotReportView;
}) {
  const [openBots, setOpenBots] = useState<Set<string>>(() => new Set());
  const maxSearchAvgNs = maxSearchAverageNs(report.standings);

  const toggleBot = (bot: string) => {
    setOpenBots((current) => toggleSetValue(current, bot));
  };

  return (
    <section
      aria-labelledby={`lab-report-tab-${view}`}
      className={`${styles.panel} ${styles.entrantWorkbench}`}
      data-view={view}
      id={`lab-report-panel-${view}`}
      role="tabpanel"
    >
      <div className={styles.headerRow}>
        <div>
          <h2>Results</h2>
          <p className={styles.reportNote}>{botReportIntro(view)}</p>
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
              maxSearchAvgNs={maxSearchAvgNs}
              onToggle={() => toggleBot(standing.bot)}
            />
          );
        })}
      </div>
    </section>
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
    </div>
  );
}

function EntrantRow({
  report,
  standing,
  rank,
  view,
  isOpen,
  maxSearchAvgNs,
  onToggle,
}: {
  report: PublishedBotReport;
  standing: StandingReport;
  rank: number;
  view: BotReportView;
  isOpen: boolean;
  maxSearchAvgNs: number;
  onToggle: () => void;
}) {
  const score = scorePercent(standing.wins, standing.draws, standing.match_count);
  const pairwiseEntries = rankedPairsForBot(report, standing.bot);
  const canExpand = view === "ranking";
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
        <SearchTimeSplit maxSearchAvgNs={maxSearchAvgNs} standing={standing} />
      </summary>
      {view === "ranking" && expanded ? (
        <RankingDrilldown report={report} bot={standing.bot} pairs={pairwiseEntries} />
      ) : null}
    </details>
  );
}

function BotLabel({
  badgePlacement = "after",
  bot,
}: {
  badgePlacement?: "after" | "before";
  bot: string;
}) {
  const label = displayBotSpec(bot);
  const [primary, ...rest] = label.split("+");
  const preset = presetForLabSpec(bot);
  return (
    <strong className={styles.botLabel}>
      <span className={styles.botLabelPrimary}>
        {preset && badgePlacement === "before" ? <PresetBadge preset={preset} /> : null}
        <span className={styles.botSpecPrimary}>{primary}</span>
        {preset && badgePlacement === "after" ? <PresetBadge preset={preset} /> : null}
      </span>
      {rest.length > 0 ? <span className={styles.botSpecSecondary}>{rest.join("+")}</span> : null}
    </strong>
  );
}

function PresetBadge({ preset }: { preset: BotPresetId }) {
  return (
    <span className={`${styles.presetBadge} ${presetClassName(preset)}`}>
      {presetLabel(preset)}
    </span>
  );
}

function presetLabel(preset: BotPresetId): string {
  switch (preset) {
    case "easy":
      return "Easy";
    case "hard":
      return "Hard";
    case "normal":
      return "Normal";
  }
}

function presetClassName(preset: BotPresetId | null): string {
  switch (preset) {
    case "easy":
      return styles.presetEasy;
    case "hard":
      return styles.presetHard;
    case "normal":
      return styles.presetNormal;
    default:
      return "";
  }
}

function MetricCell({
  kind,
  label,
  primary,
  secondary,
  nowrap,
}: {
  kind: "results" | "search";
  label: string;
  primary: string;
  secondary?: string;
  nowrap?: boolean;
}) {
  const kindClass = kind === "results" ? styles.metricResults : styles.metricSearch;
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

function SearchTimeSplit({
  maxSearchAvgNs,
  standing,
}: {
  maxSearchAvgNs: number;
  standing: StandingReport;
}) {
  const segments = stageSegments(standing).filter((segment) => segment.percent > 0.05);
  const totalPercent =
    maxSearchAvgNs <= 0 ? 0 : Math.min(100, (averageSearchNs(standing) / maxSearchAvgNs) * 100);
  const label = segments
    .map((segment) => `${segment.label} ${formatPercent(segment.percent)}`)
    .join(", ");
  return (
    <span
      className={`${styles.metricSearch} ${styles.searchTimeSplit}`}
      aria-label={`Search time ${formatNumber(averageSearchNs(standing) / 1_000_000)} ms vs slowest ${formatNumber(maxSearchAvgNs / 1_000_000)} ms. Split: ${label}`}
    >
      <span className={styles.searchTimeScale}>
        <span className={styles.searchTimeTrack} style={{ width: `${totalPercent}%` }}>
          {segments.map((segment) => (
            <span
              className={`${styles.searchTimeSegment} ${segment.className}`}
              key={segment.key}
              style={{ width: `${Math.max(segment.percent, 1)}%` }}
              title={`${segment.label}: ${formatPercent(segment.percent)} (${stageAvgMsLabel(
                segment.stageNs,
                standing.search_move_count,
              )})`}
            />
          ))}
        </span>
      </span>
      <span className={styles.searchTimeLegend}>
        {segments.map((segment) => (
          <span className={styles.searchTimeLegendItem} key={`${segment.key}-legend`}>
            <span className={`${styles.searchTimeSwatch} ${segment.className}`} aria-hidden="true" />
            {segment.shortLabel}
          </span>
        ))}
      </span>
    </span>
  );
}

interface SearchStageSegment {
  className: string;
  key: string;
  label: string;
  percent: number;
  shortLabel: string;
  stageNs: number;
}

function stageSegments(standing: StandingReport) {
  return [
    {
      className: styles.searchTimeMoveGen,
      key: "move-gen",
      label: "Move gen",
      shortLabel: "gen",
      stageNs: standing.stage_move_gen_ns ?? 0,
    },
    {
      className: styles.searchTimeOrdering,
      key: "ordering",
      label: "Ordering",
      shortLabel: "order",
      stageNs: standing.stage_ordering_ns ?? 0,
    },
    {
      className: styles.searchTimeScoring,
      key: "scoring",
      label: "Scoring",
      shortLabel: "score",
      stageNs: standing.stage_eval_ns ?? 0,
    },
    {
      className: styles.searchTimeThreat,
      key: "threat",
      label: "Threat detection",
      shortLabel: "threat",
      stageNs: standing.stage_threat_ns ?? 0,
    },
    {
      className: styles.searchTimeProof,
      key: "proof",
      label: "Proof",
      shortLabel: "proof",
      stageNs: standing.stage_proof_ns ?? 0,
    },
    {
      className: styles.searchTimeOther,
      key: "other",
      label: "Other",
      shortLabel: "other",
      stageNs: stageOtherNs(standing),
    },
  ].map((segment): SearchStageSegment => ({
    ...segment,
    percent: stageSharePercent(segment.stageNs, standing),
  }));
}

function RankingDrilldown({
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
        <BotLabel badgePlacement="before" bot={opponent} />
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
              <span className={styles.boardCaption}>Game #{match.match_index}</span>
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

const FINISHED_BOARD_CELL_SIZE = 24;
const FINISHED_BOARD_LABEL_SIZE = 18;

function FinishedBoard({
  moveCells,
  boardSize,
}: {
  moveCells: number[];
  boardSize: number;
}) {
  const stones = new Map<number, { stone: "black" | "white"; sequence: number }>();
  moveCells.forEach((cell, index) => {
    stones.set(cell, {
      stone: index % 2 === 0 ? "black" : "white",
      sequence: index + 1,
    });
  });
  const gridSpan = Math.max(0, boardSize - 1) * FINISHED_BOARD_CELL_SIZE + 1;
  const columnLabels = Array.from({ length: boardSize }, (_, col) =>
    String.fromCharCode("A".charCodeAt(0) + col),
  );
  const style = {
    "--proof-cell-size": `${FINISHED_BOARD_CELL_SIZE}px`,
    "--proof-label-size": `${FINISHED_BOARD_LABEL_SIZE}px`,
    "--proof-grid-span": `${gridSpan}px`,
    gridTemplateColumns: `var(--proof-label-size) repeat(${boardSize}, var(--proof-cell-size)) var(--proof-label-size)`,
    gridTemplateRows: `var(--proof-label-size) repeat(${boardSize}, var(--proof-cell-size)) var(--proof-label-size)`,
  } as CSSProperties;

  return (
    <div
      aria-label={`Finished board for game with ${moveCells.length} moves`}
      className={styles.proofBoard}
      role="img"
      style={style}
    >
      <div className={`${styles.proofCoordinate} ${styles.proofCorner}`} aria-hidden="true" />
      {columnLabels.map((label) => (
        <div className={`${styles.proofCoordinate} ${styles.proofCoordinateTop}`} key={`col-${label}`}>
          {label}
        </div>
      ))}
      <div className={`${styles.proofCoordinate} ${styles.proofCorner}`} aria-hidden="true" />
      {Array.from({ length: boardSize }, (_, row) => (
        <Fragment key={`row-${row}`}>
          <div className={`${styles.proofCoordinate} ${styles.proofCoordinateLeft}`}>{row + 1}</div>
          {Array.from({ length: boardSize }, (_, col) => {
            const cell = row * boardSize + col;
            const marker = stones.get(cell);
            return (
              <div className={styles.proofCell} key={cell} data-move={cellNotation(cell, boardSize)}>
                {marker ? (
                  <span
                    className={`${styles.proofStone} ${
                      marker.stone === "black" ? styles.proofStoneBlack : styles.proofStoneWhite
                    }`}
                    aria-label={`${marker.stone} move ${marker.sequence} at ${cellNotation(cell, boardSize)}`}
                  >
                    {marker.sequence}
                  </span>
                ) : null}
              </div>
            );
          })}
          <div className={`${styles.proofCoordinate} ${styles.proofCorner}`} aria-hidden="true" />
        </Fragment>
      ))}
      <div className={`${styles.proofCoordinate} ${styles.proofCorner}`} aria-hidden="true" />
      {Array.from({ length: boardSize }, (_, col) => (
        <div
          className={`${styles.proofCoordinate} ${styles.proofCorner}`}
          key={`bottom-${col}`}
          aria-hidden="true"
        />
      ))}
      <div className={`${styles.proofCoordinate} ${styles.proofCorner}`} aria-hidden="true" />
    </div>
  );
}

function BotHowToReadSection({ view }: { view: BotReportView }) {
  const terms = view === "ranking" ? rankingTerms : searchTerms;
  return (
    <section className={`${styles.panel} ${styles.howToRead}`}>
      <h2>Lab Notes</h2>
      <dl className={styles.termList}>
        {terms.map((term) => (
          <TermRow key={term.title} title={term.title} body={term.body} />
        ))}
      </dl>
    </section>
  );
}

function AnalysisHowToReadSection() {
  return (
    <section className={`${styles.panel} ${styles.howToRead}`}>
      <h2>Lab Notes</h2>
      <dl className={styles.termList}>
        {analysisTerms.map((term) => (
          <TermRow key={term.title} title={term.title} body={term.body} />
        ))}
      </dl>
    </section>
  );
}

const rankingTerms = [
  {
    title: "Score",
    body: "Win = 1 point, draw = 0.5. Open a row to compare that bot against each opponent.",
  },
  {
    title: "Elo",
    body: "Relative rating within this report only. Shuffled Elo averages repeated passes over randomized match order.",
  },
  {
    title: "Width",
    body: "Average number of child moves searched after tactical filtering. The secondary pre value is width before trimming.",
  },
  {
    title: "Games",
    body: "Opponent rows show score against that bot. Open a game row to inspect the finished board.",
  },
];

const searchTerms = [
  {
    title: "Nodes",
    body: "Average search nodes per move. Higher is not automatically better; it mainly explains cost.",
  },
  {
    title: "Time Split",
    body: "Per-move CPU time split across move generation, ordering, scoring, threat detection, corridor proof, and remaining overhead.",
  },
  {
    title: "Budget Hit",
    body: "Share of moves cut off by the tournament CPU budget before search finished naturally.",
  },
  {
    title: "Proof",
    body: "Corridor proof is the extra tactical verification pass used by proof-enabled variants.",
  },
];

const analysisTerms = [
  {
    title: "Failure",
    body: "The losing-side failure mode: missed response, missed lethal prevention, missed escape, unclear, or error.",
  },
  {
    title: "Lethal Onset",
    body: "The ply where the position first became guaranteed lost under the analyzer model.",
  },
  {
    title: "Setup Corridor",
    body: "The forced setup before onset. The length value counts that corridor, not the dead turns after the loss was already guaranteed.",
  },
  {
    title: "Frames",
    body: "Open a game to read frames backward from the win. Boxes are candidate replies or threat evidence; letters are proof outcomes.",
  },
];

function botReportIntro(view: BotReportView): string {
  if (view === "search") {
    return "Per-move search cost profile. Width and budget hits show how each config spends compute.";
  }
  return "Round-robin bot results. Score is the primary outcome; shuffled Elo is a report-local stability check.";
}

function TermRow({ title, body }: { title: string; body: string }) {
  return (
    <Fragment>
      <dt>{title}</dt>
      <dd>{body}</dd>
    </Fragment>
  );
}

function BotProvenanceSection({ report, view }: { report: PublishedBotReport; view: BotReportView }) {
  const terms = view === "ranking" ? rankingProvenanceRows(report) : searchProvenanceRows(report);
  return (
    <section className={`${styles.panel} ${styles.provenance}`}>
      <h2>Provenance</h2>
      <dl>
        {terms.map(([label, value]) => (
          <Fragment key={label}>
            <dt>{label}</dt>
            <dd>{value}</dd>
          </Fragment>
        ))}
      </dl>
    </section>
  );
}

function AnalysisProvenanceSection({ report }: { report: PublishedAnalysisReport }) {
  return (
    <section className={`${styles.panel} ${styles.provenance}`}>
      <h2>Provenance</h2>
      <dl>
        <dt>Source</dt>
        <dd>{report.source_report}</dd>
        <dt>Selector</dt>
        <dd>{report.selector}</dd>
        <dt>Games</dt>
        <dd>{report.analyzed}/{report.total}</dd>
        <dt>Model</dt>
        <dd>{analysisModelLabel(report)}</dd>
        <dt>Generated in</dt>
        <dd>{formatDurationMs(report.total_elapsed_ms)}</dd>
        <dt>Schema</dt>
        <dd>v{report.schema_version}</dd>
      </dl>
    </section>
  );
}

function rankingProvenanceRows(report: PublishedBotReport): Array<[string, string]> {
  return [
    ["Schedule", scheduleSummary(report)],
    ["Rules", report.run.rules.variant],
    ["Opening", openingSummary(report)],
    ["Budget", budgetLabel(report)],
    ["Finish", finishSummary(report)],
    ["Generated", report.provenance?.generated_at_local ?? "unknown"],
    ["Git revision", revisionLabel(report)],
    ["Schema", `v${report.schema_version} / ${report.move_codec}`],
  ];
}

function searchProvenanceRows(report: PublishedBotReport): Array<[string, string]> {
  return [
    ["Schedule", scheduleSummary(report)],
    ["Rules", report.run.rules.variant],
    ["Budget", budgetLabel(report)],
    ["Wall clock", formatDurationMs(report.run.total_wall_time_ms)],
    ["Generated", report.provenance?.generated_at_local ?? "unknown"],
    ["Git revision", revisionLabel(report)],
    ["Schema", `v${report.schema_version} / ${report.move_codec}`],
  ];
}

function analysisModelLabel(report: PublishedAnalysisReport): string {
  return `corridor search, depth ${report.model.max_depth}, traceback ${report.model.max_scan_plies}`;
}

function ReportStatePanel({ title, message }: { title: string; message: string }) {
  return (
    <section className={styles.state}>
      <h1>{title}</h1>
      <p>{message}</p>
    </section>
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

function averageSearchNs(standing: StandingReport): number {
  if (standing.search_move_count > 0) {
    return stageDenominatorNs(standing) / standing.search_move_count;
  }
  return Math.max(0, standing.avg_search_time_ms) * 1_000_000;
}

function maxSearchAverageNs(standings: StandingReport[]): number {
  return Math.max(1, ...standings.map(averageSearchNs));
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
