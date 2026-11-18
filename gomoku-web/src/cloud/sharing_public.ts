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
  validateCatalogPageCounts,
  validateCatalogEntries,
  validatePublicProfileStatsCounters,
  type PublicCatalogEntry,
  type PublicCatalogPageCounts,
  type PublicCatalogPageSnapshot,
  type PublicProfileSnapshot,
  type PublicProfileStatsCounters,
  type SaveCatalogPageResult,
  type SharingFirestoreOptions,
  type SharingPageId,
} from "./sharing_common.js";
import { decodePortableReplay, encodePortableReplay, portableReplayFingerprint } from "../replay/portable_replay_codec.js";

export type {
  PublicCatalogEntry,
  PublicCatalogPageCounts,
  PublicCatalogPageSnapshot,
  PublicProfileSnapshot,
  PublicProfileStatsCounters,
  PublicProfileStatsSnapshot,
  SaveCatalogPageResult,
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

function publicCatalogTitle(value: string): string {
  const normalized = value.trim();
  const chars = Array.from(normalized).length;
  if (!normalized || chars > 80 || new TextEncoder().encode(normalized).length > 320
    || /[\r\n\u2028\u2029]/u.test(normalized)) {
    throw new SharingDataError("Published replay title must contain 1 to 80 characters and at most 320 UTF-8 bytes.");
  }
  return normalized;
}

function readProfile(data: unknown, uid: string): PublicProfileSnapshot {
  return parsePublicProfileSnapshot(data, uid);
}

function readPage(data: unknown, uid: string, pageId: SharingPageId): PublicCatalogPageSnapshot {
  return parseCatalogPageSnapshot(data, uid, pageId);
}

const EMPTY_CATALOG_PAGE_COUNTS: PublicCatalogPageCounts = [0, 0, 0, 0];

function newProfileDocument(uid: string, displayName: string, enabled: boolean, generation: number, revision: number) {
  return {
    schema_version: SHARING_SCHEMA_VERSION,
    uid,
    display_name: displayName,
    enabled,
    generation,
    revision,
    updated_at: serverTimestamp(),
    shared_stats: null,
    catalog_page_counts: EMPTY_CATALOG_PAGE_COUNTS,
  };
}

function updatedProfileDocument(
  current: Record<string, unknown>,
  uid: string,
  displayName: string,
  enabled: boolean,
  generation: number,
  revision: number,
  overrides: { sharedStats?: unknown; catalogPageCounts?: unknown } = {},
) {
  const document: Record<string, unknown> = {
    ...current,
    schema_version: SHARING_SCHEMA_VERSION,
    uid,
    display_name: displayName,
    enabled,
    generation,
    revision,
    updated_at: serverTimestamp(),
  };
  if (Object.prototype.hasOwnProperty.call(overrides, "sharedStats")) document.shared_stats = overrides.sharedStats;
  if (Object.prototype.hasOwnProperty.call(overrides, "catalogPageCounts")) document.catalog_page_counts = overrides.catalogPageCounts;
  return document;
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
      transaction.set(rootRef, newProfileDocument(uid, normalizedName, enabled, generation, revision));
      SHARING_PAGE_IDS.forEach((pageId) => {
        transaction.set(catalogPageRef(firestore, uid, Number(pageId) as SharingPageId), emptyPageDocument(generation, 1));
      });
      result = {
        uid, displayName: normalizedName, enabled, generation, revision, updatedAt: timestampNow(),
        sharedStats: null, catalogPageCounts: [...EMPTY_CATALOG_PAGE_COUNTS],
      };
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

    transaction.set(rootRef, updatedProfileDocument(
      rootSnapshot.data()!,
      uid,
      normalizedName,
      enabled,
      generation,
      revision,
      changedVisibility
        ? { sharedStats: null, catalogPageCounts: EMPTY_CATALOG_PAGE_COUNTS }
        : {},
    ));
    result = {
      uid,
      displayName: normalizedName,
      enabled,
      generation,
      revision,
      updatedAt: timestampNow(),
      sharedStats: changedVisibility ? null : current.sharedStats,
      catalogPageCounts: changedVisibility ? [...EMPTY_CATALOG_PAGE_COUNTS] : current.catalogPageCounts,
    };
  });

  if (!result) throw new SharingDataError("Public profile update completed without a result.");
  return result;
}

