import { Fragment, useEffect, useState, type KeyboardEvent } from "react";

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
  const totalMatches = report.matches.length;
  const topStanding = report.standings[0];
  const [openBots, setOpenBots] = useState<Set<string>>(() => new Set());
  const [openPairs, setOpenPairs] = useState<Set<string>>(() => new Set());

  const toggleBot = (bot: string) => {
    setOpenBots((current) => toggleSetValue(current, bot));
  };
  const togglePair = (pair: PairwiseReport) => {
    setOpenPairs((current) => toggleSetValue(current, pairKey(pair)));
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
            <ReportChip label="Rule" value={report.run.rules.variant} />
            <ReportChip label="Schedule" value={report.run.schedule} />
            <ReportChip label="Matches" value={`${totalMatches}`} />
            <ReportChip label="Budget" value={budgetLabel(report)} />
            <ReportChip label="Top bot" value={topStanding ? displayBotSpec(topStanding.bot) : "-"} />
            <ReportChip label="Revision" value={revisionLabel(report)} />
          </div>
        </header>

        <section className={styles.panel}>
          <h2>Ranking</h2>
          <div className={styles.tableWrap}>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th>#</th>
                  <th>Bot</th>
                  <th>Score</th>
                  <th>W / D / L</th>
                  <th>Elo</th>
                  <th>Time</th>
                  <th>Nodes</th>
                  <th>Depth</th>
                  <th>Width</th>
                  <th>Budget</th>
                </tr>
              </thead>
              <tbody>
                {report.standings.map((standing, index) => {
                  const isOpen = openBots.has(standing.bot);
                  return (
                    <Fragment key={standing.bot}>
                      <StandingRow
                        rank={index + 1}
                        standing={standing}
                        isOpen={isOpen}
                        onToggle={() => toggleBot(standing.bot)}
                      />
                      {isOpen ? (
                        <tr className={styles.expandedRow}>
                          <td colSpan={10}>
                            <StandingDetails report={report} bot={standing.bot} />
                          </td>
                        </tr>
                      ) : null}
                    </Fragment>
                  );
                })}
              </tbody>
            </table>
          </div>
        </section>

        <section className={styles.panel}>
          <h2>Pairwise</h2>
          <div className={styles.tableWrap}>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th>Pair</th>
                  <th>Score A</th>
                  <th>Score B</th>
                  <th>W / D / L</th>
                  <th>Total</th>
                </tr>
              </thead>
              <tbody>
                {report.pairwise.map((pair) => {
                  const key = pairKey(pair);
                  const isOpen = openPairs.has(key);
                  return (
                    <Fragment key={key}>
                      <PairwiseRow pair={pair} isOpen={isOpen} onToggle={() => togglePair(pair)} />
                      {isOpen ? (
                        <tr className={styles.expandedRow}>
                          <td colSpan={5}>
                            <PairwiseDetails report={report} pair={pair} />
                          </td>
                        </tr>
                      ) : null}
                    </Fragment>
                  );
                })}
              </tbody>
            </table>
          </div>
        </section>
      </div>
    </main>
  );
}

function StandingRow({
  rank,
  standing,
  isOpen,
  onToggle,
}: {
  rank: number;
  standing: StandingReport;
  isOpen: boolean;
  onToggle: () => void;
}) {
  const score = scorePercent(standing.wins, standing.draws, standing.match_count);
  return (
    <tr
      className={styles.clickableRow}
      role="button"
      tabIndex={0}
      aria-expanded={isOpen}
      onClick={onToggle}
      onKeyDown={(event) => handleToggleKey(event, onToggle)}
    >
      <td>{isOpen ? "v" : ">"} {rank}</td>
      <td>
        <span className={styles.botCell}>
          <strong>{displayBotSpec(standing.bot)}</strong>
          <span>{standing.bot}</span>
        </span>
      </td>
      <td>{formatPercent(score)}</td>
      <td>{`${standing.wins} / ${standing.draws} / ${standing.losses}`}</td>
      <td>{formatNumber(standing.shuffled_elo_avg)}</td>
      <td>{formatMs(standing.avg_search_time_ms)}</td>
      <td>{formatCompact(standing.avg_nodes)}</td>
      <td>{formatNumber(standing.avg_depth)}</td>
      <td>{formatNumber(standing.avg_child_moves_after ?? 0)}</td>
      <td>{formatPercent((standing.budget_exhausted_rate ?? 0) * 100)}</td>
    </tr>
  );
}

