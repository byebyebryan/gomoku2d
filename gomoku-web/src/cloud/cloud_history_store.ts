import { createJSONStorage, persist, type StateStorage } from "zustand/middleware";
import { createStore, type StoreApi } from "zustand/vanilla";

import type { GameVariant } from "../core/bot_protocol";
import { savedMatchIsAfterReset, type SavedMatchV1 } from "../match/saved_match";
import { localProfileStore } from "../profile/local_profile_store";

import type { CloudAuthUser } from "./auth_store";
import { saveCloudHistorySnapshot } from "./cloud_history";
import {
  CLOUD_REPLAY_MATCHES_LIMIT,
  cloudMatchHistoryHasMatch,
  cloudProfileSyncDue,
  mergeCloudSavedMatches,
  type CloudProfile,
} from "./cloud_profile";
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
  loadFromProfile: (user: CloudAuthUser, profile: CloudProfile, historyResetAt?: string | null) => void;
  loadStatus: CloudHistoryLoadStatus;
  resetUserCache: (uid: string) => void;
  syncMatchForUser: (user: CloudAuthUser, match: SavedMatchV1, historyResetAt?: string | null) => Promise<void>;
  syncPendingForUser: (user: CloudAuthUser, historyResetAt?: string | null) => Promise<void>;
  syncStatus: CloudHistorySyncStatus;
  users: Record<string, CloudHistoryUserCache>;
}

export interface CloudHistoryStoreOptions {
  cloudProfileForUser?: (uid: string) => CloudProfile | null;
  historyResetAtForUser?: (uid: string) => string | null | undefined;
  loadHistory?: (user: CloudAuthUser, historyResetAt?: string | null) => Promise<SavedMatchV1[]>;
  now?: () => string;
  refreshCloudProfileForUser?: (user: CloudAuthUser, preferredVariant: GameVariant) => Promise<CloudProfile | null>;
  saveHistory?: (
    user: CloudAuthUser,
    input: {
      cloudProfile: CloudProfile;
      displayName: string;
      matches: SavedMatchV1[];
      preferredVariant: GameVariant;
    },
  ) => Promise<{ matches: SavedMatchV1[]; profile: CloudProfile }>;
  storage?: StateStorage;
}

const STORAGE_KEY = "gomoku2d.cloud-history.v3";

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

