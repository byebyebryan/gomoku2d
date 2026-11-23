import { cloudAuthStore, type CloudAuthUser } from "../cloud/auth_store";
import type { PrivateLibraryEntry, PrivateReplayMetadata } from "../cloud/sharing_private";
import {
  currentReplayLibraryOwner,
  replayLibraryStore,
  validateReplayLibraryEntries,
  verifyReplayLibraryEntries,
  type ReplayLibraryOwner,
  type ReplayLibraryKind,
} from "./replay_library_store";
import {
  mergeReplayEntries,
  matchingFingerprintHasSamePayloadAndKind,
  replayEntriesHaveSameMetadata,
  validateReplayMergeChoices,
} from "./replay_library_merge";
import { replayLibrarySync } from "./replay_library_sync";

export interface ReplayLibraryImportRow {
  fingerprint: string;
  status: "guest-only" | "already-present" | "metadata-conflict";
  account: PrivateLibraryEntry | null;
  guest: PrivateLibraryEntry;
}

export interface ReplayLibraryImportPlan {
  uid: string;
  owner: ReplayLibraryOwner;
  localRevision: number;
  cloudRevision: number | null;
  guestRevision: number;
  accountEntries: PrivateLibraryEntry[];
  guestEntries: PrivateLibraryEntry[];
  rows: ReplayLibraryImportRow[];
}

export interface ReplayLibraryImportResult {
  cloudStatus: "synced" | "pending";
  localRevision: number;
}

export type ReplayLibraryImportChoice = "account" | "guest";

function actionGuard(owner: ReplayLibraryOwner) {
  const uid = owner === "guest" ? null : owner.slice(5);
  const user = cloudAuthStore.getState().user;
  const lifecycle = uid ? replayLibrarySync.lifecycleVersion(uid) : 0;
  return () => {
    if (currentReplayLibraryOwner() !== owner || (uid && cloudAuthStore.getState().user !== user)) {
      throw new Error("Account changed. Review and save again for the current account.");
    }
    if (uid && replayLibrarySync.lifecycleVersion(uid) !== lifecycle) {
      throw new Error("The account library was cleared while this action was pending. Review local and cloud copies before trying again.");
    }
  };
}

function authUserFor(uid: string): CloudAuthUser | null {
  const auth = cloudAuthStore.getState();
  return auth.status === "signed_in" && auth.user?.uid === uid ? auth.user : null;
}

const importAuthUsers = new WeakMap<ReplayLibraryImportPlan, CloudAuthUser | null>();

function cloneEntries(entries: PrivateLibraryEntry[]): PrivateLibraryEntry[] {
  return entries.map((entry) => ({ ...entry, metadata: { ...entry.metadata } }));
}

function compareCloudRevision(uid: string, cloudRevision: number | null): void {
  const sync = replayLibrarySync.store.getState().users[uid];
  if (!sync?.loaded || sync.revision !== cloudRevision || sync.hasIncoming || sync.status === "conflict") {
    throw new Error("The account cloud library changed after review. Reload and resolve that copy before importing guest replays.");
  }
}

// These actions are invoked by explicit Save/Import/Details controls.
export async function saveReplayCopy(replay: import("./portable_replay").PortableReplayV1, kind: ReplayLibraryKind = "saved") {
  const owner = currentReplayLibraryOwner();
  const uid = owner === "guest" ? null : owner.slice(5);
  const check = actionGuard(owner);
  if (uid && !replayLibrarySync.store.getState().users[uid]?.loaded) {
    try { await replayLibrarySync.reload(uid); } catch { /* The explicit local save remains available offline. */ }
  }
  check();
  const fingerprint = await replayLibraryStore.getState().saveReplay(owner, replay, kind, check);
  check();
  if (uid && cloudAuthStore.getState().user?.uid === uid) {
    const sync = replayLibrarySync.store.getState().users[uid];
    if (sync?.loaded && !sync.hasIncoming && sync.status !== "conflict") {
      try { await replayLibrarySync.sync(uid); } catch { /* Local and cloud status remain visible for recovery. */ }
    }
  }
  return fingerprint;
}

export async function updateReplayDetails(
  owner: ReplayLibraryOwner,
  fingerprint: string,
  metadata: PrivateReplayMetadata,
  expectedMetadata?: PrivateReplayMetadata,
): Promise<string | void> {
  const check = actionGuard(owner);
  const uid = owner === "guest" ? null : owner.slice(5);
  if (uid && !replayLibrarySync.store.getState().users[uid]?.loaded) {
    try { await replayLibrarySync.reload(uid); } catch { /* Preserve offline edits with their existing unknown base. */ }
  }
  check();
  await replayLibraryStore.getState().updateReplayMetadata(owner, fingerprint, metadata, expectedMetadata, check);
  check();
  const local = replayLibraryStore.getState();
  const volatile = local.sessionOnly || local.owners[owner]?.durable === false;
  if (!uid) return volatile
    ? "Details changed for this session only. Device storage is unavailable."
    : "Details saved on this device.";
  const syncState = replayLibrarySync.store.getState().users[uid];
  if (!syncState?.loaded || syncState.hasIncoming || syncState.status === "conflict") {
    return volatile
      ? "Details changed for this session only. Review the cloud library before syncing."
      : "Details saved on this device. Review the cloud library before syncing.";
  }
  try {
    await replayLibrarySync.sync(uid);
    check();
    const latest = replayLibrarySync.store.getState().users[uid];
    if (latest?.status === "synced" && !replayLibraryStore.getState().owners[owner]?.dirty) {
      return volatile
        ? "Details saved to your account. This device could not keep a durable local copy."
        : "Details saved on this device and synced to cloud.";
    }
    return volatile
      ? "Details are pending cloud sync and remain for this session only."
      : "Details saved on this device and remain pending for cloud sync.";
  } catch (error) {
    check();
    return `${volatile ? "Details remain for this session only" : "Details saved on this device"}. Cloud sync needs attention: ${error instanceof Error ? error.message : "retry sync"}`;
  }
}

