import {
  doc,
  getDoc,
  setDoc,
  type Firestore,
} from "firebase/firestore";

import type { GameVariant } from "../core/bot_protocol";
import { savedMatchIsAfterReset, type SavedMatchV1 } from "../match/saved_match";

import type { CloudAuthUser } from "./auth_store";
import {
  CLOUD_REPLAY_MATCHES_LIMIT,
  cloudProfileFromDocument,
  cloudProfileSnapshotUpdate,
  cloudSettingsForVariant,
  mergeCloudMatchSummaryState,
  mergeCloudReplayMatches,
  type CloudMatchHistory,
  type CloudProfile,
  type CloudProfileDocument,
} from "./cloud_profile";
import { getFirebaseClients } from "./firebase";

export const CLOUD_HISTORY_LIMIT = CLOUD_REPLAY_MATCHES_LIMIT;

export interface CloudHistoryBackend {
  loadProfile: () => Promise<CloudProfileDocument | null>;
  updateProfile: (patch: Record<string, unknown>) => Promise<void>;
}

export interface CloudHistoryOptions {
  backend?: CloudHistoryBackend;
  firestore?: Firestore;
}

export interface CloudSaveHistoryResult {
  matches: SavedMatchV1[];
  profile: CloudProfile;
}

function createFirestoreCloudHistoryBackend(user: CloudAuthUser, firestore: Firestore): CloudHistoryBackend {
  const profileRef = doc(firestore, "profiles", user.uid);

  return {
    loadProfile: async () => {
      const snapshot = await getDoc(profileRef);
      return snapshot.exists() ? (snapshot.data() as CloudProfileDocument) : null;
    },
    updateProfile: async (patch) => {
      await setDoc(profileRef, patch, { merge: true });
    },
  };
}

function resolveCloudHistoryBackend(user: CloudAuthUser, options: CloudHistoryOptions): CloudHistoryBackend {
  if (options.backend) {
    return options.backend;
  }

  const firestore = options.firestore ?? getFirebaseClients()?.firestore;
  if (!firestore) {
    throw new Error("Cloud history is not configured for this build.");
  }

  return createFirestoreCloudHistoryBackend(user, firestore);
}

export function cloudHistoryFromProfile(
  profile: Pick<CloudProfile, "matchHistory" | "resetAt">,
  historyResetAt: string | null | undefined = profile.resetAt,
): SavedMatchV1[] {
  return profile.matchHistory.replayMatches.filter((match) => savedMatchIsAfterReset(match, historyResetAt));
}

export async function loadCloudHistory(
  user: CloudAuthUser,
  options: CloudHistoryOptions & { historyResetAt?: string | null } = {},
): Promise<SavedMatchV1[]> {
  const backend = resolveCloudHistoryBackend(user, options);
  const document = await backend.loadProfile();
  const profile = cloudProfileFromDocument(user, "freestyle", document);
  return cloudHistoryFromProfile(profile, options.historyResetAt);
}

export async function saveCloudHistorySnapshot(
  user: CloudAuthUser,
  input: {
    cloudProfile: CloudProfile;
    displayName: string;
    matches: SavedMatchV1[];
    preferredVariant: GameVariant;
  },
  options: CloudHistoryOptions = {},
): Promise<CloudSaveHistoryResult> {
  const backend = resolveCloudHistoryBackend(user, options);
  const replayMatches = mergeCloudReplayMatches(user, input.matches, input.cloudProfile.resetAt);
  const summaryState = mergeCloudMatchSummaryState({
    archivedStats: input.cloudProfile.matchHistory.archivedStats,
    matches: input.matches,
    replayMatches,
    resetAt: input.cloudProfile.resetAt,
    summaries: input.cloudProfile.matchHistory.summaryMatches,
    user,
  });
  const matchHistory: CloudMatchHistory = {
    archivedStats: summaryState.archivedStats,
    replayMatches,
    summaryMatches: summaryState.summaryMatches,
  };
  const patch = cloudProfileSnapshotUpdate({
    displayName: input.displayName,
    matchHistory,
    preferredVariant: input.preferredVariant,
    user,
  });

  await backend.updateProfile(patch);
  const syncedAt = new Date().toISOString();

  return {
    matches: replayMatches,
    profile: {
      ...input.cloudProfile,
      displayName: input.displayName,
      matchHistory,
      settings: cloudSettingsForVariant(input.preferredVariant),
      updatedAt: syncedAt,
    },
  };
}
