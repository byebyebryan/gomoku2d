import { doc, setDoc, type Firestore } from "firebase/firestore";

import { savedMatchIsAfterReset, type SavedMatchV1 } from "../match/saved_match";
import {
  DEFAULT_GUEST_DISPLAY_NAME,
  type GuestProfileIdentity,
  type GuestProfileSettings,
  type GuestSavedMatch,
} from "../profile/guest_profile_store";

import type { CloudAuthUser } from "./auth_store";
import {
  CLOUD_PROFILE_SCHEMA_VERSION,
  cloudProfileNeedsSnapshotSync,
  cloudProfileSnapshotUpdate,
  existingCloudProfileUpdate,
  mergeCloudRecentMatches,
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
  cloudHistory?: SavedMatchV1[];
}

export interface GuestPromotionResult {
  importedMatches: number;
  profileDisplayNamePromoted: boolean;
  promotedDisplayName: string | null;
  skippedMatches: number;
  totalMatches: number;
}

export interface CloudPromotionBackend {
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
    recent_matches: {
      matches: input.cloudHistory ?? [],
      schema_version: 1,
      updated_at: null,
    },
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
    cloudHistory: (input.cloudHistory ?? []).map((match) => [match.id, match.saved_at, match.move_count]),
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
  const eligibleHistory = input.guestHistory.filter((match) => savedMatchIsAfterReset(match, input.historyResetAt));
  const recentMatches = mergeCloudRecentMatches(
    input.user,
    [...(input.cloudHistory ?? []), ...eligibleHistory],
    input.historyResetAt,
  );
  const profileUpdate = cloudProfilePromotionUpdate(input);
  const displayName = typeof profileUpdate?.display_name === "string"
    ? profileUpdate.display_name
    : input.cloudDisplayName ?? input.user.displayName;
  const preferredVariant = profileUpdate?.preferred_variant === "freestyle" || profileUpdate?.preferred_variant === "renju"
    ? profileUpdate.preferred_variant
    : input.cloudPreferredVariant ?? input.settings.preferredVariant;
  const needsSnapshotSync = cloudProfileNeedsSnapshotSync({
    cloudDisplayName: input.cloudDisplayName ?? input.user.displayName,
    cloudPreferredVariant: input.cloudPreferredVariant ?? input.settings.preferredVariant,
    cloudRecentMatches: input.cloudHistory ?? [],
    displayName,
    preferredVariant,
    recentMatches,
  });

  if (needsSnapshotSync || profileUpdate) {
    await backend.updateProfile(cloudProfileSnapshotUpdate({
      displayName,
      preferredVariant,
      recentMatches,
      user: input.user,
    }));
  }

  return {
    importedMatches: eligibleHistory.length,
    profileDisplayNamePromoted: Boolean(
      profileUpdate && Object.prototype.hasOwnProperty.call(profileUpdate, "display_name"),
    ),
    promotedDisplayName: profileUpdate && typeof profileUpdate.display_name === "string"
      ? profileUpdate.display_name
      : null,
    skippedMatches: 0,
    totalMatches: eligibleHistory.length,
  };
}
