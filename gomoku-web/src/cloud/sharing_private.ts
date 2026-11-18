import {
  doc,
  getDocFromServer,
  runTransaction,
  serverTimestamp,
  type DocumentReference,
  type Firestore,
} from "firebase/firestore";

import {
  SHARING_PRIVATE_JSON_LIMIT,
  SHARING_PRIVATE_LIBRARY_LIMIT,
  SHARING_PAYLOAD_LIMIT,
  SharingConflictError,
  SharingDataError,
  assertExpectedRevision,
  assertRevision,
  hasExactKeys,
  isRecord,
  nextRevision,
  resolveSharingFirestore,
  timestampIso,
  validatePrivateLibraryEntries as validateLegacyPrivateLibraryEntries,
  decodePrivateLibraryJson,
  type PrivateLibraryEntry as LegacyPrivateLibraryEntry,
  type SharingFirestoreOptions,
} from "./sharing_common.js";
import {
  DEFAULT_PRIVATE_REPLAY_METADATA,
  validatePrivateReplayMetadata,
  type PrivateReplayMetadata,
} from "../replay/private_replay_metadata.js";

export const PRIVATE_LIBRARY_SCHEMA_VERSION = 2;

export interface PrivateLibraryEntry extends LegacyPrivateLibraryEntry {
  metadata: PrivateReplayMetadata;
}

export interface PrivateLibrarySnapshot {
  uid: string;
  revision: number;
  entries: PrivateLibraryEntry[];
  updatedAt: string;
}

export type { PrivateReplayMetadata } from "../replay/private_replay_metadata.js";

const FINGERPRINT_PATTERN = /^[A-Za-z0-9_-]{43}$/;
const TOKEN_PATTERN = /^[A-Za-z0-9_-]{1,8192}$/;
const BASE64URL_PATTERN = /^[A-Za-z0-9_-]*$/;
const BASE64URL_ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

function canonicalIsoDate(value: unknown, label: string): asserts value is string {
  if (typeof value !== "string" || !/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{3}Z$/.test(value)
    || !Number.isFinite(Date.parse(value)) || new Date(value).toISOString() !== value) {
    throw new SharingDataError(`${label} must be a canonical ISO timestamp.`);
  }
}

function encodeBase64Url(value: string): string {
  const bytes = new TextEncoder().encode(value);
  let output = "";
  for (let index = 0; index < bytes.length; index += 3) {
    const first = bytes[index]!;
    const second = bytes[index + 1];
    const third = bytes[index + 2];
    output += BASE64URL_ALPHABET[first >> 2];
    output += BASE64URL_ALPHABET[((first & 3) << 4) | ((second ?? 0) >> 4)];
    if (second !== undefined) output += BASE64URL_ALPHABET[((second & 15) << 2) | ((third ?? 0) >> 6)];
    if (third !== undefined) output += BASE64URL_ALPHABET[third & 63];
  }
  return output;
}

function decodeBase64Url(value: unknown, label: string): string {
  if (typeof value !== "string" || !BASE64URL_PATTERN.test(value) || value.length % 4 === 1) {
    throw new SharingDataError(`${label} must be canonical unpadded base64url.`);
  }
  const bytes: number[] = [];
  let accumulator = 0;
  let bits = 0;
  for (const character of value) {
    const digit = BASE64URL_ALPHABET.indexOf(character);
    if (digit < 0) throw new SharingDataError(`${label} must be canonical unpadded base64url.`);
    accumulator = (accumulator << 6) | digit;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      bytes.push((accumulator >> bits) & 0xff);
      accumulator &= (1 << bits) - 1;
    }
  }
  if (bits > 0 && accumulator !== 0) throw new SharingDataError(`${label} has non-zero trailing base64url bits.`);
  let decoded: string;
  try {
    decoded = new TextDecoder("utf-8", { fatal: true }).decode(new Uint8Array(bytes));
  } catch {
    throw new SharingDataError(`${label} does not contain valid UTF-8.`);
  }
  if (encodeBase64Url(decoded) !== value) throw new SharingDataError(`${label} is not canonical unpadded base64url.`);
  return decoded;
}

function validateMetadata(value: unknown): PrivateReplayMetadata {
  try {
    return validatePrivateReplayMetadata(value);
  } catch (error) {
    throw new SharingDataError(error instanceof Error ? error.message : "Private replay details are invalid.");
  }
}

