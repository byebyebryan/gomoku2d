import {
  doc,
  getDocFromServer,
  runTransaction,
  serverTimestamp,
  type DocumentReference,
  type Firestore,
} from "firebase/firestore";

import {
  SHARING_PAGE_IDS,
  SHARING_SCHEMA_VERSION,
  SharingConflictError,
  SharingDataError,
  assertExpectedRevision,
  assertPageId,
  nextRevision,
  parseCatalogPageSnapshot,
  parsePublicProfileSnapshot,
  resolveSharingFirestore,
  validateCatalogEntries,
  type PublicCatalogEntry,
  type PublicCatalogPageSnapshot,
  type PublicProfileSnapshot,
  type SharingFirestoreOptions,
  type SharingPageId,
} from "./sharing_common.js";

export type {
  PublicCatalogEntry,
  PublicCatalogPageSnapshot,
  PublicProfileSnapshot,
} from "./sharing_common.js";

function publicProfileRef(firestore: Firestore, uid: string): DocumentReference {
  return doc(firestore, "public_profiles", uid);
}

function catalogPageRef(firestore: Firestore, uid: string, pageId: SharingPageId): DocumentReference {
  return doc(firestore, "public_profiles", uid, "pages", String(pageId));
}

function publicName(value: string, enabled: boolean): string {
  if (!enabled) return "";
  const normalized = value.trim();
  const chars = Array.from(normalized).length;
  if (!normalized || chars > 80 || new TextEncoder().encode(normalized).length > 320
    || /[\r\n\u2028\u2029]/u.test(normalized)) {
    throw new SharingDataError("Public name must contain 1 to 80 characters and at most 320 UTF-8 bytes.");
  }
  return normalized;
}

function readProfile(data: unknown, uid: string): PublicProfileSnapshot {
  return parsePublicProfileSnapshot(data, uid);
}

function readPage(data: unknown, uid: string, pageId: SharingPageId): PublicCatalogPageSnapshot {
  return parseCatalogPageSnapshot(data, uid, pageId);
}

function profileDocument(uid: string, displayName: string, enabled: boolean, generation: number, revision: number) {
  return {
    schema_version: SHARING_SCHEMA_VERSION,
    uid,
    display_name: displayName,
    enabled,
    generation,
    revision,
    updated_at: serverTimestamp(),
  };
}

function pageDocument(generation: number, revision: number, entries: PublicCatalogEntry[]) {
  return {
    schema_version: SHARING_SCHEMA_VERSION,
    generation,
    revision,
    updated_at: serverTimestamp(),
    entries,
  };
}

function emptyPageDocument(generation: number, revision: number) {
  return pageDocument(generation, revision, []);
}

function timestampNow(): string {
  return new Date().toISOString();
}

export async function loadPublicProfile(
  uid: string,
  options: SharingFirestoreOptions = {},
): Promise<PublicProfileSnapshot | null> {
  const firestore = resolveSharingFirestore(options);
  const snapshot = await getDocFromServer(publicProfileRef(firestore, uid));
  return snapshot.exists() ? readProfile(snapshot.data(), uid) : null;
}

export async function loadCatalogPage(
  uid: string,
  pageId: number,
  options: SharingFirestoreOptions = {},
): Promise<PublicCatalogPageSnapshot | null> {
  assertPageId(pageId);
  const firestore = resolveSharingFirestore(options);
  const snapshot = await getDocFromServer(catalogPageRef(firestore, uid, pageId));
  return snapshot.exists() ? readPage(snapshot.data(), uid, pageId) : null;
}

/**
 * Creates or changes an account's public name and visibility. A null expected
 * revision means the public profile must not exist yet.
 */
