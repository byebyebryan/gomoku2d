import { useEffect, useRef, useState, type ComponentProps } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { useStore } from "zustand";

import { useDocumentTitle } from "../app/useDocumentTitle";
import { cloudAuthStore, type CloudAuthUser } from "../cloud/auth_store";
import {
  editPublishedReplay,
  loadCatalogPage,
  loadPublicProfile,
  type PublicCatalogEntry,
  type PublicCatalogPageSnapshot,
  type PublicProfileSnapshot,
} from "../cloud/sharing_public";
import { SharingConflictError } from "../cloud/sharing_common";
import { portableReplaySavedMatch } from "../replay/portable_replay_adapters";
import { decodePortableReplay, portableReplayFingerprint } from "../replay/portable_replay_codec";
import type { PrivateReplayMetadata } from "../replay/private_replay_metadata";
import type { SavedMatchV2 } from "../match/saved_match";
import { ReplayViewer } from "./ReplayRoute";
import styles from "./ReplayRoute.module.css";

interface PublishedReplayEditReturn {
  fingerprint: string;
  pageId: number;
  scrollTop: number;
}

interface LoadedPublication {
  uid: string;
  sessionUser: CloudAuthUser;
  profile: PublicProfileSnapshot;
  page: PublicCatalogPageSnapshot;
  entry: PublicCatalogEntry;
  match: SavedMatchV2;
  replay: ReturnType<typeof decodePortableReplay>["replay"];
  openingPly: number;
}

type EditRouteState =
  | { uid: string; sessionUser: CloudAuthUser; status: "loading" }
  | { uid: string; sessionUser: CloudAuthUser; status: "error"; message: string }
  | ({ status: "ready" } & LoadedPublication);

function validPageId(search: string): number | null {
  const values = new URLSearchParams(search).getAll("page");
  return values.length === 1 && /^[0-3]$/.test(values[0] ?? "") ? Number(values[0]) : null;
}

function publishedReplayEditReturnFromState(
  value: unknown,
  fingerprint: string,
  pageId: number | null,
): PublishedReplayEditReturn | null {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return null;
  const origin = (value as Record<string, unknown>).publishedReplayEditReturn;
  if (typeof origin !== "object" || origin === null || Array.isArray(origin)) return null;
  const record = origin as Record<string, unknown>;
  if (record.fingerprint !== fingerprint || !Number.isInteger(record.pageId)
    || record.pageId !== pageId || !Number.isInteger(record.scrollTop)
    || (record.scrollTop as number) < 0 || (record.scrollTop as number) > 1_000_000) return null;
  return {
    fingerprint,
    pageId: record.pageId as number,
    scrollTop: record.scrollTop as number,
  };
}

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error && error.message ? error.message : fallback;
}

