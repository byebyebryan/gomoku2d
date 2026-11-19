import { useEffect, useState } from "react";
import { Link, Navigate, useLocation } from "react-router-dom";
import { useStore } from "zustand";

import { cloudAuthStore } from "../cloud/auth_store";
import { cloudProfileStore } from "../cloud/cloud_profile_store";
import { currentReplayLibraryOwner, replayLibraryStore } from "../replay/replay_library_store";
import { replayLibrarySync } from "../replay/replay_library_sync";
import { updateReplayDetails } from "../replay/replay_library_actions";
import { decodePortableReplay } from "../replay/portable_replay_codec";
import { portableReplayToSavedMatch, type PortableReplaySavedMatchProjection } from "../replay/portable_replay_adapters";
import { ReplayViewer } from "./ReplayRoute";
import { SAVED_REPLAYS_PATH, savedReplayReturnFromState } from "./saved_replay_navigation";
import { myGamesPublishIntentFromState } from "./my_games_navigation";
import styles from "./ReplayRoute.module.css";

export function StoredReplayRoute({ fingerprint }: { fingerprint: string }) {
  const auth = useStore(cloudAuthStore);
  const location = useLocation();
  const launch = myGamesPublishIntentFromState(location.state);
  const resetAt = useStore(cloudProfileStore, (state) => state.profile?.uid === auth.user?.uid ? state.profile?.resetAt : null);
  const owner = currentReplayLibraryOwner();
  const entry = useStore(replayLibraryStore, (state) => state.owners[owner]?.entries.find((item) => item.fingerprint === fingerprint));
  const sync = useStore(replayLibrarySync.store, (state) => launch && auth.user ? state.users[auth.user.uid] : undefined);
  const [loaded, setLoaded] = useState<{
    owner: string; payload: string; initialMove: number | undefined; value: PortableReplaySavedMatchProjection;
  } | null>(null);
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
        if (!cancelled) setLoaded({ owner, payload: entry.payload, initialMove: entry.metadata.opening_ply ?? undefined, value });
      } catch (error) { if (!cancelled) setFailure(error instanceof Error ? error.message : "Replay unavailable."); }
    })();
    return () => { cancelled = true; };
  }, [auth.status, auth.user?.uid, entry?.payload, fingerprint, location.key, owner]);
  const origin = savedReplayReturnFromState(location.state, owner);
  const ownerPractice = entry?.kind === "practice" && (!resetAt || entry.saved_at > resetAt);
  if (launch && (failure || (entry ? !ownerPractice : sync?.loaded || sync?.status === "conflict"))) {
    return <Navigate to="/profile" replace state={{ myGamesFocusTab: true }} />;
  }
  if (loaded?.owner === owner && entry?.payload === loaded.payload) return <ReplayViewer key={`${owner}:${fingerprint}`}
    match={loaded.value.match} localDisplayName="" preserveNames source={loaded.value.replay}
    fingerprint={fingerprint} publishEligible={ownerPractice}
    initialMoveIndex={loaded.initialMove}
    details={entry.kind === "saved" ? {
      kind: "private",
      value: entry.metadata,
      onSave: (metadata, previous) => updateReplayDetails(owner, fingerprint, metadata, previous),
    } : undefined}
    privateReplayReturn={entry.kind === "saved" ? {
      to: SAVED_REPLAYS_PATH,
      state: origin ? { savedReplayReturn: origin } : undefined,
    } : undefined}
    contextLabel={entry.kind === "practice" ? "My game" : "Saved replay"} />;
  return <main className={styles.page}><section className={`${styles.notFound} uiPanel`}>
    <h1>{(entry || launch) && !failure ? "Loading saved replay" : "Replay unavailable"}</h1>
    <p>{failure || "Open Saved replays in Profile to load your library."}</p>
    <Link className="uiAction uiActionSecondary" to="/profile">Profile</Link>
  </section></main>;
}
