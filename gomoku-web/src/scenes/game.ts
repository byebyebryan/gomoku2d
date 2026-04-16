import Phaser from "phaser";
import { BOARD_SIZE, WIN_LENGTH, SPRITE, FRAME_SIZE, STONE_ANIMS, WARNING_ANIMS } from "../board/constants";
import { BoardRenderer } from "../board/board_renderer";
import { PlayerCard, ResetButton, PlayerInfo } from "../board/ui";
import { WasmBoard, WasmBot } from "../core/wasm_bridge";

type CellState = 0 | 1 | null;

interface GameResult {
  winner: 0 | 1 | null;
  winningCells?: { row: number; col: number }[];
}

const SIDEBAR_W = 200;
const BOT_DELAY_MS = 300;

export class GameScene extends Phaser.Scene {
  private board!: BoardRenderer;
  private cellSize: number = 0;

  private wasmBoard!: WasmBoard;
  private resetting: boolean = false;
  private stoneSprites: Map<string, Phaser.GameObjects.Sprite> = new Map();

  private players: [PlayerInfo, PlayerInfo] = [
    { name: "Black", wins: 0, isHuman: true },
    { name: "White", wins: 0, isHuman: false },
  ];

  private bots: [(WasmBot | null), (WasmBot | null)] = [null, null];
  private botTimer: Phaser.Time.TimerEvent | null = null;

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

    const boardAreaW = width - SIDEBAR_W;
    const boardAreaH = height;
    const padding = 20;
    const available = Math.min(boardAreaW, boardAreaH) - padding * 2;
    this.cellSize = Math.floor(available / BOARD_SIZE);

    const boardCenterX = boardAreaW / 2;
    const boardCenterY = height / 2;
    const originX = Math.floor(boardCenterX - ((BOARD_SIZE - 1) * this.cellSize) / 2);
    const originY = Math.floor(boardCenterY - ((BOARD_SIZE - 1) * this.cellSize) / 2);

    this.board = new BoardRenderer(this, this.cellSize, originX, originY);
    this.board.drawBoard();

    this.wasmBoard = new WasmBoard();
    this.resetting = false;
    this.stoneSprites.clear();

    // Create bots for non-human players
    this.bots = [null, null];
    for (let i = 0; i < 2; i++) {
      if (!this.players[i].isHuman) {
        this.bots[i] = WasmBot.createBaseline(3);
      }
    }

    this.pointer = this.board.createPointer();

    this.zones = this.board.createInteractiveZones((row, col) => this.onCellClick(row, col));

    const uiScale      = this.cellSize / FRAME_SIZE;
    const sideGap      = Math.round(6 * uiScale);
    const boardRightX  = originX + (BOARD_SIZE - 1) * this.cellSize + this.cellSize / 2;
    const cardMargin   = Math.round(4 * uiScale);
    const cardW        = Math.floor(width - boardRightX - 2 * cardMargin);
    const sidebarX     = boardRightX + cardMargin + cardW / 2;

    this.blackCard = new PlayerCard(this, 0, 0, 0, this.players[0], uiScale, cardW);
    this.whiteCard = new PlayerCard(this, 0, 0, 1, this.players[1], uiScale, cardW);
    this.resetBtn  = new ResetButton(this, 0, 0, () => this.resetGame(), uiScale, cardW);

    let sideY = originY;

    sideY += this.blackCard.height / 2;
    this.blackCard.setPosition(sidebarX, sideY);
    sideY += this.blackCard.height / 2 + sideGap;

    sideY += this.whiteCard.height / 2;
    this.whiteCard.setPosition(sidebarX, sideY);
    sideY += this.whiteCard.height / 2 + sideGap;

    sideY += this.resetBtn.height / 2;
    this.resetBtn.setPosition(sidebarX, sideY);

    this.updatePointerTint();

