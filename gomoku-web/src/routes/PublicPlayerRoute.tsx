import { useEffect, useMemo, useRef, useState } from "react";
import { Link, useLocation } from "react-router-dom";

import { useDocumentTitle } from "../app/useDocumentTitle";
import {
  loadCatalogPage,
  loadPublicProfile,
  type PublicCatalogPageSnapshot,
  type PublicProfileSnapshot,
} from "../cloud/sharing_public";
import { playerWinRate } from "../profile/player_stats";
import {
  publicGameDetails,
  publicLogicalPageCount,
  publicLogicalPageEntries,
  publicLogicalPagePhysicalIds,
  publicProfileDateLabel,
} from "./public_catalog";

import styles from "./SharingPanels.module.css";

const PUBLIC_UID_PATTERN = /^[A-Za-z0-9_-]{1,128}$/;
const PUBLIC_PAGE_IDS = [0, 1, 2, 3] as const;

interface PublicPlayerView {
  uid: string;
  status: "loading" | "unavailable" | "error" | "ready";
  profile?: PublicProfileSnapshot;
  counts: [number, number, number, number] | null;
  pageCache: Partial<Record<number, PublicCatalogPageSnapshot>>;
  pageIndex: number;
  pageLoading: boolean;
  pageError: string | null;
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

function unavailableView(uid: string, message: string): PublicPlayerView {
  return {
    uid,
    status: "unavailable",
    counts: null,
    pageCache: {},
    pageIndex: 0,
    pageLoading: false,
    pageError: null,
    message,
  };
}

export function PublicPlayerRoute() {
  useDocumentTitle("Public Player");
  const { search } = useLocation();
  const uid = new URLSearchParams(search).get("id") ?? "";
  const [retryKey, setRetryKey] = useState(0);
  const [view, setView] = useState<PublicPlayerView | null>(null);
  const requestRef = useRef(0);

  useEffect(() => {
    const request = ++requestRef.current;
    let cancelled = false;
    const current = () => !cancelled && requestRef.current === request;

    if (!PUBLIC_UID_PATTERN.test(uid)) {
      setView(unavailableView(uid, "This public player link is unavailable."));
      return () => { cancelled = true; };
    }

    setView({
      uid,
      status: "loading",
      counts: null,
      pageCache: {},
      pageIndex: 0,
      pageLoading: false,
      pageError: null,
      message: null,
    });

    void (async () => {
      try {
        const profile = await loadPublicProfile(uid);
        if (!current()) return;
        if (!profile?.enabled || profile.uid !== uid) {
          setView(unavailableView(uid, "This public player is unavailable."));
          return;
        }

        let counts: [number, number, number, number];
        let pageCache: Partial<Record<number, PublicCatalogPageSnapshot>> = {};
        if (profile.catalogPageCounts === null) {
          const legacyPages = await Promise.all(PUBLIC_PAGE_IDS.map((pageId) => loadCatalogPage(uid, pageId)));
          if (!current()) return;
          if (legacyPages.some((page, pageId) => (
            !page || page.generation !== profile.generation || page.pageId !== pageId
          ))) {
            throw new Error(pageMismatchMessage());
          }
          counts = legacyPages.map((page) => page!.entries.length) as [number, number, number, number];
          pageCache = Object.fromEntries(legacyPages.map((page) => [page!.pageId, page!])) as PublicPlayerView["pageCache"];
        } else {
          counts = [...profile.catalogPageCounts] as [number, number, number, number];
          const firstPageIds = publicLogicalPagePhysicalIds(counts, 0);
          const firstPages = await Promise.all(firstPageIds.map((pageId) => loadCatalogPage(uid, pageId)));
          if (!current()) return;
          if (firstPages.some((page, index) => (
            !page || page.generation !== profile.generation || page.pageId !== firstPageIds[index]
              || page.entries.length !== counts[firstPageIds[index]!]
          ))) {
            throw new Error(pageMismatchMessage());
          }
          pageCache = Object.fromEntries(firstPages.map((page) => [page!.pageId, page!])) as PublicPlayerView["pageCache"];
        }

        setView({
          uid,
          status: "ready",
          profile,
          counts,
          pageCache,
          pageIndex: 0,
          pageLoading: false,
          pageError: null,
          message: null,
        });
      } catch (error) {
        if (!current()) return;
        setView({
          ...unavailableView(
            uid,
            isUnavailableReadError(error)
              ? "This public player is unavailable."
              : errorText(error, "This public player could not be loaded."),
          ),
          status: isUnavailableReadError(error) ? "unavailable" : "error",
        });
      }
    })();

    return () => { cancelled = true; };
  }, [retryKey, uid]);

  const visibleView = view?.uid === uid ? view : null;
  const pageCount = visibleView?.counts ? publicLogicalPageCount(visibleView.counts) : 0;
  const visibleEntries = useMemo(() => (
    visibleView?.counts
      ? publicLogicalPageEntries(visibleView.counts, visibleView.pageCache, visibleView.pageIndex)
      : []
  ), [visibleView?.counts, visibleView?.pageCache, visibleView?.pageIndex]);
  const visibleRows = useMemo(() => visibleView?.profile
    ? visibleEntries.map((entry) => ({ entry, details: publicGameDetails(entry, visibleView.profile!.uid) }))
    : [], [visibleEntries, visibleView?.profile]);

  const loadLogicalPage = async (nextPageIndex: number) => {
    if (!visibleView?.profile || !visibleView.counts || visibleView.pageLoading
      || nextPageIndex < 0 || nextPageIndex >= pageCount) return;
    const physicalIds = publicLogicalPagePhysicalIds(visibleView.counts, nextPageIndex);
    const missingIds = physicalIds.filter((pageId) => !visibleView.pageCache[pageId]);
    const request = requestRef.current;
    setView({ ...visibleView, pageLoading: true, pageError: null });
    try {
      const loadedPages = await Promise.all(missingIds.map((pageId) => loadCatalogPage(uid, pageId)));
      if (requestRef.current !== request) return;
      if (loadedPages.some((page, index) => (
        !page || page.generation !== visibleView.profile!.generation || page.pageId !== missingIds[index]
          || page.entries.length !== visibleView.counts![missingIds[index]!]
      ))) {
        throw new Error(pageMismatchMessage());
      }
      const pageCache = {
        ...visibleView.pageCache,
        ...Object.fromEntries(loadedPages.map((page) => [page!.pageId, page!])),
      };
      setView({
        ...visibleView,
        pageCache,
        pageIndex: nextPageIndex,
        pageLoading: false,
        pageError: null,
      });
    } catch (error) {
      if (requestRef.current !== request) return;
      if (isUnavailableReadError(error)) {
        setView(unavailableView(uid, "This public player is unavailable."));
        return;
      }
      setView({
        ...visibleView,
        pageLoading: false,
        pageError: errorText(error, "This catalog page could not be loaded."),
      });
    }
  };

  const retryLabel = visibleView?.status === "loading" ? "Loading" : "Try again";

  return (
    <main className={styles.page}>
      <section className={`${styles.pageCard} uiPanel`} aria-labelledby="public-player-title">
        <p className="uiPageEyebrow">Gomoku 2D · Public profile</p>
        {visibleView?.status === "ready" && visibleView.profile ? (
          <>
            <h1 className={styles.pageTitle} id="public-player-title">{visibleView.profile.displayName}</h1>
            {visibleView.profile.sharedStats ? (
              <section className={styles.publicStats} aria-label="Shared player statistics">
                <div className={styles.publicStatsGrid}>
                  {([
                    ["Matches", visibleView.profile.sharedStats.matches],
                    ["Wins", visibleView.profile.sharedStats.wins],
                    ["Losses", visibleView.profile.sharedStats.losses],
                    ["Draws", visibleView.profile.sharedStats.draws],
                    ["Win rate", (() => {
                      const rate = playerWinRate(visibleView.profile!.sharedStats!);
                      return rate === null ? "—" : `${(rate * 100).toFixed(1)}%`;
                    })()],
                  ] as const).map(([label, value]) => (
                    <div className={styles.publicStatsTile} key={label}>
                      <span className={styles.publicStatsValue}>{value}</span>
                      <span className={styles.publicStatsLabel}>{label}</span>
                    </div>
                  ))}
                </div>
                <p className={styles.publicStatsDate}>
                  Updated {publicProfileDateLabel(visibleView.profile.sharedStats.updatedAt)}
                </p>
              </section>
            ) : null}
            <p className={styles.mutedText}>Games selected for public sharing</p>
            <p className={styles.disclosure}>
              Player names and game details are supplied by the publisher. These games are casual records and are not independently verified.
            </p>
            {visibleRows.length ? (
              <ol className={styles.publicGameList} aria-label={`Public games, page ${visibleView.pageIndex + 1}`}>
                {visibleRows.map(({ entry, details }) => (
                  <li className={styles.publicGame} key={entry.fingerprint}>
                    <span className={styles.publicGameTitle}>{entry.title}</span>
                    <span className={styles.publicGameOpponent}>
                      <span className="uiSrOnly">Opponent: </span>
                      {details.opponent ? `vs ${details.opponent}` : "Unavailable"}
                    </span>
                    <span className={`${styles.publicGameResult} ${
                      details.result === "Win" ? styles.publicGameResultWin
                        : details.result === "Loss" ? styles.publicGameResultLoss
                          : details.result === "Draw" ? styles.publicGameResultDraw : ""
                    }`}>
                      <span className="uiSrOnly">Result: </span>
                      {details.result ?? "Unavailable"}
                    </span>
                    <span className={styles.publicGameMeta}>
                      <span className="uiSrOnly">Rule: </span>
                      {details.rule ?? "Unavailable"}
                    </span>
                    <span className={styles.publicGameMeta}>
                      <span className="uiSrOnly">Move count: </span>
                      {details.moves === null ? "Unavailable" : `${details.moves} moves`}
                    </span>
                    {details.canReplay ? (
                      <Link
                        className="uiAction uiActionSecondary"
                        to={`/replay/shared/#g2r1=${entry.payload}`}
                      >
                        Replay
                      </Link>
                    ) : (
                      <span className={styles.replayUnavailable}>Replay unavailable</span>
                    )}
                  </li>
                ))}
              </ol>
            ) : (
              <p className={styles.emptyState}>No games have been published.</p>
            )}
            {pageCount > 1 ? (
              <div className={styles.pagingControls}>
                <span>Page {visibleView.pageIndex + 1} of {pageCount}</span>
                {visibleView.pageIndex > 0 ? (
                  <button className="uiAction uiActionNeutral" type="button" disabled={visibleView.pageLoading}
                    onClick={() => void loadLogicalPage(visibleView.pageIndex - 1)}>
                    Previous page
                  </button>
                ) : null}
                {visibleView.pageIndex < pageCount - 1 ? (
                  <button
                    className="uiAction uiActionNeutral"
                    type="button"
                    disabled={visibleView.pageLoading}
                    onClick={() => void loadLogicalPage(visibleView.pageIndex + 1)}
                  >
                    {visibleView.pageLoading ? "Loading page" : visibleView.pageError ? "Retry page" : "Next page"}
                  </button>
                ) : null}
              </div>
            ) : null}
            {visibleView.pageError ? (
              <div className={styles.pageError}>
                <p role="alert" className={styles.errorText}>{visibleView.pageError}</p>
                <button className="uiAction uiActionNeutral" type="button" onClick={() => setRetryKey((key) => key + 1)}>
                  Reload public profile
                </button>
              </div>
            ) : null}
          </>
        ) : (
          <>
            <h1 className={styles.pageTitle} id="public-player-title">
              {visibleView?.status === "loading" || !visibleView ? "Loading public profile" : "Public profile unavailable"}
            </h1>
            <p className={styles.mutedText} role={visibleView?.status === "error" ? "alert" : undefined}>
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
          </>
        )}
        <Link className="uiAction uiActionNeutral" to="/">Home</Link>
      </section>
    </main>
  );
}
