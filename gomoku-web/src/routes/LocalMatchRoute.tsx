import { useEffect, useRef, useState } from "react";
import { Link, useLocation, useNavigationType, useSearchParams } from "react-router-dom";
import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import { buildLocalMatchBoardModel } from "../board/board_model";
import { Board } from "../components/Board/Board";
import { useDocumentTitle } from "../app/useDocumentTitle";
import {
  DEFAULT_BOT_CONFIG,
  botConfigSummary,
  botPlayerName,
  botLabel,
} from "../core/bot_config";
import {
  ensureLocalMatchSession,
  localMatchSessionStore,
} from "../game/local_match_session";
import { canUndoLocalMatch, type LocalMatchState } from "../game/local_match_store";
import type { CellPosition } from "../game/types";
import { localProfileStore } from "../profile/local_profile_store";
import type { BoardHintSettings } from "../profile/profile_settings";
import { variantLabel } from "../replay/local_replay";
import { Icon } from "../ui/Icon";
import { confirmLocalMatchReplacement } from "./confirm_local_match_replacement";
import { sourceReplayToken } from "./source_replay_navigation";

import styles from "./LocalMatchRoute.module.css";

const MOBILE_TOUCH_QUERY =
  "(max-width: 720px) and (orientation: portrait) and (hover: none) and (pointer: coarse)";

function loadingCells(): LocalMatchState["cells"] {
  return Array.from({ length: 15 }, () => Array.from({ length: 15 }, () => null));
}

const loadingMatchStore = createStore<LocalMatchState>(() => ({
  botError: null,
  cells: loadingCells(),
  counterThreatEvidenceCells: [],
  counterThreatMoves: [],
  currentPlayer: 1,
  currentBotConfig: DEFAULT_BOT_CONFIG,
  currentVariant: "freestyle",
  forbiddenMoves: [],
  immediateThreatEvidenceCells: [],
  imminentThreatEvidenceCells: [],
  imminentThreatMoves: [],
  lastMove: null,
  moves: [],
  pendingBotMove: false,
  placeHumanMove: () => false,
  retryBotMove: () => undefined,
  players: [
    { kind: "human", name: "Guest", stone: "black" },
    { kind: "bot", name: botPlayerName(DEFAULT_BOT_CONFIG), stone: "white" },
  ],
  playerClockMs: [0, 0],
  selectedBotConfig: DEFAULT_BOT_CONFIG,
  selectedVariant: "freestyle",
  selectBotConfig: () => undefined,
  selectVariant: () => undefined,
  startNewMatch: () => undefined,
  startNextRound: () => undefined,
  status: "playing",
  threatMoves: [],
  turnStartedAtMs: Date.now(),
  undoFloor: 0,
  undoLastTurn: () => false,
  dispose: () => undefined,
  winningEvidenceCells: [],
  winningMoves: [],
  winningCells: [],
}));

function statusLabel(
  state: Pick<LocalMatchState, "botError" | "currentPlayer" | "pendingBotMove" | "players" | "status">,
): string {
  if (state.status === "black_won" || state.status === "white_won") {
    const winningStone = state.status === "black_won" ? "black" : "white";
    const winner = state.players.find((player) => player.stone === winningStone);
    return `${winner?.name ?? winningStone} wins`;
  }
  if (state.status === "draw") {
    return "Draw";
  }
  if (state.pendingBotMove) {
    return "Thinking";
  }
  if (state.botError) {
    return "Bot paused";
  }

  return `${state.players[state.currentPlayer - 1]?.name ?? "Unknown"} to move`;
}

function moveCountLabel(moveCount: number): string {
  return `Move ${moveCount}`;
}

