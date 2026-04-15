import Phaser from "phaser";
import { BOARD_SIZE, SPRITE, FRAME_SIZE } from "./constants";

// Board colors sampled from the original sprite
const BOARD_COLOR = 0xffcc66;       // rgb(255,204,102) from grid sprite frame 0
const BOARD_SIDE_COLOR = 0x261e0f;  // dark brown side depth
const GRID_LINE_COLOR = 0x261e0f;   // dark brown grid lines

/**
 * Renders a 15x15 Gomoku board.
 * Board, side edge, and grid drawn as simple shapes.
 * Stones rendered as sprites on top.
 */
export class BoardRenderer {
  private scene: Phaser.Scene;
  private cellSize: number;
  private originX: number;
  private originY: number;

  constructor(scene: Phaser.Scene, cellSize: number, originX: number, originY: number) {
    this.scene = scene;
    this.cellSize = cellSize;
    this.originX = originX;
    this.originY = originY;
  }

  cellToPixel(row: number, col: number): { x: number; y: number } {
    return {
      x: this.originX + col * this.cellSize,
      y: this.originY + row * this.cellSize,
    };
  }

  pixelToCell(px: number, py: number): { row: number; col: number } | null {
    const col = Math.round((px - this.originX) / this.cellSize);
    const row = Math.round((py - this.originY) / this.cellSize);
    if (row < 0 || row >= BOARD_SIZE || col < 0 || col >= BOARD_SIZE) return null;
    return { row, col };
  }

  drawBoard(): void {
    const gfx = this.scene.add.graphics();
    gfx.setDepth(0);

    const top = this.originY - this.cellSize / 2;
    const left = this.originX - this.cellSize / 2;
    const boardWidth = BOARD_SIZE * this.cellSize;
    const boardHeight = BOARD_SIZE * this.cellSize;
    const sideHeight = this.cellSize / 2;

    // Side/depth (drawn first, below and behind)
    gfx.fillStyle(BOARD_SIDE_COLOR, 1);
    gfx.fillRect(left, top + boardHeight, boardWidth, sideHeight);

    // Board surface
    gfx.fillStyle(BOARD_COLOR, 1);
    gfx.fillRect(left, top, boardWidth, boardHeight);

    // Grid lines — only between intersection points, not extending to board edge
    // Line thickness = 1 sprite pixel scaled to screen
    const lineThickness = Math.max(1, Math.round(this.cellSize / FRAME_SIZE));
    gfx.lineStyle(lineThickness, GRID_LINE_COLOR, 1);

    // Horizontal lines
    for (let row = 0; row < BOARD_SIZE; row++) {
      const { x: x0, y } = this.cellToPixel(row, 0);
      const { x: x1 } = this.cellToPixel(row, BOARD_SIZE - 1);
      const x0i = Math.round(x0);
      const x1i = Math.round(x1);
      const yi = Math.round(y) + 0.5; // half-pixel for crisp horizontal lines
      gfx.beginPath();
      gfx.moveTo(x0i, yi);
      gfx.lineTo(x1i, yi);
      gfx.strokePath();
    }

    // Vertical lines
    for (let col = 0; col < BOARD_SIZE; col++) {
      const { x, y: y0 } = this.cellToPixel(0, col);
      const { y: y1 } = this.cellToPixel(BOARD_SIZE - 1, col);
      const xi = Math.round(x) + 0.5; // half-pixel for crisp vertical lines
      const y0i = Math.round(y0);
      const y1i = Math.round(y1);
      gfx.beginPath();
      gfx.moveTo(xi, y0i);
      gfx.lineTo(xi, y1i);
      gfx.strokePath();
    }
  }

  placeStone(row: number, col: number, color: 0 | 1): Phaser.GameObjects.Sprite {
    const { x, y } = this.cellToPixel(row, col);
    const scale = this.cellSize / FRAME_SIZE;

    const stone = this.scene.add.sprite(x, y, SPRITE.STONE, 0);
    stone.setScale(scale);
    stone.setDepth(1);

    if (color === 0) {
      stone.setTint(0x404040);
    } else {
      stone.setTint(0xffffff);
    }

    return stone;
  }
}
