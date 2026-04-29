import { describe, expect, it } from "vitest";

import { CLOUD_REPLAY_MATCHES_LIMIT } from "../cloud/cloud_profile";
import { createLocalSavedMatch } from "../match/saved_match";

import type { LocalProfileStorage } from "./local_profile_store";
import { createLocalProfileStore } from "./local_profile_store";

function createMemoryStorage(): LocalProfileStorage {
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

describe("createLocalProfileStore", () => {
  it("creates a local profile on first meaningful interaction and persists edits", () => {
    const storage = createMemoryStorage();
    const store = createLocalProfileStore({ storage });

    expect(store.getState().profile).toBeNull();

    const profile = store.getState().ensureLocalProfile();
    expect(profile.kind).toBe("local");
    expect(profile.displayName).toBe("Guest");

    store.getState().renameDisplayName("Bryan Local");

    const reloadedStore = createLocalProfileStore({ storage });
    expect(reloadedStore.getState().profile).toMatchObject({
      displayName: "Bryan Local",
      id: profile.id,
      kind: "local",
    });
    expect(reloadedStore.getState().settings).toEqual({
      preferredVariant: "freestyle",
    });
  });

  it("persists the preferred rules variant", () => {
    const storage = createMemoryStorage();
    const store = createLocalProfileStore({ storage });

    store.getState().updateSettings({ preferredVariant: "renju" });

    const reloadedStore = createLocalProfileStore({ storage });
    expect(reloadedStore.getState().settings.preferredVariant).toBe("renju");
  });

  it("records finished local matches and keeps newest history first", () => {
    const store = createLocalProfileStore({ storage: createMemoryStorage() });
    store.getState().ensureLocalProfile();

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

    expect(state.matchHistory.replayMatches).toHaveLength(2);
    expect(state.matchHistory.replayMatches[0]).toMatchObject({
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
    expect(state.matchHistory.replayMatches[1]).toMatchObject({
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

  it("keeps older local matches in the summary tier after replay overflow", () => {
    const store = createLocalProfileStore({ storage: createMemoryStorage() });
    store.getState().ensureLocalProfile();

    for (let index = 0; index <= CLOUD_REPLAY_MATCHES_LIMIT; index += 1) {
      store.getState().recordFinishedMatch({
        mode: "bot",
        moves: [{ col: index % 15, moveNumber: 1, player: 1, row: 7 }],
        players: [
          { kind: "human", name: "Guest", stone: "black" },
          { kind: "bot", name: "Practice Bot", stone: "white" },
        ],
        status: "draw",
        variant: "freestyle",
        winningCells: [],
      });
    }

    const history = store.getState().matchHistory;

    expect(history.replayMatches).toHaveLength(CLOUD_REPLAY_MATCHES_LIMIT);
    expect(history.summaryMatches).toHaveLength(1);
    expect(history.summaryMatches[0]).toMatchObject({
      move_count: 1,
      outcome: "draw",
      trust: "local_only",
    });
  });

  it("ignores legacy local-profile keys instead of migrating them into local v3", () => {
    const storage = createMemoryStorage();
    storage.setItem(
      "gomoku2d.guest-profile.v2",
      JSON.stringify({
        state: {
          history: [{ id: "legacy-match" }],
          profile: {
            avatarUrl: null,
            createdAt: "2026-04-20T12:00:00.000Z",
            displayName: "Guest",
            id: "local-1",
            kind: "local",
            updatedAt: "2026-04-20T12:00:00.000Z",
            username: null,
          },
          settings: { preferredVariant: "freestyle" },
        },
        version: 0,
      }),
    );

    const store = createLocalProfileStore({ storage });

    expect(storage.getItem("gomoku2d.guest-profile.v2")).not.toBeNull();
    expect(storage.getItem("gomoku2d.local-profile.v3")).toBeNull();
    expect(store.getState().profile).toBeNull();
    expect(store.getState().matchHistory.replayMatches).toEqual([]);
  });

  it("resets local identity, history, and settings", () => {
    const storage = createMemoryStorage();
    const store = createLocalProfileStore({ storage });

    store.getState().ensureLocalProfile();
    store.getState().renameDisplayName("Bryan Local");
    store.getState().updateSettings({ preferredVariant: "renju" });
    store.getState().recordFinishedMatch({
      mode: "bot",
      moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
      players: [
        { kind: "human", name: "Bryan Local", stone: "black" },
        { kind: "bot", name: "Practice Bot", stone: "white" },
      ],
      status: "draw",
      variant: "freestyle",
      winningCells: [],
    });

    store.getState().resetLocalProfile();

    const resetState = store.getState();
    expect(resetState.matchHistory.replayMatches).toEqual([]);
    expect(resetState.profile).toBeNull();
    expect(resetState.settings).toEqual({
      preferredVariant: "freestyle",
    });
  });
});
