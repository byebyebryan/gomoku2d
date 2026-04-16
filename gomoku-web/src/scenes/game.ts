import Phaser from "phaser";
import { BOARD_SIZE, WIN_LENGTH, SPRITE, FRAME_SIZE, STONE_ANIMS, WARNING_ANIMS } from "../board/constants";
import { BoardRenderer } from "../board/board_renderer";
import { PlayerCard, ResetButton, PlayerInfo, SettingsButton, SettingsPanel, InfoBar } from "../board/ui";
import { WasmBoard, WasmBot } from "../core/wasm_bridge";

type CellState = 0 | 1 | null;

interface GameResult {
  winner: 0 | 1 | null;
  winningCells?: { row: number; col: number }[];
}

const SIDEBAR_W = 240;
const BOT_DELAY_MS = 300;

function defaultNames(p1Human: boolean, p2Human: boolean): [string, string] {
  if (p1Human  && p2Human)  return ["Human 1", "Human 2"];
  if (!p1Human && !p2Human) return ["Bot 1",   "Bot 2"];
  if (p1Human)              return ["Human",   "Bot"];
  return                           ["Bot",     "Human"];
}

export class GameScene extends Phaser.Scene {
  private board!: BoardRenderer;
  private cellSize: number = 0;

  private wasmBoard!: WasmBoard;
  private resetting: boolean = false;
  private stoneSprites: Map<string, Phaser.GameObjects.Sprite> = new Map();

  // Player profiles — indexed by player number (0=P1, 1=P2), not color slot.
  private profiles: [PlayerInfo, PlayerInfo] = [
    { name: "Human", wins: 0, isHuman: true  },
    { name: "Bot",   wins: 0, isHuman: false },
  ];
  // Which profile index occupies the black slot (0 or 1).
  private blackProfileIdx: 0 | 1 = 0;

  private bots: [(WasmBot | null), (WasmBot | null)] = [null, null];
  private botTimer: Phaser.Time.TimerEvent | null = null;

  private pointer!: Phaser.GameObjects.Sprite;
  private blackCard!: PlayerCard;
  private whiteCard!: PlayerCard;
  private resetBtn!: ResetButton;
  private settingsBtn!: SettingsButton;
  private settingsPanel!: SettingsPanel;
  private showingSettings: boolean = false;
  private gameVariant: "freestyle" | "renju" = "freestyle";
  private zones: Phaser.GameObjects.Zone[] = [];
  private gameStartTime: number = 0;
  private turnStartTime: number = 0;
  private accumulatedMs: [number, number] = [0, 0];
  private gameEndTime: number = 0;
  private infoBar!: InfoBar;

  constructor() {
    super({ key: "GameScene" });
  }

  create(): void {
    this.initGame();
  }

  private formatTime(ms: number): string {
    const totalSec = Math.floor(ms / 1000);
    const min = Math.floor(totalSec / 60);
    const sec = totalSec % 60;
    return `${min.toString().padStart(2, "0")}:${sec.toString().padStart(2, "0")}`;
  }

