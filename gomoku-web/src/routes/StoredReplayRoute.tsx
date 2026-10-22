import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { useStore } from "zustand";

import { cloudAuthStore } from "../cloud/auth_store";
import { currentReplayLibraryOwner, replayLibraryStore } from "../replay/replay_library_store";
import { replayLibrarySync } from "../replay/replay_library_sync";
import { decodePortableReplay } from "../replay/portable_replay_codec";
import { portableReplayToSavedMatch, type PortableReplaySavedMatchProjection } from "../replay/portable_replay_adapters";
import { ReplayViewer } from "./ReplayRoute";
import styles from "./ReplayRoute.module.css";

export function StoredReplayRoute({ fingerprint }: { fingerprint: string }) {
  const auth = useStore(cloudAuthStore);
  const owner = currentReplayLibraryOwner();
  const entry = useStore(replayLibraryStore, (state) => state.owners[owner]?.entries.find((item) => item.fingerprint === fingerprint));
  const [loaded, setLoaded] = useState<{ owner: string; payload: string; value: PortableReplaySavedMatchProjection } | null>(null);
  const [failure, setFailure] = useState("");
  useEffect(() => {
    let cancelled = false;
    setFailure("");
    if (!entry && auth.status === "signed_in" && auth.user
      && !replayLibrarySync.store.getState().users[auth.user.uid]?.loaded) {
      void replayLibrarySync.reload(auth.user.uid).catch((error: unknown) => {
        if (!cancelled) setFailure(error instanceof Error ? error.message : "The account library could not load.");
      });
    }
    if (entry) void (async () => {
      try {
        const value = await portableReplayToSavedMatch(decodePortableReplay(entry.payload).replay);
        if (value.fingerprint !== fingerprint) throw new Error("This saved replay does not match its content fingerprint.");
        if (!cancelled) setLoaded({ owner, payload: entry.payload, value });
      } catch (error) { if (!cancelled) setFailure(error instanceof Error ? error.message : "Replay unavailable."); }
    })();
    return () => { cancelled = true; };
  }, [auth.status, auth.user?.uid, entry?.payload, fingerprint, owner]);
  if (loaded?.owner === owner && entry?.payload === loaded.payload) return <ReplayViewer key={`${owner}:${fingerprint}`}
    match={loaded.value.match} localDisplayName="" preserveNames source={loaded.value.replay}
    fingerprint={fingerprint} publishEligible={entry.kind === "practice"}
    contextLabel={entry.kind === "practice" ? "Practice game" : "Saved replay"} />;
  return <main className={styles.page}><section className={`${styles.notFound} uiPanel`}>
    <h1>{entry && !failure ? "Loading saved replay" : "Replay unavailable"}</h1>
    <p>{failure || "Open Saved replays in Profile to load your library."}</p>
    <Link className="uiAction uiActionSecondary" to="/profile">Profile</Link>
  </section></main>;
}
