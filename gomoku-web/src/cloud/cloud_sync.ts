import type { CloudAuthUser } from "./auth_store";
import {
  cloudProfileSyncDue,
  cloudSettingsForVariant,
  cloudMatchHistoryIsEmpty,
  mergeCloudArchivedMatchStats,
  mergeCloudMatchSummaryState,
  mergeCloudReplayMatches,
  type CloudProfile,
} from "./cloud_profile";
import { cloudProfileStore } from "./cloud_profile_store";
import { cloudPromotionStore } from "./cloud_promotion_store";
import { DEFAULT_LOCAL_DISPLAY_NAME, localProfileStore, type LocalProfileMatchHistory } from "../profile/local_profile_store";

export interface FlushCloudProfileSyncOptions {
  localMatchHistory?: LocalProfileMatchHistory;
}

export async function flushCloudProfileSync(
  user: CloudAuthUser,
  options: FlushCloudProfileSyncOptions = {},
): Promise<CloudProfile | null> {
  const localStore = localProfileStore.getState();
  const localProfile = localStore.profile ?? localStore.ensureLocalProfile();
  const settings = localStore.settings;
  let cloudState = cloudProfileStore.getState();

  if (cloudState.profile?.uid !== user.uid || cloudState.status !== "ready") {
    await cloudState.loadForUser(user, settings.preferredVariant);
    cloudState = cloudProfileStore.getState();
  }

  if (cloudState.profile?.uid !== user.uid || cloudState.status !== "ready") {
    return null;
  }

  const localMatchHistory = options.localMatchHistory ?? localStore.matchHistory;
  const localReplayMatches = localMatchHistory.replayMatches;
  const replayMatches = mergeCloudReplayMatches(
    user,
    [...cloudState.profile.matchHistory.replayMatches, ...localReplayMatches],
    cloudState.profile.resetAt,
  );
  const archivedStats = cloudMatchHistoryIsEmpty(cloudState.profile.matchHistory)
    ? mergeCloudArchivedMatchStats(
      cloudState.profile.matchHistory.archivedStats,
      localMatchHistory.archivedStats,
    )
    : cloudState.profile.matchHistory.archivedStats;
  const summaryState = mergeCloudMatchSummaryState({
    archivedStats,
    matches: [...cloudState.profile.matchHistory.replayMatches, ...localReplayMatches],
    replayMatches,
    resetAt: cloudState.profile.resetAt,
    summaries: [
      ...cloudState.profile.matchHistory.summaryMatches,
      ...localMatchHistory.summaryMatches.map((summary) => ({
        ...summary,
        trust: "client_uploaded" as const,
      })),
    ],
    user,
  });
  const nextMatchHistory = {
    archivedStats: summaryState.archivedStats,
    replayMatches,
    summaryMatches: summaryState.summaryMatches,
  };
  const nextSettings = cloudSettingsForVariant(settings.preferredVariant, settings.practiceBot);
  const historyChanged = JSON.stringify(nextMatchHistory) !== JSON.stringify(cloudState.profile.matchHistory);
  const profileChanged =
    JSON.stringify(cloudState.profile.settings) !== JSON.stringify(nextSettings)
    || (
      localProfile.displayName.trim()
      && localProfile.displayName !== DEFAULT_LOCAL_DISPLAY_NAME
      && cloudState.profile.displayName === user.displayName
    );

  if ((historyChanged || profileChanged) && !cloudProfileSyncDue(cloudState.profile)) {
    return cloudState.profile;
  }

  await cloudPromotionStore.getState().promote({
    cloudDisplayName: cloudState.profile.displayName,
    cloudMatchHistory: cloudState.profile.matchHistory,
    cloudSettings: cloudState.profile.settings,
    localMatchHistory,
    localProfile,
    resetAt: cloudState.profile.resetAt,
    settings,
    user,
  });

  if (cloudPromotionStore.getState().status !== "complete") {
    return cloudProfileStore.getState().profile;
  }

  const promotionResult = cloudPromotionStore.getState().result;
  const syncedAt = new Date().toISOString();
  const profilePatch: Partial<CloudProfile> = {
    matchHistory: nextMatchHistory,
    settings: nextSettings,
    updatedAt: syncedAt,
  };

  if (promotionResult?.profileDisplayNamePromoted && promotionResult.promotedDisplayName) {
    profilePatch.displayName = promotionResult.promotedDisplayName;
  }

  cloudProfileStore.getState().applyLocalPatch(profilePatch);
  return cloudProfileStore.getState().profile;
}
