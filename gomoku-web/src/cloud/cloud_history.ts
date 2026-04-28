import {
  collection,
  doc,
  getDoc,
  getDocs,
  limit,
  orderBy,
  query,
  setDoc,
  type Firestore,
} from "firebase/firestore";

import { isSavedMatchV1, type SavedMatchV1 } from "../match/saved_match";

import type { CloudAuthUser } from "./auth_store";
import {
  cloudDirectSavedMatchId,
  createCloudDirectSavedDocument,
  createCloudDirectSavedMatch,
  type CloudDirectSavedDocument,
} from "./cloud_match";
import { getFirebaseClients } from "./firebase";

export const CLOUD_HISTORY_LIMIT = 24;

export interface CloudHistoryBackend {
  createMatch: (matchId: string, document: CloudDirectSavedDocument) => Promise<void>;
  loadMatches: (limitCount: number) => Promise<unknown[]>;
  matchExists: (matchId: string) => Promise<boolean>;
}

export interface CloudHistoryOptions {
  backend?: CloudHistoryBackend;
  firestore?: Firestore;
}

export interface CloudSaveMatchResult {
  match: SavedMatchV1;
  matchId: string;
  skipped: boolean;
}

function createFirestoreCloudHistoryBackend(user: CloudAuthUser, firestore: Firestore): CloudHistoryBackend {
  const profileRef = doc(firestore, "profiles", user.uid);
  const matchesRef = collection(profileRef, "matches");

  return {
    createMatch: async (matchId, document) => {
      await setDoc(doc(matchesRef, matchId), document);
    },
    loadMatches: async (limitCount) => {
      const snapshot = await getDocs(query(matchesRef, orderBy("saved_at", "desc"), limit(limitCount)));
      return snapshot.docs.map((entry) => entry.data());
    },
    matchExists: async (matchId) => {
      const snapshot = await getDoc(doc(matchesRef, matchId));
      return snapshot.exists();
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

export async function saveCloudMatch(
  user: CloudAuthUser,
  match: SavedMatchV1,
  options: CloudHistoryOptions = {},
): Promise<CloudSaveMatchResult> {
  const backend = resolveCloudHistoryBackend(user, options);
  const matchId = cloudDirectSavedMatchId(match);
  const cloudMatch = createCloudDirectSavedMatch(user, match);

  if (await backend.matchExists(matchId)) {
    return { match: cloudMatch, matchId, skipped: true };
  }

  try {
    await backend.createMatch(matchId, createCloudDirectSavedDocument(user, match));
    return { match: cloudMatch, matchId, skipped: false };
  } catch (error) {
    if (await backend.matchExists(matchId)) {
      return { match: cloudMatch, matchId, skipped: true };
    }

    throw error;
  }
}

export async function loadCloudHistory(
  user: CloudAuthUser,
  options: CloudHistoryOptions & { limitCount?: number } = {},
): Promise<SavedMatchV1[]> {
  const backend = resolveCloudHistoryBackend(user, options);
  const documents = await backend.loadMatches(options.limitCount ?? CLOUD_HISTORY_LIMIT);

  return documents.filter(isSavedMatchV1);
}
