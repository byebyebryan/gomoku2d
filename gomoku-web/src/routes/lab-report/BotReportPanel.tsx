import {
  Fragment,
  useState,
  type CSSProperties,
  type KeyboardEvent,
} from "react";

import { presetForLabSpec, type BotPresetId } from "../../core/bot_config";
import {
  displayBotSpec,
  scorePercent,
  type PairwiseReport,
  type PublishedBotReport,
  type PublishedMatchReport,
  type StandingReport,
} from "../../reports/bot_report";
import { botReportIntro, type BotReportView } from "./ReportSupport";

import styles from "../ReportRoute.module.css";

export function BotReportPanel({
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
      data-report-board="finished"
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
