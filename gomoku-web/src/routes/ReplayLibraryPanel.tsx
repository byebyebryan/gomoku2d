import { useEffect, useMemo, useRef, useState } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { useStore } from "zustand";

import { cloudAuthStore } from "../cloud/auth_store";
import { decodePortableReplay } from "../replay/portable_replay_codec";
import type { PortableReplayV1 } from "../replay/portable_replay";
import { currentReplayLibraryOwner, replayLibraryStore } from "../replay/replay_library_store";
import {
  commitReplayLibraryMerge, EMPTY_LIBRARY_SYNC, planReplayLibraryMerge, replayLibrarySync, type ReplayLibraryMergePlan,
} from "../replay/replay_library_sync";
import {
  commitGuestReplayImport, planGuestReplayImport, saveReplayCopy, type ReplayLibraryImportPlan,
} from "../replay/replay_library_actions";
import { Icon } from "../ui/Icon";
import { profileRecordScroller } from "./profile_scroll";
import { ReplayThumbnail } from "./ReplayThumbnail";
import { ReplayLibraryReview, replayReviewTitle, type ReplayReviewRow } from "./ReplayLibraryReview";
import {
  rememberSavedReplayReturn, savedReplayControlId, savedReplayReturnFromState, SAVED_REPLAYS_VISIBLE_BATCH,
} from "./saved_replay_navigation";
import styles from "./ReplayLibraryPanel.module.css";

function useOwner() {
  useStore(cloudAuthStore, (state) => state.status);
  useStore(cloudAuthStore, (state) => state.user?.uid);
  return currentReplayLibraryOwner();
}

export function SaveReplayButton({ replay, fingerprint }: { replay: PortableReplayV1; fingerprint: string }) {
  const owner = useOwner();
  const authStatus = useStore(cloudAuthStore, (state) => state.status);
  const bucket = useStore(replayLibraryStore, (state) => state.owners[owner]);
  const storageError = useStore(replayLibraryStore, (state) => state.storageError);
  const readOnly = useStore(replayLibraryStore, (state) => state.readOnly);
  const volatile = useStore(replayLibraryStore, (state) => state.sessionOnly);
  const sync = useStore(replayLibrarySync.store, (state) => owner === "guest" ? EMPTY_LIBRARY_SYNC : state.users[owner.slice(5)] ?? EMPTY_LIBRARY_SYNC);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const saved = bucket?.entries.some((entry) => entry.fingerprint === fingerprint);
  const cloudSaved = saved && owner !== "guest" && !bucket?.dirty && sync.loaded && sync.status === "synced";
  const sessionOnly = !cloudSaved && (saved ? bucket?.durable === false : volatile);
  let saveMessage = "";
  if (cloudSaved) {
    saveMessage = "Saved in your account library.";
    if (storageError) saveMessage += " A copy could not be stored on this device.";
  } else if (sessionOnly) {
    saveMessage = saved
      ? "Saved for this session only. This copy will be lost when you reload or close this tab. Keep the replay link to open it again."
      : "Device storage is unavailable. Saving keeps a copy for this session only; keep the replay link to open it again.";
    if (owner !== "guest") saveMessage += " Account sync has not been confirmed. Open Saved replays in Profile to review.";
  } else if (saved) {
    saveMessage = owner === "guest" ? "Saved on this device"
      : bucket?.dirty
        ? "Saved on this device; account sync is pending. Open Saved replays in Profile to review."
        : "Saved on this device; open Saved replays to confirm cloud copies.";
  }
  return <section className={styles.panel} aria-label="Save replay">
    <button className="uiAction uiActionSecondary" type="button" disabled={saved || busy || readOnly || authStatus === "loading"}
      onClick={async () => {
        setBusy(true); setError("");
        try { await saveReplayCopy(replay); } catch (failure) { setError(String(failure instanceof Error ? failure.message : failure)); }
        finally { setBusy(false); }
      }}><Icon name="save" /><span className="uiActionLabel">{saved ? sessionOnly ? "Saved for this session" : "Saved replay" : busy ? "Saving replay" : "Save replay"}</span></button>
    {!saved && !readOnly && !volatile ? <div className={styles.saveScope}>
      <span className="uiSectionLabel">Private copy</span>
      <span className={styles.saveTarget}>{owner === "guest" ? "This device" : "Account library"}</span>
    </div> : saveMessage && <p className={storageError || (owner !== "guest" && !cloudSaved) ? styles.saveNotice : styles.saveStatus}>{saveMessage}</p>}
    {(error || bucket?.error || storageError) && <p role="alert">{error || bucket?.error || storageError}</p>}
  </section>;
}

