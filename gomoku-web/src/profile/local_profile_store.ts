import { createStore, type StoreApi } from "zustand/vanilla";
import { createJSONStorage, persist } from "zustand/middleware";

import {
  CLOUD_REPLAY_MATCHES_LIMIT,
  archivedStatsFromDocument,
  emptyCloudArchivedMatchStats,
  isCloudMatchSummaryV1,
  mergeCloudMatchSummaryState,
  type CloudArchivedMatchStatsV1,
  type CloudMatchSummaryV1,
} from "../cloud/cloud_profile";
import type { GameVariant } from "../core/bot_protocol";
import type { MatchMove, MatchPlayer } from "../game/types";
import {
  createLocalSavedMatch,
  isLocalSavedMatchV1,
  type LocalSavedMatchV1,
  type SavedMatchStatus,
} from "../match/saved_match";

export interface LocalProfileStorage {
  getItem: (name: string) => string | null;
  setItem: (name: string, value: string) => void;
  removeItem: (name: string) => void;
}

export interface LocalProfileIdentity {
  avatarUrl: null;
  createdAt: string;
  displayName: string;
  id: string;
  kind: "local";
  updatedAt: string;
  username: null;
}

export interface LocalProfileSettings {
  preferredVariant: GameVariant;
}

export type LocalProfileSavedMatch = LocalSavedMatchV1;

export interface LocalProfileMatchHistory {
  archivedStats: CloudArchivedMatchStatsV1;
  replayMatches: LocalProfileSavedMatch[];
  summaryMatches: CloudMatchSummaryV1[];
}

export interface FinishedLocalMatchInput {
  mode: "bot";
  moves: MatchMove[];
  players: [MatchPlayer, MatchPlayer];
  status: SavedMatchStatus;
  undoFloor?: number;
  variant: GameVariant;
  winningCells?: unknown;
}

export interface LocalProfileState {
  ensureLocalProfile: () => LocalProfileIdentity;
  matchHistory: LocalProfileMatchHistory;
  profile: LocalProfileIdentity | null;
  recordFinishedMatch: (match: FinishedLocalMatchInput) => string;
  resetLocalProfile: () => void;
  renameDisplayName: (displayName: string) => void;
  settings: LocalProfileSettings;
  updateSettings: (patch: Partial<LocalProfileSettings>) => void;
}

export interface LocalProfileStoreOptions {
  storage?: LocalProfileStorage;
}

const STORAGE_KEY = "gomoku2d.local-profile.v3";
const STORAGE_SCHEMA_VERSION = 3;
export const DEFAULT_LOCAL_DISPLAY_NAME = "Guest";

function createDefaultSettings(): LocalProfileSettings {
  return {
    preferredVariant: "freestyle",
  };
}

function nowIso(): string {
  return new Date().toISOString();
}

