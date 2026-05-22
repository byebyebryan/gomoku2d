import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useStore } from "zustand";

import { cloudAuthStore } from "../cloud/auth_store";
import { cloudHistoryStore } from "../cloud/cloud_history_store";
import {
  type CloudArchivedMatchStatsV1,
  type CloudMatchHistory,
  type CloudMatchSummaryV1,
} from "../cloud/cloud_profile";
import { cloudProfileStore } from "../cloud/cloud_profile_store";
import { cloudPromotionStore } from "../cloud/cloud_promotion_store";
import {
  matchUserSide,
  savedMatchPlayerForSide,
  savedMatchPlayers,
  savedMatchWinningSide,
  type SavedMatchV2,
  type SavedMatchSide,
} from "../match/saved_match";
import { localProfileStore, type LocalProfileMatchHistory } from "../profile/local_profile_store";
import { createDefaultProfileSettings } from "../profile/profile_settings";
import { useActiveHistory } from "../profile/use_active_history";
import { replayPlayerName, variantLabel } from "../replay/local_replay";
import { Icon } from "../ui/Icon";

import styles from "./ProfileRoute.module.css";

interface HistoryIdentity {
  localProfileId?: string | null;
  profileUid?: string | null;
}

type HistorySyncTone = "busy" | "error" | "pending" | "synced";
type ProfileDangerAction = "delete" | "reset";

interface HistorySyncStatus {
  label: string;
  tone: HistorySyncTone;
}

interface HistoryStats {
  draws: number;
  losses: number;
  matches: number;
  wins: number;
}

const HISTORY_VISIBLE_BATCH_SIZE = 16;

function historyLocalSide(match: SavedMatchV2, identity: HistoryIdentity): SavedMatchSide | null {
  const localSide = matchUserSide(match, identity);
  if (localSide) {
    return localSide;
  }

  return savedMatchPlayers(match).find(({ player }) => player.kind === "human")?.side ?? null;
}

function historyResultLabel(
  match: SavedMatchV2,
  identity: HistoryIdentity,
): "Win" | "Loss" | "Draw" {
  if (match.status === "draw") {
    return "Draw";
  }

  return savedMatchWinningSide(match) === historyLocalSide(match, identity) ? "Win" : "Loss";
}

function emptyHistoryStats(): HistoryStats {
  return {
    draws: 0,
    losses: 0,
    matches: 0,
    wins: 0,
  };
}

function addHistoryStats(target: HistoryStats, source: HistoryStats): void {
  target.draws += source.draws;
  target.losses += source.losses;
  target.matches += source.matches;
  target.wins += source.wins;
}

function statsFromReplayMatches(matches: SavedMatchV2[], identity: HistoryIdentity): HistoryStats {
  const stats = emptyHistoryStats();

  for (const match of matches) {
    stats.matches += 1;
    const result = historyResultLabel(match, identity);
    if (result === "Win") {
      stats.wins += 1;
    } else if (result === "Loss") {
      stats.losses += 1;
    } else {
      stats.draws += 1;
    }
  }

  return stats;
}

function statsFromSummaryMatches(
  summaries: CloudMatchSummaryV1[],
  replayIds: Set<string>,
): HistoryStats {
  const stats = emptyHistoryStats();

  for (const summary of summaries) {
    if (replayIds.has(summary.id)) {
      continue;
    }

    stats.matches += 1;
    if (summary.outcome === "win") {
      stats.wins += 1;
    } else if (summary.outcome === "loss") {
      stats.losses += 1;
    } else {
      stats.draws += 1;
    }
  }

  return stats;
}

function statsFromArchive(archive: CloudArchivedMatchStatsV1): HistoryStats {
  return {
    draws: archive.totals.draws,
    losses: archive.totals.losses,
    matches: archive.totals.matches,
    wins: archive.totals.wins,
  };
}

function statsFromMatchHistory(input: {
  identity: HistoryIdentity;
  replayHistory: SavedMatchV2[];
  sourceHistory: CloudMatchHistory | LocalProfileMatchHistory | null;
}): HistoryStats {
  const stats = statsFromReplayMatches(input.replayHistory, input.identity);
  if (!input.sourceHistory) {
    return stats;
  }

  const replayIds = new Set(input.replayHistory.map((match) => match.id));
  addHistoryStats(stats, statsFromSummaryMatches(input.sourceHistory.summaryMatches, replayIds));
  addHistoryStats(stats, statsFromArchive(input.sourceHistory.archivedStats));
  return stats;
}

