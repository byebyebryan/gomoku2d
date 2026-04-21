import { useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import { Board } from "../components/Board/Board";
import { createLocalMatchStore } from "../game/local_match_store";
import type { LocalMatchState } from "../game/local_match_store";
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

export function LocalMatchRoute() {
  const historyBodyRef = useRef<HTMLDivElement | null>(null);
  const previousMoveCountRef = useRef(0);
  const storeRef = useRef<ReturnType<typeof createLocalMatchStore> | null>(null);
  const [storeReady, setStoreReady] = useState(false);
  const profile = useStore(guestProfileStore, (snapshot) => snapshot.profile);
  const preferredVariant = useStore(guestProfileStore, (snapshot) => snapshot.settings.preferredVariant);
  const state = useStore(storeRef.current ?? loadingMatchStore, (snapshot) => snapshot);

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
        guestProfileStore.getState().recordFinishedMatch(match);
      },
      variant: guestProfileStore.getState().settings.preferredVariant,
    });
    setStoreReady(true);
  }, [profile]);

  useEffect(() => {
    return () => {
      storeRef.current?.getState().dispose();
      storeRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (!storeReady) {
      return;
    }

    const historyBody = historyBodyRef.current;
    if (!historyBody) {
      previousMoveCountRef.current = state.moves.length;
      return;
    }

    if (state.moves.length === 0) {
      historyBody.scrollTop = 0;
      previousMoveCountRef.current = 0;
      return;
    }

    if (state.moves.length > previousMoveCountRef.current) {
      historyBody.scrollTop = historyBody.scrollHeight;
    }

    previousMoveCountRef.current = state.moves.length;
  }, [state.moves.length, storeReady]);

  if (!storeReady || !storeRef.current) {
    return <main className={styles.page}>Loading match…</main>;
  }

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div>
          <p className={styles.eyebrow}>Face the Classic Bot</p>
          <h1 className={styles.title}>Local Match</h1>
        </div>
        <div className={styles.headerActions}>
          <button className={`${styles.secondaryAction} ${styles.successAction}`} onClick={state.startNewMatch} type="button">
            New Game
          </button>
          <Link className={`${styles.secondaryAction} ${styles.infoAction}`} to="/profile">
            Profile
          </Link>
          <Link className={`${styles.secondaryAction} ${styles.accentAction}`} to="/">
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

        <aside className={styles.sidebar}>
          <section className={styles.statusCard}>
            <p className={styles.sectionLabel}>Status</p>
            <p className={styles.statusText}>{statusLabel(state)}</p>
          </section>

          <section className={styles.rulesCard}>
            <div className={styles.rulesHeader}>
              <p className={styles.sectionLabel}>Rules</p>
              <p className={styles.rulesMeta}>Current: {variantLabel(state.currentVariant)}</p>
            </div>
            <div className={styles.variantButtons}>
              {(["freestyle", "renju"] as const).map((variant) => (
                <button
                  className={
                    preferredVariant === variant
                      ? `${styles.variantButton} ${styles.variantButtonActive}`
                      : styles.variantButton
                  }
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
              <p className={styles.rulesMeta}>Next game: {variantLabel(state.selectedVariant)}</p>
            ) : null}
          </section>

          <section className={styles.playerList}>
            {state.players.map((player, index) => {
              const active =
                state.status === "playing" &&
                !state.pendingBotMove &&
                state.currentPlayer === index + 1;
              const stoneToneClass = index === 0 ? styles.playerCardBlack : styles.playerCardWhite;

              return (
                <article
                  className={
                    active
                      ? `${styles.playerCard} ${stoneToneClass} ${styles.playerCardActive}`
                      : `${styles.playerCard} ${stoneToneClass}`
                  }
                  key={player.stone}
                >
                  <div className={styles.playerMeta}>
                    <p className={styles.playerStone}>{player.stone}</p>
                    <div>
                      <h2 className={styles.playerName}>{player.name}</h2>
                      <p className={styles.playerKind}>{player.kind === "human" ? "Human" : "Bot"}</p>
                    </div>
                  </div>
                </article>
              );
            })}
          </section>

          <section className={styles.historyCard}>
            <div className={styles.historyHeader}>
              <p className={styles.sectionLabel}>Move history</p>
              <p className={styles.historyCount}>{state.moves.length} moves</p>
            </div>
            <div className={styles.historyBody} ref={historyBodyRef}>
              {state.moves.length === 0 ? (
                <p className={styles.emptyHistory}>Moves appear here as the game unfolds.</p>
              ) : (
                <ol className={styles.historyList}>
                  {state.moves.map((move) => (
                    <li className={styles.historyItem} key={move.moveNumber}>
                      <span>M{move.moveNumber}</span>
                      <span>{move.player === 1 ? "Black" : "White"}</span>
                      <span>
                        {move.row + 1},{move.col + 1}
                      </span>
                    </li>
                  ))}
                </ol>
              )}
            </div>
          </section>
        </aside>
      </section>
    </main>
  );
}