function clockLabel(ms: number): string {
  const safeMs = Math.max(0, Math.floor(ms));
  if (safeMs < 60_000) {
    return `${(safeMs / 1000).toFixed(1)}s`;
  }

  const totalSeconds = Math.floor(safeMs / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function ActiveTurnClock({ startedAtMs }: { startedAtMs: number }) {
  const [nowMs, setNowMs] = useState(Date.now);

  useEffect(() => {
    setNowMs(Date.now());
    const timer = window.setInterval(() => setNowMs(Date.now()), 100);
    return () => window.clearInterval(timer);
  }, [startedAtMs]);

  return <span className={styles.playerTurnClock}>+{clockLabel(nowMs - startedAtMs)}</span>;
}

function setupChanged(state: Pick<
  LocalMatchState,
  "currentBotConfig" | "currentVariant" | "selectedBotConfig" | "selectedVariant"
>): boolean {
  return state.currentVariant !== state.selectedVariant
    || JSON.stringify(state.currentBotConfig) !== JSON.stringify(state.selectedBotConfig);
}

function nextSetupLabel(
  state: Pick<LocalMatchState, "selectedBotConfig" | "selectedVariant">,
): string {
  return `${variantLabel(state.selectedVariant)} · ${botLabel(state.selectedBotConfig)}`;
}

function visibleBoardHints(
  state: Pick<
    LocalMatchState,
    | "counterThreatEvidenceCells"
    | "counterThreatMoves"
    | "immediateThreatEvidenceCells"
    | "imminentThreatEvidenceCells"
    | "imminentThreatMoves"
    | "threatMoves"
    | "winningEvidenceCells"
    | "winningMoves"
  >,
  settings: BoardHintSettings,
): Pick<
  LocalMatchState,
  | "counterThreatEvidenceCells"
  | "counterThreatMoves"
  | "immediateThreatEvidenceCells"
  | "imminentThreatEvidenceCells"
  | "imminentThreatMoves"
  | "threatMoves"
  | "winningEvidenceCells"
  | "winningMoves"
> {
  const showEvidence = settings.evidence === "on";
  return {
    counterThreatEvidenceCells: showEvidence && settings.imminent === "threat_counter"
      ? state.counterThreatEvidenceCells
      : [],
    counterThreatMoves: settings.imminent === "threat_counter" ? state.counterThreatMoves : [],
    immediateThreatEvidenceCells: showEvidence && settings.immediate === "win_threat"
      ? state.immediateThreatEvidenceCells
      : [],
    imminentThreatEvidenceCells: showEvidence && settings.imminent !== "off"
      ? state.imminentThreatEvidenceCells
      : [],
    imminentThreatMoves: settings.imminent === "off" ? [] : state.imminentThreatMoves,
    threatMoves: settings.immediate === "win_threat" ? state.threatMoves : [],
    winningEvidenceCells: showEvidence && settings.immediate !== "off" ? state.winningEvidenceCells : [],
    winningMoves: settings.immediate === "off" ? [] : state.winningMoves,
  };
}

export function LocalMatchRoute() {
  useDocumentTitle("Local Match");

  const location = useLocation();
  const navigationType = useNavigationType();
  const matchHeading = useRef<HTMLHeadingElement>(null);
  const sourceReplayLink = useRef<HTMLAnchorElement>(null);
  const [searchParams] = useSearchParams();
  const [compactTouchMode, setCompactTouchMode] = useState(false);
  const [touchCandidate, setTouchCandidate] = useState<CellPosition | null>(null);
  const [touchCandidatePlaceable, setTouchCandidatePlaceable] = useState(false);
  const [touchCandidateResetVersion, setTouchCandidateResetVersion] = useState(0);
  const profile = useStore(localProfileStore, (snapshot) => snapshot.profile);
  const matchStore = useStore(localMatchSessionStore, (snapshot) => snapshot.matchStore);
  const latestReplayId = useStore(localMatchSessionStore, (snapshot) => snapshot.latestReplayId);
  const sourceReplay = useStore(localMatchSessionStore, (snapshot) => snapshot.sourceReplay);
  const settings = useStore(localProfileStore, (snapshot) => snapshot.settings);
  const state = useStore(matchStore ?? loadingMatchStore, (snapshot) => snapshot);
  const visibleHints = visibleBoardHints(state, settings.boardHints);
  const mediaMode = searchParams.get("media");
  const suppressSequenceNumbers = mediaMode === "og" || mediaMode === "showcase";

  useEffect(() => {
    localProfileStore.getState().ensureLocalProfile();
  }, []);

  useEffect(() => {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
      return undefined;
    }

    const mediaQuery = window.matchMedia(MOBILE_TOUCH_QUERY);
    const sync = () => setCompactTouchMode(mediaQuery.matches);

    sync();
    mediaQuery.addEventListener("change", sync);
    return () => {
      mediaQuery.removeEventListener("change", sync);
    };
  }, []);

  useEffect(() => {
    if (!profile) {
      return;
    }

    if (!matchStore) {
      ensureLocalMatchSession();
    }
  }, [matchStore, profile]);

  useEffect(() => {
    if (compactTouchMode) {
      return;
    }

    setTouchCandidate(null);
    setTouchCandidatePlaceable(false);
  }, [compactTouchMode]);

  useEffect(() => {
    const token = sourceReplayToken(sourceReplay?.state);
    if (navigationType === "PUSH" && token && location.state?.resumeSourceToken === token) {
      sourceReplayLink.current?.focus();
    }
  }, [location, navigationType, sourceReplay]);

  if (!matchStore) {
    return <main className={styles.page}>Loading match…</main>;
  }

  const humanToMove =
    !state.pendingBotMove &&
    state.status === "playing" &&
    state.players[state.currentPlayer - 1].kind === "human";

  const resetTouchCandidate = () => {
    setTouchCandidate(null);
    setTouchCandidatePlaceable(false);
    setTouchCandidateResetVersion((version) => version + 1);
  };
  const advanceRound = () => {
    const current = matchStore.getState();
    if (current.status === "playing") return;
    resetTouchCandidate();
    current.startNextRound();
  };
  const boardModel = buildLocalMatchBoardModel({
    forbiddenMoves: state.forbiddenMoves,
    hints: visibleHints,
    interaction: {
      interactive: humanToMove,
      kind: "play",
      onAdvanceRound: advanceRound,
      onPlace: state.placeHumanMove,
      onTouchCandidateChange: (candidate, canPlace) => {
        setTouchCandidate((previous) => {
          if (
            previous?.row === candidate?.row &&
            previous?.col === candidate?.col
          ) {
            return previous;
          }

          return candidate;
        });
        setTouchCandidatePlaceable((previous) => (
          previous === canPlace ? previous : canPlace
        ));
      },
      touchCandidateResetVersion,
      touchControlMode: compactTouchMode ? settings.touchControl : "none",
    },
    position: {
      cells: state.cells,
      currentPlayer: state.currentPlayer,
      lastMove: state.lastMove,
      moves: state.moves,
      showSequenceNumbers: !suppressSequenceNumbers,
      status: state.status,
    },
    winningCells: state.winningCells,
  });

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerCopy}>
          <p className="uiPageEyebrow">Solo play</p>
          <h1 className={styles.title} ref={matchHeading} tabIndex={-1}>Local Match</h1>
        </div>
        <div className={styles.headerActions}>
          {state.status === "playing" && <button
            aria-label="New Game"
            className="uiAction uiActionPrimary"
            onClick={() => {
              if (!confirmLocalMatchReplacement()) return;
              resetTouchCandidate();
              state.startNewMatch();
            }}
            title="Start a new game"
            type="button"
          >
            <Icon className="uiIconDesktop" name="plus" />
            <span className="uiActionLabel">New Game</span>
          </button>}
          <Link aria-label="Profile" className="uiAction uiActionSecondary" to="/profile">
            <Icon className="uiIconDesktop" name="profile" />
            <span className="uiActionLabel">Profile</span>
          </Link>
          <Link aria-label="Settings" className="uiAction uiActionSecondary" to="/settings">
            <Icon className="uiIconDesktop" name="settings" />
            <span className="uiActionLabel">Settings</span>
          </Link>
          <Link aria-label="Home" className="uiAction uiActionNeutral" to="/">
            <Icon className="uiIconDesktop" name="home" />
            <span className="uiActionLabel">Home</span>
          </Link>
        </div>
      </header>

      <section className={styles.layout}>
        <div className={styles.boardPanel}>
          <Board model={boardModel} />
        </div>

        <aside className={styles.hud}>
          <section
            className={`${styles.hudSection} ${styles.statusSection} ${state.status !== "playing" ? styles.finishedStatus : ""}`}
          >
            <p className="uiSectionLabel">Status</p>
            <p className={styles.statusText} data-testid="match-status">
              {statusLabel(state)}
            </p>
          </section>

          <div className="uiDivider" />

          <section className={`${styles.hudSection} ${styles.matchSection}`}>
            <p className={`uiSectionLabel ${styles.matchLabel}`}>Match</p>
            <div className={styles.metaRows}>
              <div className={styles.metaRow}>
                <span className={styles.metaLabel}>Rule</span>
                <span className={styles.metaValue} data-testid="match-rule">
                  {variantLabel(state.currentVariant)}
                </span>
              </div>
              <div className={styles.metaRow}>
                <span className={styles.metaLabel}>Bot</span>
                <span className={`${styles.metaValue} ${styles.botValue}`} data-testid="match-bot">
                  <span>{botLabel(state.currentBotConfig)}</span>
                  <span className={styles.botSpec}>{botConfigSummary(state.currentBotConfig)}</span>
                </span>
              </div>
              {setupChanged(state) ? (
                <p className={styles.pendingText}>
                  Next: {nextSetupLabel(state)}
                </p>
              ) : null}
              <div className={`${styles.metaRow} ${styles.moveRow}`}>
                <span className={styles.metaLabel}>Move</span>
                <span className={styles.metaValue} data-testid="match-move-count">
                  {moveCountLabel(state.moves.length)}
                </span>
              </div>
              <Link className="uiAction uiActionNeutral" to="/guide/">
                <Icon name="help" /><span className="uiActionLabel">Board guide</span>
              </Link>
            </div>

            <div className={styles.playerRows}>
              {state.players.map((player, index) => {
                const active =
                  state.status === "playing" &&
                  !state.pendingBotMove &&
                  state.currentPlayer === index + 1;

                return (
                  <article
                    className={[
                      styles.playerRow,
                      player.stone === "black" ? styles.playerRowBlack : styles.playerRowWhite,
                      active ? styles.playerRowActive : "",
                    ].join(" ").trim()}
                    data-testid={`player-row-${player.stone}`}
                    key={player.stone}
                  >
                    <div className={styles.playerCopy}>
                      <div className={styles.playerHead}>
                        <h2 className={styles.playerName}>{player.name}</h2>
                        <span
                          aria-label={player.kind === "human" ? "Player" : "Bot"}
                          className={styles.playerKindIcon}
                          role="img"
                        >
                          <Icon name={player.kind === "human" ? "human" : "bot"} />
                        </span>
                      </div>
                      <p className={styles.playerClock} data-testid={`player-clock-${player.stone}`}>
                        <span>{clockLabel(state.playerClockMs[index] ?? 0)}</span>
                        {state.status === "playing" && state.currentPlayer === index + 1 ? (
                          <ActiveTurnClock startedAtMs={state.turnStartedAtMs} />
                        ) : null}
                      </p>
                    </div>
                  </article>
                );
              })}
            </div>

          </section>

          <div className="uiDivider" />

          <section className={`${styles.hudSection} ${styles.actionSection}`}>
            {state.botError ? <p className={styles.pendingText} role="alert">{state.botError}</p> : null}
            <div className={styles.matchActions}>
              {state.botError ? (
                <button className="uiAction uiActionPrimary" onClick={state.retryBotMove} type="button">
                  <span className="uiActionLabel">Retry bot</span>
                </button>
              ) : null}
              {compactTouchMode && state.status === "playing" ? (
                <button
                  className="uiAction uiActionPrimary"
                  disabled={!humanToMove || !touchCandidate || !touchCandidatePlaceable}
                  onClick={() => {
                    if (!touchCandidate) {
                      return;
                    }

                    state.placeHumanMove(touchCandidate.row, touchCandidate.col);
                  }}
                  type="button"
                >
                  <span className="uiActionLabel">Place</span>
                </button>
              ) : null}
              {state.status !== "playing" && (
                <button
                  className={`uiAction uiActionPrimary ${styles.nextRound}`}
                  onClick={() => {
                    advanceRound();
                    matchHeading.current?.focus();
                  }}
                  title="Swap colors and play again"
                  type="button"
                >
                  <Icon className="uiIconDesktop" name="plus" />
                  <span className="uiActionLabel">Next round</span>
                </button>
              )}
              {state.status !== "playing" && latestReplayId ? (
                <Link
                  aria-label="Analyze"
                  className="uiAction uiActionSecondary"
                  to={`/replay/${latestReplayId}`}
                >
                  <Icon className="uiIconDesktop" name="replay" />
                  <span className="uiActionLabel">Analyze</span>
                </Link>
              ) : null}
              <button
                aria-label="Undo"
                className="uiAction uiActionNeutral"
                disabled={!canUndoLocalMatch(state)}
                onClick={() => {
                  resetTouchCandidate();
                  state.undoLastTurn();
                }}
                type="button"
              >
                <Icon className="uiIconDesktop" name="undo" />
                <span className="uiActionLabel">Undo</span>
              </button>
              {sourceReplay && (
                <Link
                  className={`uiAction uiActionSecondary ${styles.sourceReplay}`}
                  ref={sourceReplayLink}
                  to={sourceReplay.to}
                  state={sourceReplay.state}
                >
                  <Icon name="back" />
                  <span className="uiActionLabel">Source replay</span>
                </Link>
              )}
            </div>
          </section>
        </aside>
      </section>
    </main>
  );
}
