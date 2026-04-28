import type { SavedMatchV1 } from "../match/saved_match";

type HistoryPriority = 1 | 2 | 3;

export interface ResolveActiveHistoryInput {
  cloudHistory: SavedMatchV1[];
  localHistory: SavedMatchV1[];
}

function guestImportLocalMatchId(match: SavedMatchV1): string | null {
  const candidate = match as SavedMatchV1 & { local_match_id?: unknown };
  return typeof candidate.local_match_id === "string" && candidate.local_match_id
    ? candidate.local_match_id
    : null;
}

function dedupeKey(match: SavedMatchV1): string {
  if (match.source === "guest_import") {
    return guestImportLocalMatchId(match) ?? match.id;
  }

  return match.id;
}

function priority(match: SavedMatchV1): HistoryPriority {
  if (match.source === "cloud_saved") {
    return 3;
  }

  if (match.source === "guest_import") {
    return 2;
  }

  return 1;
}

function shouldReplaceActiveMatch(existing: SavedMatchV1, candidate: SavedMatchV1): boolean {
  const existingPriority = priority(existing);
  const candidatePriority = priority(candidate);

  if (candidatePriority !== existingPriority) {
    return candidatePriority > existingPriority;
  }

  return candidate.saved_at >= existing.saved_at;
}

export function resolveActiveHistory(input: ResolveActiveHistoryInput): SavedMatchV1[] {
  const byKey = new Map<string, SavedMatchV1>();

  for (const match of [...input.localHistory, ...input.cloudHistory]) {
    const key = dedupeKey(match);
    const existing = byKey.get(key);

    if (!existing || shouldReplaceActiveMatch(existing, match)) {
      byKey.set(key, match);
    }
  }

  return Array.from(byKey.values()).sort((left, right) => right.saved_at.localeCompare(left.saved_at));
}
