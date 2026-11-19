import { useEffect } from "react";

import type { SavedMatchV2 } from "../match/saved_match";
import { localProfileStore } from "../profile/local_profile_store";
import { useActiveHistory, type ActiveHistoryView } from "../profile/use_active_history";

import { replayUndoFloor } from "./local_replay";

export interface ReplayMatchState {
  localDisplayName: string;
  match: SavedMatchV2 | null;
  replayFloor: number;
  replayMayStillLoad: boolean;
}

export function replayHistoryMayStillLoad({ cloudAuth, cloudHistory, cloudProfile, cloudUserCache }: ActiveHistoryView): boolean {
  return (
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
}

export function useReplayMatch(matchId: string | undefined): ReplayMatchState {
  const activeHistory = useActiveHistory();
  const { history, localDisplayName } = activeHistory;
  useEffect(() => {
    localProfileStore.getState().ensureLocalProfile();
  }, []);

  const match = history.find((entry) => entry.id === matchId) ?? null;

  return {
    localDisplayName,
    match,
    replayFloor: match ? replayUndoFloor(match) : 0,
    replayMayStillLoad: !match && replayHistoryMayStillLoad(activeHistory),
  };
}
