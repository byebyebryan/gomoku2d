import { useEffect, useRef } from "react";
import * as Phaser from "phaser";

import { BoardScene } from "../../board/board_scene";
import type { BoardAnalysisOverlay, BoardTouchControlMode } from "../../board/board_scene_logic";
import type { CellPosition, CellStone, MatchMove, MatchStatus } from "../../game/types";

import styles from "./Board.module.css";

export interface BoardProps {
  analysisOverlays: BoardAnalysisOverlay[];
  cells: CellStone[][];
  counterThreatMoves: CellPosition[];
  currentPlayer: 1 | 2;
  forbiddenMoves: CellPosition[];
  imminentThreatMoves: CellPosition[];
  interactive: boolean;
  lastMove: CellPosition | null;
  moves: MatchMove[];
  nextReplayMove: CellPosition | null;
  onAdvanceRound: () => void;
  onPlace: (row: number, col: number) => void;
  onTouchCandidateChange: (candidate: CellPosition | null, canPlace: boolean) => void;
  touchControlMode: BoardTouchControlMode;
  touchCandidateResetVersion: number;
  showSequenceNumbers: boolean;
  status: MatchStatus;
  threatMoves: CellPosition[];
  winningMoves: CellPosition[];
  winningCells: CellPosition[];
}

function fitBoardViewport(width: number, height: number): { width: number; height: number } {
  const safeWidth = Math.max(1, Math.floor(width));
  const safeHeight = Math.max(1, Math.floor(height));
  const boardAspect = 1;

  const widthFromHeight = Math.floor(safeHeight * boardAspect);

  if (widthFromHeight <= safeWidth) {
    return {
      width: Math.max(1, widthFromHeight),
      height: safeHeight,
    };
  }

  return {
    width: safeWidth,
    height: Math.max(1, Math.floor(safeWidth / boardAspect)),
  };
}

export function Board(props: BoardProps) {
  const frameRef = useRef<HTMLDivElement | null>(null);
  const hostRef = useRef<HTMLDivElement | null>(null);
  const gameRef = useRef<Phaser.Game | null>(null);
  const lastHostSizeRef = useRef<{ width: number; height: number } | null>(null);
  const sceneRef = useRef<BoardScene | null>(null);

  useEffect(() => {
    if (!frameRef.current || !hostRef.current) {
      return undefined;
    }

    const frame = frameRef.current;
    const host = hostRef.current;
    const scene = new BoardScene();
    sceneRef.current = scene;

    const syncHostBox = (): { width: number; height: number } => {
      const nextSize = fitBoardViewport(frame.clientWidth, frame.clientHeight);
      const width = `${nextSize.width}px`;
      const height = `${nextSize.height}px`;

      if (host.style.width !== width) {
        host.style.width = width;
      }
      if (host.style.height !== height) {
        host.style.height = height;
      }

      return nextSize;
    };

    const syncGameToHost = (force = false): void => {
      const game = gameRef.current;

      if (!game) {
        return;
      }

      const nextSize = {
        width: Math.max(1, host.clientWidth),
        height: Math.max(1, host.clientHeight),
      };
      const previousSize = lastHostSizeRef.current;
      const scaleMatches =
        game.scale.width === nextSize.width &&
        game.scale.height === nextSize.height;

      if (
        !force &&
        previousSize &&
        previousSize.width === nextSize.width &&
        previousSize.height === nextSize.height &&
        scaleMatches
      ) {
        return;
      }

      game.scale.resize(nextSize.width, nextSize.height);
      lastHostSizeRef.current = nextSize;
    };

    const size = syncHostBox();

    const game = new Phaser.Game({
      backgroundColor: "#111111",
      height: size.height,
      parent: hostRef.current,
      pixelArt: true,
      scale: {
        mode: Phaser.Scale.RESIZE,
      },
      scene: [scene],
      type: Phaser.AUTO,
      width: size.width,
    });

    gameRef.current = game;
    syncGameToHost(true);
    const frameSyncId = window.requestAnimationFrame(() => {
      syncHostBox();
      syncGameToHost(true);
    });
    const frameResizeObserver = new ResizeObserver(() => {
      syncHostBox();
      window.requestAnimationFrame(() => {
        syncGameToHost(true);
      });
    });
    const hostResizeObserver = new ResizeObserver(() => {
      syncGameToHost(true);
    });
    frameResizeObserver.observe(frame);
    hostResizeObserver.observe(host);

    return () => {
      window.cancelAnimationFrame(frameSyncId);
      frameResizeObserver.disconnect();
      hostResizeObserver.disconnect();
      game.destroy(true);
      gameRef.current = null;
      lastHostSizeRef.current = null;
      sceneRef.current = null;
    };
  }, []);

  useEffect(() => {
    sceneRef.current?.setBoardState(props);
  }, [props]);

  return (
    <div className={styles.frame} ref={frameRef}>
      <div className={styles.viewport} ref={hostRef} />
    </div>
  );
}
