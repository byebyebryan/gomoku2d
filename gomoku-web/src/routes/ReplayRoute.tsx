import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { useStore } from "zustand";

import { Board } from "../components/Board/Board";
import { guestProfileStore } from "../profile/guest_profile_store";
import {
  buildLocalReplayFrame,
  replayPlayerLabel,
  replayWinnerLabel,
  shouldShowReplaySequenceNumbers,
  variantLabel,
} from "../replay/local_replay";

import styles from "./ReplayRoute.module.css";

const AUTOPLAY_DELAY_MS = 700;

function moveCountLabel(moveIndex: number, totalMoves: number): string {
  return `Move ${moveIndex} / ${totalMoves}`;
}

export function ReplayRoute() {
  const { matchId } = useParams<{ matchId: string }>();
  const history = useStore(guestProfileStore, (state) => state.history);
  const profile = useStore(guestProfileStore, (state) => state.profile);
  const [moveIndex, setMoveIndex] = useState(0);
  const [autoplaying, setAutoplaying] = useState(false);

  useEffect(() => {
    guestProfileStore.getState().ensureGuestProfile();
  }, []);

  const match = history.find((entry) => entry.id === matchId) ?? null;
  const guestDisplayName = profile?.displayName ?? "Guest";

  useEffect(() => {
    setMoveIndex(0);
    setAutoplaying(false);
  }, [matchId]);

  useEffect(() => {
    if (!match || !autoplaying) {
      return undefined;
    }

    if (moveIndex >= match.moves.length) {
      setAutoplaying(false);
      return undefined;
    }

    const timer = window.setTimeout(() => {
      setMoveIndex((current) => Math.min(match.moves.length, current + 1));
    }, AUTOPLAY_DELAY_MS);

    return () => {
      window.clearTimeout(timer);
    };
  }, [autoplaying, match, moveIndex]);

  if (!match) {
    return (
      <main className={styles.page}>
        <section className={`${styles.notFound} uiPanel`}>
          <h1 className={styles.title}>Replay unavailable</h1>
          <p className={styles.notFoundText}>This replay is no longer stored on this device.</p>
          <Link className="uiAction uiActionSecondary" to="/profile">
            Back to Profile
          </Link>
        </section>
      </main>
    );
  }

  const frame = buildLocalReplayFrame(match, moveIndex);

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div>
          <p className="uiPageEyebrow">Match replay</p>
          <h1 className={styles.title}>Replay</h1>
          <p className={styles.summary}>{replayPlayerLabel(match, guestDisplayName)}</p>
        </div>
        <div className={styles.headerActions}>
          <Link className="uiAction uiActionSecondary" to="/profile">
            Profile
          </Link>
          <Link className="uiAction uiActionAccent" to="/">
            Home
          </Link>
        </div>
      </header>

      <section className={styles.layout}>
        <div className={styles.boardPanel}>
          <Board
            cells={frame.cells}
            currentPlayer={frame.currentPlayer}
            forbiddenMoves={[]}
            interactive={false}
            lastMove={frame.lastMove}
            moves={frame.moves}
            onAdvanceRound={() => undefined}
            onPlace={() => undefined}
            showSequenceNumbers={shouldShowReplaySequenceNumbers(frame)}
            status={frame.status}
            threatMoves={[]}
            winningMoves={[]}
            winningCells={frame.winningCells}
          />
        </div>

        <aside className={`uiPanel ${styles.deck}`}>
          <section className={styles.deckSection}>
            <p className="uiSectionLabel">Result</p>
            <p className={styles.resultText} data-testid="replay-result">
              {replayWinnerLabel(match, guestDisplayName)}
            </p>
            <p className={styles.metaLine}>
              <span data-testid="replay-move-count">{moveCountLabel(frame.moveIndex, match.moves.length)}</span>
              <span aria-hidden="true">·</span>
              <span data-testid="replay-rule">{variantLabel(match.variant)}</span>
            </p>
          </section>

          <div className="uiDivider" />

          <section className={styles.deckSection}>
            <div className={styles.playbackHeader}>
              <p className="uiSectionLabel">Playback</p>
              <button
                className="uiAction uiActionPrimary"
                onClick={() => {
                  setAutoplaying((current) => !current);
                }}
                type="button"
              >
                {autoplaying ? "Pause" : "Auto play"}
              </button>
            </div>

            <div className={styles.controlsRow} data-testid="replay-step-controls">
              <button
                className="uiAction uiActionNeutral"
                onClick={() => {
                  setAutoplaying(false);
                  setMoveIndex(0);
                }}
                type="button"
              >
                Start
              </button>
              <button
                className="uiAction uiActionNeutral"
                onClick={() => {
                  setAutoplaying(false);
                  setMoveIndex(match.moves.length);
                }}
                type="button"
              >
                End
              </button>
              <button
                className="uiAction uiActionNeutral"
                onClick={() => {
                  setAutoplaying(false);
                  setMoveIndex((current) => Math.max(0, current - 1));
                }}
                type="button"
              >
                Previous move
              </button>
              <button
                className="uiAction uiActionNeutral"
                onClick={() => {
                  setAutoplaying(false);
                  setMoveIndex((current) => Math.min(match.moves.length, current + 1));
                }}
                type="button"
              >
                Next move
              </button>
            </div>

            <label className={styles.timeline}>
              <span className={styles.timelineLabel}>Replay timeline</span>
              <input
                aria-label="Replay timeline"
                className={styles.timelineInput}
                max={match.moves.length}
                min={0}
                onChange={(event) => {
                  setAutoplaying(false);
                  setMoveIndex(Number(event.target.value));
                }}
                style={
                  {
                    "--timeline-progress":
                      match.moves.length === 0 ? "0%" : `${(frame.moveIndex / match.moves.length) * 100}%`,
                  } as React.CSSProperties
                }
                type="range"
                value={frame.moveIndex}
              />
            </label>
          </section>
        </aside>
      </section>
    </main>
  );
}
