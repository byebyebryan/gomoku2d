import Phaser from "phaser";
import { BOARD_SIZE, WIN_LENGTH, SPRITE, FRAME_SIZE, STONE_ANIMS, POINTER_ANIMS, WARNING_ANIMS } from "../board/constants";
import { BoardBounds, BoardRenderer } from "../board/board_renderer";
import { PlayerCard, ResetButton, PlayerInfo, SettingsButton, SettingsPanel, InfoBar } from "../board/ui";
import { WasmBoard, WasmBot } from "../core/wasm_bridge";
import { getGameSizeForViewport, getLayoutMode, getViewportSize } from "../layout";

const SIDEBAR_W = 240;
const BOT_DELAY_MS = 300;
const SETTINGS_SLIDE_MS = 220;
const EDGE_RATIO = 1 / 3;
const POINTER_IDLE_ANIMS = [POINTER_ANIMS.OUT, POINTER_ANIMS.IN, POINTER_ANIMS.FULL] as const;
const STONE_IDLE_ANIMS   = [STONE_ANIMS.RELAX_1, STONE_ANIMS.RELAX_2, STONE_ANIMS.RELAX_3, STONE_ANIMS.RELAX_4] as const;

class IdleCycle {
  private scene: Phaser.Scene;
  private timer: Phaser.Time.TimerEvent | null = null;
  private sprite: Phaser.GameObjects.Sprite | null = null;
  private anims: readonly { key: string }[];
  private delayMin: number;
  private delayMax: number;
  private resetFrame: number | null;
  private active: boolean = false;

  constructor(
    scene: Phaser.Scene,
    anims: readonly { key: string }[],
    delayMin: number,
    delayMax: number,
    resetFrame: number | null,
  ) {
    this.scene = scene;
    this.anims = anims;
    this.delayMin = delayMin;
    this.delayMax = delayMax;
    this.resetFrame = resetFrame;
  }

  start(sprite: Phaser.GameObjects.Sprite): void {
    this.stop();
    this.sprite = sprite;
    this.active = true;
    this.scheduleNext();
  }

  stop(): void {
    this.active = false;
    if (this.timer) {
      this.timer.destroy();
      this.timer = null;
    }
    if (this.sprite) {
      this.sprite.removeAllListeners(Phaser.Animations.Events.ANIMATION_COMPLETE);
      if (this.resetFrame !== null) this.sprite.setFrame(this.resetFrame);
      this.sprite = null;
    }
  }

  private scheduleNext(): void {
    if (!this.active || !this.sprite) return;
    const delay = this.delayMin + Math.random() * (this.delayMax - this.delayMin);
    this.timer = this.scene.time.delayedCall(delay, () => {
      if (!this.active || !this.sprite) return;
      const anim = this.anims[Math.floor(Math.random() * this.anims.length)];
      this.sprite!.play(anim.key);
      this.sprite!.once(Phaser.Animations.Events.ANIMATION_COMPLETE, () => {
        if (!this.active || !this.sprite) return;
        if (this.resetFrame !== null) this.sprite!.setFrame(this.resetFrame);
        this.scheduleNext();
      });
    });
  }
}

function defaultNames(p1Human: boolean, p2Human: boolean): [string, string] {
  if (p1Human  && p2Human)  return ["Human 1", "Human 2"];
  if (!p1Human && !p2Human) return ["Bot 1",   "Bot 2"];
  if (p1Human)              return ["Human",   "Bot"];
  return                           ["Bot",     "Human"];
}

type SettingsDraft = ReturnType<SettingsPanel["getValues"]>;

export class GameScene extends Phaser.Scene {
  private board!: BoardRenderer;
  private boardBounds!: BoardBounds;
  private boardContent!: Phaser.GameObjects.Container;
  private settingsContent!: Phaser.GameObjects.Container;
  private cellSize: number = 0;

  private wasmBoard!: WasmBoard;
  private resetting: boolean = false;
  private stoneSprites: Map<string, Phaser.GameObjects.Sprite> = new Map();

  private profiles: [PlayerInfo, PlayerInfo] = [
    { name: "Human", wins: 0, isHuman: true  },
    { name: "Bot",   wins: 0, isHuman: false },
  ];
  private blackProfileIdx: 0 | 1 = 0;

