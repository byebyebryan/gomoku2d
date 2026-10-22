import {
  doc,
  getDocFromServer,
  runTransaction,
  serverTimestamp,
  type DocumentReference,
  type Firestore,
} from "firebase/firestore";

import {
  SHARING_SCHEMA_VERSION,
  SharingConflictError,
  SharingDataError,
  assertExpectedRevision,
  nextRevision,
  parsePrivateLibrarySnapshot,
  privateLibraryJson,
  resolveSharingFirestore,
  type PrivateLibraryEntry,
  type PrivateLibrarySnapshot,
  type SharingFirestoreOptions,
} from "./sharing_common.js";

export type { PrivateLibraryEntry, PrivateLibrarySnapshot } from "./sharing_common.js";

export function privateLibraryRef(firestore: Firestore, uid: string): DocumentReference {
  return doc(firestore, "replay_libraries", uid);
}

export function privateLibraryDocument(uid: string, revision: number, entriesJson: string) {
  return {
    schema_version: SHARING_SCHEMA_VERSION,
    uid,
    revision,
    updated_at: serverTimestamp(),
    entries_json: entriesJson,
  };
}

export async function loadPrivateLibrary(
  uid: string,
  options: SharingFirestoreOptions = {},
): Promise<PrivateLibrarySnapshot | null> {
  const firestore = resolveSharingFirestore(options);
  const snapshot = await getDocFromServer(privateLibraryRef(firestore, uid));
  return snapshot.exists() ? parsePrivateLibrarySnapshot(snapshot.data(), uid) : null;
}

/**
 * Replaces the complete private snapshot when its retained revision marker is
 * still the one the caller loaded. A null expected revision means no marker.
 */
export async function savePrivateLibrary(
  uid: string,
  entries: unknown,
  expectedRevision: number | null,
  options: SharingFirestoreOptions = {},
): Promise<PrivateLibrarySnapshot> {
  assertExpectedRevision(expectedRevision, "Expected private library revision");
  const entriesJson = privateLibraryJson(entries);
  const firestore = resolveSharingFirestore(options);
  const libraryRef = privateLibraryRef(firestore, uid);
  let result: PrivateLibrarySnapshot | null = null;

  await runTransaction(firestore, async (transaction) => {
    const snapshot = await transaction.get(libraryRef);
    if (!snapshot.exists()) {
      if (expectedRevision !== null) {
        throw new SharingConflictError("Private library marker no longer exists. Reload before saving.");
      }
      const revision = 1;
      transaction.set(libraryRef, privateLibraryDocument(uid, revision, entriesJson));
      result = { uid, revision, entries: JSON.parse(entriesJson) as PrivateLibraryEntry[], updatedAt: new Date().toISOString() };
      return;
    }

    const current = parsePrivateLibrarySnapshot(snapshot.data(), uid);
    if (expectedRevision === null || current.revision !== expectedRevision) {
      throw new SharingConflictError(`Private library revision changed from ${expectedRevision ?? "missing"} to ${current.revision}. Reload and review before saving.`);
    }
    const revision = nextRevision(current.revision, "Private library revision");
    transaction.set(libraryRef, privateLibraryDocument(uid, revision, entriesJson));
    result = { uid, revision, entries: JSON.parse(entriesJson) as PrivateLibraryEntry[], updatedAt: new Date().toISOString() };
  });

  if (!result) throw new SharingDataError("Private library update completed without a result.");
  return result;
}

