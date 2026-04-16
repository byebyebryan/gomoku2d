import Phaser from "phaser";
import { BOARD_SIZE, WIN_LENGTH, SPRITE, FRAME_SIZE, STONE_ANIMS, POINTER_ANIMS, WARNING_ANIMS } from "../board/constants";
import { BoardRenderer } from "../board/board_renderer";
import { PlayerCard, ResetButton, PlayerInfo, SettingsButton, SettingsPanel, InfoBar } from "../board/ui";
import { WasmBoard, WasmBot } from "../core/wasm_bridge";

const SIDEBAR_W = 240;
const BOT_DELAY_MS = 300;
// sideH = cellSize/3, so total vertical span = (BOARD_SIZE + 1/3) * cellSize fills the screen exactly
const EDGE_RATIO = 1 / 3;
const POINTER_IDLE_ANIMS = [POINTER_ANIMS.OUT, POINTER_ANIMS.IN, POINTER_ANIMS.FULL] as const;
const STONE_IDLE_ANIMS   = [STONE_ANIMS.RELAX_1, STONE_ANIMS.RELAX_2, STONE_ANIMS.RELAX_3, STONE_ANIMS.RELAX_4] as const;

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
  private pointerIdleTimer: Phaser.Time.TimerEvent | null = null;
  private lastStoneSprite: Phaser.GameObjects.Sprite | null = null;
  private stoneIdleTimer: Phaser.Time.TimerEvent | null = null;
  private blackCard!: PlayerCard;
  private whiteCard!: PlayerCard;
  private resetBtn!: ResetButton;
  private settingsBtn!: SettingsButton;
  private settingsPanel!: SettingsPanel;
  private showingSettings: boolean = false;
  private gameVariant: "freestyle" | "renju" = "freestyle";
  private zones: Phaser.GameObjects.Zone[] = [];
  private forbiddenSprites: Phaser.GameObjects.Sprite[] = [];
  private winSprites: Phaser.GameObjects.Sprite[] = [];
  private gameStartTime: number = 0;
  private turnStartTime: number = 0;
  private accumulatedMs: [number, number] = [0, 0];
  private gameOver: boolean = false;
  // Cached to avoid crossing the Wasm boundary on every frame and every input event.
  private currentTurn: 1 | 2 = 1;
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
    this.cellSize = Math.min(boardAreaW / BOARD_SIZE, height / (BOARD_SIZE + EDGE_RATIO));
    const originY = this.cellSize / 2;
    const originX = (boardAreaW - (BOARD_SIZE - 1) * this.cellSize) / 2;

    this.board = new BoardRenderer(this, this.cellSize, originX, originY, height);
    this.board.drawBoard();

    this.wasmBoard = WasmBoard.createWithVariant(this.gameVariant);
    this.resetting = false;
    this.stoneSprites.clear();
    this.currentTurn = 1;
    this.gameOver = false;

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

    // When showing settings, show human players' current names; bots default to "Human".
    const p1HumanName = this.profiles[0].isHuman ? this.profiles[0].name : "Human";
    const p2HumanName = this.profiles[1].isHuman ? this.profiles[1].name : "Human";

    this.settingsPanel = new SettingsPanel(
      this, 0, 0, uiScale, cardW,
      this.gameVariant,
      this.profiles[0].isHuman,
      this.profiles[1].isHuman,
      p1HumanName,
      p2HumanName,
      (variant, p1IsHuman, p2IsHuman, p1Name, p2Name) => {
        this.gameVariant = variant;
        const [defN1, defN2] = defaultNames(p1IsHuman, p2IsHuman);
        this.profiles[0] = { name: p1IsHuman ? p1Name : defN1, wins: 0, isHuman: p1IsHuman };
        this.profiles[1] = { name: p2IsHuman ? p2Name : defN2, wins: 0, isHuman: p2IsHuman };
        this.blackProfileIdx = 0;
        this.hideSettings();
        this.rebuildScene();
      },
      () => this.hideSettings(),
    );
    this.settingsPanel.setPosition(sidebarX, originY + (BOARD_SIZE - 1) * this.cellSize / 2);
    this.settingsPanel.setVisible(false);

    this.gameStartTime = Date.now();
    this.turnStartTime = Date.now();
    this.accumulatedMs = [0, 0];
    this.infoBar = new InfoBar(this, 0, 0, uiScale, cardW, this.gameVariant);

    const totalH = this.infoBar.height + sectionGap
      + this.blackCard.height + innerGap
      + this.whiteCard.height + sectionGap
      + this.settingsBtn.height + innerGap
      + this.resetBtn.height;

    let sideY = originY + (BOARD_SIZE - 1) * this.cellSize / 2 - totalH / 2;

    sideY += this.infoBar.height / 2;
    this.infoBar.setPosition(sidebarX, sideY);
    sideY += this.infoBar.height / 2 + sectionGap;

    sideY += this.blackCard.height / 2;
    this.blackCard.setPosition(sidebarX, sideY);
    sideY += this.blackCard.height / 2 + innerGap;

    sideY += this.whiteCard.height / 2;
    this.whiteCard.setPosition(sidebarX, sideY);
    sideY += this.whiteCard.height / 2 + sectionGap;

    sideY += this.settingsBtn.height / 2;
    this.settingsBtn.setPosition(sidebarX, sideY);
    sideY += this.settingsBtn.height / 2 + innerGap;

    sideY += this.resetBtn.height / 2;
    this.resetBtn.setPosition(sidebarX, sideY);

    this.updatePointerTint();
    this.refreshForbiddenOverlays();

    this.input.on("pointermove", (pointer: Phaser.Input.Pointer) => {
      if (this.wasmBoard.result() !== "ongoing") { this.hidePointer(); return; }
      if (this.bots[this.currentTurn === 1 ? 0 : 1] !== null) { this.hidePointer(); return; }

      const cell = this.board.pixelToCell(pointer.x, pointer.y);
      if (cell && this.wasmBoard.cell(cell.row, cell.col) === 0) {
        const { x, y } = this.board.cellToPixel(cell.row, cell.col);
        this.pointer.setPosition(x, y);
        if (!this.pointer.visible) {
          this.pointer.setVisible(true);
          this.startPointerCycle();
        }
      } else {
        this.hidePointer();
      }
    });

    this.input.on("pointerout", () => { this.hidePointer(); });

    // If it's a bot's turn first (unusual but possible), schedule it
    this.scheduleBotIfNeeded();
  }

  update(): void {
    if (this.showingSettings || this.gameOver) return;
    if (!this.infoBar) return;

    const now = Date.now();
    this.infoBar.setTimer(this.formatTime(now - this.gameStartTime));

    const currentIdx = this.currentTurn === 1 ? 0 : 1;
    const turnElapsed = now - this.turnStartTime;

    for (let i = 0; i < 2; i++) {
      const card = i === 0 ? this.blackCard : this.whiteCard;
      if (i === currentIdx) {
        card.showPendingTimer(this.formatTime(this.accumulatedMs[i]), "+" + this.formatTime(turnElapsed));
      } else {
        card.setTimer(this.formatTime(this.accumulatedMs[i]));
      }
    }
  }

  private updatePointerTint(): void {
    this.pointer.setTint(this.currentTurn === 1 ? 0x404040 : 0xffffff);
    this.blackCard.setActive(this.currentTurn === 1);
    this.whiteCard.setActive(this.currentTurn === 2);
  }

  private refreshForbiddenOverlays(): void {
    for (const sprite of this.forbiddenSprites) sprite.destroy();
    this.forbiddenSprites = [];

    if (this.gameVariant !== "renju") return;
    if (this.wasmBoard.result() !== "ongoing") return;
    if (this.currentTurn !== 1) return; // only on black's turn
    if (this.bots[0] !== null) return;  // only when black is human

    // Collect empty cells within radius 2 of any placed stone — forbidden
    // moves require proximity to existing stones, so no need to scan the full board.
    const candidates = new Set<number>();
    for (let row = 0; row < BOARD_SIZE; row++) {
      for (let col = 0; col < BOARD_SIZE; col++) {
        if (this.wasmBoard.cell(row, col) === 0) continue;
        for (let dr = -2; dr <= 2; dr++) {
          for (let dc = -2; dc <= 2; dc++) {
            const r = row + dr, c = col + dc;
            if (r < 0 || r >= BOARD_SIZE || c < 0 || c >= BOARD_SIZE) continue;
            if (this.wasmBoard.cell(r, c) !== 0) continue;
            candidates.add(r * BOARD_SIZE + c);
          }
        }
      }
    }

    for (const idx of candidates) {
      const row = Math.floor(idx / BOARD_SIZE), col = idx % BOARD_SIZE;
      if (this.wasmBoard.isLegal(row, col)) continue;
      const { x, y } = this.board.cellToPixel(row, col);
      this.forbiddenSprites.push(this.createWarnSprite(x, y, 0xff4444));
    }
  }

  private createWarnSprite(x: number, y: number, tint: number, animKey: string = WARNING_ANIMS.POINTER.key, depth: number = 0.5): Phaser.GameObjects.Sprite {
    const sprite = this.add.sprite(x, y, SPRITE.WARNING, 0);
    sprite.setScale(this.cellSize / FRAME_SIZE);
    sprite.setDepth(depth);
    sprite.setTint(tint);
    sprite.play({ key: animKey, repeat: -1 });
    return sprite;
  }

  private hidePointer(): void {
    this.cancelPointerIdle();
    this.pointer.stop();
    this.pointer.setVisible(false);
  }

  private startPointerCycle(): void {
    this.scheduleNextPointerAnim();
  }

  private scheduleNextPointerAnim(): void {
    const delay = 500 + Math.random() * 1000;
    this.pointerIdleTimer = this.time.delayedCall(delay, () => {
      if (!this.pointer.visible) return;
      const anim = POINTER_IDLE_ANIMS[Math.floor(Math.random() * POINTER_IDLE_ANIMS.length)];
      this.pointer.play(anim.key);
      this.pointer.once(Phaser.Animations.Events.ANIMATION_COMPLETE, () => {
        if (!this.pointer.visible) return;
        this.scheduleNextPointerAnim();
      });
    });
  }

  private cancelPointerIdle(): void {
    if (this.pointerIdleTimer) {
      this.pointerIdleTimer.destroy();
      this.pointerIdleTimer = null;
    }
    this.pointer.removeAllListeners(Phaser.Animations.Events.ANIMATION_COMPLETE);
  }

  private startStoneIdle(stone: Phaser.GameObjects.Sprite): void {
    this.stopStoneIdle();
    this.lastStoneSprite = stone;
    this.scheduleNextStoneAnim();
  }

  private stopStoneIdle(): void {
    if (this.stoneIdleTimer) {
      this.stoneIdleTimer.destroy();
      this.stoneIdleTimer = null;
    }
    if (this.lastStoneSprite) {
      this.lastStoneSprite.removeAllListeners(Phaser.Animations.Events.ANIMATION_COMPLETE);
      this.lastStoneSprite.setFrame(0);
      this.lastStoneSprite = null;
    }
  }

  private scheduleNextStoneAnim(): void {
    const delay = 700 + Math.random() * 1500;
    this.stoneIdleTimer = this.time.delayedCall(delay, () => {
      if (!this.lastStoneSprite) return;
      const anim = STONE_IDLE_ANIMS[Math.floor(Math.random() * STONE_IDLE_ANIMS.length)];
      this.lastStoneSprite.play(anim.key);
      this.lastStoneSprite.once(Phaser.Animations.Events.ANIMATION_COMPLETE, () => {
        if (!this.lastStoneSprite) return;
        this.lastStoneSprite.setFrame(0);
        this.scheduleNextStoneAnim();
      });
    });
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
    if (this.bots[this.currentTurn === 1 ? 0 : 1] !== null) return;
    if (!this.wasmBoard.isLegal(row, col)) return;
    this.applyAndRender(row, col);
  }

  private applyAndRender(row: number, col: number): void {
    this.hidePointer();

    const mover = this.currentTurn;
    const moveResult = this.wasmBoard.applyMove(row, col);
    if (moveResult.error) return;

    const moverIdx = mover === 1 ? 0 : 1; // 0=black, 1=white — used for both timer and stone color
    this.accumulatedMs[moverIdx] += Date.now() - this.turnStartTime;
    this.turnStartTime = Date.now();

    const stone = this.board.placeStone(row, col, moverIdx as 0 | 1);
    this.stoneSprites.set(this.cellKey(row, col), stone);
    stone.play(STONE_ANIMS.FORM.key);
    stone.once(Phaser.Animations.Events.ANIMATION_COMPLETE, () => {
      if (this.wasmBoard.result() === "ongoing") this.startStoneIdle(stone);
    });

    const wasmResult = this.wasmBoard.result();
    if (wasmResult === "black" || wasmResult === "white") {
      this.gameOver = true;
      this.refreshForbiddenOverlays();
      this.blackCard.setTimer(this.formatTime(this.accumulatedMs[0]));
      this.whiteCard.setTimer(this.formatTime(this.accumulatedMs[1]));

      const winner = wasmResult === "black" ? 0 : 1;
      const winCells = this.checkWin(row, col, winner);
      if (winCells) this.highlightWin(winCells);

      this.blackCard.setActive(false);
      this.whiteCard.setActive(false);
      // Map winning color slot → profile; show pending +1 before folding in.
      const winnerProfileIdx = winner === 0 ? this.blackProfileIdx : (1 - this.blackProfileIdx) as 0 | 1;
      const oldWins = this.profiles[winnerProfileIdx].wins;
      this.profiles[winnerProfileIdx].wins++;
      if (winner === 0) this.blackCard.showPendingWin(oldWins);
      else              this.whiteCard.showPendingWin(oldWins);
      return;
    }

    if (wasmResult === "draw") {
      this.gameOver = true;
      this.refreshForbiddenOverlays();
      this.blackCard.setTimer(this.formatTime(this.accumulatedMs[0]));
      this.whiteCard.setTimer(this.formatTime(this.accumulatedMs[1]));
      this.blackCard.setActive(false);
      this.whiteCard.setActive(false);
      return;
    }

    this.currentTurn = this.wasmBoard.currentPlayer() as 1 | 2;
    this.updatePointerTint();
    this.refreshForbiddenOverlays();
    this.scheduleBotIfNeeded();
  }

  private scheduleBotIfNeeded(): void {
    if (this.wasmBoard.result() !== "ongoing") return;
    const bot = this.bots[this.currentTurn === 1 ? 0 : 1];
    if (!bot) return;

    this.hidePointer();

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

  private checkWin(row: number, col: number, player: 0 | 1): { row: number; col: number }[] | null {
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

      if (cells.length >= WIN_LENGTH) return cells;
    }

    return null;
  }

  private highlightWin(cells: { row: number; col: number }[]): void {
    this.stopStoneIdle();
    for (const { row, col } of cells) {
      const { x, y } = this.board.cellToPixel(row, col);
      this.winSprites.push(this.createWarnSprite(x, y, 0x00ff44, WARNING_ANIMS.HOVER.key, 2.5));
    }
  }

  private resetGame(): void {
    if (this.resetting) return;
    this.resetting = true;
    this.stopStoneIdle();

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
        if (pending === 0) this.rebuildScene();
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
    for (const sprite of this.forbiddenSprites) sprite.destroy();
    this.forbiddenSprites = [];
    for (const sprite of this.winSprites) sprite.destroy();
    this.winSprites = [];
    this.stopStoneIdle();
    this.hidePointer();
    this.wasmBoard.free();
    this.input.removeAllListeners();
    this.children.removeAll();
    this.zones = [];
    this.stoneSprites.clear();
    this.initGame();
  }
}