  private bots: [(WasmBot | null), (WasmBot | null)] = [null, null];
  private botTimer: Phaser.Time.TimerEvent | null = null;

  private pointer!: Phaser.GameObjects.Sprite;
  private pointerCycle: IdleCycle | null = null;
  private stoneCycle: IdleCycle | null = null;
  private blackCard!: PlayerCard;
  private whiteCard!: PlayerCard;
  private resetBtn!: ResetButton;
  private settingsBtn!: SettingsButton;
  private settingsPanel!: SettingsPanel;
  private showingSettings: boolean = false;
  private gameVariant: "freestyle" | "renju" = "freestyle";
  private forbiddenSprites: Phaser.GameObjects.Sprite[] = [];
  private winSprites: Phaser.GameObjects.Sprite[] = [];
  private gameStartTime: number = 0;
  private turnStartTime: number = 0;
  private accumulatedMs: [number, number] = [0, 0];
  private gameOver: boolean = false;
  private currentTurn: 1 | 2 = 1;
  private infoBar!: InfoBar;
  private lastTimerSec: number = -1;

  private isNarrow: boolean = false;
  private settingsTransitioning: boolean = false;
  private winningCells: { row: number; col: number }[] | null = null;
  private viewportResizeHandler: (() => void) | null = null;

  constructor() {
    super({ key: "GameScene" });
  }

  create(): void {
    this.initGame();
    this.attachViewportResizeHandlers();
    this.syncGameSizeToViewport();
  }

  private attachViewportResizeHandlers(): void {
    this.viewportResizeHandler = () => this.syncGameSizeToViewport();

    window.addEventListener("resize", this.viewportResizeHandler);
    window.visualViewport?.addEventListener("resize", this.viewportResizeHandler);

    this.events.once(Phaser.Scenes.Events.SHUTDOWN, this.detachViewportResizeHandlers, this);
    this.events.once(Phaser.Scenes.Events.DESTROY, this.detachViewportResizeHandlers, this);
  }

  private detachViewportResizeHandlers(): void {
    if (!this.viewportResizeHandler) return;

    window.removeEventListener("resize", this.viewportResizeHandler);
    window.visualViewport?.removeEventListener("resize", this.viewportResizeHandler);
    this.viewportResizeHandler = null;
  }

  private syncGameSizeToViewport(): void {
    const viewport = getViewportSize();
    const targetSize = getGameSizeForViewport(viewport.width, viewport.height);

    if (this.scale.width === targetSize.width && this.scale.height === targetSize.height) {
      return;
    }

    // Under FIT mode, Phaser expects setGameSize so it can recompute the display size
    // and preserve the aspect ratio inside the parent viewport.
    this.scale.setGameSize(targetSize.width, targetSize.height);
    this.relayout();
  }

  private getSettingsTransitionOffset(): { x: number; y: number } {
    if (this.isNarrow) {
      return {
        x: this.boardBounds.width + Math.round(this.cellSize * 1.5),
        y: 0,
      };
    }

    return {
      x: 0,
      y: this.boardBounds.height + Math.round(this.cellSize * 1.5),
    };
  }

  private setActionButtonsVisible(visible: boolean): void {
    this.settingsBtn?.setVisible(visible);
    this.resetBtn?.setVisible(visible);
  }

  private configureActionButtonsForGame(): void {
    this.settingsBtn.setLabel("SETTINGS");
    this.settingsBtn.setOnClick(() => this.showSettings());
    this.resetBtn.setLabel("RESET");
    this.resetBtn.setOnClick(() => this.resetGame());
  }

  private configureActionButtonsForSettings(): void {
    this.settingsBtn.setLabel("RESUME");
    this.settingsBtn.setOnClick(() => this.hideSettings());
    this.resetBtn.setLabel("NEW GAME");
    this.resetBtn.setOnClick(() => this.applySettingsAndRestart());
  }

