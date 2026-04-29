import type { CloudAuthUser } from "./auth_store";
import { cloudProfileSyncDue, mergeCloudRecentMatches, type CloudProfile } from "./cloud_profile";
import { cloudProfileStore } from "./cloud_profile_store";
import { cloudPromotionStore } from "./cloud_promotion_store";
import { DEFAULT_GUEST_DISPLAY_NAME, guestProfileStore, type GuestSavedMatch } from "../profile/guest_profile_store";

export interface FlushCloudProfileSyncOptions {
  guestHistory?: GuestSavedMatch[];
}

export async function flushCloudProfileSync(
  user: CloudAuthUser,
  options: FlushCloudProfileSyncOptions = {},
): Promise<CloudProfile | null> {
  const guestStore = guestProfileStore.getState();
  const guestProfile = guestStore.profile ?? guestStore.ensureGuestProfile();
  const settings = guestStore.settings;
  let cloudState = cloudProfileStore.getState();

  if (cloudState.profile?.uid !== user.uid || cloudState.status !== "ready") {
    await cloudState.loadForUser(user, settings.preferredVariant);
    cloudState = cloudProfileStore.getState();
  }

  if (cloudState.profile?.uid !== user.uid || cloudState.status !== "ready") {
    return null;
  }

  const guestHistory = options.guestHistory ?? [];
  const nextRecentMatches = mergeCloudRecentMatches(
    user,
    [...cloudState.profile.recentMatches.matches, ...guestHistory],
    cloudState.profile.historyResetAt,
  );
  const historyChanged = JSON.stringify(nextRecentMatches) !== JSON.stringify(cloudState.profile.recentMatches.matches);
  const profileChanged =
    cloudState.profile.preferredVariant !== settings.preferredVariant
    || (
      guestProfile.displayName.trim()
      && guestProfile.displayName !== DEFAULT_GUEST_DISPLAY_NAME
      && cloudState.profile.displayName === user.displayName
    );

  if ((historyChanged || profileChanged) && !cloudProfileSyncDue(cloudState.profile)) {
    return cloudState.profile;
  }

  await cloudPromotionStore.getState().promote({
    cloudDisplayName: cloudState.profile.displayName,
    cloudHistory: cloudState.profile.recentMatches.matches,
    cloudPreferredVariant: cloudState.profile.preferredVariant,
    guestHistory,
    guestProfile,
    historyResetAt: cloudState.profile.historyResetAt,
    settings,
    user,
  });

  if (cloudPromotionStore.getState().status !== "complete") {
    return cloudProfileStore.getState().profile;
  }

  const promotionResult = cloudPromotionStore.getState().result;
  const syncedAt = new Date().toISOString();
  const profilePatch: Partial<CloudProfile> = {
    preferredVariant: settings.preferredVariant,
    recentMatches: {
      matches: nextRecentMatches,
      schemaVersion: cloudState.profile.recentMatches.schemaVersion,
      updatedAt: syncedAt,
    },
    updatedAt: syncedAt,
  };

  if (promotionResult?.profileDisplayNamePromoted && promotionResult.promotedDisplayName) {
    profilePatch.displayName = promotionResult.promotedDisplayName;
  }

  cloudProfileStore.getState().applyLocalPatch(profilePatch);
  return cloudProfileStore.getState().profile;
}
