import { createJSONStorage, persist, type StateStorage } from "zustand/middleware";
import { createStore, type StoreApi } from "zustand/vanilla";

import type { SavedMatchV1 } from "../match/saved_match";

import type { CloudAuthUser } from "./auth_store";
import { loadCloudHistory, saveCloudMatch } from "./cloud_history";

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
  errorMessage: string | null;
  loadForUser: (user: CloudAuthUser) => Promise<void>;
  loadStatus: CloudHistoryLoadStatus;
  resetUserCache: (uid: string) => void;
  syncMatchForUser: (user: CloudAuthUser, match: SavedMatchV1) => Promise<void>;
  syncPendingForUser: (user: CloudAuthUser) => Promise<void>;
  syncStatus: CloudHistorySyncStatus;
  users: Record<string, CloudHistoryUserCache>;
}

export interface CloudHistoryStoreOptions {
  loadHistory?: (user: CloudAuthUser) => Promise<SavedMatchV1[]>;
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

export function createCloudHistoryStore(
  options: CloudHistoryStoreOptions = {},
): StoreApi<CloudHistoryState> {
  const loadHistory = options.loadHistory ?? loadCloudHistory;
  const saveMatch = options.saveMatch ?? saveCloudMatch;
  const now = options.now ?? (() => new Date().toISOString());
  const storage = createJSONStorage<Pick<CloudHistoryState, "users">>(() => options.storage ?? localStorage);

  return createStore<CloudHistoryState>()(
    persist(
      (set, get) => ({
        errorMessage: null,
        loadForUser: async (user) => {
          set({
            errorMessage: null,
            loadStatus: "loading",
          });

          try {
            const matches = await loadHistory(user);
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
        syncMatchForUser: async (user, match) => {
          set((state) => ({
            errorMessage: null,
            syncStatus: "syncing",
            users: updateUserCache(state.users, user.uid, (cache) =>
              cacheWithSyncMeta(cache, match, "syncing", null, now()),
            ),
          }));

          try {
            const result = await saveMatch(user, match);
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
          } catch (error) {
            const message = errorMessageFor(error);
            set((state) => ({
              errorMessage: message,
              syncStatus: "error",
              users: updateUserCache(state.users, user.uid, (cache) =>
                cacheWithSyncMeta(cache, match, "error", message, now()),
              ),
            }));
          }
        },
        syncPendingForUser: async (user) => {
          const pending = Object.values(get().users[user.uid]?.pendingMatches ?? {});
          for (const match of pending) {
            await get().syncMatchForUser(user, match);
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

export const cloudHistoryStore = createCloudHistoryStore();