  private applySettingsAndRestart(): void {
    const { variant, p1IsHuman, p2IsHuman, p1Name, p2Name } = this.settingsPanel.getValues();
    const [defN1, defN2] = defaultNames(p1IsHuman, p2IsHuman);

    this.gameVariant = variant;
    this.profiles[0] = { name: p1IsHuman ? p1Name : defN1, wins: 0, isHuman: p1IsHuman };
    this.profiles[1] = { name: p2IsHuman ? p2Name : defN2, wins: 0, isHuman: p2IsHuman };
    this.blackProfileIdx = 0;
    this.showingSettings = false;
    this.settingsTransitioning = false;
    this.rebuildScene();
  }

  private setSettingsViewState(open: boolean): void {
    const offset = this.getSettingsTransitionOffset();

    this.boardContent.x = open ? -offset.x : 0;
    this.boardContent.y = open ? -offset.y : 0;
    this.settingsContent.x = open ? 0 : offset.x;
    this.settingsContent.y = open ? 0 : offset.y;
    this.settingsContent.setVisible(open);
    this.settingsPanel.setVisible(open);
    this.setActionButtonsVisible(true);
    if (open) {
      this.configureActionButtonsForSettings();
    } else {
      this.configureActionButtonsForGame();
    }
  }

  private animateSettingsSwap(open: boolean, onComplete?: () => void): void {
    const offset = this.getSettingsTransitionOffset();

    this.settingsTransitioning = true;
    this.settingsContent.setVisible(true);
    this.settingsPanel.setVisible(true);

    if (open) {
      this.boardContent.x = 0;
      this.boardContent.y = 0;
      this.settingsContent.x = offset.x;
      this.settingsContent.y = offset.y;
    } else {
      this.boardContent.x = -offset.x;
      this.boardContent.y = -offset.y;
      this.settingsContent.x = 0;
      this.settingsContent.y = 0;
    }

    this.tweens.killTweensOf([this.boardContent, this.settingsContent]);
    this.tweens.add({
      targets: this.boardContent,
      x: open ? -offset.x : 0,
      y: open ? -offset.y : 0,
      duration: SETTINGS_SLIDE_MS,
      ease: "Cubic.Out",
    });
    this.tweens.add({
      targets: this.settingsContent,
      x: open ? 0 : offset.x,
      y: open ? 0 : offset.y,
      duration: SETTINGS_SLIDE_MS,
      ease: "Cubic.Out",
      onComplete: () => {
        this.settingsTransitioning = false;
        if (!open) {
          this.settingsPanel.setVisible(false);
          this.settingsContent.setVisible(false);
        }
        onComplete?.();
      },
    });
  }

  private createBoardViewportMask(): void {
    const maskSource = this.add.graphics();
    maskSource.fillStyle(0xffffff, 1);
    maskSource.fillRect(
      this.boardBounds.left,
      this.boardBounds.top,
      this.boardBounds.width,
      this.boardBounds.height,
    );
    maskSource.setVisible(false);

    const mask = maskSource.createGeometryMask();
    this.boardContent.setMask(mask);
    this.settingsContent.setMask(mask);
  }

  private createSettingsView(uiScale: number, settingsDraft: SettingsDraft | null = null): void {
    const panelPad = Math.round(24 * uiScale);
    const panelWidth = Math.max(320, Math.floor(this.boardBounds.width - 2 * panelPad));
    const settingsState = settingsDraft ?? {
      variant: this.gameVariant,
      p1IsHuman: this.profiles[0].isHuman,
      p2IsHuman: this.profiles[1].isHuman,
      p1Name: this.profiles[0].isHuman ? this.profiles[0].name : "Human",
      p2Name: this.profiles[1].isHuman ? this.profiles[1].name : "Human",
    };

    this.settingsPanel = new SettingsPanel(
      this,
      this.boardBounds.centerX,
      this.boardBounds.centerY,
      uiScale,
      panelWidth,
      settingsState.variant,
      settingsState.p1IsHuman,
      settingsState.p2IsHuman,
      settingsState.p1Name,
      settingsState.p2Name,
    );
    this.settingsPanel.attachTo(this.settingsContent);
    this.settingsPanel.setVisible(false);
  }

  private formatTime(ms: number): string {
    const totalSec = Math.floor(ms / 1000);
    const min = Math.floor(totalSec / 60);
    const sec = totalSec % 60;
    return `${min.toString().padStart(2, "0")}:${sec.toString().padStart(2, "0")}`;
  }

