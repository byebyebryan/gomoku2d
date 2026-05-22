import { useEffect } from "react";

import type { SavedMatchV2 } from "../match/saved_match";
import { localProfileStore } from "../profile/local_profile_store";
import { useActiveHistory } from "../profile/use_active_history";

import { replayUndoFloor } from "./local_replay";

export interface ReplayMatchState {
  localDisplayName: string;
  match: SavedMatchV2 | null;
  replayFloor: number;
  replayMayStillLoad: boolean;
}

export function useReplayMatch(matchId: string | undefined): ReplayMatchState {
  const {
    cloudAuth,
    cloudHistory,
    cloudProfile,
    cloudUserCache,
    history,
    localDisplayName,
  } = useActiveHistory();

  useEffect(() => {
    localProfileStore.getState().ensureLocalProfile();
  }, []);

  const match = history.find((entry) => entry.id === matchId) ?? null;
  const replayMayStillLoad = !match && (
    (cloudAuth.isConfigured && cloudAuth.status === "loading")
    || (
      cloudAuth.status === "signed_in"
      && (
        cloudProfile.status === "loading"
        || (
          cloudProfile.status === "ready"
          && !cloudUserCache?.loadedAt
          && cloudHistory.loadStatus !== "error"
        )
      )
    )
  );

  return {
    localDisplayName,
    match,
    replayFloor: match ? replayUndoFloor(match) : 0,
    replayMayStillLoad,
  };
}
