import { useEffect, useId, useRef, useState } from "react";
import * as Phaser from "phaser";

import { BoardScene, type BoardSceneState } from "../../board/board_scene";
import type { BoardOverlay, BoardViewModel } from "../../board/board_model";

import styles from "./Board.module.css";

export interface BoardProps {
  // Teaching diagrams provide their own text alternative on the parent image.
  decorative?: boolean;
  model: BoardViewModel;
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
  const instructionsId = useId();
  const [focusedCell, setFocusedCell] = useState(() => Math.floor(props.model.boardSize / 2) * (props.model.boardSize + 1));
  const cellRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const model = props.model;
  const interactive = model.interaction.kind === "play" && model.interaction.interactive;
  const sideToMove = model.position.currentPlayer === 1 ? "Black" : "White";
  const positionStatus = model.position.status === "playing"
    ? `${sideToMove} to move.`
    : model.position.status === "draw" ? "Draw." : model.position.status === "black_won" ? "Black wins." : "White wins.";

  useEffect(() => {
    if (!frameRef.current || !hostRef.current) {
      return undefined;
    }

    const frame = frameRef.current;
    const host = hostRef.current;
    const scene = new BoardScene();
    sceneRef.current = scene;
    const motionQuery = typeof window.matchMedia === "function"
      ? window.matchMedia("(prefers-reduced-motion: reduce)") : null;
    const syncMotion = () => scene.setReducedMotion(motionQuery?.matches ?? false);
    syncMotion();
    motionQuery?.addEventListener("change", syncMotion);

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
      motionQuery?.removeEventListener("change", syncMotion);
      game.destroy(true);
      gameRef.current = null;
      lastHostSizeRef.current = null;
      sceneRef.current = null;
    };
  }, []);

  useEffect(() => {
    sceneRef.current?.setBoardState(boardSceneStateFromModel(props.model));
  }, [props]);

  return (
    <div className={styles.frame} ref={frameRef}>
      <div aria-hidden={props.decorative || undefined} className={styles.viewport} inert={props.decorative || undefined} ref={hostRef}>
        <span className={styles.screenReaderText} id={instructionsId}>
          Use arrow keys to explore the board. Home and End move to the ends of a row.
          {model.interaction.kind === "play"
            ? model.position.status === "playing"
              ? " Enter or Space places a stone on your turn."
              : " Game finished. Choose Analyze to inspect it or Next round to play again."
            : " This board is read only."}
        </span>
        <span className={styles.screenReaderText} role="status" aria-live="polite" aria-atomic="true">
          {positionStatus} {model.position.lastMove
            ? `Last move ${cellNotation(model.position.lastMove.row, model.position.lastMove.col)}.`
            : "Board empty."}
        </span>
        <div
          aria-colcount={model.boardSize}
          aria-describedby={instructionsId}
          aria-label="Gomoku board"
          aria-readonly={!interactive}
          aria-rowcount={model.boardSize}
          className={styles.keyboardGrid}
          role="grid"
          style={{ gridTemplateColumns: `repeat(${model.boardSize}, 1fr)`, gridTemplateRows: `repeat(${model.boardSize}, 1fr)` }}
        >
          {model.position.cells.map((row, rowIndex) => (
            <div className={styles.keyboardRow} key={rowIndex} role="row" aria-rowindex={rowIndex + 1}>
              {row.map((stone, colIndex) => {
                const index = rowIndex * model.boardSize + colIndex;
                const forbidden = model.forbiddenMoves.some((cell) => cell.row === rowIndex && cell.col === colIndex);
                const placeable = interactive && stone === null && !forbidden;
                const annotations = model.overlays
                  .filter((overlay) => overlay.cell.row === rowIndex && overlay.cell.col === colIndex)
                  .map(overlayDescription)
                  .filter(Boolean);
                return (
                  <button
                    aria-colindex={colIndex + 1}
                    aria-disabled={!placeable}
                    aria-label={`${cellNotation(rowIndex, colIndex)}: ${stone === null ? "empty" : stone === 0 ? "Black stone" : "White stone"}${forbidden ? ", forbidden" : ""}${annotations.length ? `, ${annotations.join(", ")}` : ""}`}
                    className={styles.keyboardCell}
                    key={colIndex}
                    onClick={() => {
                      if (placeable && model.interaction.kind === "play") model.interaction.onPlace(rowIndex, colIndex);
                    }}
                    onFocus={() => setFocusedCell(index)}
                    onKeyDown={(event) => {
                      let nextRow = rowIndex;
                      let nextCol = colIndex;
                      switch (event.key) {
                        case "ArrowUp": nextRow -= 1; break;
                        case "ArrowDown": nextRow += 1; break;
                        case "ArrowLeft": nextCol -= 1; break;
                        case "ArrowRight": nextCol += 1; break;
                        case "Home": nextCol = 0; if (event.ctrlKey) nextRow = 0; break;
                        case "End": nextCol = model.boardSize - 1; if (event.ctrlKey) nextRow = model.boardSize - 1; break;
                        default: return;
                      }
                      event.preventDefault();
                      nextRow = Math.max(0, Math.min(model.boardSize - 1, nextRow));
                      nextCol = Math.max(0, Math.min(model.boardSize - 1, nextCol));
                      cellRefs.current[nextRow * model.boardSize + nextCol]?.focus();
                    }}
                    ref={(element) => { cellRefs.current[index] = element; }}
                    role="gridcell"
                    tabIndex={focusedCell === index ? 0 : -1}
                    type="button"
                  />
                );
              })}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function cellNotation(row: number, col: number): string {
  return `${String.fromCharCode(65 + col)}${row + 1}`;
}

function overlayDescription(overlay: BoardOverlay): string {
  if (overlay.kind === "analysis") {
    if (overlay.marker === "escape") return "confirmed escape";
    return (overlay.marker ?? overlay.highlight ?? "").replace(/([a-z])([A-Z])/g, "$1 $2").toLowerCase();
  }
  if (overlay.kind === "hint" || overlay.kind === "evidence") {
    const label = {
      winning: "winning move",
      immediateThreat: "must-block threat",
      imminentThreat: "forcing-three response",
      counterThreat: "counter threat",
    }[overlay.role];
    return overlay.kind === "evidence" ? `${label} evidence` : label;
  }
  if (overlay.kind === "nextReplayMove") return "next replay move";
  if (overlay.kind === "winningLine") return "winning line";
  return "";
}

function boardSceneStateFromModel(model: BoardViewModel): BoardSceneState {
  const state: BoardSceneState = {
    analysisOverlays: [],
    boardSize: model.boardSize,
    cells: model.position.cells,
    counterThreatEvidenceCells: [],
    counterThreatMoves: [],
    currentPlayer: model.position.currentPlayer,
    forbiddenMoves: model.forbiddenMoves,
    focusStones: [],
    immediateThreatEvidenceCells: [],
    imminentThreatEvidenceCells: [],
    imminentThreatMoves: [],
    interactive: model.interaction.kind === "play" && model.interaction.interactive,
    lastMove: model.position.lastMove,
    moves: model.position.moves,
    nextReplayMove: null,
    onAdvanceRound: () => undefined,
    onPlace: () => undefined,
    onTouchCandidateChange: () => undefined,
    touchControlMode: "none",
    touchCandidateResetVersion: 0,
    showSequenceNumbers: model.position.showSequenceNumbers,
    status: model.position.status,
    threatMoves: [],
    winningEvidenceCells: [],
    winningMoves: [],
    winningCells: [],
  };

  if (model.interaction.kind === "play") {
    state.onAdvanceRound = model.interaction.onAdvanceRound;
    state.onPlace = model.interaction.onPlace;
    state.onTouchCandidateChange = model.interaction.onTouchCandidateChange;
    state.touchControlMode = model.interaction.touchControlMode;
    state.touchCandidateResetVersion = model.interaction.touchCandidateResetVersion;
  }

  for (const overlay of model.overlays) {
    switch (overlay.kind) {
      case "analysis":
        state.analysisOverlays.push({
          col: overlay.cell.col,
          highlight: overlay.highlight,
          marker: overlay.marker,
          row: overlay.cell.row,
          side: overlay.side,
        });
        break;
      case "evidence":
        if (overlay.role === "winning") {
          state.winningEvidenceCells.push(overlay.cell);
        } else if (overlay.role === "immediateThreat") {
          state.immediateThreatEvidenceCells.push(overlay.cell);
        } else if (overlay.role === "imminentThreat") {
          state.imminentThreatEvidenceCells.push(overlay.cell);
        } else {
          state.counterThreatEvidenceCells.push(overlay.cell);
        }
        break;
      case "focusStone":
        state.focusStones.push({
          col: overlay.cell.col,
          row: overlay.cell.row,
          side: overlay.side,
        });
        break;
      case "hint":
        if (overlay.role === "winning") {
          state.winningMoves.push(overlay.cell);
        } else if (overlay.role === "immediateThreat") {
          state.threatMoves.push(overlay.cell);
        } else if (overlay.role === "imminentThreat") {
          state.imminentThreatMoves.push(overlay.cell);
        } else {
          state.counterThreatMoves.push(overlay.cell);
        }
        break;
      case "nextReplayMove":
        state.nextReplayMove = overlay.cell;
        break;
      case "winningLine":
        state.winningCells.push(overlay.cell);
        break;
    }
  }

  return state;
}