/**
 * Atomically writes one fixed catalog page and its root manifest only while
 * the loaded root, generation, and page revisions remain current.
 */
export async function saveCatalogPage(
  uid: string,
  pageId: number,
  entries: unknown,
  expectedRootRevision: number,
  expectedGeneration: number,
  expectedPageRevision: number,
  options: SharingFirestoreOptions = {},
): Promise<SaveCatalogPageResult> {
  assertPageId(pageId);
  assertExpectedRevision(expectedRootRevision, "Expected public profile revision");
  assertExpectedRevision(expectedGeneration, "Expected public profile generation");
  assertExpectedRevision(expectedPageRevision, "Expected catalog page revision");
  const normalizedEntries = validateCatalogEntries(entries);
  const firestore = resolveSharingFirestore(options);
  const rootRef = publicProfileRef(firestore, uid);
  const pageRef = catalogPageRef(firestore, uid, pageId);
  let result: SaveCatalogPageResult | null = null;

  await runTransaction(firestore, async (transaction) => {
    const rootSnapshot = await transaction.get(rootRef);
    if (!rootSnapshot.exists()) throw new SharingConflictError("Public profile is missing. Reload before publishing.");
    const profile = readProfile(rootSnapshot.data(), uid);
    if (profile.revision !== expectedRootRevision) {
      throw new SharingConflictError(`Public profile revision changed from ${expectedRootRevision} to ${profile.revision}. Reload before publishing.`);
    }
    if (!profile.enabled) throw new SharingDataError("Enable the public profile before publishing games.");
    if (profile.generation !== expectedGeneration) {
      throw new SharingConflictError("Public profile generation changed. Reload and review before publishing.");
    }

    let page: PublicCatalogPageSnapshot;
    let catalogPageCounts: PublicCatalogPageCounts;
    if (profile.catalogPageCounts === null) {
      const pageSnapshots = await Promise.all(SHARING_PAGE_IDS.map((candidatePageId) => (
        transaction.get(catalogPageRef(firestore, uid, Number(candidatePageId) as SharingPageId))
      )));
      const pages = pageSnapshots.map((snapshot, index) => {
        if (!snapshot.exists()) throw new SharingDataError("A legacy public catalog marker is missing. Reload before publishing.");
        const loadedPage = readPage(snapshot.data(), uid, index as SharingPageId);
        if (loadedPage.generation !== profile.generation) {
          throw new SharingDataError("Legacy public catalog generations do not match. Reload before publishing.");
        }
        return loadedPage;
      });
      page = pages[pageId];
      catalogPageCounts = validateCatalogPageCounts(pages.map((candidate) => candidate.entries.length));
    } else {
      const pageSnapshot = await transaction.get(pageRef);
      if (!pageSnapshot.exists()) throw new SharingConflictError("Public catalog page is missing. Reload before publishing.");
      page = readPage(pageSnapshot.data(), uid, pageId);
      if (page.entries.length !== profile.catalogPageCounts[pageId]) {
        throw new SharingDataError("Public catalog counts do not match this page. Reload before publishing.");
      }
      catalogPageCounts = [...profile.catalogPageCounts] as PublicCatalogPageCounts;
    }

    if (page.generation !== expectedGeneration || page.revision !== expectedPageRevision) {
      throw new SharingConflictError("Public catalog changed on another device. Reload and review before publishing.");
    }

    const revision = nextRevision(page.revision, "Catalog page revision");
    const rootRevision = nextRevision(profile.revision, "Public profile revision");
    catalogPageCounts[pageId] = normalizedEntries.length;
    transaction.set(pageRef, pageDocument(profile.generation, revision, normalizedEntries));
    transaction.set(rootRef, updatedProfileDocument(
      rootSnapshot.data()!,
      uid,
      profile.displayName,
      profile.enabled,
      profile.generation,
      rootRevision,
      { catalogPageCounts },
    ));
    const updatedAt = timestampNow();
    result = {
      profile: { ...profile, revision: rootRevision, catalogPageCounts, updatedAt },
      page: { uid, pageId, generation: profile.generation, revision, entries: normalizedEntries, updatedAt },
    };
  });

  if (!result) throw new SharingDataError("Catalog page update completed without a result.");
  return result;
}

