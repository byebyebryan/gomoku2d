import { createStore } from "zustand/vanilla";

import {
  loadPrivateLibrary,
  savePrivateLibrary,
  privateLibraryV2Json,
  type PrivateLibraryEntry,
  type PrivateLibrarySnapshot,
} from "../cloud/sharing_private";
import { nextRevision, SharingConflictError } from "../cloud/sharing_common";
import { cloudAuthStore, type CloudAuthUser } from "../cloud/auth_store";
import {
  replayLibraryStore,
  validateReplayLibraryEntries,
  verifyReplayLibraryEntries,
  type ReplayLibraryOwner,
  type ReplayLibraryState,
} from "./replay_library_store";
import type { StoreApi } from "zustand/vanilla";

export interface LibrarySyncUser {
  loaded: boolean;
  revision: number | null;
  status: "idle" | "loading" | "pending" | "syncing" | "synced" | "conflict" | "error";
  error: string | null;
  hasIncoming: boolean;
  incoming: PrivateLibrarySnapshot | null;
  requiresBaseVerification: boolean;
}

export interface ReplayLibraryMergeRow {
  fingerprint: string;
  status: "local-only" | "cloud-only" | "same" | "metadata-conflict";
  local: PrivateLibraryEntry | null;
  cloud: PrivateLibraryEntry | null;
}

export interface ReplayLibraryMergePlan {
  uid: string;
  owner: ReplayLibraryOwner;
  localRevision: number;
  cloudRevision: number | null;
  localEntries: PrivateLibraryEntry[];
  cloudEntries: PrivateLibraryEntry[];
  rows: ReplayLibraryMergeRow[];
}

export const EMPTY_LIBRARY_SYNC: LibrarySyncUser = {
  loaded: false,
  revision: null,
  status: "idle",
  error: null,
  hasIncoming: false,
  incoming: null,
  requiresBaseVerification: false,
};

const EMPTY_ENTRIES: PrivateLibraryEntry[] = [];

function signedInUid() {
  const auth = cloudAuthStore.getState();
  return auth.status === "signed_in" ? auth.user?.uid ?? null : null;
}

function sameMetadata(left: PrivateLibraryEntry, right: PrivateLibraryEntry): boolean {
  return JSON.stringify(left.metadata) === JSON.stringify(right.metadata);
}

