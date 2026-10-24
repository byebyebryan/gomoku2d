import { useEffect, useLayoutEffect, useMemo, useRef, useState, type MouseEvent } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";

import { useDocumentTitle } from "../app/useDocumentTitle";
import {
  loadCatalogPage,
  loadPublicProfile,
  type PublicCatalogPageSnapshot,
  type PublicProfileSnapshot,
} from "../cloud/sharing_public";
import { playerWinRate } from "../profile/player_stats";
import { Icon } from "../ui/Icon";
import {
  publicGameDetails,
  publicLogicalPageCount,
  publicLogicalPageEntries,
  publicLogicalPagePhysicalIds,
  publicProfileDateLabel,
} from "./public_catalog";
import {
  makePublicReplayReturn,
  publicProfileHref,
  publicProfilePath,
  publicReplayControlId,
  publicReplayPath,
  publicReplayReturnFromState,
  requestedPublicPage,
  rememberPublicReplayReturnInCurrentHistory,
  PUBLIC_PLAYER_UID_PATTERN,
  type PublicReplayReturn,
} from "./public_player_navigation";

import styles from "./PublicPlayerRoute.module.css";

const PUBLIC_PAGE_IDS = [0, 1, 2, 3] as const;

interface PublicPlayerView {
  uid: string;
  requestKey: number;
  status: "loading" | "unavailable" | "error" | "ready";
  profile?: PublicProfileSnapshot;
  counts: [number, number, number, number] | null;
  pageCache: Partial<Record<number, PublicCatalogPageSnapshot>>;
  pageIndex: number;
  pageLoading: boolean;
  pageError: string | null;
  pageErrorIndex: number | null;
  message: string | null;
}

function errorText(error: unknown, fallback: string): string {
  return error instanceof Error && error.message ? error.message : fallback;
}

function isUnavailableReadError(error: unknown): boolean {
  if (typeof error !== "object" || error === null || !("code" in error)) return false;
  const code = (error as { code?: unknown }).code;
  if (typeof code !== "string") return false;
  const parts = code.split("/");
  const codeName = parts[parts.length - 1] ?? code;
  return ["permission-denied", "not-found"].includes(codeName);
}

function pageMismatchMessage(): string {
  return "The public catalog changed or no longer matches its page counts. Reload the public profile to try again.";
}

function unavailableView(uid: string, message: string, requestKey: number): PublicPlayerView {
  return {
    uid,
    requestKey,
    status: "unavailable",
    counts: null,
    pageCache: {},
    pageIndex: 0,
    pageLoading: false,
    pageError: null,
    pageErrorIndex: null,
    message,
  };
}

function compactViewport(): boolean {
  return typeof window.matchMedia === "function" && window.matchMedia("(max-width: 720px)").matches;
}

function profileScrollElement(page: HTMLElement | null): HTMLElement | null {
  // The app shell scrolls main on desktop and the document in portrait.
  if (page && page.scrollHeight > page.clientHeight + 1) return page;
  return document.scrollingElement instanceof HTMLElement ? document.scrollingElement : page;
}

function requestedLogicalPage(search: string, counts: readonly number[]): number {
  const pageCount = publicLogicalPageCount(counts);
  return pageCount === 0 ? 0 : Math.min(requestedPublicPage(search) - 1, pageCount - 1);
}

function pageIsConsistent(
  page: PublicCatalogPageSnapshot | null,
  uid: string,
  generation: number,
  pageId: number,
  expectedCount: number,
): boolean {
  return Boolean(page && page.uid === uid && page.generation === generation
    && page.pageId === pageId && page.entries.length === expectedCount);
}

