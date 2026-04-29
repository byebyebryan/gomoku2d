import { useEffect, useRef, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useStore } from "zustand";

import { cloudAuthStore, type CloudAuthUser } from "../cloud/auth_store";
import { cloudHistoryStore } from "../cloud/cloud_history_store";
import {
  cloudMatchHistoryHasMatch,
  type CloudArchivedMatchStatsV1,
  type CloudMatchHistory,
  type CloudMatchSummaryV1,
} from "../cloud/cloud_profile";
import { cloudProfileStore } from "../cloud/cloud_profile_store";
import { cloudPromotionStore } from "../cloud/cloud_promotion_store";
import { flushCloudProfileSync } from "../cloud/cloud_sync";
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
  DEFAULT_LOCAL_DISPLAY_NAME,
  localProfileStore,
  type LocalProfileMatchHistory,
} from "../profile/local_profile_store";
import { replayPlayerName, variantLabel } from "../replay/local_replay";
import { Icon } from "../ui/Icon";

import styles from "./ProfileRoute.module.css";

interface HistoryIdentity {
  localProfileId?: string | null;
  profileUid?: string | null;
}

type HistorySyncTone = "busy" | "error" | "pending" | "synced";

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

function syncCloudHistoryForUser(user: CloudAuthUser, historyResetAt: string | null, flushProfile = false): void {
  const profileSync = flushProfile ? flushCloudProfileSync(user) : Promise.resolve(null);

  void profileSync.then((cloudProfile) => {
    const activeProfile = cloudProfile ?? cloudProfileStore.getState().profile;
    const activeHistoryResetAt = activeProfile?.resetAt ?? historyResetAt;
    if (activeProfile) {
      cloudHistoryStore.getState().loadFromProfile(user, activeProfile, activeHistoryResetAt);
    } else {
      void cloudHistoryStore.getState().loadForUser(user, activeHistoryResetAt);
    }

    void cloudHistoryStore.getState().syncPendingForUser(user, activeHistoryResetAt);
  });
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

function statsFromReplayMatches(matches: SavedMatchV1[], identity: HistoryIdentity): HistoryStats {
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
  replayHistory: SavedMatchV1[];
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
  match: SavedMatchV1,
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

function shouldAdoptCloudDisplayName(
  localDisplayName: string | undefined,
  cloudDisplayName: string | undefined,
): cloudDisplayName is string {
  return (
    localDisplayName === DEFAULT_LOCAL_DISPLAY_NAME
    && Boolean(cloudDisplayName?.trim())
    && cloudDisplayName !== DEFAULT_LOCAL_DISPLAY_NAME
  );
}

export function ProfileRoute() {
  const navigate = useNavigate();
  const initialPromotionKeyRef = useRef<string | null>(null);
  const [resetConfirming, setResetConfirming] = useState(false);
  const [resetBusy, setResetBusy] = useState(false);
  const [visibleHistoryCount, setVisibleHistoryCount] = useState(HISTORY_VISIBLE_BATCH_SIZE);
  const cloudAuth = useStore(cloudAuthStore, (state) => state);
  const cloudHistory = useStore(cloudHistoryStore, (state) => state);
  const cloudProfile = useStore(cloudProfileStore, (state) => state);
  const cloudPromotion = useStore(cloudPromotionStore, (state) => state);
  const localMatchHistory = useStore(localProfileStore, (state) => state.matchHistory);
  const localHistory = localMatchHistory.replayMatches;
  const profile = useStore(localProfileStore, (state) => state.profile);
  const settings = useStore(localProfileStore, (state) => state.settings);

  useEffect(() => {
    localProfileStore.getState().ensureLocalProfile();
  }, []);

  useEffect(() => {
    setVisibleHistoryCount(HISTORY_VISIBLE_BATCH_SIZE);
  }, [cloudAuth.status, cloudAuth.user?.uid, profile?.id]);

  useEffect(() => {
    cloudAuthStore.getState().start();

    return () => {
      cloudAuthStore.getState().stop();
    };
  }, []);

  useEffect(() => {
    if (cloudAuth.status === "signed_in" && cloudAuth.user) {
      void cloudProfileStore.getState().loadForUser(
        cloudAuth.user,
        localProfileStore.getState().settings.preferredVariant,
      );
      return;
    }

    cloudProfileStore.getState().reset();
  }, [cloudAuth.status, cloudAuth.user]);

  useEffect(() => {
    const cloudDisplayName = cloudProfile.profile?.displayName;
    if (
      cloudAuth.status === "signed_in"
      && cloudProfile.status === "ready"
      && shouldAdoptCloudDisplayName(profile?.displayName, cloudDisplayName)
    ) {
      localProfileStore.getState().renameDisplayName(cloudDisplayName);
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
    const initialPromotionKey = (
      cloudAuth.status === "signed_in"
      && cloudAuth.user
      && cloudProfile.profile
    )
      ? [
        cloudAuth.user.uid,
        cloudProfile.profile.resetAt ?? "",
      ].join(":")
      : null;

    if (
      cloudAuth.status === "signed_in"
      && cloudAuth.user
      && cloudProfile.status === "ready"
      && profile
      && !waitingForCloudNameAdoption
      && initialPromotionKey
      && initialPromotionKeyRef.current !== initialPromotionKey
    ) {
      initialPromotionKeyRef.current = initialPromotionKey;
      const user = cloudAuth.user;
      void flushCloudProfileSync(user, { localMatchHistory: localMatchHistory }).then((cloudProfile) => {
        if (!cloudProfile) {
          return;
        }

        cloudHistoryStore.getState().loadFromProfile(user, cloudProfile);
        for (const match of localHistory) {
          if (!cloudMatchHistoryHasMatch(cloudProfile.matchHistory, match.id)) {
            void cloudHistoryStore.getState().syncMatchForUser(user, match, cloudProfile.resetAt);
          }
        }
      });
      return undefined;
    }

    if (cloudAuth.status !== "signed_in" || cloudProfile.status !== "ready") {
      initialPromotionKeyRef.current = null;
      cloudPromotionStore.getState().reset();
    }
    return undefined;
  }, [
    cloudAuth.status,
    cloudAuth.user,
    cloudProfile.profile?.displayName,
    cloudProfile.profile?.resetAt,
    cloudProfile.status,
    localHistory,
    localMatchHistory,
    profile,
  ]);

  useEffect(() => {
    if (cloudAuth.status !== "signed_in" || !cloudAuth.user || cloudProfile.status !== "ready") {
      return;
    }

    const user = cloudAuth.user;
    const historyResetAt = cloudProfile.profile?.resetAt ?? null;
    syncCloudHistoryForUser(user, historyResetAt);
  }, [
    cloudAuth.status,
    cloudAuth.user,
    cloudProfile.profile?.resetAt,
    cloudProfile.status,
    cloudPromotion.status,
  ]);

  useEffect(() => {
    if (cloudAuth.status !== "signed_in" || !cloudAuth.user || cloudProfile.status !== "ready") {
      return;
    }

    const user = cloudAuth.user;
    const historyResetAt = cloudProfile.profile?.resetAt ?? null;
    const retryCloudHistory = () => {
      syncCloudHistoryForUser(user, historyResetAt, true);
    };

    window.addEventListener("online", retryCloudHistory);
    return () => {
      window.removeEventListener("online", retryCloudHistory);
    };
  }, [
    cloudAuth.status,
    cloudAuth.user,
    cloudProfile.profile?.resetAt,
    cloudProfile.status,
  ]);

  const cloudCache =
    cloudAuth.status === "signed_in" && cloudAuth.user
      ? cloudHistory.users[cloudAuth.user.uid]?.cachedMatches ?? []
      : [];
  const cloudUserCache =
    cloudAuth.status === "signed_in" && cloudAuth.user
      ? cloudHistory.users[cloudAuth.user.uid]
      : null;
  const pendingCloudMatches = cloudUserCache?.pendingMatches ?? {};
  const pendingCloudMatchCount = Object.keys(pendingCloudMatches).length;
  const hasPendingCloudMatchError = Object.values(cloudUserCache?.sync ?? {}).some((sync) => (
    sync.status === "error" && sync.matchId in pendingCloudMatches
  ));
  const history = resolveActiveHistory({
    cloudHistory: cloudCache,
    historyResetAt: cloudAuth.status === "signed_in" ? cloudProfile.profile?.resetAt : null,
    localHistory,
  });
  const visibleHistory = history.slice(0, visibleHistoryCount);
  const hiddenHistoryCount = Math.max(0, history.length - visibleHistory.length);
  const historyIdentity: HistoryIdentity = {
    localProfileId: profile?.id,
    profileUid: cloudAuth.status === "signed_in" ? cloudAuth.user?.uid : null,
  };
  const sourceHistory =
    cloudAuth.status === "signed_in"
      ? cloudProfile.profile?.matchHistory ?? null
      : localMatchHistory;
  const stats = statsFromMatchHistory({
    identity: historyIdentity,
    replayHistory: history,
    sourceHistory,
  });
  const localDisplayName = profile?.displayName ?? DEFAULT_LOCAL_DISPLAY_NAME;
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
  const resetDisabled = resetBusy || (signedIn && cloudProfile.status === "loading");
  const resetConfirmationText = signedIn
    ? "Reset cloud profile and clear both cloud and local match history?"
    : "Reset local profile and clear local match history?";

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

    const localStore = localProfileStore.getState();
    localStore.resetLocalProfile();
    localStore.ensureLocalProfile();
  }

  async function confirmResetProfile(): Promise<void> {
    setResetBusy(true);
    try {
      if (signedIn) {
        await resetSignedInProfile();
      } else {
        const store = localProfileStore.getState();
        store.resetLocalProfile();
        store.ensureLocalProfile();
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
          <p className="uiPageEyebrow">Player record</p>
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
                    localProfileStore.getState().updateSettings({ preferredVariant: variant });
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
                    <span className="uiActionLabel">Reset</span>
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
                className={`uiAction uiActionNeutral ${styles.resetTrigger}`}
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
