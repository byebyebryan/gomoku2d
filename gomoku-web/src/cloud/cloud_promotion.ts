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
  type CloudGuestImportDocument,
} from "./cloud_match";
import { existingCloudProfileUpdate } from "./cloud_profile";
import { getFirebaseClients } from "./firebase";

export interface GuestPromotionInput {
  /** Current display name on the cloud profile; undefined means not yet loaded. */
  cloudDisplayName?: string | null;
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
  createMatch: (matchId: string, document: CloudGuestImportDocument) => Promise<void>;
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

function cloudDisplayNameIsProviderDefault(cloudName: string | null | undefined, providerName: string): boolean {
  return cloudName === null || cloudName === providerName;
}

export function cloudProfilePromotionUpdate(input: GuestPromotionInput): Record<string, unknown> {
  const update: Record<string, unknown> = {
    ...existingCloudProfileUpdate(input.user, input.settings.preferredVariant),
  };
  const customLocal = customGuestDisplayName(input.guestProfile);

  // Only promote the local custom name if the cloud name hasn't been customized
  // by another device (i.e., still holds the provider default or hasn't been set).
  if (customLocal && cloudDisplayNameIsProviderDefault(input.cloudDisplayName, input.user.displayName)) {
    update.display_name = customLocal;
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
    cloud: [input.cloudDisplayName !== undefined, input.cloudDisplayName ?? null],
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
