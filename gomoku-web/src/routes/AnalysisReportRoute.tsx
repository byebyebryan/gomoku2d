import { Fragment, type CSSProperties, useEffect, useState } from "react";

import { presetForLabSpec, type BotPresetId } from "../core/bot_config";
import {
  type AnalysisEntry,
  type AnalysisSection,
  type ProofFrame,
  type PublishedAnalysisReport,
} from "../reports/analysis_report";
import { displayBotSpec } from "../reports/bot_report";

import styles from "./ReportRoute.module.css";

const DEFAULT_REPORT_BOARD_SIZE = 15;
const ANALYSIS_BOARD_CELL_SIZE = 20;
const ANALYSIS_BOARD_LABEL_SIZE = 16;

export function AnalysisReportContent({
  report,
  initialMatchPath = null,
}: {
  report: PublishedAnalysisReport;
  initialMatchPath?: string | null;
}) {
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

function ProofFrames({ entry }: { entry: AnalysisEntry }) {
  const frames = entry.proof_details?.proof_frames ?? [];
  if (frames.length === 0) {
    return <p className={styles.muted}>No proof frames captured for this entry.</p>;
  }

  const boardSize = boardSizeFromMoveCells(entry.match_report.move_cells);
  const winner = winnerSide(entry);
  const winningFrame = frames.find((frame) => frame.label === "winning_ply") ?? frames[0];
  const turnFrames = winner
    ? frames.filter((frame) => frame !== winningFrame && sideFromValue(frame.side_to_move) === opponentSide(winner))
    : frames.filter((frame) => frame !== winningFrame);

  return (
    <div className={styles.frames}>
      <ProofLegend />
      <ProofFrameArticle
        boardSize={boardSize}
        frame={winningFrame}
        moveCells={entry.match_report.move_cells}
        title={`Final ply ${winningFrame.ply}`}
        lines={[
          ["Winner move", `${winningFrame.ply}: ${winningFrame.move_played_notation ?? "-"}`],
          ["Result", `${winningFrame.side_to_move} won`],
        ]}
      />
      {turnFrames.map((defenderFrame) => {
        const attackerFrame = frames.find((frame) => (
          frame.ply === defenderFrame.ply - 1
          && winner != null
          && sideFromValue(frame.side_to_move) === winner
        ));
        const title = attackerFrame
          ? `Turn ${attackerFrame.ply}-${defenderFrame.ply}`
          : `Before ply ${defenderFrame.ply}`;
        const winnerMove = attackerFrame
          ? `${attackerFrame.ply}: ${attackerFrame.move_played_notation ?? "-"}`
          : "-";
        const loserReply = `${defenderFrame.ply}: ${defenderFrame.move_played_notation ?? "-"}`;
        const decision = winner
          ? `${defenderFrame.side_to_move} to respond / ${perspectiveProofStatusLabel(defenderFrame, winner)}`
          : `${defenderFrame.side_to_move} to respond / ${proofStatusLabel(defenderFrame.status)}`;
        const lines: Array<[string, string]> = [
          ["Winner move", winnerMove],
          ["Loser reply", loserReply],
        ];
        lines.push(["Decision", decision]);

        return (
          <ProofFrameArticle
            boardSize={boardSize}
            extraActual={attackerFrame ? actualMoveFromFrame(attackerFrame) : null}
            frame={defenderFrame}
            key={`${defenderFrame.label}-${defenderFrame.ply}`}
            lines={lines}
            moveCells={entry.match_report.move_cells}
            title={title}
          />
        );
      })}
    </div>
  );
}

function ProofLegend() {
  return (
    <div className={styles.proofLegend} aria-label="Proof marker legend">
      <div className={styles.proofLegendRow}>
        <LegendRole className={styles.legendWinning} label="immediate win" />
        <LegendRole className={styles.legendThreat} label="immediate threat" />
        <LegendRole className={styles.legendImminent} label="imminent threat" />
        <LegendRole className={styles.legendOffensive} label="counter threat" />
        <LegendRole className={styles.legendCorridorEntry} label="corridor entry" />
      </div>
      <div className={styles.proofLegendRow}>
        <LegendOutcome className={styles.legendImmediateLoss} marker="!" label="immediate loss" />
        <LegendOutcome className={styles.legendForced} marker="L" label="forced loss" />
        <LegendOutcome className={styles.legendForbidden} marker="X" label="forbidden" />
        <LegendOutcome className={styles.legendConfirmed} marker="E" label="confirmed escape" />
        <LegendOutcome className={styles.legendPossible} marker="P" label="possible escape" />
        <LegendOutcome className={styles.legendUnknown} marker="?" label="unknown" />
      </div>
    </div>
  );
}

function LegendRole({ className, label }: { className: string; label: string }) {
  return <span className={`${styles.legendRole} ${className}`}>{label}</span>;
}

function LegendOutcome({ className, marker, label }: { className: string; marker: string; label: string }) {
  return (
    <span className={styles.legendOutcome}>
      <strong className={className}>{marker}</strong> {label}
    </span>
  );
}

function ProofFrameArticle({
  boardSize,
  extraActual,
  frame,
  lines,
  moveCells,
  title,
}: {
  boardSize: number;
  extraActual?: ActualMove | null;
  frame: ProofFrame;
  lines: Array<[string, string]>;
  moveCells: number[];
  title: string;
}) {
  return (
    <article className={`${styles.frame} ${styles.analysisFrame}`} data-ply={frame.ply}>
      <ProofBoard
        boardSize={boardSize}
        extraActual={extraActual}
        frame={frame}
        moveCells={moveCells}
      />
      <div className={styles.proofFrameCopy}>
        <h3>{title}</h3>
        <div className={styles.proofFrameLines}>
          {lines.map(([label, value]) => (
            <div className={styles.proofFrameLine} key={label}>
              <span>{label}</span>
              <strong>{value}</strong>
            </div>
          ))}
        </div>
        <ReplyOutcomes frame={frame} />
      </div>
    </article>
  );
}

function ReplyOutcomes({ frame }: { frame: ProofFrame }) {
  if (frame.reply_outcomes.length === 0) {
    return null;
  }

  return (
    <div className={styles.replyOutcomes}>
      <div className={`${styles.replyOutcomeRow} ${styles.replyOutcomeHeader}`}>
        <span>Move</span>
        <span>Role</span>
        <span>Outcome</span>
      </div>
      {frame.reply_outcomes.map((reply) => (
        <div className={styles.replyOutcomeRow} key={`${reply.notation}-${reply.outcome}`}>
          <strong>{reply.notation}</strong>
          <span>{reply.roles.map(replyRoleLabel).join(", ") || "-"}</span>
          <span>{replyOutcomeLabel(reply.outcome)}</span>
        </div>
      ))}
    </div>
  );
}

function ProofBoard({
  boardSize,
  extraActual,
  frame,
  moveCells,
}: {
  boardSize: number;
  extraActual?: ActualMove | null;
  frame: ProofFrame;
  moveCells: number[];
}) {
  const stones = stonesBeforePly(moveCells, frame.ply);
  const markersByCell = new Map<number, ProofFrame["markers"][number]>();
  for (const marker of frame.markers) {
    const cell = notationToCell(marker.notation, boardSize);
    if (cell != null) {
      markersByCell.set(cell, marker);
    }
  }
  const extraActualCell = extraActual ? notationToCell(extraActual.notation, boardSize) : null;
  const gridSpan = Math.max(0, boardSize - 1) * ANALYSIS_BOARD_CELL_SIZE + 1;
  const columnLabels = Array.from({ length: boardSize }, (_, col) =>
    String.fromCharCode("A".charCodeAt(0) + col),
  );
  const style = {
    "--proof-cell-size": `${ANALYSIS_BOARD_CELL_SIZE}px`,
    "--proof-label-size": `${ANALYSIS_BOARD_LABEL_SIZE}px`,
    "--proof-grid-span": `${gridSpan}px`,
    gridTemplateColumns: `var(--proof-label-size) repeat(${boardSize}, var(--proof-cell-size)) var(--proof-label-size)`,
    gridTemplateRows: `var(--proof-label-size) repeat(${boardSize}, var(--proof-cell-size)) var(--proof-label-size)`,
  } as CSSProperties;

  return (
    <div
      aria-label={`Analysis board before ply ${frame.ply}: ${frame.side_to_move} to move, ${proofStatusLabel(frame.status)}`}
      className={styles.proofBoard}
      data-proof-board="analysis"
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
            const stone = stones.get(cell);
            const marker = markersByCell.get(cell);
            const label = markerLabel(marker);
            const markerActualSide = marker?.kinds.includes("actual")
              ? sideFromValue(frame.side_to_move)
              : null;
            const actualSide = markerActualSide ?? (extraActualCell === cell ? extraActual?.side ?? null : null);
            const showSolidStone = stone && actualSide == null;
            return (
              <div
                className={proofCellClassName(marker)}
                key={cell}
                data-move={cellNotation(cell, boardSize)}
              >
                {showSolidStone ? (
                  <span
                    className={`${styles.proofStone} ${
                      stone === "black" ? styles.proofStoneBlack : styles.proofStoneWhite
                    }`}
                    aria-label={`${stone} stone at ${cellNotation(cell, boardSize)}`}
                  />
                ) : null}
                {actualSide ? (
                  <span
                    className={`${styles.proofActualStone} ${
                      actualSide === "Black"
                        ? styles.proofActualStoneBlack
                        : styles.proofActualStoneWhite
                    }`}
                    aria-label={`${actualSide} actual move at ${cellNotation(cell, boardSize)}`}
                  />
                ) : null}
                {label ? <span className={styles.proofMarker}>{label}</span> : null}
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

type ProofMarker = ProofFrame["markers"][number];
type Side = "Black" | "White";
type ActualMove = {
  notation: string;
  side: Side;
};

function boardSizeFromMoveCells(moveCells: number[]): number {
  const maxCell = moveCells.reduce((max, cell) => Math.max(max, cell), 0);
  return Math.max(DEFAULT_REPORT_BOARD_SIZE, Math.ceil(Math.sqrt(maxCell + 1)));
}

function stonesBeforePly(moveCells: number[], ply: number): Map<number, "black" | "white"> {
  const stones = new Map<number, "black" | "white">();
  const prefixLength = Math.max(0, Math.min(moveCells.length, ply - 1));
  moveCells.slice(0, prefixLength).forEach((cell, index) => {
    stones.set(cell, index % 2 === 0 ? "black" : "white");
  });
  return stones;
}

function notationToCell(notation: string, boardSize: number): number | null {
  const match = /^([A-Z])(\d+)$/.exec(notation.trim().toUpperCase());
  if (!match) {
    return null;
  }
  const col = match[1].charCodeAt(0) - "A".charCodeAt(0);
  const row = Number.parseInt(match[2], 10) - 1;
  if (!Number.isInteger(row) || row < 0 || row >= boardSize || col < 0 || col >= boardSize) {
    return null;
  }
  return row * boardSize + col;
}

function cellNotation(cell: number, boardSize: number): string {
  const row = Math.floor(cell / boardSize);
  const col = cell % boardSize;
  return `${String.fromCharCode("A".charCodeAt(0) + col)}${row + 1}`;
}

function proofCellClassName(marker: ProofMarker | undefined): string {
  const classes = [styles.proofCell];
  if (!marker) {
    return classes.join(" ");
  }
  for (const kind of marker.kinds) {
    const markerClass = markerKindClass(kind);
    if (markerClass) {
      classes.push(markerClass);
    }
  }
  return classes.join(" ");
}

function markerKindClass(kind: string): string | null {
  switch (kind) {
    case "winning":
      return styles.markerWinning;
    case "threat":
      return styles.markerThreat;
    case "imminent_defense":
      return styles.markerImminentDefense;
    case "offensive_counter":
      return styles.markerOffensiveCounter;
    case "winning_evidence":
      return styles.markerWinningEvidence;
    case "threat_evidence":
      return styles.markerThreatEvidence;
    case "imminent_evidence":
      return styles.markerImminentEvidence;
    case "offensive_evidence":
      return styles.markerOffensiveEvidence;
    case "corridor_entry_black":
      return styles.markerCorridorEntryBlack;
    case "corridor_entry_white":
      return styles.markerCorridorEntryWhite;
    case "forbidden":
      return styles.markerForbidden;
    case "forced_loss":
      return styles.markerForcedLoss;
    case "immediate_loss":
      return styles.markerImmediateLoss;
    case "confirmed_escape":
      return styles.markerConfirmedEscape;
    case "possible_escape":
      return styles.markerPossibleEscape;
    case "unknown_outcome":
      return styles.markerUnknownOutcome;
    default:
      return null;
  }
}

function markerLabel(marker: ProofMarker | undefined): string {
  if (!marker || marker.kinds.includes("actual")) {
    return "";
  }
  if (marker.kinds.includes("forbidden")) {
    return "X";
  }
  if (marker.kinds.includes("immediate_loss")) {
    return "!";
  }
  if (marker.kinds.includes("forced_loss")) {
    return "L";
  }
  if (marker.kinds.includes("confirmed_escape")) {
    return "E";
  }
  if (marker.kinds.includes("possible_escape")) {
    return "P";
  }
  if (marker.kinds.includes("unknown_outcome")) {
    return "?";
  }
  if (marker.kinds.includes("threat")) {
    return "L";
  }
  return "";
}

function winnerSide(entry: AnalysisEntry): Side | null {
  if (entry.match_report.result === "black_won") {
    return "Black";
  }
  if (entry.match_report.result === "white_won") {
    return "White";
  }
  return sideFromValue(entry.match_report.winner ?? "");
}

function sideFromValue(value: string): Side | null {
  const normalized = value.trim().toLowerCase();
  if (normalized === "black") {
    return "Black";
  }
  if (normalized === "white") {
    return "White";
  }
  return null;
}

function opponentSide(side: Side): Side {
  return side === "Black" ? "White" : "Black";
}

function actualMoveFromFrame(frame: ProofFrame): ActualMove | null {
  const side = sideFromValue(frame.side_to_move);
  if (!side || !frame.move_played_notation) {
    return null;
  }
  return {
    notation: frame.move_played_notation,
    side,
  };
}

function perspectiveProofStatusLabel(frame: ProofFrame, winner: Side): string {
  const side = sideFromValue(frame.side_to_move);
  if (frame.status === "forced_win") {
    if (side === winner) {
      return frame.lethal_onset_reached ? "guaranteed win" : "forced win";
    }
    return frame.lethal_onset_reached ? "guaranteed loss" : "forced loss";
  }
  if (frame.status === "escape_found") {
    return side === winner ? "win not forced" : "can escape";
  }
  return proofStatusLabel(frame.status);
}

function proofStatusLabel(value: string): string {
  return value.replace(/_/g, " ");
}

function replyRoleLabel(value: string): string {
  switch (value) {
    case "immediate_defense":
      return "immediate";
    case "imminent_defense":
      return "imminent";
    case "offensive_counter":
      return "counter";
    case "corridor_entry":
      return "corridor";
    default:
      return value.replace(/_/g, " ");
  }
}

function replyOutcomeLabel(value: string): string {
  return value.replace(/_/g, " ");
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
