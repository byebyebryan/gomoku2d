import { doc, getDoc, setDoc, type Firestore } from "firebase/firestore";

import {
  DEFAULT_GUEST_DISPLAY_NAME,
  type GuestProfileIdentity,
  type GuestProfileSettings,
  type GuestSavedMatch,
} from "../profile/guest_profile_store";

import type { CloudAuthUser } from "./auth_store";
import {
  cloudMatchIdForGuestMatch,
  cloudSavedMatchFromGuestMatch,
  type CloudSavedMatchDocument,
} from "./cloud_match";
import { existingCloudProfileUpdate } from "./cloud_profile";
import { getFirebaseClients } from "./firebase";

export interface GuestPromotionInput {
  guestHistory: GuestSavedMatch[];
  guestProfile: GuestProfileIdentity;
  settings: GuestProfileSettings;
  user: CloudAuthUser;
}

export interface GuestPromotionResult {
  importedMatches: number;
  profileDisplayNamePromoted: boolean;
  promotedDisplayName: string | null;
  skippedMatches: number;
  totalMatches: number;
}

export interface CloudPromotionBackend {
  createMatch: (matchId: string, document: CloudSavedMatchDocument) => Promise<void>;
  matchExists: (matchId: string) => Promise<boolean>;
  updateProfile: (patch: Record<string, unknown>) => Promise<void>;
}

export interface PromoteGuestToCloudOptions {
  backend?: CloudPromotionBackend;
  firestore?: Firestore;
}

function customGuestDisplayName(profile: GuestProfileIdentity): string | null {
  const name = profile.displayName.trim();
  return name && name !== DEFAULT_GUEST_DISPLAY_NAME ? name : null;
}

export function cloudProfilePromotionUpdate(input: GuestPromotionInput): Record<string, unknown> {
  const update: Record<string, unknown> = {
    ...existingCloudProfileUpdate(input.user, input.settings.preferredVariant),
  };
  const displayName = customGuestDisplayName(input.guestProfile);

  if (displayName) {
    update.display_name = displayName;
  }

  return update;
}

function createFirestorePromotionBackend(user: CloudAuthUser, firestore: Firestore): CloudPromotionBackend {
  const profileRef = doc(firestore, "profiles", user.uid);

  return {
    createMatch: async (matchId, document) => {
      await setDoc(doc(profileRef, "matches", matchId), document);
    },
    matchExists: async (matchId) => {
      const snapshot = await getDoc(doc(profileRef, "matches", matchId));
      return snapshot.exists();
    },
    updateProfile: async (patch) => {
      await setDoc(profileRef, patch, { merge: true });
    },
  };
}

function resolvePromotionBackend(
  user: CloudAuthUser,
  options: PromoteGuestToCloudOptions,
): CloudPromotionBackend {
  if (options.backend) {
    return options.backend;
  }

  const firestore = options.firestore ?? getFirebaseClients()?.firestore;
  if (!firestore) {
    throw new Error("Cloud promotion is not configured for this build.");
  }

  return createFirestorePromotionBackend(user, firestore);
}

export function promotionInputKey(input: GuestPromotionInput): string {
  return JSON.stringify({
    history: input.guestHistory.map((match) => [match.id, match.saved_at, match.move_count]),
    profile: [input.guestProfile.id, input.guestProfile.displayName],
    rule: input.settings.preferredVariant,
    uid: input.user.uid,
  });
}

export async function promoteGuestToCloud(
  input: GuestPromotionInput,
  options: PromoteGuestToCloudOptions = {},
): Promise<GuestPromotionResult> {
  const backend = resolvePromotionBackend(input.user, options);
  const profileUpdate = cloudProfilePromotionUpdate(input);

  await backend.updateProfile(profileUpdate);

  let importedMatches = 0;
  let skippedMatches = 0;

  for (const match of input.guestHistory) {
    const matchId = cloudMatchIdForGuestMatch(match);
    if (await backend.matchExists(matchId)) {
      skippedMatches += 1;
      continue;
    }

    try {
      await backend.createMatch(matchId, cloudSavedMatchFromGuestMatch(input.user, input.guestProfile, match));
      importedMatches += 1;
    } catch (error) {
      if (await backend.matchExists(matchId)) {
        skippedMatches += 1;
        continue;
      }

      throw error;
    }
  }

  return {
    importedMatches,
    profileDisplayNamePromoted: Object.prototype.hasOwnProperty.call(profileUpdate, "display_name"),
    promotedDisplayName: typeof profileUpdate.display_name === "string" ? profileUpdate.display_name : null,
    skippedMatches,
    totalMatches: input.guestHistory.length,
  };
}