  private initGame(): void {
    const width = this.cameras.main.width;
    const height = this.cameras.main.height;

    const boardAreaW = width - SIDEBAR_W;
    // Float cellSize so top=0 and edge-bottom=height with no rounding gaps.
    // Edge ratio: sideH = cellSize/3, total vertical = (BOARD_SIZE + 1/3) * cellSize.
    const EDGE_RATIO = 1 / 3;
    this.cellSize = Math.min(boardAreaW / BOARD_SIZE, height / (BOARD_SIZE + EDGE_RATIO));
    const originY = this.cellSize / 2;
    const originX = (boardAreaW - (BOARD_SIZE - 1) * this.cellSize) / 2;

    this.board = new BoardRenderer(this, this.cellSize, originX, originY, height);
    this.board.drawBoard();

    this.wasmBoard = new WasmBoard();
    this.resetting = false;
    this.stoneSprites.clear();

    // Create bots per color slot based on the assigned profile.
    const whiteProfileIdx = (1 - this.blackProfileIdx) as 0 | 1;
    this.bots = [null, null];
    if (!this.profiles[this.blackProfileIdx].isHuman) this.bots[0] = WasmBot.createBaseline(3);
    if (!this.profiles[whiteProfileIdx].isHuman)      this.bots[1] = WasmBot.createBaseline(3);

    this.pointer = this.board.createPointer();

    this.zones = this.board.createInteractiveZones((row, col) => this.onCellClick(row, col));

    const uiScale      = this.cellSize / FRAME_SIZE;
    const innerGap     = 0.2 * uiScale;
    const sectionGap   = Math.round(12 * uiScale);
    const boardRightX  = originX + (BOARD_SIZE - 1) * this.cellSize + this.cellSize / 2;
    const cardMargin   = Math.round(4 * uiScale);
    const cardW        = Math.floor(width - boardRightX - 2 * cardMargin);
    const sidebarX     = boardRightX + cardMargin + cardW / 2;

    this.blackCard = new PlayerCard(this, 0, 0, 0, this.profiles[this.blackProfileIdx], uiScale, cardW);
    this.whiteCard = new PlayerCard(this, 0, 0, 1, this.profiles[whiteProfileIdx], uiScale, cardW);
    this.resetBtn  = new ResetButton(this, 0, 0, () => this.resetGame(), uiScale, cardW);
    this.settingsBtn = new SettingsButton(this, 0, 0, () => this.showSettings(), uiScale, cardW);

    this.settingsPanel = new SettingsPanel(
      this,
      0,
      0,
      uiScale,
      cardW,
      this.gameVariant,
      this.profiles[0].isHuman,
      this.profiles[1].isHuman,
      (variant, p1IsHuman, p2IsHuman) => {
        this.gameVariant = variant;
        const [n1, n2] = defaultNames(p1IsHuman, p2IsHuman);
        this.profiles[0] = { name: n1, wins: 0, isHuman: p1IsHuman };
        this.profiles[1] = { name: n2, wins: 0, isHuman: p2IsHuman };
        this.blackProfileIdx = 0; // P1 always starts as black after settings
        this.hideSettings();
        this.rebuildScene();
      },
      () => this.hideSettings(),
    );
    // Center panel vertically within the board area
    const panelCenterY = originY + (BOARD_SIZE - 1) * this.cellSize / 2;
    this.settingsPanel.setPosition(sidebarX, panelCenterY);
    this.settingsPanel.setVisible(false);

    this.gameStartTime = Date.now();
    this.turnStartTime = Date.now();
    this.accumulatedMs = [0, 0];
    this.gameEndTime = 0;
    this.infoBar = new InfoBar(this, 0, 0, uiScale, cardW, this.gameVariant);

    // Calculate total sidebar content height
    const totalH = this.infoBar.height + sectionGap
      + this.blackCard.height + innerGap
      + this.whiteCard.height + sectionGap
      + this.settingsBtn.height + innerGap
      + this.resetBtn.height;

    // Center vertically within board area
    const boardMidY = originY + (BOARD_SIZE - 1) * this.cellSize / 2;
    let sideY = boardMidY - totalH / 2;

    // --- Info section ---
    sideY += this.infoBar.height / 2;
    this.infoBar.setPosition(sidebarX, sideY);
    sideY += this.infoBar.height / 2 + sectionGap;

    // --- Player section (tight) ---
    sideY += this.blackCard.height / 2;
    this.blackCard.setPosition(sidebarX, sideY);
    sideY += this.blackCard.height / 2 + innerGap;

    sideY += this.whiteCard.height / 2;
    this.whiteCard.setPosition(sidebarX, sideY);
    sideY += this.whiteCard.height / 2 + sectionGap;

    // --- Button section (tight) ---
    sideY += this.settingsBtn.height / 2;
    this.settingsBtn.setPosition(sidebarX, sideY);
    sideY += this.settingsBtn.height / 2 + innerGap;

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
      if (this.bots[currentIdx] !== null) {
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

  update(): void {
    if (this.showingSettings || this.gameEndTime > 0) return;
    if (!this.infoBar) return;

    const now = Date.now();

    // Game timer
    const gameElapsed = now - this.gameStartTime;
    this.infoBar.setTimer(this.formatTime(gameElapsed));

    // Player timers
    const currentIdx = this.wasmBoard.currentPlayer() === 1 ? 0 : 1;
    const turnElapsed = now - this.turnStartTime;

    for (let i = 0; i < 2; i++) {
      const acc = this.accumulatedMs[i] + (i === currentIdx ? turnElapsed : 0);
      const card = i === 0 ? this.blackCard : this.whiteCard;
      card.setTimer(this.formatTime(acc));
    }
  }

  private updatePointerTint(): void {
    const wasmPlayer = this.wasmBoard.currentPlayer();
    this.pointer.setTint(wasmPlayer === 1 ? 0x404040 : 0xffffff);
    this.blackCard.setActive(wasmPlayer === 1);
    this.whiteCard.setActive(wasmPlayer === 2);
  }

  private showSettings(): void {
    this.showingSettings = true;

    if (this.botTimer) {
      this.botTimer.destroy();
      this.botTimer = null;
    }

    this.infoBar.setVisible(false);
    this.blackCard.setVisible(false);
    this.whiteCard.setVisible(false);
    this.resetBtn.setVisible(false);
    this.settingsBtn.setVisible(false);

    this.settingsPanel.setVisible(true);

    this.zones.forEach(z => z.setActive(false));
  }

  private hideSettings(): void {
    this.showingSettings = false;

    this.infoBar.setVisible(true);
    this.blackCard.setVisible(true);
    this.whiteCard.setVisible(true);
    this.resetBtn.setVisible(true);
    this.settingsBtn.setVisible(true);

    this.settingsPanel.setVisible(false);

    this.zones.forEach(z => z.setActive(true));

    // Resume turn clock and reschedule bot if needed
    this.turnStartTime = Date.now();
    this.scheduleBotIfNeeded();
  }

  private cellKey(row: number, col: number): string {
    return `${row},${col}`;
  }

  private onCellClick(row: number, col: number): void {
    if (this.wasmBoard.result() !== "ongoing") {
      // Swap color slots so the loser opens as black next game.
      this.blackProfileIdx = (1 - this.blackProfileIdx) as 0 | 1;
      this.resetGame();
      return;
    }
    // Only humans can click
    const currentIdx = this.wasmBoard.currentPlayer() === 1 ? 0 : 1;
    if (this.bots[currentIdx] !== null) return;
    if (!this.wasmBoard.isLegal(row, col)) return;

    this.applyAndRender(row, col);
  }

  private applyAndRender(row: number, col: number): void {
    this.pointer.setVisible(false);

    const mover = this.wasmBoard.currentPlayer();
    const moveResult = this.wasmBoard.applyMove(row, col);
    if (moveResult.error) return;

    const moverIdx = mover === 1 ? 0 : 1;
    this.accumulatedMs[moverIdx] += Date.now() - this.turnStartTime;
    this.turnStartTime = Date.now();

    const wasmPlayer = mover === 1 ? 0 : 1;
    const stone = this.board.placeStone(row, col, wasmPlayer);
    this.stoneSprites.set(this.cellKey(row, col), stone);
    stone.play(STONE_ANIMS.FORM.key);

    const wasmResult = this.wasmBoard.result();
    if (wasmResult === "black" || wasmResult === "white") {
      this.gameEndTime = Date.now();
      const winner = wasmResult === "black" ? 0 : 1;
      const winResult = this.checkWin(row, col, winner);
      if (winResult && winResult.winningCells) {
        this.highlightWin(winResult.winningCells);
      }

      this.blackCard.setActive(false);
      this.whiteCard.setActive(false);
      // Map winning color slot → profile and update the correct card.
      const winnerProfileIdx = winner === 0 ? this.blackProfileIdx : (1 - this.blackProfileIdx) as 0 | 1;
      this.profiles[winnerProfileIdx].wins++;
      if (winner === 0) this.blackCard.setWins(this.profiles[this.blackProfileIdx].wins);
      else              this.whiteCard.setWins(this.profiles[(1 - this.blackProfileIdx) as 0 | 1].wins);
      return;
    }

    if (wasmResult === "draw") {
      this.gameEndTime = Date.now();
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
