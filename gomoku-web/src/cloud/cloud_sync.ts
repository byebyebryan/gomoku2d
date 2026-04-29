import type { CloudAuthUser } from "./auth_store";
import type { CloudProfile } from "./cloud_profile";
import { cloudProfileStore } from "./cloud_profile_store";
import { cloudPromotionStore } from "./cloud_promotion_store";
import { guestProfileStore, type GuestSavedMatch } from "../profile/guest_profile_store";

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

  await cloudPromotionStore.getState().promote({
    cloudDisplayName: cloudState.profile.displayName,
    cloudPreferredVariant: cloudState.profile.preferredVariant,
    guestHistory: options.guestHistory ?? [],
    guestProfile,
    historyResetAt: cloudState.profile.historyResetAt,
    settings,
    user,
  });

  if (cloudPromotionStore.getState().status !== "complete") {
    return cloudProfileStore.getState().profile;
  }

  const promotionResult = cloudPromotionStore.getState().result;
  const profilePatch: Partial<Pick<CloudProfile, "displayName" | "preferredVariant">> = {
    preferredVariant: settings.preferredVariant,
  };

  if (promotionResult?.profileDisplayNamePromoted && promotionResult.promotedDisplayName) {
    profilePatch.displayName = promotionResult.promotedDisplayName;
  }

  cloudProfileStore.getState().applyLocalPatch(profilePatch);
  return cloudProfileStore.getState().profile;
}
