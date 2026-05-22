import { useStore } from "zustand";

import { cloudAuthStore } from "../cloud/auth_store";
import { cloudHistoryStore, type CloudHistoryUserCache } from "../cloud/cloud_history_store";
import type { CloudMatchHistory } from "../cloud/cloud_profile";
import { cloudProfileStore } from "../cloud/cloud_profile_store";
import {
  DEFAULT_LOCAL_DISPLAY_NAME,
  localProfileStore,
  type LocalProfileMatchHistory,
} from "./local_profile_store";
import { resolveActiveHistory } from "./active_history";

export interface ActiveHistoryView {
  cloudAuth: ReturnType<typeof cloudAuthStore.getState>;
  cloudHistory: ReturnType<typeof cloudHistoryStore.getState>;
  cloudProfile: ReturnType<typeof cloudProfileStore.getState>;
  cloudUserCache: CloudHistoryUserCache | null;
  hasPendingCloudMatchError: boolean;
  history: ReturnType<typeof resolveActiveHistory>;
  localDisplayName: string;
  localMatchHistory: LocalProfileMatchHistory;
  localProfile: ReturnType<typeof localProfileStore.getState>["profile"];
  pendingCloudMatchCount: number;
  sourceHistory: CloudMatchHistory | LocalProfileMatchHistory | null;
}

export function useActiveHistory(): ActiveHistoryView {
  const cloudAuth = useStore(cloudAuthStore, (state) => state);
  const cloudHistory = useStore(cloudHistoryStore, (state) => state);
  const cloudProfile = useStore(cloudProfileStore, (state) => state);
  const localMatchHistory = useStore(localProfileStore, (state) => state.matchHistory);
  const localProfile = useStore(localProfileStore, (state) => state.profile);

  const cloudUserCache =
    cloudAuth.status === "signed_in" && cloudAuth.user
      ? cloudHistory.users[cloudAuth.user.uid] ?? null
      : null;
  const cloudCache = cloudUserCache?.cachedMatches ?? [];
  const localHistory = localMatchHistory.replayMatches;
  const pendingCloudMatches = cloudUserCache?.pendingMatches ?? {};

  return {
    cloudAuth,
    cloudHistory,
    cloudProfile,
    cloudUserCache,
    hasPendingCloudMatchError: Object.values(cloudUserCache?.sync ?? {}).some((sync) => (
      sync.status === "error" && sync.matchId in pendingCloudMatches
    )),
    history: resolveActiveHistory({
      cloudHistory: cloudCache,
      historyResetAt: cloudAuth.status === "signed_in" ? cloudProfile.profile?.resetAt : null,
      localHistory,
    }),
    localDisplayName: localProfile?.displayName ?? cloudAuth.user?.displayName ?? DEFAULT_LOCAL_DISPLAY_NAME,
    localMatchHistory,
    localProfile,
    pendingCloudMatchCount: Object.keys(pendingCloudMatches).length,
    sourceHistory:
      cloudAuth.status === "signed_in"
        ? cloudProfile.profile?.matchHistory ?? null
        : localMatchHistory,
  };
}
