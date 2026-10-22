import { createStore, type StoreApi } from "zustand/vanilla";

import type { PrivateLibraryEntry } from "../cloud/sharing_private";
import { cloudAuthStore } from "../cloud/auth_store";
import { bestEffortStorage, type BrowserStorage } from "../profile/best_effort_storage";
import {
  decodePortableReplay,
  encodePortableReplay,
  portableReplayFingerprint,
} from "./portable_replay_codec";
import { validatePortableReplay, type PortableReplayV1 } from "./portable_replay";

export const REPLAY_LIBRARY_LIMIT = 64;
export const REPLAY_LIBRARY_STORAGE_KEY = "gomoku2d.replay-library.v1";
export const REPLAY_LIBRARY_SCHEMA_VERSION = 1;

// Keep the local envelope within the same bounded byte budget as a cloud
// private-library snapshot without importing the cloud implementation.
const REPLAY_LIBRARY_STORAGE_BYTES_LIMIT = 600_000;
const FINGERPRINT_PATTERN = /^[A-Za-z0-9_-]{43}$/;
const PAYLOAD_PATTERN = /^[A-Za-z0-9_-]{1,8192}$/;
const CONTROL_CHARACTER_PATTERN = /[\u0000-\u001f\u007f-\u009f]/u;

export type ReplayLibraryOwner = "guest" | `user:${string}`;
export type ReplayLibraryKind = PrivateLibraryEntry["kind"];

export interface ReplayLibraryOwnerState {
  dirty: boolean;
  entries: PrivateLibraryEntry[];
  error: string | null;
}

export interface ReplayLibraryState {
  owners: Record<string, ReplayLibraryOwnerState>;
  storageError: string | null;
  clearOwner: (owner: ReplayLibraryOwner) => Promise<void>;
  markDirty: (owner: ReplayLibraryOwner) => void;
  markSynced: (owner: ReplayLibraryOwner) => void;
  removeReplay: (owner: ReplayLibraryOwner, fingerprint: string) => Promise<void>;
  replaceEntries: (owner: ReplayLibraryOwner, entries: unknown) => Promise<void>;
  saveReplay: (owner: ReplayLibraryOwner, replay: unknown, kind: ReplayLibraryKind) => Promise<string>;
}

export interface ReplayLibraryStoreOptions {
  onPersistenceUnavailable?: (error?: unknown) => void;
  now?: () => string;
  storage?: BrowserStorage;
}

interface PersistedOwnerState {
  dirty: boolean;
  entries: PrivateLibraryEntry[];
}

interface PersistedReplayLibrary {
  schema_version: typeof REPLAY_LIBRARY_SCHEMA_VERSION;
  owners: Record<string, PersistedOwnerState>;
}

const EMPTY_OWNER_STATE: ReplayLibraryOwnerState = {
  dirty: false,
  entries: [],
  error: null,
};

function isPlainRecord(value: unknown): value is Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return false;
  }
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function hasExactKeys(record: Record<string, unknown>, keys: readonly string[]): boolean {
  const actual = Object.keys(record).sort();
  const expected = [...keys].sort();
  return actual.length === expected.length && actual.every((key, index) => key === expected[index]);
}

function messageFor(error: unknown): string {
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string" && message.trim()) return message;
  }
  return "Replay library storage is unavailable.";
}

function assertOwner(value: string): asserts value is ReplayLibraryOwner {
  if (value === "guest") {
    return;
  }
  if (!value.startsWith("user:")) {
    throw new Error("Replay library owner must be guest or a namespaced user UID.");
  }
  const uid = value.slice("user:".length);
  if (!uid || Array.from(uid).length > 128 || CONTROL_CHARACTER_PATTERN.test(uid)) {
    throw new Error("Replay library user owner must include a bounded UID.");
  }
}