function ReplayLibraryEntries() {
  const owner = useOwner();
  const location = useLocation();
  const navigate = useNavigate();
  const origin = useMemo(() => savedReplayReturnFromState(location.state, owner), [location.state, owner]);
  const returned = useRef("");
  const bucket = useStore(replayLibraryStore, (state) => state.owners[owner]);
  const [visible, setVisible] = useState(origin?.visible ?? SAVED_REPLAYS_VISIBLE_BATCH);
  const [error, setError] = useState("");
  const entries = useMemo(() => (bucket?.entries ?? []).filter((entry) => entry.kind === "saved"), [bucket?.entries]);
  const rows = useMemo(() => entries.slice(0, visible).map((entry) => {
    try {
      const { replay } = decodePortableReplay(entry.payload);
      const opening = entry.metadata.opening_ply ?? replay.moves.length;
      return {
        entry,
        title: entry.metadata.title || `${replay.black.name} · ${replay.white.name}`,
        info: `${replay.ruleset === "renju" ? "Renju" : "Freestyle"} · ${replay.moves.length} moves`,
        position: { moves: replay.moves, opening },
      };
    } catch {
      return { entry, title: "Replay unavailable", info: "", position: null };
    }
  }), [entries, visible]);

  useEffect(() => {
    if (!origin || returned.current === location.key) return;
    const request = window.requestAnimationFrame(() => {
      const control = document.getElementById(savedReplayControlId(origin.fingerprint));
      if (!control) return;
      returned.current = location.key;
      const scroller = profileRecordScroller(control);
      if (scroller) scroller.scrollTop = origin.scrollTop;
      control.focus({ preventScroll: true });
      control.scrollIntoView({ block: "nearest", inline: "nearest" });
    });
    return () => window.cancelAnimationFrame(request);
  }, [location.key, origin, rows.length]);

  if (entries.length === 0) return <p>No saved replays yet.</p>;
  return <>
    <ul className={styles.entries}>
      {rows.map(({ entry, title, info, position }) => (
        <li className={styles.entry} key={entry.fingerprint}>
          <div className={styles.thumbnail}><ReplayThumbnail position={position} /></div>
          <div className={styles.entryCopy}>
            <h3>{title}</h3>
            {info && <small>{info}</small>}
            <small>
              {position && (position.opening === position.moves.length ? "Final move · " : `Opens at move ${position.opening} · `)}
              Saved {new Date(entry.saved_at).toLocaleDateString()}
            </small>
          </div>
          <div className={styles.actions}>
            <Link aria-label="Inspect replay" className="uiAction uiActionSecondary" id={savedReplayControlId(entry.fingerprint)}
              to={`/replay/portable-${entry.fingerprint}`}
              onClick={(event) => {
                if (event.button !== 0 || event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) return;
                const savedReplayReturn = {
                  owner, fingerprint: entry.fingerprint, visible,
                  scrollTop: Math.max(0, profileRecordScroller(event.currentTarget)?.scrollTop ?? 0),
                };
                rememberSavedReplayReturn(savedReplayReturn);
                event.preventDefault();
                navigate(`/replay/portable-${entry.fingerprint}`, { state: { savedReplayReturn } });
              }}>
              <Icon name="replay" /><span className="uiActionLabel">Replay</span>
            </Link>
            <button aria-label="Remove replay" className="uiAction uiActionNeutral uiActionIconOnly" type="button" onClick={async () => {
              if (!window.confirm("Remove this saved replay and its private details?")) return;
              setError("");
              try {
                await replayLibraryStore.getState().removeReplay(owner, entry.fingerprint);
                if (owner !== "guest" && replayLibrarySync.store.getState().users[owner.slice(5)]?.loaded) {
                  await replayLibrarySync.sync(owner.slice(5));
                }
              } catch (failure) { setError(failure instanceof Error ? failure.message : "Removal could not sync."); }
            }}><Icon name="trash" /></button>
          </div>
        </li>
      ))}
    </ul>
    {visible < entries.length && <button className="uiAction uiActionNeutral" type="button"
      onClick={() => setVisible(Math.min(64, visible + SAVED_REPLAYS_VISIBLE_BATCH))}>Show more replays</button>}
    {error && <p role="alert">{error}</p>}
  </>;
}

