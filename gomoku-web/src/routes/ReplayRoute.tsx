import { useEffect, useLayoutEffect, useRef, useState, type ComponentProps } from "react";
import { Link, Navigate, useLocation, useNavigate, useParams, useSearchParams } from "react-router-dom";
import { useStore } from "zustand";

import { useDocumentTitle } from "../app/useDocumentTitle";
import { buildReplayBoardModel } from "../board/board_model";
import { localProfileStore } from "../profile/local_profile_store";
import { cloudAuthStore } from "../cloud/auth_store";
import {
  buildLocalReplayFrame,
  defaultReplayMoveIndex,
  shouldShowReplaySequenceNumbers,
} from "../replay/local_replay";
import {
  analysisOverlaysForFrame,
  nextReplayMove,
  replayAnalysisStatusSummary,
  replayTimelineAnalysis,
} from "../replay/replay_analysis_overlays";
import { useReplayAnalysis } from "../replay/use_replay_analysis";
import { useReplayMatch } from "../replay/use_replay_match";
import { publishReplaySide } from "../replay/publish_replay_eligibility";
import { useReplayWinningCells } from "../replay/use_replay_winning_cells";
import { Icon } from "../ui/Icon";
import type { SavedMatchV2 } from "../match/saved_match";
import type { PortableReplayV1 } from "../replay/portable_replay";
import { ShareReplayPanel } from "./ShareReplayPanel";
import { SaveReplayButton } from "./ReplayLibraryPanel";
import { StoredReplayRoute } from "./StoredReplayRoute";
import { PublishReplayPanel } from "./PublishReplayPanel";
import { PublishedReplayEditRoute } from "./PublishedReplayEditRoute";
import { ReplayDetailsPanel, type ReplayDetailsBinding, type ReplayDraftState } from "./ReplayDetailsPanel";
import { protectReplayDraftNavigation } from "./replay_draft_navigation";
import { portableReplayFromSavedMatch, validatePortableReplay } from "../replay/portable_replay";
import { portableReplayFingerprint } from "../replay/portable_replay_codec";
import { ensureLocalMatchSession, localMatchSessionStore } from "../game/local_match_session";
import { botLabel } from "../core/bot_config";
import { publicProfilePath, type PublicReplayReturn } from "./public_player_navigation";
import { myGamesPublishIntentFromState, myGamesReturnFromState } from "./my_games_navigation";

import styles from "./ReplayRoute.module.css";
import {
  ReplayBoardPanel,
  ReplayMatchPanel,
  ReplayPlaybackPanel,
  ReplayStatusPanel,
} from "./ReplayRoutePanels";

const AUTOPLAY_DELAY_MS = 700;
const PUBLISHED_EDIT_PREFIX = "published-edit-";

