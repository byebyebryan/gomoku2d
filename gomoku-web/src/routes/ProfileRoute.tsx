import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useStore } from "zustand";

import { cloudAuthStore } from "../cloud/auth_store";
import { cloudHistoryStore } from "../cloud/cloud_history_store";
import { cloudProfileStore } from "../cloud/cloud_profile_store";
import { cloudPromotionStore } from "../cloud/cloud_promotion_store";
import {
  matchUserSide,
  savedMatchPlayerForSide,
  savedMatchPlayers,
  savedMatchWinningSide,
  type SavedMatchV1,
  type SavedMatchSide,
} from "../match/saved_match";
import { resolveActiveHistory } from "../profile/active_history";
import {
  DEFAULT_GUEST_DISPLAY_NAME,
  guestProfileStore,
} from "../profile/guest_profile_store";
import { replayPlayerName, variantLabel } from "../replay/local_replay";
import { Icon } from "../ui/Icon";

import styles from "./ProfileRoute.module.css";

interface HistoryIdentity {
  localProfileId?: string | null;
  profileUid?: string | null;
}

function historyLocalSide(match: SavedMatchV1, identity: HistoryIdentity): SavedMatchSide | null {
  const localSide = matchUserSide(match, identity);
  if (localSide) {
    return localSide;
  }

  return savedMatchPlayers(match).find(({ player }) => player.kind === "human")?.side ?? null;
}

function historyResultLabel(
  match: SavedMatchV1,
  identity: HistoryIdentity,
): "Win" | "Loss" | "Draw" {
  if (match.status === "draw") {
    return "Draw";
  }

  return savedMatchWinningSide(match) === historyLocalSide(match, identity) ? "Win" : "Loss";
}

function historyOpponentLabel(
  match: SavedMatchV1,
  identity: HistoryIdentity,
  guestDisplayName: string,
): string {
  const localSide = historyLocalSide(match, identity);
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
    return "Local";
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
  profileStatus,
  totalPromotedMatches,
}: {
  authStatus: ReturnType<typeof cloudAuthStore.getState>["status"];
  hasCloudIdentity: boolean;
  historyCount: number;
  promotionStatus: ReturnType<typeof cloudPromotionStore.getState>["status"];
  profileStatus: ReturnType<typeof cloudProfileStore.getState>["status"];
  totalPromotedMatches: number;
}): string {
  if (authStatus === "unconfigured") {
    return "Online features disabled.";
  }

  if (authStatus === "loading") {
    return "Checking online features...";
  }

  if (authStatus === "error") {
    return "Sign-in unavailable.";
  }

  if (!hasCloudIdentity) {
    if (authStatus === "signed_in" || profileStatus === "loading") {
      return "Loading cloud profile...";
    }

    if (profileStatus === "error") {
      return "Cloud profile unavailable.";
    }

    return "Sign in for cloud history.";
  }

  if (promotionStatus === "promoting") {
    return "Syncing local history...";
  }

  if (promotionStatus === "error") {
    return "Sync failed.";
  }

  if (promotionStatus === "complete") {
    return totalPromotedMatches > 0
      ? "Local history synced to cloud."
      : "Cloud history enabled.";
  }

  return "Cloud history enabled.";
}