function validateEntry(value: unknown, index: number): PrivateLibraryEntry {
  if (!isPlainRecord(value) || !hasExactKeys(value, ["fingerprint", "payload", "kind", "saved_at"])) {
    throw new Error(`Replay library entry ${index + 1} has an invalid shape.`);
  }
  if (typeof value.fingerprint !== "string" || !FINGERPRINT_PATTERN.test(value.fingerprint)) {
    throw new Error(`Replay library entry ${index + 1} has an invalid fingerprint.`);
  }
  if (typeof value.payload !== "string" || !PAYLOAD_PATTERN.test(value.payload)) {
    throw new Error(`Replay library entry ${index + 1} has an invalid payload.`);
  }
  if (value.kind !== "saved" && value.kind !== "practice") {
    throw new Error(`Replay library entry ${index + 1} has an invalid kind.`);
  }
  if (
    typeof value.saved_at !== "string"
    || !/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{3}Z$/.test(value.saved_at)
    || !Number.isFinite(Date.parse(value.saved_at))
    || new Date(value.saved_at).toISOString() !== value.saved_at
  ) {
    throw new Error(`Replay library entry ${index + 1} has an invalid saved_at timestamp.`);
  }
  return {
    fingerprint: value.fingerprint,
    payload: value.payload,
    kind: value.kind,
    saved_at: value.saved_at,
  };
}

export function validateReplayLibraryEntries(value: unknown): PrivateLibraryEntry[] {
  if (!Array.isArray(value) || value.length > REPLAY_LIBRARY_LIMIT) {
    throw new Error(`A replay library can contain at most ${REPLAY_LIBRARY_LIMIT} entries.`);
  }
  const seen = new Set<string>();
  return value.map((entry, index) => {
    const validated = validateEntry(entry, index);
    if (seen.has(validated.fingerprint)) {
      throw new Error("Replay library entries cannot repeat a fingerprint.");
    }
    const decoded = decodePortableReplay(validated.payload);
    if (validated.kind === "practice" && !decoded.replay.practice) {
      throw new Error(`Replay library practice entry ${index + 1} is missing its practice origin.`);
    }
    seen.add(validated.fingerprint);
    return validated;
  });
}

async function verifyPortableEntries(value: unknown): Promise<PrivateLibraryEntry[]> {
  const entries = validateReplayLibraryEntries(value);
  const verified: PrivateLibraryEntry[] = [];
  for (const entry of entries) {
    const decoded = decodePortableReplay(entry.payload);
    const fingerprint = await portableReplayFingerprint(decoded);
    if (fingerprint !== entry.fingerprint) {
      throw new Error(`Replay library payload does not match fingerprint ${entry.fingerprint}.`);
    }
    const { viewPly: _viewPly, ...withoutViewPly } = decoded.replay;
    verified.push({ ...entry, payload: encodePortableReplay(withoutViewPly) });
  }
  return verified;
}

function persistedFromRaw(raw: string | null): PersistedReplayLibrary {
  if (raw === null) {
    return { schema_version: REPLAY_LIBRARY_SCHEMA_VERSION, owners: {} };
  }
  if (new TextEncoder().encode(raw).byteLength > REPLAY_LIBRARY_STORAGE_BYTES_LIMIT) {
    throw new Error(`Replay library cache exceeds ${REPLAY_LIBRARY_STORAGE_BYTES_LIMIT} bytes.`);
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    throw new Error("Replay library cache is not valid JSON.");
  }
  if (!isPlainRecord(parsed) || !hasExactKeys(parsed, ["schema_version", "owners"])) {
    throw new Error("Replay library cache has an unsupported shape.");
  }
  if (parsed.schema_version !== REPLAY_LIBRARY_SCHEMA_VERSION) {
    throw new Error(`Replay library cache uses unsupported schema version ${String(parsed.schema_version)}.`);
  }
  if (!isPlainRecord(parsed.owners)) {
    throw new Error("Replay library cache owners must be an object.");
  }

  const owners: Record<string, PersistedOwnerState> = Object.create(null) as Record<string, PersistedOwnerState>;
  for (const [owner, value] of Object.entries(parsed.owners)) {
    assertOwner(owner);
    if (!isPlainRecord(value) || !hasExactKeys(value, ["entries", "dirty"]) || typeof value.dirty !== "boolean") {
      throw new Error(`Replay library cache for ${owner} has an invalid shape.`);
    }
    owners[owner] = { dirty: value.dirty, entries: validateReplayLibraryEntries(value.entries) };
  }
  return { schema_version: REPLAY_LIBRARY_SCHEMA_VERSION, owners };
}

