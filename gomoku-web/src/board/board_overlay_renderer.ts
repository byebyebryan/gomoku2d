import * as Phaser from "phaser";

import { BoardRenderer } from "./board_renderer";
import {
  BOARD_RENDER_DEPTHS,
  COLOR,
  HIGHLIGHTER_ANIMS,
  HOVER_ANIMS,
  SPRITE,
} from "./constants";
import { SEQUENCE_FONT_FAMILY } from "./sequence_font";
import {
  analysisHighlightAnimationForRole,
  analysisHighlightTintForRole,
  analysisMarkerAnimationForRole,
  analysisMarkerSpriteForRole,
  analysisMarkerTintForRole,
  overlayAnimationForRole,
  overlaySpriteForRole,
  sequenceNumberFontSize,
  sequenceNumberPosition,
  shouldRenderStandaloneForbiddenOverlay,
} from "./board_scene_logic";
import type { BoardSceneState } from "./board_scene";

export interface BoardOverlayRenderResult {
  analysisSprites: Phaser.GameObjects.Sprite[];
  forbiddenSprites: Phaser.GameObjects.Sprite[];
  hintSprites: Phaser.GameObjects.Sprite[];
  sequenceLabels: Phaser.GameObjects.Text[];
  winSprites: Phaser.GameObjects.Sprite[];
}

export interface BoardOverlayRenderContext {
  board: BoardRenderer;
  cellSize: number;
  createForbiddenSprite: (x: number, y: number) => Phaser.GameObjects.Sprite;
  createOverlaySprite: (
    x: number,
    y: number,
    tint: number,
    animKey: string,
    depth: number,
    texture: string,
  ) => Phaser.GameObjects.Sprite;
  scene: Phaser.Scene;
  sequenceLayer: Phaser.GameObjects.Container;
  state: BoardSceneState;
}

