import { useEffect, useState } from "react";
import { Link, useNavigate, useParams, useSearchParams } from "react-router-dom";
import { useStore } from "zustand";

import { useDocumentTitle } from "../app/useDocumentTitle";
import { buildReplayBoardModel } from "../board/board_model";
import { localProfileStore } from "../profile/local_profile_store";
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

import styles from "./ReplayRoute.module.css";
import {
  ReplayBoardPanel,
  ReplayMatchPanel,
  ReplayPlaybackPanel,
  ReplayStatusPanel,
} from "./ReplayRoutePanels";

const AUTOPLAY_DELAY_MS = 700;

export function ReplayViewer({ match, localDisplayName, preserveNames = false, initialMoveIndex, source }: {
  match: SavedMatchV2;
  localDisplayName: string;
  preserveNames?: boolean;
  initialMoveIndex?: number;
  source?: PortableReplayV1;
}) {
  useDocumentTitle("Replay Analysis");
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const replayFloor = match.undo_floor;
  const settings = useStore(localProfileStore, (state) => state.settings);
  const [moveIndex, setMoveIndex] = useState(defaultReplayMoveIndex(0));
  const [autoplaying, setAutoplaying] = useState(false);
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
          <p className="uiPageEyebrow">{preserveNames ? "Shared replay" : "Saved match"}</p>
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

      <section className={styles.layout}>
        <ReplayBoardPanel model={boardModel} />

        <aside className={styles.deck}>
          <ReplayStatusPanel detail={analysisStatus.detail} label={analysisStatus.label} />

          <div className="uiDivider" />

          <ReplayMatchPanel frame={frame} localDisplayName={localDisplayName} match={match} preserveNames={preserveNames} />

          <div className="uiDivider" />

          <ReplayPlaybackPanel
            autoplaying={autoplaying}
            frame={frame}
            match={match}
            onResume={(resumeSeed) => {
              navigate(localMatchResumePath, { state: { resumeSeed } });
            }}
            replayFloor={replayFloor}
            setAutoplaying={setAutoplaying}
            setMoveIndex={setMoveIndex}
            timelineAnalysis={timelineAnalysis}
            timelineStyle={timelineStyle}
          />
          <ShareReplayPanel match={match} moveIndex={frame.moveIndex} source={source} />
        </aside>
      </section>
    </main>
  );
}

export function ReplayRoute() {
  const { matchId } = useParams<{ matchId: string }>();
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
