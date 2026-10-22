import type { Firestore } from "firebase/firestore";

import { getFirebaseClients } from "./firebase.js";

export const SHARING_SCHEMA_VERSION = 1;
export const SHARING_PUBLIC_PAGE_COUNT = 4;
export const SHARING_CATALOG_PAGE_LIMIT = 8;
export const SHARING_PRIVATE_LIBRARY_LIMIT = 64;
export const SHARING_PAYLOAD_LIMIT = 8192;
export const SHARING_PRIVATE_JSON_LIMIT = 600_000;
export const SHARING_REVISION_LIMIT = 2_147_483_647;
export const SHARING_PAGE_IDS = ["0", "1", "2", "3"] as const;

export type SharingPageId = 0 | 1 | 2 | 3;

export interface SharingFirestoreOptions {
  firestore?: Firestore;
}

export interface PublicCatalogEntry {
  fingerprint: string;
  payload: string;
  title: string;
  view_ply: number;
}

export interface PublicProfileSnapshot {
  uid: string;
  displayName: string;
  enabled: boolean;
  generation: number;
  revision: number;
  updatedAt: string;
}

export interface PublicCatalogPageSnapshot {
  uid: string;
  pageId: SharingPageId;
  generation: number;
  revision: number;
  entries: PublicCatalogEntry[];
  updatedAt: string;
}

export interface PrivateLibraryEntry {
  fingerprint: string;
  payload: string;
  kind: "saved" | "practice";
  saved_at: string;
}

export interface PrivateLibrarySnapshot {
  uid: string;
  revision: number;
  entries: PrivateLibraryEntry[];
  updatedAt: string;
}

export class SharingDataError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "SharingDataError";
  }
}

export class SharingConflictError extends Error {
  constructor(message = "Sharing data changed on another device. Reload and review the latest version before saving.") {
    super(message);
    this.name = "SharingConflictError";
  }
}