export function renderBoardOverlays(context: BoardOverlayRenderContext): BoardOverlayRenderResult {
  const analysisSprites: Phaser.GameObjects.Sprite[] = [];
  const forbiddenSprites: Phaser.GameObjects.Sprite[] = [];
  const hintSprites: Phaser.GameObjects.Sprite[] = [];
  const sequenceLabels: Phaser.GameObjects.Text[] = [];
  const winSprites: Phaser.GameObjects.Sprite[] = [];
  const forbiddenKeys = new Set(
    context.state.forbiddenMoves.map((cell) => cellKey(cell.row, cell.col)),
  );

  for (const cell of context.state.forbiddenMoves) {
    if (!shouldRenderStandaloneForbiddenOverlay(cell, context.state.threatMoves)) {
      continue;
    }

    const point = context.board.cellToPixel(cell.row, cell.col);
    forbiddenSprites.push(context.createForbiddenSprite(point.x, point.y));
  }

  renderHintEvidenceCells(context, hintSprites, context.state.winningEvidenceCells, COLOR.WIN_MOVE);
  renderHintEvidenceCells(context, hintSprites, context.state.immediateThreatEvidenceCells, COLOR.THREAT);
  renderHintEvidenceCells(context, hintSprites, context.state.imminentThreatEvidenceCells, COLOR.IMMINENT_THREAT);
  renderHintEvidenceCells(context, hintSprites, context.state.counterThreatEvidenceCells, COLOR.COUNTER_THREAT);

  for (const cell of context.state.winningMoves) {
    const point = context.board.cellToPixel(cell.row, cell.col);
    hintSprites.push(
      context.createOverlaySprite(
        point.x,
        point.y,
        COLOR.WIN_MOVE,
        overlayAnimationForRole("winningMove"),
        BOARD_RENDER_DEPTHS.OVERLAY_SURFACE,
        overlaySpriteForRole("winningMove"),
      ),
    );
  }

  for (const cell of context.state.threatMoves) {
    const point = context.board.cellToPixel(cell.row, cell.col);
    const isForbidden = forbiddenKeys.has(cellKey(cell.row, cell.col));
    hintSprites.push(
      context.createOverlaySprite(
        point.x,
        point.y,
        COLOR.THREAT,
        overlayAnimationForRole("threatMove", isForbidden),
        BOARD_RENDER_DEPTHS.OVERLAY_SURFACE,
        overlaySpriteForRole("threatMove", isForbidden),
      ),
    );
  }

  for (const cell of context.state.imminentThreatMoves) {
    const point = context.board.cellToPixel(cell.row, cell.col);
    hintSprites.push(
      context.createOverlaySprite(
        point.x,
        point.y,
        COLOR.IMMINENT_THREAT,
        overlayAnimationForRole("imminentThreatMove"),
        BOARD_RENDER_DEPTHS.OVERLAY_SURFACE,
        overlaySpriteForRole("imminentThreatMove"),
      ),
    );
  }

  for (const cell of context.state.counterThreatMoves) {
    const point = context.board.cellToPixel(cell.row, cell.col);
    hintSprites.push(
      context.createOverlaySprite(
        point.x,
        point.y,
        COLOR.COUNTER_THREAT,
        overlayAnimationForRole("counterThreatMove"),
        BOARD_RENDER_DEPTHS.OVERLAY_SURFACE,
        overlaySpriteForRole("counterThreatMove"),
      ),
    );
  }

  for (const overlay of context.state.analysisOverlays) {
    if (!overlay.highlight) {
      continue;
    }

    const point = context.board.cellToPixel(overlay.row, overlay.col);
    analysisSprites.push(
      context.createOverlaySprite(
        point.x,
        point.y,
        analysisHighlightTintForRole(overlay.highlight, overlay.side),
        analysisHighlightAnimationForRole(overlay.highlight),
        BOARD_RENDER_DEPTHS.OVERLAY_SURFACE,
        SPRITE.HIGHLIGHTER,
      ),
    );
  }

  for (const overlay of context.state.analysisOverlays) {
    if (!overlay.marker) {
      continue;
    }

    const point = context.board.cellToPixel(overlay.row, overlay.col);
    analysisSprites.push(
      context.createOverlaySprite(
        point.x,
        point.y,
        analysisMarkerTintForRole(overlay.marker),
        analysisMarkerAnimationForRole(overlay.marker),
        BOARD_RENDER_DEPTHS.OVERLAY_MARKER,
        analysisMarkerSpriteForRole(overlay.marker),
      ),
    );
  }

  if (context.state.nextReplayMove !== null) {
    const point = context.board.cellToPixel(
      context.state.nextReplayMove.row,
      context.state.nextReplayMove.col,
    );
    analysisSprites.push(
      context.createOverlaySprite(
        point.x,
        point.y,
        context.state.currentPlayer === 1 ? COLOR.STONE_BLACK : COLOR.STONE_WHITE,
        HOVER_ANIMS.HOVER.key,
        BOARD_RENDER_DEPTHS.OVERLAY_HOVER,
        SPRITE.HOVER,
      ),
    );
  }

  if (context.state.showSequenceNumbers && context.state.status !== "playing") {
    for (const move of context.state.moves) {
      const cell = context.state.cells[move.row][move.col];
      if (cell === null) {
        continue;
      }

      const point = context.board.cellToPixel(move.row, move.col);
      const position = sequenceNumberPosition(point.x, point.y);
      const label = context.scene.add.text(position.x, position.y, String(move.moveNumber), {
        color: cssColor(cell === 0 ? COLOR.SEQ_ON_BLACK : COLOR.SEQ_ON_WHITE),
        fontFamily: SEQUENCE_FONT_FAMILY,
        fontSize: `${sequenceNumberFontSize(context.cellSize)}px`,
      });
      label.setOrigin(0.5, 0.5);
      label.setDepth(BOARD_RENDER_DEPTHS.SEQUENCE_NUMBER);
      context.sequenceLayer.add(label);
      sequenceLabels.push(label);
    }
  }

  for (const cell of context.state.winningCells) {
    const point = context.board.cellToPixel(cell.row, cell.col);
    winSprites.push(
      context.createOverlaySprite(
        point.x,
        point.y,
        COLOR.WIN_CELLS,
        overlayAnimationForRole("winningLine"),
        BOARD_RENDER_DEPTHS.OVERLAY_SURFACE,
        overlaySpriteForRole("winningLine"),
      ),
    );
  }

  return {
    analysisSprites,
    forbiddenSprites,
    hintSprites,
    sequenceLabels,
    winSprites,
  };
}

function renderHintEvidenceCells(
  context: BoardOverlayRenderContext,
  hintSprites: Phaser.GameObjects.Sprite[],
  cells: BoardSceneState["winningEvidenceCells"],
  tint: number,
): void {
  for (const cell of cells) {
    const point = context.board.cellToPixel(cell.row, cell.col);
    hintSprites.push(
      context.createOverlaySprite(
        point.x,
        point.y,
        tint,
        HIGHLIGHTER_ANIMS.SOFT.key,
        BOARD_RENDER_DEPTHS.OVERLAY_SURFACE,
        SPRITE.HIGHLIGHTER,
      ),
    );
  }
}

function cssColor(color: number): string {
  return `#${color.toString(16).padStart(6, "0")}`;
}

function cellKey(row: number, col: number): string {
  return `${row},${col}`;
}
