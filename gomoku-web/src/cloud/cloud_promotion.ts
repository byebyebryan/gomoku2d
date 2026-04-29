import { doc, getDoc, setDoc, type Firestore } from "firebase/firestore";

import { savedMatchIsAfterReset } from "../match/saved_match";
import {
  DEFAULT_GUEST_DISPLAY_NAME,
  type GuestProfileIdentity,
  type GuestProfileSettings,
  type GuestSavedMatch,
} from "../profile/guest_profile_store";

import type { CloudAuthUser } from "./auth_store";
import {
  cloudDirectSavedMatchId,
  cloudMatchIdForGuestMatch,
  cloudSavedMatchFromGuestMatch,
  type CloudGuestImportDocument,
} from "./cloud_match";
import {
  CLOUD_PROFILE_SCHEMA_VERSION,
  existingCloudProfileUpdate,
  type CloudProfileDocument,
} from "./cloud_profile";
import { getFirebaseClients } from "./firebase";

export interface GuestPromotionInput {
  /** Current display name on the cloud profile; undefined means not yet loaded. */
  cloudDisplayName?: string | null;
  /** Current preferred rule on the cloud profile; undefined means not yet loaded. */
  cloudPreferredVariant?: GuestProfileSettings["preferredVariant"] | null;
  guestHistory: GuestSavedMatch[];
  guestProfile: GuestProfileIdentity;
  historyResetAt?: string | null;
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

function currentCloudProfileDocument(input: GuestPromotionInput): CloudProfileDocument | null {
  if (!input.cloudPreferredVariant) {
    return null;
  }

  return {
    auth_providers: input.user.providerIds,
    avatar_url: input.user.avatarUrl,
    email: input.user.email,
    preferred_variant: input.cloudPreferredVariant,
    schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
    uid: input.user.uid,
  };
}

export function cloudProfilePromotionUpdate(input: GuestPromotionInput): Record<string, unknown> | null {
  const cloudDocument = currentCloudProfileDocument(input);
  const baseUpdate = cloudDocument
    ? existingCloudProfileUpdate(input.user, cloudDocument)
    : existingCloudProfileUpdate(input.user);
  const update: Record<string, unknown> = baseUpdate ? { ...baseUpdate } : {};
  const customLocal = customGuestDisplayName(input.guestProfile);

  if (input.cloudPreferredVariant !== input.settings.preferredVariant) {
    update.preferred_variant = input.settings.preferredVariant;
  }

  // Only promote the local custom name if the cloud name hasn't been customized
  // by another device (i.e., still holds the provider default or hasn't been set).
  if (customLocal && cloudDisplayNameIsProviderDefault(input.cloudDisplayName, input.user.displayName)) {
    update.display_name = customLocal;
  }

  return Object.keys(update).length > 0 ? update : null;
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
    cloud: [
      input.cloudDisplayName !== undefined,
      input.cloudDisplayName ?? null,
      input.cloudPreferredVariant !== undefined,
      input.cloudPreferredVariant ?? null,
    ],
    history: input.guestHistory.map((match) => [match.id, match.saved_at, match.move_count]),
    reset: input.historyResetAt ?? null,
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

  if (profileUpdate) {
    await backend.updateProfile(profileUpdate);
  }

  let importedMatches = 0;
  let skippedMatches = 0;

  const eligibleHistory = input.guestHistory.filter((match) => savedMatchIsAfterReset(match, input.historyResetAt));

  for (const match of eligibleHistory) {
    const matchId = cloudMatchIdForGuestMatch(match);
    const directMatchId = cloudDirectSavedMatchId(match);
    if (await backend.matchExists(matchId) || await backend.matchExists(directMatchId)) {
      skippedMatches += 1;
      continue;
    }

    try {
      await backend.createMatch(matchId, cloudSavedMatchFromGuestMatch(input.user, input.guestProfile, match));
      importedMatches += 1;
    } catch (error) {
      if (await backend.matchExists(matchId) || await backend.matchExists(directMatchId)) {
        skippedMatches += 1;
        continue;
      }

      throw error;
    }
  }

  return {
    importedMatches,
    profileDisplayNamePromoted: Boolean(
      profileUpdate && Object.prototype.hasOwnProperty.call(profileUpdate, "display_name"),
    ),
    promotedDisplayName: profileUpdate && typeof profileUpdate.display_name === "string"
      ? profileUpdate.display_name
      : null,
    skippedMatches,
    totalMatches: eligibleHistory.length,
  };
}
