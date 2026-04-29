import { doc, setDoc, type Firestore } from "firebase/firestore";

import { savedMatchIsAfterReset, type SavedMatchV1 } from "../match/saved_match";
import {
  DEFAULT_LOCAL_DISPLAY_NAME,
  type LocalProfileMatchHistory,
  type LocalProfileIdentity,
  type LocalProfileSettings,
} from "../profile/local_profile_store";

import type { CloudAuthUser } from "./auth_store";
import {
  cloudProfileNeedsSnapshotSync,
  cloudProfileSnapshotUpdate,
  cloudSettingsDocument,
  cloudSettingsForVariant,
  cloudMatchHistoryIsEmpty,
  existingCloudProfileUpdate,
  emptyCloudMatchHistory,
  mergeCloudArchivedMatchStats,
  mergeCloudMatchSummaryState,
  mergeCloudReplayMatches,
  type CloudMatchHistory,
  type CloudProfileSettings,
} from "./cloud_profile";
import { getFirebaseClients } from "./firebase";

export interface LocalProfilePromotionInput {
  /** Current display name on the cloud profile; undefined means not yet loaded. */
  cloudDisplayName?: string | null;
  cloudMatchHistory?: CloudMatchHistory | null;
  /** Current cloud settings; undefined means not yet loaded. */
  cloudSettings?: CloudProfileSettings | null;
  localMatchHistory: LocalProfileMatchHistory;
  localProfile: LocalProfileIdentity;
  resetAt?: string | null;
  settings: LocalProfileSettings;
  user: CloudAuthUser;
}

export interface LocalProfilePromotionResult {
  localMatchesSynced: number;
  profileDisplayNamePromoted: boolean;
  promotedDisplayName: string | null;
}

export interface CloudPromotionBackend {
  updateProfile: (patch: Record<string, unknown>) => Promise<void>;
}

export interface PromoteLocalProfileToCloudOptions {
  backend?: CloudPromotionBackend;
  firestore?: Firestore;
}

function customLocalDisplayName(profile: LocalProfileIdentity): string | null {
  const name = profile.displayName.trim();
  return name && name !== DEFAULT_LOCAL_DISPLAY_NAME ? name : null;
}

function cloudDisplayNameIsProviderDefault(cloudName: string | null | undefined, providerName: string): boolean {
  return cloudName === null || cloudName === providerName;
}

export function cloudProfilePromotionUpdate(input: LocalProfilePromotionInput): Record<string, unknown> | null {
  const baseUpdate = input.cloudSettings ? null : existingCloudProfileUpdate(input.user);
  const update: Record<string, unknown> = baseUpdate ? { ...baseUpdate } : {};
  const customLocal = customLocalDisplayName(input.localProfile);
  const nextSettings = cloudSettingsForVariant(input.settings.preferredVariant);

  if (!input.cloudSettings || JSON.stringify(input.cloudSettings) !== JSON.stringify(nextSettings)) {
    update.settings = cloudSettingsDocument(nextSettings);
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
  options: PromoteLocalProfileToCloudOptions,
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

export function promotionInputKey(input: LocalProfilePromotionInput): string {
  return JSON.stringify({
    cloud: [
      input.cloudDisplayName !== undefined,
      input.cloudDisplayName ?? null,
      input.cloudSettings !== undefined,
      input.cloudSettings ?? null,
    ],
    history: input.localMatchHistory,
    cloudHistory: input.cloudMatchHistory ?? null,
    reset: input.resetAt ?? null,
    profile: [input.localProfile.id, input.localProfile.displayName],
    rule: input.settings.preferredVariant,
    uid: input.user.uid,
  });
}

export async function promoteLocalProfileToCloud(
  input: LocalProfilePromotionInput,
  options: PromoteLocalProfileToCloudOptions = {},
): Promise<LocalProfilePromotionResult> {
  const backend = resolvePromotionBackend(input.user, options);
  const localReplayMatches = input.localMatchHistory.replayMatches;
  const eligibleHistory = localReplayMatches.filter((match) => savedMatchIsAfterReset(match, input.resetAt));
  const cloudMatchHistory = input.cloudMatchHistory ?? emptyCloudMatchHistory();
  const replayMatches = mergeCloudReplayMatches(
    input.user,
    [...cloudMatchHistory.replayMatches, ...eligibleHistory],
    input.resetAt,
  );
  const archivedStats = cloudMatchHistoryIsEmpty(cloudMatchHistory)
    ? mergeCloudArchivedMatchStats(cloudMatchHistory.archivedStats, input.localMatchHistory.archivedStats)
    : cloudMatchHistory.archivedStats;
  const summaryState = mergeCloudMatchSummaryState({
    archivedStats,
    matches: [...cloudMatchHistory.replayMatches, ...eligibleHistory],
    replayMatches,
    resetAt: input.resetAt,
    summaries: [
      ...cloudMatchHistory.summaryMatches,
      ...input.localMatchHistory.summaryMatches.map((summary) => ({
        ...summary,
        trust: "client_uploaded" as const,
      })),
    ],
    user: input.user,
  });
  const matchHistory: CloudMatchHistory = {
    archivedStats: summaryState.archivedStats,
    replayMatches,
    summaryMatches: summaryState.summaryMatches,
  };
  const profileUpdate = cloudProfilePromotionUpdate(input);
  const displayName = typeof profileUpdate?.display_name === "string"
    ? profileUpdate.display_name
    : input.cloudDisplayName ?? input.user.displayName;
  const nextSettings = cloudSettingsForVariant(input.settings.preferredVariant);
  const needsSnapshotSync = cloudProfileNeedsSnapshotSync({
    cloudDisplayName: input.cloudDisplayName ?? input.user.displayName,
    cloudMatchHistory,
    cloudSettings: input.cloudSettings ?? nextSettings,
    displayName,
    matchHistory,
    preferredVariant: input.settings.preferredVariant,
  });

  if (needsSnapshotSync || profileUpdate) {
    const snapshotUpdate = cloudProfileSnapshotUpdate({
      displayName,
      matchHistory,
      preferredVariant: input.settings.preferredVariant,
      user: input.user,
    });

    await backend.updateProfile(profileUpdate ? { ...profileUpdate, ...snapshotUpdate } : snapshotUpdate);
  }

  return {
    localMatchesSynced: eligibleHistory.length,
    profileDisplayNamePromoted: Boolean(
      profileUpdate && Object.prototype.hasOwnProperty.call(profileUpdate, "display_name"),
    ),
    promotedDisplayName: profileUpdate && typeof profileUpdate.display_name === "string"
      ? profileUpdate.display_name
      : null,
  };
}
