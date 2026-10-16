import { localProfileStore } from "../profile/local_profile_store";

import { cloudAuthStore, type CloudAuthUser } from "./auth_store";
import type { CloudProfile } from "./cloud_profile";

let restoredSession: CloudAuthUser | null = null;
cloudAuthStore.subscribe((state, previous) => {
  if (state.user !== previous.user || state.status !== "signed_in") restoredSession = null;
});

/** Restore before any upload path captures local settings, once per sign-in. */
export function restoreCloudSetup(user: CloudAuthUser, profile: CloudProfile): void {
  if (cloudAuthStore.getState().user !== user || profile.uid !== user.uid || restoredSession === user) return;
  restoredSession = user;
  if (JSON.stringify(localProfileStore.getState().settings) !== JSON.stringify(profile.settings)) {
    localProfileStore.getState().updateSettings(profile.settings);
  }
}
