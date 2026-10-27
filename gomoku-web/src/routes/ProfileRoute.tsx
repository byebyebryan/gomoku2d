import { useEffect, useState, type KeyboardEvent } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { useStore } from "zustand";

import { useDocumentTitle } from "../app/useDocumentTitle";
import { cloudAuthStore } from "../cloud/auth_store";
import { cloudHistoryStore } from "../cloud/cloud_history_store";
import { cloudProfileStore } from "../cloud/cloud_profile_store";
import { clearSharingData } from "../cloud/sharing_lifecycle";
import { loadPrivateLibrary, savePrivateLibrary } from "../cloud/sharing_private";
import { replayLibraryStore } from "../replay/replay_library_store";
import { replayLibrarySync } from "../replay/replay_library_sync";
import { cloudPromotionStore } from "../cloud/cloud_promotion_store";
import {
  savedMatchIsAfterReset,
  savedMatchPlayerForSide,
  type SavedMatchV2,
  type SavedMatchSide,
} from "../match/saved_match";
import { localProfileStore } from "../profile/local_profile_store";
import { createDefaultProfileSettings } from "../profile/profile_settings";
import { useActiveHistory } from "../profile/use_active_history";
import {
  playerSideForMatch as historyLocalSide,
  playerResultForMatch as historyResultLabel,
  playerStatsFromHistory,
} from "../profile/player_stats";
import { replayPlayerName, variantLabel } from "../replay/local_replay";
import { clearReplayAnalysisCache } from "../replay/replay_analysis_cache";
import { Icon } from "../ui/Icon";

import styles from "./ProfileRoute.module.css";
import { PublicProfileManager } from "./PublicProfileManager";
import { ReplayLibraryPanel } from "./ReplayLibraryPanel";

interface HistoryIdentity {
  localProfileId?: string | null;
  profileUid?: string | null;
}

type HistorySyncTone = "busy" | "error" | "pending" | "synced";
type ProfileDangerAction = "delete" | "reset";

class ProfileActionAccountChanged extends Error {
  constructor() { super("Account changed before cleanup finished. Sign in to the original account and retry; some cleanup may already have completed."); }
}

interface HistorySyncStatus {
  label: string;
  tone: HistorySyncTone;
}

const HISTORY_VISIBLE_BATCH_SIZE = 16;
const COMPACT_PROFILE_QUERY = "(max-width: 720px), (max-width: 980px) and (max-height: 500px)";

