import type { ReplayLibraryOwner } from "../replay/replay_library_store";

export const SAVED_REPLAYS_PATH = "/profile?tab=saved";
export const SAVED_REPLAYS_VISIBLE_BATCH = 8;

export interface SavedReplayReturn {
  owner: ReplayLibraryOwner;
  fingerprint: string;
  visible: number;
  scrollTop: number;
}

export function savedReplayReturnFromState(state: unknown, owner: ReplayLibraryOwner): SavedReplayReturn | undefined {
  if (!state || typeof state !== "object" || !("savedReplayReturn" in state)) return;
  const value = state.savedReplayReturn as Partial<SavedReplayReturn> | null;
  if (!value || value.owner !== owner || typeof value.fingerprint !== "string" || !/^[A-Za-z0-9_-]{43}$/.test(value.fingerprint)) return;
  const { visible, scrollTop } = value;
  if (typeof visible !== "number" || !Number.isInteger(visible) || visible < 8 || visible > 64
    || typeof scrollTop !== "number" || !Number.isFinite(scrollTop) || scrollTop < 0) return;
  return { owner, fingerprint: value.fingerprint, visible, scrollTop };
}

export function savedReplayControlId(fingerprint: string) {
  return `saved-replay-${fingerprint}`;
}

export function rememberSavedReplayReturn(value: SavedReplayReturn) {
  // Keep the origin for browser Back as well as the explicit Saved replays link.
  const current = window.history.state;
  if (!current || typeof current !== "object") return;
  window.history.replaceState({ ...current, usr: { ...current.usr, savedReplayReturn: value } }, "");
}
