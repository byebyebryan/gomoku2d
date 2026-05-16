import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useStore } from "zustand";

import { Board } from "../components/Board/Board";
import { cloudAuthStore } from "../cloud/auth_store";
import { cloudHistoryStore } from "../cloud/cloud_history_store";
import { cloudProfileStore } from "../cloud/cloud_profile_store";
import type { LocalMatchResumeSeed } from "../game/local_match_store";
import type { CellPosition } from "../game/types";
import { savedMatchPlayers } from "../match/saved_match";
import { resolveActiveHistory } from "../profile/active_history";
import { localProfileStore } from "../profile/local_profile_store";
import {
  buildLocalReplayFrame,
  canResumeReplay,
  defaultReplayMoveIndex,
  replayResumeUndoFloor,
  replayUndoFloor,
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
  const cloudAuth = useStore(cloudAuthStore, (state) => state);
  const cloudHistory = useStore(cloudHistoryStore, (state) => state);
  const cloudProfile = useStore(cloudProfileStore, (state) => state);
  const localHistory = useStore(localProfileStore, (state) => state.matchHistory.replayMatches);
  const localProfile = useStore(localProfileStore, (state) => state.profile);
  const [moveIndex, setMoveIndex] = useState(defaultReplayMoveIndex(0));
  const [autoplaying, setAutoplaying] = useState(false);
  const [coreWinningCells, setCoreWinningCells] = useState<CellPosition[]>([]);

  useEffect(() => {
    localProfileStore.getState().ensureLocalProfile();
  }, []);

  useEffect(() => {
    cloudAuthStore.getState().start();

    return () => {
      cloudAuthStore.getState().stop();
    };
  }, []);

  useEffect(() => {
    if (cloudAuth.status === "signed_in" && cloudAuth.user) {
      void cloudProfileStore.getState().loadForUser(cloudAuth.user, localProfileStore.getState().settings.preferredVariant);
    } else {
      cloudProfileStore.getState().reset();
    }
  }, [cloudAuth.status, cloudAuth.user]);

  useEffect(() => {
    if (cloudAuth.status === "signed_in" && cloudAuth.user && cloudProfile.status === "ready") {
      void cloudHistoryStore.getState().loadForUser(cloudAuth.user, cloudProfile.profile?.resetAt ?? null);
    }
  }, [cloudAuth.status, cloudAuth.user, cloudProfile.profile?.resetAt, cloudProfile.status]);

  const cloudCache =
    cloudAuth.status === "signed_in" && cloudAuth.user
      ? cloudHistory.users[cloudAuth.user.uid]?.cachedMatches ?? []
      : [];
  const history = resolveActiveHistory({
    cloudHistory: cloudCache,
    historyResetAt: cloudAuth.status === "signed_in" ? cloudProfile.profile?.resetAt : null,
    localHistory,
  });
  const match = history.find((entry) => entry.id === matchId) ?? null;
  const localDisplayName = localProfile?.displayName ?? cloudAuth.user?.displayName ?? "Guest";
  const replayFloor = match ? replayUndoFloor(match) : 0;

  useEffect(() => {
    setMoveIndex(defaultReplayMoveIndex(match?.move_count ?? 0, replayFloor));
    setAutoplaying(false);
    setCoreWinningCells([]);
  }, [match?.move_count, matchId, replayFloor]);

  useEffect(() => {
    if (!match || !autoplaying) {
      return undefined;
    }

    if (moveIndex >= match.move_count) {
      setAutoplaying(false);
      return undefined;
    }

    const timer = window.setTimeout(() => {
      setMoveIndex((current) => Math.min(match.move_count, current + 1));
    }, AUTOPLAY_DELAY_MS);

    return () => {
      window.clearTimeout(timer);
    };
  }, [autoplaying, match, moveIndex]);

  useEffect(() => {
    let cancelled = false;

    if (!match || moveIndex !== match.move_count) {
      setCoreWinningCells([]);
      return () => {
        cancelled = true;
      };
    }

    void import("../replay/local_replay_core").then(({ winningCellsFromCore }) => {
      if (!cancelled) {
        setCoreWinningCells(winningCellsFromCore(match));
      }
    });

    return () => {
      cancelled = true;
    };
  }, [match, moveIndex]);

  if (!match) {
    return (
      <main className={styles.page}>
        <section className={`${styles.notFound} uiPanel`}>
          <h1 className={styles.title}>Replay unavailable</h1>
          <p className={styles.notFoundText}>This replay is no longer available.</p>
          <Link className="uiAction uiActionSecondary" to="/profile">
            <Icon className="uiIconDesktop" name="profile" />
            <span className="uiActionLabel">Back to Profile</span>
          </Link>
        </section>
      </main>
    );
  }

  const frame = buildLocalReplayFrame(match, moveIndex, () => coreWinningCells);
  const resumeSeed: LocalMatchResumeSeed = {
    currentPlayer: frame.currentPlayer,
    moves: frame.moves.map((move) => ({ ...move })),
    undoFloor: replayResumeUndoFloor(match, frame),
    variant: match.variant,
  };

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerCopy}>
          <p className="uiPageEyebrow">Saved match</p>
          <h1 className={styles.title}>Replay</h1>
        </div>
        <div className={styles.headerActions}>
          <Link aria-label="Profile" className="uiAction uiActionSecondary" to="/profile">
            <Icon className="uiIconDesktop" name="profile" />
            <span className="uiActionLabel">Profile</span>
          </Link>
          <Link aria-label="Home" className="uiAction uiActionNeutral" to="/">
            <Icon className="uiIconDesktop" name="home" />
            <span className="uiActionLabel">Home</span>
          </Link>
        </div>
      </header>

      <section className={styles.layout}>
        <div className={styles.boardPanel}>
          <Board
            cells={frame.cells}
            counterThreatMoves={[]}
            currentPlayer={frame.currentPlayer}
            forbiddenMoves={[]}
            imminentThreatMoves={[]}
            interactive={false}
            lastMove={frame.lastMove}
            moves={frame.moves}
            onAdvanceRound={() => undefined}
            onPlace={() => undefined}
            onTouchCandidateChange={() => undefined}
            touchControlMode="none"
            touchCandidateResetVersion={0}
            showSequenceNumbers={shouldShowReplaySequenceNumbers(frame)}
            status={frame.status}
            threatMoves={[]}
            winningMoves={[]}
            winningCells={frame.winningCells}
          />
        </div>

        <aside className={styles.deck}>
          <section className={`${styles.deckSection} ${styles.resultSection}`}>
            <p className="uiSectionLabel">Result</p>
            <p className={styles.statusText} data-testid="replay-result">
              {replayWinnerLabel(match, localDisplayName)}
            </p>
          </section>

          <div className="uiDivider" />

          <section className={`${styles.deckSection} ${styles.matchSection}`}>
            <p className={`uiSectionLabel ${styles.matchLabel}`}>Match</p>
            <div className={styles.metaRows}>
              <div className={`${styles.metaRow} ${styles.ruleRow}`}>
                <span className={styles.metaLabel}>Rule</span>
                <span className={styles.metaValue} data-testid="replay-rule">
                  {variantLabel(match.variant)}
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

          <div className="uiDivider" />

          <section className={`${styles.deckSection} ${styles.playbackSection}`}>
            <div className={styles.playbackHeader}>
              <p className={`uiSectionLabel ${styles.playbackLabel}`}>Playback</p>
            </div>

            <div className={styles.timeline}>
              <input
                aria-label="Replay timeline"
                className={styles.timelineInput}
                max={match.move_count}
                min={0}
                onChange={(event) => {
                  setAutoplaying(false);
                  setMoveIndex(Number(event.target.value));
                }}
                style={
                  {
                    "--timeline-progress":
                      match.move_count === 0 ? "0%" : `${(frame.moveIndex / match.move_count) * 100}%`,
                  } as React.CSSProperties
                }
                type="range"
                value={frame.moveIndex}
              />
            </div>

            <div className={styles.controlsRow} data-testid="replay-step-controls">
              <button
                aria-label="Start"
                className="uiAction uiActionNeutral uiActionIconOnly"
                onClick={() => {
                  setAutoplaying(false);
                  setMoveIndex(replayStartMoveIndex(match.move_count));
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
                  setMoveIndex((current) => Math.min(match.move_count, current + 1));
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
                  setMoveIndex(match.move_count);
                }}
                type="button"
              >
                <Icon name="last" />
              </button>
            </div>

            <button
              className={`uiAction uiActionSecondary ${styles.resumeAction}`}
              disabled={!canResumeReplay(frame, replayFloor)}
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
