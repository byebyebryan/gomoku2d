import Phaser from "phaser";
import { BOARD_SIZE, WIN_LENGTH, SPRITE, FRAME_SIZE, STONE_ANIMS, POINTER_ANIMS } from "./constants";

// Board colors sampled from the original sprite
const BOARD_COLOR = 0xffcc66;
const BOARD_SIDE_COLOR = 0x261e0f;
const GRID_LINE_COLOR = 0x261e0f;

type CellState = 0 | 1 | null; // null = empty, 0 = black, 1 = white

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

  getCellSize(): number {
    return this.cellSize;
  }

  drawBoard(): void {
    const gfx = this.scene.add.graphics();
    gfx.setDepth(0);

    const top = this.originY - this.cellSize / 2;
    const left = this.originX - this.cellSize / 2;
    const boardWidth = BOARD_SIZE * this.cellSize;
    const boardHeight = BOARD_SIZE * this.cellSize;
    const sideHeight = this.cellSize / 2;

    // Side/depth
    gfx.fillStyle(BOARD_SIDE_COLOR, 1);
    gfx.fillRect(left, top + boardHeight, boardWidth, sideHeight);

    // Board surface
    gfx.fillStyle(BOARD_COLOR, 1);
    gfx.fillRect(left, top, boardWidth, boardHeight);

    // Grid lines
    const lineThickness = Math.max(1, Math.round(this.cellSize / FRAME_SIZE));
    gfx.lineStyle(lineThickness, GRID_LINE_COLOR, 1);

    for (let row = 0; row < BOARD_SIZE; row++) {
      const { x: x0, y } = this.cellToPixel(row, 0);
      const { x: x1 } = this.cellToPixel(row, BOARD_SIZE - 1);
      gfx.beginPath();
      gfx.moveTo(Math.round(x0), Math.round(y) + 0.5);
      gfx.lineTo(Math.round(x1), Math.round(y) + 0.5);
      gfx.strokePath();
    }

    for (let col = 0; col < BOARD_SIZE; col++) {
      const { x, y: y0 } = this.cellToPixel(0, col);
      const { y: y1 } = this.cellToPixel(BOARD_SIZE - 1, col);
      gfx.beginPath();
      gfx.moveTo(Math.round(x) + 0.5, Math.round(y0));
      gfx.lineTo(Math.round(x) + 0.5, Math.round(y1));
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

  createPointer(): Phaser.GameObjects.Sprite {
    const scale = this.cellSize / FRAME_SIZE;
    const pointer = this.scene.add.sprite(-100, -100, SPRITE.POINTER, 0);
    pointer.setScale(scale);
    pointer.setDepth(2);
    pointer.setVisible(false);
    return pointer;
  }

  createInteractiveZones(onCellClick: (row: number, col: number) => void): Phaser.GameObjects.Zone[] {
    const zones: Phaser.GameObjects.Zone[] = [];

    for (let row = 0; row < BOARD_SIZE; row++) {
      for (let col = 0; col < BOARD_SIZE; col++) {
        const { x, y } = this.cellToPixel(row, col);
        const zone = this.scene.add.zone(x, y, this.cellSize, this.cellSize);
        zone.setInteractive({ useHandCursor: true });
        zone.setDepth(3);

        const r = row;
        const c = col;
        zone.on("pointerdown", () => onCellClick(r, c));

        zones.push(zone);
      }
    }

    return zones;
  }
}
