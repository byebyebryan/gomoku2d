import { describe, expect, it } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";
import type { LocalProfileSavedMatch } from "../profile/local_profile_store";

import {
  createReplayAnalyzer,
  replayAnalysisOptionsJson,
  savedMatchToReplayJson,
} from "./replay_analysis_core";

const WINNING_MOVES = [
  { col: 5, moveNumber: 1, player: 1 as const, row: 7 },
  { col: 0, moveNumber: 2, player: 2 as const, row: 0 },
  { col: 6, moveNumber: 3, player: 1 as const, row: 7 },
  { col: 1, moveNumber: 4, player: 2 as const, row: 0 },
  { col: 7, moveNumber: 5, player: 1 as const, row: 7 },
  { col: 2, moveNumber: 6, player: 2 as const, row: 0 },
  { col: 8, moveNumber: 7, player: 1 as const, row: 7 },
  { col: 3, moveNumber: 8, player: 2 as const, row: 0 },
  { col: 9, moveNumber: 9, player: 1 as const, row: 7 },
];

function localMatch(patch: Partial<LocalProfileSavedMatch> = {}): LocalProfileSavedMatch {
  return {
    ...createLocalSavedMatch({
      id: "match-1",
      localProfileId: "local-1",
      moves: WINNING_MOVES,
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Normal Bot", stone: "white" },
      ],
      savedAt: "2026-05-16T12:00:00.000Z",
      status: "black_won",
      ruleset: "renju",
    }),
    ...patch,
  };
}

describe("savedMatchToReplayJson", () => {
  it("converts saved match history into core replay JSON with exact hashes", () => {
    const replayJson = savedMatchToReplayJson(localMatch());
    const replay = JSON.parse(replayJson) as {
      black: string;
      hash_algo: { algorithm: string; seed: number };
      moves: Array<{ hash: number; mv: string; time_ms: number }>;
      result: string;
      rules: { board_size: number; variant: string; win_length: number };
      schema_version: number;
      white: string;
    };

    expect(replay.schema_version).toBe(1);
    expect(replay.hash_algo.algorithm).toBe("xorshift64");
    expect(replay.hash_algo.seed).toBeTypeOf("number");
    expect(replayJson).toContain('"seed":16045690984503098046');
    expect(replayJson).not.toContain('"hash":"');
    expect(replay.rules).toEqual({ board_size: 15, variant: "renju", win_length: 5 });
    expect(replay.black).toBe("Guest");
    expect(replay.white).toBe("Normal Bot");
    expect(replay.result).toBe("black_wins");
    expect(replay.moves.map((move) => move.mv)).toEqual(["F8", "A1", "G8", "B1", "H8", "C1", "I8", "D1", "J8"]);
    expect(replay.moves.every((move) => move.time_ms === 0)).toBe(true);
    expect(replay.moves.every((move) => Number.isInteger(move.hash) && move.hash > 0)).toBe(true);
  });

  it("throws when saved match moves cannot be replayed by the core board", () => {
    const match = localMatch({ move_cells: [112, 112], move_count: 2 });

    expect(() => savedMatchToReplayJson(match)).toThrow("Saved match cannot be replayed by core rules");
  });
});

describe("createReplayAnalyzer", () => {
  it("creates a wasm analyzer that can analyze the converted saved match", () => {
    const analyzer = createReplayAnalyzer(localMatch(), { maxDepth: 4, maxScanPlies: 64 });

    try {
      const result = JSON.parse(analyzer.step(128)) as { analysis: unknown; done: boolean; status: string };

      expect(result.done).toBe(true);
      expect(result.status).toBe("resolved");
      expect(result.analysis).not.toBeNull();
    } finally {
      analyzer.dispose();
      analyzer.free();
    }
  });

  it("serializes analyzer options using the wasm bridge schema", () => {
    expect(JSON.parse(replayAnalysisOptionsJson({ maxDepth: 2, maxScanPlies: null }))).toEqual({
      max_depth: 2,
      max_scan_plies: null,
    });
  });
});
