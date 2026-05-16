import { savedMatchIsAfterReset, type SavedMatchV2 } from "../match/saved_match";

type HistoryPriority = 1 | 3;

export interface ResolveActiveHistoryInput {
  cloudHistory: SavedMatchV2[];
  historyResetAt?: string | null;
  localHistory: SavedMatchV2[];
}

function dedupeKey(match: SavedMatchV2): string {
  return match.id;
}

function priority(match: SavedMatchV2): HistoryPriority {
  if (match.source === "cloud_saved") {
    return 3;
  }

  return 1;
}

function shouldReplaceActiveMatch(existing: SavedMatchV2, candidate: SavedMatchV2): boolean {
  const existingPriority = priority(existing);
  const candidatePriority = priority(candidate);

  if (candidatePriority !== existingPriority) {
    return candidatePriority > existingPriority;
  }

  return candidate.saved_at >= existing.saved_at;
}

export function resolveActiveHistory(input: ResolveActiveHistoryInput): SavedMatchV2[] {
  const byKey = new Map<string, SavedMatchV2>();

  for (const match of [...input.localHistory, ...input.cloudHistory]) {
    if (!savedMatchIsAfterReset(match, input.historyResetAt)) {
      continue;
    }

    const key = dedupeKey(match);
    const existing = byKey.get(key);

    if (!existing || shouldReplaceActiveMatch(existing, match)) {
      byKey.set(key, match);
    }
  }

  return Array.from(byKey.values()).sort((left, right) => right.saved_at.localeCompare(left.saved_at));
}
