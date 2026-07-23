import { describe, expect, it, vi } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";
import type { LocalProfileSavedMatch } from "../profile/local_profile_store";

import {
  clearReplayAnalysisCache,
  readReplayAnalysisCache,
  replayAnalysisCacheKey,
  writeReplayAnalysisCache,
  type ReplayAnalysisCachedResult,
} from "./replay_analysis_cache";
import type { ReplayAnalysisStepResult } from "./replay_analysis_protocol";

const MOVES = [
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

class MemoryStorage implements Pick<Storage, "getItem" | "removeItem" | "setItem"> {
  private readonly values = new Map<string, string>();

  getItem(key: string): string | null {
    return this.values.get(key) ?? null;
  }

  removeItem(key: string): void {
    this.values.delete(key);
  }

  setItem(key: string, value: string): void {
    this.values.set(key, value);
  }
}

function localMatch(patch: Partial<LocalProfileSavedMatch> = {}): LocalProfileSavedMatch {
  return {
    ...createLocalSavedMatch({
      id: "match-1",
      localProfileId: "local-1",
      moves: MOVES,
      players: [
        { kind: "human", name: "Black", stone: "black" },
        { kind: "bot", name: "White", stone: "white" },
      ],
      ruleset: "renju",
      savedAt: "2026-05-16T12:00:00.000Z",
      status: "black_won",
    }),
    ...patch,
  };
}

function step(status: ReplayAnalysisStepResult["status"], done = true): ReplayAnalysisStepResult {
  return {
    analysis: done ? { schema_version: 1 } : null,
    annotations: [],
    counters: { branch_roots: 2, prefixes_analyzed: 3, proof_nodes: 321 },
    current_ply: done ? null : 7,
    done,
    error: null,
    schema_version: 1,
    status,
  };
}

function cachedResult(status: ReplayAnalysisStepResult["status"] = "resolved"): ReplayAnalysisCachedResult {
  return {
    annotationsByPly: {
      9: {
        highlights: [
          {
            mv: { col: 8, row: 7 },
            notation: "I8",
            role: "immediate_threat",
            side: "Black",
          },
        ],
        markers: [],
        ply: 9,
        side_to_move: "White",
      },
    },
    step: step(status),
  };
}

describe("replay analysis cache", () => {
  it("clears every cached analysis result", () => {
    const storage = new MemoryStorage();
    const options = { maxDepth: 4, maxScanPlies: 64 };

    writeReplayAnalysisCache(localMatch(), options, cachedResult(), storage);
    clearReplayAnalysisCache(storage);

    expect(readReplayAnalysisCache(localMatch(), options, storage)).toBeNull();
  });

  it("round-trips completed analysis with accumulated annotations", () => {
    const storage = new MemoryStorage();
    const match = localMatch();
    const options = { maxDepth: 4, maxScanPlies: 64 };
    const result = cachedResult();

    writeReplayAnalysisCache(match, options, result, storage);

    expect(readReplayAnalysisCache(match, options, storage)).toEqual(result);
  });

  it("keys by moves, rules, status, schema, and analyzer options", () => {
    const match = localMatch();
    const base = replayAnalysisCacheKey(match, { maxDepth: 4, maxScanPlies: 64 });

    expect(replayAnalysisCacheKey(localMatch({ status: "white_won" }), { maxDepth: 4, maxScanPlies: 64 })).not.toBe(base);
    expect(replayAnalysisCacheKey(localMatch({ move_cells: [...match.move_cells, 10], move_count: 10 }), { maxDepth: 4, maxScanPlies: 64 })).not.toBe(base);
    expect(replayAnalysisCacheKey(match, { maxDepth: 8, maxScanPlies: 64 })).not.toBe(base);
  });

  it("does not cache running or error results", () => {
    const storage = new MemoryStorage();
    const match = localMatch();
    const options = { maxDepth: 4, maxScanPlies: 64 };

    writeReplayAnalysisCache(match, options, { ...cachedResult(), step: step("running", false) }, storage);
    writeReplayAnalysisCache(match, options, { ...cachedResult(), step: step("error") }, storage);

    expect(readReplayAnalysisCache(match, options, storage)).toBeNull();
  });

  it("drops corrupt storage and falls back to a miss", () => {
    const storage = new MemoryStorage();
    storage.setItem("gomoku2d:replay-analysis-cache:v1", "{not-json");

    expect(readReplayAnalysisCache(localMatch(), { maxDepth: 4, maxScanPlies: 64 }, storage)).toBeNull();
    expect(storage.getItem("gomoku2d:replay-analysis-cache:v1")).toBeNull();
  });

  it("keeps only the latest twenty entries", () => {
    const storage = new MemoryStorage();
    const options = { maxDepth: 4, maxScanPlies: 64 };
    vi.spyOn(Date, "now").mockReturnValue(1);

    for (let index = 0; index < 21; index += 1) {
      vi.mocked(Date.now).mockReturnValue(index + 1);
      writeReplayAnalysisCache(
        localMatch({ move_cells: [...MOVES.map((move) => move.row * 15 + move.col), index], move_count: 10 }),
        options,
        cachedResult("unclear"),
        storage,
      );
    }

    expect(readReplayAnalysisCache(localMatch({
      move_cells: [...MOVES.map((move) => move.row * 15 + move.col), 0],
      move_count: 10,
    }), options, storage)).toBeNull();
    expect(readReplayAnalysisCache(localMatch({
      move_cells: [...MOVES.map((move) => move.row * 15 + move.col), 20],
      move_count: 10,
    }), options, storage)).not.toBeNull();

    vi.restoreAllMocks();
  });
});
