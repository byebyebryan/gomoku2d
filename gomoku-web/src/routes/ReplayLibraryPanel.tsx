import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { useStore } from "zustand";

import { cloudAuthStore } from "../cloud/auth_store";
import { decodePortableReplay } from "../replay/portable_replay_codec";
import type { PortableReplayV1 } from "../replay/portable_replay";
import { currentReplayLibraryOwner, replayLibraryStore, type ReplayLibraryKind } from "../replay/replay_library_store";
import { EMPTY_LIBRARY_SYNC, replayLibrarySync } from "../replay/replay_library_sync";
import { importGuestReplayCopies, saveReplayCopy } from "../replay/replay_library_actions";
import { Icon } from "../ui/Icon";
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
  const sync = useStore(replayLibrarySync.store, (state) => owner === "guest" ? EMPTY_LIBRARY_SYNC : state.users[owner.slice(5)] ?? EMPTY_LIBRARY_SYNC);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const saved = bucket?.entries.some((entry) => entry.fingerprint === fingerprint);
  const cloudSaved = saved && owner !== "guest" && !bucket?.dirty && sync.loaded && sync.status === "synced";
  const sessionOnly = !!storageError && !cloudSaved;
  let saveMessage = "";
  if (cloudSaved) {
    saveMessage = "Saved in your account library.";
    if (storageError) saveMessage += " A copy could not be stored on this device.";
  } else if (storageError) {
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
    <button className="uiAction uiActionSecondary" type="button" disabled={saved || busy || authStatus === "loading"}
      onClick={async () => {
        setBusy(true); setError("");
        try { await saveReplayCopy(replay); } catch (failure) { setError(String(failure instanceof Error ? failure.message : failure)); }
        finally { setBusy(false); }
      }}><Icon name="save" /><span className="uiActionLabel">{saved ? sessionOnly ? "Saved for this session" : "Saved replay" : busy ? "Saving replay" : "Save replay"}</span></button>
    {!saved && !storageError ? <div className={styles.saveScope}>
      <span className="uiSectionLabel">Private copy</span>
      <span className={styles.saveTarget}>{owner === "guest" ? "This device" : "Account library"}</span>
    </div> : <p className={storageError || (owner !== "guest" && !cloudSaved) ? styles.saveNotice : styles.saveStatus}>{saveMessage}</p>}
    {(error || bucket?.error || storageError) && <p role="alert">{error || bucket?.error || storageError}</p>}
  </section>;
}

export function ReplayLibraryEntries({ kind }: { kind: ReplayLibraryKind }) {
  const owner = useOwner();
  const bucket = useStore(replayLibraryStore, (state) => state.owners[owner]);
  const [visible, setVisible] = useState(8);
  const [error, setError] = useState("");
  const entries = useMemo(() => (bucket?.entries ?? []).filter((entry) => entry.kind === kind), [bucket?.entries, kind]);
  useEffect(() => { setVisible(8); setError(""); }, [owner, kind]);
  if (entries.length === 0) return <p>{kind === "practice" ? "No practice games yet." : "No saved replays yet."}</p>;
  return <>
    <ul className={styles.entries}>
      {entries.slice(0, visible).map((entry) => {
        let description = "Replay unavailable";
        try {
          const replay = decodePortableReplay(entry.payload).replay;
          description = `${replay.black.name} · ${replay.white.name} · ${replay.moves.length} moves · ${replay.ruleset === "renju" ? "Renju" : "Freestyle"}`;
        } catch { /* Retain the removable record; the viewer reports validation details. */ }
        return <li className={styles.entry} key={entry.fingerprint}>
          <div><p>{description}</p><small>{new Date(entry.saved_at).toLocaleString()}</small></div>
          <div className={styles.actions}>
            <Link className="uiAction uiActionSecondary" to={`/replay/portable-${entry.fingerprint}`}>Inspect replay</Link>
            <button className="uiAction uiActionNeutral" type="button" onClick={async () => {
              setError("");
              try {
                await replayLibraryStore.getState().removeReplay(owner, entry.fingerprint);
                if (owner !== "guest" && replayLibrarySync.store.getState().users[owner.slice(5)]?.loaded) {
                  await replayLibrarySync.sync(owner.slice(5));
                }
              } catch (failure) { setError(failure instanceof Error ? failure.message : "Removal could not sync."); }
            }}>Remove replay</button>
          </div>
        </li>;
      })}
    </ul>
    {visible < entries.length && <button className="uiAction uiActionNeutral" type="button" onClick={() => setVisible(visible + 8)}>Show more replays</button>}
    {error && <p role="alert">{error}</p>}
  </>;
}

