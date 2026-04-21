import { useEffect, useRef } from "react";
import { Link } from "react-router-dom";
import { useStore } from "zustand";

import { Board } from "../components/Board/Board";
import { createLocalMatchStore } from "../game/local_match_store";
import type { LocalMatchState } from "../game/local_match_store";
import { guestProfileStore } from "../profile/guest_profile_store";
import { variantLabel } from "../replay/local_replay";

import styles from "./LocalMatchRoute.module.css";

function statusLabel(state: Pick<LocalMatchState, "currentPlayer" | "pendingBotMove" | "status">): string {
  if (state.status === "black_won") {
    return "Black wins";
  }
  if (state.status === "white_won") {
    return "White wins";
  }
  if (state.status === "draw") {
    return "Draw";
  }
  if (state.pendingBotMove) {
    return "Bot is thinking...";
  }

  return state.currentPlayer === 1 ? "Black to move" : "White to move";
}

export function LocalMatchRoute() {
  const storeRef = useRef<ReturnType<typeof createLocalMatchStore> | null>(null);

  if (!storeRef.current) {
    const guestProfile = guestProfileStore.getState();
    const profile = guestProfile.ensureGuestProfile();
    storeRef.current = createLocalMatchStore({
      humanDisplayName: profile.displayName,
      onMatchFinished: (match) => {
        guestProfileStore.getState().recordFinishedMatch(match);
      },
      variant: guestProfile.settings.preferredVariant,
    });
  }

  const state = useStore(storeRef.current, (snapshot) => snapshot);
  const preferredVariant = useStore(guestProfileStore, (snapshot) => snapshot.settings.preferredVariant);

  useEffect(() => {
    const store = storeRef.current;

    return () => {
      store?.getState().dispose();
    };
  }, []);

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
          </section>
        </aside>
      </section>
    </main>
  );
}
