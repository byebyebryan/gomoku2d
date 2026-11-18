import { beforeEach, describe, expect, it, vi } from "vitest";

import { cloudAuthStore, type CloudAuthUser } from "../cloud/auth_store";
import { SharingConflictError } from "../cloud/sharing_common";
import type { PrivateLibrarySnapshot } from "../cloud/sharing_private";
import { replayLibraryStore, REPLAY_LIBRARY_STORAGE_KEY } from "./replay_library_store";
import { replayLibrarySync } from "./replay_library_sync";
import { EMPTY_LIBRARY_SYNC } from "./replay_library_sync";
import type { PortableReplayV1 } from "./portable_replay";
import { commitGuestReplayImport, importGuestReplayCopies, planGuestReplayImport, saveReplayCopy } from "./replay_library_actions";

const sharingMocks = vi.hoisted(() => ({
  loadPrivateLibrary: vi.fn(),
  savePrivateLibrary: vi.fn(),
}));

vi.mock("../cloud/sharing_private", async (importOriginal) => ({
  ...await importOriginal<typeof import("../cloud/sharing_private")>(),
  loadPrivateLibrary: sharingMocks.loadPrivateLibrary,
  savePrivateLibrary: sharingMocks.savePrivateLibrary,
}));

const BOT: PortableReplayV1["white"] = {
  bot: {
    config: { mode: "preset", preset: "normal", version: 1 },
    configVersion: 1,
    engine: "search_bot",
    version: 1,
  },
  kind: "bot",
  name: "Normal Bot",
};

function winningReplay(name = "Guest"): PortableReplayV1 {
  return {
    black: { kind: "human", name },
    moves: [110, 0, 111, 1, 112, 2, 113, 3, 114],
    ruleset: "freestyle",
    version: 1,
    white: BOT,
  };
}

function clearedSnapshot(uid: string): PrivateLibrarySnapshot {
  return {
    uid,
    revision: 9,
    entries: [],
    updatedAt: "2026-10-21T00:00:00.000Z",
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((finish) => { resolve = finish; });
  return { promise, resolve };
}

function signIn(uid: string) {
  const user: CloudAuthUser = {
    avatarUrl: null,
    displayName: "Test player",
    email: null,
    providerIds: [],
    uid,
  };
  cloudAuthStore.setState({ status: "signed_in", user });
}

describe("replay library actions during cloud clearing", () => {
  beforeEach(async () => {
    localStorage.clear();
    replayLibraryStore.setState({ owners: {}, revision: 0, sessionOnly: false, readOnly: false, storageError: null });
    await replayLibraryStore.getState().refreshFromStorage();
    replayLibrarySync.store.setState({ users: {} }, true);
    cloudAuthStore.setState({ status: "signed_out", user: null });
    sharingMocks.loadPrivateLibrary.mockReset();
    sharingMocks.savePrivateLibrary.mockReset();
  });

  it("invalidates a pending Save when the account library is cleared during its initial load", async () => {
    const uid = "save-pending-user";
    signIn(uid);
    const loading = deferred<PrivateLibrarySnapshot | null>();
    const started = deferred<void>();
    sharingMocks.loadPrivateLibrary.mockImplementationOnce(() => {
      started.resolve();
      return loading.promise;
    });

    const pendingSave = saveReplayCopy(winningReplay("Before clear"));
    await started.promise;
    replayLibrarySync.acceptCleared(uid, clearedSnapshot(uid));
    loading.resolve(null);

    await expect(pendingSave).rejects.toThrow(/cleared while this action was pending/i);
    expect(replayLibraryStore.getState().owners[`user:${uid}`]?.entries ?? []).toEqual([]);
    expect(replayLibrarySync.store.getState().users[uid]).toMatchObject({
      revision: 9,
      status: "synced",
    });
    expect(sharingMocks.savePrivateLibrary).not.toHaveBeenCalled();
  });

  it("invalidates guest import during its initial load and preserves the guest source", async () => {
    const uid = "import-pending-user";
    signIn(uid);
    await replayLibraryStore.getState().saveReplay("guest", winningReplay("Keep guest copy"), "saved");
    const guestBefore = replayLibraryStore.getState().owners.guest?.entries;
    expect(guestBefore).toHaveLength(1);

    const loading = deferred<PrivateLibrarySnapshot | null>();
    const started = deferred<void>();
    sharingMocks.loadPrivateLibrary.mockImplementationOnce(() => {
      started.resolve();
      return loading.promise;
    });

    const pendingImport = importGuestReplayCopies();
    await started.promise;
    replayLibrarySync.acceptCleared(uid, clearedSnapshot(uid));
    loading.resolve(null);

    await expect(pendingImport).rejects.toThrow(/cleared while this action was pending/i);
    expect(replayLibraryStore.getState().owners.guest?.entries).toEqual(guestBefore);
    expect(replayLibraryStore.getState().owners[`user:${uid}`]?.entries ?? []).toEqual([]);
    expect(replayLibrarySync.store.getState().users[uid]).toMatchObject({
      revision: 9,
      status: "synced",
    });
    expect(sharingMocks.savePrivateLibrary).not.toHaveBeenCalled();
  });

  it("requires review before importing guests after a failed cloud reload", async () => {
    const uid = "review-after-error";
    signIn(uid);
    await replayLibraryStore.getState().saveReplay("guest", winningReplay(), "saved");
    replayLibrarySync.store.setState({ users: { [uid]: {
      ...EMPTY_LIBRARY_SYNC, loaded: true, revision: 9, status: "error", error: "Offline",
      hasIncoming: true, incoming: clearedSnapshot(uid),
    } } });

    await expect(importGuestReplayCopies()).rejects.toThrow(/review/i);
    expect(replayLibraryStore.getState().owners[`user:${uid}`]).toBeUndefined();
    expect(replayLibraryStore.getState().owners.guest?.entries).toHaveLength(1);
    expect(sharingMocks.savePrivateLibrary).not.toHaveBeenCalled();
  });

  it("preserves both account and guest details when cloud rejects a reviewed import", async () => {
    const uid = "stale-import";
    const owner = `user:${uid}` as const;
    signIn(uid);
    sharingMocks.loadPrivateLibrary.mockResolvedValueOnce(null);
    await replayLibrarySync.reload(uid);
    const fingerprint = await replayLibraryStore.getState().saveReplay("guest", winningReplay(), "saved");
    await replayLibraryStore.getState().saveReplay(owner, winningReplay(), "saved");
    await replayLibraryStore.getState().updateReplayMetadata("guest", fingerprint, { title: "Guest", note: "Guest note", opening_ply: 0 });
    await replayLibraryStore.getState().updateReplayMetadata(owner, fingerprint, { title: "Account", note: "Account note", opening_ply: 2 });
    const plan = await planGuestReplayImport(uid);
    expect(plan.rows[0]?.status).toBe("metadata-conflict");
    const originalBytes = localStorage.getItem(REPLAY_LIBRARY_STORAGE_KEY);
    const originalDevice = replayLibraryStore.getState().owners;
    sharingMocks.savePrivateLibrary.mockRejectedValueOnce(new SharingConflictError());

    await expect(commitGuestReplayImport(plan, { [fingerprint]: "guest" })).rejects.toThrow(SharingConflictError);
    expect(sharingMocks.savePrivateLibrary).toHaveBeenCalledOnce();
    expect(replayLibraryStore.getState().owners).toBe(originalDevice);
    expect(localStorage.getItem(REPLAY_LIBRARY_STORAGE_KEY)).toBe(originalBytes);
  });
});
