import { useEffect } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useStore } from "zustand";

import { cloudAuthStore } from "../cloud/auth_store";
import { cloudProfileStore } from "../cloud/cloud_profile_store";
import { cloudPromotionStore } from "../cloud/cloud_promotion_store";
import {
  savedMatchLocalSide,
  savedMatchPlayerForSide,
  savedMatchPlayers,
  savedMatchWinningSide,
  type SavedMatchSide,
} from "../match/saved_match";
import {
  DEFAULT_GUEST_DISPLAY_NAME,
  guestProfileStore,
  type GuestSavedMatch,
} from "../profile/guest_profile_store";
import { replayPlayerName, variantLabel } from "../replay/local_replay";
import { Icon } from "../ui/Icon";

import styles from "./ProfileRoute.module.css";

function historyLocalSide(match: GuestSavedMatch, localProfileId: string | null | undefined): SavedMatchSide | null {
  const localSide = savedMatchLocalSide(match, localProfileId);
  if (localSide) {
    return localSide;
  }

  return savedMatchPlayers(match).find(({ player }) => player.kind === "human")?.side ?? null;
}

function historyResultLabel(
  match: GuestSavedMatch,
  localProfileId: string | null | undefined,
): "Win" | "Loss" | "Draw" {
  if (match.status === "draw") {
    return "Draw";
  }

  return savedMatchWinningSide(match) === historyLocalSide(match, localProfileId) ? "Win" : "Loss";
}

function historyOpponentLabel(
  match: GuestSavedMatch,
  localProfileId: string | null | undefined,
  guestDisplayName: string,
): string {
  const localSide = historyLocalSide(match, localProfileId);
  const opponentSide = localSide === "black" ? "white" : localSide === "white" ? "black" : null;
  if (!opponentSide) {
    return "Opponent";
  }

  const opponent = savedMatchPlayerForSide(match, opponentSide);
  return `vs ${replayPlayerName(opponent, guestDisplayName)}`;
}

function historyDateLabel(savedAt: string): string {
  return new Date(savedAt).toLocaleDateString(undefined, {
    month: "2-digit",
    day: "2-digit",
    year: "2-digit",
  });
}

