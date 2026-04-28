import { describe, expect, it } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";

import { resolveActiveHistory } from "./active_history";

function localMatch(id: string, savedAt: string) {
  return createLocalSavedMatch({
    id,
    localProfileId: "guest-1",
    moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
    players: [
      { kind: "human", name: "Bryan", stone: "black" },
      { kind: "bot", name: "Practice Bot", stone: "white" },
    ],
    savedAt,
    status: "draw",
    variant: "freestyle",
  });
}

describe("resolveActiveHistory", () => {
  it("sorts local and cloud history together", () => {
    const older = localMatch("older", "2026-04-28T01:00:00.000Z");
    const newer = {
      ...localMatch("newer", "2026-04-28T02:00:00.000Z"),
      source: "cloud_saved" as const,
      trust: "client_uploaded" as const,
    };

    expect(resolveActiveHistory({ cloudHistory: [newer], localHistory: [older] }).map((match) => match.id)).toEqual([
      "newer",
      "older",
    ]);
  });

  it("prefers direct cloud saves over local duplicates", () => {
    const local = localMatch("match-1", "2026-04-28T01:00:00.000Z");
    const cloud = {
      ...local,
      player_black: {
        ...local.player_black,
        local_profile_id: null,
        profile_uid: "uid-1",
      },
      source: "cloud_saved" as const,
      trust: "client_uploaded" as const,
    };

    const history = resolveActiveHistory({ cloudHistory: [cloud], localHistory: [local] });

    expect(history).toHaveLength(1);
    expect(history[0]).toMatchObject({
      id: "match-1",
      source: "cloud_saved",
    });
  });

  it("dedupes guest imports by local_match_id", () => {
    const local = localMatch("match-1", "2026-04-28T01:00:00.000Z");
    const imported = {
      ...local,
      id: "local-match-1",
      local_match_id: "match-1",
      source: "guest_import" as const,
      trust: "client_uploaded" as const,
    };

    const history = resolveActiveHistory({ cloudHistory: [imported], localHistory: [local] });

    expect(history).toHaveLength(1);
    expect(history[0]).toMatchObject({
      id: "local-match-1",
      source: "guest_import",
    });
  });

  it("keeps the newest record when duplicate entries have equal priority", () => {
    const older = {
      ...localMatch("match-1", "2026-04-28T01:00:00.000Z"),
      source: "cloud_saved" as const,
      trust: "client_uploaded" as const,
    };
    const newer = {
      ...older,
      saved_at: "2026-04-28T02:00:00.000Z",
    };

    const history = resolveActiveHistory({ cloudHistory: [newer, older], localHistory: [] });

    expect(history).toHaveLength(1);
    expect(history[0]?.saved_at).toBe("2026-04-28T02:00:00.000Z");
  });
});
