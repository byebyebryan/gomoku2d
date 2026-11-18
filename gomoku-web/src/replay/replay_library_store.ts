import { createStore, type StoreApi } from "zustand/vanilla";

import type { PrivateLibraryEntry, PrivateReplayMetadata } from "../cloud/sharing_private";
import { cloudAuthStore } from "../cloud/auth_store";
import type { BrowserStorage } from "../profile/best_effort_storage";
import {
  DEFAULT_PRIVATE_REPLAY_METADATA,
  validatePrivateReplayMetadata,
} from "./private_replay_metadata";
import {
  decodePortableReplay,
  encodePortableReplay,
  portableReplayFingerprint,
} from "./portable_replay_codec";
import { validatePortableReplay, type PortableReplayV1 } from "./portable_replay";

export const REPLAY_LIBRARY_LIMIT = 64;
export const REPLAY_LIBRARY_STORAGE_KEY = "gomoku2d.replay-library.v2";
export const REPLAY_LIBRARY_LEGACY_STORAGE_KEY = "gomoku2d.replay-library.v1";
export const REPLAY_LIBRARY_SCHEMA_VERSION = 2;
export const REPLAY_LIBRARY_STORAGE_BYTES_LIMIT = 600_000;
export const REPLAY_LIBRARY_LOCK_NAME = "gomoku2d.replay-library.v2";

const FINGERPRINT_PATTERN = /^[A-Za-z0-9_-]{43}$/;
const PAYLOAD_PATTERN = /^[A-Za-z0-9_-]{1,8192}$/;
const CONTROL_CHARACTER_PATTERN = /[\u0000-\u001f\u007f-\u009f]/u;

export type ReplayLibraryOwner = "guest" | `user:${string}`;
export type ReplayLibraryKind = PrivateLibraryEntry["kind"];
export type ReplayLibraryBaseRevision = number | null | "unknown";

export interface ReplayLibraryOwnerState {
  baseRevision: ReplayLibraryBaseRevision;
  dirty: boolean;
  durable: boolean;
  entries: PrivateLibraryEntry[];
  error: string | null;
  restoreReview: boolean;
}

export interface ReplayLibraryReplaceOptions {
  baseRevision?: ReplayLibraryBaseRevision;
  dirty?: boolean;
  expectedRevision?: number;
  preflightOnly?: boolean;
  restoreReview?: boolean;
}

export interface ReplayLibraryLockManager {
  request<T>(name: string, options: { mode: "exclusive" }, callback: () => Promise<T>): Promise<T>;
}

export interface ReplayLibraryState {
  owners: Record<string, ReplayLibraryOwnerState>;
  revision: number;
  sessionOnly: boolean;
  readOnly: boolean;
  storageError: string | null;
  refreshFromStorage: () => Promise<boolean>;
  clearOwner: (owner: ReplayLibraryOwner) => Promise<void>;
  markDirty: (owner: ReplayLibraryOwner) => Promise<void>;
  markSynced: (owner: ReplayLibraryOwner, baseRevision: number | null, expectedRevision?: number, canReplace?: () => boolean) => Promise<void>;
  markRestoreReview: (owner: ReplayLibraryOwner, restoreReview?: boolean) => Promise<void>;
  removeReplay: (owner: ReplayLibraryOwner, fingerprint: string) => Promise<void>;
  replaceEntries: (
    owner: ReplayLibraryOwner,
    entries: unknown,
    canReplace?: () => boolean,
    options?: ReplayLibraryReplaceOptions,
  ) => Promise<void>;
  saveReplay: (owner: ReplayLibraryOwner, replay: unknown, kind: ReplayLibraryKind, guard?: () => void) => Promise<string>;
  updateReplayMetadata: (
    owner: ReplayLibraryOwner,
    fingerprint: string,
    metadata: PrivateReplayMetadata,
    expectedMetadata?: PrivateReplayMetadata,
    guard?: () => void,
  ) => Promise<void>;
}

export interface ReplayLibraryStoreOptions {
  onPersistenceUnavailable?: (error?: unknown) => void;
  now?: () => string;
  storage?: BrowserStorage;
  lockManager?: ReplayLibraryLockManager | null;
}

interface PersistedOwnerState {
  entries: PrivateLibraryEntry[];
  dirty: boolean;
  base_revision: ReplayLibraryBaseRevision;
  restore_review: boolean;
}

interface PersistedReplayLibrary {
  schema_version: typeof REPLAY_LIBRARY_SCHEMA_VERSION;
  revision: number;
  owners: Record<string, PersistedOwnerState>;
}

interface NormalizedChange {
  owners: Record<string, ReplayLibraryOwnerState>;
  changed: boolean;
}