function cacheWithPendingMatch(
  cache: CloudHistoryUserCache,
  match: SavedMatchV1,
  status: CloudMatchSyncStatus,
  errorMessage: string | null,
  updatedAt: string,
): CloudHistoryUserCache {
  const withoutExisting = cache.cachedMatches.filter((entry) => entry.id !== match.id);

  return {
    ...cache,
    cachedMatches: sortMatches([match, ...withoutExisting]).slice(0, CLOUD_REPLAY_MATCHES_LIMIT),
    pendingMatches: { ...cache.pendingMatches, [match.id]: match },
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

function cacheWithSyncedMatches(
  cache: CloudHistoryUserCache,
  matches: SavedMatchV1[],
  syncedMatchIds: Set<string>,
): CloudHistoryUserCache {
  const pendingMatches = Object.fromEntries(
    Object.entries(cache.pendingMatches).filter(([matchId]) => !syncedMatchIds.has(matchId)),
  );
  const pendingMatchIds = new Set(Object.keys(pendingMatches));
  const sync = Object.fromEntries(
    Object.entries(cache.sync).filter(([matchId]) => pendingMatchIds.has(matchId)),
  );

  return {
    ...cache,
    cachedMatches: sortMatches([...matches, ...Object.values(pendingMatches)]).slice(0, CLOUD_REPLAY_MATCHES_LIMIT),
    pendingMatches,
    sync,
  };
}

function cacheWithLoadedMatches(
  cache: CloudHistoryUserCache,
  matches: SavedMatchV1[],
  historyResetAt: string | null | undefined,
  loadedAt: string,
): CloudHistoryUserCache {
  const activeMatches = matches.filter((match) => savedMatchIsAfterReset(match, historyResetAt));
  const loadedMatchIds = new Set(activeMatches.map((match) => match.id));
  const pendingMatches = Object.fromEntries(
    Object.entries(cache.pendingMatches).filter(([matchId, match]) => (
      savedMatchIsAfterReset(match, historyResetAt) && !loadedMatchIds.has(matchId)
    )),
  );
  const pendingMatchIds = new Set(Object.keys(pendingMatches));
  const sync = Object.fromEntries(
    Object.entries(cache.sync).filter(([matchId]) => pendingMatchIds.has(matchId)),
  );

  return {
    ...cache,
    cachedMatches: sortMatches([...activeMatches, ...Object.values(pendingMatches)]).slice(
      0,
      CLOUD_REPLAY_MATCHES_LIMIT,
    ),
    loadedAt,
    pendingMatches,
    sync,
  };
}

function cacheWithProfileSnapshot(
  cache: CloudHistoryUserCache,
  profile: CloudProfile,
  historyResetAt: string | null | undefined,
  loadedAt: string,
): CloudHistoryUserCache {
  const activeMatches = profile.matchHistory.replayMatches.filter((match) =>
    savedMatchIsAfterReset(match, historyResetAt)
  );
  const pendingMatches = Object.fromEntries(
    Object.entries(cache.pendingMatches).filter(([matchId, match]) => (
      savedMatchIsAfterReset(match, historyResetAt)
        && !cloudMatchHistoryHasMatch(profile.matchHistory, matchId)
    )),
  );
  const pendingMatchIds = new Set(Object.keys(pendingMatches));
  const sync = Object.fromEntries(
    Object.entries(cache.sync).filter(([matchId]) => pendingMatchIds.has(matchId)),
  );

  return {
    ...cache,
    cachedMatches: sortMatches([...activeMatches, ...Object.values(pendingMatches)]).slice(
      0,
      CLOUD_REPLAY_MATCHES_LIMIT,
    ),
    loadedAt,
    pendingMatches,
    sync,
  };
}

function cacheHasPendingError(cache: CloudHistoryUserCache): boolean {
  return Object.values(cache.sync).some((sync) => (
    sync.status === "error" && sync.matchId in cache.pendingMatches
  ));
}

function errorMessageAfterCacheRefresh(
  currentErrorMessage: string | null,
  cache: CloudHistoryUserCache,
): string | null {
  return cacheHasPendingError(cache) ? currentErrorMessage : null;
}

function syncStatusAfterCacheRefresh(
  currentSyncStatus: CloudHistorySyncStatus,
  cache: CloudHistoryUserCache,
): CloudHistorySyncStatus {
  if (currentSyncStatus !== "error") {
    return currentSyncStatus;
  }

  return cacheHasPendingError(cache) ? "error" : "idle";
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
  const cloudProfileForUser = options.cloudProfileForUser ?? ((uid) => {
    const profile = cloudProfileStore.getState().profile;
    return profile?.uid === uid ? profile : null;
  });
  const refreshCloudProfileForUser = options.refreshCloudProfileForUser ?? (async (user, preferredVariant) => {
    if (options.cloudProfileForUser) {
      return cloudProfileForUser(user.uid);
    }

    await cloudProfileStore.getState().loadForUser(user, preferredVariant);
    const profile = cloudProfileStore.getState().profile;
    return profile?.uid === user.uid ? profile : null;
  });
  const historyResetAtForUser = options.historyResetAtForUser ?? ((uid) => cloudProfileForUser(uid)?.resetAt);
  const loadHistory = options.loadHistory ?? (async (_user, historyResetAt) => {
    const profile = cloudProfileForUser(_user.uid);
    return profile
      ? profile.matchHistory.replayMatches.filter((match) => savedMatchIsAfterReset(match, historyResetAt))
      : [];
  });
  const saveHistory = options.saveHistory ?? saveCloudHistorySnapshot;
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
            set((state) => {
              const users = updateUserCache(state.users, user.uid, (cache) =>
                cacheWithLoadedMatches(cache, matches, historyResetAt, now())
              );
              return {
                errorMessage: errorMessageAfterCacheRefresh(
                  state.errorMessage,
                  users[user.uid] ?? defaultUserCache(),
                ),
                loadStatus: "ready",
                syncStatus: syncStatusAfterCacheRefresh(
                  state.syncStatus,
                  users[user.uid] ?? defaultUserCache(),
                ),
                users,
              };
            });
          } catch (error) {
            set({
              errorMessage: errorMessageFor(error),
              loadStatus: "error",
            });
          }
        },
        loadFromProfile: (user, profile, historyResetAt = profile.resetAt) => {
          set((state) => {
            const users = updateUserCache(state.users, user.uid, (cache) =>
              cacheWithProfileSnapshot(cache, profile, historyResetAt, now())
            );
            return {
              errorMessage: errorMessageAfterCacheRefresh(
                state.errorMessage,
                users[user.uid] ?? defaultUserCache(),
                ),
                loadStatus: "ready",
                syncStatus: syncStatusAfterCacheRefresh(
                  state.syncStatus,
                  users[user.uid] ?? defaultUserCache(),
                ),
                users,
              };
            });
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
            users: updateUserCache(state.users, user.uid, (cache) =>
              cacheWithPendingMatch(cache, match, "pending", null, now()),
            ),
          }));

          await get().syncPendingForUser(user, historyResetAt);
        },
        syncPendingForUser: async (user, historyResetAt = null) => {
          let profile = cloudProfileForUser(user.uid);
          if (profile) {
            const activeProfile = profile;
            const resetAt = latestResetAt(user.uid, historyResetAt ?? activeProfile.resetAt);
            set((state) => {
              const users = updateUserCache(state.users, user.uid, (cache) =>
                cacheWithProfileSnapshot(cache, activeProfile, resetAt, now())
              );
              return {
                errorMessage: errorMessageAfterCacheRefresh(
                  state.errorMessage,
                  users[user.uid] ?? defaultUserCache(),
                ),
                syncStatus: syncStatusAfterCacheRefresh(
                  state.syncStatus,
                  users[user.uid] ?? defaultUserCache(),
                ),
                users,
              };
            });
          }

          let cache = get().users[user.uid] ?? defaultUserCache();
          let pending = Object.values(cache.pendingMatches);
          if (!profile || pending.length === 0 || resettingUids.has(user.uid)) {
            return;
          }

          if (!cloudProfileSyncDue(profile)) {
            return;
          }

          const localProfile = localProfileStore.getState();
          const refreshedProfile = await refreshCloudProfileForUser(user, localProfile.settings.preferredVariant);
          profile = refreshedProfile ?? cloudProfileForUser(user.uid);
          if (profile) {
            const activeProfile = profile;
            const resetAt = latestResetAt(user.uid, historyResetAt ?? activeProfile.resetAt);
            set((state) => {
              const users = updateUserCache(state.users, user.uid, (currentCache) =>
                cacheWithProfileSnapshot(currentCache, activeProfile, resetAt, now())
              );
              return {
                errorMessage: errorMessageAfterCacheRefresh(
                  state.errorMessage,
                  users[user.uid] ?? defaultUserCache(),
                ),
                syncStatus: syncStatusAfterCacheRefresh(
                  state.syncStatus,
                  users[user.uid] ?? defaultUserCache(),
                ),
                users,
              };
            });
          }
          if (!profile || !cloudProfileSyncDue(profile)) {
            return;
          }

          const refreshedCache = get().users[user.uid] ?? defaultUserCache();
          const refreshedPending = Object.values(refreshedCache.pendingMatches);
          if (refreshedPending.length === 0 || resettingUids.has(user.uid)) {
            return;
          }

          const pendingMatchIds = new Set(refreshedPending.map((match) => match.id));

          set({
            errorMessage: null,
            syncStatus: "syncing",
          });

          const syncPromise = (async () => {
            const resetAt = latestResetAt(user.uid, historyResetAt);
            const candidateMatches = mergeCloudSavedMatches(
              user,
              [...profile.matchHistory.replayMatches, ...refreshedCache.cachedMatches, ...refreshedPending],
              resetAt,
            );
            const result = await saveHistory(user, {
              cloudProfile: profile,
              displayName: localProfile.profile?.displayName ?? profile.displayName,
              matches: candidateMatches,
              preferredVariant: localProfile.settings.preferredVariant,
            });

            cloudProfileStore.getState().applyLocalPatch({
              displayName: result.profile.displayName,
              matchHistory: result.profile.matchHistory,
              settings: result.profile.settings,
              updatedAt: result.profile.updatedAt,
            });

            set((state) => ({
              errorMessage: null,
              syncStatus: "idle",
              users: updateUserCache(state.users, user.uid, (currentCache) =>
                cacheWithSyncedMatches(currentCache, result.matches, pendingMatchIds),
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
              users: updateUserCache(state.users, user.uid, (currentCache) => ({
                ...currentCache,
                sync: Object.fromEntries(
                  Object.keys(currentCache.pendingMatches).map((matchId) => [
                    matchId,
                    {
                      errorMessage: message,
                      matchId,
                      status: "error" as const,
                      updatedAt: now(),
                    },
                  ]),
                ),
              })),
            }));
          } finally {
            const syncs = activeSyncs.get(user.uid);
            syncs?.delete(syncPromise);
            if (syncs?.size === 0) {
              activeSyncs.delete(user.uid);
            }
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
