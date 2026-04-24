import { useEffect, useRef, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import { Board } from "../components/Board/Board";
import { createLocalMatchStore } from "../game/local_match_store";
import type { LocalMatchResumeSeed, LocalMatchState } from "../game/local_match_store";
import type { CellPosition } from "../game/types";
import { guestProfileStore } from "../profile/guest_profile_store";
import { variantLabel } from "../replay/local_replay";
import { Icon } from "../ui/Icon";

import styles from "./LocalMatchRoute.module.css";

const MOBILE_TOUCH_QUERY =
  "(max-width: 720px) and (orientation: portrait) and (hover: none) and (pointer: coarse)";

function loadingCells(): LocalMatchState["cells"] {
  return Array.from({ length: 15 }, () => Array.from({ length: 15 }, () => null));
}

const loadingMatchStore = createStore<LocalMatchState>(() => ({
  cells: loadingCells(),
  currentPlayer: 1,
  currentVariant: "freestyle",
  forbiddenMoves: [],
  lastMove: null,
  moves: [],
  pendingBotMove: false,
  placeHumanMove: () => false,
  players: [
    { kind: "human", name: "Guest", stone: "black" },
    { kind: "bot", name: "Practice Bot", stone: "white" },
  ],
  selectedVariant: "freestyle",
  selectVariant: () => undefined,
  startNewMatch: () => undefined,
  startNextRound: () => undefined,
  status: "playing",
  threatMoves: [],
  undoLastTurn: () => false,
  dispose: () => undefined,
  winningMoves: [],
  winningCells: [],
}));

function statusLabel(
  state: Pick<LocalMatchState, "currentPlayer" | "pendingBotMove" | "players" | "status">,
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
    return `${state.players[state.currentPlayer - 1]?.name ?? "Bot"} is thinking...`;
  }

  return `${state.players[state.currentPlayer - 1]?.name ?? "Unknown"} to move`;
}

function canUndo(
  state: Pick<LocalMatchState, "moves" | "players">,
): boolean {
  const minimumMoves = state.players[0]?.kind === "bot" ? 1 : 0;
  return state.moves.length > minimumMoves;
}

function moveCountLabel(moveCount: number): string {
  return `Move ${moveCount}`;
}

