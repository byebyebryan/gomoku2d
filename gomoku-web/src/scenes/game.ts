import Phaser from "phaser";
import { BOARD_SIZE, FRAME_SIZE } from "../board/constants";
import { BoardRenderer } from "../board/board_renderer";

export class GameScene extends Phaser.Scene {
  private board!: BoardRenderer;
  private cellSize: number = 0;

  constructor() {
    super({ key: "GameScene" });
  }

  create(): void {
    const width = this.cameras.main.width;
    const height = this.cameras.main.height;

    // Calculate cell size to fit the board with padding
    const padding = 80;
    const available = Math.min(width, height) - padding * 2;
    this.cellSize = Math.floor(available / (BOARD_SIZE - 1));

    // Board origin: center of camera, offset so cell [7,7] is at center
    const originX = Math.floor(width / 2) - Math.floor(((BOARD_SIZE - 1) * this.cellSize) / 2);
    const originY = Math.floor(height / 2) - Math.floor(((BOARD_SIZE - 1) * this.cellSize) / 2);

    this.board = new BoardRenderer(this, this.cellSize, originX, originY);

    // Draw the board (surface, side edge, grid lines)
    this.board.drawBoard();

    // Place some test stones to verify rendering
    // Center: black
    this.board.placeStone(7, 7, 0);
    // Neighbours: white
    this.board.placeStone(7, 8, 1);
    this.board.placeStone(6, 7, 1);
    // Corner stones
    this.board.placeStone(0, 0, 0);
    this.board.placeStone(0, 14, 1);
    this.board.placeStone(14, 0, 1);
    this.board.placeStone(14, 14, 0);
    // A small sequence
    this.board.placeStone(3, 3, 0);
    this.board.placeStone(3, 4, 0);
    this.board.placeStone(3, 5, 0);
    this.board.placeStone(3, 6, 0);
    this.board.placeStone(3, 7, 0);

    // Turn indicator
    this.add
      .text(width / 2, 20, "Phase B — Board Renderer", {
        fontFamily: "minecraft",
        fontSize: "16px",
        color: "#7fffaa",
      })
      .setOrigin(0.5);
  }
}