export function resolveSharingFirestore(options: SharingFirestoreOptions = {}): Firestore {
  const firestore = options.firestore ?? getFirebaseClients()?.firestore;
  if (!firestore) {
    throw new SharingDataError("Sharing is not configured for this build.");
  }
  return firestore;
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function hasExactKeys(value: Record<string, unknown>, keys: readonly string[]): boolean {
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  return actual.length === expected.length && actual.every((key, index) => key === expected[index]);
}

export function assertRevision(value: unknown, label: string): asserts value is number {
  if (!Number.isInteger(value) || (value as number) < 1 || (value as number) > SHARING_REVISION_LIMIT) {
    throw new SharingDataError(`${label} must be a positive safe revision.`);
  }
}

export function assertExpectedRevision(value: number | null, label: string): void {
  if (value !== null) assertRevision(value, label);
}

export function nextRevision(value: number, label: string): number {
  assertRevision(value, label);
  if (value >= SHARING_REVISION_LIMIT) {
    throw new SharingDataError(`${label} has reached its supported limit.`);
  }
  return value + 1;
}

export function assertPageId(value: number): asserts value is SharingPageId {
  if (!Number.isInteger(value) || value < 0 || value >= SHARING_PUBLIC_PAGE_COUNT) {
    throw new SharingDataError("Public catalog page must be one of 0, 1, 2, or 3.");
  }
}

export function timestampIso(value: unknown, label: string): string {
  if (value instanceof Date && Number.isFinite(value.getTime())) return value.toISOString();
  const timestamp = value as { toDate?: unknown } | null;
  if (timestamp && typeof timestamp.toDate === "function") {
    const date = (timestamp.toDate as () => Date)();
    if (date instanceof Date && Number.isFinite(date.getTime())) return date.toISOString();
  }
  throw new SharingDataError(`${label} is not a valid server timestamp.`);
}

function unicodeLength(value: string): number {
  return Array.from(value).length;
}

function validToken(value: unknown, label: string): asserts value is string {
  if (typeof value !== "string" || value.length < 1 || value.length > SHARING_PAYLOAD_LIMIT
    || !/^[A-Za-z0-9_-]+$/.test(value)) {
    throw new SharingDataError(`${label} must be an unpadded base64url token of at most ${SHARING_PAYLOAD_LIMIT} ASCII characters.`);
  }
}

function validFingerprint(value: unknown, label: string): asserts value is string {
  if (typeof value !== "string" || !/^[A-Za-z0-9_-]{43}$/.test(value)) {
    throw new SharingDataError(`${label} must be a 43-character base64url fingerprint.`);
  }
}

export function validateCatalogEntries(value: unknown): PublicCatalogEntry[] {
  if (!Array.isArray(value) || value.length > SHARING_CATALOG_PAGE_LIMIT) {
    throw new SharingDataError(`A public catalog page can contain at most ${SHARING_CATALOG_PAGE_LIMIT} entries.`);
  }

  const seen = new Set<string>();
  return value.map((entry, index) => {
    if (!isRecord(entry) || !hasExactKeys(entry, ["fingerprint", "payload", "title", "view_ply"])) {
      throw new SharingDataError(`Catalog entry ${index + 1} has an invalid shape.`);
    }
    validFingerprint(entry.fingerprint, `Catalog entry ${index + 1} fingerprint`);
    validToken(entry.payload, `Catalog entry ${index + 1} payload`);
    if (typeof entry.title !== "string" || unicodeLength(entry.title) > 80
      || new TextEncoder().encode(entry.title).length > 320 || /[\r\n\u2028\u2029]/u.test(entry.title)) {
      throw new SharingDataError(`Catalog entry ${index + 1} title must be at most 80 characters and 320 UTF-8 bytes.`);
    }
    if (!Number.isInteger(entry.view_ply) || (entry.view_ply as number) < 0 || (entry.view_ply as number) > 225) {
      throw new SharingDataError(`Catalog entry ${index + 1} view_ply must be an integer from 0 to 225.`);
    }
    if (seen.has(entry.fingerprint)) throw new SharingDataError("Catalog entries cannot repeat a fingerprint.");
    seen.add(entry.fingerprint);
    return {
      fingerprint: entry.fingerprint,
      payload: entry.payload,
      title: entry.title,
      view_ply: entry.view_ply as number,
    };
  });
}

function validIsoDate(value: unknown, label: string): asserts value is string {
  if (typeof value !== "string" || !/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{3}Z$/.test(value)
    || !Number.isFinite(Date.parse(value)) || new Date(value).toISOString() !== value) {
    throw new SharingDataError(`${label} must be a canonical ISO timestamp.`);
  }
}

export function validatePrivateLibraryEntries(value: unknown): PrivateLibraryEntry[] {
  if (!Array.isArray(value) || value.length > SHARING_PRIVATE_LIBRARY_LIMIT) {
    throw new SharingDataError(`A private library can contain at most ${SHARING_PRIVATE_LIBRARY_LIMIT} entries.`);
  }
  const seen = new Set<string>();
  return value.map((entry, index) => {
    if (!isRecord(entry) || !hasExactKeys(entry, ["fingerprint", "payload", "kind", "saved_at"])) {
      throw new SharingDataError(`Private library entry ${index + 1} has an invalid shape.`);
    }
    validFingerprint(entry.fingerprint, `Private library entry ${index + 1} fingerprint`);
    validToken(entry.payload, `Private library entry ${index + 1} payload`);
    if (entry.kind !== "saved" && entry.kind !== "practice") {
      throw new SharingDataError(`Private library entry ${index + 1} kind must be saved or practice.`);
    }
    validIsoDate(entry.saved_at, `Private library entry ${index + 1} saved_at`);
    if (seen.has(entry.fingerprint)) throw new SharingDataError("Private library entries cannot repeat a fingerprint.");
    seen.add(entry.fingerprint);
    return {
      fingerprint: entry.fingerprint,
      payload: entry.payload,
      kind: entry.kind,
      saved_at: entry.saved_at,
    };
  });
}

export function privateLibraryJson(entries: unknown): string {
  const normalized = validatePrivateLibraryEntries(entries);
  const json = JSON.stringify(normalized);
  if (json.length > SHARING_PRIVATE_JSON_LIMIT || !/^[\x20-\x7e]*$/.test(json)) {
    throw new SharingDataError(`Private library snapshot exceeds ${SHARING_PRIVATE_JSON_LIMIT} ASCII characters.`);
  }
  return json;
}

export function decodePrivateLibraryJson(value: unknown): PrivateLibraryEntry[] {
  if (typeof value !== "string" || value.length > SHARING_PRIVATE_JSON_LIMIT || !/^[\x20-\x7e]*$/.test(value)) {
    throw new SharingDataError(`Private library snapshot must be at most ${SHARING_PRIVATE_JSON_LIMIT} ASCII characters.`);
  }
  let decoded: unknown;
  try {
    decoded = JSON.parse(value);
  } catch {
    throw new SharingDataError("Private library snapshot is not valid JSON.");
  }
  return validatePrivateLibraryEntries(decoded);
}

export const PUBLIC_PROFILE_KEYS = [
  "schema_version", "uid", "display_name", "enabled", "generation", "revision", "updated_at",
] as const;

export const CATALOG_PAGE_KEYS = ["schema_version", "generation", "revision", "updated_at", "entries"] as const;

export const PRIVATE_LIBRARY_KEYS = ["schema_version", "uid", "revision", "updated_at", "entries_json"] as const;

export function parsePublicProfileSnapshot(value: unknown, expectedUid: string): PublicProfileSnapshot {
  if (!isRecord(value) || !hasExactKeys(value, PUBLIC_PROFILE_KEYS)
    || value.schema_version !== SHARING_SCHEMA_VERSION || value.uid !== expectedUid
    || typeof value.display_name !== "string" || typeof value.enabled !== "boolean") {
    throw new SharingDataError("Public profile has an invalid document shape.");
  }
  const nameLength = unicodeLength(value.display_name);
  if ((value.enabled && (nameLength < 1 || nameLength > 80 || new TextEncoder().encode(value.display_name).length > 320))
    || (!value.enabled && value.display_name !== "")) {
    throw new SharingDataError("Public profile display name is inconsistent with its visibility.");
  }
  assertRevision(value.generation, "Public profile generation");
  assertRevision(value.revision, "Public profile revision");
  return {
    uid: expectedUid,
    displayName: value.display_name,
    enabled: value.enabled,
    generation: value.generation,
    revision: value.revision,
    updatedAt: timestampIso(value.updated_at, "Public profile updated_at"),
  };
}

export function parseCatalogPageSnapshot(value: unknown, uid: string, pageId: SharingPageId): PublicCatalogPageSnapshot {
  if (!isRecord(value) || !hasExactKeys(value, CATALOG_PAGE_KEYS) || value.schema_version !== SHARING_SCHEMA_VERSION) {
    throw new SharingDataError("Public catalog page has an invalid document shape.");
  }
  assertRevision(value.generation, "Catalog page generation");
  assertRevision(value.revision, "Catalog page revision");
  return {
    uid,
    pageId,
    generation: value.generation,
    revision: value.revision,
    entries: validateCatalogEntries(value.entries),
    updatedAt: timestampIso(value.updated_at, "Catalog page updated_at"),
  };
}

export function parsePrivateLibrarySnapshot(value: unknown, uid: string): PrivateLibrarySnapshot {
  if (!isRecord(value) || !hasExactKeys(value, PRIVATE_LIBRARY_KEYS) || value.schema_version !== SHARING_SCHEMA_VERSION
    || value.uid !== uid) {
    throw new SharingDataError("Private library has an invalid document shape.");
  }
  assertRevision(value.revision, "Private library revision");
  return {
    uid,
    revision: value.revision,
    entries: decodePrivateLibraryJson(value.entries_json),
    updatedAt: timestampIso(value.updated_at, "Private library updated_at"),
  };
}
