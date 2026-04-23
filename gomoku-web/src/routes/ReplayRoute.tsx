import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useStore } from "zustand";

import { Board } from "../components/Board/Board";
import type { LocalMatchResumeSeed } from "../game/local_match_store";
import { guestProfileStore } from "../profile/guest_profile_store";
import {
  buildLocalReplayFrame,
  canResumeReplay,
  defaultReplayMoveIndex,
  replayPlayerName,
  replayWinnerLabel,
  replayStartMoveIndex,
  shouldShowReplaySequenceNumbers,
  variantLabel,
} from "../replay/local_replay";
import { Icon } from "../ui/Icon";

import styles from "./ReplayRoute.module.css";

const AUTOPLAY_DELAY_MS = 700;

function moveCountLabel(moveIndex: number, totalMoves: number): string {
  return `Move ${moveIndex} / ${totalMoves}`;
}

export function ReplayRoute() {
  const { matchId } = useParams<{ matchId: string }>();
  const navigate = useNavigate();
  const history = useStore(guestProfileStore, (state) => state.history);
  const profile = useStore(guestProfileStore, (state) => state.profile);
  const [moveIndex, setMoveIndex] = useState(defaultReplayMoveIndex(0));
  const [autoplaying, setAutoplaying] = useState(false);

  useEffect(() => {
    guestProfileStore.getState().ensureGuestProfile();
  }, []);

  const match = history.find((entry) => entry.id === matchId) ?? null;
  const guestDisplayName = profile?.displayName ?? "Guest";

  useEffect(() => {
    setMoveIndex(defaultReplayMoveIndex(match?.moves.length ?? 0));
    setAutoplaying(false);
  }, [match?.moves.length, matchId]);

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
            <Icon className="uiIconDesktop" name="profile" />
            <span className="uiActionLabel">Back to Profile</span>
          </Link>
        </section>
      </main>
    );
  }

  const frame = buildLocalReplayFrame(match, moveIndex);
  const resumeSeed: LocalMatchResumeSeed = {
    currentPlayer: frame.currentPlayer,
    moves: frame.moves.map((move) => ({ ...move })),
    variant: match.variant,
  };

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div>
          <p className="uiPageEyebrow">Match replay</p>
          <h1 className={styles.title}>Replay</h1>
        </div>
        <div className={styles.headerActions}>
          <Link className="uiAction uiActionSecondary" to="/profile">
            <Icon className="uiIconDesktop" name="profile" />
            <span className="uiActionLabel">Profile</span>
          </Link>
          <Link className="uiAction uiActionNeutral" to="/">
            <Icon className="uiIconDesktop" name="home" />
            <span className="uiActionLabel">Home</span>
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

        <aside className={styles.deck}>
          <section className={styles.deckSection}>
            <p className="uiSectionLabel">Result</p>
            <p className={styles.statusText} data-testid="replay-result">
              {replayWinnerLabel(match, guestDisplayName)}
            </p>
          </section>

          <div className="uiDivider" />

          <section className={styles.deckSection}>
            <p className="uiSectionLabel">Match</p>
            <div className={styles.metaRows}>
              <div className={styles.metaRow}>
                <span className={styles.metaLabel}>Rule</span>
                <span className={styles.metaValue} data-testid="replay-rule">
                  {variantLabel(match.variant)}
                </span>
              </div>
              <div className={styles.metaRow}>
                <span className={styles.metaLabel}>Move</span>
                <span className={styles.metaValue} data-testid="replay-move-count">
                  {moveCountLabel(frame.moveIndex, match.moves.length)}
                </span>
              </div>
            </div>
            <div className={styles.playerRows}>
              {match.players.map((player, index) => {
                const active = frame.status === "playing" && frame.currentPlayer === index + 1;

                return (
                  <article
                    className={[
                      styles.playerRow,
                      player.stone === "black" ? styles.playerRowBlack : styles.playerRowWhite,
                      active ? styles.playerRowActive : "",
                    ].join(" ").trim()}
                    data-testid={`replay-player-row-${player.stone}`}
                    key={player.stone}
                  >
                    <div className={styles.playerCopy}>
                      <div className={styles.playerHead}>
                        <h2 className={styles.playerName}>{replayPlayerName(player, guestDisplayName)}</h2>
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

          <div className="uiDivider" />

          <section className={styles.deckSection}>
            <div className={styles.playbackHeader}>
              <p className="uiSectionLabel">Playback</p>
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

            <div className={styles.controlsRow} data-testid="replay-step-controls">
              <button
                aria-label="Start"
                className="uiAction uiActionNeutral uiActionIconOnly"
                onClick={() => {
                  setAutoplaying(false);
                  setMoveIndex(replayStartMoveIndex(match.moves.length));
                }}
                type="button"
              >
                <Icon name="first" />
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
                <Icon name="fastRewind" />
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
                  setMoveIndex((current) => Math.min(match.moves.length, current + 1));
                }}
                type="button"
              >
                <Icon name="fastForward" />
              </button>
              <button
                aria-label="End"
                className="uiAction uiActionNeutral uiActionIconOnly"
                onClick={() => {
                  setAutoplaying(false);
                  setMoveIndex(match.moves.length);
                }}
                type="button"
              >
                <Icon name="last" />
              </button>
            </div>

            <button
              className={`uiAction uiActionSecondary ${styles.resumeAction}`}
              disabled={!canResumeReplay(frame)}
              onClick={() => {
                navigate("/match/local", { state: { resumeSeed } });
              }}
              type="button"
            >
              <Icon className="uiIconDesktop" name="plus" />
              <span className="uiActionLabel">Play From Here</span>
            </button>
          </section>
        </aside>
      </section>
    </main>
  );
}
