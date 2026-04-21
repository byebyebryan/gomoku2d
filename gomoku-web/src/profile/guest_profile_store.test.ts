import { describe, expect, it } from "vitest";

import type { GuestProfileStorage } from "./guest_profile_store";
import { createGuestProfileStore } from "./guest_profile_store";

function createMemoryStorage(): GuestProfileStorage {
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

describe("createGuestProfileStore", () => {
  it("creates a guest profile on first meaningful interaction and persists edits", () => {
    const storage = createMemoryStorage();
    const store = createGuestProfileStore({ storage });

    expect(store.getState().profile).toBeNull();

    const profile = store.getState().ensureGuestProfile();
    expect(profile.kind).toBe("guest");
    expect(profile.displayName).toBe("Guest");

    store.getState().renameDisplayName("Bryan Guest");

    const reloadedStore = createGuestProfileStore({ storage });
    expect(reloadedStore.getState().profile).toMatchObject({
      displayName: "Bryan Guest",
      id: profile.id,
      kind: "guest",
    });
    expect(reloadedStore.getState().settings).toEqual({
      preferredVariant: "freestyle",
    });
  });

  it("persists the preferred rules variant", () => {
    const storage = createMemoryStorage();
    const store = createGuestProfileStore({ storage });

    store.getState().updateSettings({ preferredVariant: "renju" });

    const reloadedStore = createGuestProfileStore({ storage });
    expect(reloadedStore.getState().settings.preferredVariant).toBe("renju");
  });

  it("records finished guest matches and keeps newest history first", () => {
    const store = createGuestProfileStore({ storage: createMemoryStorage() });
    store.getState().ensureGuestProfile();

    store.getState().recordFinishedMatch({
      mode: "bot",
      moves: [
        { col: 7, moveNumber: 1, player: 1, row: 7 },
        { col: 6, moveNumber: 2, player: 2, row: 5 },
      ],
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Classic Bot", stone: "white" },
      ],
      status: "white_won",
      variant: "freestyle",
      winningCells: [
        { row: 5, col: 6 },
        { row: 6, col: 6 },
      ],
    });

    store.getState().recordFinishedMatch({
      mode: "bot",
      moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
      players: [
        { kind: "bot", name: "Classic Bot", stone: "black" },
        { kind: "human", name: "Guest", stone: "white" },
      ],
      status: "draw",
      variant: "freestyle",
      winningCells: [],
    });

    const state = store.getState();

    expect(state.history).toHaveLength(2);
    expect(state.history[0]).toMatchObject({
      guestStone: "white",
      status: "draw",
    });
    expect(state.history[1]).toMatchObject({
      guestStone: "black",
      status: "white_won",
    });
  });

  it("resets guest identity, history, and settings", () => {
    const storage = createMemoryStorage();
    const store = createGuestProfileStore({ storage });

    store.getState().ensureGuestProfile();
    store.getState().renameDisplayName("Bryan Guest");
    store.getState().updateSettings({ preferredVariant: "renju" });
    store.getState().recordFinishedMatch({
      mode: "bot",
      moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
      players: [
        { kind: "human", name: "Bryan Guest", stone: "black" },
        { kind: "bot", name: "Classic Bot", stone: "white" },
      ],
      status: "draw",
      variant: "freestyle",
      winningCells: [],
    });

    store.getState().resetGuestProfile();

    const resetState = store.getState();
    expect(resetState.history).toEqual([]);
    expect(resetState.profile).toBeNull();
    expect(resetState.settings).toEqual({
      preferredVariant: "freestyle",
    });
  });
});
