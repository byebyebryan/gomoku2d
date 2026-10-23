import { describe, expect, it } from "vitest";

import type { PublicCatalogEntry, PublicCatalogPageSnapshot } from "../cloud/sharing_common";
import { createLocalSavedMatch, movesFromMoveCells } from "../match/saved_match";
import { portableReplayFromSavedMatch } from "../replay/portable_replay";
import { encodePortableReplay } from "../replay/portable_replay_codec";
import { publicGameDetails, publicLogicalPageEntries, publicLogicalPagePhysicalIds, publicLogicalPageCount } from "./public_catalog";

function entry(index: number): PublicCatalogEntry {
  return {
    fingerprint: String(index).padStart(43, "x"),
    payload: "YWJj",
    title: `game-${index}`,
    view_ply: 0,
  };
}

function page(pageId: 1 | 3, entries: PublicCatalogEntry[]): PublicCatalogPageSnapshot {
  return {
    uid: "owner-1",
    pageId,
    generation: 2,
    revision: 1,
    entries,
    updatedAt: "2026-10-22T00:00:00.000Z",
  };
}

function attributedGame(publisherSide: "black" | "white", result: "Win" | "Loss", publisherUid = "publisher-1"): PublicCatalogEntry {
  const winnerSide = result === "Win" ? publisherSide : publisherSide === "black" ? "white" : "black";
  const moves = winnerSide === "black"
    ? [110, 0, 111, 1, 112, 2, 113, 3, 114]
    : [0, 110, 2, 111, 4, 112, 6, 113, 8, 114];
  const match = createLocalSavedMatch({
    id: "public-game-test",
    localProfileId: "local-owner",
    moves: movesFromMoveCells(moves),
    players: [
      { kind: publisherSide === "black" ? "human" : "bot", name: publisherSide === "black" ? "Publisher" : "Normal Bot", stone: "black" },
      { kind: publisherSide === "white" ? "human" : "bot", name: publisherSide === "white" ? "Publisher" : "Normal Bot", stone: "white" },
    ],
    ruleset: "freestyle",
    savedAt: "2026-10-21T00:00:00.000Z",
    status: winnerSide === "black" ? "black_won" : "white_won",
  });
  const replay = portableReplayFromSavedMatch(match).replay;
  if (!replay) throw new Error("Fixture replay must be portable.");
  const publisher = replay[publisherSide];
  if (publisher.kind !== "human") throw new Error("Fixture publisher must be human.");
  return {
    ...entry(1),
    payload: encodePortableReplay({
      ...replay,
      [publisherSide]: { ...publisher, ...(publisherUid ? { profileUid: publisherUid } : {}) },
    }),
  };
}

describe("public logical catalog paging", () => {
  it("uses only physical page IDs needed to fill a logical page across sparse storage", () => {
    const counts = [0, 3, 0, 8] as const;
    expect(publicLogicalPageCount(counts)).toBe(2);
    expect(publicLogicalPagePhysicalIds(counts, 0)).toEqual([1, 3]);
    expect(publicLogicalPagePhysicalIds(counts, 1)).toEqual([3]);
  });

  it("slices the logical second page from its physical offset without prefix documents cached", () => {
    const counts = [0, 3, 0, 8] as const;
    const secondStoragePage = page(3, Array.from({ length: 8 }, (_, index) => entry(index + 31)));
    const rows = publicLogicalPageEntries(counts, { 3: secondStoragePage }, 1);
    expect(rows.map(({ title }) => title)).toEqual(["game-36", "game-37", "game-38"]);
  });

  it.each([
    ["black", "Win"],
    ["black", "Loss"],
    ["white", "Win"],
    ["white", "Loss"],
  ] as const)("attributes a %s-side publisher's %s from the portable replay", (side, result) => {
    const details = publicGameDetails(attributedGame(side, result), "publisher-1");
    expect(details).toMatchObject({ canReplay: true, opponent: "Normal Bot", result, rule: "Freestyle" });
    expect(details.moves).toBe(result === "Win" && side === "black" || result === "Loss" && side === "white" ? 9 : 10);
  });

  it("shows neutral metadata for malformed or unattributed portable games", () => {
    const malformed = publicGameDetails({ ...entry(5), payload: "YWJj" }, "publisher-1");
    expect(malformed).toMatchObject({ canReplay: false, opponent: null, result: null, rule: null, moves: null });

    const unattributed = publicGameDetails(attributedGame("white", "Win", ""), "publisher-1");
    expect(unattributed).toMatchObject({ canReplay: true, opponent: null, result: null, rule: null, moves: null });
  });
});