const EMPTY_OWNER_STATE: ReplayLibraryOwnerState = {
  baseRevision: "unknown",
  dirty: false,
  durable: true,
  entries: [],
  error: null,
  restoreReview: false,
};

function isPlainRecord(value: unknown): value is Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function hasExactKeys(record: Record<string, unknown>, keys: readonly string[]): boolean {
  const actual = Object.keys(record).sort();
  const expected = [...keys].sort();
  return actual.length === expected.length && actual.every((key, index) => key === expected[index]);
}

function messageFor(error: unknown): string {
  if (error instanceof Error && error.message.trim()) return error.message;
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string" && message.trim()) return message;
  }
  return "Replay library storage is unavailable.";
}

function assertOwner(value: string): asserts value is ReplayLibraryOwner {
  if (value === "guest") return;
  if (!value.startsWith("user:")) throw new Error("Replay library owner must be guest or a namespaced user UID.");
  const uid = value.slice("user:".length);
  if (!uid || Array.from(uid).length > 128 || CONTROL_CHARACTER_PATTERN.test(uid)) {
    throw new Error("Replay library user owner must include a bounded UID.");
  }
}

function assertBaseRevision(value: unknown, owner: string): asserts value is ReplayLibraryBaseRevision {
  if (value === null || value === "unknown") return;
  if (!Number.isSafeInteger(value) || (value as number) < 1) {
    throw new Error(`Replay library cache for ${owner} has an invalid base revision.`);
  }
}

function validateEntry(value: unknown, index: number, allowLegacy: boolean): PrivateLibraryEntry {
  if (!isPlainRecord(value)) throw new Error(`Replay library entry ${index + 1} has an invalid shape.`);
  const hasMetadata = Object.prototype.hasOwnProperty.call(value, "metadata");
  if (!hasExactKeys(value, hasMetadata
    ? ["fingerprint", "payload", "kind", "saved_at", "metadata"]
    : ["fingerprint", "payload", "kind", "saved_at"])) {
    throw new Error(`Replay library entry ${index + 1} has an invalid shape.`);
  }
  if (!hasMetadata && !allowLegacy) throw new Error(`Replay library entry ${index + 1} is missing private details.`);
  if (typeof value.fingerprint !== "string" || !FINGERPRINT_PATTERN.test(value.fingerprint)) {
    throw new Error(`Replay library entry ${index + 1} has an invalid fingerprint.`);
  }
  if (typeof value.payload !== "string" || !PAYLOAD_PATTERN.test(value.payload)) {
    throw new Error(`Replay library entry ${index + 1} has an invalid payload.`);
  }
  if (value.kind !== "saved" && value.kind !== "practice") {
    throw new Error(`Replay library entry ${index + 1} has an invalid kind.`);
  }
  if (typeof value.saved_at !== "string" || !/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{3}Z$/.test(value.saved_at)
    || !Number.isFinite(Date.parse(value.saved_at)) || new Date(value.saved_at).toISOString() !== value.saved_at) {
    throw new Error(`Replay library entry ${index + 1} has an invalid saved_at timestamp.`);
  }
  const decoded = decodePortableReplay(value.payload);
  if (value.kind === "practice" && !decoded.replay.practice) {
    throw new Error(`Replay library practice entry ${index + 1} is missing its practice origin.`);
  }
  let metadata: PrivateReplayMetadata;
  try {
    metadata = validatePrivateReplayMetadata(hasMetadata ? value.metadata : DEFAULT_PRIVATE_REPLAY_METADATA, decoded.replay.moves.length);
  } catch (error) {
    throw new Error(`Replay library entry ${index + 1} has invalid private details: ${messageFor(error)}`);
  }
  if (hasMetadata && JSON.stringify(value.metadata) !== JSON.stringify(metadata)) {
    throw new Error(`Replay library entry ${index + 1} private details are not normalized.`);
  }
  return {
    fingerprint: value.fingerprint,
    payload: value.payload,
    kind: value.kind,
    saved_at: value.saved_at,
    metadata,
  };
}

export function validateReplayLibraryEntries(value: unknown): PrivateLibraryEntry[] {
  if (!Array.isArray(value) || value.length > REPLAY_LIBRARY_LIMIT) {
    throw new Error(`A replay library can contain at most ${REPLAY_LIBRARY_LIMIT} entries.`);
  }
  const seen = new Set<string>();
  return value.map((entry, index) => {
    const validated = validateEntry(entry, index, true);
    if (seen.has(validated.fingerprint)) throw new Error("Replay library entries cannot repeat a fingerprint.");
    seen.add(validated.fingerprint);
    return validated;
  });
}