  private initGame(): void {
    this.wasmBoard = WasmBoard.createWithVariant(this.gameVariant);
    this.resetting = false;
    this.stoneSprites.clear();
    this.currentTurn = 1;
    this.gameOver = false;
    this.winningCells = null;

    const whiteProfileIdx = (1 - this.blackProfileIdx) as 0 | 1;
    this.bots = [null, null];
    if (!this.profiles[this.blackProfileIdx].isHuman) this.bots[0] = WasmBot.createBaseline(3);
    if (!this.profiles[whiteProfileIdx].isHuman)      this.bots[1] = WasmBot.createBaseline(3);

    this.gameStartTime = Date.now();
    this.turnStartTime = Date.now();
    this.accumulatedMs = [0, 0];

    this.initVisuals();
    this.scheduleBotIfNeeded();
  }

  private initVisuals(settingsDraft: SettingsDraft | null = null): void {
    const settingsOpen = this.showingSettings;

    this.settingsPanel?.destroy();
    this.input.removeAllListeners();
    this.children.removeAll();
    this.stoneSprites.clear();
    this.forbiddenSprites = [];
    this.winSprites = [];
    this.pointerCycle?.stop();
    this.pointerCycle = null;
    this.stoneCycle?.stop();
    this.stoneCycle = null;
    this.settingsTransitioning = false;

    const width = this.cameras.main.width;
    const height = this.cameras.main.height;
    this.isNarrow = getLayoutMode(width, height) === "portrait";

    this.boardContent = this.add.container(0, 0);
    this.settingsContent = this.add.container(0, 0);
    this.boardContent.setDepth(0);
    this.settingsContent.setDepth(1);

    let boardAreaW: number;
    let originX: number;
    let originY: number;
    let uiScale: number;
    let cardW: number;
    let portraitButtonGap = 0;
    let portraitButtonPad = 0;
    let portraitButtonSidePad = 0;
    let portraitButtonW = 0;

    if (this.isNarrow) {
      boardAreaW = width;
      this.cellSize = boardAreaW / BOARD_SIZE;
      uiScale = this.cellSize / FRAME_SIZE;
      originX = (boardAreaW - (BOARD_SIZE - 1) * this.cellSize) / 2;

      const topBarGap = Math.round(4 * uiScale);
      cardW = Math.floor((boardAreaW - 2 * topBarGap) / 3);

      this.infoBar = new InfoBar(this, 0, 0, uiScale, cardW, this.gameVariant);
      this.blackCard = new PlayerCard(this, 0, 0, 0, this.profiles[this.blackProfileIdx], uiScale, cardW);
      this.whiteCard = new PlayerCard(this, 0, 0, 1, this.profiles[(1 - this.blackProfileIdx) as 0 | 1], uiScale, cardW);

      const topBarActual = Math.max(this.blackCard.height, this.infoBar.height, this.whiteCard.height);
      const topBarPad = Math.round(6 * uiScale);
      const topBarH = topBarActual + topBarPad;

      portraitButtonGap = Math.round(8 * uiScale);
      portraitButtonPad = Math.round(10 * uiScale);
      portraitButtonSidePad = Math.round(8 * uiScale);
      const portraitButtonMaxW = Math.floor(boardAreaW * 0.3);
      const portraitButtonAvailableW = Math.floor((boardAreaW - 2 * portraitButtonSidePad - portraitButtonGap) / 2);
      portraitButtonW = Math.min(portraitButtonAvailableW, portraitButtonMaxW);
      this.settingsBtn = new SettingsButton(this, 0, 0, () => this.showSettings(), uiScale, portraitButtonW);
      this.resetBtn = new ResetButton(this, 0, 0, () => this.resetGame(), uiScale, portraitButtonW);
      const bottomBarH = Math.max(this.settingsBtn.height, this.resetBtn.height) + portraitButtonPad;

      originY = topBarH + this.cellSize / 2;

      const boardEdgeH = BOARD_SIZE * this.cellSize + this.cellSize / 2;
      const totalContentH = topBarH + boardEdgeH + bottomBarH;
      const yOffset = Math.max(0, (height - totalContentH) / 2);
      originY += yOffset;

      const topBarTotalW = 3 * cardW + 2 * topBarGap;
      const topBarLeft = (width - topBarTotalW) / 2;
      const topBarY = yOffset + topBarPad / 2 + topBarActual / 2;

      this.blackCard.setPosition(topBarLeft + cardW / 2, topBarY);
      this.infoBar.setPosition(topBarLeft + cardW + topBarGap + cardW / 2, topBarY);
      this.whiteCard.setPosition(topBarLeft + 2 * (cardW + topBarGap) + cardW / 2, topBarY);
    } else {
      boardAreaW = width - SIDEBAR_W;
      this.cellSize = Math.min(boardAreaW / BOARD_SIZE, height / (BOARD_SIZE + EDGE_RATIO));
      originY = this.cellSize / 2;
      originX = (boardAreaW - (BOARD_SIZE - 1) * this.cellSize) / 2;
      uiScale = this.cellSize / FRAME_SIZE;
      const boardRightX = originX + (BOARD_SIZE - 1) * this.cellSize + this.cellSize / 2;
      const cardMargin = Math.round(4 * uiScale);
      cardW = Math.floor(width - boardRightX - 2 * cardMargin);
      const boardPixelW = (BOARD_SIZE - 1) * this.cellSize;
      const sidebarX = originX + boardPixelW + this.cellSize / 2 + cardW / 2 + Math.round(4 * uiScale);
      const innerGap = 0.2 * uiScale;
      const sectionGap = Math.round(12 * uiScale);

      this.blackCard = new PlayerCard(this, 0, 0, 0, this.profiles[this.blackProfileIdx], uiScale, cardW);
      this.whiteCard = new PlayerCard(this, 0, 0, 1, this.profiles[(1 - this.blackProfileIdx) as 0 | 1], uiScale, cardW);
      this.resetBtn = new ResetButton(this, 0, 0, () => this.resetGame(), uiScale, cardW);
      this.settingsBtn = new SettingsButton(this, 0, 0, () => this.showSettings(), uiScale, cardW);
      this.infoBar = new InfoBar(this, 0, 0, uiScale, cardW, this.gameVariant);

      const totalH = this.infoBar.height + sectionGap
        + this.blackCard.height + innerGap
        + this.whiteCard.height + sectionGap
        + this.settingsBtn.height + innerGap
        + this.resetBtn.height;

      let sideY = originY + boardPixelW / 2 - totalH / 2;

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
    }

    this.board = new BoardRenderer(this, this.cellSize, originX, originY, height, this.boardContent);
    this.board.drawBoard();
    this.boardBounds = this.board.getBounds();
    this.createBoardViewportMask();

    if (this.isNarrow) {
      const buttonsY = this.boardBounds.bottom + portraitButtonPad / 2 + this.settingsBtn.height / 2;
      const totalButtonsW = 2 * portraitButtonW + portraitButtonGap;
      const buttonsLeft = (width - totalButtonsW) / 2;
      this.settingsBtn.setPosition(buttonsLeft + portraitButtonW / 2, buttonsY);
      this.resetBtn.setPosition(buttonsLeft + portraitButtonW + portraitButtonGap + portraitButtonW / 2, buttonsY);
    }

    this.createSettingsView(uiScale, settingsDraft);

    this.pointer = this.board.createPointer();
    this.pointerCycle = new IdleCycle(this, POINTER_IDLE_ANIMS, 500, 1500, null);
    this.stoneCycle = new IdleCycle(this, STONE_IDLE_ANIMS, 700, 2200, 0);

    for (let row = 0; row < BOARD_SIZE; row++) {
      for (let col = 0; col < BOARD_SIZE; col++) {
        const cell = this.wasmBoard.cell(row, col);
        if (cell !== 0) {
          const player = cell === 1 ? 0 : 1;
          const stone = this.board.placeStone(row, col, player as 0 | 1);
          this.stoneSprites.set(this.cellKey(row, col), stone);
        }
      }
    }

    this.blackCard.setTimer(this.formatTime(this.accumulatedMs[0]));
    this.whiteCard.setTimer(this.formatTime(this.accumulatedMs[1]));
    this.refreshForbiddenOverlays();
    this.lastTimerSec = -1;
    this.setSettingsViewState(settingsOpen);

    if (this.wasmBoard.result() !== "ongoing") {
      if (this.winningCells) {
        this.highlightWin(this.winningCells);
      }
      this.blackCard.setActive(false);
      this.whiteCard.setActive(false);
    } else if (settingsOpen) {
      this.blackCard.setActive(false);
      this.whiteCard.setActive(false);
    } else {
      this.updatePointerTint();
    }

    this.setupInputHandlers();
  }

