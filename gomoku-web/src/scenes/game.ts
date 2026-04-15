import Phaser from "phaser";
import { BOARD_SIZE, WIN_LENGTH, SPRITE, FRAME_SIZE, STONE_ANIMS, WARNING_ANIMS } from "../board/constants";
import { BoardRenderer } from "../board/board_renderer";

type CellState = 0 | 1 | null;

interface GameResult {
  winner: 0 | 1 | null; // null = draw
  winningCells?: { row: number; col: number }[];
}

export class GameScene extends Phaser.Scene {
  private board!: BoardRenderer;
  private cellSize: number = 0;

  // Game state
  private grid: CellState[][] = [];
  private currentPlayer: 0 | 1 = 0; // 0 = black, 1 = white
  private moveCount: number = 0;
  private gameOver: boolean = false;
  private stoneSprites: Map<string, Phaser.GameObjects.Sprite> = new Map();

  // UI
  private pointer!: Phaser.GameObjects.Sprite;
  private turnText!: Phaser.GameObjects.Text;
  private resultText!: Phaser.GameObjects.Text;
  private zones: Phaser.GameObjects.Zone[] = [];

  constructor() {
    super({ key: "GameScene" });
  }

  create(): void {
    this.initGame();
  }

  private initGame(): void {
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

    // Draw the board
    this.board.drawBoard();

    // Init game state
    this.grid = Array.from({ length: BOARD_SIZE }, () => Array(BOARD_SIZE).fill(null));
    this.currentPlayer = 0;
    this.moveCount = 0;
    this.gameOver = false;
    this.stoneSprites.clear();

    // Create pointer sprite (tinted to current player's color)
    this.pointer = this.board.createPointer();
    this.updatePointerTint();

    // Create interactive zones for each cell
    this.zones = this.board.createInteractiveZones((row, col) => this.onCellClick(row, col));

    // Turn indicator
    this.turnText = this.add
      .text(width / 2, 20, "Black's turn", {
        fontFamily: "minecraft",
        fontSize: "16px",
        color: "#ffffff",
      })
      .setOrigin(0.5)
      .setDepth(10);

    // Result text (hidden initially)
    this.resultText = this.add
      .text(width / 2, height - 30, "", {
        fontFamily: "minecraft",
        fontSize: "14px",
        color: "#7fffaa",
      })
      .setOrigin(0.5)
      .setDepth(10);

    // Pointer follows mouse
    this.input.on("pointermove", (pointer: Phaser.Input.Pointer) => {
      if (this.gameOver) {
        this.pointer.setVisible(false);
        return;
      }
      const cell = this.board.pixelToCell(pointer.x, pointer.y);
      if (cell && this.grid[cell.row][cell.col] === null) {
        const { x, y } = this.board.cellToPixel(cell.row, cell.col);
        this.pointer.setPosition(x, y);
        this.pointer.setVisible(true);
      } else {
        this.pointer.setVisible(false);
      }
    });

    // Mouse leave canvas
    this.input.on("pointerout", () => {
      this.pointer.setVisible(false);
    });
  }

  private updatePointerTint(): void {
    this.pointer.setTint(this.currentPlayer === 0 ? 0x404040 : 0xffffff);
  }

  private cellKey(row: number, col: number): string {
    return `${row},${col}`;
  }

  private onCellClick(row: number, col: number): void {
    if (this.gameOver) {
      // Click after game over = reset
      this.resetGame();
      return;
    }

    if (this.grid[row][col] !== null) return;

    // Hide pointer immediately — don't reappear until mouse moves to new cell
    this.pointer.setVisible(false);

    // Place stone
    this.grid[row][col] = this.currentPlayer;
    this.moveCount++;

    const stone = this.board.placeStone(row, col, this.currentPlayer);
    this.stoneSprites.set(this.cellKey(row, col), stone);

    // Play stone-form animation
    stone.play(STONE_ANIMS.FORM.key);

    // Check win
    const result = this.checkWin(row, col, this.currentPlayer);
    if (result) {
      this.gameOver = true;
      this.pointer.setVisible(false);

      // Highlight winning stones
      if (result.winningCells) {
        this.highlightWin(result.winningCells);
      }

      const winnerName = result.winner === 0 ? "Black" : "White";
      this.turnText.setText(`${winnerName} wins!`);
      this.resultText.setText("Click anywhere to start a new game");
      return;
    }

    // Check draw
    if (this.moveCount >= BOARD_SIZE * BOARD_SIZE) {
      this.gameOver = true;
      this.pointer.setVisible(false);
      this.turnText.setText("Draw!");
      this.resultText.setText("Click anywhere to start a new game");
      return;
    }

    // Switch player
    this.currentPlayer = this.currentPlayer === 0 ? 1 : 0;
    this.turnText.setText(this.currentPlayer === 0 ? "Black's turn" : "White's turn");
    this.updatePointerTint();
  }

  private checkWin(row: number, col: number, player: CellState): GameResult | null {
    // 4 directions: horizontal, vertical, diagonal /, diagonal \
    const directions = [
      { dr: 0, dc: 1 },  // horizontal
      { dr: 1, dc: 0 },  // vertical
      { dr: 1, dc: 1 },  // diagonal \
      { dr: 1, dc: -1 }, // diagonal /
    ];

    for (const { dr, dc } of directions) {
      const cells: { row: number; col: number }[] = [{ row, col }];

      // Count forward
      for (let i = 1; i < WIN_LENGTH; i++) {
        const r = row + dr * i;
        const c = col + dc * i;
        if (r < 0 || r >= BOARD_SIZE || c < 0 || c >= BOARD_SIZE) break;
        if (this.grid[r][c] !== player) break;
        cells.push({ row: r, col: c });
      }

      // Count backward
      for (let i = 1; i < WIN_LENGTH; i++) {
        const r = row - dr * i;
        const c = col - dc * i;
        if (r < 0 || r >= BOARD_SIZE || c < 0 || c >= BOARD_SIZE) break;
        if (this.grid[r][c] !== player) break;
        cells.push({ row: r, col: c });
      }

      if (cells.length >= WIN_LENGTH) {
        return { winner: player as 0 | 1, winningCells: cells };
      }
    }

    return null;
  }

  private highlightWin(cells: { row: number; col: number }[]): void {
    const scale = this.cellSize / FRAME_SIZE;

    for (const { row, col } of cells) {
      const key = this.cellKey(row, col);

      // Add warning overlay (below stones, depth 0.5)
      const { x, y } = this.board.cellToPixel(row, col);
      const warning = this.add.sprite(x, y, SPRITE.WARNING_L, 0);
      warning.setScale(scale);
      warning.setDepth(0.5);
      warning.setTint(0x00ff44);
      warning.play({ key: WARNING_ANIMS.SURFACE.key, repeat: -1 });
    }
  }

  private resetGame(): void {
    // Play destroy animation on all stones, then rebuild
    const stones = Array.from(this.stoneSprites.values());
    if (stones.length === 0) {
      this.rebuildScene();
      return;
    }

    let pending = stones.length;
    for (const stone of stones) {
      stone.once(Phaser.Animations.Events.ANIMATION_COMPLETE, () => {
        pending--;
        if (pending === 0) {
          this.rebuildScene();
        }
      });
      stone.play(STONE_ANIMS.DESTROY.key);
    }
  }

  private rebuildScene(): void {
    this.children.removeAll();
    this.zones = [];
    this.stoneSprites.clear();
    this.initGame();
  }
}
