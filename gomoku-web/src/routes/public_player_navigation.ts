export const PUBLIC_PLAYER_UID_PATTERN = /^[A-Za-z0-9_-]{1,128}$/;

const PUBLIC_PROFILE_PAGE_COUNT = 4;
const PUBLIC_REPLAY_FINGERPRINT_PATTERN = /^[A-Za-z0-9_-]{43}$/;
const PUBLIC_REPLAY_SCROLL_LIMIT = 1_000_000;

export interface PublicProfilePageTarget {
  uid: string;
  page: number;
}

export interface PublicReplayReturn extends PublicProfilePageTarget {
  scrollTop: number;
  focusFingerprint: string | null;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function requestedPublicPage(search: string): number {
  const params = new URLSearchParams(search);
  const values = params.getAll("page");
  if (values.length !== 1 || !/^[1-9][0-9]{0,3}$/.test(values[0] ?? "")) return 1;
  return Math.min(Number(values[0]), PUBLIC_PROFILE_PAGE_COUNT);
}

export function publicProfilePath(uid: string, pageIndex = 0): string {
  const params = new URLSearchParams({ id: uid });
  if (pageIndex > 0) params.set("page", String(pageIndex + 1));
  return `/player/?${params.toString()}`;
}

export function publicProfileHref(uid: string): string {
  const url = new URL(`${import.meta.env.BASE_URL}player/`, window.location.origin);
  url.searchParams.set("id", uid);
  return url.href;
}

export function publicReplayPath(payload: string, origin: PublicProfilePageTarget): string {
  const url = new URL("/replay/shared/", window.location.origin);
  if (PUBLIC_PLAYER_UID_PATTERN.test(origin.uid)
    && Number.isInteger(origin.page)
    && origin.page >= 1
    && origin.page <= PUBLIC_PROFILE_PAGE_COUNT) {
    url.searchParams.set("fromUid", origin.uid);
    url.searchParams.set("fromPage", String(origin.page));
  }
  url.hash = `g2r1=${payload}`;
  return `${url.pathname}${url.search}${url.hash}`;
}

export function publicReplayReturnTarget(search: string): PublicProfilePageTarget | null {
  const params = new URLSearchParams(search);
  const uidValues = params.getAll("fromUid");
  const pageValues = params.getAll("fromPage");
  if (uidValues.length !== 1 || pageValues.length !== 1) return null;
  const uid = uidValues[0] ?? "";
  const pageText = pageValues[0] ?? "";
  if (!PUBLIC_PLAYER_UID_PATTERN.test(uid) || !/^[1-4]$/.test(pageText)) return null;
  return { uid, page: Number(pageText) };
}

export function publicReplayReturnFromState(state: unknown): PublicReplayReturn | null {
  if (!isRecord(state) || !isRecord(state.publicReplayReturn)) return null;
  const value = state.publicReplayReturn;
  if (typeof value.uid !== "string" || !PUBLIC_PLAYER_UID_PATTERN.test(value.uid)
    || !Number.isInteger(value.page) || (value.page as number) < 1 || (value.page as number) > PUBLIC_PROFILE_PAGE_COUNT) {
    return null;
  }
  const scrollTop = Number.isInteger(value.scrollTop) && (value.scrollTop as number) >= 0
    && (value.scrollTop as number) <= PUBLIC_REPLAY_SCROLL_LIMIT
    ? value.scrollTop as number
    : 0;
  const focusFingerprint = typeof value.focusFingerprint === "string"
    && PUBLIC_REPLAY_FINGERPRINT_PATTERN.test(value.focusFingerprint)
    ? value.focusFingerprint
    : null;
  return { uid: value.uid, page: value.page as number, scrollTop, focusFingerprint };
}

export function makePublicReplayReturn(
  uid: string,
  page: number,
  scrollTop: number,
  focusFingerprint: string,
): PublicReplayReturn | null {
  if (!Number.isFinite(scrollTop)) return null;
  return publicReplayReturnFromState({
    publicReplayReturn: {
      uid,
      page,
      scrollTop: Math.max(0, Math.min(Math.floor(scrollTop), PUBLIC_REPLAY_SCROLL_LIMIT)),
      focusFingerprint,
    },
  });
}

export function publicReplayReturnWithLocationState(
  search: string,
  state: unknown,
): PublicReplayReturn | null {
  const target = publicReplayReturnTarget(search);
  if (!target) return null;
  const remembered = publicReplayReturnFromState(state);
  return remembered?.uid === target.uid && remembered.page === target.page
    ? remembered
    : { ...target, scrollTop: 0, focusFingerprint: null };
}

export function rememberPublicReplayReturnInCurrentHistory(origin: PublicReplayReturn): void {
  if (!PUBLIC_PLAYER_UID_PATTERN.test(origin.uid)
    || !Number.isInteger(origin.page)
    || origin.page < 1
    || origin.page > PUBLIC_PROFILE_PAGE_COUNT) return;

  const currentState = isRecord(window.history.state) ? window.history.state : {};
  const currentUserState = isRecord(currentState.usr) ? currentState.usr : {};
  try {
    window.history.replaceState({
      ...currentState,
      usr: { ...currentUserState, publicReplayReturn: origin },
    }, "", window.location.href);
  } catch {
    // The replay URL still carries the validated public UID and page for explicit return.
  }
}

export function publicReplayControlId(fingerprint: string): string {
  return `public-replay-${fingerprint}`;
}
