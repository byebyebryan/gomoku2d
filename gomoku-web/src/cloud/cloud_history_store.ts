import { createJSONStorage, persist, type StateStorage } from "zustand/middleware";
import { createStore, type StoreApi } from "zustand/vanilla";

import { savedMatchIsAfterReset, type SavedMatchV1 } from "../match/saved_match";

import type { CloudAuthUser } from "./auth_store";
import { clearCloudHistory, loadCloudHistory, saveCloudMatch } from "./cloud_history";
import { cloudProfileStore } from "./cloud_profile_store";

export type CloudHistoryLoadStatus = "idle" | "loading" | "ready" | "error";
export type CloudHistorySyncStatus = "idle" | "syncing" | "error";
export type CloudMatchSyncStatus = "pending" | "syncing" | "synced" | "error";

export interface CloudMatchSyncMeta {
  errorMessage: string | null;
  matchId: string;
  status: CloudMatchSyncStatus;
  updatedAt: string;
}

export interface CloudHistoryUserCache {
  cachedMatches: SavedMatchV1[];
  loadedAt: string | null;
  pendingMatches: Record<string, SavedMatchV1>;
  sync: Record<string, CloudMatchSyncMeta>;
}

export interface CloudHistoryState {
  clearForUser: (user: CloudAuthUser) => Promise<void>;
  errorMessage: string | null;
  loadForUser: (user: CloudAuthUser, historyResetAt?: string | null) => Promise<void>;
  loadStatus: CloudHistoryLoadStatus;
  resetUserCache: (uid: string) => void;
  syncMatchForUser: (user: CloudAuthUser, match: SavedMatchV1, historyResetAt?: string | null) => Promise<void>;
  syncPendingForUser: (user: CloudAuthUser, historyResetAt?: string | null) => Promise<void>;
  syncStatus: CloudHistorySyncStatus;
  users: Record<string, CloudHistoryUserCache>;
}

export interface CloudHistoryStoreOptions {
  clearHistory?: (user: CloudAuthUser) => Promise<number>;
  historyResetAtForUser?: (uid: string) => string | null | undefined;
  loadHistory?: (user: CloudAuthUser, historyResetAt?: string | null) => Promise<SavedMatchV1[]>;
  now?: () => string;
  saveMatch?: (user: CloudAuthUser, match: SavedMatchV1) => Promise<{ match: SavedMatchV1 }>;
  storage?: StateStorage;
}

const STORAGE_KEY = "gomoku2d.cloud-history.v1";
const CACHE_LIMIT = 24;

function errorMessageFor(error: unknown): string {
  return error instanceof Error ? error.message : "Cloud history sync failed.";
}

function defaultUserCache(): CloudHistoryUserCache {
  return {
    cachedMatches: [],
    loadedAt: null,
    pendingMatches: {},
    sync: {},
  };
}

function sortMatches(matches: SavedMatchV1[]): SavedMatchV1[] {
  return [...matches].sort((left, right) => right.saved_at.localeCompare(left.saved_at));
}

function upsertCachedMatch(cache: CloudHistoryUserCache, match: SavedMatchV1): CloudHistoryUserCache {
  const withoutExisting = cache.cachedMatches.filter((entry) => entry.id !== match.id);

  return {
    ...cache,
    cachedMatches: sortMatches([match, ...withoutExisting]).slice(0, CACHE_LIMIT),
  };
}

function updateUserCache(
  users: Record<string, CloudHistoryUserCache>,
  uid: string,
  update: (cache: CloudHistoryUserCache) => CloudHistoryUserCache,
): Record<string, CloudHistoryUserCache> {
  return {
    ...users,
    [uid]: update(users[uid] ?? defaultUserCache()),
  };
}

function cacheWithSyncMeta(
  cache: CloudHistoryUserCache,
  match: SavedMatchV1,
  status: CloudMatchSyncStatus,
  errorMessage: string | null,
  updatedAt: string,
): CloudHistoryUserCache {
  return {
    ...cache,
    pendingMatches:
      status === "synced"
        ? Object.fromEntries(Object.entries(cache.pendingMatches).filter(([id]) => id !== match.id))
        : { ...cache.pendingMatches, [match.id]: match },
    sync: {
      ...cache.sync,
      [match.id]: {
        errorMessage,
        matchId: match.id,
        status,
        updatedAt,
      },
    },
  };
}

function cacheWithoutMatch(cache: CloudHistoryUserCache, matchId: string): CloudHistoryUserCache {
  const { [matchId]: _pending, ...pendingMatches } = cache.pendingMatches;
  const { [matchId]: _sync, ...sync } = cache.sync;

  return {
    ...cache,
    cachedMatches: cache.cachedMatches.filter((entry) => entry.id !== matchId),
    pendingMatches,
    sync,
  };
}

function newerResetAt(
  left: string | null | undefined,
  right: string | null | undefined,
): string | null {
  if (!left) {
    return right ?? null;
  }

  if (!right) {
    return left;
  }

  return left >= right ? left : right;
}

function activeSyncsFor(
  activeSyncs: Map<string, Set<Promise<void>>>,
  uid: string,
): Set<Promise<void>> {
  const existing = activeSyncs.get(uid);
  if (existing) {
    return existing;
  }

  const created = new Set<Promise<void>>();
  activeSyncs.set(uid, created);
  return created;
}