export function ReplayLibraryPanel() {
  const owner = useOwner();
  const uid = owner === "guest" ? null : owner.slice(5);
  const bucket = useStore(replayLibraryStore, (state) => state.owners[owner]);
  const storageError = useStore(replayLibraryStore, (state) => state.storageError);
  const readOnly = useStore(replayLibraryStore, (state) => state.readOnly);
  const volatile = useStore(replayLibraryStore, (state) => state.sessionOnly);
  const guestCount = useStore(replayLibraryStore, (state) => state.owners.guest?.entries.length ?? 0);
  const sync = useStore(replayLibrarySync.store, (state) => uid ? state.users[uid] ?? EMPTY_LIBRARY_SYNC : EMPTY_LIBRARY_SYNC);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [review, setReview] = useState<{
    kind: "merge"; plan: ReplayLibraryMergePlan;
  } | {
    kind: "import"; plan: ReplayLibraryImportPlan;
  } | null>(null);
  const visibleReview = review && review.plan.uid === uid ? review : null;
  const syncError = visibleReview && sync.hasIncoming ? null : sync.error;
  const reviewRows = useMemo<ReplayReviewRow[]>(() => !visibleReview ? [] : visibleReview.kind === "merge"
    ? visibleReview.plan.rows.map(({ fingerprint, status, local, cloud }) => ({
      fingerprint, conflict: status === "metadata-conflict", local, incoming: cloud,
      title: replayReviewTitle((local ?? cloud)!),
    }))
    : visibleReview.plan.rows.map(({ fingerprint, status, account, guest }) => ({
      fingerprint, conflict: status === "metadata-conflict", local: account, incoming: guest,
      title: replayReviewTitle(account ?? guest),
    })), [visibleReview]);
  const run = async (operation: () => Promise<unknown>) => {
    setBusy(true); setError("");
    try { await operation(); } catch (failure) { setError(failure instanceof Error ? failure.message : "Library operation failed."); }
    finally { setBusy(false); }
  };
  useEffect(() => {
    setError("");
    setReview(null);
    if (uid) void run(() => replayLibrarySync.reload(uid));
  }, [uid]);
  return <section className={styles.panel} aria-label="Saved replays">
    <h2>Saved replays</h2>
    <p>Private copies for study. These do not count as games you played.</p>
    {uid ? <>
      <p role="status">{sync.status === "synced" && !bucket?.dirty ? "Account library synced."
        : sync.status === "loading" ? "Loading account library…"
        : sync.status === "syncing" ? "Syncing account library…" : "Local copies have not been confirmed in the cloud."}</p>
      <div className={styles.actions}>
        <button className="uiAction uiActionSecondary" type="button" disabled={busy} onClick={() => {
          setReview(null);
          void run(async () => {
            await replayLibraryStore.getState().refreshFromStorage();
            if (currentReplayLibraryOwner() === owner) await replayLibrarySync.reload(uid);
          });
        }}><Icon name="reset" />Reload cloud library</button>
        <button className="uiAction uiActionSecondary" type="button" disabled={busy || readOnly || !!visibleReview || sync.hasIncoming || sync.status === "conflict"} onClick={() => void run(() => replayLibrarySync.sync(uid))}><Icon name="sync" />Sync saved replays</button>
        {guestCount > 0 && <button className="uiAction uiActionNeutral" type="button"
          disabled={busy || readOnly || !!visibleReview || sync.hasIncoming || sync.status === "conflict"}
          onClick={() => void run(async () => {
            const plan = await planGuestReplayImport(uid);
            if (currentReplayLibraryOwner() === owner) setReview({ kind: "import", plan });
          })}><Icon name="copy" />Copy guest replays into account ({guestCount})</button>}
      </div>
      {sync.hasIncoming && !visibleReview && <div className={styles.conflict}>
        <p>Review library changes: {bucket?.entries.length ?? 0} local copies; {sync.incoming?.entries.length ?? 0} cloud copies.</p>
        <p>Merging keeps both lists, including cloud copies removed locally. Previously removed local copies may be added back only if you choose to merge.</p>
        <div className={styles.actions}>
          <button className="uiAction uiActionSecondary" type="button" disabled={busy} onClick={() => {
            if (window.confirm("Replace this device's account library with the reviewed cloud library? Unsynced local copies will be removed from this library.")) {
              void run(() => replayLibrarySync.useCloud(uid, true));
            }
          }}>Use cloud library</button>
          <button className="uiAction uiActionSecondary" type="button" disabled={busy} onClick={() => void run(async () => {
            const plan = await planReplayLibraryMerge(uid);
            if (currentReplayLibraryOwner() === owner) setReview({ kind: "merge", plan });
          })}>Review merge</button>
        </div>
      </div>}
      {visibleReview && <ReplayLibraryReview kind={visibleReview.kind} rows={reviewRows} busy={busy}
        onCancel={() => setReview(null)} onApply={(choices) => void run(async () => {
          if (visibleReview.kind === "merge") {
            const selected = Object.fromEntries(Object.entries(choices).map(([fingerprint, choice]) =>
              [fingerprint, choice === "local" ? "local" : "cloud"] as const));
            await commitReplayLibraryMerge(visibleReview.plan, selected);
          } else {
            const selected = Object.fromEntries(Object.entries(choices).map(([fingerprint, choice]) =>
              [fingerprint, choice === "local" ? "account" : "guest"] as const));
            await commitGuestReplayImport(visibleReview.plan, selected);
          }
          if (currentReplayLibraryOwner() === owner) setReview(null);
        })} />}
      {volatile && <p>Device storage is unavailable. Copies not confirmed in your account library are kept for this session only; keep their replay links in case you reload or close this tab.</p>}
    </> : !readOnly && <p>{volatile
      ? "Guest copies are kept for this session only and will be lost when you reload or close this tab. Keep their replay links to open them again."
      : "Guest copies stay on this device."} Sign in to copy them to an account explicitly.</p>}
    {(error || syncError || bucket?.error || storageError) && <p role="alert">{error || syncError || bucket?.error || storageError}</p>}
    <ReplayLibraryEntries key={owner} />
  </section>;
}
