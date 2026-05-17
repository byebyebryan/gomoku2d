import { useEffect, useRef, useState } from "react";
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
  nextReplayTurnMoveIndex,
  previousReplayTurnMoveIndex,
  replayResumeUndoFloor,
  replayUndoFloor,
  replayPlayerName,
  shouldShowReplaySequenceNumbers,
  variantLabel,
} from "../replay/local_replay";
import {
  analysisOverlaysForFrame,
  mergeReplayAnalysisAnnotations,
  nextReplayMove,
  replayAnalysisStatusSummary,
  replayTimelineAnalysis,
  type ReplayAnalysisAnnotationsByPly,
} from "../replay/replay_analysis_overlays";
import type { ReplayAnalysisStepResult } from "../replay/replay_analysis_protocol";
import { ReplayAnalysisRunner } from "../replay/replay_analysis_runner";
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
  const [analysisAnnotations, setAnalysisAnnotations] = useState<ReplayAnalysisAnnotationsByPly>({});
  const [analysisStep, setAnalysisStep] = useState<ReplayAnalysisStepResult | null>(null);
  const [coreWinningCells, setCoreWinningCells] = useState<CellPosition[]>([]);
  const analysisRunnerRef = useRef<ReplayAnalysisRunner | null>(null);

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
      void cloudProfileStore.getState().loadForUser(cloudAuth.user, localProfileStore.getState().settings);
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
    setMoveIndex(defaultReplayMoveIndex(match?.move_count ?? 0));
    setAutoplaying(false);
    setAnalysisAnnotations({});
    setAnalysisStep(null);
    setCoreWinningCells([]);
  }, [match?.move_count, matchId]);

  useEffect(() => {
    return () => {
      analysisRunnerRef.current?.cancel();
      analysisRunnerRef.current?.dispose();
      analysisRunnerRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (!match) {
      analysisRunnerRef.current?.cancel();
      setAnalysisAnnotations({});
      setAnalysisStep(null);
      return undefined;
    }

    const mergeStep = (step: ReplayAnalysisStepResult) => {
      setAnalysisStep(step);
      setAnalysisAnnotations((current) => mergeReplayAnalysisAnnotations(current, step));
    };

    try {
      let runner = analysisRunnerRef.current;
      if (!runner) {
        runner = new ReplayAnalysisRunner();
        analysisRunnerRef.current = runner;
      }

      runner.analyze(
        match,
        {
          onComplete: mergeStep,
          onError: (error) => {
            setAnalysisAnnotations({});
            setAnalysisStep({
              analysis: null,
              annotations: [],
              counters: { branch_roots: 0, prefixes_analyzed: 0, proof_nodes: 0 },
              current_ply: null,
              done: true,
              error: error.message,
              schema_version: 1,
              status: "error",
            });
          },
          onProgress: mergeStep,
        },
        { maxDepth: 4, maxScanPlies: 64 },
        1,
      );
    } catch {
      setAnalysisAnnotations({});
      setAnalysisStep({
        analysis: null,
        annotations: [],
        counters: { branch_roots: 0, prefixes_analyzed: 0, proof_nodes: 0 },
        current_ply: null,
        done: true,
        error: "Replay analyzer could not start",
        schema_version: 1,
        status: "error",
      });
    }

    return () => {
      analysisRunnerRef.current?.cancel();
    };
  }, [match]);

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

    void import("../replay/local_replay_core")
      .then(({ winningCellsFromCore }) => {
        if (!cancelled) {
          setCoreWinningCells(winningCellsFromCore(match));
        }
      })
      .catch(() => {
        if (!cancelled) {
          setCoreWinningCells([]);
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
  const analysisOverlays = analysisOverlaysForFrame(analysisAnnotations, match, frame.moveIndex);
  const analysisStatus = replayAnalysisStatusSummary(analysisStep, analysisAnnotations, match, frame);
  const timelineAnalysis = replayTimelineAnalysis(analysisAnnotations, match.move_count);
  const timelineStyle = {
    "--timeline-corridor-end": timelineAnalysis.corridorEndPercent ?? "0%",
    "--timeline-corridor-start": timelineAnalysis.corridorStartPercent ?? "0%",
    "--timeline-escape": timelineAnalysis.escapePercent ?? "0%",
  } as React.CSSProperties;
  const replayMovePreview = nextReplayMove(match, frame.moveIndex);
  const resumeSeed: LocalMatchResumeSeed = {
    currentPlayer: frame.currentPlayer,
    moves: frame.moves.map((move) => ({ ...move })),
    undoFloor: replayResumeUndoFloor(match, frame),
    variant: match.ruleset,
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
            analysisOverlays={analysisOverlays}
            cells={frame.cells}
            counterThreatMoves={[]}
            currentPlayer={frame.currentPlayer}
            forbiddenMoves={[]}
            imminentThreatMoves={[]}
            interactive={false}
            lastMove={frame.lastMove}
            moves={frame.moves}
            nextReplayMove={replayMovePreview}
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
          <section className={`${styles.deckSection} ${styles.statusSection}`}>
            <p className="uiSectionLabel">Status</p>
            <p className={styles.statusText} data-testid="replay-analysis-status">
              {analysisStatus.label}
            </p>
            <p className={styles.statusDetail} data-testid="replay-analysis-detail">
              {analysisStatus.detail}
            </p>
          </section>

          <div className="uiDivider" />

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

          <div className="uiDivider" />

          <section className={`${styles.deckSection} ${styles.playbackSection}`}>
            <div className={styles.playbackHeader}>
              <p className={`uiSectionLabel ${styles.playbackLabel}`}>Playback</p>
            </div>

            <div className={styles.timeline} data-testid="replay-timeline" style={timelineStyle}>
              <div aria-hidden="true" className={styles.timelineTrack}>
                {timelineAnalysis.corridorStartPercent && timelineAnalysis.corridorEndPercent ? (
                  <span className={styles.timelineCorridor} data-testid="replay-timeline-corridor" />
                ) : null}
                {timelineAnalysis.escapePercent ? (
                  <span className={styles.timelineEscape} data-testid="replay-timeline-escape" />
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
