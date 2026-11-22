import { localMatchSessionStore } from "../game/local_match_session";

export function confirmLocalMatchReplacement(
  message = "End your active game and start a new one?",
): boolean {
  const active = localMatchSessionStore.getState().matchStore?.getState();
  return active?.status !== "playing" || active.moves.length === 0 || window.confirm(message);
}
