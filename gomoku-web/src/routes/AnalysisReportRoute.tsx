import { useEffect, useState } from "react";

import {
  loadAnalysisReport,
  type AnalysisEntry,
  type AnalysisSection,
  type ProofFrame,
  type PublishedAnalysisReport,
} from "../reports/analysis_report";

import styles from "./ReportRoute.module.css";

const baseUrl = import.meta.env.BASE_URL;

type LoadState =
  | { status: "loading" }
  | { status: "loaded"; report: PublishedAnalysisReport }
  | { status: "error"; message: string };

export function AnalysisReportRoute() {
  const [state, setState] = useState<LoadState>({ status: "loading" });

  useEffect(() => {
    document.title = "Gomoku2D Analysis Report";
    let cancelled = false;
    loadAnalysisReport()
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
    return <ReportState title="Replay Analysis" message="Loading report…" />;
  }
  if (state.status === "error") {
    return <ReportState title="Replay Analysis" message={state.message} />;
  }

  return <AnalysisReportPage report={state.report} />;
}

function AnalysisReportPage({ report }: { report: PublishedAnalysisReport }) {
  return (
    <main className={styles.page}>
      <div className={styles.shell}>
        <header className={styles.hero}>
          <div className={styles.headerRow}>
            <div>
              <p className="uiPageEyebrow">Gomoku2D lab</p>
              <h1 className={styles.title}>Replay Analysis</h1>
            </div>
            <nav className={styles.links} aria-label="Report links">
              <a href={baseUrl}>Game</a>
              <a href={`${baseUrl}bot-report/`}>Bots</a>
              <a href={`${baseUrl}assets/`}>Assets</a>
            </nav>
          </div>
          <div className={styles.chips} aria-label="Analysis summary">
            <ReportChip label="Source" value={report.source_report} />
            <ReportChip label="Selector" value={report.selector} />
            <ReportChip label="Entries" value={`${report.analyzed}/${report.total}`} />
            <ReportChip label="Unclear" value={`${report.summary.unclear}`} />
            <ReportChip label="Errors" value={`${report.failed}`} />
            <ReportChip label="Probe depth" value={`${report.model.max_depth}`} />
          </div>
        </header>

        {report.sections.map((section, sectionIndex) => (
          <AnalysisSectionPanel
            key={section.label}
            section={section}
            defaultOpenFirst={sectionIndex === 0}
          />
        ))}
      </div>
    </main>
  );
}

function AnalysisSectionPanel({
  section,
  defaultOpenFirst,
}: {
  section: AnalysisSection;
  defaultOpenFirst: boolean;
}) {
  return (
    <section className={styles.panel}>
      <div className={styles.headerRow}>
        <div>
          <h2>{section.label}</h2>
          <p className={styles.muted}>
            {cleanBotName(section.entrant_a)} vs {cleanBotName(section.entrant_b)}
          </p>
        </div>
        <span className={styles.metric}>
          <span>Entries</span>
          <strong>{section.analyzed}/{section.total}</strong>
        </span>
      </div>
      <div className={styles.list}>
        {section.entries.map((entry, index) => (
          <AnalysisEntryCard
            key={`${section.label}-${entry.match_report.match_index}`}
            entry={entry}
            defaultOpen={defaultOpenFirst && index === 0}
          />
        ))}
      </div>
    </section>
  );
}

function AnalysisEntryCard({
  entry,
  defaultOpen,
}: {
  entry: AnalysisEntry;
  defaultOpen: boolean;
}) {
  const title = entryTitle(entry);
  const setupLength = entry.setup_corridor
    ? entry.setup_corridor.end_ply - entry.setup_corridor.start_ply
    : null;

  return (
    <details className={styles.entry} open={defaultOpen}>
      <summary>
        <span className={styles.entryTitle}>
          <strong>{title.match}</strong>
          <span>{title.players}</span>
        </span>
        <SummaryMetric label="Winner" value={entry.match_report.winner ?? "-"} />
        <SummaryMetric label="Failure" value={failureLabel(entry)} />
        <SummaryMetric label="Onset" value={plyLabel(entry.lethal_onset?.ply)} />
        <SummaryMetric label="Corridor len" value={setupLength == null ? "-" : `${setupLength}`} />
        <SummaryMetric label="Game len" value={`${entry.match_report.move_count}`} />
      </summary>
      <div className={styles.details}>
        <div className={styles.detailGrid}>
          <DetailCard label="Root cause" value={entry.root_cause ?? "-"} />
          <DetailCard label="Last chance" value={plyLabel(entry.last_chance_ply)} />
          <DetailCard label="Critical ply" value={plyLabel(entry.critical_loser_ply)} />
          <DetailCard label="Search time" value={`${entry.elapsed_ms}ms`} />
        </div>
        {entry.failure ? (
          <DetailCard
            label="Failure step"
            value={`${entry.failure.actual_notation ?? "-"}: ${entry.failure.mode}`}
          />
        ) : null}
        <ProofFrames frames={entry.proof_details?.proof_frames ?? []} />
      </div>
    </details>
  );
}

function ProofFrames({ frames }: { frames: ProofFrame[] }) {
  if (frames.length === 0) {
    return <p className={styles.muted}>No proof frames captured for this entry.</p>;
  }

  return (
    <div className={styles.frames}>
      {frames.slice(0, 8).map((frame) => (
        <article className={styles.frame} key={`${frame.label}-${frame.ply}`}>
          <div className={styles.headerRow}>
            <span className={styles.entryTitle}>
              <strong>{frame.label}</strong>
              <span>
                ply {frame.ply} / {frame.side_to_move} / {frame.status}
              </span>
            </span>
            {frame.move_played_notation ? (
              <span className={styles.muted}>actual {frame.move_played_notation}</span>
            ) : null}
          </div>
          <div className={styles.markerList}>
            {frame.markers.slice(0, 24).map((marker) => (
              <span className={styles.marker} key={`${marker.notation}-${marker.kinds.join(".")}`}>
                {marker.notation}: {marker.kinds.map(markerKindLabel).join(", ")}
              </span>
            ))}
          </div>
          {frame.reply_outcomes.length > 0 ? (
            <div className={styles.markerList}>
              {frame.reply_outcomes.slice(0, 12).map((reply) => (
                <span className={styles.marker} key={`${reply.notation}-${reply.outcome}`}>
                  {reply.notation}: {reply.outcome}
                </span>
              ))}
            </div>
          ) : null}
        </article>
      ))}
    </div>
  );
}

function SummaryMetric({ label, value }: { label: string; value: string }) {
  return (
    <span className={styles.metric}>
      <span>{label}</span>
      <strong>{value}</strong>
    </span>
  );
}

function DetailCard({ label, value }: { label: string; value: string }) {
  return (
    <span className={styles.miniCard}>
      <span>{label}</span>
      <strong>{value}</strong>
    </span>
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

function entryTitle(entry: AnalysisEntry): { match: string; players: string } {
  return {
    match: `#${entry.match_report.match_index}`,
    players: `${cleanBotName(entry.match_report.black)} vs ${cleanBotName(entry.match_report.white)}`,
  };
}

function cleanBotName(value: string): string {
  return value
    .replace(/_/g, "+")
    .replace(/\+corridor-proof-c\d+-d\d+-w\d+/g, "+corridor-proof");
}

function failureLabel(entry: AnalysisEntry): string {
  return entry.failure?.mode ?? entry.root_cause ?? entry.unclear_reason ?? "-";
}

function plyLabel(value: number | null | undefined): string {
  return value == null ? "-" : `${value}`;
}

function markerKindLabel(value: string): string {
  return value.replace(/_/g, " ");
}
