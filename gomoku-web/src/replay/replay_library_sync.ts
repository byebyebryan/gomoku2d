import { createStore } from "zustand/vanilla";

import { cloudAuthStore } from "../cloud/auth_store";
import { loadPrivateLibrary, savePrivateLibrary, type PrivateLibraryEntry, type PrivateLibrarySnapshot } from "../cloud/sharing_private";
import { SharingConflictError } from "../cloud/sharing_common";
import { replayLibraryStore, type ReplayLibraryState } from "./replay_library_store";
import type { StoreApi } from "zustand/vanilla";

export interface LibrarySyncUser {
  loaded: boolean;
  revision: number | null;
  status: "idle" | "loading" | "pending" | "syncing" | "synced" | "conflict" | "error";
  error: string | null;
  hasIncoming: boolean;
  incoming: PrivateLibrarySnapshot | null;
}

export const EMPTY_LIBRARY_SYNC: LibrarySyncUser = {
  loaded: false, revision: null, status: "idle", error: null, hasIncoming: false, incoming: null,
};
const EMPTY_ENTRIES: PrivateLibraryEntry[] = [];

function signedInUid() {
  const auth = cloudAuthStore.getState();
  return auth.status === "signed_in" ? auth.user?.uid ?? null : null;
}

export function createReplayLibrarySync(options: {
  local?: StoreApi<ReplayLibraryState>;
  currentUid?: () => string | null;
  load?: typeof loadPrivateLibrary;
  save?: typeof savePrivateLibrary;
} = {}) {
  const local = options.local ?? replayLibraryStore;
  const currentUid = options.currentUid ?? signedInUid;
  const load = options.load ?? loadPrivateLibrary;
  const save = options.save ?? savePrivateLibrary;
  const store = createStore<{ users: Record<string, LibrarySyncUser> }>(() => ({ users: {} }));
  const requests = new Map<string, number>();
  const lifecycles = new Map<string, number>();
  const lifecycleVersion = (uid: string) => lifecycles.get(uid) ?? 0;
  const state = (uid: string) => store.getState().users[uid] ?? EMPTY_LIBRARY_SYNC;
  const patch = (uid: string, value: Partial<LibrarySyncUser>) => store.setState((current) => ({
    users: { ...current.users, [uid]: { ...state(uid), ...value } },
  }));
  const checkAccount = (uid: string) => {
    if (currentUid() !== uid) throw new Error("Account changed. Open the library for the current account and try again.");
  };
  const owner = (uid: string) => `user:${uid}` as const;
  const entries = (uid: string) => local.getState().owners[owner(uid)]?.entries ?? EMPTY_ENTRIES;
  const dirty = (uid: string) => local.getState().owners[owner(uid)]?.dirty ?? false;
  const message = (error: unknown) => error instanceof Error ? error.message : "Replay library could not sync.";

  async function reload(uid: string): Promise<void> {
    checkAccount(uid);
    const request = (requests.get(uid) ?? 0) + 1;
    requests.set(uid, request);
    const before = entries(uid);
    const previous = state(uid);
    patch(uid, { status: "loading", error: null });
    try {
      const incoming = await load(uid);
      checkAccount(uid);
      if (requests.get(uid) !== request) return;
      if (dirty(uid) || entries(uid) !== before) {
        if (previous.loaded && previous.status !== "conflict" && !previous.hasIncoming
          && previous.revision === (incoming?.revision ?? null)) {
          patch(uid, { status: "pending", hasIncoming: false, incoming: null });
          return;
        }
        patch(uid, { status: "conflict", hasIncoming: true, incoming,
          error: "Cloud and local copies need review. Choose the cloud library or explicitly merge copies." });
        return;
      }
      await local.getState().replaceEntries(owner(uid), incoming?.entries ?? [],
        () => currentUid() === uid && requests.get(uid) === request);
      checkAccount(uid);
      patch(uid, { loaded: true, revision: incoming?.revision ?? null, status: "synced",
        hasIncoming: false, incoming: null, error: null });
    } catch (error) {
      if (currentUid() === uid && requests.get(uid) === request) patch(uid, { status: "error", error: message(error) });
      throw error;
    }
  }

  async function sync(uid: string): Promise<void> {
    checkAccount(uid);
    if (!state(uid).loaded) await reload(uid);
    checkAccount(uid);
    if (state(uid).status === "conflict") throw new SharingConflictError(state(uid).error ?? undefined);
    if (!dirty(uid)) return;
    if (state(uid).status === "syncing") throw new Error("Replay library sync is already in progress.");
    const outgoing = entries(uid);
    const request = requests.get(uid);
    patch(uid, { status: "syncing", error: null });
    try {
      const saved = await save(uid, outgoing, state(uid).revision);
      checkAccount(uid);
      if (requests.get(uid) !== request) return;
      const unchanged = entries(uid) === outgoing;
      if (unchanged) local.getState().markSynced(owner(uid));
      patch(uid, { loaded: true, revision: saved.revision, status: unchanged ? "synced" : "pending",
        error: null, hasIncoming: false, incoming: null });
    } catch (error) {
      if (currentUid() === uid) patch(uid, { status: error instanceof SharingConflictError ? "conflict" : "error",
        error: message(error) });
      throw error;
    }
  }

  async function useCloud(uid: string): Promise<void> {
    checkAccount(uid);
    const current = state(uid);
    if (!current.hasIncoming) throw new Error("Reload the cloud library before choosing its version.");
    const request = requests.get(uid);
    await local.getState().replaceEntries(owner(uid), current.incoming?.entries ?? [],
      () => currentUid() === uid && requests.get(uid) === request && state(uid) === current);
    checkAccount(uid);
    patch(uid, { loaded: true, revision: current.incoming?.revision ?? null, status: "synced",
      error: null, hasIncoming: false, incoming: null });
  }

  async function mergeCopies(uid: string): Promise<void> {
    checkAccount(uid);
    const current = state(uid);
    if (!current.hasIncoming) throw new Error("Reload and review the cloud library before merging.");
    const before = entries(uid);
    const merged = new Map((current.incoming?.entries ?? []).map((entry) => [entry.fingerprint, entry]));
    for (const entry of before) {
      const existing = merged.get(entry.fingerprint);
      if (existing && existing.payload !== entry.payload) throw new Error("Conflicting replay attribution needs a separate saved copy.");
      merged.set(entry.fingerprint, existing?.kind === "practice" ? existing : entry);
    }
    const outgoing = [...merged.values()];
    const request = requests.get(uid);
    patch(uid, { status: "syncing", error: null });
    try {
      const saved = await save(uid, outgoing, current.incoming?.revision ?? null);
      checkAccount(uid);
      if (requests.get(uid) !== request) return;
      const after = entries(uid);
      const changed = before !== after;
      const latest = new Map(saved.entries.map((entry) => [entry.fingerprint, entry]));
      if (changed) {
        const remaining = new Set(after.map((entry) => entry.fingerprint));
        for (const entry of before) if (!remaining.has(entry.fingerprint)) latest.delete(entry.fingerprint);
        const previous = new Map(before.map((entry) => [entry.fingerprint, entry]));
        for (const entry of after) if (previous.get(entry.fingerprint) !== entry) latest.set(entry.fingerprint, entry);
      }
      await local.getState().replaceEntries(owner(uid), [...latest.values()],
        () => currentUid() === uid && requests.get(uid) === request);
      checkAccount(uid);
      if (changed) local.getState().markDirty(owner(uid));
      patch(uid, { loaded: true, revision: saved.revision, status: changed ? "pending" : "synced", error: null,
        hasIncoming: false, incoming: null });
    } catch (error) {
      if (currentUid() === uid) patch(uid, { status: "conflict", error: message(error) });
      throw error;
    }
  }

  function acceptCleared(uid: string, snapshot: PrivateLibrarySnapshot) {
    lifecycles.set(uid, lifecycleVersion(uid) + 1);
    requests.set(uid, (requests.get(uid) ?? 0) + 1);
    const needsReview = dirty(uid) && entries(uid).length > 0;
    patch(uid, { loaded: true, revision: snapshot.revision, status: needsReview ? "conflict" : "synced",
      error: needsReview ? "Cloud library was cleared. Review local copies before explicitly merging them back." : null,
      hasIncoming: needsReview, incoming: needsReview ? snapshot : null });
  }

  return { store, reload, sync, useCloud, mergeCopies, acceptCleared, lifecycleVersion };
}

export const replayLibrarySync = createReplayLibrarySync();