function serializedSize(value: string): number {
  return new TextEncoder().encode(value).byteLength;
}

function ownerStateFromPersisted(value: PersistedOwnerState): ReplayLibraryOwnerState {
  return { dirty: value.dirty, entries: value.entries, error: null };
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
  const ownerEpochs = new Map<ReplayLibraryOwner, number>();
  const ownerEpoch = (owner: ReplayLibraryOwner): number => ownerEpochs.get(owner) ?? 0;
  const advanceOwnerEpoch = (owner: ReplayLibraryOwner): void => {
    ownerEpochs.set(owner, ownerEpoch(owner) + 1);
  };
  let observedRaw: string | null | undefined;
  const persistedRef: { value: PersistedReplayLibrary | null } = { value: null };
  let storageError: string | null = null;
  const backing = (): BrowserStorage => options.storage ?? localStorage;
  const unavailable = (error?: unknown): void => {
    storageError = messageFor(error);
    options.onPersistenceUnavailable?.(error);
  };

  const guardedStorage: BrowserStorage = {
    getItem: (name) => {
      const raw = backing().getItem(name);
      if (name === REPLAY_LIBRARY_STORAGE_KEY) {
        observedRaw = raw;
        if (raw === null) {
          persistedRef.value = { schema_version: REPLAY_LIBRARY_SCHEMA_VERSION, owners: {} };
        } else {
          persistedRef.value = persistedFromRaw(raw);
        }
      }
      return raw;
    },
    removeItem: (name) => {
      const target = backing();
      if (name === REPLAY_LIBRARY_STORAGE_KEY && observedRaw !== undefined && target.getItem(name) !== observedRaw) {
        throw new Error("Replay library changed in another tab.");
      }
      target.removeItem(name);
      if (name === REPLAY_LIBRARY_STORAGE_KEY) {
        observedRaw = null;
        persistedRef.value = { schema_version: REPLAY_LIBRARY_SCHEMA_VERSION, owners: {} };
      }
    },
    setItem: (name, value) => {
      const target = backing();
      if (name === REPLAY_LIBRARY_STORAGE_KEY && observedRaw !== undefined && target.getItem(name) !== observedRaw) {
        throw new Error("Replay library changed in another tab.");
      }
      if (name === REPLAY_LIBRARY_STORAGE_KEY) {
        // This wrapper is private to the store; persistOwners only sends its
        // already validated envelope, so avoid re-decoding every saved replay.
        persistedRef.value = null;
      }
      target.setItem(name, value);
      if (name === REPLAY_LIBRARY_STORAGE_KEY) observedRaw = value;
    },
  };
  const storage = bestEffortStorage(() => guardedStorage, unavailable);

  storage.getItem(REPLAY_LIBRARY_STORAGE_KEY);
  const initialOwners: Record<string, ReplayLibraryOwnerState> = Object.create(null) as Record<string, ReplayLibraryOwnerState>;
  for (const [owner, value] of Object.entries(persistedRef.value?.owners ?? {})) {
    initialOwners[owner] = ownerStateFromPersisted(value);
  }

  const persistOwners = (owners: Record<string, ReplayLibraryOwnerState>): string | null => {
    const persistedOwners: Record<string, PersistedOwnerState> = Object.create(null) as Record<string, PersistedOwnerState>;
    for (const [owner, state] of Object.entries(owners)) {
      persistedOwners[owner] = { dirty: state.dirty, entries: state.entries };
    }
    const serialized = JSON.stringify({ schema_version: REPLAY_LIBRARY_SCHEMA_VERSION, owners: persistedOwners });
    if (serializedSize(serialized) > REPLAY_LIBRARY_STORAGE_BYTES_LIMIT) {
      return `Replay library cache exceeds ${REPLAY_LIBRARY_STORAGE_BYTES_LIMIT} bytes.`;
    }
    storage.setItem(REPLAY_LIBRARY_STORAGE_KEY, serialized);
    return storageError;
  };

  const commitOwner = (
    set: (partial: Partial<ReplayLibraryState>) => void,
    get: () => ReplayLibraryState,
    owner: ReplayLibraryOwner,
    state: ReplayLibraryOwnerState,
    persist: boolean,
  ): void => {
    const owners = { ...get().owners, [owner]: state };
    const failure = persist ? persistOwners(owners) : null;
    const error = failure ?? (persist ? null : get().storageError);
    const committed = error && persist ? { ...state, error } : state;
    set({ owners: { ...owners, [owner]: committed }, storageError: error });
  };

  return createStore<ReplayLibraryState>((set, get) => ({
    owners: initialOwners,
    storageError,
    clearOwner: async (owner) => {
      assertOwner(owner);
      advanceOwnerEpoch(owner);
      commitOwner(set, get, owner, { dirty: true, entries: [], error: null }, true);
    },
    markDirty: (owner) => {
      assertOwner(owner);
      const current = get().owners[owner] ?? EMPTY_OWNER_STATE;
      commitOwner(set, get, owner, { ...current, dirty: true, error: null }, true);
    },
    markSynced: (owner) => {
      assertOwner(owner);
      const current = get().owners[owner] ?? EMPTY_OWNER_STATE;
      commitOwner(set, get, owner, { ...current, dirty: false, error: null }, true);
    },
    removeReplay: async (owner, fingerprint) => {
      assertOwner(owner);
      if (!FINGERPRINT_PATTERN.test(fingerprint)) {
        throw new Error("Replay fingerprint is invalid.");
      }
      const current = get().owners[owner] ?? EMPTY_OWNER_STATE;
      const entries = current.entries.filter((entry) => entry.fingerprint !== fingerprint);
      if (entries.length === current.entries.length) {
        return;
      }
      commitOwner(set, get, owner, { dirty: true, entries, error: null }, true);
    },
    replaceEntries: async (owner, entries) => {
      assertOwner(owner);
      const before = get().owners[owner]?.entries ?? EMPTY_OWNER_STATE.entries;
      try {
        const validated = await verifyPortableEntries(entries);
        const current = get().owners[owner] ?? EMPTY_OWNER_STATE;
        if (current.entries !== before) {
          throw new Error("Replay library changed while validating replacement; retry after reviewing the latest local copies.");
        }
        advanceOwnerEpoch(owner);
        commitOwner(set, get, owner, { dirty: false, entries: validated, error: null }, true);
      } catch (error) {
        const current = get().owners[owner] ?? EMPTY_OWNER_STATE;
        set({ owners: { ...get().owners, [owner]: { ...current, error: messageFor(error) } } });
        throw error;
      }
    },
    saveReplay: async (owner, value, kind) => {
      assertOwner(owner);
      const startingEpoch = ownerEpoch(owner);
      try {
        if (kind !== "saved" && kind !== "practice") {
          throw new Error("Replay library kind must be saved or practice.");
        }
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
        }, 0);
        if (ownerEpoch(owner) !== startingEpoch) {
          throw new Error("Replay library was cleared or replaced while this save was being validated; retry the save.");
        }
        const current = get().owners[owner] ?? EMPTY_OWNER_STATE;
        const existing = current.entries.find((candidate) => candidate.fingerprint === fingerprint);
        if (existing) {
          if (existing.payload !== entry.payload || existing.kind !== entry.kind) {
            throw new Error("Replay fingerprint already exists with a different payload or library kind.");
          }
          return fingerprint;
        }
        if (current.entries.length >= REPLAY_LIBRARY_LIMIT) {
          throw new Error(`Replay library is full (${REPLAY_LIBRARY_LIMIT} entries); remove a replay before saving another.`);
        }
        commitOwner(set, get, owner, {
          dirty: true,
          entries: [entry, ...current.entries],
          error: null,
        }, true);
        return fingerprint;
      } catch (error) {
        const current = get().owners[owner] ?? EMPTY_OWNER_STATE;
        set({ owners: { ...get().owners, [owner]: { ...current, error: messageFor(error) } } });
        throw error;
      }
    },
  }));
}

export const replayLibraryStore = createReplayLibraryStore();
