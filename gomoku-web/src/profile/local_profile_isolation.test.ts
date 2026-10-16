import { expect, it, vi } from "vitest";

// Guest persistence must not initialize the optional Firebase SDK.
vi.mock("firebase/firestore", () => { throw new Error("Firebase unavailable"); });

import { createLocalProfileStore, type LocalProfileStorage } from "./local_profile_store";

it("records and reloads guest history without Firebase", () => {
  const data = new Map<string, string>();
  const storage: LocalProfileStorage = {
    getItem: (key) => data.get(key) ?? null,
    removeItem: (key) => { data.delete(key); },
    setItem: (key, value) => { data.set(key, value); },
  };
  const store = createLocalProfileStore({ storage });
  store.getState().recordFinishedMatch({
    mode: "bot",
    moves: Array.from({ length: 9 }, (_, i) => ({
      row: i % 2 === 0 ? 7 : 0,
      col: i % 2 === 0 ? 7 + i / 2 : (i - 1) / 2,
      player: i % 2 === 0 ? 1 as const : 2 as const,
      moveNumber: i + 1,
    })),
    players: [
      { kind: "human", name: "Guest", stone: "black" },
      { kind: "bot", name: "Normal Bot", stone: "white" },
    ],
    status: "black_won",
    variant: "freestyle",
  });

  const reloaded = createLocalProfileStore({ storage });
  expect(reloaded.getState().matchHistory.replayMatches).toHaveLength(1);
  expect(reloaded.getState().matchHistory.replayMatches[0]).toMatchObject({
    move_count: 9, source: "local_history", status: "black_won", trust: "local_only",
  });
});