function cloudTitleText({
  authStatus,
  cloudDisplayName,
}: {
  authStatus: ReturnType<typeof cloudAuthStore.getState>["status"];
  cloudDisplayName: string | undefined;
}): string {
  if (authStatus === "unconfigured") {
    return "Local profile";
  }

  if (authStatus === "loading") {
    return "Checking sign-in";
  }

  if (authStatus === "signed_in") {
    return cloudDisplayName ? `Signed in as ${cloudDisplayName}` : "Signed in";
  }

  if (authStatus === "error") {
    return "Sign-in needs attention";
  }

  return "Cloud profile";
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
  const [resetConfirming, setResetConfirming] = useState(false);
  const [resetBusy, setResetBusy] = useState(false);
  const cloudAuth = useStore(cloudAuthStore, (state) => state);
  const cloudHistory = useStore(cloudHistoryStore, (state) => state);
  const cloudProfile = useStore(cloudProfileStore, (state) => state);
  const cloudPromotion = useStore(cloudPromotionStore, (state) => state);
  const localHistory = useStore(guestProfileStore, (state) => state.history);
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
        cloudDisplayName: cloudProfile.profile?.displayName ?? null,
        guestHistory: localHistory,
        guestProfile: profile,
        historyResetAt: cloudProfile.profile?.historyResetAt ?? null,
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
    cloudProfile.profile?.historyResetAt,
    cloudProfile.status,
    localHistory,
    profile,
    settings,
  ]);

  useEffect(() => {
    if (cloudAuth.status !== "signed_in" || !cloudAuth.user || cloudProfile.status !== "ready") {
      return;
    }

    const user = cloudAuth.user;
    const historyResetAt = cloudProfile.profile?.historyResetAt ?? null;
    void cloudHistoryStore.getState().loadForUser(user, historyResetAt).then(() => {
      void cloudHistoryStore.getState().syncPendingForUser(user, historyResetAt);
    });
  }, [
    cloudAuth.status,
    cloudAuth.user,
    cloudProfile.profile?.historyResetAt,
    cloudProfile.status,
    cloudPromotion.status,
  ]);

  const cloudCache =
    cloudAuth.status === "signed_in" && cloudAuth.user
      ? cloudHistory.users[cloudAuth.user.uid]?.cachedMatches ?? []
      : [];
  const history = resolveActiveHistory({
    cloudHistory: cloudCache,
    historyResetAt: cloudAuth.status === "signed_in" ? cloudProfile.profile?.historyResetAt : null,
    localHistory,
  });
  const historyIdentity: HistoryIdentity = {
    localProfileId: profile?.id,
    profileUid: cloudAuth.status === "signed_in" ? cloudAuth.user?.uid : null,
  };
  const wins = history.filter((match) => {
    return historyResultLabel(match, historyIdentity) === "Win";
  }).length;
  const draws = history.filter((match) => match.status === "draw").length;
  const losses = history.length - wins - draws;
  const guestDisplayName = profile?.displayName ?? DEFAULT_GUEST_DISPLAY_NAME;
  const cloudBadge = cloudStateLabel(cloudAuth.status, cloudProfile.status);
  const cloudIdentity = cloudProfile.profile ?? null;
  const cloudDisplayName = cloudPromotion.result?.promotedDisplayName ?? cloudIdentity?.displayName;
  const cloudError =
    cloudAuth.errorMessage
    ?? cloudProfile.errorMessage
    ?? cloudPromotion.errorMessage
    ?? cloudHistory.errorMessage;
  const cloudText = cloudCopyText({
    authStatus: cloudAuth.status,
    hasCloudIdentity: Boolean(cloudIdentity),
    historyCount: localHistory.length,
    promotionStatus: cloudPromotion.status,
    profileStatus: cloudProfile.status,
    totalPromotedMatches: cloudPromotion.result?.totalMatches ?? 0,
  });
  const cloudTitle = cloudTitleText({
    authStatus: cloudAuth.status,
    cloudDisplayName,
  });
  const signedIn = cloudAuth.status === "signed_in" && Boolean(cloudAuth.user);
  const resetDisabled = resetBusy || (signedIn && cloudProfile.status === "loading");
  const resetConfirmationText = signedIn
    ? "This clears cloud history, resets cloud profile values, and clears this device's local cache."
    : "This clears the local profile and match history on this device.";

  async function resetSignedInProfile(): Promise<void> {
    if (cloudAuth.status !== "signed_in" || !cloudAuth.user) {
      return;
    }

    const user = cloudAuth.user;
    const defaultVariant = "freestyle" as const;
    await cloudProfileStore.getState().resetForUser(user, defaultVariant);

    await cloudHistoryStore.getState().clearForUser(user);
    cloudHistoryStore.getState().resetUserCache(user.uid);
    cloudPromotionStore.getState().reset();

    const guestStore = guestProfileStore.getState();
    guestStore.resetGuestProfile();
    guestStore.ensureGuestProfile();
  }

  async function confirmResetProfile(): Promise<void> {
    setResetBusy(true);
    try {
      if (signedIn) {
        await resetSignedInProfile();
      } else {
        const store = guestProfileStore.getState();
        store.resetGuestProfile();
        store.ensureGuestProfile();
      }
      setResetConfirming(false);
    } catch {
      // Store actions already expose foreground reset failures through cloudError.
    } finally {
      setResetBusy(false);
    }
  }

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
            <div className={styles.cloudStatus}>
              <div className={styles.cloudCopy}>
                <p className={styles.cloudTitle}>{cloudTitle}</p>
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
                    {cloudAuth.status === "loading" ? "Checking" : "Sign in"}
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
            {resetConfirming ? (
              <div className={styles.resetConfirm}>
                <p className={styles.resetConfirmText}>{resetConfirmationText}</p>
                <div className={styles.resetConfirmActions}>
                  <button
                    className="uiAction uiActionDanger"
                    disabled={resetDisabled}
                    onClick={() => {
                      void confirmResetProfile();
                    }}
                    type="button"
                  >
                    <span className="uiActionLabel">{resetBusy ? "Resetting" : "Confirm reset"}</span>
                  </button>
                  <button
                    className="uiAction uiActionNeutral"
                    disabled={resetBusy}
                    onClick={() => {
                      setResetConfirming(false);
                    }}
                    type="button"
                  >
                    <span className="uiActionLabel">Cancel</span>
                  </button>
                </div>
              </div>
            ) : (
              <button
                className="uiAction uiActionDanger"
                disabled={resetDisabled}
                onClick={() => {
                  setResetConfirming(true);
                }}
                type="button"
              >
                <Icon className="uiIconDesktop" name="reset" />
                <span className="uiActionLabel">Reset Profile</span>
              </button>
            )}
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
                  const localSide = historyLocalSide(match, historyIdentity);
                  const result = historyResultLabel(match, historyIdentity);

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
                          {historyOpponentLabel(match, historyIdentity, guestDisplayName)}
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
