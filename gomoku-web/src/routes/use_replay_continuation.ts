import { useLayoutEffect, useRef, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { useStore } from "zustand";

import { localMatchSessionStore, startReplayContinuation, type LocalMatchSessionState } from "../game/local_match_session";
import type { LocalMatchResumeSeed } from "../game/local_match_store";
import type { SavedMatchV2 } from "../match/saved_match";
import { portableReplayFromSavedMatch, validatePortableReplay, type PortableReplayV1 } from "../replay/portable_replay";
import { portableReplayFingerprint } from "../replay/portable_replay_codec";
import { confirmLocalMatchReplacement } from "./confirm_local_match_replacement";
import { sourceReplayMove, sourceReplayToken } from "./source_replay_navigation";

export function useReplayContinuation({
  match, moveIndex, source, fingerprint, resumePath, returnState,
}: {
  match: SavedMatchV2;
  moveIndex: number;
  source?: PortableReplayV1;
  fingerprint?: string;
  resumePath: string;
  returnState: Record<string, unknown>;
}) {
  const location = useLocation();
  const navigate = useNavigate();
  const requestRef = useRef<object | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const sourcePath = `${location.pathname}${location.search}${location.hash}`;
  const visitedToken = sourceReplayToken(location.state);
  const visitedMove = sourceReplayMove(location.state, match);

  // Session updates cover both source replacement and a bot finishing while Replay is open.
  function canResumeCurrent({ latestReplayId, matchStore, sourceReplay }: LocalMatchSessionState): boolean {
    return Boolean(visitedToken && visitedMove !== undefined
      && latestReplayId === null && matchStore?.getState().status === "playing"
      && sourceReplay?.to === sourcePath
      && sourceReplayToken(sourceReplay.state) === visitedToken
      && sourceReplayMove(sourceReplay.state, match) === visitedMove);
  }

  const canResume = useStore(localMatchSessionStore, canResumeCurrent);

  useLayoutEffect(() => {
    setPending(false);
    setError("");
    return () => { requestRef.current = null; };
  }, [location, match.id, moveIndex]);

  async function start(seed: LocalMatchResumeSeed) {
    if (requestRef.current) return;
    const request = {};
    requestRef.current = request;
    setPending(true);
    setError("");
    let launched = false;
    try {
      const portable = source ?? portableReplayFromSavedMatch(match).replay;
      const sourceFingerprint = fingerprint ?? await portableReplayFingerprint(validatePortableReplay(portable));
      if (requestRef.current !== request) return;
      if (!confirmLocalMatchReplacement("Replace your active game with a new game from this position?")) return;
      if (requestRef.current !== request) return;

      const inheritedPly = seed.moves.length;
      const token = Array.from(crypto.getRandomValues(new Uint32Array(4))).join("-");
      startReplayContinuation({ ...seed, practiceOrigin: { sourceFingerprint, inheritedPly } }, {
        to: sourcePath,
        state: {
          ...returnState,
          sourceReplayFrame: { matchId: match.id, move: inheritedPly },
          sourceReplayToken: token,
        },
      });
      navigate(resumePath);
      launched = true;
    } catch (failure) {
      if (requestRef.current === request) {
        setError(failure instanceof Error ? failure.message : "The game could not start.");
      }
    } finally {
      if (!launched && requestRef.current === request) {
        requestRef.current = null;
        setPending(false);
      }
    }
  }

  const resumeCurrent = canResume && visitedToken ? () => {
    if (canResumeCurrent(localMatchSessionStore.getState())) {
      navigate(resumePath, { state: { resumeSourceToken: visitedToken } });
    }
  } : undefined;

  return { error, pending, resumeCurrent, start };
}