export function LocalMatchRoute() {
  const location = useLocation();
  const storeRef = useRef<ReturnType<typeof createLocalMatchStore> | null>(null);
  const [compactTouchMode, setCompactTouchMode] = useState(false);
  const [latestReplayId, setLatestReplayId] = useState<string | null>(null);
  const [storeReady, setStoreReady] = useState(false);
  const [touchCandidate, setTouchCandidate] = useState<CellPosition | null>(null);
  const [touchCandidatePlaceable, setTouchCandidatePlaceable] = useState(false);
  const [touchCandidateResetVersion, setTouchCandidateResetVersion] = useState(0);
  const profile = useStore(guestProfileStore, (snapshot) => snapshot.profile);
  const state = useStore(storeRef.current ?? loadingMatchStore, (snapshot) => snapshot);
  const resumeSeed = (location.state as { resumeSeed?: LocalMatchResumeSeed } | null)?.resumeSeed ?? null;

  useEffect(() => {
    guestProfileStore.getState().ensureGuestProfile();
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
    if (!profile || storeRef.current) {
      return;
    }

    storeRef.current = createLocalMatchStore({
      humanDisplayName: profile.displayName,
      onMatchFinished: (match) => {
        const replayId = guestProfileStore.getState().recordFinishedMatch(match);
        setLatestReplayId(replayId);
      },
      resumeState: resumeSeed ?? undefined,
      variant: resumeSeed?.variant ?? guestProfileStore.getState().settings.preferredVariant,
    });
    setStoreReady(true);
  }, [profile, resumeSeed]);

  useEffect(() => {
    return () => {
      storeRef.current?.getState().dispose();
      storeRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (state.status === "playing" && latestReplayId) {
      setLatestReplayId(null);
    }
  }, [latestReplayId, state.status]);

  useEffect(() => {
    if (compactTouchMode) {
      return;
    }

    setTouchCandidate(null);
    setTouchCandidatePlaceable(false);
  }, [compactTouchMode]);

  if (!storeReady || !storeRef.current) {
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

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerCopy}>
          <p className="uiPageEyebrow">Solo play</p>
          <h1 className={styles.title}>Local Match</h1>
        </div>
        <div className={styles.headerActions}>
          <button
            aria-label="New Game"
            className="uiAction uiActionPrimary"
            onClick={() => {
              resetTouchCandidate();
              state.startNewMatch();
            }}
            type="button"
          >
            <Icon className="uiIconDesktop" name="plus" />
            <span className="uiActionLabel">New Game</span>
          </button>
          {state.status !== "playing" && latestReplayId ? (
            <Link
              aria-label="Replay"
              className="uiAction uiActionSecondary"
              to={`/replays/local/${latestReplayId}`}
            >
              <Icon className="uiIconDesktop" name="replay" />
              <span className="uiActionLabel">Replay</span>
            </Link>
          ) : null}
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
            cells={state.cells}
            currentPlayer={state.currentPlayer}
            forbiddenMoves={state.forbiddenMoves}
            interactive={humanToMove}
            lastMove={state.lastMove}
            mobileTouchPlacement={compactTouchMode}
            moves={state.moves}
            onAdvanceRound={state.startNextRound}
            onPlace={state.placeHumanMove}
            onTouchCandidateChange={(candidate, canPlace) => {
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
            }}
            touchCandidateResetVersion={touchCandidateResetVersion}
            showSequenceNumbers
            status={state.status}
            threatMoves={state.threatMoves}
            winningMoves={state.winningMoves}
            winningCells={state.winningCells}
          />
        </div>

        <aside className={styles.hud}>
          <section className={`${styles.hudSection} ${styles.statusSection}`}>
            <p className="uiSectionLabel">Status</p>
            <p className={styles.statusText} data-testid="match-status">
              {statusLabel(state)}
            </p>
          </section>

          <div className="uiDivider" />

          <section className={`${styles.hudSection} ${styles.matchSection}`}>
            <p className={`uiSectionLabel ${styles.matchLabel}`}>Match</p>
            <div className={styles.metaRows}>
              <div className={`${styles.metaRow} ${styles.metaRowControls} ${styles.ruleRow}`}>
                <div className={styles.ruleControlCopy}>
                  <span className={styles.metaLabel}>Rule</span>
                  <p
                    aria-hidden={state.selectedVariant === state.currentVariant}
                    className={styles.pendingText}
                    data-active={state.selectedVariant !== state.currentVariant}
                  >
                    Applies next game.
                  </p>
                </div>
                <div className={styles.variantButtons}>
                  {(["freestyle", "renju"] as const).map((variant) => (
                    <button
                      className={state.selectedVariant === variant ? "uiSegment uiSegmentActive" : "uiSegment"}
                      data-testid={
                        state.currentVariant === variant
                          ? "match-rule"
                          : state.selectedVariant !== state.currentVariant && state.selectedVariant === variant
                            ? "match-next-rule"
                            : undefined
                      }
                      key={variant}
                      onClick={() => {
                        if (state.moves.length === 0) {
                          resetTouchCandidate();
                        }
                        guestProfileStore.getState().updateSettings({ preferredVariant: variant });
                        state.selectVariant(variant);
                      }}
                      type="button"
                    >
                      {variantLabel(variant)}
                    </button>
                  ))}
                </div>
              </div>
              <div className={`${styles.metaRow} ${styles.moveRow}`}>
                <span className={styles.metaLabel}>Move</span>
                <span className={styles.metaValue} data-testid="match-move-count">
                  {moveCountLabel(state.moves.length)}
                </span>
              </div>
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
                    </div>
                  </article>
                );
              })}
            </div>

          </section>

          <div className="uiDivider" />

          <section className={`${styles.hudSection} ${styles.actionSection}`}>
            <div className={styles.matchActions}>
              {compactTouchMode ? (
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
              <button
                aria-label="Undo"
                className="uiAction uiActionNeutral"
                disabled={!canUndo(state)}
                onClick={() => {
                  resetTouchCandidate();
                  state.undoLastTurn();
                }}
                type="button"
              >
                <Icon className="uiIconDesktop" name="undo" />
                <span className="uiActionLabel">Undo</span>
              </button>
            </div>
          </section>
        </aside>
      </section>
    </main>
  );
}