function validateV2Entries(value: unknown): PrivateLibraryEntry[] {
  if (!Array.isArray(value) || value.length > REPLAY_LIBRARY_LIMIT) {
    throw new Error(`A replay library can contain at most ${REPLAY_LIBRARY_LIMIT} entries.`);
  }
  const seen = new Set<string>();
  return value.map((entry, index) => {
    const validated = validateEntry(entry, index, false);
    if (seen.has(validated.fingerprint)) throw new Error("Replay library entries cannot repeat a fingerprint.");
    seen.add(validated.fingerprint);
    return validated;
  });
}

export async function verifyReplayLibraryEntries(value: unknown): Promise<PrivateLibraryEntry[]> {
  const entries = validateReplayLibraryEntries(value);
  const verified: PrivateLibraryEntry[] = [];
  for (const entry of entries) {
    const decoded = decodePortableReplay(entry.payload);
    const fingerprint = await portableReplayFingerprint(decoded);
    if (fingerprint !== entry.fingerprint) {
      throw new Error(`Replay library payload does not match fingerprint ${entry.fingerprint}.`);
    }
    const metadata = validatePrivateReplayMetadata(entry.metadata, decoded.replay.moves.length);
    const { viewPly: _viewPly, ...withoutViewPly } = decoded.replay;
    verified.push({ ...entry, metadata, payload: encodePortableReplay(withoutViewPly) });
  }
  return verified;
}

function parseLegacy(raw: string | null): Record<string, PersistedOwnerState> {
  if (raw === null) return Object.create(null) as Record<string, PersistedOwnerState>;
  if (new TextEncoder().encode(raw).byteLength > REPLAY_LIBRARY_STORAGE_BYTES_LIMIT) {
    throw new Error(`Replay library cache exceeds ${REPLAY_LIBRARY_STORAGE_BYTES_LIMIT} bytes.`);
  }
  let parsed: unknown;
  try { parsed = JSON.parse(raw); } catch { throw new Error("Legacy replay library cache is not valid JSON."); }
  if (!isPlainRecord(parsed) || !hasExactKeys(parsed, ["schema_version", "owners"]) || parsed.schema_version !== 1
    || !isPlainRecord(parsed.owners)) {
    throw new Error("Legacy replay library cache has an unsupported shape or schema version.");
  }
  const owners: Record<string, PersistedOwnerState> = Object.create(null) as Record<string, PersistedOwnerState>;
  for (const [owner, candidate] of Object.entries(parsed.owners)) {
    assertOwner(owner);
    if (!isPlainRecord(candidate) || !hasExactKeys(candidate, ["entries", "dirty"]) || typeof candidate.dirty !== "boolean") {
      throw new Error(`Legacy replay library cache for ${owner} has an invalid shape.`);
    }
    const entries = validateReplayLibraryEntries(candidate.entries);
    owners[owner] = { entries, dirty: candidate.dirty, base_revision: "unknown", restore_review: false };
  }
  return owners;
}

function parseV2(raw: string): PersistedReplayLibrary {
  if (new TextEncoder().encode(raw).byteLength > REPLAY_LIBRARY_STORAGE_BYTES_LIMIT) {
    throw new Error(`Replay library cache exceeds ${REPLAY_LIBRARY_STORAGE_BYTES_LIMIT} bytes.`);
  }
  let parsed: unknown;
  try { parsed = JSON.parse(raw); } catch { throw new Error("Replay library cache is not valid JSON."); }
  if (!isPlainRecord(parsed) || !hasExactKeys(parsed, ["schema_version", "revision", "owners"])) {
    throw new Error("Replay library cache has an unsupported shape.");
  }
  if (parsed.schema_version !== REPLAY_LIBRARY_SCHEMA_VERSION) {
    throw new Error(`Replay library cache uses unsupported schema version ${String(parsed.schema_version)}.`);
  }
  if (!Number.isSafeInteger(parsed.revision) || (parsed.revision as number) < 1) {
    throw new Error("Replay library cache has an invalid revision.");
  }
  if (!isPlainRecord(parsed.owners)) throw new Error("Replay library cache owners must be an object.");
  const owners: Record<string, PersistedOwnerState> = Object.create(null) as Record<string, PersistedOwnerState>;
  for (const [owner, candidate] of Object.entries(parsed.owners)) {
    assertOwner(owner);
    if (!isPlainRecord(candidate) || !hasExactKeys(candidate, ["entries", "dirty", "base_revision", "restore_review"])
      || typeof candidate.dirty !== "boolean" || typeof candidate.restore_review !== "boolean") {
      throw new Error(`Replay library cache for ${owner} has an invalid shape.`);
    }
    assertBaseRevision(candidate.base_revision, owner);
    if (owner === "guest" && (candidate.base_revision !== "unknown" || candidate.restore_review)) {
      throw new Error("Guest replay-library buckets must use an unknown base and cannot require cloud restore review.");
    }
    owners[owner] = {
      entries: validateV2Entries(candidate.entries),
      dirty: candidate.dirty,
      base_revision: candidate.base_revision,
      restore_review: candidate.restore_review,
    };
  }
  return { schema_version: REPLAY_LIBRARY_SCHEMA_VERSION, revision: parsed.revision as number, owners };
}

