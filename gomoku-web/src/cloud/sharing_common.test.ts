import { describe, expect, it } from "vitest";

import {
  parsePublicProfileSnapshot,
  SharingDataError,
  validateCatalogPageCounts,
  validatePublicProfileStatsCounters,
} from "./sharing_common.js";

function legacyProfile(overrides: Record<string, unknown> = {}) {
  return {
    schema_version: 1,
    uid: "uid-1",
    display_name: "Public Name",
    enabled: true,
    generation: 1,
    revision: 1,
    updated_at: new Date("2026-10-22T12:00:00.000Z"),
    ...overrides,
  };
}

describe("public profile shared fields", () => {
  it("accepts an exact legacy root and known optional extension fields", () => {
    expect(parsePublicProfileSnapshot(legacyProfile(), "uid-1")).toMatchObject({
      sharedStats: null,
      catalogPageCounts: null,
    });
    expect(parsePublicProfileSnapshot(legacyProfile({
      shared_stats: null,
      catalog_page_counts: [2, 0, 1, 0],
    }), "uid-1")).toMatchObject({
      sharedStats: null,
      catalogPageCounts: [2, 0, 1, 0],
    });
    expect(parsePublicProfileSnapshot(legacyProfile({
      shared_stats: {
        matches: 3, wins: 2, losses: 1, draws: 0,
        updated_at: new Date("2026-10-21T12:00:00.000Z"),
      },
      catalog_page_counts: [0, 0, 0, 0],
    }), "uid-1")).toMatchObject({
      sharedStats: { matches: 3, wins: 2, losses: 1, draws: 0 },
      catalogPageCounts: [0, 0, 0, 0],
    });
  });

  it("rejects unknown root fields and malformed or out-of-range shared values", () => {
    expect(() => parsePublicProfileSnapshot(legacyProfile({ future_field: true }), "uid-1"))
      .toThrow(SharingDataError);
    for (const counts of [[], [0, 0, 0], [0, 0, 0, 9], [0, 0, -1, 0], [0, 0, 1.5, 0]]) {
      expect(() => validateCatalogPageCounts(counts)).toThrow(SharingDataError);
    }
    for (const counters of [
      { matches: 1, wins: 1, losses: 1, draws: 0 },
      { matches: -1, wins: 0, losses: 0, draws: 0 },
      { matches: 1.5, wins: 1, losses: 0, draws: 0 },
      { matches: 2_147_483_648, wins: 2_147_483_648, losses: 0, draws: 0 },
      { matches: 0, wins: 0, losses: 0, draws: 0, extra: 0 },
    ]) {
      expect(() => validatePublicProfileStatsCounters(counters)).toThrow(SharingDataError);
    }
    expect(validatePublicProfileStatsCounters({
      matches: 2_147_483_647, wins: 2_147_483_647, losses: 0, draws: 0,
    }).matches).toBe(2_147_483_647);
    expect(() => parsePublicProfileSnapshot(legacyProfile({
      shared_stats: {
        matches: 1, wins: 1, losses: 0, draws: 0,
        updated_at: "2026-10-21T12:00:00.000Z",
      },
    }), "uid-1")).toThrow(SharingDataError);
    expect(() => parsePublicProfileSnapshot(legacyProfile({
      enabled: false,
      display_name: "",
      shared_stats: {
        matches: 0, wins: 0, losses: 0, draws: 0,
        updated_at: new Date("2026-10-21T12:00:00.000Z"),
      },
    }), "uid-1")).toThrow(SharingDataError);
  });
});
