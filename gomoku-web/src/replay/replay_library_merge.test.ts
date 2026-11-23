import { describe, expect, it } from "vitest";

import type { PrivateLibraryEntry } from "../cloud/sharing_private";
import {
  mergeReplayEntries,
  matchingFingerprintHasSamePayloadAndKind,
  replayEntriesHaveSameMetadata,
  validateReplayMergeChoices,
} from "./replay_library_merge";

function entry(overrides: Partial<PrivateLibraryEntry> = {}): PrivateLibraryEntry {
  return {
    fingerprint: "F".repeat(43),
    payload: "portable-payload",
    kind: "saved",
    saved_at: "2026-10-21T00:00:00.000Z",
    metadata: { title: "Title", note: "Note", opening_ply: 0 },
    ...overrides,
  };
}

describe("replay-library merge rules", () => {
  it("matches a replay identity only when its payload and kind both match", () => {
    const original = entry();
    expect(matchingFingerprintHasSamePayloadAndKind(original, entry())).toBe(true);
    expect(matchingFingerprintHasSamePayloadAndKind(original, entry({ payload: "different-payload" }))).toBe(false);
    expect(matchingFingerprintHasSamePayloadAndKind(original, entry({ kind: "practice" }))).toBe(false);
  });

  it("compares verified private details and validates one choice per conflicting row", () => {
    const rows = [
      { fingerprint: "conflict", status: "metadata-conflict" },
      { fingerprint: "same", status: "same" },
    ];
    const nonConflictMessage = "Metadata choices are only valid for replay rows with differing details.";

    expect(replayEntriesHaveSameMetadata(entry(), entry())).toBe(true);
    expect(replayEntriesHaveSameMetadata(entry(), entry({ metadata: { title: "Other", note: "Note", opening_ply: 0 } }))).toBe(false);
    expect(() => validateReplayMergeChoices(rows, { conflict: "cloud" }, ["local", "cloud"], nonConflictMessage)).not.toThrow();
    expect(() => validateReplayMergeChoices(rows, {}, ["local", "cloud"], nonConflictMessage)).toThrow(
      "Choose one complete set of private details for every differing replay.");
    expect(() => validateReplayMergeChoices(rows, { conflict: "guest" }, ["local", "cloud"], nonConflictMessage)).toThrow(
      "Choose one complete set of private details for every differing replay.");
    expect(() => validateReplayMergeChoices(rows, { conflict: "local", same: "local" }, ["local", "cloud"], nonConflictMessage)).toThrow(
      nonConflictMessage);
    expect(() => validateReplayMergeChoices(rows, { conflict: "local", outside: "cloud" }, ["local", "cloud"], nonConflictMessage)).toThrow(
      "A metadata choice refers to a replay outside this review.");
  });

  it("keeps the selected complete entry while retaining the earliest save time", () => {
    const left = entry({
      saved_at: "2026-10-22T00:00:00.000Z",
      metadata: { title: "Left title", note: "Left note", opening_ply: 2 },
    });
    const right = entry({
      saved_at: "2026-10-20T00:00:00.000Z",
      metadata: { title: "Right title", note: "Right note", opening_ply: 4 },
    });

    expect(mergeReplayEntries(left, right, "left")).toEqual({
      ...left,
      saved_at: right.saved_at,
    });
    expect(mergeReplayEntries(left, right, "right")).toEqual(right);
  });
});