function historyTimeLabel(savedAt: string): string {
  return new Date(savedAt).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function historySideLabel(side: SavedMatchSide | null): "Black" | "White" | "Unknown" {
  if (side === "black") {
    return "Black";
  }

  if (side === "white") {
    return "White";
  }

  return "Unknown";
}

function cloudStateLabel(
  authStatus: ReturnType<typeof cloudAuthStore.getState>["status"],
  profileStatus: ReturnType<typeof cloudProfileStore.getState>["status"],
): string {
  if (authStatus === "unconfigured") {
    return "Unavailable";
  }

  if (authStatus === "loading" || profileStatus === "loading") {
    return "Checking";
  }

  if (authStatus === "signed_in" && profileStatus === "ready") {
    return "Cloud";
  }

  if (authStatus === "error" || profileStatus === "error") {
    return "Error";
  }

  return "Local";
}

function cloudCopyText({
  authStatus,
  hasCloudIdentity,
  historyCount,
  promotionStatus,
  totalPromotedMatches,
}: {
  authStatus: ReturnType<typeof cloudAuthStore.getState>["status"];
  hasCloudIdentity: boolean;
  historyCount: number;
  promotionStatus: ReturnType<typeof cloudPromotionStore.getState>["status"];
  totalPromotedMatches: number;
}): string {
  if (authStatus === "unconfigured") {
    return "Cloud sign-in is not configured for this build.";
  }

  if (!hasCloudIdentity) {
    return "Sign in when you want this profile to follow you across browsers.";
  }

  if (promotionStatus === "promoting") {
    return "Copying local history to private cloud...";
  }

  if (promotionStatus === "error") {
    return "Cloud import failed. Local history is safe on this device.";
  }

  if (promotionStatus === "complete") {
    return totalPromotedMatches > 0
      ? "Local history copied to private cloud. Local copies stay on this device."
      : "Private cloud identity is linked. No local matches need importing yet.";
  }

  return historyCount > 0
    ? "Private cloud identity is linked. Local history will copy to private cloud automatically."
    : "Private cloud identity is linked. Local play still works without cloud history.";
}

function shouldAdoptCloudDisplayName(
  localDisplayName: string | undefined,
  cloudDisplayName: string | undefined,
): cloudDisplayName is string {
  return (
    localDisplayName === DEFAULT_GUEST_DISPLAY_NAME
    && Boolean(cloudDisplayName?.trim())
    && cloudDisplayName !== DEFAULT_GUEST_DISPLAY_NAME
  );
}

export function ProfileRoute() {
  const navigate = useNavigate();
  const cloudAuth = useStore(cloudAuthStore, (state) => state);
  const cloudProfile = useStore(cloudProfileStore, (state) => state);
  const cloudPromotion = useStore(cloudPromotionStore, (state) => state);
  const history = useStore(guestProfileStore, (state) => state.history);
  const profile = useStore(guestProfileStore, (state) => state.profile);
  const settings = useStore(guestProfileStore, (state) => state.settings);

  useEffect(() => {
    guestProfileStore.getState().ensureGuestProfile();
  }, []);

  useEffect(() => {
    cloudAuthStore.getState().start();

    return () => {
      cloudAuthStore.getState().stop();
    };
  }, []);

  useEffect(() => {
    if (cloudAuth.status === "signed_in" && cloudAuth.user) {
      void cloudProfileStore.getState().loadForUser(cloudAuth.user, settings.preferredVariant);
      return;
    }

    cloudProfileStore.getState().reset();
  }, [cloudAuth.status, cloudAuth.user, settings.preferredVariant]);

  useEffect(() => {
    const cloudDisplayName = cloudProfile.profile?.displayName;
    if (
      cloudAuth.status === "signed_in"
      && cloudProfile.status === "ready"
      && shouldAdoptCloudDisplayName(profile?.displayName, cloudDisplayName)
    ) {
      guestProfileStore.getState().renameDisplayName(cloudDisplayName);
    }
  }, [
    cloudAuth.status,
    cloudProfile.profile?.displayName,
    cloudProfile.status,
    profile?.displayName,
  ]);

  useEffect(() => {
    const waitingForCloudNameAdoption = shouldAdoptCloudDisplayName(
      profile?.displayName,
      cloudProfile.profile?.displayName,
    );

    if (
      cloudAuth.status === "signed_in"
      && cloudAuth.user
      && cloudProfile.status === "ready"
      && profile
      && !waitingForCloudNameAdoption
    ) {
      void cloudPromotionStore.getState().promote({
        guestHistory: history,
        guestProfile: profile,
        settings,
        user: cloudAuth.user,
      });
      return;
    }

    cloudPromotionStore.getState().reset();
  }, [
    cloudAuth.status,
    cloudAuth.user,
    cloudProfile.profile?.displayName,
    cloudProfile.status,
    history,
    profile,
    settings,
  ]);

  const wins = history.filter((match) => {
    return historyResultLabel(match, profile?.id) === "Win";
  }).length;
  const draws = history.filter((match) => match.status === "draw").length;
  const losses = history.length - wins - draws;
  const guestDisplayName = profile?.displayName ?? DEFAULT_GUEST_DISPLAY_NAME;
  const cloudBadge = cloudStateLabel(cloudAuth.status, cloudProfile.status);
  const cloudIdentity = cloudProfile.profile ?? null;
  const cloudDisplayName = cloudPromotion.result?.promotedDisplayName ?? cloudIdentity?.displayName;
  const cloudError = cloudAuth.errorMessage ?? cloudProfile.errorMessage ?? cloudPromotion.errorMessage;
  const cloudText = cloudCopyText({
    authStatus: cloudAuth.status,
    hasCloudIdentity: Boolean(cloudIdentity),
    historyCount: history.length,
    promotionStatus: cloudPromotion.status,
    totalPromotedMatches: cloudPromotion.result?.totalMatches ?? 0,
  });

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerCopy}>
          <p className="uiPageEyebrow">Local record</p>
          <h1 className={styles.title}>Profile</h1>
        </div>
        <div className={styles.headerActions}>
          <Link aria-label="Play" className="uiAction uiActionPrimary" to="/match/local">
            <Icon className="uiIconDesktop" name="play" />
            <span className="uiActionLabel">Play</span>
          </Link>
          <Link aria-label="Home" className="uiAction uiActionNeutral" to="/">
            <Icon className="uiIconDesktop" name="home" />
            <span className="uiActionLabel">Home</span>
          </Link>
        </div>
      </header>

      <section className={styles.layout}>
        <aside className={styles.sidePanel}>
          <section className={`${styles.sideSection} ${styles.identitySection}`}>
            <div className={styles.sectionHeader}>
              <p className="uiSectionLabel">Identity</p>
              <p className={styles.badge}>{cloudBadge}</p>
            </div>
            <label className={styles.field}>
              <span className={styles.fieldLabel}>Display name</span>
              <input
                className="uiInput"
                onChange={(event) => {
                  guestProfileStore.getState().renameDisplayName(event.target.value);
                }}
                placeholder="Display name"
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
                <dd className={styles.metaValue}>
                  {cloudIdentity?.uid ?? (cloudAuth.status === "unconfigured" ? "Not configured" : "Not linked")}
                </dd>
              </div>
            </dl>
            <div className={styles.cloudStatus}>
              <div className={styles.cloudCopy}>
                <p className={styles.cloudTitle}>
                  {cloudDisplayName ? `Signed in as ${cloudDisplayName}` : "Cloud profile"}
                </p>
                <p className={styles.cloudText}>{cloudText}</p>
                {cloudError ? <p className={styles.cloudError}>{cloudError}</p> : null}
              </div>
              {cloudAuth.status === "signed_in" ? (
                <button
                  className="uiAction uiActionNeutral"
                  onClick={() => {
                    void cloudAuthStore.getState().signOut();
                  }}
                  type="button"
                >
                  <span className="uiActionLabel">Sign out</span>
                </button>
              ) : (
                <button
                  className="uiAction uiActionSecondary"
                  disabled={!cloudAuth.isConfigured || cloudAuth.status === "loading"}
                  onClick={() => {
                    void cloudAuthStore.getState().signInWithGoogle();
                  }}
                  type="button"
                >
                  <span className="uiActionLabel">
                    {cloudAuth.status === "loading" ? "Checking" : "Sign in with Google"}
                  </span>
                </button>
              )}
            </div>
          </section>

          <div className="uiDivider" />

          <section className={`${styles.sideSection} ${styles.rulesSection}`}>
            <p className="uiSectionLabel">Default rule</p>
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

          <section className={`${styles.sideSection} ${styles.resetSection}`}>
            <button
              className="uiAction uiActionDanger"
              onClick={() => {
                const store = guestProfileStore.getState();
                store.resetGuestProfile();
                store.ensureGuestProfile();
              }}
              type="button"
            >
              <Icon className="uiIconDesktop" name="reset" />
              <span className="uiActionLabel">Reset local profile</span>
            </button>
          </section>
        </aside>

        <section className={styles.recordPanel}>
          <div className={styles.recordHeader}>
            <p className="uiSectionLabel">Match History</p>
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
          {history.length > 0 ? (
            <div className={styles.historyHead} aria-hidden="true">
              <span className={styles.historyHeadLabel}>Result</span>
              <span className={styles.historyHeadLabel}>Opponent</span>
              <span className={`${styles.historyHeadLabel} ${styles.historyHeadSide}`}>Side</span>
              <span className={`${styles.historyHeadLabel} ${styles.historyHeadRule}`}>Rule</span>
              <span className={`${styles.historyHeadLabel} ${styles.historyHeadMoves}`}>Moves</span>
              <span className={`${styles.historyHeadLabel} ${styles.historyHeadPlayed}`}>Played</span>
              <span className={styles.historyHeadSpacer} />
            </div>
          ) : null}
          <div className={styles.historyBody}>
            {history.length === 0 ? (
              <p className={styles.emptyState}>Finished matches are saved here.</p>
            ) : (
              <ol className={styles.historyList}>
                {history.map((match) => {
                  const localSide = historyLocalSide(match, profile?.id);
                  const result = historyResultLabel(match, profile?.id);

                  return (
                    <li className={styles.historyItem} key={match.id}>
                      <div className={styles.historySummary}>
                        <p
                          className={`${styles.historyResult} ${
                            result === "Win"
                              ? styles.historyResultWin
                              : result === "Loss"
                                ? styles.historyResultLoss
                                : styles.historyResultDraw
                          }`}
                        >
                          {result}
                        </p>
                        <p className={styles.historyOpponent}>
                          {historyOpponentLabel(match, profile?.id, guestDisplayName)}
                        </p>
                      </div>
                      <div className={styles.historyDetails}>
                        <p
                          className={`${styles.historyField} ${styles.historyStone} ${styles.historySide} ${
                            localSide === "black"
                              ? styles.historyStoneBlack
                              : localSide === "white"
                                ? styles.historyStoneWhite
                                : ""
                          }`}
                          data-label="Side"
                        >
                          {historySideLabel(localSide)}
                        </p>
                        <p className={`${styles.historyField} ${styles.historyRule}`} data-label="Rule">
                          {variantLabel(match.variant)}
                        </p>
                        <p className={`${styles.historyField} ${styles.historyMoves}`} data-label="Moves">
                          {`Moves ${match.move_count}`}
                        </p>
                        <p className={`${styles.historyPlayed} ${styles.historyPlayedField}`} data-label="Played">
                          <span className={styles.historyDate}>{historyDateLabel(match.saved_at)}</span>
                          <span className={styles.historyTime}>{historyTimeLabel(match.saved_at)}</span>
                        </p>
                      </div>
                      <button
                        aria-label="Replay"
                        className={`uiAction uiActionSecondary ${styles.historyReplayAction}`}
                        onClick={() => {
                          navigate(`/replays/local/${match.id}`);
                        }}
                        type="button"
                      >
                        <Icon className="uiIconDesktop" name="replay" />
                        <span className="uiActionLabel">Replay</span>
                      </button>
                    </li>
                  );
                })}
              </ol>
            )}
          </div>
        </section>
      </section>
    </main>
  );
}