function serializedSize(value: string): number {
  return new TextEncoder().encode(value).byteLength;
}

function ownerStateFromPersisted(value: PersistedOwnerState, durable = true): ReplayLibraryOwnerState {
  return {
    baseRevision: value.base_revision,
    dirty: value.dirty,
    durable,
    entries: value.entries,
    error: null,
    restoreReview: value.restore_review,
  };
}

function persistedOwners(owners: Record<string, ReplayLibraryOwnerState>): Record<string, PersistedOwnerState> {
  const result: Record<string, PersistedOwnerState> = Object.create(null) as Record<string, PersistedOwnerState>;
  for (const [owner, state] of Object.entries(owners)) {
    result[owner] = {
      entries: state.entries,
      dirty: state.dirty,
      base_revision: state.baseRevision,
      restore_review: state.restoreReview,
    };
  }
  return result;
}

function serializeLibrary(owners: Record<string, ReplayLibraryOwnerState>, revision: number): string {
  if (!Number.isSafeInteger(revision) || revision < 1) throw new Error("Replay library revision has reached its supported limit.");
  const envelope: PersistedReplayLibrary = {
    schema_version: REPLAY_LIBRARY_SCHEMA_VERSION,
    revision,
    owners: persistedOwners(owners),
  };
  const serialized = JSON.stringify(envelope);
  if (serializedSize(serialized) > REPLAY_LIBRARY_STORAGE_BYTES_LIMIT) {
    throw new Error(`Replay library cache exceeds ${REPLAY_LIBRARY_STORAGE_BYTES_LIMIT} bytes.`);
  }
  return serialized;
}

function getBrowserLockManager(): ReplayLibraryLockManager | null {
  try {
    if (typeof navigator === "undefined") return null;
    const locks = (navigator as Navigator & { locks?: ReplayLibraryLockManager }).locks;
    return locks ?? null;
  } catch {
    return null;
  }
}

function wrapMutationError(error: unknown): Error {
  return error instanceof Error ? error : new Error(messageFor(error));
}

class StorageUnavailableError extends Error {
  constructor(error: unknown) {
    super(messageFor(error));
    this.name = "StorageUnavailableError";
  }
}

export function currentReplayLibraryOwner(): ReplayLibraryOwner {
  const auth = cloudAuthStore.getState();
  if (auth.status === "signed_in" && auth.user?.uid) {
    const owner = `user:${auth.user.uid}`;
    assertOwner(owner);
    return owner;
  }
  return "guest";
}

