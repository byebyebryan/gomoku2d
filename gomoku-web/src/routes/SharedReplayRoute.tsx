import { useEffect, useState } from "react";
import { Link, useLocation } from "react-router-dom";

import { portableReplayToSavedMatch, type PortableReplaySavedMatchProjection } from "../replay/portable_replay_adapters";
import { decodePortableReplayFragment } from "../replay/portable_replay_codec";
import { useDocumentTitle } from "../app/useDocumentTitle";
import { ReplayViewer } from "./ReplayRoute";
import styles from "./ReplayRoute.module.css";
import { publicProfilePath, publicReplayReturnWithLocationState } from "./public_player_navigation";

export function SharedReplayRoute() {
  useDocumentTitle("Shared Replay");
  const location = useLocation();
  const { hash, search, state } = location;
  const publicReplayReturn = publicReplayReturnWithLocationState(search, state);
  const [loaded, setLoaded] = useState<{ hash: string; value: PortableReplaySavedMatchProjection } | null>(null);
  const [failure, setFailure] = useState<{ hash: string; message: string } | null>(null);
  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const decoded = decodePortableReplayFragment(hash);
        const value = await portableReplayToSavedMatch(decoded.replay);
        if (!cancelled) setLoaded({ hash, value });
      } catch (error) {
        if (!cancelled) setFailure({ hash, message: error instanceof Error ? error.message : "This link could not be opened." });
      }
    })();
    return () => { cancelled = true; };
  }, [hash]);
  if (loaded?.hash === hash) return <ReplayViewer key={hash} match={loaded.value.match}
    localDisplayName="" preserveNames source={loaded.value.replay}
    fingerprint={loaded.value.fingerprint}
    initialMoveIndex={loaded.value.replay.viewPly ?? loaded.value.match.move_count}
    publicReplayReturn={publicReplayReturn ?? undefined} />;
  const returnLink = publicReplayReturn ? (
    <Link
      className="uiAction uiActionSecondary"
      state={{ publicReplayReturn }}
      to={publicProfilePath(publicReplayReturn.uid, publicReplayReturn.page - 1)}
    >
      Return to public profile
    </Link>
  ) : null;
  return <main className={styles.page}><section className={`${styles.notFound} uiPanel`}>
    <h1>{failure?.hash === hash ? "Replay unavailable" : "Loading shared replay"}</h1>
    <p>{failure?.hash === hash ? failure.message : "Checking the game with the board rules."}</p>
    {returnLink}
    <Link className="uiAction uiActionSecondary" to="/">Home</Link>
  </section></main>;
}