export function createCloudHistoryStore(
  options: CloudHistoryStoreOptions = {},
): StoreApi<CloudHistoryState> {
  const clearHistory = options.clearHistory ?? clearCloudHistory;
  const historyResetAtForUser = options.historyResetAtForUser ?? (() => null);
  const loadHistory =
    options.loadHistory
    ?? ((user: CloudAuthUser, historyResetAt: string | null | undefined) =>
      loadCloudHistory(user, { historyResetAt }));
  const saveMatch = options.saveMatch ?? saveCloudMatch;
  const now = options.now ?? (() => new Date().toISOString());
  const storage = createJSONStorage<Pick<CloudHistoryState, "users">>(() => options.storage ?? localStorage);
  const latestResetAt = (uid: string, requestedResetAt: string | null | undefined) =>
    newerResetAt(requestedResetAt, historyResetAtForUser(uid));
  const activeSyncs = new Map<string, Set<Promise<void>>>();
  const resettingUids = new Set<string>();

  return createStore<CloudHistoryState>()(
    persist(
      (set, get) => ({
        clearForUser: async (user) => {
          set({
            errorMessage: null,
            syncStatus: "syncing",
          });
          resettingUids.add(user.uid);

          try {
            await Promise.allSettled(activeSyncs.get(user.uid) ?? []);
            await clearHistory(user);
            set((state) => {
              const { [user.uid]: _removed, ...users } = state.users;
              return {
                errorMessage: null,
                syncStatus: "idle",
                users,
              };
            });
          } catch (error) {
            const message = errorMessageFor(error);
            set({
              errorMessage: message,
              syncStatus: "error",
            });
            throw new Error(message);
          } finally {
            resettingUids.delete(user.uid);
          }
        },
        errorMessage: null,
        loadForUser: async (user, historyResetAt = null) => {
          set({
            errorMessage: null,
            loadStatus: "loading",
          });

          try {
            const matches = await loadHistory(user, historyResetAt);
            set((state) => ({
              errorMessage: null,
              loadStatus: "ready",
              users: updateUserCache(state.users, user.uid, (cache) => ({
                ...cache,
                cachedMatches: sortMatches(matches).slice(0, CACHE_LIMIT),
                loadedAt: now(),
              })),
            }));
          } catch (error) {
            set({
              errorMessage: errorMessageFor(error),
              loadStatus: "error",
            });
          }
        },
        loadStatus: "idle",
        resetUserCache: (uid) => {
          set((state) => {
            const { [uid]: _removed, ...users } = state.users;
            return { users };
          });
        },
        syncMatchForUser: async (user, match, historyResetAt = null) => {
          if (resettingUids.has(user.uid) || !savedMatchIsAfterReset(match, latestResetAt(user.uid, historyResetAt))) {
            set((state) => ({
              errorMessage: null,
              users: updateUserCache(state.users, user.uid, (cache) => cacheWithoutMatch(cache, match.id)),
            }));
            return;
          }

          set((state) => ({
            errorMessage: null,
            syncStatus: "syncing",
            users: updateUserCache(state.users, user.uid, (cache) =>
              cacheWithSyncMeta(cache, match, "syncing", null, now()),
            ),
          }));

          const syncPromise = (async () => {
            const result = await saveMatch(user, match);
            if (!savedMatchIsAfterReset(result.match, latestResetAt(user.uid, historyResetAt))) {
              set((state) => ({
                errorMessage: null,
                syncStatus: "idle",
                users: updateUserCache(state.users, user.uid, (cache) => cacheWithoutMatch(cache, match.id)),
              }));
              return;
            }

            set((state) => ({
              errorMessage: null,
              syncStatus: "idle",
              users: updateUserCache(state.users, user.uid, (cache) =>
                upsertCachedMatch(
                  cacheWithSyncMeta(cache, match, "synced", null, now()),
                  result.match,
                ),
              ),
            }));
          })();
          activeSyncsFor(activeSyncs, user.uid).add(syncPromise);

          try {
            await syncPromise;
          } catch (error) {
            const message = errorMessageFor(error);
            set((state) => ({
              errorMessage: message,
              syncStatus: "error",
              users: updateUserCache(state.users, user.uid, (cache) =>
                cacheWithSyncMeta(cache, match, "error", message, now()),
              ),
            }));
          } finally {
            const syncs = activeSyncs.get(user.uid);
            syncs?.delete(syncPromise);
            if (syncs?.size === 0) {
              activeSyncs.delete(user.uid);
            }
          }
        },
        syncPendingForUser: async (user, historyResetAt = null) => {
          const pending = Object.values(get().users[user.uid]?.pendingMatches ?? {});
          for (const match of pending) {
            await get().syncMatchForUser(user, match, historyResetAt);
          }
        },
        syncStatus: "idle",
        users: {},
      }),
      {
        name: STORAGE_KEY,
        partialize: (state) => ({ users: state.users }),
        storage,
      },
    ),
  );
}

export const cloudHistoryStore = createCloudHistoryStore({
  historyResetAtForUser: (uid) => {
    const profile = cloudProfileStore.getState().profile;
    return profile?.uid === uid ? profile.historyResetAt : null;
  },
});
