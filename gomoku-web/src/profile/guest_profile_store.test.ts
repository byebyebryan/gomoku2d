import { describe, expect, it } from "vitest";

import { movesFromMoveCells } from "../match/saved_match";

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
        { col: 8, moveNumber: 3, player: 1, row: 7 },
        { col: 6, moveNumber: 4, player: 2, row: 6 },
        { col: 9, moveNumber: 5, player: 1, row: 7 },
        { col: 6, moveNumber: 6, player: 2, row: 7 },
      ],
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Practice Bot", stone: "white" },
      ],
      status: "white_won",
      undoFloor: 5,
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
        { kind: "bot", name: "Practice Bot", stone: "black" },
        { kind: "human", name: "Guest", stone: "white" },
      ],
      status: "draw",
      variant: "freestyle",
      winningCells: [],
    });

    const state = store.getState();

    expect(state.history).toHaveLength(2);
    expect(state.history[0]).toMatchObject({
      match_kind: "local_vs_bot",
      move_cells: [112],
      move_count: 1,
      player_black: {
        bot: expect.objectContaining({ id: "practice_bot" }),
        kind: "bot",
      },
      player_white: {
        kind: "human",
        local_profile_id: state.profile?.id,
      },
      status: "draw",
      undo_floor: 0,
    });
    expect(state.history[1]).toMatchObject({
      match_kind: "local_vs_bot",
      move_cells: [112, 81, 113, 96, 114, 111],
      move_count: 6,
      player_black: {
        kind: "human",
        local_profile_id: state.profile?.id,
      },
      player_white: {
        bot: expect.objectContaining({ engine: "baseline_search", id: "practice_bot" }),
        kind: "bot",
      },
      status: "white_won",
      undo_floor: 5,
    });
  });

  it("migrates legacy v1 local history into canonical v2 storage", () => {
    const storage = createMemoryStorage();
    storage.setItem(
      "gomoku2d.guest-profile.v1",
      JSON.stringify({
        state: {
          history: [
            {
              guestStone: "black",
              id: "legacy-match",
              mode: "bot",
              moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
              players: [
                { kind: "human", name: "Guest", stone: "black" },
                { kind: "bot", name: "Practice Bot", stone: "white" },
              ],
              savedAt: "2026-04-21T12:00:00.000Z",
              status: "draw",
              undoFloor: 8,
              variant: "renju",
              winningCells: [],
            },
          ],
          profile: {
            avatarUrl: null,
            createdAt: "2026-04-20T12:00:00.000Z",
            displayName: "Guest",
            id: "guest-1",
            kind: "guest",
            updatedAt: "2026-04-20T12:00:00.000Z",
            username: null,
          },
          settings: { preferredVariant: "renju" },
        },
        version: 0,
      }),
    );

    const store = createGuestProfileStore({ storage });
    const migrated = store.getState().history[0];

    expect(storage.getItem("gomoku2d.guest-profile.v1")).not.toBeNull();
    expect(storage.getItem("gomoku2d.guest-profile.v2")).not.toBeNull();
    expect(migrated).toMatchObject({
      id: "legacy-match",
      match_kind: "local_vs_bot",
      move_cells: [112],
      move_count: 1,
      player_black: {
        kind: "human",
        local_profile_id: "guest-1",
      },
      saved_at: "2026-04-21T12:00:00.000Z",
      schema_version: 1,
      source: "local_history",
      trust: "local_only",
      undo_floor: 1,
      variant: "renju",
    });
    expect(movesFromMoveCells(migrated.move_cells)).toEqual([
      { col: 7, moveNumber: 1, player: 1, row: 7 },
    ]);
  });

  it("drops malformed schema v1 records instead of loading them", () => {
    const storage = createMemoryStorage();
    storage.setItem(
      "gomoku2d.guest-profile.v2",
      JSON.stringify({
        state: {
          history: [
            {
              id: "bad-match",
              move_count: 1,
              saved_at: "2026-04-21T12:00:00.000Z",
              schema_version: 1,
              status: "draw",
            },
          ],
          profile: {
            avatarUrl: null,
            createdAt: "2026-04-20T12:00:00.000Z",
            displayName: "Guest",
            id: "guest-1",
            kind: "guest",
            updatedAt: "2026-04-20T12:00:00.000Z",
            username: null,
          },
          settings: { preferredVariant: "freestyle" },
        },
        version: 0,
      }),
    );

    const store = createGuestProfileStore({ storage });

    expect(store.getState().history).toEqual([]);
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
        { kind: "bot", name: "Practice Bot", stone: "white" },
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