function sameEntries(left: PrivateLibraryEntry[], right: PrivateLibraryEntry[]): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function cloneEntries(entries: PrivateLibraryEntry[]): PrivateLibraryEntry[] {
  return entries.map((entry) => ({ ...entry, metadata: { ...entry.metadata } }));
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
  const captureAuthUser = (uid: string): CloudAuthUser | null => {
    const auth = cloudAuthStore.getState();
    return auth.status === "signed_in" && auth.user?.uid === uid ? auth.user : null;
  };
  const checkAccount = (uid: string, authUser: CloudAuthUser | null) => {
    if (currentUid() !== uid) throw new Error("Account changed. Open the library for the current account and try again.");
    if (authUser && cloudAuthStore.getState().user !== authUser) {
      throw new Error("The signed-in session changed. Review the current account library before continuing.");
    }
  };
  const requestCurrent = (uid: string, request: number, authUser: CloudAuthUser | null) =>
    currentUid() === uid && requests.get(uid) === request && (!authUser || cloudAuthStore.getState().user === authUser);
  const owner = (uid: string): ReplayLibraryOwner => `user:${uid}`;
  const localOwner = (uid: string) => local.getState().owners[owner(uid)];
  const entries = (uid: string) => localOwner(uid)?.entries ?? EMPTY_ENTRIES;
  const dirty = (uid: string) => localOwner(uid)?.dirty ?? false;
  const localRevision = () => local.getState().revision;
  const message = (error: unknown) => error instanceof Error ? error.message : "Replay library could not sync.";
  const localUnavailable = (): string | null => {
    const state = local.getState();
    if (state.readOnly) return state.storageError ?? "Device replay-library storage needs recovery before sync.";
    return null;
  };
  const planAuthUsers = new WeakMap<ReplayLibraryMergePlan, CloudAuthUser | null>();

  async function reload(uid: string): Promise<void> {
    const authUser = captureAuthUser(uid);
    const check = () => checkAccount(uid, authUser);
    check();
    const request = (requests.get(uid) ?? 0) + 1;
    requests.set(uid, request);
    const before = entries(uid);
    const beforeRevision = localRevision();
    patch(uid, { status: "loading", error: null });
    try {
      const incoming = await load(uid);
      const verifiedIncomingEntries = await verifyReplayLibraryEntries(incoming?.entries ?? []);
      const verifiedIncoming = incoming ? { ...incoming, entries: verifiedIncomingEntries } : null;
      check();
      if (requests.get(uid) !== request) return;
      const currentBucket = localOwner(uid);
      if (localRevision() !== beforeRevision || entries(uid) !== before) {
        patch(uid, { status: "conflict", hasIncoming: true, incoming: verifiedIncoming,
          error: "Local copies changed while cloud data loaded. Review both copies before continuing." });
        return;
      }
      const cloudRevision = verifiedIncoming?.revision ?? null;
      const baseMatches = currentBucket?.baseRevision === cloudRevision;
      const hasLocalCopies = (currentBucket?.entries.length ?? 0) > 0;

      if (currentBucket?.dirty && baseMatches && !currentBucket.restoreReview) {
        patch(uid, { loaded: true, revision: cloudRevision, status: "pending", hasIncoming: false, incoming: null,
          error: null, requiresBaseVerification: false });
        return;
      }

      const unverifiedLegacyOrOfflineCopies = hasLocalCopies && currentBucket?.baseRevision === "unknown";
      const emptyCloudCleanupBoundary = hasLocalCopies && verifiedIncomingEntries.length === 0 && !baseMatches;
      if (currentBucket?.restoreReview || currentBucket?.dirty || unverifiedLegacyOrOfflineCopies || emptyCloudCleanupBoundary) {
        if (!currentBucket?.restoreReview && hasLocalCopies && verifiedIncomingEntries.length === 0) {
          try { await local.getState().markRestoreReview(owner(uid), true); } catch { /* conflict remains visible below */ }
          check();
          if (requests.get(uid) !== request) return;
        }
        patch(uid, { loaded: true, revision: cloudRevision, status: "conflict", hasIncoming: true, incoming: verifiedIncoming,
          error: currentBucket?.restoreReview || verifiedIncomingEntries.length === 0
            ? "Cloud is empty or was cleared after these local copies were saved. Review the local copies before discarding or restoring them."
            : "Cloud and local copies need review. Choose a complete version or resolve metadata conflicts.",
          requiresBaseVerification: false });
        return;
      }

      const expectedRevision = localRevision();
      await local.getState().replaceEntries(owner(uid), verifiedIncomingEntries,
        () => requestCurrent(uid, request, authUser),
        { baseRevision: cloudRevision, dirty: false, expectedRevision, restoreReview: false });
      check();
      if (requests.get(uid) !== request) return;
      patch(uid, { loaded: true, revision: cloudRevision, status: "synced", hasIncoming: false, incoming: null,
        error: null, requiresBaseVerification: false });
    } catch (error) {
      if (requestCurrent(uid, request, authUser)) patch(uid, { status: "error", error: message(error) });
      throw error;
    }
  }

  async function sync(uid: string): Promise<void> {
    const authUser = captureAuthUser(uid);
    const check = () => checkAccount(uid, authUser);
    check();
    if (!state(uid).loaded || state(uid).requiresBaseVerification) await reload(uid);
    check();
    const current = state(uid);
    if (current.hasIncoming || current.status === "conflict") {
      throw new SharingConflictError(current.hasIncoming
        ? "Cloud and local copies need review before sync."
        : current.error ?? undefined);
    }
    const unavailable = localUnavailable();
    if (unavailable) {
      patch(uid, { status: "error", error: unavailable });
      throw new Error(unavailable);
    }
    const bucket = localOwner(uid);
    if (bucket?.restoreReview) {
      const error = "This account library requires an explicit restore review before it can sync.";
      patch(uid, { status: "conflict", error });
      throw new SharingConflictError(error);
    }
    if (!dirty(uid)) return;
    if (bucket?.baseRevision === "unknown" || bucket?.baseRevision !== current.revision) {
      const error = "The pending local library has no verified matching cloud base. Reload and review before syncing.";
      patch(uid, { status: "conflict", error, hasIncoming: false, incoming: null });
      throw new SharingConflictError(error);
    }
    if (current.status === "syncing") throw new Error("Replay library sync is already in progress.");
    const outgoing = cloneEntries(entries(uid));
    const outgoingRevision = localRevision();
    const request = requests.get(uid) ?? 0;
    requests.set(uid, request);
    patch(uid, { status: "syncing", error: null });
    try {
      const saved = await save(uid, outgoing, current.revision);
      check();
      if (requests.get(uid) !== request) return;
      if (sameEntries(entries(uid), outgoing) && localRevision() === outgoingRevision) {
        try {
          await local.getState().markSynced(owner(uid), saved.revision, outgoingRevision,
            () => requestCurrent(uid, request, authUser));
          check();
          if (requests.get(uid) !== request) return;
          patch(uid, { loaded: true, revision: saved.revision, status: "synced", error: null, hasIncoming: false, incoming: null,
            requiresBaseVerification: false });
        } catch (error) {
          if (!requestCurrent(uid, request, authUser)) return;
          patch(uid, { loaded: true, revision: saved.revision, status: "pending",
            error: `Cloud save completed at revision ${saved.revision}, but this device could not record that base: ${message(error)}`,
            hasIncoming: false, incoming: null, requiresBaseVerification: true });
        }
        return;
      }

      patch(uid, { loaded: true, revision: saved.revision, status: "conflict",
        error: "Cloud save completed, but newer local edits arrived while it was pending. Review them against the saved cloud copy.",
        hasIncoming: true, incoming: saved, requiresBaseVerification: false });
    } catch (error) {
      if (requestCurrent(uid, request, authUser)) {
        const conflict = error instanceof SharingConflictError;
        patch(uid, { status: conflict ? "conflict" : "error", error: message(error),
          hasIncoming: conflict ? false : state(uid).hasIncoming,
          incoming: conflict ? null : state(uid).incoming });
      }
      throw error;
    }
  }

  async function useCloud(uid: string, confirmDiscard = false): Promise<void> {
    const authUser = captureAuthUser(uid);
    const check = () => checkAccount(uid, authUser);
    check();
    const current = state(uid);
    if (!current.hasIncoming) throw new Error("Reload the cloud library before choosing its version.");
    const bucket = localOwner(uid);
    const hasLocal = (bucket?.entries.length ?? 0) > 0 || Boolean(bucket?.dirty);
    if (hasLocal && !confirmDiscard) {
      throw new Error("Confirm that you want to replace the local copies with the reviewed cloud library.");
    }
    const request = requests.get(uid) ?? 0;
    requests.set(uid, request);
    const incoming = current.incoming;
    const expectedLocalRevision = localRevision();
    await local.getState().replaceEntries(owner(uid), incoming?.entries ?? [],
      () => requestCurrent(uid, request, authUser) && state(uid) === current,
      { expectedRevision: expectedLocalRevision, baseRevision: incoming?.revision ?? null, dirty: false, restoreReview: false });
    check();
    if (requests.get(uid) !== request) return;
    patch(uid, { loaded: true, revision: incoming?.revision ?? null, status: "synced", error: null, hasIncoming: false,
      incoming: null, requiresBaseVerification: false });
  }

  async function commitReviewedEntries(
    uid: string,
    reviewedEntries: PrivateLibraryEntry[],
    expected: { localRevision: number; cloudRevision: number | null },
    authUser = captureAuthUser(uid),
  ): Promise<PrivateLibrarySnapshot> {
    checkAccount(uid, authUser);
    const current = state(uid);
    const reviewedCloudRevision = current.hasIncoming ? current.incoming?.revision ?? null : current.revision;
    if (current.status === "syncing" || reviewedCloudRevision !== expected.cloudRevision) {
      throw new SharingConflictError("The cloud library changed after review. Reload and review again.");
    }
    privateLibraryV2Json(reviewedEntries);
    const request = requests.get(uid) ?? 0;
    requests.set(uid, request);
    const canContinue = () => requestCurrent(uid, request, authUser);
    const localOptions = { expectedRevision: expected.localRevision, dirty: false, restoreReview: false };
    const nextCloudRevision = expected.cloudRevision === null ? 1 : nextRevision(expected.cloudRevision, "Private library revision");
    // Check the combined cache budget and the on-disk revision under the local
    // lock. Release it before the transaction and leave both inputs intact.
    await local.getState().replaceEntries(owner(uid), reviewedEntries,
      () => canContinue() && state(uid) === current,
      { ...localOptions, baseRevision: nextCloudRevision, preflightOnly: true });
    if (!canContinue() || state(uid) !== current || localRevision() !== expected.localRevision) {
      throw new SharingConflictError("The local or cloud library changed after review. Reload and review again.");
    }

    patch(uid, { status: "syncing", error: null });
    let saved: PrivateLibrarySnapshot;
    try {
      saved = await save(uid, reviewedEntries, expected.cloudRevision);
    } catch (error) {
      if (canContinue()) {
        const conflict = error instanceof SharingConflictError;
        patch(uid, { status: conflict ? "conflict" : "error", error: message(error),
          hasIncoming: conflict ? false : current.hasIncoming, incoming: conflict ? null : current.incoming });
      }
      throw error;
    }

    checkAccount(uid, authUser);
    if (!canContinue()) return saved;
    try {
      await local.getState().replaceEntries(owner(uid), saved.entries, canContinue,
        { ...localOptions, baseRevision: saved.revision });
    } catch (error) {
      if (canContinue()) {
        patch(uid, { loaded: true, revision: saved.revision, status: "conflict",
          error: `Cloud save completed. This device kept its own copies: ${message(error)}`,
          hasIncoming: true, incoming: saved, requiresBaseVerification: false });
      }
      return saved;
    }
    checkAccount(uid, authUser);
    if (canContinue()) {
      patch(uid, { loaded: true, revision: saved.revision, status: "synced", error: null,
        hasIncoming: false, incoming: null, requiresBaseVerification: false });
    }
    return saved;
  }

  async function planReplayLibraryMerge(uid: string): Promise<ReplayLibraryMergePlan> {
    const authUser = captureAuthUser(uid);
    const check = () => checkAccount(uid, authUser);
    check();
    const current = state(uid);
    if (!current.hasIncoming) throw new Error("Reload and review the cloud library before planning a merge.");
    const ownerKey = owner(uid);
    const localEntries = await verifyReplayLibraryEntries(entries(uid));
    const cloudEntries = await verifyReplayLibraryEntries(current.incoming?.entries ?? []);
    const cloudByFingerprint = new Map(cloudEntries.map((entry) => [entry.fingerprint, entry]));
    const localByFingerprint = new Map(localEntries.map((entry) => [entry.fingerprint, entry]));
    for (const [fingerprint, localEntry] of localByFingerprint) {
      const cloudEntry = cloudByFingerprint.get(fingerprint);
      if (cloudEntry && (cloudEntry.payload !== localEntry.payload || cloudEntry.kind !== localEntry.kind)) {
        throw new Error("A matching replay identity has different game data or kind. Keep both inputs and resolve that identity before merging.");
      }
    }
    const fingerprints = [...cloudEntries.map((entry) => entry.fingerprint),
      ...localEntries.filter((entry) => !cloudByFingerprint.has(entry.fingerprint)).map((entry) => entry.fingerprint)];
    const rows = fingerprints.map((fingerprint): ReplayLibraryMergeRow => {
      const localEntry = localByFingerprint.get(fingerprint) ?? null;
      const cloudEntry = cloudByFingerprint.get(fingerprint) ?? null;
      if (!localEntry) return { fingerprint, status: "cloud-only", local: null, cloud: cloudEntry };
      if (!cloudEntry) return { fingerprint, status: "local-only", local: localEntry, cloud: null };
      return {
        fingerprint,
        status: sameMetadata(localEntry, cloudEntry) ? "same" : "metadata-conflict",
        local: localEntry,
        cloud: cloudEntry,
      };
    });
    const plan: ReplayLibraryMergePlan = {
      uid,
      owner: ownerKey,
      localRevision: localRevision(),
      cloudRevision: current.incoming?.revision ?? null,
      localEntries: cloneEntries(localEntries),
      cloudEntries: cloneEntries(cloudEntries),
      rows,
    };
    planAuthUsers.set(plan, authUser);
    check();
    return plan;
  }

  async function commitReplayLibraryMerge(
    plan: ReplayLibraryMergePlan,
    choices: Readonly<Record<string, "local" | "cloud">>,
  ): Promise<PrivateLibrarySnapshot> {
    const { uid } = plan;
    const authUser = planAuthUsers.get(plan) ?? null;
    const check = () => checkAccount(uid, authUser);
    check();
    const current = state(uid);
    if (plan.owner !== owner(uid) || !current.hasIncoming || localRevision() !== plan.localRevision
      || (current.incoming?.revision ?? null) !== plan.cloudRevision
      || !sameEntries(entries(uid), plan.localEntries)
      || !sameEntries(current.incoming?.entries ?? [], plan.cloudEntries)) {
      throw new Error("The local or cloud library changed after review. Reload and review again.");
    }
    for (const row of plan.rows) {
      if (row.status === "metadata-conflict" && choices[row.fingerprint] !== "local" && choices[row.fingerprint] !== "cloud") {
        throw new Error("Choose one complete set of private details for every differing replay.");
      }
      if (row.status !== "metadata-conflict" && Object.prototype.hasOwnProperty.call(choices, row.fingerprint)) {
        throw new Error("Metadata choices are only valid for replay rows with differing details.");
      }
    }
    for (const fingerprint of Object.keys(choices)) {
      if (!plan.rows.some((row) => row.fingerprint === fingerprint && row.status === "metadata-conflict")) {
        throw new Error("A metadata choice refers to a replay outside this review.");
      }
    }

    const localByFingerprint = new Map(plan.localEntries.map((entry) => [entry.fingerprint, entry]));
    const cloudByFingerprint = new Map(plan.cloudEntries.map((entry) => [entry.fingerprint, entry]));
    const merged = plan.rows.map((row) => {
      if (row.status === "local-only") return row.local!;
      if (row.status === "cloud-only") return row.cloud!;
      const localEntry = localByFingerprint.get(row.fingerprint)!;
      const cloudEntry = cloudByFingerprint.get(row.fingerprint)!;
      const selected = row.status === "metadata-conflict" && choices[row.fingerprint] === "cloud" ? cloudEntry : localEntry;
      return {
        ...selected,
        saved_at: localEntry.saved_at < cloudEntry.saved_at ? localEntry.saved_at : cloudEntry.saved_at,
      };
    });
    return commitReviewedEntries(uid, validateReplayLibraryEntries(merged), plan, authUser);
  }

  // Compatibility wrapper for callers that only need automatic coalescing.
  // Metadata differences remain unresolved and cannot be silently selected.
  async function mergeCopies(uid: string): Promise<void> {
    const plan = await planReplayLibraryMerge(uid);
    const hasMetadataConflicts = plan.rows.some((row) => row.status === "metadata-conflict");
    if (hasMetadataConflicts) throw new Error("Private details differ. Review both complete detail sets before merging.");
    await commitReplayLibraryMerge(plan, {});
  }

  function acceptCleared(uid: string, snapshot: PrivateLibrarySnapshot, optionsForClear: { preserveLocal?: boolean } = {}): void {
    lifecycles.set(uid, lifecycleVersion(uid) + 1);
    requests.set(uid, (requests.get(uid) ?? 0) + 1);
    const preserveLocal = optionsForClear.preserveLocal ?? true;
    const bucket = localOwner(uid);
    const needsReview = preserveLocal && Boolean(bucket?.entries.length);
    if (needsReview) void local.getState().markRestoreReview(owner(uid), true).catch(() => undefined);
    patch(uid, { loaded: true, revision: snapshot.revision, status: needsReview ? "conflict" : "synced",
      error: needsReview ? "Cloud library was cleared. Review local copies before explicitly restoring them." : null,
      hasIncoming: needsReview, incoming: needsReview ? snapshot : null });
  }

  return {
    store,
    reload,
    sync,
    useCloud,
    mergeCopies,
    commitReviewedEntries,
    planReplayLibraryMerge,
    commitReplayLibraryMerge,
    acceptCleared,
    lifecycleVersion,
  };
}

export const replayLibrarySync = createReplayLibrarySync();
export const planReplayLibraryMerge = replayLibrarySync.planReplayLibraryMerge;
export const commitReplayLibraryMerge = replayLibrarySync.commitReplayLibraryMerge;
