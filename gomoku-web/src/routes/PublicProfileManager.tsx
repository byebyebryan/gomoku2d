import { useEffect, useRef, useState } from "react";
import { useStore } from "zustand";

import { cloudAuthStore } from "../cloud/auth_store";
import {
  loadCatalogPage,
  loadPublicProfile,
  saveCatalogPage,
  setPublicProfile,
  setPublicProfileStats,
  type PublicCatalogPageSnapshot,
  type PublicProfileSnapshot,
  type PublicProfileStatsCounters,
} from "../cloud/sharing_public";
import { playerWinRate, type PlayerStats } from "../profile/player_stats";
import { localProfileStore } from "../profile/local_profile_store";
import { publicProfileDateLabel } from "./public_catalog";
import { Icon } from "../ui/Icon";

import styles from "./SharingPanels.module.css";

interface OwnerSnapshot {
  uid: string;
  profile: PublicProfileSnapshot | null;
  pages: (PublicCatalogPageSnapshot | null)[] | null;
}

const PUBLIC_PAGE_IDS = [0, 1, 2, 3] as const;

function emptyCatalogPagesAfterVisibilityChange(
  uid: string,
  generation: number,
  updatedAt: string,
  priorPages: (PublicCatalogPageSnapshot | null)[] | null,
): PublicCatalogPageSnapshot[] {
  return PUBLIC_PAGE_IDS.map((pageId) => {
    const prior = priorPages?.[pageId] ?? null;
    return {
      uid,
      pageId,
      generation,
      revision: prior ? prior.revision + 1 : 1,
      entries: [],
      updatedAt,
    };
  });
}

function errorText(error: unknown, fallback: string): string {
  return error instanceof Error && error.message ? error.message : fallback;
}

function publicProfileHref(uid: string): string {
  const url = new URL(`${import.meta.env.BASE_URL}player/`, window.location.origin);
  url.searchParams.set("id", uid);
  return url.href;
}

function snapshotMarkersValid(
  uid: string,
  profile: PublicProfileSnapshot | null,
  pages: (PublicCatalogPageSnapshot | null)[] | null,
): boolean {
  if (!pages || pages.length !== 4) return false;
  if (!profile) return pages.every((page) => page === null);
  return pages.every((page, pageId) => (
    page !== null
    && page.uid === uid
    && page.pageId === pageId
    && page.generation === profile.generation
    && (profile.catalogPageCounts === null || page.entries.length === profile.catalogPageCounts[pageId])
  ));
}

function catalogReady(profile: PublicProfileSnapshot | null, pages: (PublicCatalogPageSnapshot | null)[] | null): boolean {
  return Boolean(profile?.enabled && pages?.length === 4 && pages.every((page) => page !== null));
}