    this.input.on("pointermove", (pointer: Phaser.Input.Pointer) => {
      if (this.wasmBoard.result() !== "ongoing") {
        this.pointer.setVisible(false);
        return;
      }
      // Hide pointer during bot's turn
      const currentIdx = this.wasmBoard.currentPlayer() === 1 ? 0 : 1;
      if (!this.players[currentIdx].isHuman) {
        this.pointer.setVisible(false);
        return;
      }
      const cell = this.board.pixelToCell(pointer.x, pointer.y);
      if (cell && this.wasmBoard.cell(cell.row, cell.col) === 0) {
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

    // If it's a bot's turn first (unusual but possible), schedule it
    this.scheduleBotIfNeeded();
  }

  private updatePointerTint(): void {
    const wasmPlayer = this.wasmBoard.currentPlayer();
    this.pointer.setTint(wasmPlayer === 1 ? 0x404040 : 0xffffff);
    this.blackCard.setActive(wasmPlayer === 1);
    this.whiteCard.setActive(wasmPlayer === 2);
  }

  private cellKey(row: number, col: number): string {
    return `${row},${col}`;
  }

  private onCellClick(row: number, col: number): void {
    if (this.wasmBoard.result() !== "ongoing") {
      this.resetGame();
      return;
    }
    // Only humans can click
    const currentIdx = this.wasmBoard.currentPlayer() === 1 ? 0 : 1;
    if (!this.players[currentIdx].isHuman) return;
    if (!this.wasmBoard.isLegal(row, col)) return;

    this.applyAndRender(row, col);
  }

  private applyAndRender(row: number, col: number): void {
    this.pointer.setVisible(false);

    const mover = this.wasmBoard.currentPlayer();
    const moveResult = this.wasmBoard.applyMove(row, col);
    if (moveResult.error) return;

    const wasmPlayer = mover === 1 ? 0 : 1;
    const stone = this.board.placeStone(row, col, wasmPlayer);
    this.stoneSprites.set(this.cellKey(row, col), stone);
    stone.play(STONE_ANIMS.FORM.key);

    const wasmResult = this.wasmBoard.result();
    if (wasmResult === "black" || wasmResult === "white") {
      const winner = wasmResult === "black" ? 0 : 1;
      const winResult = this.checkWin(row, col, winner);
      if (winResult && winResult.winningCells) {
        this.highlightWin(winResult.winningCells);
      }

      this.blackCard.setActive(false);
      this.whiteCard.setActive(false);
      this.players[winner].wins++;
      if (winner === 0) this.blackCard.setWins(this.players[0].wins);
      else this.whiteCard.setWins(this.players[1].wins);
      return;
    }

    if (wasmResult === "draw") {
      this.blackCard.setActive(false);
      this.whiteCard.setActive(false);
      return;
    }

    this.updatePointerTint();
    this.scheduleBotIfNeeded();
  }

  private scheduleBotIfNeeded(): void {
    if (this.wasmBoard.result() !== "ongoing") return;
    const currentIdx = this.wasmBoard.currentPlayer() === 1 ? 0 : 1;
    const bot = this.bots[currentIdx];
    if (!bot) return;

    // Hide pointer during bot's turn
    this.pointer.setVisible(false);

    this.botTimer = this.time.delayedCall(BOT_DELAY_MS, () => {
      this.executeBotMove(bot);
    });
  }

  private executeBotMove(bot: WasmBot): void {
    if (this.wasmBoard.result() !== "ongoing") return;

    const move = bot.chooseMove(this.wasmBoard);
    if (!move) return;

    this.applyAndRender(move.row, move.col);
  }

  private checkWin(row: number, col: number, player: CellState): GameResult | null {
    const directions = [
      { dr: 0, dc: 1 },
      { dr: 1, dc: 0 },
      { dr: 1, dc: 1 },
      { dr: 1, dc: -1 },
    ];

    const wasmPlayer = player === 0 ? 1 : 2;

    for (const { dr, dc } of directions) {
      const cells: { row: number; col: number }[] = [{ row, col }];

      for (let i = 1; i < WIN_LENGTH; i++) {
        const r = row + dr * i;
        const c = col + dc * i;
        if (r < 0 || r >= BOARD_SIZE || c < 0 || c >= BOARD_SIZE) break;
        if (this.wasmBoard.cell(r, c) !== wasmPlayer) break;
        cells.push({ row: r, col: c });
      }

      for (let i = 1; i < WIN_LENGTH; i++) {
        const r = row - dr * i;
        const c = col - dc * i;
        if (r < 0 || r >= BOARD_SIZE || c < 0 || c >= BOARD_SIZE) break;
        if (this.wasmBoard.cell(r, c) !== wasmPlayer) break;
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

    // Kill any pending bot move immediately
    if (this.botTimer) {
      this.botTimer.destroy();
      this.botTimer = null;
    }

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
    if (this.botTimer) {
      this.botTimer.destroy();
      this.botTimer = null;
    }
    for (const bot of this.bots) {
      if (bot) bot.free();
    }
    this.bots = [null, null];
    this.wasmBoard.free();
    this.input.removeAllListeners();
    this.children.removeAll();
    this.zones = [];
    this.stoneSprites.clear();
    this.initGame();
  }
}