export function PublishedReplayEditRoute({ fingerprint }: { fingerprint: string }) {
  useDocumentTitle("Edit publication");
  const location = useLocation();
  const navigate = useNavigate();
  const auth = useStore(cloudAuthStore);
  const sessionUser = auth.status === "signed_in" ? auth.user : null;
  const uid = sessionUser?.uid ?? null;
  const pageId = validPageId(location.search);
  const returnState = publishedReplayEditReturnFromState(location.state, fingerprint, pageId)
    ?? (pageId === null ? null : { fingerprint, pageId, scrollTop: 0 });
  const [routeState, setRouteState] = useState<EditRouteState | null>(null);
  const [reloadKey, setReloadKey] = useState(0);
  const requestRef = useRef(0);
  const latestUidRef = useRef<string | null>(uid);
  latestUidRef.current = uid;
  const initialSessionRef = useRef<CloudAuthUser | null>(null);
  const closingForSessionChangeRef = useRef(false);

  const returnToManager = () => navigate("/profile?tab=public", {
    replace: true,
    state: returnState ? { publishedReplayEditReturn: returnState } : undefined,
  });

  useEffect(() => {
    if (initialSessionRef.current !== null && initialSessionRef.current !== sessionUser) {
      closingForSessionChangeRef.current = true;
      initialSessionRef.current = sessionUser;
      requestRef.current += 1;
      returnToManager();
      return;
    }
    if (sessionUser) initialSessionRef.current = sessionUser;
  }, [sessionUser]);

  useEffect(() => {
    const request = ++requestRef.current;
    let cancelled = false;
    const isCurrent = () => !cancelled
      && latestUidRef.current === uid
      && cloudAuthStore.getState().status === "signed_in"
      && cloudAuthStore.getState().user?.uid === uid
      && cloudAuthStore.getState().user === sessionUser
      && requestRef.current === request;

    if (!uid) {
      setRouteState(null);
      return () => { cancelled = true; };
    }
    if (closingForSessionChangeRef.current) {
      setRouteState(null);
      return () => { cancelled = true; };
    }
    if (!/^[A-Za-z0-9_-]{43}$/.test(fingerprint)) {
      setRouteState({ uid, sessionUser: sessionUser!, status: "error", message: "This publication link is invalid." });
      return () => { cancelled = true; };
    }
    if (pageId === null) {
      setRouteState({ uid, sessionUser: sessionUser!, status: "error", message: "The publication page is missing. Return to Public settings and open it again." });
      return () => { cancelled = true; };
    }

    setRouteState({ uid, sessionUser: sessionUser!, status: "loading" });
    void (async () => {
      try {
        const [profile, page] = await Promise.all([
          loadPublicProfile(uid),
          loadCatalogPage(uid, pageId),
        ]);
        if (!isCurrent()) return;
        if (!profile?.enabled || profile.uid !== uid) {
          throw new SharingConflictError("This public profile is disabled or unavailable. Reload before editing this publication.");
        }
        if (!page || page.uid !== uid || page.pageId !== pageId || page.generation !== profile.generation) {
          throw new SharingConflictError("The public catalog page changed or is missing. Reload Public settings before editing.");
        }
        if (profile.catalogPageCounts !== null && page.entries.length !== profile.catalogPageCounts[pageId]) {
          throw new SharingConflictError("The public catalog no longer matches its page counts. Reload Public settings before editing.");
        }
        const entry = page.entries.find((candidate) => candidate.fingerprint === fingerprint);
        if (!entry) {
          throw new SharingConflictError("This publication was removed. Edits cannot recreate a removed listing.");
        }
        const validated = decodePortableReplay(entry.payload);
        const actualFingerprint = await portableReplayFingerprint(validated);
        if (!isCurrent()) return;
        if (actualFingerprint !== fingerprint) {
          throw new Error("This publication payload does not match its fingerprint.");
        }
        const openingPly = validated.replay.viewPly ?? validated.replay.moves.length;
        const match = portableReplaySavedMatch(validated, fingerprint, new Date().toISOString());
        setRouteState({
          uid,
          sessionUser: sessionUser!,
          status: "ready",
          profile,
          page,
          entry,
          match,
          replay: validated.replay,
          openingPly,
        });
      } catch (error) {
        if (isCurrent()) setRouteState({ uid, sessionUser: sessionUser!, status: "error", message: errorMessage(error, "This publication could not be loaded.") });
      }
    })();

    return () => { cancelled = true; };
  }, [auth.user, fingerprint, pageId, reloadKey, sessionUser, uid]);

  const visible = routeState?.uid === uid && routeState.sessionUser === sessionUser ? routeState : null;
  if (!uid) return <main className={styles.page}><section className={`${styles.notFound} uiPanel`}>
    <h1>{auth.status === "loading" ? "Checking account" : "Sign in to edit this publication"}</h1>
    <p>{auth.status === "loading" ? "Waiting for the current account." : "Sign in as the owner of this public profile, then reopen the publication from Public settings."}</p>
    <Link className="uiAction uiActionSecondary" to="/profile?tab=public">Public settings</Link>
  </section></main>;

  if (visible?.status === "ready") {
    const publication = visible;
    const details: NonNullable<ComponentProps<typeof ReplayViewer>["details"]> = {
      kind: "public",
      value: { title: publication.entry.title, note: "", opening_ply: publication.openingPly },
      returnLabel: "Published games",
      onCancel: returnToManager,
      onSave: async (draft: PrivateReplayMetadata) => {
        const current = cloudAuthStore.getState();
        if (current.status !== "signed_in" || current.user?.uid !== publication.uid
          || current.user !== publication.sessionUser) {
          throw new SharingConflictError("The signed-in account changed. Reopen this publication from its owner profile.");
        }
        if (draft.note) throw new Error("Public publications do not store private replay notes.");
        const effectiveOpening = draft.opening_ply ?? publication.replay.moves.length;
        const unchanged = draft.title === publication.entry.title && effectiveOpening === publication.openingPly;
        if (unchanged) {
          returnToManager();
          return;
        }
        await editPublishedReplay(
          publication.uid,
          publication.page.pageId,
          fingerprint,
          { title: draft.title, opening_ply: draft.opening_ply },
          publication.profile.revision,
          publication.profile.generation,
          publication.page.revision,
          {},
          () => {
            const latest = cloudAuthStore.getState();
            return latest.status === "signed_in" && latest.user?.uid === publication.uid
              && latest.user === publication.sessionUser;
          },
        );
        const afterSave = cloudAuthStore.getState();
        if (afterSave.status !== "signed_in" || afterSave.user?.uid !== publication.uid
          || afterSave.user !== publication.sessionUser) {
          throw new SharingConflictError("The signed-in account changed before the publication update finished.");
        }
        returnToManager();
      },
    };
    return <ReplayViewer key={`${publication.uid}:${fingerprint}:${publication.page.revision}`}
      match={publication.match}
      localDisplayName=""
      preserveNames
      initialMoveIndex={publication.openingPly}
      source={publication.replay}
      fingerprint={fingerprint}
      publishEligible={false}
      contextLabel="Published game"
      details={details} />;
  }

  const error = visible?.status === "error" ? visible.message : null;
  return <main className={styles.page}><section className={`${styles.notFound} uiPanel`}>
    <h1>{error ? "Publication unavailable" : "Loading publication"}</h1>
    <p>{error ?? "Checking the current public catalog and owner account."}</p>
    <div>
      {error && <button className="uiAction uiActionPrimary" type="button" onClick={() => setReloadKey((key) => key + 1)}>Reload publication</button>}
      <button className="uiAction uiActionSecondary" type="button" onClick={returnToManager}>Public settings</button>
    </div>
  </section></main>;
}
