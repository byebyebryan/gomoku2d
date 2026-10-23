import { useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { useStore } from "zustand";

import { cloudAuthStore } from "../cloud/auth_store";
import {
  loadCatalogPage,
  loadPublicProfile,
  saveCatalogPage,
  type PublicProfileSnapshot,
} from "../cloud/sharing_public";
import { matchUserSide, type SavedMatchSide, type SavedMatchV2 } from "../match/saved_match";
import { localProfileStore } from "../profile/local_profile_store";
import { portableReplayFromSavedMatch, validatePortableReplay, type PortableReplayV1 } from "../replay/portable_replay";
import { encodePortableReplay, portableReplayFingerprint } from "../replay/portable_replay_codec";
import { Icon } from "../ui/Icon";

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

function humanSideForReplay(
  match: SavedMatchV2,
  replay: PortableReplayV1,
  uid: string,
  localProfileId: string | null,
  isPractice: boolean,
): SavedMatchSide | null {
  const matched = matchUserSide(match, { profileUid: uid, localProfileId });
  if (matched) return matched;
  if (!isPractice) return null;
  const humanSides: SavedMatchSide[] = [];
  if (replay.black.kind === "human") humanSides.push("black");
  if (replay.white.kind === "human") humanSides.push("white");
  return humanSides.length === 1 ? humanSides[0]! : null;
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
}: {
  match: SavedMatchV2;
  source?: PortableReplayV1;
  eligible: boolean;
  moveIndex: number;
  open: boolean;
  onClose: () => void;
}) {
  const auth = useStore(cloudAuthStore);
  const localProfileId = useStore(localProfileStore, (state) => state.profile?.id ?? null);
  const uid = auth.status === "signed_in" ? auth.user?.uid ?? null : null;
  const backRef = useRef<HTMLButtonElement>(null);
  const [publisher, setPublisher] = useState<LoadedPublisher | null>(null);
  const [title, setTitle] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [previewReloadRequired, setPreviewReloadRequired] = useState(false);
  const latestUidRef = useRef<string | null>(uid);
  latestUidRef.current = uid;
  const requestRef = useRef(0);

  useEffect(() => {
    requestRef.current += 1;
    setPublisher(null);
    setPending(false);
    setError(null);
    setNotice(null);
    setPreviewReloadRequired(false);
    setTitle("");
  }, [uid]);

  const preview = useMemo<ReplayPreview>(() => {
    if (!eligible || !uid) return { replay: null, payload: "", openingPly: moveIndex, error: null };
    let original: PortableReplayV1 | null = source ?? null;
    try {
      original ??= portableReplayFromSavedMatch(match).replay;
      if (!original) throw new Error("This game could not be converted to a portable replay.");
      const side = humanSideForReplay(match, original, uid, localProfileId, Boolean(source?.practice));
      if (!side) throw new Error("This game does not identify your human player for public attribution.");
      const replay = publisherIdentity({ ...original, viewPly: moveIndex }, side, uid);
      const validated = validatePortableReplay(replay);
      return {
        replay: validated.replay,
        payload: encodePortableReplay(validated.replay),
        openingPly: validated.replay.viewPly ?? moveIndex,
        error: null,
      };
    } catch (previewError) {
      return {
        replay: original,
        payload: "",
        openingPly: moveIndex,
        error: errorText(previewError, "This game cannot be published."),
      };
    }
  }, [eligible, localProfileId, match, moveIndex, source, uid]);

  const publisherForCurrentUid = publisher?.uid === uid ? publisher : null;

  const loadPublisher = async (requestedUid = uid) => {
    if (!requestedUid || cloudAuthStore.getState().status !== "signed_in"
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
      setPublisher({ uid: currentUid, status: "ready", profile: updated.profile, message: null });
      setNotice(`Published “${updated.page.entries[updated.page.entries.length - 1]?.title ?? cleanTitle}” to ${updated.profile.displayName}'s public profile.`);
    } catch (publishError) {
      if (isCurrent()) setError(errorText(publishError, "This replay could not be published."));
    } finally {
      if (isCurrent()) setPending(false);
    }
  };

  useEffect(() => {
    if (open && uid && eligible) {
      setTitle(`Replay after move ${moveIndex}`);
      void loadPublisher(uid);
      backRef.current?.focus({ preventScroll: true });
      backRef.current?.scrollIntoView?.({ block: "nearest" });
    } else {
      requestRef.current += 1;
      setPending(false);
    }
    // The opening position is fixed while this sidebar replaces playback controls.
  }, [open, uid, eligible, match.id]);

  useEffect(() => () => { requestRef.current += 1; }, []);

  if (!eligible || !uid) return null;

  return (
    <section className={styles.section} aria-label="Publish replay" hidden={!open} onKeyDown={(event) => {
      if (event.key === "Escape") { event.preventDefault(); onClose(); }
    }}>
      <button className={`uiAction uiActionNeutral ${styles.back}`} type="button" ref={backRef} onClick={onClose}>
        <Icon name="back" /><span className="uiActionLabel">Back to replay</span>
      </button>
      <div className={styles.heading}><Icon name="publish" /><h2>Publish replay</h2></div>
      {open && (
        <div className={styles.body}>
          {publisherForCurrentUid?.status === "loading" && <p role="status">Loading your public profile.</p>}
          {!publisherForCurrentUid && (
            <button className="uiAction uiActionNeutral" type="button" onClick={() => void loadPublisher(uid)}>
              Load public profile
            </button>
          )}
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

          {publisherForCurrentUid?.status === "ready" && !publisherForCurrentUid.profile?.enabled && (
            <div className={styles.choice}>
              <p className={styles.note}>Enable your public profile to publish games.</p>
              <Link className="uiAction uiActionSecondary" to="/profile?tab=public"><Icon name="settings" />Manage public profile</Link>
            </div>
          )}

          {publisherForCurrentUid?.status === "ready" && publisherForCurrentUid.profile?.enabled && (
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
                    <input className="uiInput" maxLength={160} value={title}
                      onChange={(event) => { setTitle(Array.from(event.target.value).slice(0, 80).join("")); setError(null); }} />
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
                  <button className="uiAction uiActionPrimary" type="button" disabled={pending || !title.trim()}
                    onClick={() => void publish()}>
                    <Icon name="publish" />{pending ? "Publishing replay" : "Publish this replay"}
                  </button>
                  <p className={styles.note}>Anyone can copy this replay. Unpublishing removes the listing; shared copies remain.</p>
                  <p className={styles.access}>Casual record · Unverified result</p>
                </>
              )}
            </>
          )}
        </div>
      )}
    </section>
  );
}
