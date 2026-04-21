import { useEffect } from "react";
import { Link } from "react-router-dom";
import { useStore } from "zustand";

import { guestProfileStore, type GuestSavedMatch } from "../profile/guest_profile_store";

import styles from "./ProfileRoute.module.css";

function historyCountLabel(count: number): string {
  return count === 1 ? "1 local match" : `${count} local matches`;
}

function playerLabel(match: GuestSavedMatch, guestDisplayName: string): string {
  return match.players
    .map((player) => {
      const name = player.kind === "human" ? guestDisplayName : player.name;
      return `${name} (${player.stone})`;
    })
    .join(" vs ");
}

function winnerLabel(match: GuestSavedMatch, guestDisplayName: string): string {
  if (match.status === "draw") {
    return "Draw";
  }

  const winningStone = match.status === "black_won" ? "black" : "white";
  const winner = match.players.find((player) => player.stone === winningStone);
  const winnerName = winner?.kind === "human" ? guestDisplayName : winner?.name ?? winningStone;

  return `${winnerName} wins`;
}

export function ProfileRoute() {
  const history = useStore(guestProfileStore, (state) => state.history);
  const profile = useStore(guestProfileStore, (state) => state.profile);
  const settings = useStore(guestProfileStore, (state) => state.settings);

  useEffect(() => {
    guestProfileStore.getState().ensureGuestProfile();
  }, []);

  const wins = history.filter((match) => {
    const winner = match.status === "black_won" ? "black" : match.status === "white_won" ? "white" : null;
    return winner !== null && winner === match.guestStone;
  }).length;
  const draws = history.filter((match) => match.status === "draw").length;
  const losses = history.length - wins - draws;
  const guestDisplayName = profile?.displayName ?? "Guest";

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div>
          <p className={styles.eyebrow}>Phase 2 / local guest profile</p>
          <h1 className={styles.title}>Profile</h1>
        </div>
        <div className={styles.headerActions}>
          <Link className={styles.primaryAction} to="/match/local">
            Play Bot
          </Link>
          <Link className={styles.secondaryAction} to="/">
            Home
          </Link>
        </div>
      </header>

      <section className={styles.layout}>
        <div className={styles.column}>
          <section className={styles.card}>
            <div className={styles.cardHeader}>
              <p className={styles.sectionLabel}>Identity</p>
              <p className={styles.badge}>Guest</p>
            </div>
            <label className={styles.field}>
              <span className={styles.fieldLabel}>Display name</span>
              <input
                className={styles.textInput}
                onChange={(event) => {
                  guestProfileStore.getState().renameDisplayName(event.target.value);
                }}
                type="text"
                value={guestDisplayName}
              />
            </label>
            <dl className={styles.metaList}>
              <div>
                <dt className={styles.metaLabel}>Profile id</dt>
                <dd className={styles.metaValue}>{profile?.id ?? "Pending"}</dd>
              </div>
              <div>
                <dt className={styles.metaLabel}>Public handle</dt>
                <dd className={styles.metaValue}>Sign-in feature comes later</dd>
              </div>
            </dl>
          </section>

          <section className={styles.card}>
            <div className={styles.cardHeader}>
              <p className={styles.sectionLabel}>Settings</p>
            </div>
            <div className={styles.settingsList}>
              <label className={styles.toggleRow}>
                <span>Reduced motion</span>
                <input
                  checked={settings.reducedMotion}
                  onChange={(event) => {
                    guestProfileStore.getState().updateSettings({ reducedMotion: event.target.checked });
                  }}
                  type="checkbox"
                />
              </label>
              <label className={styles.toggleRow}>
                <span>Sound</span>
                <input
                  checked={settings.soundEnabled}
                  onChange={(event) => {
                    guestProfileStore.getState().updateSettings({ soundEnabled: event.target.checked });
                  }}
                  type="checkbox"
                />
              </label>
              <button
                className={styles.dangerAction}
                onClick={() => {
                  const store = guestProfileStore.getState();
                  store.resetGuestProfile();
                  store.ensureGuestProfile();
                }}
                type="button"
              >
                Reset local profile
              </button>
              <div className={styles.staticSetting}>
                <span>Board theme</span>
                <strong>Classic</strong>
              </div>
            </div>
          </section>
        </div>

        <div className={styles.column}>
          <section className={styles.card}>
            <div className={styles.cardHeader}>
              <p className={styles.sectionLabel}>Local History</p>
              <p className={styles.historyCount}>{historyCountLabel(history.length)}</p>
            </div>
            <div className={styles.summaryGrid}>
              <article className={styles.summaryTile}>
                <span className={styles.summaryValue}>{history.length}</span>
                <span className={styles.summaryLabel}>Finished</span>
              </article>
              <article className={styles.summaryTile}>
                <span className={styles.summaryValue}>{wins}</span>
                <span className={styles.summaryLabel}>Wins</span>
              </article>
              <article className={styles.summaryTile}>
                <span className={styles.summaryValue}>{losses}</span>
                <span className={styles.summaryLabel}>Losses</span>
              </article>
              <article className={styles.summaryTile}>
                <span className={styles.summaryValue}>{draws}</span>
                <span className={styles.summaryLabel}>Draws</span>
              </article>
            </div>

            {history.length === 0 ? (
              <p className={styles.emptyState}>Finished local matches will appear here.</p>
            ) : (
              <ol className={styles.historyList}>
                {history.map((match) => (
                  <li className={styles.historyItem} key={match.id}>
                    <div className={styles.historyRow}>
                      <div>
                        <p className={styles.historyTitle}>{winnerLabel(match, guestDisplayName)}</p>
                        <p className={styles.historyMeta}>{playerLabel(match, guestDisplayName)}</p>
                      </div>
                    </div>
                    <div className={styles.historyRow}>
                      <p className={styles.historyMeta}>{match.moves.length} moves</p>
                      <p className={styles.historyMeta}>{new Date(match.savedAt).toLocaleString()}</p>
                    </div>
                  </li>
                ))}
              </ol>
            )}
          </section>
        </div>
      </section>
    </main>
  );
}
