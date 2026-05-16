import { createStore, type StoreApi } from "zustand/vanilla";
import { createJSONStorage, persist } from "zustand/middleware";

export type TouchControlMode = "pointer" | "touchpad";
export type ImmediateHintMode = "off" | "win" | "win_threat";
export type ImminentHintMode = "off" | "threat" | "threat_counter";

export interface BoardHintSettings {
  immediate: ImmediateHintMode;
  imminent: ImminentHintMode;
}

export interface UiPreferencesStorage {
  getItem: (name: string) => string | null;
  setItem: (name: string, value: string) => void;
  removeItem: (name: string) => void;
}

export interface UiPreferencesState {
  boardHints: BoardHintSettings;
  setBoardHints: (boardHints: Partial<BoardHintSettings>) => void;
  setTouchControl: (touchControl: TouchControlMode) => void;
  touchControl: TouchControlMode;
}

export interface UiPreferencesStoreOptions {
  storage?: UiPreferencesStorage;
}

const STORAGE_KEY = "gomoku2d.ui-preferences.v1";
const STORAGE_SCHEMA_VERSION = 1;
const DEFAULT_BOARD_HINTS: BoardHintSettings = {
  immediate: "win_threat",
  imminent: "threat_counter",
};

function createDefaultBoardHints(): BoardHintSettings {
  return { ...DEFAULT_BOARD_HINTS };
}

function sanitizeTouchControl(value: unknown): TouchControlMode {
  return value === "pointer" ? "pointer" : "touchpad";
}

function sanitizeImmediateHintMode(value: unknown): ImmediateHintMode {
  return value === "off" || value === "win" || value === "win_threat"
    ? value
    : DEFAULT_BOARD_HINTS.immediate;
}

function sanitizeImminentHintMode(value: unknown): ImminentHintMode {
  return value === "off" || value === "threat" || value === "threat_counter"
    ? value
    : DEFAULT_BOARD_HINTS.imminent;
}

function sanitizeBoardHints(value: unknown): BoardHintSettings {
  const candidate = value as Partial<Record<keyof BoardHintSettings, unknown>> | null;

  return {
    immediate: sanitizeImmediateHintMode(candidate?.immediate),
    imminent: sanitizeImminentHintMode(candidate?.imminent),
  };
}

function sanitizedRaw(raw: string | null): string | null {
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw) as {
      state?: Partial<Pick<UiPreferencesState, "boardHints" | "touchControl">>;
    };
    return JSON.stringify({
      state: {
        boardHints: sanitizeBoardHints(parsed.state?.boardHints),
        touchControl: sanitizeTouchControl(parsed.state?.touchControl),
      },
      version: STORAGE_SCHEMA_VERSION,
    });
  } catch {
    return null;
  }
}

function validatedUiPreferencesStorage(storage: UiPreferencesStorage): UiPreferencesStorage {
  return {
    getItem: (name) => (
      name === STORAGE_KEY ? sanitizedRaw(storage.getItem(name)) : storage.getItem(name)
    ),
    removeItem: (name) => {
      storage.removeItem(name);
    },
    setItem: (name, value) => {
      storage.setItem(name, value);
    },
  };
}

export function createUiPreferencesStore(
  options: UiPreferencesStoreOptions = {},
): StoreApi<UiPreferencesState> {
  const baseStorage = options.storage ?? localStorage;
  const storage = createJSONStorage<Pick<UiPreferencesState, "boardHints" | "touchControl">>(
    () => validatedUiPreferencesStorage(baseStorage),
  );

  return createStore<UiPreferencesState>()(
    persist(
      (set) => ({
        boardHints: createDefaultBoardHints(),
        setBoardHints: (boardHints) => {
          set((state) => ({
            boardHints: sanitizeBoardHints({
              ...state.boardHints,
              ...boardHints,
            }),
          }));
        },
        setTouchControl: (touchControl) => {
          set({ touchControl: sanitizeTouchControl(touchControl) });
        },
        touchControl: "touchpad",
      }),
      {
        name: STORAGE_KEY,
        partialize: (state) => ({
          boardHints: sanitizeBoardHints(state.boardHints),
          touchControl: state.touchControl,
        }),
        storage,
        version: STORAGE_SCHEMA_VERSION,
      },
    ),
  );
}

export const uiPreferencesStore = createUiPreferencesStore();