function PairwiseRow({
  pair,
  isOpen,
  onToggle,
}: {
  pair: PairwiseReport;
  isOpen: boolean;
  onToggle: () => void;
}) {
  return (
    <tr
      className={styles.clickableRow}
      role="button"
      tabIndex={0}
      aria-expanded={isOpen}
      onClick={onToggle}
      onKeyDown={(event) => handleToggleKey(event, onToggle)}
    >
      <td>
        <span className={styles.botCell}>
          <strong>{isOpen ? "v" : ">"} {displayBotSpec(pair.bot_a)}</strong>
          <span>vs {displayBotSpec(pair.bot_b)}</span>
        </span>
      </td>
      <td>{formatNumber(pair.score_a)}</td>
      <td>{formatNumber(pair.score_b)}</td>
      <td>{`${pair.wins_a} / ${pair.draws} / ${pair.wins_b}`}</td>
      <td>{pair.total}</td>
    </tr>
  );
}

function StandingDetails({
  report,
  bot,
}: {
  report: PublishedBotReport;
  bot: string;
}) {
  const pairs = report.pairwise.filter((pair) => pair.bot_a === bot || pair.bot_b === bot);
  return (
    <div className={styles.expansionPanel}>
      <h3>{displayBotSpec(bot)} by opponent</h3>
      <div className={styles.compactGrid}>
        {pairs.map((pair) => {
          const opponent = opponentForPair(pair, bot);
          return (
            <span className={styles.compactCard} key={pairKey(pair)}>
              <strong>vs {displayBotSpec(opponent)}</strong>
              <span>{formatPercent(pairScoreForBot(pair, bot))} score</span>
              <span>{pairRecordForBot(pair, bot)}</span>
            </span>
          );
        })}
      </div>
    </div>
  );
}

function PairwiseDetails({
  report,
  pair,
}: {
  report: PublishedBotReport;
  pair: PairwiseReport;
}) {
  const matches = report.matches.filter((match) => matchIsPair(match, pair)).slice(0, 24);
  return (
    <div className={styles.expansionPanel}>
      <h3>
        {displayBotSpec(pair.bot_a)} vs {displayBotSpec(pair.bot_b)}
      </h3>
      <div className={styles.compactGrid}>
        {matches.map((match) => (
          <span className={styles.compactCard} key={match.match_index}>
            <strong>#{match.match_index}</strong>
            <span>
              B {displayBotSpec(match.black)} / W {displayBotSpec(match.white)}
            </span>
            <span>{matchResultLabel(match)} / {match.move_count} moves</span>
          </span>
        ))}
      </div>
    </div>
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

function budgetLabel(report: PublishedBotReport): string {
  if (report.run.search_cpu_time_ms) {
    return `${report.run.search_cpu_time_ms}ms CPU`;
  }
  if (report.run.search_time_ms) {
    return `${report.run.search_time_ms}ms`;
  }
  return "uncapped";
}

function revisionLabel(report: PublishedBotReport): string {
  const commit = report.provenance?.git_commit ?? "unknown";
  return report.provenance?.git_dirty ? `${commit} dirty` : commit;
}

function formatNumber(value: number): string {
  return value.toFixed(1);
}

function formatMs(value: number): string {
  return `${value.toFixed(1)}ms`;
}

function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}

function formatCompact(value: number): string {
  if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(1)}M`;
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
    return `${pair.wins_a} / ${pair.draws} / ${pair.wins_b}`;
  }
  return `${pair.wins_b} / ${pair.draws} / ${pair.wins_a}`;
}

function matchIsPair(match: PublishedMatchReport, pair: PairwiseReport): boolean {
  return (
    (match.black === pair.bot_a && match.white === pair.bot_b) ||
    (match.black === pair.bot_b && match.white === pair.bot_a)
  );
}

function matchResultLabel(match: PublishedMatchReport): string {
  if (!match.winner) {
    return "draw";
  }
  return `${displayBotSpec(match.winner)} won`;
}
