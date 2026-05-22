import type React from "react";
import type { Dispatch, SetStateAction } from "react";

import { Board } from "../components/Board/Board";
import type { BoardViewModel } from "../board/board_model";
import type { LocalMatchResumeSeed } from "../game/local_match_store";
import { savedMatchPlayers, type SavedMatchV2 } from "../match/saved_match";
import {
  buildLocalReplayFrame,
  canResumeReplay,
  nextReplayTurnMoveIndex,
  previousReplayTurnMoveIndex,
  replayResumeUndoFloor,
  replayPlayerName,
  variantLabel,
} from "../replay/local_replay";
import type { replayTimelineAnalysis } from "../replay/replay_analysis_overlays";
import { Icon } from "../ui/Icon";

import styles from "./ReplayRoute.module.css";

type ReplayFrame = ReturnType<typeof buildLocalReplayFrame>;
type ReplayTimelineAnalysis = ReturnType<typeof replayTimelineAnalysis>;

interface ReplayBoardPanelProps {
  model: BoardViewModel;
}

export function ReplayBoardPanel({ model }: ReplayBoardPanelProps) {
  return (
    <div className={styles.boardPanel}>
      <Board model={model} />
    </div>
  );
}

interface ReplayStatusPanelProps {
  detail: string;
  label: string;
}

export function ReplayStatusPanel({ detail, label }: ReplayStatusPanelProps) {
  return (
    <section className={`${styles.deckSection} ${styles.statusSection}`}>
      <p className="uiSectionLabel">Status</p>
      <p className={styles.statusText} data-testid="replay-analysis-status">
        {label}
      </p>
      <p className={styles.statusDetail} data-testid="replay-analysis-detail">
        {detail}
      </p>
    </section>
  );
}

interface ReplayMatchPanelProps {
  frame: ReplayFrame;
  localDisplayName: string;
  match: SavedMatchV2;
}

export function ReplayMatchPanel({ frame, localDisplayName, match }: ReplayMatchPanelProps) {
  return (
    <section className={`${styles.deckSection} ${styles.matchSection}`}>
      <p className={`uiSectionLabel ${styles.matchLabel}`}>Match</p>
      <div className={styles.metaRows}>
        <div className={`${styles.metaRow} ${styles.ruleRow}`}>
          <span className={styles.metaLabel}>Rule</span>
          <span className={styles.metaValue} data-testid="replay-rule">
            {variantLabel(match.ruleset)}
          </span>
        </div>
        <div className={`${styles.metaRow} ${styles.moveRow}`}>
          <span className={styles.metaLabel}>Move</span>
          <span className={styles.metaValue} data-testid="replay-move-count">
            {moveCountLabel(frame.moveIndex, match.move_count)}
          </span>
        </div>
      </div>
      <div className={styles.playerRows}>
        {savedMatchPlayers(match).map(({ player, side }) => {
          const active = frame.status === "playing" && frame.currentPlayer === (side === "black" ? 1 : 2);

          return (
            <article
              className={[
                styles.playerRow,
                side === "black" ? styles.playerRowBlack : styles.playerRowWhite,
                active ? styles.playerRowActive : "",
              ].join(" ").trim()}
              data-testid={`replay-player-row-${side}`}
              key={side}
            >
              <div className={styles.playerCopy}>
                <div className={styles.playerHead}>
                  <h2 className={styles.playerName}>{replayPlayerName(player, localDisplayName)}</h2>
                  <span
                    aria-label={player.kind === "human" ? "Player" : "Bot"}
                    className={styles.playerKindIcon}
                    role="img"
                  >
                    <Icon name={player.kind === "human" ? "human" : "bot"} />
                  </span>
                </div>
              </div>
            </article>
          );
        })}
      </div>
    </section>
  );
}

interface ReplayPlaybackPanelProps {
  autoplaying: boolean;
  frame: ReplayFrame;
  match: SavedMatchV2;
  onResume: (seed: LocalMatchResumeSeed) => void;
  replayFloor: number;
  setAutoplaying: Dispatch<SetStateAction<boolean>>;
  setMoveIndex: Dispatch<SetStateAction<number>>;
  timelineAnalysis: ReplayTimelineAnalysis;
  timelineStyle: React.CSSProperties;
}