function compactProfile(): boolean {
  return typeof window.matchMedia === "function" && window.matchMedia(COMPACT_PROFILE_QUERY).matches;
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

    return "Sign in to sync this profile and its history.";
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
      : "Cloud sync is on.";
  }

  return "Cloud sync is on.";
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
  useDocumentTitle("Profile");
  const [recordTab, setRecordTab] = useState<"games" | "saved">("games");
  const [searchParams, setSearchParams] = useSearchParams();
  const activeTab = searchParams.get("tab") === "public" ? "public" : recordTab;

  const navigate = useNavigate();
  const [confirmingProfileAction, setConfirmingProfileAction] = useState<ProfileDangerAction | null>(null);
  const [profileActionBusy, setProfileActionBusy] = useState(false);
  const [profileActionError, setProfileActionError] = useState<string | null>(null);
  const [visibleHistoryCount, setVisibleHistoryCount] = useState(HISTORY_VISIBLE_BATCH_SIZE);
  const [isCompact, setIsCompact] = useState(compactProfile);
  const [accountOpen, setAccountOpen] = useState(() => !compactProfile());
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
    if (typeof window.matchMedia !== "function") return;
    const query = window.matchMedia(COMPACT_PROFILE_QUERY);
    const update = () => {
      setIsCompact(query.matches);
      setAccountOpen(!query.matches);
    };
    update();
    if (typeof query.addEventListener === "function") {
      query.addEventListener("change", update);
      return () => query.removeEventListener("change", update);
    }
    query.addListener(update);
    return () => query.removeListener(update);
  }, []);

  useEffect(() => { setConfirmingProfileAction(null); setProfileActionError(null); }, [cloudAuth.user?.uid]);

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
  const stats = playerStatsFromHistory({
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
  useEffect(() => {
    if (cloudError || profileActionError) setAccountOpen(true);
  }, [cloudError, profileActionError]);
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
  const cachedMatchIds = new Set(cloudUserCache?.cachedMatches.map((match) => match.id) ?? []);
  const currentProfileMatchIds = cloudProfile.profile?.matchHistory.replayMatches
    .filter((match) => savedMatchIsAfterReset(match, cloudProfile.profile?.resetAt))
    .map((match) => match.id) ?? [];
  const profileHistoryMatchesCache = currentProfileMatchIds.every((matchId) => cachedMatchIds.has(matchId));
  const statsReady = Boolean(
    signedIn
    && cloudProfile.status === "ready"
    && cloudProfile.profile?.uid === cloudAuth.user?.uid
    && cloudHistory.loadStatus === "ready"
    && cloudUserCache?.loadedAt
    && profileHistoryMatchesCache
    && cloudHistory.syncStatus !== "syncing"
    && cloudHistory.syncStatus !== "error"
    && pendingCloudMatchCount === 0
    && !hasPendingCloudMatchError
    && cloudPromotion.status !== "promoting"
    && cloudPromotion.status !== "error",
  );
  const statsBlocker = !signedIn
    ? "Sign in before sharing public statistics."
    : cloudProfile.status !== "ready" || cloudProfile.profile?.uid !== cloudAuth.user?.uid
      ? "Wait for this account’s cloud profile to load."
      : cloudHistory.loadStatus !== "ready" || !cloudUserCache?.loadedAt
        ? "Wait for this account’s private game history to finish loading."
        : !profileHistoryMatchesCache
          ? "Wait for the loaded game history to match this account’s cloud profile."
        : cloudPromotion.status === "promoting"
          ? "Wait for local history promotion to finish."
          : cloudPromotion.status === "error"
            ? "Resolve the history promotion error before sharing statistics."
            : cloudHistory.syncStatus === "syncing" || pendingCloudMatchCount > 0
              ? "Wait for pending game history sync to finish."
              : cloudHistory.syncStatus === "error" || hasPendingCloudMatchError
                ? "Resolve the game history sync error before sharing statistics."
                : "Your complete game history is not ready to share.";

  function selectTab(tab: "games" | "saved" | "public"): void {
    if (tab === "public") {
      setSearchParams((current) => {
        const next = new URLSearchParams(current);
        next.set("tab", "public");
        return next;
      });
      return;
    }
    setRecordTab(tab);
    if (searchParams.has("tab")) {
      setSearchParams((current) => {
        const next = new URLSearchParams(current);
        next.delete("tab");
        return next;
      });
    }
  }

  function handleTabKeyDown(event: KeyboardEvent<HTMLButtonElement>): void {
    const tabs = ["games", "saved", "public"] as const;
    const currentIndex = tabs.indexOf(activeTab);
    let nextIndex: number | null = null;
    if (event.key === "ArrowLeft") nextIndex = (currentIndex + tabs.length - 1) % tabs.length;
    if (event.key === "ArrowRight") nextIndex = (currentIndex + 1) % tabs.length;
    if (event.key === "Home") nextIndex = 0;
    if (event.key === "End") nextIndex = tabs.length - 1;
    if (nextIndex === null) return;
    const nextTab = tabs[nextIndex]!;
    selectTab(nextTab);
    document.getElementById(`${nextTab}-tab`)?.focus();
    event.preventDefault();
  }
  const profileActionDisabled =
    profileActionBusy
    || cloudAuth.status === "loading"
    || (signedIn && cloudProfile.status === "loading");
  const confirmationText = confirmingProfileAction === "delete"
    ? "Delete your cloud profile, history, saved replays, public name, shared statistics and publications, then sign out? Local copies stay on this device. Empty revision markers remain to block stale writes. Copied replay links still work."
    : signedIn
      ? "Reset private settings, games and saved replays here and in the cloud? Local replay analyses are cleared. Your public profile, shared statistics and publications stay unchanged; empty cloud revision markers remain."
      : "Reset this profile? This clears settings, games, saved replays and replay analyses on this device.";
  const confirmationButtonText = confirmingProfileAction === "delete" ? "Delete" : "Reset";

  function resetLocalProfileData(): void {
    const localStore = localProfileStore.getState();
    localStore.resetLocalProfile();
    clearReplayAnalysisCache();
    localStore.ensureLocalProfile();
  }

  async function resetSignedInProfile(): Promise<void> {
    if (cloudAuth.status !== "signed_in" || !cloudAuth.user) {
      return;
    }

    const user = cloudAuth.user;
    const localProfileId = localProfileStore.getState().profile?.id;
    const isCurrent = () => cloudAuthStore.getState().user === user
      && localProfileStore.getState().profile?.id === localProfileId;
    const library = await loadPrivateLibrary(user.uid);
    if (!isCurrent()) throw new ProfileActionAccountChanged();
    const cleared = await savePrivateLibrary(user.uid, [], library?.revision ?? null);
    if (!isCurrent()) throw new ProfileActionAccountChanged();
    await replayLibraryStore.getState().replaceEntries(`user:${user.uid}`, [], isCurrent);
    if (!isCurrent()) throw new ProfileActionAccountChanged();
    replayLibrarySync.acceptCleared(user.uid, cleared);
    await cloudProfileStore.getState().resetForUser(user, createDefaultProfileSettings());
    if (!isCurrent()) throw new ProfileActionAccountChanged();
    await cloudHistoryStore.getState().clearForUser(user);
    if (!isCurrent()) throw new ProfileActionAccountChanged();
    cloudHistoryStore.getState().resetUserCache(user.uid);
    cloudPromotionStore.getState().reset();
    resetLocalProfileData();
  }

  async function deleteSignedInProfile(): Promise<void> {
    if (cloudAuth.status !== "signed_in" || !cloudAuth.user) {
      return;
    }

    const user = cloudAuth.user;
    const isCurrent = () => cloudAuthStore.getState().user === user;
    const cleared = await clearSharingData(user.uid);
    if (!isCurrent()) throw new ProfileActionAccountChanged();
    // Preserve local copies, but require review before any future re-upload.
    if (replayLibraryStore.getState().owners[`user:${user.uid}`]?.entries.length) {
      replayLibraryStore.getState().markDirty(`user:${user.uid}`);
    }
    replayLibrarySync.acceptCleared(user.uid, cleared.privateLibrary);
    await cloudHistoryStore.getState().clearForUser(user);
    if (!isCurrent()) throw new ProfileActionAccountChanged();
    await cloudProfileStore.getState().deleteForUser(user);
    if (!isCurrent()) throw new ProfileActionAccountChanged();
    cloudHistoryStore.getState().resetUserCache(user.uid);
    cloudPromotionStore.getState().reset();
    await cloudAuthStore.getState().signOut();
  }

  async function confirmProfileAction(): Promise<void> {
    const action = confirmingProfileAction;
    if (!action) {
      return;
    }

    const actionUser = cloudAuthStore.getState().user;
    const actionLocalId = localProfileStore.getState().profile?.id;
    setProfileActionBusy(true);
    setProfileActionError(null);
    try {
      if (action === "delete") {
        await deleteSignedInProfile();
      } else if (signedIn) {
        await resetSignedInProfile();
      } else {
        await replayLibraryStore.getState().clearOwner("guest");
        resetLocalProfileData();
      }
      setConfirmingProfileAction(null);
    } catch (error) {
      if (error instanceof ProfileActionAccountChanged ||
        (cloudAuthStore.getState().user === actionUser && localProfileStore.getState().profile?.id === actionLocalId)) {
        setProfileActionError(error instanceof Error ? error.message : "The operation could not finish. Retry to complete cleanup.");
      }
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
                aria-label="Name"
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
                  aria-label="Sign out"
                  className="uiAction uiActionNeutral"
                  disabled={profileActionBusy}
                  onClick={() => {
                    void cloudAuthStore.getState().signOut();
                  }}
                  type="button"
                >
                  <Icon name="signOut" /><span className="uiActionLabel">Sign out</span>
                </button>
              ) : (
                <button
                  aria-label={cloudAuth.status === "loading" ? "Checking" : "Sign in"}
                  className="uiAction uiActionSecondary"
                  disabled={!cloudAuth.isConfigured || cloudAuth.status === "loading"}
                  onClick={() => {
                    void cloudAuthStore.getState().signInWithGoogle();
                  }}
                  type="button"
                >
                  <Icon name="signIn" />
                  <span className="uiActionLabel">
                    {cloudAuth.status === "loading" ? "Checking" : "Sign in"}
                  </span>
                </button>
              )}
            </div>
          </section>

          <div className="uiDivider" />

          <details key={isCompact ? "compact" : "wide"} className={styles.accountControls} open={!isCompact || accountOpen}
            onToggle={(event) => { if (isCompact) setAccountOpen(event.currentTarget.open); }}>
            <summary><Icon name="settings" /><span>Profile controls</span></summary>
            {isCompact && <div className={styles.mobileCloudDetails}>
              <p className={styles.cloudTitle}>{cloudTitle}</p>
              <p className={styles.cloudText}>{cloudText}</p>
              {cloudError ? <p className={styles.cloudError} role="alert">{cloudError}</p> : null}
            </div>}
            <section className={`${styles.sideSection} ${styles.resetSection}`}>
              {profileActionError ? <p role="alert">{profileActionError}</p> : null}
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
                        <Icon name={confirmingProfileAction === "delete" ? "trash" : "reset"} />
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
                        <Icon name="trash" /><span className="uiActionLabel">Delete Cloud</span>
                      </button>
                    ) : null}
                  </div>
                </div>
              ) : (
                <button
                  className={`uiAction uiActionNeutral ${styles.resetTrigger}`}
                  disabled={profileActionDisabled}
                  onClick={() => {
                    setProfileActionError(null);
                    setConfirmingProfileAction("reset");
                  }}
                  type="button"
                >
                  <Icon className="uiIconDesktop" name="reset" />
                  <span className="uiActionLabel">Reset Profile</span>
                </button>
              )}
            </section>
          </details>
        </aside>

        <section className={styles.recordPanel}>
          <div className={styles.recordHeader}>
            <div role="tablist" aria-label="Profile sections" className={styles.recordTabs}>
              {([['games', 'My games'], ['saved', 'Saved replays'], ['public', 'Public profile']] as const).map(([tab, label]) =>
                <button key={tab} id={`${tab}-tab`} type="button" role="tab" aria-selected={activeTab === tab}
                  aria-controls={`${tab}-records`} tabIndex={activeTab === tab ? 0 : -1}
                  className={`uiAction ${activeTab === tab ? 'uiActionSecondary' : 'uiActionNeutral'}`}
                  onClick={() => selectTab(tab)} onKeyDown={handleTabKeyDown}>{label}</button>)}
            </div>
            {historyStatus && activeTab === "games" ? (
              <p className={historyStatusClassName} aria-live="polite">{historyStatus.label}</p>
            ) : null}
          </div>
          <div id={`${activeTab}-records`} role="tabpanel" aria-labelledby={`${activeTab}-tab`}
            className={activeTab === 'games' ? styles.recordContents : styles.libraryContents}>
          {activeTab === 'public' ? (
            <PublicProfileManager stats={stats} statsReady={statsReady} statsBlocker={statsBlocker} />
          ) : activeTab === 'saved' ? <ReplayLibraryPanel /> : <>
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
                        aria-label="Inspect"
                        className={`uiAction uiActionSecondary ${styles.historyReplayAction}`}
                        onClick={() => {
                          navigate(`/replay/${match.id}`);
                        }}
                        type="button"
                      >
                        <Icon className="uiIconDesktop" name="replay" />
                        <span className="uiActionLabel">Inspect</span>
                      </button>
                    </li>
                  );
                })}
              </ol>
            ) : (
              <div className={styles.historyEmpty}>
                <div className={styles.historyEmptyCopy}>
                  <p className={styles.historyEmptyTitle}>No matches yet</p>
                  <p className={styles.historyEmptyText}>
                    Finish a game to inspect it here.
                  </p>
                </div>
                <Link className="uiAction uiActionPrimary" to="/match/local">
                  <Icon className="uiIconDesktop" name="play" />
                  <span className="uiActionLabel">Play a game</span>
                </Link>
              </div>
            )}
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
          </>}
          </div>
        </section>
      </section>
    </main>
  );
}
