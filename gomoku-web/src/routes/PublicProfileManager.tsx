import { useEffect, useRef, useState } from "react";
import { useStore } from "zustand";

import { cloudAuthStore } from "../cloud/auth_store";
import {
  loadCatalogPage,
  loadPublicProfile,
  saveCatalogPage,
  setPublicProfile,
  type PublicCatalogPageSnapshot,
  type PublicProfileSnapshot,
} from "../cloud/sharing_public";
import { localProfileStore } from "../profile/local_profile_store";

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

export function PublicProfileManager() {
  const auth = useStore(cloudAuthStore);
  const localProfile = useStore(localProfileStore, (state) => state.profile);
  const uid = auth.status === "signed_in" ? auth.user?.uid ?? null : null;
  const [open, setOpen] = useState(false);
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
  }, [uid]);

  const visibleSnapshot = snapshot?.uid === uid ? snapshot : null;
  const profile = visibleSnapshot?.profile ?? null;
  const pages = visibleSnapshot?.pages ?? null;
  const catalogReady = Boolean(
    profile
    && pages?.length === 4
    && pages.every((page) => page !== null && page.generation === profile.generation),
  );
  const markerStateValid = Boolean(
    visibleSnapshot
    && (profile
      ? pages?.length === 4 && pages.every((page) => page !== null && page.generation === profile.generation)
      : pages?.length === 4 && pages.every((page) => page === null)),
  );

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
      if (loadedProfile) {
        if (catalogPages.length !== 4 || catalogPages.some((page) => !page || page.generation !== loadedProfile.generation)) {
          throw new Error("The public catalog markers do not match this profile. Reload before changing anything.");
        }
      } else if (catalogPages.some((page) => page !== null)) {
        throw new Error("Public profile markers are incomplete. Reload before enabling a profile.");
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

  const saveName = async () => {
    if (!uid || !profile?.enabled || !markerStateValid) return;
    const request = ++requestRef.current;
    setPendingAction("name");
    setError(null);
    setNotice(null);
    try {
      const updated = await setPublicProfile(uid, draftName, true, profile.revision);
      if (!isCurrent(uid, request)) return;
      setSnapshot((current) => current?.uid === uid
        ? { ...current, profile: updated }
        : current);
      setDraftName(updated.displayName);
      setNotice("Public name saved.");
    } catch (saveError) {
      if (isCurrent(uid, request)) setError(errorText(saveError, "The public name could not be saved."));
    } finally {
      if (isCurrent(uid, request)) setPendingAction(null);
    }
  };

  const enableProfile = async () => {
    if (!uid || !markerStateValid || !confirmEnable || profile?.enabled) return;
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
      setNotice("Public profile enabled with an empty catalog.");
    } catch (enableError) {
      if (isCurrent(uid, request)) setError(errorText(enableError, "The public profile could not be enabled."));
    } finally {
      if (isCurrent(uid, request)) setPendingAction(null);
    }
  };

  const disableProfile = async () => {
    if (!uid || !profile?.enabled || !confirmDisable || !markerStateValid) return;
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
      setNotice("Public profile disabled and its catalog removed.");
    } catch (disableError) {
      if (isCurrent(uid, request)) setError(errorText(disableError, "The public profile could not be disabled."));
    } finally {
      if (isCurrent(uid, request)) setPendingAction(null);
    }
  };

  const unpublish = async (page: PublicCatalogPageSnapshot, fingerprint: string) => {
    if (!uid || !profile?.enabled || !markerStateValid || !pages) return;
    const request = ++requestRef.current;
    setPendingAction(fingerprint);
    setError(null);
    setNotice(null);
    try {
      const entries = page.entries.filter((entry) => entry.fingerprint !== fingerprint);
      const updated = await saveCatalogPage(uid, page.pageId, entries, profile.generation, page.revision);
      if (!isCurrent(uid, request)) return;
      setSnapshot((current) => current?.uid === uid && current.pages
        ? {
          ...current,
          pages: current.pages.map((candidate) => candidate?.pageId === updated.pageId ? updated : candidate),
        }
        : current);
      setNotice("Publication removed from the public catalog.");
    } catch (unpublishError) {
      if (isCurrent(uid, request)) setError(errorText(unpublishError, "The publication could not be removed."));
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

  return (
    <section className={`${styles.manager} uiPanel`} aria-labelledby="public-profile-manager-title">
      <div className={styles.managerHeader}>
        <div>
          <p className="uiPageEyebrow">Optional sharing</p>
          <h2 className={styles.sectionTitle} id="public-profile-manager-title">Public profile</h2>
        </div>
        <button
          className="uiAction uiActionSecondary"
          type="button"
          aria-expanded={open}
          aria-controls="public-profile-manager-body"
          onClick={() => {
            const nextOpen = !open;
            setOpen(nextOpen);
            setError(null);
            setNotice(null);
            if (nextOpen && uid && pendingAction === null) {
              setDraftName(localProfile?.displayName ?? "Player");
              void reload(uid);
            }
          }}
        >
          {open ? "Public profile" : "Manage public profile"}
        </button>
      </div>

      {open && (
        <div className={styles.managerBody} id="public-profile-manager-body">
          {!uid ? (
            <p className={styles.mutedText}>
              Sign in from this Profile page to manage a public profile. Your public profile stays off until you enable it.
            </p>
          ) : (
            <>
              <div className={styles.managerActions}>
                <button className="uiAction uiActionNeutral" type="button" disabled={loading || pendingAction !== null}
                  onClick={() => void reload(uid)}>
                  {loading ? "Loading profile" : "Reload public profile"}
                </button>
                {profile?.enabled && (
                  <button className="uiAction uiActionNeutral" type="button" onClick={() => void copyLink()}>
                    Copy public profile link
                  </button>
                )}
              </div>

              {loading && <p className={styles.mutedText} role="status">Loading your public profile.</p>}
              {error && <p className={styles.errorText} role="alert">{error}</p>}
              {notice && <p className={styles.successText} role="status">{notice}</p>}
              {linkMessage && <p className={styles.mutedText} role="status">{linkMessage}</p>}

              {profile?.enabled && (
                <label className={styles.field}>
                  <span>Public profile link</span>
                  <input ref={linkRef} className="uiInput" readOnly value={publicProfileHref(uid)} />
                </label>
              )}

              {visibleSnapshot && !loading && markerStateValid && (
                <>
                  {profile?.enabled ? (
                    <>
                      <label className={styles.field}>
                        <span>Public name</span>
                        <input className="uiInput" maxLength={80} value={draftName}
                          onChange={(event) => setDraftName(event.target.value)} />
                      </label>
                      <button className="uiAction uiActionPrimary" type="button"
                        disabled={pendingAction !== null || draftName.trim() === profile.displayName}
                        onClick={() => void saveName()}>
                        {pendingAction === "name" ? "Saving name" : "Save public name"}
                      </button>
                    </>
                  ) : (
                    <>
                      <label className={styles.field}>
                        <span>Choose your public name</span>
                        <input className="uiInput" maxLength={80} value={draftName}
                          onChange={(event) => { setDraftName(event.target.value); setConfirmEnable(false); }} />
                      </label>
                      <p className={styles.preview}>
                        Anyone with your profile link will see <strong>{draftName.trim() || "your chosen name"}</strong> and games you explicitly publish.
                      </p>
                      <label className={styles.confirmRow}>
                        <input type="checkbox" checked={confirmEnable} onChange={(event) => setConfirmEnable(event.target.checked)} />
                        <span>I want to enable this public profile with the name shown above.</span>
                      </label>
                      <button className="uiAction uiActionPrimary" type="button"
                        disabled={!confirmEnable || pendingAction !== null}
                        onClick={() => void enableProfile()}>
                        {pendingAction === "enable" ? "Enabling public profile" : "Enable public profile"}
                      </button>
                    </>
                  )}

                  {profile?.enabled && catalogReady && (
                    <div className={styles.publicationSection}>
                      <h3 className={styles.subTitle}>Published games</h3>
                      <p className={styles.mutedText}>Unpublishing removes a catalog entry. Portable copies already shared cannot be recalled.</p>
                      {pages?.flatMap((page) => page?.entries.map((entry) => ({ page, entry })) ?? []).length ? (
                        <ul className={styles.publicationList}>
                          {pages?.flatMap((page) => page?.entries.map((entry) => ({ page, entry })) ?? []).map(({ page, entry }) => (
                            <li className={styles.publicationRow} key={entry.fingerprint}>
                              <span>{entry.title}</span>
                              <button className="uiAction uiActionDanger" type="button"
                                disabled={pendingAction !== null}
                                onClick={() => void unpublish(page, entry.fingerprint)}>
                                {pendingAction === entry.fingerprint ? "Removing" : "Unpublish"}
                              </button>
                            </li>
                          ))}
                        </ul>
                      ) : <p className={styles.emptyState}>No games are currently published.</p>}
                    </div>
                  )}

                  {profile?.enabled && !catalogReady && (
                    <p className={styles.mutedText}>Reload your public profile before making changes.</p>
                  )}

                  {profile?.enabled && (
                    <div className={styles.dangerSection}>
                      <h3 className={styles.subTitle}>Disable public profile</h3>
                      {!confirmDisable ? (
                        <button className="uiAction uiActionDanger" type="button" disabled={pendingAction !== null}
                          onClick={() => setConfirmDisable(true)}>
                          Disable public profile
                        </button>
                      ) : (
                        <div className={styles.confirmPanel}>
                          <p>
                            Disabling removes every published game and makes this page unavailable. Re-enabling starts with an empty catalog. Portable replay copies already made by others cannot be recalled. Empty cloud control markers remain to reject stale writes.
                          </p>
                          <div className={styles.managerActions}>
                            <button className="uiAction uiActionDanger" type="button" disabled={pendingAction !== null}
                              onClick={() => void disableProfile()}>
                              {pendingAction === "disable" ? "Disabling profile" : "Confirm disable"}
                            </button>
                            <button className="uiAction uiActionNeutral" type="button" disabled={pendingAction !== null}
                              onClick={() => setConfirmDisable(false)}>
                              Keep profile enabled
                            </button>
                          </div>
                        </div>
                      )}
                    </div>
                  )}
                </>
              )}

              {visibleSnapshot && !loading && !markerStateValid && (
                <p className={styles.mutedText}>Reload to inspect the current public profile before making changes.</p>
              )}
            </>
          )}
        </div>
      )}
    </section>
  );
}