export function ReplayPlaybackPanel({
  autoplaying,
  frame,
  match,
  onResume,
  replayFloor,
  setAutoplaying,
  setMoveIndex,
  timelineAnalysis,
  timelineStyle,
}: ReplayPlaybackPanelProps) {
  return (
    <section className={`${styles.deckSection} ${styles.playbackSection}`}>
      <div className={styles.playbackHeader}>
        <p className={`uiSectionLabel ${styles.playbackLabel}`}>Playback</p>
      </div>

      <div className={styles.timeline} data-testid="replay-timeline" style={timelineStyle}>
        <div aria-hidden="true" className={styles.timelineTrack}>
          {timelineAnalysis.analyzedStartPercent && timelineAnalysis.analyzedEndPercent ? (
            <span className={styles.timelineAnalyzed} data-testid="replay-timeline-analyzed" />
          ) : null}
          {timelineAnalysis.setupStartPercent && timelineAnalysis.setupEndPercent ? (
            <span className={styles.timelineSetup} data-testid="replay-timeline-setup-corridor" />
          ) : null}
          {timelineAnalysis.lethalTailStartPercent && timelineAnalysis.lethalTailEndPercent ? (
            <span className={styles.timelineLethalTail} data-testid="replay-timeline-lethal-tail" />
          ) : null}
          {timelineAnalysis.escapePercent ? (
            <span className={styles.timelineEscape} data-testid="replay-timeline-escape" />
          ) : null}
          {timelineAnalysis.lethalOnsetPercent ? (
            <span className={styles.timelineLethalOnset} data-testid="replay-timeline-lethal-onset" />
          ) : null}
        </div>
        <input
          aria-label="Replay timeline"
          className={styles.timelineInput}
          max={match.move_count}
          min={0}
          onChange={(event) => {
            setAutoplaying(false);
            setMoveIndex(Number(event.target.value));
          }}
          type="range"
          value={frame.moveIndex}
        />
      </div>

      <div className={styles.controlsRow} data-testid="replay-step-controls">
        <button
          aria-label="Previous turn"
          className="uiAction uiActionNeutral uiActionIconOnly"
          onClick={() => {
            setAutoplaying(false);
            setMoveIndex((current) => previousReplayTurnMoveIndex(current));
          }}
          type="button"
        >
          <Icon name="doublePrev" />
        </button>
        <button
          aria-label="Previous move"
          className="uiAction uiActionNeutral uiActionIconOnly"
          onClick={() => {
            setAutoplaying(false);
            setMoveIndex((current) => Math.max(0, current - 1));
          }}
          type="button"
        >
          <Icon name="prev" />
        </button>
        <button
          aria-label={autoplaying ? "Pause" : "Auto play"}
          className="uiAction uiActionPrimary uiActionIconOnly"
          onClick={() => {
            setAutoplaying((current) => !current);
          }}
          type="button"
        >
          <Icon name={autoplaying ? "pause" : "play"} />
        </button>
        <button
          aria-label="Next move"
          className="uiAction uiActionNeutral uiActionIconOnly"
          onClick={() => {
            setAutoplaying(false);
            setMoveIndex((current) => Math.min(match.move_count, current + 1));
          }}
          type="button"
        >
          <Icon name="next" />
        </button>
        <button
          aria-label="Next turn"
          className="uiAction uiActionNeutral uiActionIconOnly"
          onClick={() => {
            setAutoplaying(false);
            setMoveIndex((current) => nextReplayTurnMoveIndex(current, match.move_count));
          }}
          type="button"
        >
          <Icon name="doubleNext" />
        </button>
      </div>

      <div className={styles.replayActions}>
        <button
          className={`uiAction uiActionSecondary ${styles.resumeAction}`}
          disabled={!canResumeReplay(frame, replayFloor)}
          onClick={() => {
            onResume(resumeSeedFromFrame(match, frame));
          }}
          type="button"
        >
          <Icon className="uiIconDesktop" name="plus" />
          <span className="uiActionLabel">Play From Here</span>
        </button>
      </div>
    </section>
  );
}

function resumeSeedFromFrame(match: SavedMatchV2, frame: ReplayFrame): LocalMatchResumeSeed {
  return {
    currentPlayer: frame.currentPlayer,
    moves: frame.moves.map((move) => ({ ...move })),
    undoFloor: replayResumeUndoFloor(match, frame),
    variant: match.ruleset,
  };
}

function moveCountLabel(moveIndex: number, totalMoves: number): string {
  return `Move ${moveIndex} / ${totalMoves}`;
}
