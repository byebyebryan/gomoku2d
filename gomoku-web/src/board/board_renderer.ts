import Phaser from "phaser";
import { BOARD_SIZE, WIN_LENGTH, SPRITE, FRAME_SIZE, STONE_ANIMS, POINTER_ANIMS, COLOR } from "./constants";

type CellState = 0 | 1 | null; // null = empty, 0 = black, 1 = white

export interface BoardBounds {
  left: number;
  top: number;
  width: number;
  height: number;
  sideHeight: number;
  right: number;
  bottom: number;
  centerX: number;
  centerY: number;
}

export class BoardRenderer {
  private scene: Phaser.Scene;
  private cellSize: number;
  private originX: number;
  private originY: number;
  private screenHeight: number;
  private parent: Phaser.GameObjects.Container | null;

  constructor(
    scene: Phaser.Scene,
    cellSize: number,
    originX: number,
    originY: number,
    screenHeight: number,
    parent?: Phaser.GameObjects.Container,
  ) {
    this.scene = scene;
    this.cellSize = cellSize;
    this.originX = originX;
    this.originY = originY;
    this.screenHeight = screenHeight;
    this.parent = parent ?? null;
  }

  private attach<T extends Phaser.GameObjects.GameObject>(
    gameObject: T,
    parentOverride?: Phaser.GameObjects.Container | null,
  ): T {
    const parent = parentOverride ?? this.parent;
    if (parent) {
      parent.add(gameObject);
    }

    return gameObject;
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

  getBounds(): BoardBounds {
    const top = this.originY - this.cellSize / 2;
    const left = this.originX - this.cellSize / 2;
    const width = BOARD_SIZE * this.cellSize;
    const surfaceHeight = BOARD_SIZE * this.cellSize;
    const maxSideHeight = this.cellSize / 2;
    const sideHeight = Math.min(maxSideHeight, this.screenHeight - (top + surfaceHeight));
    const height = surfaceHeight + sideHeight;

    return {
      left,
      top,
      width,
      height,
      sideHeight,
      right: left + width,
      bottom: top + height,
      centerX: left + width / 2,
      centerY: top + height / 2,
    };
  }

  drawBoard(showGrid: boolean = true): Phaser.GameObjects.Graphics {
    const gfx = this.scene.add.graphics();
    gfx.setDepth(0);

    const bounds = this.getBounds();
    const boardWidth = bounds.width;
    const boardHeight = BOARD_SIZE * this.cellSize;

    // Side/depth
    gfx.fillStyle(COLOR.BOARD_EDGE, 1);
    gfx.fillRect(bounds.left, bounds.top + boardHeight, boardWidth, bounds.sideHeight);

    // Board surface
    gfx.fillStyle(COLOR.BOARD_SURFACE, 1);
    gfx.fillRect(bounds.left, bounds.top, boardWidth, boardHeight);

    if (!showGrid) {
      return this.attach(gfx);
    }

    const lineThickness = Math.max(1, Math.round(this.cellSize / FRAME_SIZE));
    gfx.lineStyle(lineThickness, COLOR.GRID, 1);

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

    return this.attach(gfx);
  }

  placeStone(row: number, col: number, color: 0 | 1): Phaser.GameObjects.Sprite {
    const { x, y } = this.cellToPixel(row, col);
    const scale = this.cellSize / FRAME_SIZE;

    const stone = this.scene.add.sprite(x, y, SPRITE.STONE, 0);
    stone.setScale(scale);
    stone.setDepth(1);

    stone.setTint(color === 0 ? COLOR.STONE_BLACK : COLOR.STONE_WHITE);

    return this.attach(stone);
  }

  createPointer(parentOverride?: Phaser.GameObjects.Container): Phaser.GameObjects.Sprite {
    const scale = this.cellSize / FRAME_SIZE;
    const pointer = this.scene.add.sprite(-100, -100, SPRITE.POINTER, 0);
    pointer.setScale(scale);
    pointer.setDepth(2);
    pointer.setVisible(false);
    return this.attach(pointer, parentOverride);
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
