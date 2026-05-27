import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useStore } from "zustand";

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

import styles from "./ReplayRoute.module.css";
import {
  ReplayBoardPanel,
  ReplayMatchPanel,
  ReplayPlaybackPanel,
  ReplayStatusPanel,
} from "./ReplayRoutePanels";

const AUTOPLAY_DELAY_MS = 700;

export function ReplayRoute() {
  const { matchId } = useParams<{ matchId: string }>();
  const navigate = useNavigate();
  const { localDisplayName, match, replayFloor, replayMayStillLoad } = useReplayMatch(matchId);
  const settings = useStore(localProfileStore, (state) => state.settings);
  const [moveIndex, setMoveIndex] = useState(defaultReplayMoveIndex(0));
  const [autoplaying, setAutoplaying] = useState(false);
  const { analysisAnnotations, analysisStep } = useReplayAnalysis(match);
  const coreWinningCells = useReplayWinningCells(match, moveIndex);

  useEffect(() => {
    document.title = "Gomoku2D Replay Analysis";
  }, []);

  useEffect(() => {
    setMoveIndex(defaultReplayMoveIndex(match?.move_count ?? 0));
    setAutoplaying(false);
  }, [match?.move_count, matchId]);

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

  if (!match && replayMayStillLoad) {
    return (
      <main className={styles.page}>
        <section className={`${styles.notFound} uiPanel`}>
          <h1 className={styles.title}>Loading replay</h1>
          <p className={styles.notFoundText}>Checking saved match history.</p>
        </section>
      </main>
    );
  }

  if (!match) {
    return (
      <main className={styles.page}>
        <section className={`${styles.notFound} uiPanel`}>
          <h1 className={styles.title}>Replay unavailable</h1>
          <p className={styles.notFoundText}>This replay is no longer available.</p>
          <Link className="uiAction uiActionSecondary" to="/profile">
            <Icon className="uiIconDesktop" name="profile" />
            <span className="uiActionLabel">Back to Profile</span>
          </Link>
        </section>
      </main>
    );
  }

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
          <p className="uiPageEyebrow">Saved match</p>
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

          <ReplayMatchPanel frame={frame} localDisplayName={localDisplayName} match={match} />

          <div className="uiDivider" />

          <ReplayPlaybackPanel
            autoplaying={autoplaying}
            frame={frame}
            match={match}
            onResume={(resumeSeed) => {
              navigate("/match/local", { state: { resumeSeed } });
            }}
            replayFloor={replayFloor}
            setAutoplaying={setAutoplaying}
            setMoveIndex={setMoveIndex}
            timelineAnalysis={timelineAnalysis}
            timelineStyle={timelineStyle}
          />
        </aside>
      </section>
    </main>
  );
}
