import { describe, expect, it } from "vitest";

import {
  makePublicReplayReturn,
  publicProfileHref,
  publicProfilePath,
  publicReplayPath,
  publicReplayReturnFromState,
  publicReplayReturnWithLocationState,
  requestedPublicPage,
  rememberPublicReplayReturnInCurrentHistory,
} from "./public_player_navigation";

describe("public profile replay navigation", () => {
  it("bounds the 1-based page parameter and builds only the public profile route", () => {
    expect(requestedPublicPage("")).toBe(1);
    expect(requestedPublicPage("?page=4")).toBe(4);
    expect(requestedPublicPage("?page=9999")).toBe(4);
    expect(requestedPublicPage("?page=0")).toBe(1);
    expect(requestedPublicPage("?page=2&page=3")).toBe(1);

    expect(publicProfilePath("uid-1", 2)).toBe("/player/?id=uid-1&page=3");
    expect(publicProfilePath("uid-1", 0)).toBe("/player/?id=uid-1");
  });

  it("keeps profile copies canonical and omits page and return state", () => {
    const url = new URL(publicProfileHref("uid-1"));
    expect(url.pathname).toBe(`${import.meta.env.BASE_URL}player/`);
    expect(Array.from(url.searchParams.entries())).toEqual([["id", "uid-1"]]);
  });

  it("derives a local return path from validated public UID and page values only", () => {
    const origin = publicReplayReturnWithLocationState(
      "?fromUid=uid-1&fromPage=3&return=https%3A%2F%2Fevil.example",
      null,
    );
    expect(origin).toEqual({ uid: "uid-1", page: 3, scrollTop: 0, focusFingerprint: null });
    expect(publicProfilePath(origin!.uid, origin!.page - 1)).toBe("/player/?id=uid-1&page=3");
    expect(publicReplayReturnWithLocationState("?fromUid=https%3A%2F%2Fevil.example&fromPage=1", null)).toBeNull();
    expect(publicReplayReturnWithLocationState("?fromUid=uid-1&fromUid=uid-2&fromPage=1", null)).toBeNull();
    expect(publicReplayReturnWithLocationState("?fromUid=uid-1&fromPage=5", null)).toBeNull();
  });

  it("keeps scroll and focus in bounded router state rather than the replay URL", () => {
    const fingerprint = "A".repeat(43);
    const origin = makePublicReplayReturn("uid-1", 2, 1_000_500, fingerprint);
    expect(origin).toEqual({
      uid: "uid-1",
      page: 2,
      scrollTop: 1_000_000,
      focusFingerprint: fingerprint,
    });

    const path = publicReplayPath("YWJj", origin!);
    expect(path).toBe("/replay/shared/?fromUid=uid-1&fromPage=2#g2r1=YWJj");
    expect(path).not.toMatch(/scroll|focus|return=/i);
    expect(publicReplayReturnFromState({
      publicReplayReturn: { ...origin, scrollTop: -1, focusFingerprint: "unbounded" },
    })).toEqual({ uid: "uid-1", page: 2, scrollTop: 0, focusFingerprint: null });
    expect(publicReplayReturnWithLocationState(
      "?fromUid=uid-1&fromPage=2",
      { publicReplayReturn: { ...origin!, uid: "uid-2", page: 4 } },
    )).toEqual({ uid: "uid-1", page: 2, scrollTop: 0, focusFingerprint: null });
  });

  it("adds the return origin to router user state without replacing its history key or index", () => {
    const originalState = window.history.state;
    const originUrl = window.location.href;
    const origin = makePublicReplayReturn("uid-1", 1, 123, "B".repeat(43))!;
    try {
      window.history.replaceState({ usr: { kept: true }, key: "existing-key", idx: 7 }, "", originUrl);
      rememberPublicReplayReturnInCurrentHistory(origin);
      expect(window.history.state).toEqual({
        usr: { kept: true, publicReplayReturn: origin },
        key: "existing-key",
        idx: 7,
      });
    } finally {
      window.history.replaceState(originalState, "", originUrl);
    }
  });
});