  private getPointerType(pointer: Phaser.Input.Pointer): string {
    return (pointer as any).pointerType || "mouse";
  }

  private setupInputHandlers(): void {
    this.input.on("pointermove", (pointer: Phaser.Input.Pointer) => {
      if (this.showingSettings || this.settingsTransitioning) { this.hidePointer(); return; }
      if (this.wasmBoard.result() !== "ongoing") { this.hidePointer(); return; }
      if (this.bots[this.currentTurn === 1 ? 0 : 1] !== null) { this.hidePointer(); return; }

      const cell = this.board.pixelToCell(pointer.x, pointer.y);
      if (cell && this.wasmBoard.cell(cell.row, cell.col) === 0) {
        const { x, y } = this.board.cellToPixel(cell.row, cell.col);
        this.pointer.setPosition(x, y);
        this.pointer.setVisible(true);
        if (this.getPointerType(pointer) === "mouse") {
          this.pointerCycle?.start(this.pointer);
        }
      } else {
        this.hidePointer();
      }
    });

    this.input.on("pointerdown", (pointer: Phaser.Input.Pointer) => {
      if (this.showingSettings || this.settingsTransitioning) return;
      if (this.getPointerType(pointer) !== "touch") return;
      if (this.wasmBoard.result() !== "ongoing") return;
      if (this.bots[this.currentTurn === 1 ? 0 : 1] !== null) return;

      const cell = this.board.pixelToCell(pointer.x, pointer.y);
      if (cell && this.wasmBoard.cell(cell.row, cell.col) === 0) {
        const { x, y } = this.board.cellToPixel(cell.row, cell.col);
        this.pointer.setPosition(x, y);
        this.pointer.setVisible(true);
      }
    });

    this.input.on("pointerup", (pointer: Phaser.Input.Pointer) => {
      if (this.showingSettings || this.settingsTransitioning) return;
      const cell = this.board.pixelToCell(pointer.x, pointer.y);

      if (this.wasmBoard.result() !== "ongoing") {
        if (!cell) return;
        this.blackProfileIdx = (1 - this.blackProfileIdx) as 0 | 1;
        this.resetGame();
        return;
      }
      if (this.bots[this.currentTurn === 1 ? 0 : 1] !== null) return;
      if (!cell) return;

      if (this.getPointerType(pointer) === "touch") {
        if (this.pointer.visible) {
          const pointerCell = this.board.pixelToCell(this.pointer.x, this.pointer.y);
          if (pointerCell && this.wasmBoard.isLegal(pointerCell.row, pointerCell.col)) {
            this.applyAndRender(pointerCell.row, pointerCell.col);
          }
        }
        this.hidePointer();
      } else {
        if (this.wasmBoard.isLegal(cell.row, cell.col)) {
          this.applyAndRender(cell.row, cell.col);
        }
      }
    });

    this.input.on("pointerout", () => { this.hidePointer(); });
  }