export async function planGuestReplayImport(uid: string): Promise<ReplayLibraryImportPlan> {
  const owner = `user:${uid}` as const;
  const check = actionGuard(owner);
  const authUser = authUserFor(uid);
  if (currentReplayLibraryOwner() !== owner) throw new Error("Sign in to the account receiving these guest replays first.");
  if (!replayLibrarySync.store.getState().users[uid]?.loaded) await replayLibrarySync.reload(uid);
  check();
  const sync = replayLibrarySync.store.getState().users[uid];
  if (!sync?.loaded || sync.hasIncoming || sync.status === "conflict") {
    throw new Error("Review the account cloud library before importing guest copies.");
  }
  const accountEntries = await verifyReplayLibraryEntries(replayLibraryStore.getState().owners[owner]?.entries ?? []);
  const guestEntries = await verifyReplayLibraryEntries(replayLibraryStore.getState().owners.guest?.entries ?? []);
  check();
  if (cloudAuthStore.getState().user !== authUser) throw new Error("The signed-in session changed. Review the import again.");
  const accountByFingerprint = new Map(accountEntries.map((entry) => [entry.fingerprint, entry]));
  const rows: ReplayLibraryImportRow[] = guestEntries.map((guest) => {
    const account = accountByFingerprint.get(guest.fingerprint) ?? null;
    if (account && !matchingFingerprintHasSamePayloadAndKind(account, guest)) {
      throw new Error("A matching replay identity has different game data or kind. Both copies remain available; resolve that identity before importing.");
    }
    return {
      fingerprint: guest.fingerprint,
      status: !account ? "guest-only" : replayEntriesHaveSameMetadata(account, guest) ? "already-present" : "metadata-conflict",
      account,
      guest,
    };
  });
  const plan: ReplayLibraryImportPlan = {
    uid,
    owner,
    localRevision: replayLibraryStore.getState().revision,
    cloudRevision: sync.revision,
    guestRevision: replayLibraryStore.getState().revision,
    accountEntries: cloneEntries(accountEntries),
    guestEntries: cloneEntries(guestEntries),
    rows,
  };
  importAuthUsers.set(plan, authUser);
  return plan;
}

export async function commitGuestReplayImport(
  plan: ReplayLibraryImportPlan,
  choices: Readonly<Record<string, ReplayLibraryImportChoice>>,
): Promise<ReplayLibraryImportResult> {
  const { uid, owner } = plan;
  const plannedAuthUser = importAuthUsers.get(plan) ?? null;
  const check = actionGuard(owner);
  check();
  if (plannedAuthUser && cloudAuthStore.getState().user !== plannedAuthUser) {
    throw new Error("The signed-in session changed after import review. Review the import again.");
  }
  if (currentReplayLibraryOwner() !== owner || replayLibraryStore.getState().revision !== plan.localRevision) {
    throw new Error("Local replay copies changed after review. Review the import again.");
  }
  const localState = replayLibraryStore.getState();
  const accountCurrent = localState.owners[owner]?.entries ?? [];
  const guestCurrent = localState.owners.guest?.entries ?? [];
  if (JSON.stringify(accountCurrent) !== JSON.stringify(plan.accountEntries)
    || JSON.stringify(guestCurrent) !== JSON.stringify(plan.guestEntries)) {
    throw new Error("Account or guest copies changed after review. Review the import again.");
  }
  compareCloudRevision(uid, plan.cloudRevision);
  validateReplayMergeChoices(plan.rows, choices, ["account", "guest"],
    "Metadata choices are only valid for guest replays with differing details.");

  const accountByFingerprint = new Map(plan.accountEntries.map((entry) => [entry.fingerprint, entry]));
  const merged = [...plan.accountEntries];
  for (const row of plan.rows) {
    const account = accountByFingerprint.get(row.fingerprint);
    if (!account) {
      merged.push(row.guest);
    } else {
      const index = merged.findIndex((entry) => entry.fingerprint === row.fingerprint);
      const preferred = row.status === "metadata-conflict" && choices[row.fingerprint] === "guest" ? "right" : "left";
      merged[index] = mergeReplayEntries(account, row.guest, preferred);
    }
  }
  const validated = validateReplayLibraryEntries(merged);
  compareCloudRevision(uid, plan.cloudRevision);
  await replayLibrarySync.commitReviewedEntries(uid, validated, plan, plannedAuthUser);
  check();
  const sync = replayLibrarySync.store.getState().users[uid];
  const cloudStatus = sync?.status === "synced" && !replayLibraryStore.getState().owners[owner]?.dirty
    ? "synced"
    : "pending";
  return { cloudStatus, localRevision: replayLibraryStore.getState().revision };
}

export async function importGuestReplayCopies() {
  const owner = currentReplayLibraryOwner();
  if (owner === "guest") throw new Error("Sign in before copying guest replays to an account.");
  const plan = await planGuestReplayImport(owner.slice(5));
  const conflicts = plan.rows.filter((row) => row.status === "metadata-conflict");
  if (conflicts.length) throw new Error("Guest and account details differ. Review both complete detail sets before importing.");
  return commitGuestReplayImport(plan, {});
}
