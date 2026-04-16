import Phaser from "phaser";
import { BOARD_SIZE, WIN_LENGTH, SPRITE, FRAME_SIZE, STONE_ANIMS, WARNING_ANIMS } from "../board/constants";
import { BoardRenderer } from "../board/board_renderer";
import { PlayerCard, ResetButton, PlayerInfo } from "../board/ui";

type CellState = 0 | 1 | null;

interface GameResult {
  winner: 0 | 1 | null;
  winningCells?: { row: number; col: number }[];
}

// Right sidebar — wide enough for scaled cards
const SIDEBAR_W = 200;

export class GameScene extends Phaser.Scene {
  private board!: BoardRenderer;
  private cellSize: number = 0;

  // Game state
  private grid: CellState[][] = [];
  private currentPlayer: 0 | 1 = 0;
  private moveCount: number = 0;
  private gameOver: boolean = false;
  private resetting: boolean = false;
  private stoneSprites: Map<string, Phaser.GameObjects.Sprite> = new Map();

  // Player stats
  private players: [PlayerInfo, PlayerInfo] = [
    { name: "Black", wins: 0, isHuman: true },
    { name: "White", wins: 0, isHuman: true },
  ];

  // UI
  private pointer!: Phaser.GameObjects.Sprite;
  private blackCard!: PlayerCard;
  private whiteCard!: PlayerCard;
  private resetBtn!: ResetButton;
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

    // Board fills left side (full height, square)
    const boardAreaW = width - SIDEBAR_W;
    const boardAreaH = height;
    const padding = 20;
    const available = Math.min(boardAreaW, boardAreaH) - padding * 2;
    // Divide by BOARD_SIZE (not BOARD_SIZE-1) so the half-cell border on each
    // side of the outermost grid lines also fits within the padded area.
    this.cellSize = Math.floor(available / BOARD_SIZE);

    // Center board in the left area
    const boardCenterX = boardAreaW / 2;
    const boardCenterY = height / 2;
    const originX = Math.floor(boardCenterX - ((BOARD_SIZE - 1) * this.cellSize) / 2);
    const originY = Math.floor(boardCenterY - ((BOARD_SIZE - 1) * this.cellSize) / 2);

    this.board = new BoardRenderer(this, this.cellSize, originX, originY);
    this.board.drawBoard();

    // Init game state
    this.grid = Array.from({ length: BOARD_SIZE }, () => Array(BOARD_SIZE).fill(null));
    this.currentPlayer = 0;
    this.moveCount = 0;
    this.gameOver = false;
    this.resetting = false;
    this.stoneSprites.clear();

    // Pointer
    this.pointer = this.board.createPointer();

    // Interactive zones
    this.zones = this.board.createInteractiveZones((row, col) => this.onCellClick(row, col));

    // --- Right sidebar ---
    const uiScale      = this.cellSize / FRAME_SIZE;
    const sideGap      = Math.round(6 * uiScale);
    const boardRightX  = originX + (BOARD_SIZE - 1) * this.cellSize + this.cellSize / 2;
    const cardMargin   = Math.round(4 * uiScale); // gap between board edge and card
    const cardW        = Math.floor(width - boardRightX - 2 * cardMargin);
    const sidebarX     = boardRightX + cardMargin + cardW / 2;

    // Create UI elements at (0,0), then stack them after measuring heights.
    this.blackCard = new PlayerCard(this, 0, 0, 0, this.players[0], uiScale, cardW);
    this.whiteCard = new PlayerCard(this, 0, 0, 1, this.players[1], uiScale, cardW);
    this.resetBtn  = new ResetButton(this, 0, 0, () => this.resetGame(), uiScale, cardW);

    let sideY = originY; // align top of first card to top board grid line

    sideY += this.blackCard.height / 2;
    this.blackCard.setPosition(sidebarX, sideY);
    sideY += this.blackCard.height / 2 + sideGap;

    sideY += this.whiteCard.height / 2;
    this.whiteCard.setPosition(sidebarX, sideY);
    sideY += this.whiteCard.height / 2 + sideGap;

    sideY += this.resetBtn.height / 2;
    this.resetBtn.setPosition(sidebarX, sideY);

    // Activate initial state
    this.updatePointerTint();

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

    this.input.on("pointerout", () => {
      this.pointer.setVisible(false);
    });
  }

  private updatePointerTint(): void {
    this.pointer.setTint(this.currentPlayer === 0 ? 0x404040 : 0xffffff);
    this.blackCard.setActive(this.currentPlayer === 0);
    this.whiteCard.setActive(this.currentPlayer === 1);
  }

  private cellKey(row: number, col: number): string {
    return `${row},${col}`;
  }

  private onCellClick(row: number, col: number): void {
    if (this.gameOver) {
      this.resetGame();
      return;
    }
    if (this.grid[row][col] !== null) return;

    this.pointer.setVisible(false);

    // Place stone
    this.grid[row][col] = this.currentPlayer;
    this.moveCount++;

    const stone = this.board.placeStone(row, col, this.currentPlayer);
    this.stoneSprites.set(this.cellKey(row, col), stone);
    stone.play(STONE_ANIMS.FORM.key);

    // Check win
    const result = this.checkWin(row, col, this.currentPlayer);
    if (result) {
      this.gameOver = true;
      this.pointer.setVisible(false);

      if (result.winningCells) {
        this.highlightWin(result.winningCells);
      }

      this.blackCard.setActive(false);
      this.whiteCard.setActive(false);

      if (result.winner !== null) {
        this.players[result.winner].wins++;
        if (result.winner === 0) this.blackCard.setWins(this.players[0].wins);
        else this.whiteCard.setWins(this.players[1].wins);
      }
      return;
    }

    // Check draw
    if (this.moveCount >= BOARD_SIZE * BOARD_SIZE) {
      this.gameOver = true;
      this.pointer.setVisible(false);
      this.blackCard.setActive(false);
      this.whiteCard.setActive(false);
      return;
    }

    // Switch player
    this.currentPlayer = this.currentPlayer === 0 ? 1 : 0;
    this.updatePointerTint();
  }

  private checkWin(row: number, col: number, player: CellState): GameResult | null {
    const directions = [
      { dr: 0, dc: 1 },
      { dr: 1, dc: 0 },
      { dr: 1, dc: 1 },
      { dr: 1, dc: -1 },
    ];

    for (const { dr, dc } of directions) {
      const cells: { row: number; col: number }[] = [{ row, col }];

      for (let i = 1; i < WIN_LENGTH; i++) {
        const r = row + dr * i;
        const c = col + dc * i;
        if (r < 0 || r >= BOARD_SIZE || c < 0 || c >= BOARD_SIZE) break;
        if (this.grid[r][c] !== player) break;
        cells.push({ row: r, col: c });
      }

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
      const { x, y } = this.board.cellToPixel(row, col);
      const warning = this.add.sprite(x, y, SPRITE.WARNING_L, 0);
      warning.setScale(scale);
      warning.setDepth(0.5);
      warning.setTint(0x00ff44);
      warning.play({ key: WARNING_ANIMS.SURFACE.key, repeat: -1 });
    }
  }

  private resetGame(): void {
    if (this.resetting) return;
    this.resetting = true;

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
    this.input.removeAllListeners();
    this.children.removeAll();
    this.zones = [];
    this.stoneSprites.clear();
    this.initGame();
  }
}
