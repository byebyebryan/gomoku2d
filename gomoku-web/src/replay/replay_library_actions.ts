import { cloudAuthStore } from "../cloud/auth_store";
import { decodePortableReplay } from "./portable_replay_codec";
import type { PortableReplayV1 } from "./portable_replay";
import { currentReplayLibraryOwner, replayLibraryStore, type ReplayLibraryKind } from "./replay_library_store";
import { replayLibrarySync } from "./replay_library_sync";

function actionGuard(owner: ReturnType<typeof currentReplayLibraryOwner>) {
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

// These actions are invoked by Save/Import controls, never by replay viewing.
export async function saveReplayCopy(replay: PortableReplayV1, kind: ReplayLibraryKind = "saved") {
  const owner = currentReplayLibraryOwner();
  const uid = owner === "guest" ? null : owner.slice(5);
  const check = actionGuard(owner);
  if (uid && !replayLibrarySync.store.getState().users[uid]?.loaded) {
    try { await replayLibrarySync.reload(uid); } catch { /* Keep the deliberate local save available offline. */ }
  }
  check();
  const fingerprint = await replayLibraryStore.getState().saveReplay(owner, replay, kind);
  check();
  if (uid && cloudAuthStore.getState().user?.uid === uid) {
    const sync = replayLibrarySync.store.getState().users[uid];
    if (sync?.loaded && !sync.hasIncoming && sync.status !== "conflict") {
      try { await replayLibrarySync.sync(uid); } catch { /* The sync state exposes pending/error recovery. */ }
    }
  }
  return fingerprint;
}

export async function importGuestReplayCopies() {
  const owner = currentReplayLibraryOwner();
  if (owner === "guest") throw new Error("Sign in before copying guest replays to an account.");
  const uid = owner.slice(5);
  const check = actionGuard(owner);
  if (!replayLibrarySync.store.getState().users[uid]?.loaded) await replayLibrarySync.reload(uid);
  check();
  const sync = replayLibrarySync.store.getState().users[uid];
  if (sync?.hasIncoming || sync?.status === "conflict") {
    throw new Error("Review the account library conflict before importing guest copies.");
  }
  const guest = replayLibraryStore.getState().owners.guest?.entries ?? [];
  const decoded = guest.map((entry) => ({ entry, replay: decodePortableReplay(entry.payload).replay }));
  const existing = new Set(replayLibraryStore.getState().owners[owner]?.entries.map((entry) => entry.fingerprint) ?? []);
  if (new Set([...existing, ...guest.map((entry) => entry.fingerprint)]).size > 64) {
    throw new Error("These guest copies would exceed the 64-replay account library limit.");
  }
  for (const { entry, replay } of decoded) {
    check();
    if (!existing.has(entry.fingerprint)) await replayLibraryStore.getState().saveReplay(owner, replay, entry.kind);
  }
  check();
  await replayLibrarySync.sync(uid);
}