export async function setPublicProfile(
  uid: string,
  displayName: string,
  enabled: boolean,
  expectedRootRevision: number | null,
  options: SharingFirestoreOptions = {},
): Promise<PublicProfileSnapshot> {
  assertExpectedRevision(expectedRootRevision, "Expected public profile revision");
  const normalizedName = publicName(displayName, enabled);
  const firestore = resolveSharingFirestore(options);
  const rootRef = publicProfileRef(firestore, uid);
  let result: PublicProfileSnapshot | null = null;

  await runTransaction(firestore, async (transaction) => {
    const rootSnapshot = await transaction.get(rootRef);
    if (!rootSnapshot.exists()) {
      if (expectedRootRevision !== null) {
        throw new SharingConflictError("Public profile no longer exists. Reload before saving.");
      }
      const generation = 1;
      const revision = 1;
      const pageSnapshots = await Promise.all(SHARING_PAGE_IDS.map((pageId) => (
        transaction.get(catalogPageRef(firestore, uid, Number(pageId) as SharingPageId))
      )));
      if (pageSnapshots.some((snapshot) => snapshot.exists())) {
        throw new SharingDataError("Public profile markers are incomplete. Reload before changing visibility.");
      }
      transaction.set(rootRef, profileDocument(uid, normalizedName, enabled, generation, revision));
      SHARING_PAGE_IDS.forEach((pageId) => {
        transaction.set(catalogPageRef(firestore, uid, Number(pageId) as SharingPageId), emptyPageDocument(generation, 1));
      });
      result = { uid, displayName: normalizedName, enabled, generation, revision, updatedAt: timestampNow() };
      return;
    }

    const current = readProfile(rootSnapshot.data(), uid);
    if (expectedRootRevision === null || current.revision !== expectedRootRevision) {
      throw new SharingConflictError(`Public profile revision changed from ${expectedRootRevision ?? "missing"} to ${current.revision}. Reload before saving.`);
    }

    const revision = nextRevision(current.revision, "Public profile revision");
    let generation = current.generation;
    if (current.enabled !== enabled) generation = nextRevision(current.generation, "Public profile generation");
    const changedVisibility = current.enabled !== enabled;

    if (changedVisibility) {
      const pageSnapshots = await Promise.all(SHARING_PAGE_IDS.map((pageId) => (
        transaction.get(catalogPageRef(firestore, uid, Number(pageId) as SharingPageId))
      )));
      const pages = pageSnapshots.map((snapshot, index) => {
        if (!snapshot.exists()) throw new SharingDataError("A public catalog marker is missing. Reload before changing visibility.");
        const page = readPage(snapshot.data(), uid, index as SharingPageId);
        if (page.generation !== current.generation) {
          throw new SharingDataError("Public catalog generations do not match. Reload before changing visibility.");
        }
        return page;
      });
      pages.forEach((page) => {
        transaction.set(
          catalogPageRef(firestore, uid, page.pageId),
          emptyPageDocument(generation, nextRevision(page.revision, "Catalog page revision")),
        );
      });
    }

    transaction.set(rootRef, profileDocument(uid, normalizedName, enabled, generation, revision));
    result = { uid, displayName: normalizedName, enabled, generation, revision, updatedAt: timestampNow() };
  });

  if (!result) throw new SharingDataError("Public profile update completed without a result.");
  return result;
}

/** Writes one fixed catalog page only if both its generation and revision remain current. */
export async function saveCatalogPage(
  uid: string,
  pageId: number,
  entries: unknown,
  expectedGeneration: number,
  expectedPageRevision: number,
  options: SharingFirestoreOptions = {},
): Promise<PublicCatalogPageSnapshot> {
  assertPageId(pageId);
  assertExpectedRevision(expectedGeneration, "Expected public profile generation");
  assertExpectedRevision(expectedPageRevision, "Expected catalog page revision");
  const normalizedEntries = validateCatalogEntries(entries);
  const firestore = resolveSharingFirestore(options);
  const rootRef = publicProfileRef(firestore, uid);
  const pageRef = catalogPageRef(firestore, uid, pageId);
  let result: PublicCatalogPageSnapshot | null = null;

  await runTransaction(firestore, async (transaction) => {
    const rootSnapshot = await transaction.get(rootRef);
    const pageSnapshot = await transaction.get(pageRef);
    if (!rootSnapshot.exists() || !pageSnapshot.exists()) {
      throw new SharingConflictError("Public profile or catalog page is missing. Reload before publishing.");
    }
    const profile = readProfile(rootSnapshot.data(), uid);
    const page = readPage(pageSnapshot.data(), uid, pageId);
    if (!profile.enabled) throw new SharingDataError("Enable the public profile before publishing games.");
    if (profile.generation !== expectedGeneration || page.generation !== expectedGeneration
      || page.revision !== expectedPageRevision) {
      throw new SharingConflictError("Public catalog changed on another device. Reload and review before publishing.");
    }

    const revision = nextRevision(page.revision, "Catalog page revision");
    transaction.set(pageRef, pageDocument(profile.generation, revision, normalizedEntries));
    result = {
      uid,
      pageId,
      generation: profile.generation,
      revision,
      entries: normalizedEntries,
      updatedAt: timestampNow(),
    };
  });

  if (!result) throw new SharingDataError("Catalog page update completed without a result.");
  return result;
}