  private relayout(): void {
    this.pointerCycle?.stop();
    this.stoneCycle?.stop();
    this.tweens.killTweensOf([this.boardContent, this.settingsContent]);

    if (this.botTimer) {
      this.botTimer.destroy();
      this.botTimer = null;
    }

    const settingsOpen = this.showingSettings;
    const settingsDraft = settingsOpen ? this.settingsPanel.getValues() : null;

    this.initVisuals(settingsDraft);

    if (!settingsOpen && this.wasmBoard.result() === "ongoing") {
      this.scheduleBotIfNeeded();
    }
  }

  update(): void {
    if (this.showingSettings || this.settingsTransitioning || this.gameOver) return;
    if (!this.infoBar) return;

    const now = Date.now();
    const sec = Math.floor(now / 1000);
    if (sec === this.lastTimerSec) return;
    this.lastTimerSec = sec;

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
    if (this.currentTurn !== 1) return;
    if (this.bots[0] !== null) return;

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
    this.boardContent.add(sprite);
    return sprite;
  }

  private hidePointer(): void {
    this.pointerCycle?.stop();
    this.pointer.stop();
    this.pointer.setVisible(false);
  }

  private showSettings(): void {
    if (this.showingSettings || this.settingsTransitioning) return;
    this.showingSettings = true;

    if (this.botTimer) {
      this.botTimer.destroy();
      this.botTimer = null;
    }

    this.hidePointer();
    this.blackCard.setActive(false);
    this.whiteCard.setActive(false);
    this.configureActionButtonsForSettings();
    this.animateSettingsSwap(true);
  }