function validateLogicalEntries(value: unknown): PrivateLibraryEntry[] {
  if (!Array.isArray(value) || value.length > SHARING_PRIVATE_LIBRARY_LIMIT) {
    throw new SharingDataError(`A private library can contain at most ${SHARING_PRIVATE_LIBRARY_LIMIT} entries.`);
  }
  const seen = new Set<string>();
  return value.map((candidate, index) => {
    if (!isRecord(candidate)) throw new SharingDataError(`Private library entry ${index + 1} has an invalid shape.`);
    const hasMetadata = Object.prototype.hasOwnProperty.call(candidate, "metadata");
    const expected = hasMetadata
      ? ["fingerprint", "payload", "kind", "saved_at", "metadata"]
      : ["fingerprint", "payload", "kind", "saved_at"];
    if (!hasExactKeys(candidate, expected)) throw new SharingDataError(`Private library entry ${index + 1} has an invalid shape.`);
    if (typeof candidate.fingerprint !== "string" || !FINGERPRINT_PATTERN.test(candidate.fingerprint)) {
      throw new SharingDataError(`Private library entry ${index + 1} fingerprint must be a 43-character base64url value.`);
    }
    if (typeof candidate.payload !== "string" || !TOKEN_PATTERN.test(candidate.payload)) {
      throw new SharingDataError(`Private library entry ${index + 1} payload must be an unpadded base64url token of at most ${SHARING_PAYLOAD_LIMIT} characters.`);
    }
    if (candidate.kind !== "saved" && candidate.kind !== "practice") {
      throw new SharingDataError(`Private library entry ${index + 1} kind must be saved or practice.`);
    }
    canonicalIsoDate(candidate.saved_at, `Private library entry ${index + 1} saved_at`);
    if (seen.has(candidate.fingerprint)) throw new SharingDataError("Private library entries cannot repeat a fingerprint.");
    seen.add(candidate.fingerprint);
    return {
      fingerprint: candidate.fingerprint,
      payload: candidate.payload,
      kind: candidate.kind,
      saved_at: candidate.saved_at,
      metadata: hasMetadata ? validateMetadata(candidate.metadata) : { ...DEFAULT_PRIVATE_REPLAY_METADATA },
    };
  });
}

function canonicalMetadataJson(metadata: PrivateReplayMetadata): string {
  return JSON.stringify({
    title: encodeBase64Url(metadata.title),
    note: encodeBase64Url(metadata.note),
    opening_ply: metadata.opening_ply,
  });
}

export function privateLibraryV2Json(entries: unknown): string {
  const normalized = validateLogicalEntries(entries);
  const encoded = normalized.map((entry) => ({
    fingerprint: entry.fingerprint,
    payload: entry.payload,
    kind: entry.kind,
    saved_at: entry.saved_at,
    metadata: JSON.parse(canonicalMetadataJson(entry.metadata)) as {
      title: string;
      note: string;
      opening_ply: number | null;
    },
  }));
  const json = JSON.stringify(encoded);
  if (json.length > SHARING_PRIVATE_JSON_LIMIT || !/^[\x20-\x7e]*$/.test(json)) {
    throw new SharingDataError(`Private library snapshot exceeds ${SHARING_PRIVATE_JSON_LIMIT} ASCII characters.`);
  }
  return json;
}

function decodeV2Entries(value: unknown): PrivateLibraryEntry[] {
  if (typeof value !== "string" || value.length > SHARING_PRIVATE_JSON_LIMIT || !/^[\x20-\x7e]*$/.test(value)) {
    throw new SharingDataError(`Private library snapshot must be at most ${SHARING_PRIVATE_JSON_LIMIT} ASCII characters.`);
  }
  let decoded: unknown;
  try {
    decoded = JSON.parse(value);
  } catch {
    throw new SharingDataError("Private library snapshot is not valid JSON.");
  }
  if (!Array.isArray(decoded) || decoded.length > SHARING_PRIVATE_LIBRARY_LIMIT) {
    throw new SharingDataError(`A private library can contain at most ${SHARING_PRIVATE_LIBRARY_LIMIT} entries.`);
  }
  const seen = new Set<string>();
  return decoded.map((candidate, index) => {
    if (!isRecord(candidate) || !hasExactKeys(candidate, ["fingerprint", "payload", "kind", "saved_at", "metadata"])) {
      throw new SharingDataError(`Private library entry ${index + 1} has an invalid shape.`);
    }
    if (typeof candidate.fingerprint !== "string" || !FINGERPRINT_PATTERN.test(candidate.fingerprint)) {
      throw new SharingDataError(`Private library entry ${index + 1} fingerprint must be a 43-character base64url value.`);
    }
    if (typeof candidate.payload !== "string" || !TOKEN_PATTERN.test(candidate.payload)) {
      throw new SharingDataError(`Private library entry ${index + 1} payload must be an unpadded base64url token of at most ${SHARING_PAYLOAD_LIMIT} characters.`);
    }
    if (candidate.kind !== "saved" && candidate.kind !== "practice") {
      throw new SharingDataError(`Private library entry ${index + 1} kind must be saved or practice.`);
    }
    canonicalIsoDate(candidate.saved_at, `Private library entry ${index + 1} saved_at`);
    if (seen.has(candidate.fingerprint)) throw new SharingDataError("Private library entries cannot repeat a fingerprint.");
    seen.add(candidate.fingerprint);
    if (!isRecord(candidate.metadata) || !hasExactKeys(candidate.metadata, ["title", "note", "opening_ply"])) {
      throw new SharingDataError(`Private library entry ${index + 1} metadata has an invalid shape.`);
    }
    const title = decodeBase64Url(candidate.metadata.title, `Private library entry ${index + 1} title`);
    const note = decodeBase64Url(candidate.metadata.note, `Private library entry ${index + 1} note`);
    const metadata = validateMetadata({ title, note, opening_ply: candidate.metadata.opening_ply });
    if (metadata.title !== title || metadata.note !== note) {
      throw new SharingDataError(`Private library entry ${index + 1} metadata text is not normalized.`);
    }
    return {
      fingerprint: candidate.fingerprint,
      payload: candidate.payload,
      kind: candidate.kind,
      saved_at: candidate.saved_at,
      metadata,
    };
  });
}