export function PublicPlayerRoute() {
  useDocumentTitle("Public Player");
  const location = useLocation();
  const navigate = useNavigate();
  const uid = new URLSearchParams(location.search).get("id") ?? "";
  const [retryKey, setRetryKey] = useState(0);
  const [view, setView] = useState<PublicPlayerView | null>(null);
  const [linkMessage, setLinkMessage] = useState("");
  const [copyFallbackKey, setCopyFallbackKey] = useState(0);
  const [detailsOpen, setDetailsOpen] = useState(() => !compactViewport());
  const requestRef = useRef(0);
  const pageRequestRef = useRef(0);
  const pageRef = useRef<HTMLElement>(null);
  const linkRef = useRef<HTMLInputElement>(null);
  const restoredLocationKeyRef = useRef<string | null>(null);
  const copyFallbackPendingRef = useRef(false);
  const locationRef = useRef(location);
  locationRef.current = location;

  useEffect(() => {
    setLinkMessage("");
    copyFallbackPendingRef.current = false;
    setDetailsOpen(!compactViewport());
  }, [uid]);

  useEffect(() => {
    if (typeof window.matchMedia !== "function") return undefined;
    const media = window.matchMedia("(max-width: 720px)");
    const updateDetails = () => setDetailsOpen(!media.matches);
    updateDetails();
    if (typeof media.addEventListener === "function") {
      media.addEventListener("change", updateDetails);
      return () => media.removeEventListener("change", updateDetails);
    }
    media.addListener(updateDetails);
    return () => {
      media.removeListener(updateDetails);
    };
  }, []);

  useEffect(() => {
    const request = ++requestRef.current;
    let cancelled = false;
    const current = () => !cancelled && requestRef.current === request;

    if (!PUBLIC_PLAYER_UID_PATTERN.test(uid)) {
      setView(unavailableView(uid, "This public player link is unavailable.", retryKey));
      return () => { cancelled = true; };
    }

    setView({
      uid,
      requestKey: retryKey,
      status: "loading",
      counts: null,
      pageCache: {},
      pageIndex: 0,
      pageLoading: false,
      pageError: null,
      pageErrorIndex: null,
      message: null,
    });

    void (async () => {
      try {
        const profile = await loadPublicProfile(uid);
        if (!current()) return;
        if (!profile?.enabled || profile.uid !== uid) {
          setView(unavailableView(uid, "This public player is unavailable.", retryKey));
          return;
        }

        let counts: [number, number, number, number];
        let pageCache: Partial<Record<number, PublicCatalogPageSnapshot>> = {};
        if (profile.catalogPageCounts === null) {
          const legacyPages = await Promise.all(PUBLIC_PAGE_IDS.map((pageId) => loadCatalogPage(uid, pageId)));
          if (!current()) return;
          if (legacyPages.some((page, pageId) => (
            !pageIsConsistent(page, uid, profile.generation, pageId, page?.entries.length ?? -1)
          ))) {
            throw new Error(pageMismatchMessage());
          }
          counts = legacyPages.map((page) => page!.entries.length) as [number, number, number, number];
          pageCache = Object.fromEntries(legacyPages.map((page) => [page!.pageId, page!])) as PublicPlayerView["pageCache"];
        } else {
          counts = [...profile.catalogPageCounts] as [number, number, number, number];
        }

        const requestedPageIndex = requestedLogicalPage(locationRef.current.search, counts);
        const pageCount = publicLogicalPageCount(counts);
        setView({
          uid,
          requestKey: retryKey,
          status: "ready",
          profile,
          counts,
          pageCache,
          pageIndex: requestedPageIndex,
          pageLoading: pageCount > 0 && Object.keys(pageCache).length === 0,
          pageError: null,
          pageErrorIndex: null,
          message: null,
        });
      } catch (error) {
        if (!current()) return;
        const unavailable = isUnavailableReadError(error);
        setView({
          ...unavailableView(
            uid,
            unavailable
              ? "This public player is unavailable."
              : errorText(error, "This public player could not be loaded."),
            retryKey,
          ),
          status: unavailable ? "unavailable" : "error",
        });
      }
    })();

    return () => { cancelled = true; };
  }, [retryKey, uid]);

  const visibleView = view?.uid === uid && view.requestKey === retryKey ? view : null;
  const pageCount = visibleView?.counts ? publicLogicalPageCount(visibleView.counts) : 0;
  const visibleEntries = useMemo(() => (
    visibleView?.counts
      ? publicLogicalPageEntries(visibleView.counts, visibleView.pageCache, visibleView.pageIndex)
      : []
  ), [visibleView?.counts, visibleView?.pageCache, visibleView?.pageIndex]);
  const visibleRows = useMemo(() => visibleView?.profile
    ? visibleEntries.map((entry) => ({ entry, details: publicGameDetails(entry, visibleView.profile!.uid) }))
    : [], [visibleEntries, visibleView?.profile]);
  const sharedGameCount = visibleView?.counts?.reduce((sum, count) => sum + count, 0) ?? 0;
  const sharedStats = visibleView?.profile?.sharedStats ?? null;
  const winRate = sharedStats ? playerWinRate(sharedStats) : null;

  useEffect(() => {
    if (!visibleView?.profile || !visibleView.counts || visibleView.status !== "ready") return undefined;

    const targetPageIndex = requestedLogicalPage(location.search, visibleView.counts);
    const canonicalPath = publicProfilePath(uid, targetPageIndex);
    if (`${location.pathname}${location.search}` !== canonicalPath) {
      navigate(canonicalPath, { replace: true, state: location.state });
      return undefined;
    }

    const neededPageIds = pageCount > 0
      ? publicLogicalPagePhysicalIds(visibleView.counts, targetPageIndex)
      : [];
    const missingPageIds = neededPageIds.filter((pageId) => !visibleView.pageCache[pageId]);
    const routeKey = location.key;
    const request = ++pageRequestRef.current;
    let cancelled = false;
    const current = () => !cancelled
      && pageRequestRef.current === request
      && locationRef.current.key === routeKey;

    if (missingPageIds.length === 0) {
      setView((currentView) => currentView?.uid === uid && currentView.status === "ready"
        ? {
          ...currentView,
          pageIndex: targetPageIndex,
          pageLoading: false,
          pageError: targetPageIndex === currentView.pageIndex ? currentView.pageError : null,
          pageErrorIndex: targetPageIndex === currentView.pageIndex ? currentView.pageErrorIndex : null,
        }
        : currentView);
      return () => { cancelled = true; };
    }

    setView((currentView) => currentView?.uid === uid && currentView.status === "ready"
      ? { ...currentView, pageLoading: true, pageError: null, pageErrorIndex: null }
      : currentView);

    void (async () => {
      try {
        const loadedPages = await Promise.all(missingPageIds.map((pageId) => loadCatalogPage(uid, pageId)));
        if (!current()) return;
        if (loadedPages.some((page, index) => (
          !pageIsConsistent(
            page,
            uid,
            visibleView.profile!.generation,
            missingPageIds[index]!,
            visibleView.counts![missingPageIds[index]!]!,
          )
        ))) {
          throw new Error(pageMismatchMessage());
        }
        const pageCache = {
          ...visibleView.pageCache,
          ...Object.fromEntries(loadedPages.map((page) => [page!.pageId, page!])),
        };
        setView((currentView) => currentView?.uid === uid && currentView.status === "ready"
          ? {
            ...currentView,
            pageCache,
            pageIndex: targetPageIndex,
            pageLoading: false,
            pageError: null,
            pageErrorIndex: null,
          }
          : currentView);
      } catch (error) {
        if (!current()) return;
        if (isUnavailableReadError(error)) {
          setView(unavailableView(uid, "This public player is unavailable.", retryKey));
          return;
        }
        setView((currentView) => currentView?.uid === uid && currentView.status === "ready"
          ? {
            ...currentView,
            pageLoading: false,
            pageError: errorText(error, "This catalog page could not be loaded."),
            pageErrorIndex: targetPageIndex,
          }
          : currentView);
        if (targetPageIndex !== visibleView.pageIndex) {
          navigate(publicProfilePath(uid, visibleView.pageIndex), { replace: true, state: null });
        }
      }
    })();

    return () => {
      cancelled = true;
      if (pageRequestRef.current === request) pageRequestRef.current += 1;
    };
  }, [
    location.key,
    location.pathname,
    location.search,
    location.state,
    navigate,
    pageCount,
    retryKey,
    uid,
    visibleView?.counts,
    visibleView?.profile,
    visibleView?.status,
  ]);

  const returnOrigin = publicReplayReturnFromState(location.state);
  useLayoutEffect(() => {
    if (restoredLocationKeyRef.current === location.key
      || !returnOrigin || returnOrigin.uid !== uid || !visibleView) return;
    if (visibleView.status === "loading" || (visibleView.status === "ready" && visibleView.pageLoading)) return;

    restoredLocationKeyRef.current = location.key;
    const title = document.getElementById("public-player-title");
    const scroller = profileScrollElement(pageRef.current);
    if (visibleView.status === "ready" && visibleView.pageIndex + 1 === returnOrigin.page) {
      const trigger = returnOrigin.focusFingerprint
        ? document.getElementById(publicReplayControlId(returnOrigin.focusFingerprint))
        : null;
      if (scroller) scroller.scrollTop = trigger ? returnOrigin.scrollTop : 0;
      (trigger instanceof HTMLElement ? trigger : title)?.focus({ preventScroll: true });
    } else {
      if (scroller) scroller.scrollTop = 0;
      title?.focus({ preventScroll: true });
    }
  }, [location.key, location.state, returnOrigin, uid, visibleView]);

  useLayoutEffect(() => {
    if (!copyFallbackPendingRef.current || !detailsOpen || !linkRef.current) return;
    copyFallbackPendingRef.current = false;
    linkRef.current.focus();
    linkRef.current.select();
  }, [copyFallbackKey, detailsOpen, linkMessage]);

  const retryLabel = visibleView?.status === "loading" ? "Loading" : "Try again";
  const canonicalLink = PUBLIC_PLAYER_UID_PATTERN.test(uid) ? publicProfileHref(uid) : "";

  const copyLink = async () => {
    if (!visibleView?.profile || !canonicalLink) return;
    setLinkMessage("");
    try {
      await navigator.clipboard.writeText(canonicalLink);
      setLinkMessage("Public profile link copied.");
    } catch {
      copyFallbackPendingRef.current = true;
      setCopyFallbackKey((key) => key + 1);
      setDetailsOpen(true);
      setLinkMessage("Copy the selected public profile link.");
    }
  };

  const replayOrigin = (fingerprint: string): PublicReplayReturn | null => {
    if (!visibleView?.profile) return null;
    const scrollTop = Math.max(0, Math.floor(profileScrollElement(pageRef.current)?.scrollTop ?? 0));
    return makePublicReplayReturn(uid, visibleView.pageIndex + 1, scrollTop, fingerprint);
  };

  const openPublicReplay = (event: MouseEvent<HTMLAnchorElement>, fingerprint: string, payload: string) => {
    if (event.defaultPrevented || event.button !== 0
      || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey
      || event.currentTarget.target) return;
    // Capture scroll at activation: scrolling alone does not rerender the link.
    const origin = replayOrigin(fingerprint);
    if (!origin) return;
    rememberPublicReplayReturnInCurrentHistory(origin);
    event.preventDefault();
    navigate(publicReplayPath(payload, origin), { state: { publicReplayReturn: origin } });
  };

  return (
    <main className={styles.page} ref={pageRef}>
      <div className={styles.shell}>
        <header className={styles.header}>
          <div className={styles.headerCopy}>
            <p className="uiPageEyebrow">Gomoku2D · Player record</p>
            <h1 className={styles.headerTitle}>Public profile</h1>
          </div>
          <nav className={styles.headerActions} aria-label="Main navigation">
            <Link aria-label="Play" className="uiAction uiActionPrimary" to="/match/local">
              <Icon name="play" />
              <span className="uiActionLabel">Play</span>
            </Link>
            <Link aria-label="My profile" className="uiAction uiActionSecondary" to="/profile">
              <Icon name="profile" />
              <span className="uiActionLabel">My profile</span>
            </Link>
            <Link aria-label="Home" className="uiAction uiActionNeutral" to="/">
              <Icon name="home" />
              <span className="uiActionLabel">Home</span>
            </Link>
          </nav>
        </header>

        {visibleView?.status === "ready" && visibleView.profile ? (
          <div className={styles.content}>
            <aside className={`${styles.playerCard} uiPanel`} aria-label="Public player details">
              <div className={styles.playerSummary}>
                <div className={styles.identityName}>
                  <p className="uiPageEyebrow">Player</p>
                  <h2 className={styles.playerName} id="public-player-title" tabIndex={-1} title={visibleView.profile.displayName}>
                    {visibleView.profile.displayName}
                  </h2>
                </div>
                <dl className={styles.sharedCount}>
                  <div>
                    <dt>{sharedGameCount === 1 ? "Shared game" : "Shared games"}</dt>
                    <dd>{sharedGameCount}</dd>
                  </div>
                </dl>
                <button aria-label="Copy link to public profile" className={`uiAction uiActionNeutral ${styles.copyLink}`}
                  type="button" onClick={() => void copyLink()}>
                  <Icon name="copy" />
                  <span className="uiActionLabel">Copy link</span>
                </button>
                {linkMessage ? <p className={styles.linkMessage} role="status" aria-live="polite">{linkMessage}</p> : null}
              </div>

              <details className={styles.profileDetails} open={detailsOpen}
                onToggle={(event) => setDetailsOpen(event.currentTarget.open)}>
                <summary><Icon name="profile" /><span>Profile details</span></summary>
                <div className={styles.profileDetailsContent}>
                  <section className={styles.profileAddressSection}>
                    <h3 className={styles.detailLabel}>Profile link</h3>
                    <p className={styles.profileAddress}>
                      <a href={canonicalLink}>{canonicalLink}</a>
                    </p>
                    {linkMessage === "Copy the selected public profile link." ? (
                      <input
                        aria-label="Public profile link"
                        className={`uiInput ${styles.copyFallback}`}
                        id="public-profile-link"
                        onFocus={(event) => event.currentTarget.select()}
                        readOnly
                        ref={linkRef}
                        spellCheck={false}
                        value={canonicalLink}
                      />
                    ) : null}
                  </section>
                  <details className={styles.recordNote}>
                    <summary><Icon name="help" /><span>About these records</span></summary>
                    <p>
                      {sharedStats
                        ? "Player names and game details are supplied by the publisher. Statistics are a player-shared snapshot; the games below are selected casual records. Neither is independently verified."
                        : "Player names and game details are supplied by the publisher. The games below are selected casual records and are not independently verified."}
                    </p>
                  </details>
                </div>
              </details>
            </aside>

            <section className={`${styles.catalogPanel} uiPanel`} aria-label="Public games">
              {sharedStats ? (
                <section className={styles.publicStats} aria-label="Shared player statistics">
                  <div className={styles.recordHeader}>
                    <h2 className={styles.sectionTitle}>Shared record</h2>
                    <p className={styles.publicStatsDate}>
                      Updated {publicProfileDateLabel(sharedStats.updatedAt)}
                    </p>
                  </div>
                  <dl className={styles.publicStatsGrid}>
                    {([
                      ["Matches", sharedStats.matches, styles.matches],
                      ["Wins", sharedStats.wins, styles.wins],
                      ["Losses", sharedStats.losses, styles.losses],
                      ["Draws", sharedStats.draws, styles.draws],
                      ["Win rate", winRate === null ? "—" : `${(winRate * 100).toFixed(1)}%`, styles.rate],
                    ] as const).map(([label, value, tone]) => (
                      <div className={`${styles.publicStatsTile} ${tone}`} key={label}>
                        <dt className={styles.publicStatsLabel}>{label}</dt>
                        <dd className={styles.publicStatsValue}>{value}</dd>
                      </div>
                    ))}
                  </dl>
                </section>
              ) : null}

              <section className={styles.gamesSection} aria-labelledby="public-games-title">
                <div className={styles.gamesHeader}>
                  <div className={styles.gamesHeading}>
                    <h2 className={styles.sectionTitle} id="public-games-title">Published games</h2>
                    <p className={styles.recordSource}>Player-shared · Unverified</p>
                  </div>
                  <span className={styles.gameCount}>{sharedGameCount} {sharedGameCount === 1 ? "game" : "games"}</span>
                </div>

                {sharedGameCount === 0 ? (
                  <p className={styles.emptyState}>No games have been published.</p>
                ) : visibleView.pageLoading ? (
                  <p className={styles.emptyState} role="status">Loading published games.</p>
                ) : visibleRows.length ? (
                  <ol className={styles.publicGameList} aria-label={`Public games, page ${visibleView.pageIndex + 1}`}>
                    {visibleRows.map(({ entry, details }) => {
                      const origin = { uid, page: visibleView.pageIndex + 1 };
                      return (
                        <li className={styles.publicGame} key={entry.fingerprint}>
                          <span className={`${styles.publicGameResult} ${
                            details.result === "Win" ? styles.publicGameResultWin
                              : details.result === "Loss" ? styles.publicGameResultLoss
                                : details.result === "Draw" ? styles.publicGameResultDraw : ""
                          }`}>
                            <span className="uiSrOnly">Result: </span>
                            {details.result ?? "Unavailable"}
                          </span>
                          <div className={styles.gameCopy}>
                            <span className={styles.publicGameTitle} title={entry.title}>{entry.title}</span>
                            <span className={styles.publicGameOpponent} title={details.opponent ?? undefined}>
                              <span className="uiSrOnly">Opponent: </span>
                              {details.opponent ? `vs ${details.opponent}` : "Unavailable"}
                            </span>
                          </div>
                          <span className={`${styles.publicGameMeta} ${styles.gameRule}`}>
                            <span className="uiSrOnly">Rule: </span>
                            {details.rule ?? "Unavailable"}
                          </span>
                          <span className={`${styles.publicGameMeta} ${styles.gameMoves}`}>
                            <span className="uiSrOnly">Move count: </span>
                            {details.moves === null ? "Unavailable" : `${details.moves} moves`}
                          </span>
                          {details.canReplay ? (
                            <Link
                              className="uiAction uiActionSecondary"
                              id={publicReplayControlId(entry.fingerprint)}
                              aria-label={`Replay ${entry.title}`}
                              onClick={(event) => openPublicReplay(event, entry.fingerprint, entry.payload)}
                              to={publicReplayPath(entry.payload, origin)}
                            >
                              <Icon name="replay" />
                              <span className="uiActionLabel">Replay</span>
                            </Link>
                          ) : (
                            <span className={styles.replayUnavailable}>Replay unavailable</span>
                          )}
                        </li>
                      );
                    })}
                  </ol>
                ) : visibleView.pageError ? null : (
                  <p className={styles.emptyState} role="alert">This catalog page is unavailable.</p>
                )}

                {pageCount > 1 ? (
                  <div className={styles.pagingControls}>
                    <button
                      className={`uiAction uiActionNeutral ${styles.previousPage}`}
                      type="button"
                      disabled={visibleView.pageIndex === 0 || visibleView.pageLoading}
                      onClick={() => navigate(publicProfilePath(uid, visibleView.pageIndex - 1))}
                    >
                      Previous page
                    </button>
                    <span className={styles.pageIndicator} aria-live="polite">
                      {visibleView.pageLoading ? "Loading page" : `Page ${visibleView.pageIndex + 1} of ${pageCount}`}
                    </span>
                    <button
                      className={`uiAction uiActionNeutral ${styles.nextPage}`}
                      type="button"
                      disabled={visibleView.pageIndex >= pageCount - 1 || visibleView.pageLoading}
                      onClick={() => navigate(publicProfilePath(uid, visibleView.pageIndex + 1))}
                    >
                      Next page
                    </button>
                  </div>
                ) : null}

                {visibleView.pageError ? (
                  <div className={styles.pageError}>
                    <p role="alert" className={styles.errorText}>{visibleView.pageError}</p>
                    <button className="uiAction uiActionNeutral" type="button"
                      onClick={() => setRetryKey((key) => key + 1)}>
                      Reload public profile
                    </button>
                    {visibleView.pageErrorIndex !== null ? (
                      <button className="uiAction uiActionNeutral" type="button"
                        onClick={() => navigate(publicProfilePath(uid, visibleView.pageErrorIndex!))}>
                        Retry page
                      </button>
                    ) : null}
                  </div>
                ) : null}
              </section>
            </section>
          </div>
        ) : (
          <div className={styles.content}>
            <section className={`${styles.playerCard} uiPanel`} aria-label="Public player details">
              <div className={styles.playerSummary}>
                <h2 className={styles.playerName} id="public-player-title" tabIndex={-1}>
                  {visibleView?.status === "loading" || !visibleView
                    ? "Loading public profile"
                    : "Public profile unavailable"}
                </h2>
                <p className={styles.sharedCount}>
                  {visibleView?.status === "loading" || !visibleView
                    ? "Loading shared games"
                    : "Shared games unavailable"}
                </p>
              </div>
            </section>
            <section className={`${styles.catalogPanel} uiPanel`} aria-label="Public games">
              <h2 className={styles.sectionTitle}>Published games</h2>
              <p className={styles.mutedText} role={visibleView?.status === "error" ? "alert" : "status"}>
                {visibleView?.message ?? "Loading the public player details."}
              </p>
              <button
                className="uiAction uiActionSecondary"
                type="button"
                disabled={!visibleView || visibleView.status === "loading"}
                onClick={() => setRetryKey((key) => key + 1)}
              >
                {retryLabel}
              </button>
            </section>
          </div>
        )}
      </div>
    </main>
  );
}
