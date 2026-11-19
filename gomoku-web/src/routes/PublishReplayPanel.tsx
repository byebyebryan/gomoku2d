import { useEffect, useEffectEvent, useLayoutEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { Link } from "react-router-dom";
import { useStore } from "zustand";

import { cloudAuthStore } from "../cloud/auth_store";
import {
  loadCatalogPage,
  loadPublicProfile,
  saveCatalogPage,
  type PublicProfileSnapshot,
} from "../cloud/sharing_public";
import type { SavedMatchSide, SavedMatchV2 } from "../match/saved_match";
import { localProfileStore } from "../profile/local_profile_store";
import { portableReplayFromSavedMatch, validatePortableReplay, type PortableReplayV1 } from "../replay/portable_replay";
import { encodePortableReplay, portableReplayFingerprint } from "../replay/portable_replay_codec";
import { publishReplaySide } from "../replay/publish_replay_eligibility";
import { Icon } from "../ui/Icon";
import type { PublicationTarget } from "./my_games_navigation";
import { protectReplayDraftNavigation } from "./replay_draft_navigation";

import styles from "./ShareReplayPanel.module.css";

interface ReplayPreview {
  replay: PortableReplayV1 | null;
  payload: string;
  openingPly: number;
  error: string | null;
}

interface LoadedPublisher {
  uid: string;
  status: "loading" | "ready" | "error";
  profile: PublicProfileSnapshot | null;
  message: string | null;
}

function errorText(error: unknown, fallback: string): string {
  return error instanceof Error && error.message ? error.message : fallback;
}

function publisherIdentity(
  replay: PortableReplayV1,
  side: SavedMatchSide,
  uid: string,
): PortableReplayV1 {
  const participant = replay[side];
  if (participant.kind !== "human") {
    throw new Error("The selected player is not a human participant in this game.");
  }
  return {
    ...replay,
    [side]: { ...participant, profileUid: uid },
  };
}

export function PublishReplayPanel({
  match,
  source,
  eligible,
  moveIndex,
  open,
  onClose,
  ownerPractice,
  entryUid,
  entryMove,
  backToMyGames = false,
  playback,
  onMoveChange,
  onManageProfile,
}: {
  match: SavedMatchV2;
  source?: PortableReplayV1;
  eligible: boolean;
  moveIndex: number;
  open: boolean;
  onClose: () => void;
  ownerPractice?: boolean;
  entryUid?: string;
  entryMove?: number;
  backToMyGames?: boolean;
  playback?: ReactNode;
  onMoveChange?: (move: number) => void;
  onManageProfile?: () => void;
}) {
  const auth = useStore(cloudAuthStore);
  const localProfileId = useStore(localProfileStore, (state) => state.profile?.id ?? null);
  const activeUid = auth.status === "signed_in" ? auth.user?.uid ?? null : null;
  const uid = eligible && (!entryUid || entryUid === activeUid)
    && publishReplaySide(match, activeUid, localProfileId, ownerPractice) ? activeUid : null;
  const startMove = entryMove ?? moveIndex;
  const backRef = useRef<HTMLButtonElement>(null);
  const headingRef = useRef<HTMLHeadingElement>(null);
  const titleRef = useRef<HTMLInputElement>(null);
  const pickerRef = useRef<HTMLDivElement>(null);
  const chooseRef = useRef<HTMLButtonElement>(null);
  const keepRef = useRef<HTMLButtonElement>(null);
  const listingRef = useRef<HTMLAnchorElement>(null);
  const mayPop = useRef<(delta: number) => boolean>(() => true);
  const [publisher, setPublisher] = useState<LoadedPublisher | null>(null);
  const [opening, setOpening] = useState(startMove);
  const [customTitle, setCustomTitle] = useState<string | null>(null);
  const [picking, setPicking] = useState(false);
  const [discarding, setDiscarding] = useState(false);
  const discardExit = useRef(onClose);
  const [listing, setListing] = useState<PublicationTarget | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [previewReloadRequired, setPreviewReloadRequired] = useState(false);
  const latestUidRef = useRef<string | null>(uid);
  latestUidRef.current = uid;
  const requestRef = useRef(0);
  const title = customTitle ?? `Replay after move ${opening}`;
  const dirty = !listing && (opening !== startMove || title !== `Replay after move ${startMove}`);

  const preview = useMemo<ReplayPreview>(() => {
    if (!open || !uid) return { replay: null, payload: "", openingPly: opening, error: null };
    let original: PortableReplayV1 | null = source ?? null;
    try {
      original ??= portableReplayFromSavedMatch(match).replay;
      if (!original) throw new Error("This game could not be converted to a portable replay.");
      const side = publishReplaySide(match, uid, localProfileId, ownerPractice);
      if (!side) throw new Error("This game does not identify your human player for public attribution.");
      const replay = publisherIdentity({ ...original, viewPly: opening }, side, uid);
      const validated = validatePortableReplay(replay);
      return {
        replay: validated.replay,
        payload: encodePortableReplay(validated.replay),
        openingPly: validated.replay.viewPly ?? opening,
        error: null,
      };
    } catch (previewError) {
      return {
        replay: original,
        payload: "",
        openingPly: opening,
        error: errorText(previewError, "This game cannot be published."),
      };
    }
  }, [localProfileId, match, open, opening, ownerPractice, source, uid]);

  const publisherForCurrentUid = publisher?.uid === uid ? publisher : null;

  const loadPublisher = async (requestedUid = uid) => {
    if (preview.error || !requestedUid || cloudAuthStore.getState().status !== "signed_in"
      || cloudAuthStore.getState().user?.uid !== requestedUid) return;
    const request = ++requestRef.current;
    setPublisher({ uid: requestedUid, status: "loading", profile: null, message: null });
    setError(null);
    setNotice(null);
    try {
      const profile = await loadPublicProfile(requestedUid);
      if (latestUidRef.current !== requestedUid || cloudAuthStore.getState().status !== "signed_in"
        || cloudAuthStore.getState().user?.uid !== requestedUid
        || requestRef.current !== request) return;
      setPreviewReloadRequired(false);
      setPublisher({ uid: requestedUid, status: "ready", profile, message: null });
    } catch (loadError) {
      if (latestUidRef.current === requestedUid && cloudAuthStore.getState().status === "signed_in"
        && requestRef.current === request) {
        setPublisher({ uid: requestedUid, status: "error", profile: null, message: errorText(loadError, "The public profile could not be loaded.") });
      }
    }
  };

  const publish = async () => {
    const replayForPublish = preview.replay;
    const previewProfile = publisherForCurrentUid?.profile;
    if (!uid || !replayForPublish || !preview.payload || preview.error || !previewProfile?.enabled || pending) return;
    const cleanTitle = title.trim();
    if (!cleanTitle) {
      setError("Enter a title for this published replay.");
      return;
    }
    const request = ++requestRef.current;
    const currentUid = uid;
    const isCurrent = () => latestUidRef.current === currentUid
      && cloudAuthStore.getState().status === "signed_in"
      && cloudAuthStore.getState().user?.uid === currentUid
      && requestRef.current === request;
    setPending(true);
    setError(null);
    setNotice(null);

    try {
      const root = await loadPublicProfile(currentUid);
      if (!isCurrent()) return;
      if (!root?.enabled || root.generation !== previewProfile.generation || root.revision !== previewProfile.revision) {
        setPreviewReloadRequired(true);
        setPublisher({
          uid: currentUid,
          status: "error",
          profile: null,
          message: "Your public profile changed since this preview loaded. Reload the preview before publishing.",
        });
        return;
      }

      const pages = await Promise.all([0, 1, 2, 3].map((pageId) => loadCatalogPage(currentUid, pageId)));
      if (!isCurrent()) return;
      if (pages.some((page, pageId) => (
        !page || page.generation !== root.generation
          || (root.catalogPageCounts !== null && page.entries.length !== root.catalogPageCounts[pageId])
      ))) {
        throw new Error("The public catalog changed or is incomplete. Reload your public profile and try again.");
      }

      const validated = validatePortableReplay(replayForPublish);
      const payload = encodePortableReplay(validated.replay);
      const fingerprint = await portableReplayFingerprint(validated);
      if (!isCurrent()) return;
      const existing = pages.find((page) => page!.entries.some((entry) => entry.fingerprint === fingerprint));
      if (existing) {
        setListing({ uid: currentUid, fingerprint, pageId: existing.pageId });
        setNotice("This game is already published on your public profile.");
        setPublisher({ uid: currentUid, status: "ready", profile: root, message: null });
        return;
      }

      const availablePage = pages.find((page) => page!.entries.length < 8);
      if (!availablePage) {
        throw new Error("Your public catalog is full. Remove a published game before adding another; the limit is 32 games.");
      }
      const updated = await saveCatalogPage(currentUid, availablePage.pageId, [
        ...availablePage.entries,
        {
          fingerprint,
          payload,
          title: cleanTitle,
          view_ply: validated.replay.viewPly ?? 0,
        },
      ], root.revision, root.generation, availablePage.revision);
      if (!isCurrent()) return;
      setListing({ uid: currentUid, fingerprint, pageId: updated.page.pageId });
      setPublisher({ uid: currentUid, status: "ready", profile: updated.profile, message: null });
      setNotice(`Published “${updated.page.entries[updated.page.entries.length - 1]?.title ?? cleanTitle}” to ${updated.profile.displayName}'s public profile.`);
    } catch (publishError) {
      if (isCurrent()) setError(errorText(publishError, "This replay could not be published."));
    } finally {
      if (isCurrent()) setPending(false);
    }
  };

  useEffect(() => {
    requestRef.current += 1;
    setPublisher(null);
    setPending(false);
    setError(null);
    setNotice(null);
    setPreviewReloadRequired(false);
    setOpening(startMove);
    setCustomTitle(null);
    setPicking(false);
    setDiscarding(false);
    setListing(null);
    if (open && uid) {
      void loadPublisher(uid);
      headingRef.current?.focus({ preventScroll: true });
      headingRef.current?.scrollIntoView?.({ block: "nearest" });
    }
    // The entry move seeds the draft once. Selecting another frame stays local.
  }, [open, uid, match.id]);

  useLayoutEffect(() => () => { requestRef.current += 1; }, []);

  const finishPicker = (accept: boolean) => {
    if (accept) setOpening(moveIndex);
    onMoveChange?.(accept ? moveIndex : opening);
    setPicking(false);
    setError(null);
    requestAnimationFrame(() => chooseRef.current?.focus());
  };

  const keepEditing = () => {
    setDiscarding(false);
    requestAnimationFrame(() => (titleRef.current ?? backRef.current)?.focus());
  };

  const confirmDiscard = (exit: () => void) => {
    discardExit.current = exit;
    setDiscarding(true);
  };

  const back = () => {
    if (pending) return;
    if (picking) finishPicker(false);
    else if (discarding) keepEditing();
    else if (dirty) confirmDiscard(onClose);
    else onClose();
  };

  const handleEscape = useEffectEvent((event: KeyboardEvent) => {
    if (event.key !== "Escape" || event.defaultPrevented || event.isComposing) return;
    event.preventDefault();
    if (!event.repeat) back();
  });

  useEffect(() => {
    if (!open || !uid) return;
    // The read-only replay board is outside the sidebar and remains keyboard accessible.
    const onKeyDown = (event: KeyboardEvent) => handleEscape(event);
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, uid]);

  mayPop.current = (delta) => {
    if (pending) return false;
    if (picking) finishPicker(false);
    else if (dirty) confirmDiscard(onClose);
    else if (delta < 0 && !backToMyGames) onClose();
    else return true;
    return false;
  };

  useLayoutEffect(() => {
    if (picking) pickerRef.current?.querySelector<HTMLInputElement>("input[type=range]")?.focus();
    if (discarding) keepRef.current?.focus();
  }, [picking, discarding]);

  useLayoutEffect(() => {
    if (!listing) return;
    listingRef.current?.focus({ preventScroll: true });
    listingRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [listing]);

  useEffect(() => {
    if (!open || !uid) return;
    // Keep the original history index while a blocked pop is being restored.
    // Picker and dirty-state updates can render before the restoration event.
    return protectReplayDraftNavigation((delta) => mayPop.current(delta));
  }, [open, uid, match.id]);

  useEffect(() => {
    if (!open || !uid || (!dirty && !pending)) return;
    const beforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", beforeUnload);
    return () => window.removeEventListener("beforeunload", beforeUnload);
  }, [dirty, open, pending, uid]);

  if (!uid) return null;

  return (
    <section className={styles.section} aria-label="Publish replay" hidden={!open}>
      <button className={`uiAction uiActionNeutral ${styles.back}`} type="button" ref={backRef} onClick={back} disabled={pending}>
        <Icon name="back" /><span className="uiActionLabel">{picking ? "Back to publish" : backToMyGames ? "Back to My games" : "Back to replay"}</span>
      </button>
      <div className={styles.heading}><Icon name="publish" /><h2 ref={headingRef} tabIndex={-1}>{picking ? "Choose opening" : "Publish replay"}</h2></div>
      {open && (
        <div className={styles.body}>
          {picking ? <div className={styles.choice} ref={pickerRef}>
            {playback}
            <button className="uiAction uiActionSecondary" type="button" onClick={() => finishPicker(true)}>
              Use move {moveIndex}
            </button>
            <button className="uiAction uiActionNeutral" type="button" onClick={() => finishPicker(false)}>Cancel</button>
          </div> : discarding ? <div className={styles.choice} role="group" aria-label="Discard publish draft">
            <p className={styles.note}>Discard your publish draft?</p>
            <button className="uiAction uiActionSecondary" type="button" ref={keepRef} onClick={keepEditing}>Keep editing</button>
            <button className="uiAction uiActionDanger" type="button" onClick={() => discardExit.current()}>Discard draft</button>
          </div> : preview.error ? <p className={styles.error} role="alert">{preview.error}</p> : <>
            {(!publisherForCurrentUid || publisherForCurrentUid.status === "loading") && <p role="status">Loading your public profile.</p>}
            {publisherForCurrentUid?.status === "error" && (
              <p className={styles.error} role="alert">{publisherForCurrentUid.message}</p>
            )}
            {publisherForCurrentUid?.status === "error" && (
              <button className="uiAction uiActionNeutral" type="button" onClick={() => void loadPublisher(uid)}>
                {previewReloadRequired ? "Reload preview" : "Retry profile load"}
              </button>
            )}
            {error && <p className={styles.error} role="alert">{error}</p>}
            {notice && <p className={styles.message} role="status">{notice}</p>}
            {listing && <Link ref={listingRef} className="uiAction uiActionSecondary" to="/profile?tab=public" replace state={{ publicationTarget: listing }}>
              <Icon name="profile" />View published games
            </Link>}

            {publisherForCurrentUid?.status === "ready" && !publisherForCurrentUid.profile?.enabled && (
              <div className={styles.choice}>
                <p className={styles.note}>Enable your public profile to publish games.</p>
                {onManageProfile ? <button className="uiAction uiActionSecondary" type="button" onClick={() => {
                  if (dirty) confirmDiscard(onManageProfile); else onManageProfile();
                }}>
                  <Icon name="settings" />Manage public profile
                </button> : <Link className="uiAction uiActionSecondary" to="/profile?tab=public"><Icon name="settings" />Manage public profile</Link>}
              </div>
            )}

            {!listing && publisherForCurrentUid?.status === "ready" && publisherForCurrentUid.profile?.enabled && (
              <>
                <div className={styles.choice}>
                  <p className="uiSectionLabel">Public profile</p>
                  <p className={styles.publisherName}>{publisherForCurrentUid.profile.displayName}</p>
                </div>
                {preview.error || !preview.replay ? (
                  <p className={styles.error} role="alert">{preview.error ?? "This game cannot be published."}</p>
                ) : (
                  <>
                    <label className={styles.choice}>
                      <span className="uiSectionLabel">Public replay title</span>
                      <input className="uiInput" maxLength={160} value={title} ref={titleRef} disabled={pending}
                        onChange={(event) => { setCustomTitle(Array.from(event.target.value).slice(0, 80).join("")); setError(null); }} />
                    </label>
                    <div className={styles.preview} role="group" aria-label="Publication preview">
                      <div className={styles.previewHeading}>
                        <span className="uiSectionLabel">In your profile</span>
                        <span className={styles.moment}>Move {preview.openingPly} / {match.move_count}</span>
                      </div>
                      {[preview.replay.black, preview.replay.white].map((player, index) => <div className={styles.player} key={index}>
                        <span className={`${styles.stone} ${index === 0 ? styles.stoneBlack : styles.stoneWhite}`} aria-hidden="true" />
                        <span className="uiSrOnly">{index === 0 ? "Black" : "White"}:</span>
                        <span className={styles.playerName} title={player.name}>{player.name}</span>
                        {player.kind === "human" && player.profileUid && <span className={styles.profileIcon} role="img" aria-label="Public profile link included"><Icon name="profile" /></span>}
                      </div>)}
                      <p className={styles.access}>Full game · Public catalog</p>
                      {preview.replay.practice && <p className={styles.origin}>Starts from move {preview.replay.practice.inheritedPly}</p>}
                    </div>
                    {playback && <button className="uiAction uiActionNeutral" type="button" ref={chooseRef} disabled={pending}
                      onClick={() => { onMoveChange?.(opening); setPicking(true); }}>
                      <Icon name="replay" />Choose opening
                    </button>}
                    <button className="uiAction uiActionPrimary" type="button" disabled={pending || !title.trim()}
                      onClick={() => void publish()}>
                      <Icon name="publish" />{pending ? "Publishing replay" : "Publish this replay"}
                    </button>
                    <p className={styles.note}>Anyone can copy this replay. Unpublishing removes the listing. Shared copies remain.</p>
                    <p className={styles.access}>Casual record · Unverified result</p>
                  </>
                )}
              </>
            )}
          </>}
        </div>
      )}
    </section>
  );
}
