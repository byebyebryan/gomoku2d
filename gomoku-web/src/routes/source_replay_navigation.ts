import type { SavedMatchV2 } from "../match/saved_match";

export function sourceReplayMove(state: unknown, match: SavedMatchV2): number | undefined {
  if (!state || typeof state !== "object" || !("sourceReplayFrame" in state)) return;
  const frame = state.sourceReplayFrame as { matchId?: unknown; move?: unknown } | undefined;
  if (frame?.matchId === match.id && typeof frame.move === "number" && Number.isInteger(frame.move)
    && frame.move >= 0 && frame.move <= match.move_count) {
    return frame.move;
  }
}

export function sourceReplayToken(state: unknown): string | undefined {
  if (state && typeof state === "object" && "sourceReplayToken" in state
    && typeof state.sourceReplayToken === "string") {
    return state.sourceReplayToken;
  }
}