  private hideSettings(): void {
    if (!this.showingSettings || this.settingsTransitioning) return;
    this.showingSettings = false;
    this.lastTimerSec = -1;

    this.animateSettingsSwap(false, () => {
      this.configureActionButtonsForGame();
      if (this.wasmBoard.result() === "ongoing") {
        this.updatePointerTint();
        this.turnStartTime = Date.now();
        this.scheduleBotIfNeeded();
      } else {
        this.blackCard.setActive(false);
        this.whiteCard.setActive(false);
      }
    });
  }

  private cellKey(row: number, col: number): string {
    return `${row},${col}`;
  }

  private applyAndRender(row: number, col: number): void {
    this.hidePointer();

    const mover = this.currentTurn;
    const moveResult = this.wasmBoard.applyMove(row, col);
    if (moveResult.error) return;

    const moverIdx = mover === 1 ? 0 : 1;
    this.accumulatedMs[moverIdx] += Date.now() - this.turnStartTime;
    this.turnStartTime = Date.now();

    const stone = this.board.placeStone(row, col, moverIdx as 0 | 1);
    this.stoneSprites.set(this.cellKey(row, col), stone);
    stone.play(STONE_ANIMS.FORM.key);
    stone.once(Phaser.Animations.Events.ANIMATION_COMPLETE, () => {
      if (this.wasmBoard.result() === "ongoing") this.stoneCycle!.start(stone);
    });

    const wasmResult = this.wasmBoard.result();
    if (wasmResult === "black" || wasmResult === "white") {
      this.gameOver = true;
      this.refreshForbiddenOverlays();
      this.blackCard.setTimer(this.formatTime(this.accumulatedMs[0]));
      this.whiteCard.setTimer(this.formatTime(this.accumulatedMs[1]));

      const winner = wasmResult === "black" ? 0 : 1;
      const winCells = this.checkWin(row, col, winner);
      if (winCells) {
        this.winningCells = winCells;
        this.highlightWin(winCells);
      }

      this.blackCard.setActive(false);
      this.whiteCard.setActive(false);
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
    if (this.showingSettings || this.settingsTransitioning) return;
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
    this.stoneCycle?.stop();
    for (const { row, col } of cells) {
      const { x, y } = this.board.cellToPixel(row, col);
      this.winSprites.push(this.createWarnSprite(x, y, 0x00ff44, WARNING_ANIMS.HOVER.key, 2.5));
    }
  }

  private resetGame(): void {
    if (this.resetting) return;
    this.resetting = true;
    this.stoneCycle?.stop();

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
    this.tweens.killTweensOf([this.boardContent, this.settingsContent]);
    this.showingSettings = false;
    this.settingsTransitioning = false;
    for (const bot of this.bots) {
      if (bot) bot.free();
    }
    this.bots = [null, null];
    for (const sprite of this.forbiddenSprites) sprite.destroy();
    this.forbiddenSprites = [];
    for (const sprite of this.winSprites) sprite.destroy();
    this.winSprites = [];
    this.stoneCycle?.stop();
    this.hidePointer();
    this.wasmBoard.free();
    this.input.removeAllListeners();
    this.children.removeAll();
    this.stoneSprites.clear();
    this.initGame();
  }
}
