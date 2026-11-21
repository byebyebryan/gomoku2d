import { useLayoutEffect, useRef, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";

import { localMatchSessionStore, startReplayContinuation } from "../game/local_match_session";
import type { LocalMatchResumeSeed } from "../game/local_match_store";
import type { SavedMatchV2 } from "../match/saved_match";
import { portableReplayFromSavedMatch, validatePortableReplay, type PortableReplayV1 } from "../replay/portable_replay";
import { portableReplayFingerprint } from "../replay/portable_replay_codec";

export function sourceReplayMove(state: unknown, match: SavedMatchV2): number | undefined {
  if (!state || typeof state !== "object" || !("sourceReplayFrame" in state)) return;
  const frame = state.sourceReplayFrame as { matchId?: unknown; move?: unknown } | undefined;
  if (frame?.matchId === match.id && typeof frame.move === "number" && Number.isInteger(frame.move)
    && frame.move >= 0 && frame.move <= match.move_count) {
    return frame.move;
  }
}

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
      const active = localMatchSessionStore.getState().matchStore?.getState();
      if (active?.status === "playing" && active.moves.length > 0
        && !window.confirm("Replace your active game with a new game from this position?")) return;
      if (requestRef.current !== request) return;

      const inheritedPly = seed.moves.length;
      startReplayContinuation({ ...seed, practiceOrigin: { sourceFingerprint, inheritedPly } }, {
        to: `${location.pathname}${location.search}${location.hash}`,
        state: { ...returnState, sourceReplayFrame: { matchId: match.id, move: inheritedPly } },
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

  return { error, pending, start };
}
