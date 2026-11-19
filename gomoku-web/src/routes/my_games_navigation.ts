export interface MyGamesReplayReturn {
  uid: string | null;
  matchId: string;
  action: "inspect" | "publish";
  visible: number;
  scrollTop: number;
}

export interface MyGamesPublishIntent {
  uid: string;
  matchId: string;
  inspectMove?: number;
}

export interface PublicationTarget {
  uid: string;
  fingerprint: string;
  pageId: number;
}

function object(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : {};
}

function isMatchId(value: unknown): value is string {
  return typeof value === "string" && /^[A-Za-z0-9_-]{1,128}$/.test(value);
}

function boundedNumber(value: unknown, min: number, max: number): value is number {
  return typeof value === "number" && Number.isFinite(value) && value >= min && value <= max;
}

export function myGamesReturnFromState(state: unknown, uid: string | null): MyGamesReplayReturn | undefined {
  const value = object(object(state).myGamesReplayReturn);
  if (value.uid !== uid || !isMatchId(value.matchId)
    || (value.action !== "inspect" && value.action !== "publish")
    || !boundedNumber(value.visible, 1, 10_000) || !Number.isInteger(value.visible)
    || !boundedNumber(value.scrollTop, 0, 1_000_000)) return undefined;
  return { uid, matchId: value.matchId, action: value.action, visible: value.visible, scrollTop: value.scrollTop };
}

export function myGamesPublishIntentFromState(
  state: unknown,
  key: "myGamesPublishIntent" | "myGamesInspectTarget" = "myGamesPublishIntent",
): MyGamesPublishIntent | undefined {
  const value = object(object(state)[key]);
  if (typeof value.uid !== "string" || !value.uid || !isMatchId(value.matchId)) return undefined;
  const inspectMove = boundedNumber(value.inspectMove, 0, 225) && Number.isInteger(value.inspectMove)
    ? value.inspectMove : undefined;
  return { uid: value.uid, matchId: value.matchId, inspectMove };
}

type MyGamesFocusRequest =
  | { kind: "tab" }
  | { kind: "return"; origin: MyGamesReplayReturn }
  | { kind: "choose-publish" };

export function myGamesFocusFromState(state: unknown, uid: string | null): MyGamesFocusRequest | undefined {
  const routeState = object(state);
  if (routeState.myGamesFocusTab === true) return { kind: "tab" };

  // Browser Back can restore a chooser entry that now also remembers the chosen row.
  const origin = myGamesReturnFromState(routeState, uid);
  if (origin) return { kind: "return", origin };

  if (uid !== null && routeState.myGamesChooserUid === uid) return { kind: "choose-publish" };
}

export function publicationTargetFromState(state: unknown, uid: string | null): PublicationTarget | undefined {
  const value = object(object(state).publicationTarget);
  if (!uid || value.uid !== uid || typeof value.fingerprint !== "string"
    || !/^[A-Za-z0-9_-]{43}$/.test(value.fingerprint)
    || !boundedNumber(value.pageId, 0, 3) || !Number.isInteger(value.pageId)) return undefined;
  return { uid, fingerprint: value.fingerprint, pageId: value.pageId };
}

export function myGamesControlId(matchId: string, action: MyGamesReplayReturn["action"]): string {
  return `my-games-${action}-${matchId}`;
}

export function rememberMyGamesReplayReturn(value: MyGamesReplayReturn): void {
  const current = object(window.history.state);
  // Preserve the router's key and index so browser Back returns to this list.
  if (typeof current.idx !== "number") return;
  window.history.replaceState({ ...current, usr: { ...object(current.usr), myGamesReplayReturn: value } }, "");
}
