import {
  doc,
  runTransaction,
  serverTimestamp,
} from "firebase/firestore";

import {
  SHARING_PAGE_IDS,
  SHARING_SCHEMA_VERSION,
  validateCatalogPageCounts,
  SharingConflictError,
  SharingDataError,
  nextRevision,
  parseCatalogPageSnapshot,
  parsePrivateLibrarySnapshot,
  parsePublicProfileSnapshot,
  privateLibraryJson,
  resolveSharingFirestore,
  type PublicCatalogPageSnapshot,
  type PublicProfileSnapshot,
  type PrivateLibrarySnapshot,
  type SharingFirestoreOptions,
  type SharingPageId,
} from "./sharing_common.js";
import { privateLibraryDocument, privateLibraryRef } from "./sharing_private.js";

export interface ClearSharingDataResult {
  publicProfile: PublicProfileSnapshot;
  pages: PublicCatalogPageSnapshot[];
  privateLibrary: PrivateLibrarySnapshot;
}

function snapshotToken(snapshot: { exists: () => boolean; data: () => unknown }): string {
  return snapshot.exists() ? JSON.stringify(snapshot.data()) : "missing";
}

/**
 * Clears public and private replay data in one transaction while retaining
 * empty revision markers. Transaction retries abort if any initially observed
 * document changed, so a retry cannot erase a newer write.
 */
export async function clearSharingData(
  uid: string,
  options: SharingFirestoreOptions = {},
): Promise<ClearSharingDataResult> {
  const firestore = resolveSharingFirestore(options);
  const rootRef = doc(firestore, "public_profiles", uid);
  const pageRefs = SHARING_PAGE_IDS.map((pageId) => doc(firestore, "public_profiles", uid, "pages", pageId));
  const libraryRef = privateLibraryRef(firestore, uid);
  const refs = [rootRef, ...pageRefs, libraryRef];
  const emptyPrivateJson = privateLibraryJson([]);
  let initialReadSet: string[] | null = null;
  let result: ClearSharingDataResult | null = null;

  await runTransaction(firestore, async (transaction) => {
    const snapshots = [];
    for (const ref of refs) snapshots.push(await transaction.get(ref));
    const readSet = snapshots.map(snapshotToken);
    if (initialReadSet === null) initialReadSet = readSet;
    else if (readSet.some((token, index) => token !== initialReadSet?.[index])) {
      throw new SharingConflictError("Sharing data changed while clearing. Reload and review before trying again.");
    }

    const [rootSnapshot, ...remainingSnapshots] = snapshots;
    const pageSnapshots = remainingSnapshots.slice(0, SHARING_PAGE_IDS.length);
    const librarySnapshot = remainingSnapshots[SHARING_PAGE_IDS.length];
    const current = rootSnapshot.exists() ? parsePublicProfileSnapshot(rootSnapshot.data(), uid) : null;

    const pages = pageSnapshots.map((snapshot, index) => {
      if (!snapshot.exists()) {
        if (current) throw new SharingDataError("A public catalog marker is missing. Sharing data could not be cleared safely.");
        return null;
      }
      if (!current) throw new SharingDataError("Public catalog markers exist without a profile. Sharing data could not be cleared safely.");
      const page = parseCatalogPageSnapshot(snapshot.data(), uid, index as SharingPageId);
      if (page.generation !== current.generation || (!current.enabled && page.entries.length > 0)) {
        throw new SharingDataError("Public catalog markers do not match the profile. Sharing data could not be cleared safely.");
      }
      return page;
    });
    if (current?.catalogPageCounts !== null && current?.catalogPageCounts !== undefined
      && pages.some((page, index) => page?.entries.length !== current.catalogPageCounts?.[index])) {
      throw new SharingDataError("Public catalog counts do not match the pages. Sharing data could not be cleared safely.");
    }
    const currentLibrary = librarySnapshot.exists()
      ? parsePrivateLibrarySnapshot(librarySnapshot.data(), uid)
      : null;

    const generation = current
      ? nextRevision(current.generation, "Public profile generation")
      : 1;
    const rootRevision = current
      ? nextRevision(current.revision, "Public profile revision")
      : 1;
    const publicProfile = {
      uid,
      displayName: "",
      enabled: false,
      generation,
      revision: rootRevision,
      updatedAt: new Date().toISOString(),
      sharedStats: null,
      catalogPageCounts: validateCatalogPageCounts([0, 0, 0, 0]),
    };
    const clearedPages = pages.map((page, index) => ({
      uid,
      pageId: index as SharingPageId,
      generation,
      revision: page ? nextRevision(page.revision, "Catalog page revision") : 1,
      entries: [],
      updatedAt: publicProfile.updatedAt,
    }));
    const libraryRevision = currentLibrary
      ? nextRevision(currentLibrary.revision, "Private library revision")
      : 1;
    const privateLibrary = {
      uid,
      revision: libraryRevision,
      entries: [],
      updatedAt: publicProfile.updatedAt,
    };

    transaction.set(rootRef, {
      schema_version: SHARING_SCHEMA_VERSION,
      uid,
      display_name: "",
      enabled: false,
      generation,
      revision: rootRevision,
      updated_at: serverTimestamp(),
      shared_stats: null,
      catalog_page_counts: validateCatalogPageCounts([0, 0, 0, 0]),
    });
    pageRefs.forEach((pageRef, index) => {
      transaction.set(pageRef, {
        schema_version: SHARING_SCHEMA_VERSION,
        generation,
        revision: clearedPages[index].revision,
        updated_at: serverTimestamp(),
        entries: [],
      });
    });
    transaction.set(libraryRef, privateLibraryDocument(uid, libraryRevision, emptyPrivateJson));
    result = { publicProfile, pages: clearedPages, privateLibrary };
  });

  if (!result) throw new SharingDataError("Sharing data clear completed without a result.");
  return result;
}