export function PublicProfileManager({
  stats,
  statsReady,
  statsBlocker,
}: {
  stats: PlayerStats;
  statsReady: boolean;
  statsBlocker: string;
}) {
  const auth = useStore(cloudAuthStore);
  const localProfile = useStore(localProfileStore, (state) => state.profile);
  const uid = auth.status === "signed_in" ? auth.user?.uid ?? null : null;
  const [snapshot, setSnapshot] = useState<OwnerSnapshot | null>(null);
  const [loading, setLoading] = useState(false);
  const [pendingAction, setPendingAction] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [draftName, setDraftName] = useState("");
  const [confirmEnable, setConfirmEnable] = useState(false);
  const [confirmDisable, setConfirmDisable] = useState(false);
  const [linkMessage, setLinkMessage] = useState("");
  const linkRef = useRef<HTMLInputElement>(null);
  const latestUidRef = useRef<string | null>(uid);
  latestUidRef.current = uid;
  const requestRef = useRef(0);

  const isCurrent = (requestedUid: string, request: number) => (
    latestUidRef.current === requestedUid
    && cloudAuthStore.getState().status === "signed_in"
    && cloudAuthStore.getState().user?.uid === requestedUid
    && requestRef.current === request
  );

  const reload = async (requestedUid = uid) => {
    if (!requestedUid || cloudAuthStore.getState().status !== "signed_in"
      || cloudAuthStore.getState().user?.uid !== requestedUid) return;
    const request = ++requestRef.current;
    setLoading(true);
    setError(null);
    setNotice(null);
    setLinkMessage("");
    try {
      const [loadedProfile, page0, page1, page2, page3] = await Promise.all([
        loadPublicProfile(requestedUid),
        loadCatalogPage(requestedUid, 0),
        loadCatalogPage(requestedUid, 1),
        loadCatalogPage(requestedUid, 2),
        loadCatalogPage(requestedUid, 3),
      ]);
      if (!isCurrent(requestedUid, request)) return;
      const catalogPages = [page0, page1, page2, page3];
      if (!snapshotMarkersValid(requestedUid, loadedProfile, catalogPages)) {
        throw new Error("Public profile and catalog markers do not match. Reload before changing anything.");
      }

      setSnapshot({ uid: requestedUid, profile: loadedProfile, pages: catalogPages });
      setDraftName(loadedProfile?.enabled ? loadedProfile.displayName : localProfile?.displayName ?? "Player");
      setConfirmEnable(false);
      setConfirmDisable(false);
      setNotice(null);
    } catch (loadError) {
      if (isCurrent(requestedUid, request)) setError(errorText(loadError, "The public profile could not be loaded."));
    } finally {
      if (isCurrent(requestedUid, request)) setLoading(false);
    }
  };

  useEffect(() => {
    requestRef.current += 1;
    setSnapshot(null);
    setLoading(false);
    setPendingAction(null);
    setError(null);
    setNotice(null);
    setDraftName(localProfile?.displayName ?? "Player");
    setConfirmEnable(false);
    setConfirmDisable(false);
    setLinkMessage("");
    if (uid) void reload(uid);
  }, [uid]);

  useEffect(() => () => { requestRef.current += 1; }, []);

  const visibleSnapshot = snapshot?.uid === uid ? snapshot : null;
  const profile = visibleSnapshot?.profile ?? null;
  const pages = visibleSnapshot?.pages ?? null;
  const markersValid = snapshotMarkersValid(uid ?? "", profile, pages);
  const readyCatalog = catalogReady(profile, pages);
  const publishedEntries = pages?.flatMap((page) => page?.entries.map((entry) => ({ page, entry })) ?? []) ?? [];
  const isProfileLoaded = visibleSnapshot !== null && !loading && markersValid;

  const saveName = async () => {
    if (!uid || !profile?.enabled || !markersValid) return;
    const request = ++requestRef.current;
    setPendingAction("name");
    setError(null);
    setNotice(null);
    try {
      const updated = await setPublicProfile(uid, draftName, true, profile.revision);
      if (!isCurrent(uid, request)) return;
      setSnapshot((current) => current?.uid === uid ? { ...current, profile: updated } : current);
      setDraftName(updated.displayName);
      setNotice("Public name saved.");
    } catch (saveError) {
      if (isCurrent(uid, request)) setError(errorText(saveError, "The public name could not be saved."));
    } finally {
      if (isCurrent(uid, request)) setPendingAction(null);
    }
  };

  const enableProfile = async () => {
    if (!uid || !markersValid || !confirmEnable || profile?.enabled) return;
    const request = ++requestRef.current;
    setPendingAction("enable");
    setError(null);
    setNotice(null);
    try {
      const updated = await setPublicProfile(uid, draftName, true, profile?.revision ?? null);
      if (!isCurrent(uid, request)) return;
      setSnapshot({
        uid,
        profile: updated,
        pages: emptyCatalogPagesAfterVisibilityChange(uid, updated.generation, updated.updatedAt, pages),
      });
      setDraftName(updated.displayName);
      setConfirmEnable(false);
      setNotice("Public profile enabled with an empty catalog. Public statistics stay off until you opt in.");
    } catch (enableError) {
      if (isCurrent(uid, request)) setError(errorText(enableError, "The public profile could not be enabled."));
    } finally {
      if (isCurrent(uid, request)) setPendingAction(null);
    }
  };

  const disableProfile = async () => {
    if (!uid || !profile?.enabled || !confirmDisable || !markersValid) return;
    const request = ++requestRef.current;
    setPendingAction("disable");
    setError(null);
    setNotice(null);
    try {
      const updated = await setPublicProfile(uid, "", false, profile.revision);
      if (!isCurrent(uid, request)) return;
      setSnapshot({
        uid,
        profile: updated,
        pages: emptyCatalogPagesAfterVisibilityChange(uid, updated.generation, updated.updatedAt, pages),
      });
      setConfirmDisable(false);
      setNotice("Public profile disabled. Statistics and catalog publications were removed.");
    } catch (disableError) {
      if (isCurrent(uid, request)) setError(errorText(disableError, "The public profile could not be disabled."));
    } finally {
      if (isCurrent(uid, request)) setPendingAction(null);
    }
  };

  const setStatsSharing = async (share: boolean) => {
    if (!uid || !profile?.enabled || !markersValid || pendingAction !== null) return;
    if (share && !statsReady) {
      setError(statsBlocker);
      return;
    }
    const request = ++requestRef.current;
    setPendingAction(share ? "share-stats" : "remove-stats");
    setError(null);
    setNotice(null);
    try {
      const counters: PublicProfileStatsCounters | null = share ? stats : null;
      const updated = await setPublicProfileStats(uid, counters, profile.revision);
      if (!isCurrent(uid, request)) return;
      setSnapshot((current) => current?.uid === uid ? { ...current, profile: updated } : current);
      setNotice(share ? "Your current My games summary is now public." : "Public statistics removed.");
    } catch (statsError) {
      if (isCurrent(uid, request)) setError(errorText(statsError, "Public statistics could not be updated. Reload before retrying."));
    } finally {
      if (isCurrent(uid, request)) setPendingAction(null);
    }
  };

  const copyLink = async () => {
    if (!uid || !profile?.enabled) return;
    const url = publicProfileHref(uid);
    try {
      await navigator.clipboard.writeText(url);
      setLinkMessage("Public profile link copied.");
    } catch {
      linkRef.current?.focus();
      linkRef.current?.select();
      setLinkMessage("Copy the selected public profile link.");
    }
  };

  const unpublish = async (page: PublicCatalogPageSnapshot, fingerprint: string) => {
    if (!uid || !profile?.enabled || !markersValid) return;
    const request = ++requestRef.current;
    setPendingAction(fingerprint);
    setError(null);
    setNotice(null);
    try {
      const updated = await saveCatalogPage(
        uid,
        page.pageId,
        page.entries.filter((entry) => entry.fingerprint !== fingerprint),
        profile.revision,
        profile.generation,
        page.revision,
      );
      if (!isCurrent(uid, request)) return;
      setSnapshot((current) => current?.uid === uid && current.pages
        ? {
          ...current,
          profile: updated.profile,
          pages: current.pages.map((candidate) => candidate?.pageId === updated.page.pageId ? updated.page : candidate),
        }
        : current);
      setNotice("Publication removed from the public catalog.");
    } catch (unpublishError) {
      if (isCurrent(uid, request)) setError(errorText(unpublishError, "The publication could not be removed."));
    } finally {
      if (isCurrent(uid, request)) setPendingAction(null);
    }
  };

  const sharedStats = profile?.sharedStats ?? null;
  const winRate = sharedStats ? playerWinRate(sharedStats) : null;

  return (
    <section className={`${styles.manager} uiPanel`} aria-labelledby="public-profile-manager-title">
      <div className={styles.managerHeader}>
        <div>
          <p className="uiPageEyebrow">Optional sharing · {profile?.enabled ? "Enabled" : "Off"}</p>
          <h2 className={styles.sectionTitle} id="public-profile-manager-title">Public profile</h2>
        </div>
        {uid && <button className="uiAction uiActionNeutral" type="button" aria-label={loading ? "Loading profile" : "Reload public profile"}
          disabled={loading || pendingAction !== null} onClick={() => void reload(uid)}>
          <Icon name="reset" /><span className="uiActionLabel">{loading ? "Loading" : "Reload"}</span>
        </button>}
      </div>

      {!uid ? (
        <p className={styles.mutedText}>
          Sign in to manage a public profile. Your public profile and statistics stay off until you choose to share them.
        </p>
      ) : (
        <div className={styles.managerBody}>
          {loading && <p className={styles.mutedText} role="status">Loading your public profile.</p>}
          {error && <p className={styles.errorText} role="alert">{error}</p>}
          {notice && <p className={styles.successText} role="status">{notice}</p>}
          {linkMessage && <p className={styles.mutedText} role="status">{linkMessage}</p>}

          {isProfileLoaded ? (
            <>
              <section className={styles.settingsSection} aria-labelledby="public-identity-title">
                <h3 className="uiSectionLabel" id="public-identity-title">Identity</h3>
                <div className={styles.settingRow}>
                  <div className={styles.settingCopy}>
                    <label className={styles.settingLabel} htmlFor="public-name">{profile?.enabled ? "Public name" : "Choose your public name"}</label>
                    <p className={styles.settingHint}>Shown to anyone with your profile link.</p>
                  </div>
                  <div className={styles.settingControls}>
                    <input id="public-name" className="uiInput" maxLength={160} value={draftName}
                      onChange={(event) => { setDraftName(Array.from(event.target.value).slice(0, 80).join("")); setConfirmEnable(false); }} />
                    {profile?.enabled && <button className="uiAction uiActionPrimary" type="button"
                      disabled={pendingAction !== null || draftName.trim() === profile.displayName}
                      onClick={() => void saveName()}>
                      <Icon name="save" />{pendingAction === "name" ? "Saving name" : "Save public name"}
                    </button>}
                  </div>
                </div>
                {profile?.enabled ? <div className={styles.settingRow}>
                  <div className={styles.settingCopy}>
                    <label className={styles.settingLabel} htmlFor="public-profile-link">Profile link</label>
                    <p className={styles.settingHint}>Your public page and published games.</p>
                  </div>
                  <div className={styles.settingControls}>
                    <input id="public-profile-link" aria-label="Public profile link" ref={linkRef} className={`uiInput ${styles.profileLink}`} readOnly value={publicProfileHref(uid)} />
                    <div className={styles.managerActions}>
                      <button className="uiAction uiActionNeutral" type="button" onClick={() => void copyLink()}>
                        <Icon name="copy" />Copy link
                      </button>
                      <a className="uiAction uiActionSecondary" href={publicProfileHref(uid)}>
                        <Icon name="profile" />View public profile
                      </a>
                    </div>
                  </div>
                </div> : <div className={styles.settingRow}>
                  <div className={styles.settingCopy}>
                    <p className={styles.settingLabel}>Visibility</p>
                    <p className={styles.settingHint}>Starts empty. Publish games individually; statistics stay off until you opt in.</p>
                  </div>
                  <div className={styles.settingControls}>
                    <label className={styles.confirmRow}>
                      <input type="checkbox" checked={confirmEnable} onChange={(event) => setConfirmEnable(event.target.checked)} />
                      <span>I want to enable this public profile with the name shown above.</span>
                    </label>
                    <button className="uiAction uiActionPrimary" type="button"
                      disabled={!confirmEnable || pendingAction !== null} onClick={() => void enableProfile()}>
                      <Icon name="profile" />{pendingAction === "enable" ? "Enabling public profile" : "Enable public profile"}
                    </button>
                  </div>
                </div>}
              </section>

              {profile?.enabled && <>
                <section className={styles.settingsSection} aria-labelledby="public-stats-title">
                  <h3 className="uiSectionLabel" id="public-stats-title">Public statistics</h3>
                  <div className={styles.settingRow}>
                    <div className={styles.settingCopy}>
                      <p className={styles.settingLabel}>My games summary</p>
                      <p className={styles.settingHint}>An optional snapshot. Update it when you choose.</p>
                    </div>
                    <div className={styles.settingControls}>
                      <div className={styles.statsVisibility} role="group" aria-label="Share My games statistics publicly">
                        <button className={sharedStats === null ? "uiSegment uiSegmentActive" : "uiSegment"}
                          type="button" aria-pressed={sharedStats === null} disabled={pendingAction !== null}
                          onClick={() => { if (sharedStats !== null) void setStatsSharing(false); }}>Off</button>
                        <button className={sharedStats !== null ? "uiSegment uiSegmentActive" : "uiSegment"}
                          type="button" aria-pressed={sharedStats !== null} disabled={pendingAction !== null || !statsReady}
                          onClick={() => { if (sharedStats === null) void setStatsSharing(true); }}>On</button>
                      </div>
                      {!statsReady && <p className={styles.settingHint}>{statsBlocker}</p>}
                      {sharedStats && <>
                        <div className={`${styles.publicStatsGrid} ${styles.managerStatsGrid}`} role="group" aria-label="Published statistics snapshot">
                          {([
                            ["Matches", sharedStats.matches], ["Wins", sharedStats.wins],
                            ["Losses", sharedStats.losses], ["Draws", sharedStats.draws],
                            ["Win rate", winRate === null ? "—" : `${(winRate * 100).toFixed(1)}%`],
                          ] as const).map(([label, value]) => <div className={styles.publicStatsTile} key={label}>
                            <span className={styles.publicStatsValue}>{value}</span>
                            <span className={styles.publicStatsLabel}>{label}</span>
                          </div>)}
                        </div>
                        <p className={styles.publicStatsDate}>Published {publicProfileDateLabel(sharedStats.updatedAt)}</p>
                        <button className="uiAction uiActionSecondary" type="button" disabled={pendingAction !== null || !statsReady}
                          onClick={() => void setStatsSharing(true)}>
                          <Icon name="reset" />{pendingAction === "share-stats" ? "Updating public stats" : "Update public stats"}
                        </button>
                      </>}
                    </div>
                  </div>
                </section>

                <section className={styles.settingsSection} aria-labelledby="public-games-title">
                  <h3 className="uiSectionLabel" id="public-games-title">Published games</h3>
                  <div className={styles.settingRow}>
                    <div className={styles.settingCopy}>
                      <p className={styles.settingLabel}>{publishedEntries.length} / 32 games</p>
                      <p className={styles.settingHint}>Publish from a replay. Unpublishing removes the listing; shared copies remain.</p>
                    </div>
                    <div className={styles.settingControls}>
                      {readyCatalog ? publishedEntries.length ? <ul className={styles.publicationList}>
                        {publishedEntries.map(({ page, entry }) => page ? <li className={styles.publicationRow} key={entry.fingerprint}>
                          <span>{entry.title}</span>
                          <button className="uiAction uiActionDanger" type="button" disabled={pendingAction !== null}
                            onClick={() => void unpublish(page, entry.fingerprint)}>
                            <Icon name="minus" />{pendingAction === entry.fingerprint ? "Removing" : "Unpublish"}
                          </button>
                        </li> : null)}
                      </ul> : <p className={styles.emptyState}>No games are currently published.</p>
                        : <p className={styles.settingHint}>Reload your public profile before managing publications.</p>}
                    </div>
                  </div>
                </section>

                <section className={styles.settingsSection} aria-labelledby="public-visibility-title">
                  <h3 className="uiSectionLabel" id="public-visibility-title">Visibility</h3>
                  <div className={styles.settingRow}>
                    <div className={styles.settingCopy}>
                      <p className={styles.settingLabel}>Disable public profile</p>
                      <p className={styles.settingHint}>Removes the public page, statistics, and published games.</p>
                    </div>
                    <div className={styles.settingControls}>
                      {!confirmDisable ? <button className="uiAction uiActionDanger" type="button" disabled={pendingAction !== null}
                        onClick={() => setConfirmDisable(true)}>Disable public profile</button> : <div className={styles.confirmPanel}>
                        <p>Disabling removes public statistics and every published game. Re-enabling starts empty with statistics off. Shared replay copies remain.</p>
                        <div className={styles.managerActions}>
                          <button className="uiAction uiActionDanger" type="button" disabled={pendingAction !== null}
                            onClick={() => void disableProfile()}>{pendingAction === "disable" ? "Disabling profile" : "Confirm disable"}</button>
                          <button className="uiAction uiActionNeutral" type="button" disabled={pendingAction !== null}
                            onClick={() => setConfirmDisable(false)}>Keep profile enabled</button>
                        </div>
                      </div>}
                    </div>
                  </div>
                </section>
              </>}
            </>
          ) : !loading && !error ? <p className={styles.mutedText}>Public profile data is not ready yet. Reload to try again.</p> : null}
        </div>
      )}
    </section>
  );
}
