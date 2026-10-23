import type { PublicCatalogEntry, PublicCatalogPageSnapshot } from "../cloud/sharing_common";
import { decodePortableReplay } from "../replay/portable_replay_codec";

export const PUBLIC_LOGICAL_PAGE_SIZE = 8;

export interface PublicGameDetails {
  canReplay: boolean;
  moves: number | null;
  opponent: string | null;
  result: "Win" | "Loss" | "Draw" | null;
  rule: "Freestyle" | "Renju" | null;
}

export function publicGameDetails(entry: PublicCatalogEntry, publisherUid: string): PublicGameDetails {
  let validated;
  try {
    validated = decodePortableReplay(entry.payload);
  } catch {
    return { canReplay: false, moves: null, opponent: null, result: null, rule: null };
  }

  const { replay, status } = validated;
  const publisherSides = (["black", "white"] as const).filter((side) => (
    replay[side].kind === "human" && replay[side].profileUid === publisherUid
  ));
  if (publisherSides.length !== 1) {
    return {
      canReplay: true,
      moves: null,
      opponent: null,
      result: null,
      rule: null,
    };
  }

  const publisherSide = publisherSides[0]!;
  const opponent = replay[publisherSide === "black" ? "white" : "black"];
  const result = status === "draw"
    ? "Draw"
    : status === `${publisherSide}_won`
      ? "Win"
      : status === `${publisherSide === "black" ? "white" : "black"}_won`
        ? "Loss"
        : null;

  return {
    canReplay: true,
    moves: replay.moves.length,
    opponent: opponent.name,
    result,
    rule: replay.ruleset === "renju" ? "Renju" : "Freestyle",
  };
}

export function publicLogicalPageCount(counts: readonly number[]): number {
  return Math.ceil(counts.reduce((total, count) => total + count, 0) / PUBLIC_LOGICAL_PAGE_SIZE);
}

/** Return only physical pages whose entries overlap one logical eight-row page. */
export function publicLogicalPagePhysicalIds(
  counts: readonly number[],
  logicalPageIndex: number,
): number[] {
  const start = logicalPageIndex * PUBLIC_LOGICAL_PAGE_SIZE;
  const total = counts.reduce((sum, count) => sum + count, 0);
  const end = Math.min(start + PUBLIC_LOGICAL_PAGE_SIZE, total);
  if (start < 0 || start >= end) return [];

  const ids: number[] = [];
  let physicalStart = 0;
  counts.forEach((count, pageId) => {
    const physicalEnd = physicalStart + count;
    if (count > 0 && physicalEnd > start && physicalStart < end) ids.push(pageId);
    physicalStart = physicalEnd;
  });
  return ids;
}

export function publicLogicalPageEntries(
  counts: readonly number[],
  pages: Readonly<Partial<Record<number, PublicCatalogPageSnapshot>>>,
  logicalPageIndex: number,
): PublicCatalogEntry[] {
  const start = logicalPageIndex * PUBLIC_LOGICAL_PAGE_SIZE;
  const end = Math.min(start + PUBLIC_LOGICAL_PAGE_SIZE, counts.reduce((sum, count) => sum + count, 0));
  if (start < 0 || start >= end) return [];
  const entries: PublicCatalogEntry[] = [];
  let physicalStart = 0;
  counts.forEach((count, pageId) => {
    const physicalEnd = physicalStart + count;
    const overlapStart = Math.max(start, physicalStart);
    const overlapEnd = Math.min(end, physicalEnd);
    if (overlapEnd > overlapStart) {
      const pageEntries = pages[pageId]?.entries ?? [];
      const pageStart = overlapStart - physicalStart;
      entries.push(...pageEntries.slice(pageStart, pageStart + (overlapEnd - overlapStart)));
    }
    physicalStart = physicalEnd;
  });
  return entries;
}

export function publicProfileDateLabel(timestamp: string): string {
  const date = new Date(timestamp);
  return Number.isFinite(date.getTime())
    ? date.toLocaleDateString(undefined, { month: "long", day: "numeric", year: "numeric" })
    : "Date unavailable";
}
