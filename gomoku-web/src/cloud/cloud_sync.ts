import type { CloudAuthUser } from "./auth_store";
import {
  cloudProfileSyncDue,
  cloudSettingsFromProfileSettings,
  type CloudProfile,
} from "./cloud_profile";
import { cloudProfileStore } from "./cloud_profile_store";
import { cloudProfilePromotionUpdate, planPromotedMatchHistory, type LocalProfilePromotionInput } from "./cloud_promotion";
import { cloudPromotionStore } from "./cloud_promotion_store";
import { restoreCloudSetup } from "./cloud_setup";
import { localProfileStore, type LocalProfileMatchHistory } from "../profile/local_profile_store";

export interface FlushCloudProfileSyncOptions {
  localMatchHistory?: LocalProfileMatchHistory;
}

export async function flushCloudProfileSync(
  user: CloudAuthUser,
  options: FlushCloudProfileSyncOptions = {},
): Promise<CloudProfile | null> {
  let localStore = localProfileStore.getState();
  const localProfile = localStore.profile ?? localStore.ensureLocalProfile();
  let settings = localStore.settings;
  let cloudState = cloudProfileStore.getState();

  if (cloudState.profile?.uid !== user.uid || cloudState.status !== "ready") {
    const loadedProfile = await cloudState.loadForUser(user, settings);
    cloudState = cloudProfileStore.getState();
    if (!loadedProfile || cloudState.profile !== loadedProfile) {
      return null;
    }
  }

  if (cloudState.profile?.uid !== user.uid || cloudState.status !== "ready") {
    return null;
  }

  restoreCloudSetup(user, cloudState.profile);
  localStore = localProfileStore.getState();
  if (localStore.profile?.id !== localProfile.id) return null;
  settings = localStore.settings;

  const localMatchHistory = options.localMatchHistory ?? localStore.matchHistory;
  const sourceProfile = cloudState.profile;
  const promotionInput: LocalProfilePromotionInput = {
    sourceProfile,
    cloudDisplayName: sourceProfile.displayName,
    cloudMatchHistory: sourceProfile.matchHistory,
    cloudSettings: sourceProfile.settings,
    localMatchHistory,
    localProfile,
    resetAt: sourceProfile.resetAt,
    settings,
    user,
  };
  const { matchHistory: nextMatchHistory } = planPromotedMatchHistory(promotionInput);
  const nextSettings = cloudSettingsFromProfileSettings(settings);
  const historyChanged = JSON.stringify(nextMatchHistory) !== JSON.stringify(sourceProfile.matchHistory);
  const profileChanged = cloudProfilePromotionUpdate(promotionInput) !== null;

  // A read/no-op is not a server write. Moving updatedAt would falsely close
  // the client sync gate for five minutes even when the server is eligible.
  if (!historyChanged && !profileChanged) return sourceProfile;

  if (!cloudProfileSyncDue(sourceProfile)) return sourceProfile;

  const promotionResult = await cloudPromotionStore.getState().promote(promotionInput);

  const currentCloudState = cloudProfileStore.getState();
  if (
    !promotionResult
    || currentCloudState.status !== "ready"
    || currentCloudState.profile !== sourceProfile
    || currentCloudState.profile.uid !== user.uid
    || localProfileStore.getState().profile?.id !== localProfile.id
  ) {
    return null;
  }

  const syncedAt = new Date().toISOString();
  const profilePatch: Partial<CloudProfile> = {
    matchHistory: nextMatchHistory,
    settings: nextSettings,
    updatedAt: syncedAt,
  };

  if (promotionResult?.profileDisplayNamePromoted && promotionResult.promotedDisplayName) {
    profilePatch.displayName = promotionResult.promotedDisplayName;
  }

  if (!cloudProfileStore.getState().applyLocalPatch(sourceProfile, profilePatch)) {
    return null;
  }
  return cloudProfileStore.getState().profile;
}
