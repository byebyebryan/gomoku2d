import { useEffect } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useStore } from "zustand";

import { guestProfileStore } from "../profile/guest_profile_store";
import { replayPlayerLabel, replayWinnerLabel, variantLabel } from "../replay/local_replay";

import styles from "./ProfileRoute.module.css";

function historyCountLabel(count: number): string {
  return count === 1 ? "1 local match" : `${count} local matches`;
}

export function ProfileRoute() {
  const navigate = useNavigate();
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
          <p className="uiPageEyebrow">Player record</p>
          <h1 className={styles.title}>Profile</h1>
        </div>
        <div className={styles.headerActions}>
          <Link className="uiAction uiActionPrimary" to="/match/local">
            Play
          </Link>
          <Link className="uiAction uiActionAccent" to="/">
            Home
          </Link>
        </div>
      </header>

      <section className={styles.layout}>
        <aside className={`uiPanel ${styles.sidePanel}`}>
          <section className={styles.sideSection}>
            <div className={styles.sectionHeader}>
              <p className="uiSectionLabel">Identity</p>
              <p className={styles.badge}>Guest</p>
            </div>
            <label className={styles.field}>
              <span className={styles.fieldLabel}>Display name</span>
              <input
                className="uiInput"
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
                <dt className={styles.metaLabel}>Online profile</dt>
                <dd className={styles.metaValue}>No online profile linked</dd>
              </div>
            </dl>
          </section>

          <div className="uiDivider" />

          <section className={styles.sideSection}>
            <div className={styles.sectionHeader}>
              <p className="uiSectionLabel">Preferred rules</p>
              <p className={styles.settingValue}>{variantLabel(settings.preferredVariant)}</p>
            </div>
            <div className={styles.variantButtons}>
              {(["freestyle", "renju"] as const).map((variant) => (
                <button
                  className={
                    settings.preferredVariant === variant
                      ? "uiSegment uiSegmentActive"
                      : "uiSegment"
                  }
                  key={variant}
                  onClick={() => {
                    guestProfileStore.getState().updateSettings({ preferredVariant: variant });
                  }}
                  type="button"
                >
                  {variantLabel(variant)}
                </button>
              ))}
            </div>
          </section>

          <div className="uiDivider" />

          <section className={styles.sideSection}>
            <button
              className="uiAction uiActionDanger"
              onClick={() => {
                const store = guestProfileStore.getState();
                store.resetGuestProfile();
                store.ensureGuestProfile();
              }}
              type="button"
            >
              Reset local profile
            </button>
          </section>
        </aside>

        <section className={`uiPanel ${styles.recordPanel}`}>
          <div className={styles.recordHeader}>
            <p className="uiSectionLabel">Local history</p>
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
          <div className={styles.historyBody}>
            {history.length === 0 ? (
              <p className={styles.emptyState}>Finished matches are saved here.</p>
            ) : (
              <ol className={styles.historyList}>
                {history.map((match) => (
                  <li className={styles.historyItem} key={match.id}>
                    <div className={styles.historyMain}>
                      <div>
                        <p className={styles.historyTitle}>{replayWinnerLabel(match, guestDisplayName)}</p>
                        <p className={styles.historyMatchup}>{replayPlayerLabel(match, guestDisplayName)}</p>
                      </div>
                      <button
                        className="uiAction uiActionSecondary"
                        onClick={() => {
                          navigate(`/replays/local/${match.id}`);
                        }}
                        type="button"
                      >
                        Replay
                      </button>
                    </div>
                    <p className={styles.historyMeta}>
                      {variantLabel(match.variant)} · {match.moves.length} moves · {new Date(match.savedAt).toLocaleString()}
                    </p>
                  </li>
                ))}
              </ol>
            )}
          </div>
        </section>
      </section>
    </main>
  );
}
