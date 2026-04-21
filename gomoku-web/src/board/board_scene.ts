import Phaser from "phaser";

import { BOARD_SIZE, COLOR, FRAME_SIZE, SPRITE } from "./constants";
import { BoardRenderer } from "./board_renderer";

import type { CellPosition, CellStone } from "../game/types";

const STONE_SPRITESHEET_URL = new URL("../../assets/sprites/stone.png", import.meta.url).toString();

export interface BoardSceneState {
  cells: CellStone[][];
  currentPlayer: 1 | 2;
  interactive: boolean;
  lastMove: CellPosition | null;
  onPlace: (row: number, col: number) => void;
  winningCells: CellPosition[];
}

const EDGE_RATIO = 1 / 3;
const DEFAULT_STATE: BoardSceneState = {
  cells: Array.from({ length: BOARD_SIZE }, () =>
    Array.from({ length: BOARD_SIZE }, () => null),
  ),
  currentPlayer: 1,
  interactive: false,
  lastMove: null,
  onPlace: () => undefined,
  winningCells: [],
};

export class BoardScene extends Phaser.Scene {
  private boardState: BoardSceneState = DEFAULT_STATE;
  private root: Phaser.GameObjects.Container | null = null;
  private board: BoardRenderer | null = null;
  private pointer: Phaser.GameObjects.Sprite | null = null;

  constructor() {
    super({ key: "BoardScene" });
  }

  preload(): void {
    if (!this.textures.exists(SPRITE.STONE)) {
      this.load.spritesheet(SPRITE.STONE, STONE_SPRITESHEET_URL, {
        frameHeight: FRAME_SIZE,
        frameWidth: FRAME_SIZE,
      });
    }
  }

  create(): void {
    this.scale.on(Phaser.Scale.Events.RESIZE, this.renderBoard, this);
    this.input.on("pointermove", this.handlePointerMove, this);
    this.input.on("pointerup", this.handlePointerUp, this);
    this.input.on("pointerout", this.hidePointer, this);
    this.renderBoard();
  }

  shutdown(): void {
    this.scale?.off?.(Phaser.Scale.Events.RESIZE, this.renderBoard, this);
    this.input?.off?.("pointermove", this.handlePointerMove, this);
    this.input?.off?.("pointerup", this.handlePointerUp, this);
    this.input?.off?.("pointerout", this.hidePointer, this);
    this.root?.destroy(true);
    this.root = null;
    this.pointer = null;
    this.board = null;
  }

  setBoardState(state: BoardSceneState): void {
    this.boardState = state;

    if (this.sys?.isActive()) {
      this.renderBoard();
    }
  }

  private handlePointerMove(pointer: Phaser.Input.Pointer): void {
    if (!this.boardState.interactive || !this.board || !this.pointer) {
      this.hidePointer();
      return;
    }

    const cell = this.board.pixelToCell(pointer.x, pointer.y);

    if (!cell || this.boardState.cells[cell.row][cell.col] !== null) {
      this.hidePointer();
      return;
    }

    const point = this.board.cellToPixel(cell.row, cell.col);
    this.pointer
      .setPosition(point.x, point.y)
      .setTint(this.boardState.currentPlayer === 1 ? COLOR.STONE_BLACK : COLOR.STONE_WHITE)
      .setVisible(true);
  }

  private handlePointerUp(pointer: Phaser.Input.Pointer): void {
    if (!this.boardState.interactive || !this.board) {
      return;
    }

    const cell = this.board.pixelToCell(pointer.x, pointer.y);

    if (!cell || this.boardState.cells[cell.row][cell.col] !== null) {
      return;
    }

    this.boardState.onPlace(cell.row, cell.col);
  }

  private hidePointer(): void {
    this.pointer?.setVisible(false);
  }

  private renderBoard(): void {
    this.root?.destroy(true);

    const width = this.cameras.main.width;
    const height = this.cameras.main.height;
    const cellSize = Math.min(width / BOARD_SIZE, height / (BOARD_SIZE + EDGE_RATIO));
    const boardHeight = BOARD_SIZE * cellSize + cellSize / 2;
    const originX = (width - (BOARD_SIZE - 1) * cellSize) / 2;
    const originY = (height - boardHeight) / 2 + cellSize / 2;

    this.root = this.add.container(0, 0);

    const boardLayer = this.add.container(0, 0);
    const overlayLayer = this.add.container(0, 0);
    this.root.add([boardLayer, overlayLayer]);

    this.board = new BoardRenderer(this, cellSize, originX, originY, height, boardLayer);
    this.board.drawBoard();

    for (let row = 0; row < BOARD_SIZE; row += 1) {
      for (let col = 0; col < BOARD_SIZE; col += 1) {
        const cell = this.boardState.cells[row][col];

        if (cell === null) {
          continue;
        }

        this.board.placeStone(row, col, cell);
      }
    }

    this.pointer = this.add.sprite(-1000, -1000, SPRITE.STONE, 0);
    this.pointer
      .setAlpha(0.35)
      .setDepth(2.5)
      .setScale(cellSize / FRAME_SIZE)
      .setVisible(false);
    overlayLayer.add(this.pointer);

    if (this.boardState.lastMove) {
      const point = this.board.cellToPixel(
        this.boardState.lastMove.row,
        this.boardState.lastMove.col,
      );
      const marker = this.add.graphics();
      marker.setDepth(2.25);
      marker.lineStyle(Math.max(2, Math.round(cellSize * 0.08)), COLOR.TITLE, 1);
      marker.strokeCircle(point.x, point.y, cellSize * 0.18);
      overlayLayer.add(marker);
    }

    if (this.boardState.winningCells.length > 0) {
      const highlight = this.add.graphics();
      highlight.setDepth(2.4);
      highlight.lineStyle(Math.max(2, Math.round(cellSize * 0.08)), COLOR.WIN_CELLS, 1);

      for (const cell of this.boardState.winningCells) {
        const point = this.board.cellToPixel(cell.row, cell.col);
        highlight.strokeRect(
          point.x - cellSize * 0.32,
          point.y - cellSize * 0.32,
          cellSize * 0.64,
          cellSize * 0.64,
        );
      }

      overlayLayer.add(highlight);
    }
  }
}