export function ReplayLibraryPanel() {
  const owner = useOwner();
  const uid = owner === "guest" ? null : owner.slice(5);
  const bucket = useStore(replayLibraryStore, (state) => state.owners[owner]);
  const storageError = useStore(replayLibraryStore, (state) => state.storageError);
  const guestCount = useStore(replayLibraryStore, (state) => state.owners.guest?.entries.length ?? 0);
  const sync = useStore(replayLibrarySync.store, (state) => uid ? state.users[uid] ?? EMPTY_LIBRARY_SYNC : EMPTY_LIBRARY_SYNC);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const run = async (operation: () => Promise<unknown>) => {
    setBusy(true); setError("");
    try { await operation(); } catch (failure) { setError(failure instanceof Error ? failure.message : "Library operation failed."); }
    finally { setBusy(false); }
  };
  useEffect(() => {
    setError("");
    if (uid) void run(() => replayLibrarySync.reload(uid));
  }, [uid]);
  return <section className={styles.panel} aria-label="Saved replays">
    <h2>Saved replays</h2>
    <p>Private copies for study. These do not count as games you played. Practice completions appear in My games.</p>
    {uid ? <>
      <p role="status">{sync.status === "synced" && !bucket?.dirty ? "Account library synced."
        : sync.status === "loading" ? "Loading account library…"
        : sync.status === "syncing" ? "Syncing account library…" : "Local copies have not been confirmed in the cloud."}</p>
      <div className={styles.actions}>
        <button className="uiAction uiActionSecondary" type="button" disabled={busy} onClick={() => void run(() => replayLibrarySync.reload(uid))}>Reload cloud library</button>
        <button className="uiAction uiActionSecondary" type="button" disabled={busy || sync.hasIncoming || sync.status === "conflict"} onClick={() => void run(() => replayLibrarySync.sync(uid))}>Sync saved replays</button>
        {guestCount > 0 && <button className="uiAction uiActionNeutral" type="button" disabled={busy || sync.hasIncoming || sync.status === "conflict"} onClick={() => void run(importGuestReplayCopies)}>Copy guest replays into account ({guestCount})</button>}
      </div>
      {sync.hasIncoming && <div className={styles.conflict}>
        <p>Review library changes: {bucket?.entries.length ?? 0} local copies; {sync.incoming?.entries.length ?? 0} cloud copies.</p>
        <p>Merging keeps both lists, including cloud copies removed locally. Previously removed local copies may be added back only if you choose to merge.</p>
        <div className={styles.actions}>
          <button className="uiAction uiActionSecondary" type="button" disabled={busy} onClick={() => {
            if (window.confirm("Replace this device's account library with the reviewed cloud library? Unsynced local copies will be removed from this library.")) {
              void run(() => replayLibrarySync.useCloud(uid));
            }
          }}>Use cloud library</button>
          <button className="uiAction uiActionSecondary" type="button" disabled={busy} onClick={() => void run(() => replayLibrarySync.mergeCopies(uid))}>Merge copies into cloud</button>
        </div>
      </div>}
      {storageError && <p>Device storage is unavailable. Copies not confirmed in your account library are kept for this session only; keep their replay links in case you reload or close this tab.</p>}
    </> : <p>{storageError
      ? "Guest copies are kept for this session only and will be lost when you reload or close this tab. Keep their replay links to open them again."
      : "Guest copies stay on this device."} Sign in to copy them to an account explicitly.</p>}
    {(error || sync.error || bucket?.error || storageError) && <p role="alert">{error || sync.error || bucket?.error || storageError}</p>}
    <ReplayLibraryEntries kind="saved" />
  </section>;
}
