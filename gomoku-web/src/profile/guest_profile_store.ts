import { createStore, type StoreApi } from "zustand/vanilla";
import { createJSONStorage, persist } from "zustand/middleware";

import type { GameVariant } from "../core/bot_protocol";
import type { MatchMove, MatchPlayer } from "../game/types";
import {
  createLocalSavedMatch,
  isLocalSavedMatchV1,
  migrateLegacyGuestSavedMatch,
  type LegacyGuestSavedMatch,
  type LocalSavedMatchV1,
  type SavedMatchStatus,
  type SavedMatchV1,
} from "../match/saved_match";

export interface GuestProfileStorage {
  getItem: (name: string) => string | null;
  setItem: (name: string, value: string) => void;
  removeItem: (name: string) => void;
}

export interface GuestProfileIdentity {
  avatarUrl: null;
  createdAt: string;
  displayName: string;
  id: string;
  kind: "guest";
  updatedAt: string;
  username: null;
}

export interface GuestProfileSettings {
  preferredVariant: GameVariant;
}

export type GuestSavedMatch = LocalSavedMatchV1;

export interface FinishedGuestMatchInput {
  mode: "bot";
  moves: MatchMove[];
  players: [MatchPlayer, MatchPlayer];
  status: SavedMatchStatus;
  undoFloor?: number;
  variant: GameVariant;
  winningCells?: unknown;
}

export interface GuestProfileState {
  ensureGuestProfile: () => GuestProfileIdentity;
  history: GuestSavedMatch[];
  profile: GuestProfileIdentity | null;
  recordFinishedMatch: (match: FinishedGuestMatchInput) => string;
  resetGuestProfile: () => void;
  renameDisplayName: (displayName: string) => void;
  settings: GuestProfileSettings;
  updateSettings: (patch: Partial<GuestProfileSettings>) => void;
}

export interface GuestProfileStoreOptions {
  storage?: GuestProfileStorage;
}

const LEGACY_STORAGE_KEY = "gomoku2d.guest-profile.v1";
const STORAGE_KEY = "gomoku2d.guest-profile.v2";
const HISTORY_LIMIT = 24;
export const DEFAULT_GUEST_DISPLAY_NAME = "Guest";

function createDefaultSettings(): GuestProfileSettings {
  return {
    preferredVariant: "freestyle",
  };
}

function nowIso(): string {
  return new Date().toISOString();
}

function createId(): string {
  return globalThis.crypto?.randomUUID?.() ?? `guest-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

interface PersistedGuestProfileState {
  history: unknown[];
  profile: GuestProfileIdentity | null;
  settings: GuestProfileSettings;
}

function persistedStateFromRaw(raw: string | null): PersistedGuestProfileState | null {
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw) as { state?: Partial<PersistedGuestProfileState> };
    return {
      history: Array.isArray(parsed.state?.history) ? parsed.state.history : [],
      profile: parsed.state?.profile ?? null,
      settings: parsed.state?.settings ?? createDefaultSettings(),
    };
  } catch {
    return null;
  }
}

function migrateHistory(history: unknown[], profile: GuestProfileIdentity | null): GuestSavedMatch[] {
  const localProfileId = profile?.id ?? "legacy-guest";

  return history.flatMap((match) => {
    if (isLocalSavedMatchV1(match)) {
      return [match];
    }

    const candidate = match as Partial<SavedMatchV1>;
    if (candidate.schema_version === 1) {
      return [];
    }

    try {
      return [migrateLegacyGuestSavedMatch(match as LegacyGuestSavedMatch, localProfileId)];
    } catch {
      return [];
    }
  });
}

function migrateGuestProfileStorage(storage: GuestProfileStorage): void {
  if (storage.getItem(STORAGE_KEY)) {
    return;
  }

  const legacy = persistedStateFromRaw(storage.getItem(LEGACY_STORAGE_KEY));
  if (!legacy) {
    return;
  }

  const migrated: Pick<GuestProfileState, "history" | "profile" | "settings"> = {
    history: migrateHistory(legacy.history, legacy.profile),
    profile: legacy.profile,
    settings: legacy.settings,
  };

  storage.setItem(STORAGE_KEY, JSON.stringify({ state: migrated, version: 0 }));
}

function validatedGuestProfileStorage(storage: GuestProfileStorage): GuestProfileStorage {
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

      const sanitized: Pick<GuestProfileState, "history" | "profile" | "settings"> = {
        history: migrateHistory(persisted.history, persisted.profile),
        profile: persisted.profile,
        settings: persisted.settings,
      };

      return JSON.stringify({ state: sanitized, version: 0 });
    },
    removeItem: (name) => {
      storage.removeItem(name);
    },
    setItem: (name, value) => {
      storage.setItem(name, value);
    },
  };
}

export function createGuestProfileStore(
  options: GuestProfileStoreOptions = {},
): StoreApi<GuestProfileState> {
  const baseStorage = options.storage ?? localStorage;
  migrateGuestProfileStorage(baseStorage);

  const storage = createJSONStorage<Pick<GuestProfileState, "history" | "profile" | "settings">>(
    () => validatedGuestProfileStorage(baseStorage),
  );

  return createStore<GuestProfileState>()(
    persist(
      (set, get) => ({
        ensureGuestProfile: () => {
          const existing = get().profile;
          if (existing) {
            return existing;
          }

          const created = nowIso();
          const profile: GuestProfileIdentity = {
            avatarUrl: null,
            createdAt: created,
            displayName: DEFAULT_GUEST_DISPLAY_NAME,
            id: createId(),
            kind: "guest",
            updatedAt: created,
            username: null,
          };

          set({ profile });
          return profile;
        },
        history: [],
        profile: null,
        recordFinishedMatch: (match) => {
          const profile = get().ensureGuestProfile();
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
            history: [record, ...state.history].slice(0, HISTORY_LIMIT),
            profile: { ...profile, updatedAt: savedAt },
          }));

          return id;
        },
        resetGuestProfile: () => {
          set({
            history: [],
            profile: null,
            settings: createDefaultSettings(),
          });
        },
        renameDisplayName: (displayName) => {
          const profile = get().ensureGuestProfile();
          const nextName = displayName.trim() || DEFAULT_GUEST_DISPLAY_NAME;
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
        partialize: (state) => ({
          history: state.history,
          profile: state.profile,
          settings: state.settings,
        }),
        storage,
      },
    ),
  );
}

export const guestProfileStore = createGuestProfileStore();