/**
 * Changes the title and/or public opening of an existing catalog entry.
 * The fixed page and root manifest are revision-bound so a stale edit cannot
 * recreate a removed publication or cross a visibility generation change.
 */
export async function editPublishedReplay(
  uid: string,
  pageId: number,
  fingerprint: string,
  details: { title: string; opening_ply: number | null },
  expectedRootRevision: number,
  expectedGeneration: number,
  expectedPageRevision: number,
  options: SharingFirestoreOptions = {},
  isCurrentSession?: () => boolean,
): Promise<SaveCatalogPageResult> {
  assertPageId(pageId);
  assertExpectedRevision(expectedRootRevision, "Expected public profile revision");
  assertExpectedRevision(expectedGeneration, "Expected public profile generation");
  assertExpectedRevision(expectedPageRevision, "Expected catalog page revision");

  if (!details || typeof details.title !== "string") throw new SharingDataError("Published replay title must be text.");
  const normalizedTitle = publicCatalogTitle(details.title);

  const firestore = resolveSharingFirestore(options);
  const rootRef = publicProfileRef(firestore, uid);
  const pageRef = catalogPageRef(firestore, uid, pageId);
  let result: SaveCatalogPageResult | null = null;

  await runTransaction(firestore, async (transaction) => {
    const rootSnapshot = await transaction.get(rootRef);
    if (!rootSnapshot.exists()) throw new SharingConflictError("Public profile is missing. Reload before editing this publication.");
    const profile = readProfile(rootSnapshot.data(), uid);
    if (profile.revision !== expectedRootRevision) {
      throw new SharingConflictError(`Public profile revision changed from ${expectedRootRevision} to ${profile.revision}. Reload before editing this publication.`);
    }
    if (!profile.enabled) throw new SharingConflictError("Public profile is disabled. Reload before editing this publication.");
    if (profile.generation !== expectedGeneration) {
      throw new SharingConflictError("Public profile generation changed. Reload before editing this publication.");
    }

    const pageSnapshot = await transaction.get(pageRef);
    if (!pageSnapshot.exists()) throw new SharingConflictError("Public catalog page is missing. Reload before editing this publication.");
    const page = readPage(pageSnapshot.data(), uid, pageId);
    if (page.generation !== expectedGeneration || page.revision !== expectedPageRevision) {
      throw new SharingConflictError("Public catalog changed on another device. Reload and review this publication before saving.");
    }
    if (profile.catalogPageCounts !== null && page.entries.length !== profile.catalogPageCounts[pageId]) {
      throw new SharingDataError("Public catalog counts do not match this page. Reload before editing this publication.");
    }

    const entryIndex = page.entries.findIndex((entry) => entry.fingerprint === fingerprint);
    if (entryIndex < 0) {
      throw new SharingConflictError("This publication was removed. Reload before editing; edits cannot restore removed games.");
    }
    const current = page.entries[entryIndex]!;
    const validated = decodePortableReplay(current.payload);
    const currentFingerprint = await portableReplayFingerprint(validated);
    if (currentFingerprint !== fingerprint) {
      throw new SharingDataError("This publication payload does not match its fingerprint. Reload before editing.");
    }
    const moveCount = validated.replay.moves.length;
    const openingPly = details.opening_ply === null ? moveCount : details.opening_ply;
    if (!Number.isInteger(openingPly) || openingPly < 0 || openingPly > moveCount) {
      throw new SharingDataError(`Published replay opening must be the default position or a move from 0 to ${moveCount}.`);
    }

    const currentOpening = validated.replay.viewPly ?? moveCount;
    if (normalizedTitle === current.title && openingPly === currentOpening) {
      if (isCurrentSession && !isCurrentSession()) {
        throw new SharingConflictError("The signed-in account changed before the publication edit finished.");
      }
      // A legacy root may not yet carry catalog_page_counts. This edit only
      // touches one fixed page, so preserve that legacy shape and avoid the
      // extra reads needed to initialize its manifest.
      result = { profile, page };
      return;
    }

    let payload = current.payload;
    let viewPly = current.view_ply;
    if (openingPly !== currentOpening) {
      const encoded = encodePortableReplay({ ...validated.replay, viewPly: openingPly });
      const updatedReplay = decodePortableReplay(encoded);
      const updatedFingerprint = await portableReplayFingerprint(updatedReplay);
      if (updatedFingerprint !== fingerprint) {
        throw new SharingDataError("Changing the public opening changed the replay fingerprint. This publication cannot be edited safely.");
      }
      payload = encoded;
      viewPly = openingPly;
    }

    if (isCurrentSession && !isCurrentSession()) {
      throw new SharingConflictError("The signed-in account changed before the publication edit finished.");
    }

    const entries = page.entries.map((entry, index) => index === entryIndex
      ? { ...entry, payload, title: normalizedTitle, view_ply: viewPly }
      : entry);
    const normalizedEntries = validateCatalogEntries(entries);
    const pageRevision = nextRevision(page.revision, "Catalog page revision");
    const rootRevision = nextRevision(profile.revision, "Public profile revision");
    const updatedAt = timestampNow();

    transaction.set(pageRef, pageDocument(profile.generation, pageRevision, normalizedEntries));
    transaction.set(rootRef, updatedProfileDocument(
      rootSnapshot.data()!,
      uid,
      profile.displayName,
      profile.enabled,
      profile.generation,
      rootRevision,
      profile.catalogPageCounts === null ? {} : { catalogPageCounts: profile.catalogPageCounts },
    ));
    result = {
      profile: { ...profile, revision: rootRevision, updatedAt },
      page: {
        uid,
        pageId,
        generation: profile.generation,
        revision: pageRevision,
        entries: normalizedEntries,
        updatedAt,
      },
    };
  });

  if (!result) throw new SharingDataError("Published replay update completed without a result.");
  return result;
}

