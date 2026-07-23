import { useEffect, useState } from "react";

import { presetForLabSpec, type BotPresetId } from "../core/bot_config";
import {
  type AnalysisEntry,
  type AnalysisSection,
  type PublishedAnalysisReport,
} from "../reports/analysis_report";
import { displayBotSpec } from "../reports/bot_report";
import { ProofFrames } from "./lab-report/AnalysisProofFrames";

import styles from "./ReportRoute.module.css";

export function AnalysisReportContent({
  report,
  initialMatchPath = null,
}: {
  report: PublishedAnalysisReport;
  initialMatchPath?: string | null;
}) {
  useEffect(() => {
    if (!initialMatchPath) return undefined;

    const frame = window.requestAnimationFrame(() => {
      const target = Array.from(
        document.querySelectorAll<HTMLElement>("[data-analysis-match-path]"),
      ).find((element) => element.dataset.analysisMatchPath === initialMatchPath);
      target?.scrollIntoView({ block: "center", behavior: "smooth" });
    });

    return () => window.cancelAnimationFrame(frame);
  }, [initialMatchPath, report]);

  return (
    <section
      aria-labelledby="lab-report-tab-analysis"
      className={`${styles.panel} ${styles.entrantWorkbench} ${styles.analysisWorkbench}`}
      data-view="analysis"
      id="lab-report-panel-analysis"
      role="tabpanel"
    >
      <div className={styles.headerRow}>
        <div>
          <h2>Results</h2>
          <p className={styles.reportNote}>
            Replay analyzer sample. Corridor search walks backward from the win to lethal onset, setup corridor, and last escape.
          </p>
        </div>
      </div>
      <div className={styles.entrantGrid}>
        {report.sections.map((section) => {
          const sectionHasInitialMatch = section.entries.some(
            (entry) => entry.path === initialMatchPath,
          );
          return (
            <AnalysisPairRow
              key={section.label}
              section={section}
              defaultOpen={sectionHasInitialMatch}
              defaultOpenFirstGame={false}
              initialMatchPath={initialMatchPath}
            />
          );
        })}
      </div>
    </section>
  );
}

function AnalysisPairRow({
  section,
  defaultOpen,
  defaultOpenFirstGame,
  initialMatchPath,
}: {
  section: AnalysisSection;
  defaultOpen: boolean;
  defaultOpenFirstGame: boolean;
  initialMatchPath: string | null;
}) {
  const [isOpen, setIsOpen] = useState(defaultOpen);
  const [renderGames, setRenderGames] = useState(defaultOpen);
  const title = analysisPairTitle(section);
  const failureCount = section.entries.filter((entry) => entry.failure).length;

  useEffect(() => {
    if (!defaultOpen) {
      return;
    }
    setIsOpen(true);
    setRenderGames(true);
  }, [defaultOpen]);

  return (
    <details
      className={styles.entrantRow}
      onToggle={(event) => {
        const nextOpen = event.currentTarget.open;
        setIsOpen(nextOpen);
        if (nextOpen) {
          setRenderGames(true);
        }
      }}
      open={isOpen}
    >
      <summary>
        <span className={`${styles.entryTitle} ${styles.analysisPairTitle}`}>
          <span className={styles.analysisPairLine}>
            <strong className={`${styles.analysisPairName} ${presetClassName(title.firstPreset)}`}>
              {title.firstName}
            </strong>
            <span className={styles.analysisRowValue}>{title.firstConfig}</span>
          </span>
          <span className={styles.analysisPairLine}>
            <strong className={`${styles.analysisPairName} ${presetClassName(title.secondPreset)}`}>
              {title.secondName}
            </strong>
            <span className={styles.analysisRowValue}>{title.secondConfig}</span>
          </span>
        </span>
        <SummaryMetric label="Games" value={`${section.analyzed}/${section.total}`} />
        <SummaryMetric label="Failures" value={`${failureCount}`} />
        <SummaryMetric label="Unclear" value={`${section.summary.unclear}`} />
        <SummaryMetric label="Errors" value={`${section.failed}`} />
      </summary>
      {renderGames ? (
        <div className={styles.matchList}>
          {section.entries.map((entry, index) => (
            <AnalysisGameRow
              key={`${section.label}-${entry.match_report.match_index}`}
              entry={entry}
              pair={section}
              defaultOpen={
                entry.path === initialMatchPath || (defaultOpenFirstGame && index === 0)
              }
            />
          ))}
        </div>
      ) : null}
    </details>
  );
}