function parseV1Entries(value: unknown): PrivateLibraryEntry[] {
  const legacy = decodePrivateLibraryJson(value);
  return legacy.map((entry) => ({ ...entry, metadata: { ...DEFAULT_PRIVATE_REPLAY_METADATA } }));
}

export function privateLibraryRef(firestore: Firestore, uid: string): DocumentReference {
  return doc(firestore, "replay_libraries", uid);
}

export function privateLibraryDocument(uid: string, revision: number, entriesJson: string) {
  return {
    schema_version: PRIVATE_LIBRARY_SCHEMA_VERSION,
    uid,
    revision,
    updated_at: serverTimestamp(),
    entries_json: entriesJson,
  };
}

export function parsePrivateLibrarySnapshot(value: unknown, uid: string): PrivateLibrarySnapshot {
  if (!isRecord(value) || !hasExactKeys(value, ["schema_version", "uid", "revision", "updated_at", "entries_json"])
    || (value.schema_version !== 1 && value.schema_version !== PRIVATE_LIBRARY_SCHEMA_VERSION)
    || value.uid !== uid) {
    throw new SharingDataError("Private library has an invalid document shape or unsupported schema version.");
  }
  assertRevision(value.revision, "Private library revision");
  return {
    uid,
    revision: value.revision,
    entries: value.schema_version === 1 ? parseV1Entries(value.entries_json) : decodeV2Entries(value.entries_json),
    updatedAt: timestampIso(value.updated_at, "Private library updated_at"),
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

/** Replaces the complete private v2 snapshot at its retained revision. */
export async function savePrivateLibrary(
  uid: string,
  entries: unknown,
  expectedRevision: number | null,
  options: SharingFirestoreOptions = {},
): Promise<PrivateLibrarySnapshot> {
  assertExpectedRevision(expectedRevision, "Expected private library revision");
  const entriesJson = privateLibraryV2Json(entries);
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
      result = { uid, revision, entries: validateLogicalEntries(entries), updatedAt: new Date().toISOString() };
      return;
    }

    const current = parsePrivateLibrarySnapshot(snapshot.data(), uid);
    if (expectedRevision === null || current.revision !== expectedRevision) {
      throw new SharingConflictError(`Private library revision changed from ${expectedRevision ?? "missing"} to ${current.revision}. Reload and review before saving.`);
    }
    const revision = nextRevision(current.revision, "Private library revision");
    transaction.set(libraryRef, privateLibraryDocument(uid, revision, entriesJson));
    result = { uid, revision, entries: validateLogicalEntries(entries), updatedAt: new Date().toISOString() };
  });

  if (!result) throw new SharingDataError("Private library update completed without a result.");
  return result;
}

// Validate the legacy shape for existing callers that still construct entries
// without details; the write path above supplies default details and v2 bytes.
export function validatePrivateLibraryEntries(value: unknown): PrivateLibraryEntry[] {
  try {
    const normalized = validateLogicalEntries(value);
    // Also retain the old validator's portable-independent payload/date bounds.
    validateLegacyPrivateLibraryEntries(normalized.map(({ metadata: _metadata, ...entry }) => entry));
    return normalized;
  } catch (error) {
    if (error instanceof SharingDataError) throw error;
    throw new SharingDataError(error instanceof Error ? error.message : "Private library entries are invalid.");
  }
}