/** Explicitly shares, refreshes, or removes the public aggregate statistics snapshot. */
export async function setPublicProfileStats(
  uid: string,
  stats: PublicProfileStatsCounters | null,
  expectedRootRevision: number,
  options: SharingFirestoreOptions = {},
): Promise<PublicProfileSnapshot> {
  assertExpectedRevision(expectedRootRevision, "Expected public profile revision");
  const normalizedStats = stats === null ? null : validatePublicProfileStatsCounters(stats);
  const firestore = resolveSharingFirestore(options);
  const rootRef = publicProfileRef(firestore, uid);
  let result: PublicProfileSnapshot | null = null;

  await runTransaction(firestore, async (transaction) => {
    const rootSnapshot = await transaction.get(rootRef);
    if (!rootSnapshot.exists()) throw new SharingConflictError("Public profile no longer exists. Reload before changing statistics.");
    const current = readProfile(rootSnapshot.data(), uid);
    if (current.revision !== expectedRootRevision) {
      throw new SharingConflictError(`Public profile revision changed from ${expectedRootRevision} to ${current.revision}. Reload before changing statistics.`);
    }
    if (normalizedStats !== null && !current.enabled) {
      throw new SharingDataError("Enable the public profile before sharing statistics.");
    }

    const revision = nextRevision(current.revision, "Public profile revision");
    const updatedAt = timestampNow();
    const sharedStats = normalizedStats === null ? null : { ...normalizedStats, updatedAt };
    const rawStats = normalizedStats === null
      ? null
      : { ...normalizedStats, updated_at: serverTimestamp() };
    transaction.set(rootRef, updatedProfileDocument(
      rootSnapshot.data()!,
      uid,
      current.displayName,
      current.enabled,
      current.generation,
      revision,
      { sharedStats: rawStats },
    ));
    result = { ...current, revision, sharedStats, updatedAt };
  });

  if (!result) throw new SharingDataError("Public statistics update completed without a result.");
  return result;
}

