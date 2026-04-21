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
        <section className={styles.notFound}>
          <h1 className={styles.title}>Replay unavailable</h1>
          <p className={styles.notFoundText}>This replay is no longer stored on this device.</p>
          <Link className={styles.secondaryAction} to="/profile">
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
          <p className={styles.eyebrow}>Match replay</p>
          <h1 className={styles.title}>Replay</h1>
          <p className={styles.summary}>{replayPlayerLabel(match, guestDisplayName)}</p>
        </div>
        <div className={styles.headerActions}>
          <Link className={`${styles.secondaryAction} ${styles.infoAction}`} to="/profile">
            Profile
          </Link>
          <Link className={`${styles.secondaryAction} ${styles.accentAction} ${styles.homeAction}`} to="/">
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

        <aside className={styles.sidebar}>
          <section className={styles.card}>
            <p className={styles.sectionLabel}>Result</p>
            <p className={styles.resultText}>{replayWinnerLabel(match, guestDisplayName)}</p>
            <p className={styles.moveCount}>{moveCountLabel(frame.moveIndex, match.moves.length)}</p>
            <p className={styles.moveCount}>Rules: {variantLabel(match.variant)}</p>
          </section>

          <section className={styles.card}>
            <div className={styles.controlsHeader}>
              <p className={styles.sectionLabel}>Playback</p>
              <button
                className={`${styles.secondaryAction} ${styles.successAction}`}
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
                className={styles.iconAction}
                onClick={() => {
                  setAutoplaying(false);
                  setMoveIndex(0);
                }}
                type="button"
              >
                Start
              </button>
              <button
                className={styles.iconAction}
                onClick={() => {
                  setAutoplaying(false);
                  setMoveIndex(match.moves.length);
                }}
                type="button"
              >
                End
              </button>
              <button
                className={styles.iconAction}
                onClick={() => {
                  setAutoplaying(false);
                  setMoveIndex((current) => Math.max(0, current - 1));
                }}
                type="button"
              >
                Previous move
              </button>
              <button
                className={styles.iconAction}
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

          <section className={styles.card}>
            <div className={styles.historyHeader}>
              <p className={styles.sectionLabel}>Moves</p>
              <p className={styles.historyCount}>{match.moves.length} total</p>
            </div>
            {match.moves.length === 0 ? (
              <p className={styles.emptyState}>No moves recorded.</p>
            ) : (
              <ol className={styles.historyList}>
                {match.moves.map((move) => {
                  const active = frame.moveIndex === move.moveNumber;
                  return (
                    <li className={styles.historyItem} key={move.moveNumber}>
                      <button
                        className={active ? `${styles.historyButton} ${styles.historyButtonActive}` : styles.historyButton}
                        onClick={() => {
                          setAutoplaying(false);
                          setMoveIndex(move.moveNumber);
                        }}
                        type="button"
                      >
                        <span>M{move.moveNumber}</span>
                        <span>{move.player === 1 ? "Black" : "White"}</span>
                        <span>
                          {move.row + 1},{move.col + 1}
                        </span>
                      </button>
                    </li>
                  );
                })}
              </ol>
            )}
          </section>
        </aside>
      </section>
    </main>
  );
}