export function createReplayLibraryStore(
  options: ReplayLibraryStoreOptions = {},
): StoreApi<ReplayLibraryState> {
  let storage: BrowserStorage | null = null;
  let initialV2Raw: string | null = null;
  let initialLegacyRaw: string | null = null;
  let readOnly = false;
  let storageError: string | null = null;
  let sessionOnly = false;
  const lockManager = options.lockManager === undefined ? getBrowserLockManager() : options.lockManager;
  const ownerEpochs = new Map<ReplayLibraryOwner, number>();
  const ownerEpoch = (owner: ReplayLibraryOwner) => ownerEpochs.get(owner) ?? 0;
  const advanceOwnerEpoch = (owner: ReplayLibraryOwner) => ownerEpochs.set(owner, ownerEpoch(owner) + 1);

  const backing = (): BrowserStorage => options.storage ?? localStorage;
  const unavailable = (error?: unknown): void => {
    storageError = messageFor(error);
    sessionOnly = true;
    options.onPersistenceUnavailable?.(error);
  };

  let initialOwners: Record<string, ReplayLibraryOwnerState> = Object.create(null) as Record<string, ReplayLibraryOwnerState>;
  let initialRevision = 0;
  let initialReadSucceeded = false;
  try {
    storage = backing();
    initialV2Raw = storage.getItem(REPLAY_LIBRARY_STORAGE_KEY);
    initialLegacyRaw = storage.getItem(REPLAY_LIBRARY_LEGACY_STORAGE_KEY);
    initialReadSucceeded = true;
    if (initialV2Raw !== null) {
      const parsed = parseV2(initialV2Raw);
      initialRevision = parsed.revision;
      for (const [owner, value] of Object.entries(parsed.owners)) initialOwners[owner] = ownerStateFromPersisted(value);
    } else {
      const legacyOwners = parseLegacy(initialLegacyRaw);
      for (const [owner, value] of Object.entries(legacyOwners)) initialOwners[owner] = ownerStateFromPersisted(value);
    }
  } catch (error) {
    const text = messageFor(error);
    // Unavailable storage can support explicit session-only work. A readable
    // but malformed/future cache is preserved and blocks all writes.
    if (storage === null || !initialReadSucceeded) {
      unavailable(error);
    } else {
      readOnly = true;
      storageError = text;
    }
  }

  if (!lockManager && !readOnly && !storageError) {
    sessionOnly = true;
    storageError = "Cross-tab locking is unavailable. New replay-library changes will be session-only.";
  }

  const writeSessionMutation = (
    set: (partial: Partial<ReplayLibraryState>) => void,
    get: () => ReplayLibraryState,
    owner: ReplayLibraryOwner,
    updated: NormalizedChange,
    preflightOnly: boolean,
  ): void => {
    if (!updated.changed) return;
    const revision = get().revision + 1;
    const owners = { ...updated.owners };
    const ownerState = owners[owner];
    if (ownerState) owners[owner] = {
      ...ownerState,
      durable: false,
      error: storageError ?? "Session-only: replay-library changes cannot be saved to device storage.",
    };
    serializeLibrary(owners, revision);
    if (preflightOnly) return;
    const error = storageError ?? "Session-only: replay-library changes cannot be saved to device storage.";
    set({ owners, revision, sessionOnly: true, storageError: error });
  };

  async function commitMutation<T>(
    set: (partial: Partial<ReplayLibraryState>) => void,
    get: () => ReplayLibraryState,
    owner: ReplayLibraryOwner,
    change: (owners: Record<string, ReplayLibraryOwnerState>) => { owners: Record<string, ReplayLibraryOwnerState>; changed: boolean; result: T },
    optionsForCommit: { expectedRevision?: number; canReplace?: () => boolean; guard?: () => void; preflightOnly?: boolean } = {},
  ): Promise<T> {
    assertOwner(owner);
    if (get().readOnly) throw new Error(get().storageError ?? "Replay library storage requires recovery before changes can be saved.");
    const checkGuards = (): void => {
      optionsForCommit.guard?.();
      if (optionsForCommit.expectedRevision !== undefined && get().revision !== optionsForCommit.expectedRevision) {
        throw new Error("Replay library changed after review. Reload the current copies and review again.");
      }
      if (optionsForCommit.canReplace && !optionsForCommit.canReplace()) {
        throw new Error("Replay library replacement was superseded. Review the current account library before retrying.");
      }
    };
    checkGuards();
    const preflightOnly = optionsForCommit.preflightOnly ?? false;
    const applyInMemory = () => {
      const result = change(get().owners);
      writeSessionMutation(set, get, owner, result, preflightOnly);
      return result.result;
    };
    if (get().sessionOnly) return applyInMemory();

    if (!lockManager) {
      if (!preflightOnly) {
        storageError = "Cross-tab locking is unavailable. New replay-library changes will be session-only.";
        sessionOnly = true;
        set({ sessionOnly: true, storageError });
      }
      checkGuards();
      return applyInMemory();
    }

    let callbackError: unknown;
    try {
      return await lockManager.request(REPLAY_LIBRARY_LOCK_NAME, { mode: "exclusive" }, async () => {
        try {
          checkGuards();
          const current = get();
          if (current.sessionOnly || current.readOnly) throw new Error(current.storageError ?? "Replay library storage is not writable.");
          const target = storage ?? backing();
          let rawV2: string | null;
          let rawLegacy: string | null;
          try {
            rawV2 = target.getItem(REPLAY_LIBRARY_STORAGE_KEY);
            rawLegacy = target.getItem(REPLAY_LIBRARY_LEGACY_STORAGE_KEY);
          } catch (error) {
            throw new StorageUnavailableError(error);
          }
          if (rawV2 !== initialV2Raw) {
            throw new Error("Replay library changed in another tab. Refresh this library and review again.");
          }
          if (rawV2 === null && rawLegacy !== initialLegacyRaw) {
            throw new Error("Legacy replay-library data changed in another tab. Refresh before saving.");
          }
          let persisted: PersistedReplayLibrary;
          let ownersForMutation: Record<string, ReplayLibraryOwnerState> | null = null;
          if (rawV2 !== null) {
            persisted = parseV2(rawV2);
            if (persisted.revision !== current.revision) {
              throw new Error("Replay library revision changed in another tab. Refresh this library and review again.");
            }
          } else {
            const legacyOwners = parseLegacy(rawLegacy);
            const verifiedLegacy: Record<string, PersistedOwnerState> = Object.create(null) as Record<string, PersistedOwnerState>;
            for (const [legacyOwner, value] of Object.entries(legacyOwners)) {
              verifiedLegacy[legacyOwner] = {
                ...value,
                entries: await verifyReplayLibraryEntries(value.entries),
              };
            }
            checkGuards();
            // Copy the validated v1 owners into the in-memory view before the
            // caller's mutation is applied so no account bucket is omitted.
            const copiedOwners: Record<string, ReplayLibraryOwnerState> = Object.create(null) as Record<string, ReplayLibraryOwnerState>;
            for (const [legacyOwner, value] of Object.entries(verifiedLegacy)) {
              copiedOwners[legacyOwner] = ownerStateFromPersisted(value);
            }
            const currentOwners = Object.keys(current.owners).length === 0 ? copiedOwners : { ...current.owners };
            if (Object.keys(current.owners).length > 0) {
              for (const [legacyOwner, value] of Object.entries(copiedOwners)) {
                if (!currentOwners[legacyOwner]) currentOwners[legacyOwner] = value;
              }
            }
            ownersForMutation = currentOwners;
            persisted = { schema_version: REPLAY_LIBRARY_SCHEMA_VERSION, revision: 0, owners: verifiedLegacy };
          }
          checkGuards();
          const transformed = change(ownersForMutation ?? get().owners);
          if (!transformed.changed) return transformed.result;
          const revision = (persisted.revision || 0) + 1;
          const owners = { ...transformed.owners };
          for (const [key, value] of Object.entries(owners)) owners[key] = { ...value, durable: true, error: null };
          const serialized = serializeLibrary(owners, revision);
          if (preflightOnly) return transformed.result;
          try {
            target.setItem(REPLAY_LIBRARY_STORAGE_KEY, serialized);
          } catch (error) {
            throw new StorageUnavailableError(error);
          }
          if (rawV2 === null && rawLegacy !== null) {
            try {
              if (target.getItem(REPLAY_LIBRARY_LEGACY_STORAGE_KEY) === rawLegacy) {
                target.removeItem(REPLAY_LIBRARY_LEGACY_STORAGE_KEY);
              }
            } catch (error) {
              storageError = `Replay library is saved in v2, but retired v1 data could not be removed: ${messageFor(error)}`;
            }
          }
          initialV2Raw = serialized;
          try { initialLegacyRaw = target.getItem(REPLAY_LIBRARY_LEGACY_STORAGE_KEY); } catch {
            initialLegacyRaw = rawLegacy;
            storageError = "Replay library is saved in v2, but retired v1 data could not be checked.";
          }
          set({ owners, revision, sessionOnly: false, readOnly: false, storageError });
          return transformed.result;
        } catch (error) {
          callbackError = error;
          throw error;
        }
      });
    } catch (error) {
      if (callbackError !== undefined && !(callbackError instanceof StorageUnavailableError)) throw wrapMutationError(callbackError);
      // A denied or broken lock cannot safely authorize a durable write. Keep
      // this edit in memory and label it session-only instead.
      if (!preflightOnly) {
        unavailable(callbackError instanceof StorageUnavailableError ? callbackError : error);
        set({ sessionOnly: true, storageError });
      }
      checkGuards();
      return applyInMemory();
    }
  }

  return createStore<ReplayLibraryState>((set, get) => ({
    owners: initialOwners,
    revision: initialRevision,
    sessionOnly,
    readOnly,
    storageError,
    refreshFromStorage: async () => {
      if (!lockManager) return false;
      const expectedRevision = get().revision;
      const hasPendingLocalWork = Object.entries(get().owners).some(([owner, bucket]) =>
        (owner !== "guest" && bucket.dirty) || bucket.restoreReview || !bucket.durable);
      if (hasPendingLocalWork) return false;
      let invalidSnapshot = false;
      try {
        return await lockManager.request(REPLAY_LIBRARY_LOCK_NAME, { mode: "exclusive" }, async () => {
          if (get().revision !== expectedRevision) return false;
          const target = storage ?? backing();
          const rawV2 = target.getItem(REPLAY_LIBRARY_STORAGE_KEY);
          const rawLegacy = target.getItem(REPLAY_LIBRARY_LEGACY_STORAGE_KEY);
          let revision = 0;
          const owners: Record<string, ReplayLibraryOwnerState> = Object.create(null) as Record<string, ReplayLibraryOwnerState>;
          try {
            const snapshot = rawV2 !== null ? parseV2(rawV2) : { revision: 0, owners: parseLegacy(rawLegacy) };
            revision = snapshot.revision;
            for (const [owner, value] of Object.entries(snapshot.owners)) owners[owner] = ownerStateFromPersisted(value);
          } catch (error) {
            invalidSnapshot = true;
            readOnly = true;
            storageError = messageFor(error);
            set({ readOnly: true, storageError });
            throw error;
          }
          storage = target;
          initialV2Raw = rawV2;
          initialLegacyRaw = rawLegacy;
          sessionOnly = false;
          readOnly = false;
          storageError = null;
          set({ owners, revision, sessionOnly: false, readOnly: false, storageError: null });
          return true;
        });
      } catch (error) {
        if (invalidSnapshot) throw error;
        unavailable(error);
        set({ sessionOnly: true, storageError });
        return false;
      }
    },
    clearOwner: async (owner) => {
      advanceOwnerEpoch(owner);
      await commitMutation(set, get, owner, (owners) => {
        const current = owners[owner] ?? EMPTY_OWNER_STATE;
        return { owners: { ...owners, [owner]: { ...current, dirty: true, entries: [], error: null } }, changed: true, result: undefined };
      });
    },
    markDirty: async (owner) => {
      await commitMutation(set, get, owner, (owners) => {
        const current = owners[owner] ?? EMPTY_OWNER_STATE;
        if (current.dirty) return { owners, changed: false, result: undefined };
        return { owners: { ...owners, [owner]: { ...current, dirty: true, error: null } }, changed: true, result: undefined };
      });
    },
    markSynced: async (owner, baseRevision, expectedRevision, canReplace) => {
      assertBaseRevision(baseRevision, owner);
      await commitMutation(set, get, owner, (owners) => {
        const current = owners[owner] ?? EMPTY_OWNER_STATE;
        if (!current.dirty && current.baseRevision === baseRevision && !current.restoreReview) {
          return { owners, changed: false, result: undefined };
        }
        return {
          owners: { ...owners, [owner]: { ...current, baseRevision, dirty: false, restoreReview: false, error: null } },
          changed: true,
          result: undefined,
        };
      }, { expectedRevision, canReplace });
    },
    markRestoreReview: async (owner, restoreReview = true) => {
      await commitMutation(set, get, owner, (owners) => {
        const current = owners[owner] ?? EMPTY_OWNER_STATE;
        if (current.restoreReview === restoreReview) return { owners, changed: false, result: undefined };
        return { owners: { ...owners, [owner]: { ...current, restoreReview, error: null } }, changed: true, result: undefined };
      });
    },
    removeReplay: async (owner, fingerprint) => {
      assertOwner(owner);
      if (!FINGERPRINT_PATTERN.test(fingerprint)) throw new Error("Replay fingerprint is invalid.");
      await commitMutation(set, get, owner, (owners) => {
        const current = owners[owner] ?? EMPTY_OWNER_STATE;
        const entries = current.entries.filter((entry) => entry.fingerprint !== fingerprint);
        if (entries.length === current.entries.length) return { owners, changed: false, result: undefined };
        return {
          owners: { ...owners, [owner]: { ...current, dirty: true, entries, error: null } },
          changed: true,
          result: undefined,
        };
      });
    },
    replaceEntries: async (owner, entries, canReplace, replaceOptions = {}) => {
      assertOwner(owner);
      const startingRevision = get().revision;
      const expectedRevision = replaceOptions.expectedRevision ?? startingRevision;
      const startingEpoch = ownerEpoch(owner);
      try {
        const verified = await verifyReplayLibraryEntries(entries);
        if (ownerEpoch(owner) !== startingEpoch) throw new Error("Replay library was cleared or replaced while this snapshot was being validated; retry after review.");
        if (!replaceOptions.preflightOnly) advanceOwnerEpoch(owner);
        await commitMutation(set, get, owner, (owners) => {
          const current = owners[owner] ?? EMPTY_OWNER_STATE;
          const next: ReplayLibraryOwnerState = {
            ...current,
            baseRevision: replaceOptions.baseRevision === undefined ? current.baseRevision : replaceOptions.baseRevision,
            dirty: replaceOptions.dirty ?? false,
            entries: verified,
            error: null,
            restoreReview: replaceOptions.restoreReview ?? current.restoreReview,
          };
          const changed = JSON.stringify({ ...current, error: null }) !== JSON.stringify({ ...next, durable: current.durable });
          return { owners: { ...owners, [owner]: next }, changed, result: undefined };
        }, { expectedRevision, canReplace, preflightOnly: replaceOptions.preflightOnly });
      } catch (error) {
        if (!replaceOptions.preflightOnly) {
          const current = get().owners[owner] ?? EMPTY_OWNER_STATE;
          set({ owners: { ...get().owners, [owner]: { ...current, error: messageFor(error) } } });
        }
        throw error;
      }
    },
    saveReplay: async (owner, value, kind, guard) => {
      assertOwner(owner);
      const startingEpoch = ownerEpoch(owner);
      try {
        if (kind !== "saved" && kind !== "practice") throw new Error("Replay library kind must be saved or practice.");
        const validated = validatePortableReplay(value);
        if (kind === "practice" && !validated.replay.practice) {
          throw new Error("A practice replay must include its source fingerprint and inherited ply.");
        }
        const { viewPly: _viewPly, ...withoutViewPly } = validated.replay;
        const replay = withoutViewPly as PortableReplayV1;
        const payload = encodePortableReplay(replay);
        const fingerprint = await portableReplayFingerprint({ ...validated, replay });
        const entry = validateEntry({
          fingerprint,
          payload,
          kind,
          saved_at: (options.now ?? (() => new Date().toISOString()))(),
          metadata: DEFAULT_PRIVATE_REPLAY_METADATA,
        }, 0, false);
        await commitMutation(set, get, owner, (owners) => {
          if (ownerEpoch(owner) !== startingEpoch) {
            throw new Error("Replay library was cleared or replaced while this save was being validated; retry the save.");
          }
          const current = owners[owner] ?? EMPTY_OWNER_STATE;
          const existing = current.entries.find((candidate) => candidate.fingerprint === fingerprint);
          if (existing) {
            if (existing.payload !== entry.payload || existing.kind !== entry.kind) {
              throw new Error("Replay fingerprint already exists with a different payload or library kind.");
            }
            // Preserve metadata and original save time for duplicates.
            return { owners, changed: false, result: fingerprint };
          }
          if (current.entries.length >= REPLAY_LIBRARY_LIMIT) {
            throw new Error(`Replay library is full (${REPLAY_LIBRARY_LIMIT} entries); remove a replay before saving another.`);
          }
          return {
            owners: { ...owners, [owner]: { ...current, dirty: true, entries: [entry, ...current.entries], error: null } },
            changed: true,
            result: fingerprint,
          };
        }, { guard });
        return fingerprint;
      } catch (error) {
        const current = get().owners[owner] ?? EMPTY_OWNER_STATE;
        set({ owners: { ...get().owners, [owner]: { ...current, error: messageFor(error) } } });
        throw error;
      }
    },
    updateReplayMetadata: async (owner, fingerprint, value, expectedMetadata, guard) => {
      assertOwner(owner);
      if (!FINGERPRINT_PATTERN.test(fingerprint)) throw new Error("Replay fingerprint is invalid.");
      try {
        const before = get().owners[owner]?.entries.find((entry) => entry.fingerprint === fingerprint);
        if (!before) throw new Error("This saved replay is no longer in the library.");
        const moveCount = decodePortableReplay(before.payload).replay.moves.length;
        const metadata = validatePrivateReplayMetadata(value, moveCount);
        const expected = expectedMetadata === undefined ? undefined : validatePrivateReplayMetadata(expectedMetadata, moveCount);
        const expectedRevision = get().revision;
        await commitMutation(set, get, owner, (owners) => {
          const current = owners[owner] ?? EMPTY_OWNER_STATE;
          const index = current.entries.findIndex((entry) => entry.fingerprint === fingerprint);
          if (index < 0) throw new Error("This saved replay is no longer in the library.");
          const original = current.entries[index]!;
          if (expected && JSON.stringify(original.metadata) !== JSON.stringify(expected)) {
            throw new Error("Replay details changed since this editor opened. Reopen the details and review the current copy.");
          }
          if (JSON.stringify(original.metadata) === JSON.stringify(metadata)) {
            return { owners, changed: false, result: undefined };
          }
          const entries = [...current.entries];
          entries[index] = { ...original, metadata };
          return {
            owners: { ...owners, [owner]: { ...current, dirty: true, entries, error: null } },
            changed: true,
            result: undefined,
          };
        }, { expectedRevision, guard });
      } catch (error) {
        const current = get().owners[owner] ?? EMPTY_OWNER_STATE;
        set({ owners: { ...get().owners, [owner]: { ...current, error: messageFor(error) } } });
        throw error;
      }
    },
  }));
}

export const replayLibraryStore = createReplayLibraryStore();
