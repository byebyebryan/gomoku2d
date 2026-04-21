import { useEffect, useRef } from "react";
import Phaser from "phaser";

import { BoardScene } from "../../board/board_scene";
import { getGameSizeForViewport, getViewportSize } from "../../layout";
import type { CellPosition, CellStone, MatchMove, MatchStatus } from "../../game/types";

import styles from "./Board.module.css";

export interface BoardProps {
  cells: CellStone[][];
  currentPlayer: 1 | 2;
  forbiddenMoves: CellPosition[];
  interactive: boolean;
  lastMove: CellPosition | null;
  moves: MatchMove[];
  onAdvanceRound: () => void;
  onPlace: (row: number, col: number) => void;
  status: MatchStatus;
  threatMoves: CellPosition[];
  winningMoves: CellPosition[];
  winningCells: CellPosition[];
}

export function Board(props: BoardProps) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const gameRef = useRef<Phaser.Game | null>(null);
  const sceneRef = useRef<BoardScene | null>(null);

  useEffect(() => {
    if (!hostRef.current) {
      return undefined;
    }

    const scene = new BoardScene();
    sceneRef.current = scene;

    const viewport = getViewportSize();
    const size = getGameSizeForViewport(viewport.width, viewport.height);

    const game = new Phaser.Game({
      backgroundColor: "#111111",
      height: size.height,
      parent: hostRef.current,
      pixelArt: true,
      scale: {
        autoCenter: Phaser.Scale.Center.CENTER_BOTH,
        mode: Phaser.Scale.FIT,
      },
      scene: [scene],
      type: Phaser.AUTO,
      width: size.width,
    });

    gameRef.current = game;

    const syncGameSizeToViewport = (): void => {
      const nextViewport = getViewportSize();
      const nextSize = getGameSizeForViewport(nextViewport.width, nextViewport.height);

      if (
        game.scale.width === nextSize.width &&
        game.scale.height === nextSize.height
      ) {
        return;
      }

      game.scale.setGameSize(nextSize.width, nextSize.height);
    };

    window.addEventListener("resize", syncGameSizeToViewport);
    window.visualViewport?.addEventListener("resize", syncGameSizeToViewport);

    return () => {
      window.removeEventListener("resize", syncGameSizeToViewport);
      window.visualViewport?.removeEventListener("resize", syncGameSizeToViewport);
      game.destroy(true);
      gameRef.current = null;
      sceneRef.current = null;
    };
  }, []);

  useEffect(() => {
    sceneRef.current?.setBoardState(props);
  }, [props]);

  return (
    <div className={styles.frame}>
      <div className={styles.viewport} ref={hostRef} />
    </div>
  );
}
