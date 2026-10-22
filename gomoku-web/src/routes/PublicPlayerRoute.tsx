import { useEffect, useRef, useState } from "react";
import { Link, useLocation } from "react-router-dom";

import { useDocumentTitle } from "../app/useDocumentTitle";
import {
  loadCatalogPage,
  loadPublicProfile,
  type PublicCatalogPageSnapshot,
  type PublicProfileSnapshot,
} from "../cloud/sharing_public";

import styles from "./SharingPanels.module.css";

const PUBLIC_UID_PATTERN = /^[A-Za-z0-9_-]{1,128}$/;

interface PublicPlayerView {
  uid: string;
  status: "loading" | "unavailable" | "error" | "ready";
  profile?: PublicProfileSnapshot;
  page?: PublicCatalogPageSnapshot;
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
      setView({
        uid,
        status: "unavailable",
        pageCache: {},
        pageIndex: 0,
        pageLoading: false,
        pageError: null,
        message: "This public player link is unavailable.",
      });
      return () => { cancelled = true; };
    }

    setView({
      uid,
      status: "loading",
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
        if (!profile?.enabled) {
          setView({
            uid,
            status: "unavailable",
            pageCache: {},
            pageIndex: 0,
            pageLoading: false,
            pageError: null,
            message: "This public player is unavailable.",
          });
          return;
        }

        const page = await loadCatalogPage(uid, 0);
        if (!current()) return;
        if (!page || page.generation !== profile.generation) {
          throw new Error("The public catalog is unavailable. Please try again.");
        }
        setView({
          uid,
          status: "ready",
          profile,
          page,
          pageCache: { 0: page },
          pageIndex: 0,
          pageLoading: false,
          pageError: null,
          message: null,
        });
      } catch (error) {
        if (!current()) return;
        setView({
          uid,
          status: isUnavailableReadError(error) ? "unavailable" : "error",
          pageCache: {},
          pageIndex: 0,
          pageLoading: false,
          pageError: null,
          message: isUnavailableReadError(error)
            ? "This public player is unavailable."
            : errorText(error, "This public player could not be loaded."),
        });
      }
    })();

    return () => { cancelled = true; };
  }, [retryKey, uid]);

  const visibleView = view?.uid === uid ? view : null;
  const loadNextPage = async () => {
    if (!visibleView?.profile || visibleView.pageIndex >= 3 || visibleView.pageLoading) return;
    const nextPageIndex = visibleView.pageIndex + 1;
    const cachedPage = visibleView.pageCache[nextPageIndex];
    if (cachedPage) {
      setView({
        ...visibleView,
        page: cachedPage,
        pageIndex: nextPageIndex,
        pageLoading: false,
        pageError: null,
      });
      return;
    }
    const request = requestRef.current;
    setView({ ...visibleView, pageLoading: true, pageError: null });
    try {
      const page = await loadCatalogPage(uid, nextPageIndex);
      if (requestRef.current !== request) return;
      if (!page || page.generation !== visibleView.profile.generation) {
        throw new Error("This catalog page is unavailable. Please try again.");
      }
      setView({
        ...visibleView,
        page,
        pageCache: { ...visibleView.pageCache, [nextPageIndex]: page },
        pageIndex: nextPageIndex,
        pageLoading: false,
        pageError: null,
      });
    } catch (error) {
      if (requestRef.current !== request) return;
      if (isUnavailableReadError(error)) {
        setView({
          uid,
          status: "unavailable",
          pageCache: {},
          pageIndex: 0,
          pageLoading: false,
          pageError: null,
          message: "This public player is unavailable.",
        });
        return;
      }
      setView({
        ...visibleView,
        pageLoading: false,
        pageError: errorText(error, "This catalog page could not be loaded."),
      });
    }
  };

  const loadPreviousPage = () => {
    if (!visibleView?.profile || visibleView.pageIndex <= 0 || visibleView.pageLoading) return;
    const previousIndex = visibleView.pageIndex - 1;
    const cachedPage = visibleView.pageCache[previousIndex];
    if (!cachedPage) return;
    setView({
      ...visibleView,
      page: cachedPage,
      pageIndex: previousIndex,
      pageError: null,
    });
  };

  const retryLabel = visibleView?.status === "loading" ? "Loading" : "Try again";

  return (
    <main className={styles.page}>
      <section className={`${styles.pageCard} uiPanel`} aria-labelledby="public-player-title">
        <p className="uiPageEyebrow">Gomoku 2D · Public profile</p>
        {visibleView?.status === "ready" && visibleView.profile ? (
          <>
            <h1 className={styles.pageTitle} id="public-player-title">{visibleView.profile.displayName}</h1>
            <p className={styles.mutedText}>Games selected for public sharing</p>
            <p className={styles.disclosure}>
              Player names and game details are supplied by the publisher. These games are casual records and are not independently verified.
            </p>
            {visibleView.page?.entries.length ? (
              <ol className={styles.publicGameList} aria-label={`Public games, page ${visibleView.pageIndex + 1}`}>
                {visibleView.page.entries.map((entry) => (
                  <li className={styles.publicGame} key={entry.fingerprint}>
                    <span className={styles.publicGameTitle}>{entry.title}</span>
                    <span className={styles.publicGameMeta}>Opens at move {entry.view_ply}</span>
                    <Link
                      className="uiAction uiActionSecondary"
                      to={`/replay/shared/#g2r1=${entry.payload}`}
                    >
                      View replay
                    </Link>
                  </li>
                ))}
              </ol>
            ) : (
              <p className={styles.emptyState}>No games have been published on this page.</p>
            )}
            <div className={styles.pagingControls}>
              <span>Page {visibleView.pageIndex + 1} of 4</span>
              {visibleView.pageIndex > 0 && (
                <button className="uiAction uiActionNeutral" type="button" disabled={visibleView.pageLoading}
                  onClick={loadPreviousPage}>
                  Previous page
                </button>
              )}
              {visibleView.pageIndex < 3 && (
                <button
                  className="uiAction uiActionNeutral"
                  type="button"
                  disabled={visibleView.pageLoading}
                  onClick={() => void loadNextPage()}
                >
                  {visibleView.pageLoading ? "Loading page" : visibleView.pageError ? "Retry page" : "Next page"}
                </button>
              )}
            </div>
            {visibleView.pageError && <p role="alert" className={styles.errorText}>{visibleView.pageError}</p>}
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
