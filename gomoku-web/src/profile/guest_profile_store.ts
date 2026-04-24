import { createStore, type StoreApi } from "zustand/vanilla";
import { createJSONStorage, persist } from "zustand/middleware";

import type { GameVariant } from "../core/bot_protocol";
import type { CellPosition, MatchMove, MatchPlayer, MatchStatus } from "../game/types";

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

export interface GuestSavedMatch {
  guestStone: "black" | "white";
  id: string;
  mode: "bot";
  moves: MatchMove[];
  players: [MatchPlayer, MatchPlayer];
  savedAt: string;
  status: MatchStatus;
  undoFloor?: number;
  variant: GameVariant;
  winningCells: CellPosition[];
}

export interface GuestProfileState {
  ensureGuestProfile: () => GuestProfileIdentity;
  history: GuestSavedMatch[];
  profile: GuestProfileIdentity | null;
  recordFinishedMatch: (match: Omit<GuestSavedMatch, "guestStone" | "id" | "savedAt">) => string;
  resetGuestProfile: () => void;
  renameDisplayName: (displayName: string) => void;
  settings: GuestProfileSettings;
  updateSettings: (patch: Partial<GuestProfileSettings>) => void;
}

export interface GuestProfileStoreOptions {
  storage?: GuestProfileStorage;
}

const STORAGE_KEY = "gomoku2d.guest-profile.v1";
const HISTORY_LIMIT = 24;

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

function cloneMoves(moves: MatchMove[]): MatchMove[] {
  return moves.map((move) => ({ ...move }));
}

function clonePlayers(players: [MatchPlayer, MatchPlayer]): [MatchPlayer, MatchPlayer] {
  return [{ ...players[0] }, { ...players[1] }];
}

function cloneWinningCells(cells: CellPosition[]): CellPosition[] {
  return cells.map((cell) => ({ ...cell }));
}

function deriveGuestStone(players: [MatchPlayer, MatchPlayer]): "black" | "white" {
  const human = players.find((player) => player.kind === "human");
  return human?.stone ?? "black";
}

function normalizeUndoFloor(undoFloor: number | undefined, moveCount: number): number {
  if (undoFloor === undefined || !Number.isFinite(undoFloor)) {
    return 0;
  }

  return Math.max(0, Math.min(moveCount, Math.floor(undoFloor)));
}

export function createGuestProfileStore(
  options: GuestProfileStoreOptions = {},
): StoreApi<GuestProfileState> {
  const storage = createJSONStorage<Pick<GuestProfileState, "history" | "profile" | "settings">>(
    () => options.storage ?? localStorage,
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
            displayName: "Guest",
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
          const record: GuestSavedMatch = {
            guestStone: deriveGuestStone(match.players),
            id: createId(),
            mode: match.mode,
            moves: cloneMoves(match.moves),
            players: clonePlayers(match.players),
            savedAt,
            status: match.status,
            undoFloor: normalizeUndoFloor(match.undoFloor, match.moves.length),
            variant: match.variant,
            winningCells: cloneWinningCells(match.winningCells),
          };

          set((state) => ({
            history: [record, ...state.history].slice(0, HISTORY_LIMIT),
            profile: { ...profile, updatedAt: savedAt },
          }));

          return record.id;
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
          const nextName = displayName.trim() || "Guest";
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