function historyOpponentLabel(
  match: SavedMatchV2,
  identity: HistoryIdentity,
  localDisplayName: string,
): string {
  const localSide = historyLocalSide(match, identity);
  const opponentSide = localSide === "black" ? "white" : localSide === "white" ? "black" : null;
  if (!opponentSide) {
    return "Opponent";
  }

  const opponent = savedMatchPlayerForSide(match, opponentSide);
  return `vs ${replayPlayerName(opponent, localDisplayName)}`;
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

function historySyncStatus({
  authStatus,
  hasCloudHistoryLoaded,
  hasPendingError,
  loadStatus,
  pendingCount,
  profileStatus,
  promotionStatus,
  syncStatus,
}: {
  authStatus: ReturnType<typeof cloudAuthStore.getState>["status"];
  hasCloudHistoryLoaded: boolean;
  hasPendingError: boolean;
  loadStatus: ReturnType<typeof cloudHistoryStore.getState>["loadStatus"];
  pendingCount: number;
  profileStatus: ReturnType<typeof cloudProfileStore.getState>["status"];
  promotionStatus: ReturnType<typeof cloudPromotionStore.getState>["status"];
  syncStatus: ReturnType<typeof cloudHistoryStore.getState>["syncStatus"];
}): HistorySyncStatus | null {
  if (authStatus !== "signed_in") {
    return null;
  }

  if (profileStatus === "loading" || loadStatus === "loading") {
    return { label: "Loading", tone: "busy" };
  }

  if (
    loadStatus === "error"
    || promotionStatus === "error"
    || hasPendingError
  ) {
    return { label: "Retrying", tone: "error" };
  }

  if (!hasCloudHistoryLoaded) {
    return { label: "Loading", tone: "busy" };
  }

  if (syncStatus === "syncing" || promotionStatus === "promoting") {
    return { label: "Syncing", tone: "busy" };
  }

  if (pendingCount > 0) {
    return { label: "Queued", tone: "pending" };
  }

  if (profileStatus === "ready") {
    return { label: "Synced", tone: "synced" };
  }

  return null;
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
  localMatchesSynced,
  promotionStatus,
  profileStatus,
}: {
  authStatus: ReturnType<typeof cloudAuthStore.getState>["status"];
  hasCloudIdentity: boolean;
  localMatchesSynced: number;
  promotionStatus: ReturnType<typeof cloudPromotionStore.getState>["status"];
  profileStatus: ReturnType<typeof cloudProfileStore.getState>["status"];
}): string {
  if (authStatus === "unconfigured") {
    return "Cloud sync unavailable.";
  }

  if (authStatus === "loading") {
    return "Checking cloud sync...";
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
    return "Sync will retry.";
  }

  if (promotionStatus === "complete") {
    return localMatchesSynced > 0
      ? "Local history synced to cloud."
      : "Cloud history enabled.";
  }

  return "Cloud history enabled.";
}

function cloudTitleText({
  authStatus,
}: {
  authStatus: ReturnType<typeof cloudAuthStore.getState>["status"];
}): string {
  if (authStatus === "unconfigured") {
    return "Local profile";
  }

  if (authStatus === "loading") {
    return "Checking sign-in";
  }

  if (authStatus === "signed_in") {
    return "Cloud profile";
  }

  if (authStatus === "error") {
    return "Sign-in needs attention";
  }

  return "Cloud profile";
}

