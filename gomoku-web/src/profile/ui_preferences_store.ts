import { createStore, type StoreApi } from "zustand/vanilla";
import { createJSONStorage, persist } from "zustand/middleware";

export type TouchControlMode = "pointer" | "touchpad";

export interface UiPreferencesStorage {
  getItem: (name: string) => string | null;
  setItem: (name: string, value: string) => void;
  removeItem: (name: string) => void;
}

export interface UiPreferencesState {
  setTouchControl: (touchControl: TouchControlMode) => void;
  touchControl: TouchControlMode;
}

export interface UiPreferencesStoreOptions {
  storage?: UiPreferencesStorage;
}

const STORAGE_KEY = "gomoku2d.ui-preferences.v1";
const STORAGE_SCHEMA_VERSION = 1;

function sanitizeTouchControl(value: unknown): TouchControlMode {
  return value === "pointer" ? "pointer" : "touchpad";
}

function sanitizedRaw(raw: string | null): string | null {
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw) as { state?: Partial<Pick<UiPreferencesState, "touchControl">> };
    return JSON.stringify({
      state: { touchControl: sanitizeTouchControl(parsed.state?.touchControl) },
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
  const storage = createJSONStorage<Pick<UiPreferencesState, "touchControl">>(
    () => validatedUiPreferencesStorage(baseStorage),
  );

  return createStore<UiPreferencesState>()(
    persist(
      (set) => ({
        setTouchControl: (touchControl) => {
          set({ touchControl: sanitizeTouchControl(touchControl) });
        },
        touchControl: "touchpad",
      }),
      {
        name: STORAGE_KEY,
        partialize: (state) => ({ touchControl: state.touchControl }),
        storage,
        version: STORAGE_SCHEMA_VERSION,
      },
    ),
  );
}

export const uiPreferencesStore = createUiPreferencesStore();
