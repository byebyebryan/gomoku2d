import { Fragment } from "react";

import type { PublishedAnalysisReport } from "../../reports/analysis_report";
import type {
  PublishedBotReport,
  ReportProvenance,
} from "../../reports/bot_report";

import styles from "../ReportRoute.module.css";

export type BotReportView = "ranking" | "search";

export function BotHowToReadSection({ view }: { view: BotReportView }) {
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

export function AnalysisHowToReadSection() {
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

export function botReportIntro(view: BotReportView): string {
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

export function BotProvenanceSection({ report, view }: { report: PublishedBotReport; view: BotReportView }) {
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

export function AnalysisProvenanceSection({ report }: { report: PublishedAnalysisReport }) {
  return (
    <section className={`${styles.panel} ${styles.provenance}`}>
      <h2>Provenance</h2>
      <dl>
        <dt>Source</dt>
        <dd>{report.source_report}</dd>
        <dt>Generated</dt>
        <dd>{report.provenance?.generated_at_local ?? "unknown"}</dd>
        <dt>Revision</dt>
        <dd>{provenanceRevisionLabel(report.provenance)}</dd>
        <dt>Source revision</dt>
        <dd>{provenanceRevisionLabel(report.source_report_provenance)}</dd>
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

export function ReportStatePanel({ title, message }: { title: string; message: string }) {
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
  return provenanceRevisionLabel(report.provenance);
}

function provenanceRevisionLabel(provenance?: ReportProvenance | null): string {
  const commit = provenance?.git_commit ?? "unknown";
  return provenance?.git_dirty ? `${commit} dirty` : commit;
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
