import { describe, expect, it } from "vitest";

import type { UiPreferencesStorage } from "./ui_preferences_store";
import { createUiPreferencesStore } from "./ui_preferences_store";

function createMemoryStorage(): UiPreferencesStorage {
  const backing = new Map<string, string>();

  return {
    getItem: (name) => backing.get(name) ?? null,
    removeItem: (name) => {
      backing.delete(name);
    },
    setItem: (name, value) => {
      backing.set(name, value);
    },
  };
}

describe("createUiPreferencesStore", () => {
  it("defaults to the existing touchpad placement mode", () => {
    const store = createUiPreferencesStore({ storage: createMemoryStorage() });

    expect(store.getState().touchControl).toBe("touchpad");
    expect(store.getState().boardHints).toEqual({
      immediate: "win_threat",
      imminent: "threat_counter",
    });
  });

  it("persists the selected touch control mode locally", () => {
    const storage = createMemoryStorage();
    const store = createUiPreferencesStore({ storage });

    store.getState().setTouchControl("pointer");

    const reloadedStore = createUiPreferencesStore({ storage });
    expect(reloadedStore.getState().touchControl).toBe("pointer");
  });

  it("sanitizes invalid persisted touch control values", () => {
    const storage = createMemoryStorage();
    storage.setItem("gomoku2d.ui-preferences.v1", JSON.stringify({
      state: { touchControl: "tap-to-place" },
      version: 1,
    }));

    const store = createUiPreferencesStore({ storage });

    expect(store.getState().touchControl).toBe("touchpad");
  });

  it("persists selected board hint modes locally", () => {
    const storage = createMemoryStorage();
    const store = createUiPreferencesStore({ storage });

    store.getState().setBoardHints({
      immediate: "win",
      imminent: "off",
    });

    const reloadedStore = createUiPreferencesStore({ storage });
    expect(reloadedStore.getState().boardHints).toEqual({
      immediate: "win",
      imminent: "off",
    });
  });

  it("sanitizes invalid persisted board hint values", () => {
    const storage = createMemoryStorage();
    storage.setItem("gomoku2d.ui-preferences.v1", JSON.stringify({
      state: {
        boardHints: {
          immediate: "win",
          imminent: "invalid",
        },
        touchControl: "pointer",
      },
      version: 1,
    }));

    const store = createUiPreferencesStore({ storage });

    expect(store.getState().touchControl).toBe("pointer");
    expect(store.getState().boardHints).toEqual({
      immediate: "win",
      imminent: "threat_counter",
    });
  });
});
