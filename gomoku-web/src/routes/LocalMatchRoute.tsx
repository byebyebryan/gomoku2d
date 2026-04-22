import { useEffect, useRef, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import { Board } from "../components/Board/Board";
import { createLocalMatchStore } from "../game/local_match_store";
import type { LocalMatchResumeSeed, LocalMatchState } from "../game/local_match_store";
import { guestProfileStore } from "../profile/guest_profile_store";
import { variantLabel } from "../replay/local_replay";

import styles from "./LocalMatchRoute.module.css";

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
    { kind: "bot", name: "Classic Bot", stone: "white" },
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

export function LocalMatchRoute() {
  const location = useLocation();
  const storeRef = useRef<ReturnType<typeof createLocalMatchStore> | null>(null);
  const [latestReplayId, setLatestReplayId] = useState<string | null>(null);
  const [storeReady, setStoreReady] = useState(false);
  const profile = useStore(guestProfileStore, (snapshot) => snapshot.profile);
  const state = useStore(storeRef.current ?? loadingMatchStore, (snapshot) => snapshot);
  const resumeSeed = (location.state as { resumeSeed?: LocalMatchResumeSeed } | null)?.resumeSeed ?? null;

  useEffect(() => {
    guestProfileStore.getState().ensureGuestProfile();
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

  if (!storeReady || !storeRef.current) {
    return <main className={styles.page}>Loading match…</main>;
  }

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div>
          <p className="uiPageEyebrow">Classic Bot practice</p>
          <h1 className={styles.title}>Local Match</h1>
        </div>
        <div className={styles.headerActions}>
          <button className="uiAction uiActionPrimary" onClick={state.startNewMatch} type="button">
            New Game
          </button>
          {state.status !== "playing" && latestReplayId ? (
            <Link className="uiAction uiActionSecondary" to={`/replays/local/${latestReplayId}`}>
              Replay
            </Link>
          ) : null}
          <Link className="uiAction uiActionSecondary" to="/profile">
            Profile
          </Link>
          <Link className="uiAction uiActionNeutral" to="/">
            Home
          </Link>
        </div>
      </header>

      <section className={styles.layout}>
        <div className={styles.boardPanel}>
          <Board
            cells={state.cells}
            currentPlayer={state.currentPlayer}
            forbiddenMoves={state.forbiddenMoves}
            interactive={
              !state.pendingBotMove &&
              state.status === "playing" &&
              state.players[state.currentPlayer - 1].kind === "human"
            }
            lastMove={state.lastMove}
            moves={state.moves}
            onAdvanceRound={state.startNextRound}
            onPlace={state.placeHumanMove}
            showSequenceNumbers
            status={state.status}
            threatMoves={state.threatMoves}
            winningMoves={state.winningMoves}
            winningCells={state.winningCells}
          />
        </div>

        <aside className={styles.hud}>
          <section className={styles.hudSection}>
            <div className={styles.rulesHeader}>
              <p className="uiSectionLabel">Rules</p>
              <p className={styles.rulesMeta}>{variantLabel(state.selectedVariant)}</p>
            </div>
            <div className={styles.variantButtons}>
              {(["freestyle", "renju"] as const).map((variant) => (
                <button
                  className={state.selectedVariant === variant ? "uiSegment uiSegmentActive" : "uiSegment"}
                  key={variant}
                  onClick={() => {
                    guestProfileStore.getState().updateSettings({ preferredVariant: variant });
                    state.selectVariant(variant);
                  }}
                  type="button"
                >
                  {variantLabel(variant)}
                </button>
              ))}
            </div>
            {state.selectedVariant !== state.currentVariant ? (
              <p className={styles.pendingText}>Applies next game.</p>
            ) : null}
          </section>

          <div className="uiDivider" />

          <section className={styles.hudSection}>
            <p className="uiSectionLabel">Status</p>
            <p className={styles.statusText} data-testid="match-status">
              {statusLabel(state)}
            </p>
          </section>

          <div className="uiDivider" />

          <section className={styles.hudSection}>
            <p className="uiSectionLabel">Match</p>
            <div className={styles.metaRows}>
              <div className={styles.metaRow}>
                <span className={styles.metaLabel}>Rule</span>
                <span className={styles.metaValue} data-testid="match-rule">
                  {variantLabel(state.currentVariant)}
                </span>
              </div>
              <div className={styles.metaRow}>
                <span className={styles.metaLabel}>Move</span>
                <span className={styles.metaValue} data-testid="match-move-count">
                  {state.moves.length}
                </span>
              </div>
              {state.selectedVariant !== state.currentVariant ? (
                <div className={styles.metaRow}>
                  <span className={styles.metaLabel}>Next game</span>
                  <span className={styles.metaValue} data-testid="match-next-rule">
                    {variantLabel(state.selectedVariant)}
                  </span>
                </div>
              ) : null}
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
                      <h2 className={styles.playerName}>{player.name}</h2>
                      <p className={styles.playerKind}>{player.kind === "human" ? "Player" : "Bot"}</p>
                    </div>
                  </article>
                );
              })}
            </div>

          </section>

          <div className="uiDivider" />

          <section className={styles.hudSection}>
            <div className={styles.matchActions}>
              <button
                className="uiAction uiActionNeutral"
                disabled={!canUndo(state)}
                onClick={state.undoLastTurn}
                type="button"
              >
                Undo
              </button>
            </div>
          </section>
        </aside>
      </section>
    </main>
  );
}