function AnalysisGameRow({
  entry,
  pair,
  defaultOpen,
}: {
  entry: AnalysisEntry;
  pair: AnalysisSection;
  defaultOpen: boolean;
}) {
  const [isOpen, setIsOpen] = useState(defaultOpen);
  const [renderDetails, setRenderDetails] = useState(defaultOpen);
  const title = entryTitle(entry, pair);

  useEffect(() => {
    if (!defaultOpen) {
      return;
    }
    setIsOpen(true);
    setRenderDetails(true);
  }, [defaultOpen]);

  return (
    <details
      className={styles.match}
      data-analysis-match-path={entry.path}
      onToggle={(event) => {
        const nextOpen = event.currentTarget.open;
        setIsOpen(nextOpen);
        if (nextOpen) {
          setRenderDetails(true);
        }
      }}
      open={isOpen}
    >
      <summary>
        <span className={styles.entryTitle}>
          <strong>{title.match}</strong>
          <span className={styles.analysisSideList}>
            {title.players.map((player) => (
              <span className={styles.analysisSideLine} key={`${player.side}-${player.config}`}>
                <span className={styles.analysisSideTag}>({player.side})</span>
                <span className={styles.analysisRowValue}>
                  {player.config}
                  {player.won ? <span className={styles.analysisWinnerBadge}> (Won)</span> : null}
                </span>
              </span>
            ))}
          </span>
        </span>
        <SummaryMetric label="Failure" value={failureLabel(entry)} />
        <SummaryMetric label="Lethal onset" value={lethalOnsetLabel(entry.lethal_onset)} />
        <SummaryMetric label="Setup corridor" value={setupCorridorSummaryLabel(entry.setup_corridor)} />
        <SummaryMetric label="Game len" value={`${entry.match_report.move_count}`} />
      </summary>
      {renderDetails ? (
        <div className={styles.details}>
          <div className={styles.detailGrid}>
            <DetailCard label="Failure step" value={failureStepLabel(entry)} />
            <DetailCard label="Search details" value={searchDetailsLabel(entry)} />
          </div>
          <ProofFrames entry={entry} />
        </div>
      ) : null}
    </details>
  );
}