export function ReplayViewer({ match, localDisplayName, preserveNames = false, initialMoveIndex, source,
  fingerprint, publishEligible, contextLabel, publicReplayReturn, details, privateReplayReturn }: {
  match: SavedMatchV2;
  localDisplayName: string;
  preserveNames?: boolean;
  initialMoveIndex?: number;
  source?: PortableReplayV1;
  fingerprint?: string;
  publishEligible?: boolean;
  contextLabel?: string;
  publicReplayReturn?: PublicReplayReturn;
  details?: ReplayDetailsBinding;
  privateReplayReturn?: { to: string; state?: unknown };
}) {
  useDocumentTitle("Replay Analysis");
  const navigate = useNavigate();
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const replayFloor = match.undo_floor;
  const settings = useStore(localProfileStore, (state) => state.settings);
  const localProfileId = useStore(localProfileStore, (state) => state.profile?.id ?? null);
  const signedInUid = useStore(cloudAuthStore, (state) => state.status === "signed_in" ? state.user?.uid ?? null : null);
  const ownerPractice = Boolean(preserveNames && publishEligible && source?.practice);
  const canPublish = (publishEligible ?? !preserveNames)
    && Boolean(publishReplaySide(match, signedInUid, localProfileId, ownerPractice));
  const launch = myGamesPublishIntentFromState(location.state);
  const launchAllowed = launch && launch.matchId === match.id && launch.uid === signedInUid && canPublish;
  const origin = myGamesReturnFromState(location.state, signedInUid);
  const historyReturn = origin?.matchId === match.id ? origin : undefined;
  const inspection = myGamesPublishIntentFromState(location.state, "myGamesInspectTarget");
  const inspectionMove = inspection?.matchId === match.id && inspection.uid === signedInUid ? inspection.inspectMove : undefined;
  const startMove = Math.min(match.move_count, launchAllowed ? launch.inspectMove ?? match.move_count
    : inspectionMove ?? (historyReturn ? match.move_count : initialMoveIndex ?? defaultReplayMoveIndex(match.move_count)));
  const [moveIndex, setMoveIndex] = useState(startMove);
  const [autoplaying, setAutoplaying] = useState(false);
  const [branching, setBranching] = useState(false);
  const [branchError, setBranchError] = useState("");
  const branchRequestRef = useRef<object | null>(null);
  const [sidebar, setSidebar] = useState<"replay" | "share" | "publish" | "details">(
    launchAllowed ? "publish" : details?.kind === "public" ? "details" : "replay",
  );
  const [publishEntry, setPublishEntry] = useState<{ uid: string; move: number; fromRow: boolean } | null>(
    launchAllowed ? { uid: launch.uid, move: startMove, fromRow: launch.inspectMove === undefined } : null,
  );
  const [detailsNotice, setDetailsNotice] = useState("");
  const publishOpen = sidebar === "publish" && canPublish && publishEntry?.uid === signedInUid;
  const sidebarOpen = sidebar !== "replay";
  const deckRef = useRef<HTMLElement>(null);
  const detailsTriggerRef = useRef<HTMLButtonElement>(null);
  const draftState = useRef<ReplayDraftState | null>(null);
  const shareTriggerRef = useRef<HTMLButtonElement>(null);
  const publishTriggerRef = useRef<HTMLButtonElement>(null);
  const deckScrollRef = useRef(0);
  const previousSidebarRef = useRef(sidebar);
  const { analysisAnnotations, analysisStep } = useReplayAnalysis(match);
  const coreWinningCells = useReplayWinningCells(match, moveIndex);

  useLayoutEffect(() => {
    branchRequestRef.current = null;
    setBranching(false);
    setBranchError("");
    return () => {
      branchRequestRef.current = null;
    };
  }, [location.hash, location.key, location.pathname, location.search, location.state, match.id, moveIndex]);

  useEffect(() => {
    setMoveIndex(startMove);
    setAutoplaying(false);
  }, [match.id, startMove]);

  useEffect(() => {
    if (!match || !autoplaying) {
      return undefined;
    }

    if (moveIndex >= match.move_count) {
      setAutoplaying(false);
      return undefined;
    }

    const timer = window.setTimeout(() => {
      setMoveIndex((current) => Math.min(match.move_count, current + 1));
    }, AUTOPLAY_DELAY_MS);

    return () => {
      window.clearTimeout(timer);
    };
  }, [autoplaying, match, moveIndex]);

  useEffect(() => {
    if (sidebarOpen) {
      if (deckRef.current) deckRef.current.scrollTop = 0;
    } else if (previousSidebarRef.current !== "replay") {
      const trigger = previousSidebarRef.current === "share" ? shareTriggerRef
        : previousSidebarRef.current === "details" ? detailsTriggerRef : publishTriggerRef;
      trigger.current?.focus({ preventScroll: true });
      if (deckRef.current) deckRef.current.scrollTop = deckScrollRef.current;
    }
    previousSidebarRef.current = sidebar;
  }, [sidebar, sidebarOpen]);

  useEffect(() => {
    if (inspectionMove === undefined) return;
    publishTriggerRef.current?.focus({ preventScroll: true });
    publishTriggerRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [inspectionMove]);

  useEffect(() => {
    if (sidebar === "publish" && !publishOpen) {
      setPublishEntry(null);
      setSidebar("replay");
    }
  }, [publishOpen, sidebar]);

  function canLeaveDetails(): boolean {
    if (draftState.current?.saving) return false;
    return !draftState.current?.dirty || window.confirm("Discard unsaved replay details?");
  }

  useEffect(() => {
    if (sidebar !== "details") return;
    const beforeUnload = (event: BeforeUnloadEvent) => {
      if (!draftState.current?.dirty && !draftState.current?.saving) return;
      event.preventDefault();
      event.returnValue = "";
    };
    const unprotect = protectReplayDraftNavigation(canLeaveDetails);
    window.addEventListener("beforeunload", beforeUnload);
    return () => {
      unprotect();
      window.removeEventListener("beforeunload", beforeUnload);
    };
  }, [sidebar]);

  const openSidebar = (panel: "share" | "publish" | "details") => {
    if (panel === "publish") {
      if (!canPublish || !signedInUid) return;
      setPublishEntry({ uid: signedInUid, move: frame.moveIndex, fromRow: false });
    }
    deckScrollRef.current = deckRef.current?.scrollTop ?? 0;
    setAutoplaying(false);
    setDetailsNotice("");
    setSidebar(panel);
  };

  const closePublish = () => {
    if (publishEntry?.fromRow) {
      navigate("/profile", { replace: true, state: { myGamesReplayReturn: historyReturn, myGamesFocusTab: !historyReturn } });
    } else if (publishEntry) {
      setMoveIndex(publishEntry.move);
      setPublishEntry(null);
      setSidebar("replay");
    }
  };

  const manageProfileForPublish = () => {
    if (!publishOpen || !publishEntry) return;
    const { status, user } = cloudAuthStore.getState();
    if (status !== "signed_in" || user?.uid !== publishEntry.uid) return;
    navigate("/profile?tab=public", { replace: true, state: {
      myGamesPublishIntent: {
        uid: publishEntry.uid,
        matchId: match.id,
        ...(publishEntry.fromRow ? {} : { inspectMove: publishEntry.move }),
      },
      myGamesReplayReturn: historyReturn,
    } });
  };

  const frame = buildLocalReplayFrame(match, moveIndex, () => coreWinningCells);
  const analysisSummary = analysisStep?.analysis ?? null;
  const analysisOverlays = analysisOverlaysForFrame(
    analysisAnnotations,
    match,
    frame.moveIndex,
    analysisSummary,
    settings.boardHints.evidence === "on",
  );
  const analysisStatus = replayAnalysisStatusSummary(analysisStep, analysisAnnotations, match, frame);
  const timelineAnalysis = replayTimelineAnalysis(analysisAnnotations, match.move_count, analysisSummary);
  const timelineStyle = {
    "--timeline-analyzed-end": timelineAnalysis.analyzedEndPercent ?? "0%",
    "--timeline-analyzed-start": timelineAnalysis.analyzedStartPercent ?? "0%",
    "--timeline-lethal-onset": timelineAnalysis.lethalOnsetPercent ?? "0%",
    "--timeline-lethal-tail-end": timelineAnalysis.lethalTailEndPercent ?? "0%",
    "--timeline-lethal-tail-start": timelineAnalysis.lethalTailStartPercent ?? "0%",
    "--timeline-setup-end": timelineAnalysis.setupEndPercent ?? "0%",
    "--timeline-setup-start": timelineAnalysis.setupStartPercent ?? "0%",
    "--timeline-escape": timelineAnalysis.escapePercent ?? "0%",
  } as React.CSSProperties;
  const replayMovePreview = nextReplayMove(match, frame.moveIndex);
  const mediaMode = searchParams.get("media");
  const localMatchResumePath = mediaMode
    ? `/match/local?media=${encodeURIComponent(mediaMode)}`
    : "/match/local";
  const boardModel = buildReplayBoardModel({
    analysisOverlays,
    nextReplayMove: replayMovePreview,
    position: {
      cells: frame.cells,
      currentPlayer: frame.currentPlayer,
      lastMove: frame.lastMove,
      moves: frame.moves,
      showSequenceNumbers: shouldShowReplaySequenceNumbers(frame),
      status: frame.status,
    },
    winningCells: frame.winningCells,
  });
  const playbackProps: ComponentProps<typeof ReplayPlaybackPanel> = {
    autoplaying,
    resumeBotLabel: botLabel(settings.botConfig),
    frame,
    match,
    onResume: async (resumeSeed) => {
      if (branchRequestRef.current) return;
      const request = {};
      branchRequestRef.current = request;
      let launched = false;
      setAutoplaying(false);
      setBranching(true); setBranchError("");
      const isCurrentRequest = () => branchRequestRef.current === request;
      try {
        const portable = source ?? portableReplayFromSavedMatch(match).replay;
        const sourceFingerprint = fingerprint ?? await portableReplayFingerprint(validatePortableReplay(portable));
        if (!isCurrentRequest()) return;
        const active = localMatchSessionStore.getState().matchStore?.getState();
        if (active?.status === "playing" && active.moves.length > 0
          && !window.confirm("Replace your active game with a new game from this position?")) return;
        if (!isCurrentRequest()) return;
        ensureLocalMatchSession({ resumeState: {
          ...resumeSeed, practiceOrigin: { sourceFingerprint, inheritedPly: resumeSeed.moves.length },
        } });
        navigate(localMatchResumePath);
        launched = true;
      } catch (error) {
        if (isCurrentRequest()) setBranchError(error instanceof Error ? error.message : "The game could not start.");
      } finally {
        if (!launched && isCurrentRequest()) {
          branchRequestRef.current = null;
          setBranching(false);
        }
      }
    },
    replayFloor,
    resumePending: branching,
    setAutoplaying,
    setMoveIndex,
    timelineAnalysis,
    timelineStyle,
  };

  if (launch && !launchAllowed) return <Navigate to="/profile" replace state={{ myGamesFocusTab: true }} />;

  return (
    <main className={styles.page} onKeyDown={(event) => {
      if (event.key !== "Escape" || sidebar !== "replay" || !historyReturn) return;
      event.preventDefault();
      navigate("/profile", { replace: true, state: { myGamesReplayReturn: historyReturn } });
    }} onClickCapture={(event) => {
      if (sidebar !== "details" || event.button !== 0 || event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) return;
      const link = (event.target as Element).closest("a");
      if (link && link.target !== "_blank" && !canLeaveDetails()) {
        event.preventDefault();
        event.stopPropagation();
      }
    }}>
      <header className={`${styles.header} ${publicReplayReturn ? styles.headerWithReturn : ""}`}>
        <div className={styles.headerCopy}>
          <p className="uiPageEyebrow">{contextLabel ?? (preserveNames ? "Shared replay" : "Saved match")}</p>
          <h1 className={styles.title}>Replay Analysis</h1>
        </div>
        {!publishOpen && <div className={`${styles.headerActions} ${publicReplayReturn ? styles.headerActionsWithReturn : ""}`}>
          {publicReplayReturn ? (
            <button
              aria-label="Return to public profile"
              className={`uiAction uiActionSecondary ${styles.returnAction}`}
              onClick={() => navigate(
                publicProfilePath(publicReplayReturn.uid, publicReplayReturn.page - 1),
                { state: { publicReplayReturn } },
              )}
              type="button"
            >
              <Icon className="uiIconDesktop" name="back" />
              <span className={`uiActionLabel ${styles.returnActionFull}`}>Return to public profile</span>
              <span className={`uiActionLabel ${styles.returnActionCompact}`} aria-hidden="true">Return</span>
            </button>
          ) : null}
          <Link aria-label={privateReplayReturn ? "Saved replays" : publicReplayReturn ? "My profile" : historyReturn ? "My games" : "Profile"}
            className="uiAction uiActionSecondary" to={privateReplayReturn?.to ?? "/profile"}
            state={privateReplayReturn?.state ?? (historyReturn && { myGamesReplayReturn: historyReturn })}
            onClick={(event) => {
              if (!(details?.kind === "public" && details.onCancel) || event.button !== 0
                || event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) return;
              event.preventDefault();
              details.onCancel();
            }}>
            <Icon className="uiIconDesktop" name="profile" />
            <span className="uiActionLabel">{privateReplayReturn ? "Saved replays" : publicReplayReturn ? "My profile" : historyReturn ? "My games" : "Profile"}</span>
          </Link>
          <Link aria-label="Home" className="uiAction uiActionNeutral" to="/">
            <Icon className="uiIconDesktop" name="home" />
            <span className="uiActionLabel">Home</span>
          </Link>
        </div>}
      </header>

      <section className={`${styles.layout} ${sidebarOpen ? styles.layoutSharing : ""}`}>
        <ReplayBoardPanel model={boardModel} />

        <aside className={`${styles.deck} ${sidebarOpen ? styles.deckSharing : ""}`} ref={deckRef}>
          <div className={styles.replayDeck} hidden={sidebarOpen}>
            <ReplayStatusPanel detail={analysisStatus.detail} label={analysisStatus.label} />

            <div className="uiDivider" />

            <ReplayMatchPanel frame={frame} localDisplayName={localDisplayName} match={match} preserveNames={preserveNames} />

            <div className="uiDivider" />

            {sidebar !== "details" && <ReplayPlaybackPanel {...playbackProps} />}
            {branchError && <section className={styles.deckSection}>
              <p className={styles.contextError} role="alert">{branchError}</p>
            </section>}
            <section className={`${styles.deckSection} ${styles.sharingSection}`}>
              {details?.kind === "private" && <button className="uiAction uiActionSecondary" type="button" ref={detailsTriggerRef}
                onClick={() => openSidebar("details")}><Icon name="settings" /><span className="uiActionLabel">Edit details</span></button>}
              <button className="uiAction uiActionSecondary" type="button" ref={shareTriggerRef}
                onClick={() => openSidebar("share")}><Icon name="share" /><span className="uiActionLabel">Share replay</span></button>
              {canPublish && <button className="uiAction uiActionSecondary" type="button" ref={publishTriggerRef}
                onClick={() => openSidebar("publish")}><Icon name="publish" /><span className="uiActionLabel">Publish replay</span></button>}
              {detailsNotice && <p className={styles.detailsNotice} role="status">{detailsNotice}</p>}
            </section>
            {source && fingerprint && <SaveReplayButton replay={source} fingerprint={fingerprint} />}
          </div>
          {sidebar === "details" && details && <ReplayDetailsPanel binding={details}
            move={frame.moveIndex} total={match.move_count} draftState={draftState}
            playback={<ReplayPlaybackPanel {...playbackProps} selectionOnly />}
            onBack={() => {
              if (!canLeaveDetails()) return;
              setAutoplaying(false);
              if (details.onCancel) details.onCancel();
              else setSidebar("replay");
            }}
            onSaved={(message) => {
              setAutoplaying(false);
              if (details.kind === "private") {
                setDetailsNotice(message ?? "Replay details saved.");
                setSidebar("replay");
              }
            }} />}
          <ShareReplayPanel match={match} moveIndex={frame.moveIndex} source={source} open={sidebar === "share"} onClose={() => setSidebar("replay")} />
          {publishOpen && publishEntry && <PublishReplayPanel
            key={`${publishEntry.uid}:${match.id}`} match={match} source={source} moveIndex={frame.moveIndex}
            eligible open onClose={closePublish} entryUid={publishEntry.uid} entryMove={publishEntry.move}
            ownerPractice={ownerPractice} backToMyGames={publishEntry.fromRow}
            playback={<ReplayPlaybackPanel {...playbackProps} selectionOnly />}
            onMoveChange={(move) => { setAutoplaying(false); setMoveIndex(move); }}
            onManageProfile={manageProfileForPublish}
          />}
        </aside>
      </section>
    </main>
  );
}

export function ReplayRoute() {
  const { matchId } = useParams<{ matchId: string }>();
  const location = useLocation();
  const auth = useStore(cloudAuthStore);
  const launch = myGamesPublishIntentFromState(location.state);
  if (launch && auth.status === "loading") return <ReplayUnavailable loading />;
  if (launch && (launch.matchId !== matchId || auth.status !== "signed_in" || auth.user?.uid !== launch.uid)) {
    return <Navigate to="/profile" replace state={{ myGamesFocusTab: true }} />;
  }
  if (matchId?.startsWith(PUBLISHED_EDIT_PREFIX)) {
    return <PublishedReplayEditRoute fingerprint={matchId.slice(PUBLISHED_EDIT_PREFIX.length)} />;
  }
  const key = launch ? location.key : matchId;
  return matchId?.startsWith("portable-") ? <StoredReplayRoute key={key} fingerprint={matchId.slice(9)} /> : <PrivateReplayRoute key={key} matchId={matchId} />;
}

function PrivateReplayRoute({ matchId }: { matchId?: string }) {
  const location = useLocation();
  const { localDisplayName, match, replayMayStillLoad } = useReplayMatch(matchId);
  if (match) return <ReplayViewer key={matchId} match={match} localDisplayName={localDisplayName} />;
  if (!replayMayStillLoad && myGamesPublishIntentFromState(location.state)) {
    return <Navigate to="/profile" replace state={{ myGamesFocusTab: true }} />;
  }
  return <ReplayUnavailable loading={replayMayStillLoad} />;
}

function ReplayUnavailable({ loading }: { loading: boolean }) {
  return (
    <main className={styles.page}>
      <section className={`${styles.notFound} uiPanel`}>
        <h1 className={styles.title}>{loading ? "Loading replay" : "Replay unavailable"}</h1>
        <p className={styles.notFoundText}>{loading ? "Checking saved match history." : "This replay is no longer available."}</p>
        <Link className="uiAction uiActionSecondary" to="/profile">Back to Profile</Link>
      </section>
    </main>
  );
}
