import { useEffect, useRef, useState } from "react";
import { Link, useNavigate, useParams, useSearchParams } from "react-router-dom";
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
import { useReplayWinningCells } from "../replay/use_replay_winning_cells";
import { Icon } from "../ui/Icon";
import type { SavedMatchV2 } from "../match/saved_match";
import type { PortableReplayV1 } from "../replay/portable_replay";
import { ShareReplayPanel } from "./ShareReplayPanel";
import { SaveReplayButton } from "./ReplayLibraryPanel";
import { StoredReplayRoute } from "./StoredReplayRoute";
import { PublishReplayPanel } from "./PublishReplayPanel";
import { portableReplayFromSavedMatch, validatePortableReplay } from "../replay/portable_replay";
import { portableReplayFingerprint } from "../replay/portable_replay_codec";
import { localMatchSessionStore } from "../game/local_match_session";
import { botLabel } from "../core/bot_config";

import styles from "./ReplayRoute.module.css";
import {
  ReplayBoardPanel,
  ReplayMatchPanel,
  ReplayPlaybackPanel,
  ReplayStatusPanel,
} from "./ReplayRoutePanels";

const AUTOPLAY_DELAY_MS = 700;

export function ReplayViewer({ match, localDisplayName, preserveNames = false, initialMoveIndex, source,
  fingerprint, publishEligible, contextLabel }: {
  match: SavedMatchV2;
  localDisplayName: string;
  preserveNames?: boolean;
  initialMoveIndex?: number;
  source?: PortableReplayV1;
  fingerprint?: string;
  publishEligible?: boolean;
  contextLabel?: string;
}) {
  useDocumentTitle("Replay Analysis");
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const replayFloor = match.undo_floor;
  const settings = useStore(localProfileStore, (state) => state.settings);
  const [moveIndex, setMoveIndex] = useState(defaultReplayMoveIndex(0));
  const [autoplaying, setAutoplaying] = useState(false);
  const [branching, setBranching] = useState(false);
  const [branchError, setBranchError] = useState("");
  const [sidebar, setSidebar] = useState<"replay" | "share" | "publish">("replay");
  const signedInUid = useStore(cloudAuthStore, (state) => state.status === "signed_in" ? state.user?.uid : null);
  const canPublish = Boolean((publishEligible ?? !preserveNames) && signedInUid);
  const sidebarOpen = sidebar !== "replay";
  const deckRef = useRef<HTMLElement>(null);
  const shareTriggerRef = useRef<HTMLButtonElement>(null);
  const publishTriggerRef = useRef<HTMLButtonElement>(null);
  const deckScrollRef = useRef(0);
  const previousSidebarRef = useRef(sidebar);
  const { analysisAnnotations, analysisStep } = useReplayAnalysis(match);
  const coreWinningCells = useReplayWinningCells(match, moveIndex);

  useEffect(() => {
    setMoveIndex(initialMoveIndex ?? defaultReplayMoveIndex(match.move_count));
    setAutoplaying(false);
  }, [match.id, match.move_count, initialMoveIndex]);

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
      const trigger = previousSidebarRef.current === "share" ? shareTriggerRef : publishTriggerRef;
      trigger.current?.focus({ preventScroll: true });
      if (deckRef.current) deckRef.current.scrollTop = deckScrollRef.current;
    }
    previousSidebarRef.current = sidebar;
  }, [sidebar, sidebarOpen]);

  useEffect(() => {
    if (!canPublish && sidebar === "publish") setSidebar("replay");
  }, [canPublish, sidebar]);

  const openSidebar = (panel: "share" | "publish") => {
    deckScrollRef.current = deckRef.current?.scrollTop ?? 0;
    setAutoplaying(false);
    setSidebar(panel);
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

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerCopy}>
          <p className="uiPageEyebrow">{contextLabel ?? (preserveNames ? "Shared replay" : "Saved match")}</p>
          <h1 className={styles.title}>Replay Analysis</h1>
        </div>
        <div className={styles.headerActions}>
          <Link aria-label="Profile" className="uiAction uiActionSecondary" to="/profile">
            <Icon className="uiIconDesktop" name="profile" />
            <span className="uiActionLabel">Profile</span>
          </Link>
          <Link aria-label="Home" className="uiAction uiActionNeutral" to="/">
            <Icon className="uiIconDesktop" name="home" />
            <span className="uiActionLabel">Home</span>
          </Link>
        </div>
      </header>

      <section className={`${styles.layout} ${sidebarOpen ? styles.layoutSharing : ""}`}>
        <ReplayBoardPanel model={boardModel} />

        <aside className={`${styles.deck} ${sidebarOpen ? styles.deckSharing : ""}`} ref={deckRef}>
          <div className={styles.replayDeck} hidden={sidebarOpen}>
            <ReplayStatusPanel detail={analysisStatus.detail} label={analysisStatus.label} />

            <div className="uiDivider" />

            <ReplayMatchPanel frame={frame} localDisplayName={localDisplayName} match={match} preserveNames={preserveNames} />

            <div className="uiDivider" />

            <ReplayPlaybackPanel
              autoplaying={autoplaying}
              resumeBotLabel={botLabel(settings.botConfig)}
              frame={frame}
              match={match}
              onResume={async (resumeSeed) => {
                setBranching(true); setBranchError("");
                try {
                  const portable = source ?? portableReplayFromSavedMatch(match).replay;
                  const sourceFingerprint = fingerprint ?? await portableReplayFingerprint(validatePortableReplay(portable));
                  const active = localMatchSessionStore.getState().matchStore?.getState();
                  if (active?.status === "playing" && active.moves.length > 0
                    && !window.confirm("Replace your active game with a new game from this position?")) return;
                  navigate(localMatchResumePath, { state: { resumeSeed: { ...resumeSeed,
                    practiceOrigin: { sourceFingerprint, inheritedPly: resumeSeed.moves.length } } } });
                } catch (error) { setBranchError(error instanceof Error ? error.message : "The game could not start."); }
                finally { setBranching(false); }
              }}
              replayFloor={replayFloor}
              resumePending={branching}
              setAutoplaying={setAutoplaying}
              setMoveIndex={setMoveIndex}
              timelineAnalysis={timelineAnalysis}
              timelineStyle={timelineStyle}
            />
            {branchError && <section className={styles.deckSection}>
              <p className={styles.contextError} role="alert">{branchError}</p>
            </section>}
            <section className={`${styles.deckSection} ${styles.sharingSection}`}>
              <button className="uiAction uiActionSecondary" type="button" ref={shareTriggerRef}
                onClick={() => openSidebar("share")}><Icon name="share" /><span className="uiActionLabel">Share replay</span></button>
              {canPublish && <button className="uiAction uiActionSecondary" type="button" ref={publishTriggerRef}
                onClick={() => openSidebar("publish")}><Icon name="profile" /><span className="uiActionLabel">Publish replay</span></button>}
            </section>
            {source && fingerprint && <SaveReplayButton replay={source} fingerprint={fingerprint} />}
          </div>
          <ShareReplayPanel match={match} moveIndex={frame.moveIndex} source={source} open={sidebar === "share"} onClose={() => setSidebar("replay")} />
          <PublishReplayPanel match={match} source={source} moveIndex={frame.moveIndex} eligible={canPublish}
            open={sidebar === "publish"} onClose={() => setSidebar("replay")} />
        </aside>
      </section>
    </main>
  );
}

export function ReplayRoute() {
  const { matchId } = useParams<{ matchId: string }>();
  return matchId?.startsWith("portable-") ? <StoredReplayRoute fingerprint={matchId.slice(9)} /> : <PrivateReplayRoute matchId={matchId} />;
}

function PrivateReplayRoute({ matchId }: { matchId?: string }) {
  const { localDisplayName, match, replayMayStillLoad } = useReplayMatch(matchId);
  if (!match) return (
    <main className={styles.page}>
      <section className={`${styles.notFound} uiPanel`}>
        <h1 className={styles.title}>{replayMayStillLoad ? "Loading replay" : "Replay unavailable"}</h1>
        <p className={styles.notFoundText}>{replayMayStillLoad ? "Checking saved match history." : "This replay is no longer available."}</p>
        <Link className="uiAction uiActionSecondary" to="/profile">Back to Profile</Link>
      </section>
    </main>
  );
  return <ReplayViewer key={matchId} match={match} localDisplayName={localDisplayName} />;
}