function createId(): string {
  return globalThis.crypto?.randomUUID?.() ?? `local-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

interface PersistedLocalProfileState {
  matchHistory: unknown | null;
  profile: LocalProfileIdentity | null;
  settings: LocalProfileSettings;
}

function isLocalProfileIdentity(value: unknown): value is LocalProfileIdentity {
  const candidate = value as Partial<LocalProfileIdentity> | null;
  return candidate !== null
    && typeof candidate === "object"
    && candidate.avatarUrl === null
    && typeof candidate.createdAt === "string"
    && typeof candidate.displayName === "string"
    && typeof candidate.id === "string"
    && candidate.id.length > 0
    && candidate.kind === "local"
    && typeof candidate.updatedAt === "string"
    && candidate.username === null;
}

function settingsFromRaw(value: unknown): LocalProfileSettings {
  const candidate = value as Partial<LocalProfileSettings> | null;
  return {
    preferredVariant: candidate?.preferredVariant === "renju" ? "renju" : "freestyle",
  };
}

function persistedStateFromRaw(raw: string | null): PersistedLocalProfileState | null {
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw) as { state?: Partial<PersistedLocalProfileState> };
    return {
      matchHistory: parsed.state?.matchHistory ?? null,
      profile: isLocalProfileIdentity(parsed.state?.profile) ? parsed.state.profile : null,
      settings: settingsFromRaw(parsed.state?.settings),
    };
  } catch {
    return null;
  }
}

export function emptyLocalMatchHistory(): LocalProfileMatchHistory {
  return {
    archivedStats: emptyCloudArchivedMatchStats(),
    replayMatches: [],
    summaryMatches: [],
  };
}

function sortLocalProfileMatches(matches: LocalProfileSavedMatch[]): LocalProfileSavedMatch[] {
  return [...matches].sort((left, right) => right.saved_at.localeCompare(left.saved_at));
}

function mergeLocalProfileSavedMatches(matches: LocalProfileSavedMatch[]): LocalProfileSavedMatch[] {
  const byId = new Map<string, LocalProfileSavedMatch>();

  for (const match of matches) {
    const existing = byId.get(match.id);
    if (!existing || match.saved_at > existing.saved_at) {
      byId.set(match.id, match);
    }
  }

  return sortLocalProfileMatches(Array.from(byId.values()));
}

function mergeLocalProfileMatchHistory(
  history: LocalProfileMatchHistory,
  localProfileId: string,
  matches: LocalProfileSavedMatch[],
): LocalProfileMatchHistory {
  const candidates = mergeLocalProfileSavedMatches([...matches, ...history.replayMatches]);
  const replayMatches = candidates.slice(0, CLOUD_REPLAY_MATCHES_LIMIT);
  const summaryState = mergeCloudMatchSummaryState({
    archivedStats: history.archivedStats,
    convertLocalMatches: false,
    identity: { localProfileId },
    matches: candidates,
    replayMatches,
    summaries: history.summaryMatches,
  });

  return {
    archivedStats: summaryState.archivedStats,
    replayMatches,
    summaryMatches: summaryState.summaryMatches,
  };
}

function localSummaryMatchesFromRaw(value: unknown): CloudMatchSummaryV1[] {
  return Array.isArray(value)
    ? value
      .filter(isCloudMatchSummaryV1)
      .sort((left, right) => right.saved_at.localeCompare(left.saved_at))
    : [];
}

function localReplayMatchesFromRaw(value: unknown): LocalProfileSavedMatch[] {
  return Array.isArray(value)
    ? value
      .filter(isLocalSavedMatchV1)
      .sort((left, right) => right.saved_at.localeCompare(left.saved_at))
      .slice(0, CLOUD_REPLAY_MATCHES_LIMIT)
    : [];
}

function matchHistoryFromPersisted(persisted: PersistedLocalProfileState): LocalProfileMatchHistory {
  const candidate = persisted.matchHistory as {
    archivedStats?: unknown;
    replayMatches?: unknown;
    summaryMatches?: unknown;
  } | null;
  const localProfileId = persisted.profile?.id ?? "unknown-local";
  const base: LocalProfileMatchHistory = {
    archivedStats: archivedStatsFromDocument(candidate?.archivedStats),
    replayMatches: [],
    summaryMatches: localSummaryMatchesFromRaw(candidate?.summaryMatches),
  };

  return mergeLocalProfileMatchHistory(base, localProfileId, localReplayMatchesFromRaw(candidate?.replayMatches));
}

function validatedLocalProfileStorage(storage: LocalProfileStorage): LocalProfileStorage {
  return {
    getItem: (name) => {
      const raw = storage.getItem(name);
      if (name !== STORAGE_KEY) {
        return raw;
      }

      const persisted = persistedStateFromRaw(raw);
      if (!persisted) {
        return null;
      }

      const sanitized: Pick<LocalProfileState, "matchHistory" | "profile" | "settings"> = {
        matchHistory: matchHistoryFromPersisted(persisted),
        profile: persisted.profile,
        settings: persisted.settings,
      };

      return JSON.stringify({ state: sanitized, version: STORAGE_SCHEMA_VERSION });
    },
    removeItem: (name) => {
      storage.removeItem(name);
    },
    setItem: (name, value) => {
      storage.setItem(name, value);
    },
  };
}

export function createLocalProfileStore(
  options: LocalProfileStoreOptions = {},
): StoreApi<LocalProfileState> {
  const baseStorage = options.storage ?? localStorage;

  const storage = createJSONStorage<Pick<LocalProfileState, "matchHistory" | "profile" | "settings">>(
    () => validatedLocalProfileStorage(baseStorage),
  );

  return createStore<LocalProfileState>()(
    persist(
      (set, get) => ({
        ensureLocalProfile: () => {
          const existing = get().profile;
          if (existing) {
            return existing;
          }

          const created = nowIso();
          const profile: LocalProfileIdentity = {
            avatarUrl: null,
            createdAt: created,
            displayName: DEFAULT_LOCAL_DISPLAY_NAME,
            id: createId(),
            kind: "local",
            updatedAt: created,
            username: null,
          };

          set({ profile });
          return profile;
        },
        matchHistory: emptyLocalMatchHistory(),
        profile: null,
        recordFinishedMatch: (match) => {
          const profile = get().ensureLocalProfile();
          const savedAt = nowIso();
          const id = createId();
          const record = createLocalSavedMatch({
            id,
            localProfileId: profile.id,
            moves: match.moves,
            players: match.players,
            savedAt,
            status: match.status,
            undoFloor: match.undoFloor,
            variant: match.variant,
          });

          set((state) => ({
            matchHistory: mergeLocalProfileMatchHistory(state.matchHistory, profile.id, [record]),
            profile: { ...profile, updatedAt: savedAt },
          }));

          return id;
        },
        resetLocalProfile: () => {
          set({
            matchHistory: emptyLocalMatchHistory(),
            profile: null,
            settings: createDefaultSettings(),
          });
        },
        renameDisplayName: (displayName) => {
          const profile = get().ensureLocalProfile();
          const nextName = displayName.trim() || DEFAULT_LOCAL_DISPLAY_NAME;
          set({
            profile: {
              ...profile,
              displayName: nextName,
              updatedAt: nowIso(),
            },
          });
        },
        settings: createDefaultSettings(),
        updateSettings: (patch) => {
          const nextSettings = { ...get().settings, ...patch };
          set({ settings: nextSettings });
        },
      }),
      {
        name: STORAGE_KEY,
        version: STORAGE_SCHEMA_VERSION,
        partialize: (state) => ({
          matchHistory: state.matchHistory,
          profile: state.profile,
          settings: state.settings,
        }),
        storage,
      },
    ),
  );
}

export const localProfileStore = createLocalProfileStore();