export function ProfileRoute() {
  const navigate = useNavigate();
  const [confirmingProfileAction, setConfirmingProfileAction] = useState<ProfileDangerAction | null>(null);
  const [profileActionBusy, setProfileActionBusy] = useState(false);
  const [visibleHistoryCount, setVisibleHistoryCount] = useState(HISTORY_VISIBLE_BATCH_SIZE);
  const {
    cloudAuth,
    cloudHistory,
    cloudProfile,
    cloudUserCache,
    hasPendingCloudMatchError,
    history,
    localDisplayName,
    localProfile: profile,
    pendingCloudMatchCount,
    sourceHistory,
  } = useActiveHistory();
  const cloudPromotion = useStore(cloudPromotionStore, (state) => state);

  useEffect(() => {
    localProfileStore.getState().ensureLocalProfile();
  }, []);

  useEffect(() => {
    setVisibleHistoryCount(HISTORY_VISIBLE_BATCH_SIZE);
  }, [cloudAuth.status, cloudAuth.user?.uid, profile?.id]);

  const visibleHistory = history.slice(0, visibleHistoryCount);
  const hiddenHistoryCount = Math.max(0, history.length - visibleHistory.length);
  const historyIdentity: HistoryIdentity = {
    localProfileId: profile?.id,
    profileUid: cloudAuth.status === "signed_in" ? cloudAuth.user?.uid : null,
  };
  const stats = statsFromMatchHistory({
    identity: historyIdentity,
    replayHistory: history,
    sourceHistory,
  });
  const cloudBadge = cloudStateLabel(cloudAuth.status, cloudProfile.status);
  const cloudIdentity = cloudProfile.profile ?? null;
  const cloudError =
    cloudAuth.errorMessage
    ?? cloudProfile.errorMessage
    ?? cloudPromotion.errorMessage
    ?? cloudHistory.errorMessage;
  const cloudText = cloudCopyText({
    authStatus: cloudAuth.status,
    hasCloudIdentity: Boolean(cloudIdentity),
    localMatchesSynced: cloudPromotion.result?.localMatchesSynced ?? 0,
    promotionStatus: cloudPromotion.status,
    profileStatus: cloudProfile.status,
  });
  const cloudTitle = cloudTitleText({
    authStatus: cloudAuth.status,
  });
  const historyStatus = historySyncStatus({
    authStatus: cloudAuth.status,
    hasCloudHistoryLoaded: Boolean(cloudUserCache?.loadedAt),
    hasPendingError: hasPendingCloudMatchError,
    loadStatus: cloudHistory.loadStatus,
    pendingCount: pendingCloudMatchCount,
    profileStatus: cloudProfile.status,
    promotionStatus: cloudPromotion.status,
    syncStatus: cloudHistory.syncStatus,
  });
  const historyStatusClassName = historyStatus
    ? [
      styles.historySyncStatus,
      historyStatus.tone === "busy"
        ? styles.historySyncStatusBusy
        : historyStatus.tone === "error"
          ? styles.historySyncStatusError
          : historyStatus.tone === "pending"
            ? styles.historySyncStatusPending
            : styles.historySyncStatusSynced,
    ].join(" ")
    : "";
  const signedIn = cloudAuth.status === "signed_in" && Boolean(cloudAuth.user);
  const profileActionDisabled = profileActionBusy || (signedIn && cloudProfile.status === "loading");
  const confirmationText = confirmingProfileAction === "delete"
    ? "Delete cloud profile, then sign out? Local profile stays on this device."
    : signedIn
      ? "Reset cloud profile and clear both cloud and local match history?"
      : "Reset local profile and clear local match history?";
  const confirmationButtonText = confirmingProfileAction === "delete" ? "Delete" : "Reset";

  async function resetSignedInProfile(): Promise<void> {
    if (cloudAuth.status !== "signed_in" || !cloudAuth.user) {
      return;
    }

    const user = cloudAuth.user;
    await cloudProfileStore.getState().resetForUser(user, createDefaultProfileSettings());

    await cloudHistoryStore.getState().clearForUser(user);
    cloudHistoryStore.getState().resetUserCache(user.uid);
    cloudPromotionStore.getState().reset();

    const localStore = localProfileStore.getState();
    localStore.resetLocalProfile();
    localStore.ensureLocalProfile();
  }

  async function deleteSignedInProfile(): Promise<void> {
    if (cloudAuth.status !== "signed_in" || !cloudAuth.user) {
      return;
    }

    const user = cloudAuth.user;
    await cloudHistoryStore.getState().clearForUser(user);
    await cloudProfileStore.getState().deleteForUser(user);
    cloudHistoryStore.getState().resetUserCache(user.uid);
    cloudPromotionStore.getState().reset();
    await cloudAuthStore.getState().signOut();
  }

  async function confirmProfileAction(): Promise<void> {
    const action = confirmingProfileAction;
    if (!action) {
      return;
    }

    setProfileActionBusy(true);
    try {
      if (action === "delete") {
        await deleteSignedInProfile();
      } else if (signedIn) {
        await resetSignedInProfile();
      } else {
        const store = localProfileStore.getState();
        store.resetLocalProfile();
        store.ensureLocalProfile();
      }
      setConfirmingProfileAction(null);
    } catch {
      // Store actions already expose foreground failures through cloudError.
    } finally {
      setProfileActionBusy(false);
    }
  }

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerCopy}>
          <p className="uiPageEyebrow">Player record</p>
          <h1 className={styles.title}>Profile</h1>
        </div>
        <div className={styles.headerActions}>
          <Link aria-label="Play" className="uiAction uiActionPrimary" to="/match/local">
            <Icon className="uiIconDesktop" name="play" />
            <span className="uiActionLabel">Play</span>
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
        <aside className={styles.sidePanel}>
          <section className={`${styles.sideSection} ${styles.identitySection}`}>
            <div className={styles.sectionHeader}>
              <p className="uiSectionLabel">Identity</p>
              <p className={styles.badge}>{cloudBadge}</p>
            </div>
            <label className={styles.field}>
              <span className={styles.fieldLabel}>Name</span>
              <input
                className="uiInput"
                onChange={(event) => {
                  localProfileStore.getState().renameDisplayName(event.target.value);
                }}
                placeholder="Name"
                type="text"
                value={localDisplayName}
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

          <section className={`${styles.sideSection} ${styles.resetSection}`}>
            {confirmingProfileAction ? (
              <div className={styles.resetConfirm}>
                <p className={styles.resetConfirmText}>{confirmationText}</p>
                <div className={styles.resetConfirmActions}>
                  <div className={styles.resetConfirmPrimaryActions}>
                    <button
                      className="uiAction uiActionDanger"
                      disabled={profileActionDisabled}
                      onClick={() => {
                        void confirmProfileAction();
                      }}
                      type="button"
                    >
                      <span className="uiActionLabel">{confirmationButtonText}</span>
                    </button>
                    <button
                      className="uiAction uiActionNeutral"
                      disabled={profileActionBusy}
                      onClick={() => {
                        setConfirmingProfileAction(null);
                      }}
                      type="button"
                    >
                      <span className="uiActionLabel">Cancel</span>
                    </button>
                  </div>
                  {signedIn && confirmingProfileAction === "reset" ? (
                    <button
                      className="uiAction uiActionNeutral"
                      disabled={profileActionBusy}
                      onClick={() => {
                        setConfirmingProfileAction("delete");
                      }}
                      type="button"
                    >
                      <span className="uiActionLabel">Delete Cloud</span>
                    </button>
                  ) : null}
                </div>
              </div>
            ) : (
              <button
                className={`uiAction uiActionNeutral ${styles.resetTrigger}`}
                disabled={profileActionDisabled}
                onClick={() => {
                  setConfirmingProfileAction("reset");
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
            {historyStatus ? (
              <p className={historyStatusClassName} aria-live="polite">{historyStatus.label}</p>
            ) : null}
          </div>
          <div className={styles.summaryGrid}>
            <article className={styles.summaryTile}>
              <span className={styles.summaryValue}>{stats.matches}</span>
              <span className={styles.summaryLabel}>Matches</span>
            </article>
            <article className={styles.summaryTile}>
              <span className={styles.summaryValue}>{stats.wins}</span>
              <span className={styles.summaryLabel}>Wins</span>
            </article>
            <article className={styles.summaryTile}>
              <span className={styles.summaryValue}>{stats.losses}</span>
              <span className={styles.summaryLabel}>Losses</span>
            </article>
            <article className={styles.summaryTile}>
              <span className={styles.summaryValue}>{stats.draws}</span>
              <span className={styles.summaryLabel}>Draws</span>
            </article>
          </div>
          <div
            className={`${styles.historyHead} ${
              history.length === 0 ? styles.historyHeadEmpty : ""
            }`}
            aria-hidden="true"
          >
            {history.length > 0 ? (
              <>
              <span className={styles.historyHeadLabel}>Result</span>
              <span className={styles.historyHeadLabel}>Opponent</span>
              <span className={`${styles.historyHeadLabel} ${styles.historyHeadSide}`}>Side</span>
              <span className={`${styles.historyHeadLabel} ${styles.historyHeadRule}`}>Rule</span>
              <span className={`${styles.historyHeadLabel} ${styles.historyHeadMoves}`}>Moves</span>
              <span className={`${styles.historyHeadLabel} ${styles.historyHeadPlayed}`}>Played</span>
              <span className={styles.historyHeadSpacer} />
              </>
            ) : null}
          </div>
          <div className={styles.historyBody}>
            {history.length > 0 ? (
              <ol className={styles.historyList}>
                {visibleHistory.map((match) => {
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
                          {historyOpponentLabel(match, historyIdentity, localDisplayName)}
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
                        >
                          {historySideLabel(localSide)}
                        </p>
                        <p className={`${styles.historyField} ${styles.historyRule}`}>
                          {variantLabel(match.ruleset)}
                        </p>
                        <p className={`${styles.historyField} ${styles.historyMoves}`}>
                          {`Moves ${match.move_count}`}
                        </p>
                        <p className={`${styles.historyPlayed} ${styles.historyPlayedField}`}>
                          <span className={styles.historyDate}>{historyDateLabel(match.saved_at)}</span>
                          <span className={styles.historyTime}>{historyTimeLabel(match.saved_at)}</span>
                        </p>
                      </div>
                      <button
                        aria-label="Replay"
                        className={`uiAction uiActionSecondary ${styles.historyReplayAction}`}
                        onClick={() => {
                          navigate(`/replay/${match.id}`);
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
            ) : null}
            {hiddenHistoryCount > 0 ? (
              <button
                className={`uiAction uiActionNeutral ${styles.historyMoreAction}`}
                onClick={() => {
                  setVisibleHistoryCount((count) =>
                    Math.min(history.length, count + HISTORY_VISIBLE_BATCH_SIZE)
                  );
                }}
                type="button"
              >
                <span className="uiActionLabel">Show more</span>
              </button>
            ) : null}
          </div>
        </section>
      </section>
    </main>
  );
}