function SummaryMetric({ label, value }: { label: string; value: string }) {
  return (
    <span className={styles.metric} data-label={label}>
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

function entryTitle(entry: AnalysisEntry, pair: AnalysisSection): {
  match: string;
  players: Array<{
    config: string;
    side: "BLACK" | "WHITE";
    won: boolean;
  }>;
} {
  const entrants = [pair.entrant_a, pair.entrant_b];
  return {
    match: `#${entry.match_report.match_index}`,
    players: entrants.map((entrant) => ({
      config: cleanBotName(entrant),
      side: sideForEntrant(entry, entrant),
      won: entrantWon(entry, entrant),
    })),
  };
}

function sideForEntrant(entry: AnalysisEntry, entrant: string): "BLACK" | "WHITE" {
  return sameBotSpec(entrant, entry.match_report.white) ? "WHITE" : "BLACK";
}

function entrantWon(entry: AnalysisEntry, entrant: string): boolean {
  if (entry.match_report.result === "black_won") {
    return sameBotSpec(entrant, entry.match_report.black);
  }
  if (entry.match_report.result === "white_won") {
    return sameBotSpec(entrant, entry.match_report.white);
  }
  return sameBotSpec(entrant, entry.match_report.winner ?? "");
}

function sameBotSpec(left: string, right: string): boolean {
  return left.replace(/_/g, "+") === right.replace(/_/g, "+");
}

function analysisPairTitle(section: AnalysisSection): {
  firstName: string;
  firstConfig: string;
  firstPreset: BotPresetId | null;
  secondName: string;
  secondConfig: string;
  secondPreset: BotPresetId | null;
} {
  const [firstName, secondName] = section.label.split(/\s+vs\s+/i);
  return {
    firstName: firstName || "Bot",
    firstConfig: cleanBotName(section.entrant_a),
    firstPreset: presetForLabSpec(section.entrant_a),
    secondName: secondName || "Opponent",
    secondConfig: cleanBotName(section.entrant_b),
    secondPreset: presetForLabSpec(section.entrant_b),
  };
}

function cleanBotName(value: string): string {
  return displayBotSpec(value.replace(/_/g, "+"));
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

function failureLabel(entry: AnalysisEntry): string {
  if (!entry.failure) {
    return entry.root_cause ?? entry.unclear_reason ?? "-";
  }
  switch (entry.failure.mode) {
    case "missed_immediate_win":
      return "missed win";
    case "missed_immediate_response":
      return "missed 4";
    case "missed_imminent_response":
      return "missed 3";
    case "missed_lethal_prevention":
      return missedLethalOnsetLabel(entry.lethal_onset);
    case "missed_escape":
      return "missed escape";
    case "unclear":
      return "unclear";
    default:
      return entry.failure.mode.replace(/_/g, " ");
  }
}

function failureStepLabel(entry: AnalysisEntry): string {
  const ply = `Ply ${plyLabel(entry.critical_loser_ply)}`;
  const move = entry.failure?.actual_notation?.trim();
  const outcome = failureLabel(entry);
  return move ? `${ply} - ${move}: ${outcome}` : `${ply}: ${outcome}`;
}

function searchDetailsLabel(entry: AnalysisEntry): string {
  const details = entry.search_details;
  if (!details) {
    return `${entry.elapsed_ms}ms`;
  }
  return [
    `${entry.elapsed_ms}ms`,
    `${formatCompact(details.branch_probes)} probes`,
    `${formatCompact(details.search_nodes)} nodes`,
    `d${details.max_depth_reached}`,
  ].join(" / ");
}

function lethalOnsetLabel(onset: AnalysisEntry["lethal_onset"]): string {
  if (!onset) {
    return "-";
  }
  const shape = lethalOnsetShapeLabel(onset);
  return shape ? `${onset.prefix_ply} · ${shape}` : `${onset.prefix_ply}`;
}

function missedLethalOnsetLabel(onset: AnalysisEntry["lethal_onset"]): string {
  return `missed ${lethalOnsetShapeLabel(onset) ?? "fork"}`;
}

function lethalOnsetShapeLabel(onset: AnalysisEntry["lethal_onset"]): string | null {
  const label = onset?.shape?.label?.trim();
  if (!label) {
    return null;
  }
  const mechanisms = onset?.shape?.mechanisms ?? [];
  const forbidden = mechanisms.includes("forbidden_cover");
  const multiRoute = mechanisms.includes("multi_route");
  if (
    label === "4" &&
    onset?.kind === "terminal_coverage" &&
    multiRoute &&
    (onset.terminal_targets?.length ?? 0) >= 2
  ) {
    return "open four";
  }
  if (forbidden && !multiRoute) {
    return `forbidden ${label}`;
  }
  return label;
}

function setupCorridorRangeLabel(interval: AnalysisEntry["setup_corridor"]): string {
  if (!interval) {
    return "-";
  }
  return `${interval.start_ply}-${interval.end_ply}`;
}

function setupCorridorSummaryLabel(interval: AnalysisEntry["setup_corridor"]): string {
  if (!interval) {
    return "-";
  }
  return `${setupCorridorRangeLabel(interval)} / ${setupCorridorLengthLabel(interval)}`;
}

function setupCorridorLengthLabel(interval: AnalysisEntry["setup_corridor"]): string {
  if (!interval) {
    return "-";
  }
  return `${Math.max(0, interval.end_ply - interval.start_ply + 1)}`;
}

function plyLabel(value: number | null | undefined): string {
  return value == null ? "-" : `${value}`;
}

function formatCompact(value: number): string {
  return Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: 1 }).format(value);
}
