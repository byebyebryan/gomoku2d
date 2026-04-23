import Phaser from "phaser";

import {
  BOARD_SIZE,
  COLOR,
  FRAME_SIZE,
  FONT_KEY,
  POINTER_ANIMS,
  SPRITE,
  STONE_ANIMS,
  WARNING_ANIMS,
} from "./constants";
import { BoardRenderer } from "./board_renderer";
import {
  canPlaceTouchCandidate,
  moveTouchCandidateFromDrag,
  shouldAnimatePlacedStone,
  shouldRestartPointerCycle,
  shouldStopStoneIdleCycle,
} from "./board_scene_logic";

import type { CellPosition, CellStone, MatchMove, MatchStatus } from "../game/types";

const POINTER_IDLE_ANIMS = [POINTER_ANIMS.OUT, POINTER_ANIMS.IN, POINTER_ANIMS.FULL] as const;
const STONE_IDLE_ANIMS = [
  STONE_ANIMS.RELAX_1,
  STONE_ANIMS.RELAX_2,
  STONE_ANIMS.RELAX_3,
  STONE_ANIMS.RELAX_4,
] as const;
const ASSET_URLS = {
  fontData: new URL("../../assets/fonts/PixelOperator8-Bold.fnt", import.meta.url).toString(),
  fontImage: new URL("../../assets/fonts/PixelOperator8-Bold.png", import.meta.url).toString(),
  pointer: new URL("../../assets/sprites/pointer.png", import.meta.url).toString(),
  stone: new URL("../../assets/sprites/stone.png", import.meta.url).toString(),
  warning: new URL("../../assets/sprites/warning.png", import.meta.url).toString(),
} as const;

class IdleCycle {
  private readonly scene: Phaser.Scene;
  private readonly anims: readonly { key: string }[];
  private readonly delayMax: number;
  private readonly delayMin: number;
  private readonly resetFrame: number | null;
  private active = false;
  private sprite: Phaser.GameObjects.Sprite | null = null;
  private timer: Phaser.Time.TimerEvent | null = null;

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
    this.timer?.destroy();
    this.timer = null;

    if (!this.sprite) {
      return;
    }

    this.sprite.removeAllListeners(Phaser.Animations.Events.ANIMATION_COMPLETE);
    if (this.resetFrame !== null) {
      this.sprite.setFrame(this.resetFrame);
    }
    this.sprite = null;
  }

  private scheduleNext(): void {
    if (!this.active || !this.sprite) {
      return;
    }

    const delay = this.delayMin + Math.random() * (this.delayMax - this.delayMin);
    this.timer = this.scene.time.delayedCall(delay, () => {
      const sprite = this.sprite;
      if (!this.active || !sprite || !sprite.active || !sprite.scene) {
        return;
      }

      const anim = this.anims[Math.floor(Math.random() * this.anims.length)];
      sprite.play(anim.key);
      sprite.once(Phaser.Animations.Events.ANIMATION_COMPLETE, () => {
        if (!this.active || !this.sprite || !this.sprite.active || !this.sprite.scene) {
          return;
        }

        if (this.resetFrame !== null) {
          this.sprite.setFrame(this.resetFrame);
        }
        this.scheduleNext();
      });
    });
  }
}

export interface BoardSceneState {
  cells: CellStone[][];
  currentPlayer: 1 | 2;
  forbiddenMoves: CellPosition[];
  interactive: boolean;
  lastMove: CellPosition | null;
  moves: MatchMove[];
  onAdvanceRound: () => void;
  onPlace: (row: number, col: number) => void;
  onTouchCandidateChange: (candidate: CellPosition | null, canPlace: boolean) => void;
  touchCandidateResetVersion: number;
  mobileTouchPlacement: boolean;
  showSequenceNumbers: boolean;
  status: MatchStatus;
  threatMoves: CellPosition[];
  winningMoves: CellPosition[];
  winningCells: CellPosition[];
}

const DEFAULT_STATE: BoardSceneState = {
  cells: Array.from({ length: BOARD_SIZE }, () =>
    Array.from({ length: BOARD_SIZE }, () => null),
  ),
  currentPlayer: 1,
  forbiddenMoves: [],
  interactive: false,
  lastMove: null,
  mobileTouchPlacement: false,
  moves: [],
  onAdvanceRound: () => undefined,
  onPlace: () => undefined,
  onTouchCandidateChange: () => undefined,
  touchCandidateResetVersion: 0,
  showSequenceNumbers: false,
  status: "playing",
  threatMoves: [],
  winningMoves: [],
  winningCells: [],
};

export class BoardScene extends Phaser.Scene {
  private boardState: BoardSceneState = DEFAULT_STATE;
  private board: BoardRenderer | null = null;
  private boardLayer: Phaser.GameObjects.Container | null = null;
  private currentCellSize = 0;
  private forbiddenSprites: Phaser.GameObjects.Sprite[] = [];
  private hintSprites: Phaser.GameObjects.Sprite[] = [];
  private overlayLayer: Phaser.GameObjects.Container | null = null;
  private pointer: Phaser.GameObjects.Sprite | null = null;
  private pointerCellKey: string | null = null;
  private pointerCycle: IdleCycle | null = null;
  private pointerLayer: Phaser.GameObjects.Container | null = null;
  private renderVersion = 0;
  private reportedTouchCandidateKey: string | null = null;
  private reportedTouchCanPlace = false;
  private root: Phaser.GameObjects.Container | null = null;
  private sequenceLabels: Phaser.GameObjects.BitmapText[] = [];
  private stoneCycle: IdleCycle | null = null;
  private stoneSprites: Map<string, Phaser.GameObjects.Sprite> = new Map();
  private touchCandidate: CellPosition | null = null;
  private touchDragOrigin: { x: number; y: number; candidate: CellPosition } | null = null;
  private winSprites: Phaser.GameObjects.Sprite[] = [];

  constructor() {
    super({ key: "BoardScene" });
  }

  preload(): void {
    this.preloadSpritesheet(SPRITE.STONE, ASSET_URLS.stone);
    this.preloadSpritesheet(SPRITE.POINTER, ASSET_URLS.pointer);
    this.preloadSpritesheet(SPRITE.WARNING, ASSET_URLS.warning);

    if (!this.cache.bitmapFont.exists(FONT_KEY)) {
      this.load.bitmapFont(FONT_KEY, ASSET_URLS.fontImage, ASSET_URLS.fontData);
    }
  }

  create(): void {
    this.ensureAnimations();
    this.pointerCycle = new IdleCycle(this, POINTER_IDLE_ANIMS, 500, 1500, POINTER_ANIMS.OUT.start);
    this.stoneCycle = new IdleCycle(this, STONE_IDLE_ANIMS, 700, 2200, 0);
    this.createSceneGraph();
    this.scale.on(Phaser.Scale.Events.RESIZE, this.renderBoard, this);
    this.input.on("pointermove", this.handlePointerMove, this);
    this.input.on("pointerdown", this.handlePointerDown, this);
    this.input.on("pointerup", this.handlePointerUp, this);
    this.input.on("pointerout", this.handlePointerOut, this);
    this.renderBoard();
  }

  shutdown(): void {
    this.pointerCycle?.stop();
    this.stoneCycle?.stop();
    this.scale?.off?.(Phaser.Scale.Events.RESIZE, this.renderBoard, this);
    this.input?.off?.("pointermove", this.handlePointerMove, this);
    this.input?.off?.("pointerdown", this.handlePointerDown, this);
    this.input?.off?.("pointerup", this.handlePointerUp, this);
    this.input?.off?.("pointerout", this.handlePointerOut, this);
    this.root?.destroy(true);
    this.root = null;
    this.board = null;
    this.boardLayer = null;
    this.overlayLayer = null;
    this.pointer = null;
    this.pointerCellKey = null;
    this.pointerLayer = null;
    this.reportTouchCandidate(null);
    this.stoneSprites.clear();
    this.touchCandidate = null;
    this.touchDragOrigin = null;
  }

  setBoardState(state: BoardSceneState): void {
    const previousState = this.boardState;
    this.boardState = state;

    if (shouldStopStoneIdleCycle(previousState.status, state.status)) {
      this.stoneCycle?.stop();
    }

    if (previousState.touchCandidateResetVersion !== state.touchCandidateResetVersion) {
      this.clearTouchCandidate();
      this.hidePointer();
    }

    if (this.sys?.isActive()) {
      this.syncBoardState(previousState, true);
    }
  }

  private preloadSpritesheet(key: string, url: string): void {
    if (this.textures.exists(key)) {
      return;
    }

    this.load.spritesheet(key, url, {
      frameHeight: FRAME_SIZE,
      frameWidth: FRAME_SIZE,
    });
  }

  private ensureAnimations(): void {
    if (!this.anims.exists(STONE_ANIMS.FORM.key)) {
      this.anims.create({
        key: STONE_ANIMS.FORM.key,
        frames: this.anims.generateFrameNumbers(SPRITE.STONE, {
          start: STONE_ANIMS.FORM.start,
          end: STONE_ANIMS.FORM.end,
        }),
        frameRate: STONE_ANIMS.FORM.frameRate,
      });
    }

    for (const relax of STONE_IDLE_ANIMS) {
      if (this.anims.exists(relax.key)) {
        continue;
      }

      this.anims.create({
        key: relax.key,
        frames: this.anims.generateFrameNumbers(SPRITE.STONE, {
          start: relax.start,
          end: relax.end,
        }),
        frameRate: relax.frameRate,
      });
    }

    for (const anim of POINTER_IDLE_ANIMS) {
      if (this.anims.exists(anim.key)) {
        continue;
      }

      this.anims.create({
        key: anim.key,
        frames: this.anims.generateFrameNumbers(SPRITE.POINTER, {
          start: anim.start,
          end: anim.end,
        }),
        frameRate: anim.frameRate,
      });
    }

    for (const anim of [WARNING_ANIMS.POINTER, WARNING_ANIMS.HOVER, WARNING_ANIMS.FORBIDDEN]) {
      if (this.anims.exists(anim.key)) {
        continue;
      }

      this.anims.create({
        key: anim.key,
        frames: this.anims.generateFrameNumbers(SPRITE.WARNING, {
          start: anim.start,
          end: anim.end,
        }),
        frameRate: anim.frameRate,
        repeat: -1,
      });
    }
  }

  private createSceneGraph(): void {
    this.root = this.add.container(0, 0);
    this.boardLayer = this.add.container(0, 0);
    this.overlayLayer = this.add.container(0, 0);
    this.pointerLayer = this.add.container(0, 0);
    this.root.add([this.boardLayer, this.overlayLayer, this.pointerLayer]);
  }

  private renderBoard(): void {
    this.renderVersion += 1;
    if (!this.boardLayer || !this.overlayLayer || !this.pointerLayer) {
      return;
    }

    this.pointerCycle?.stop();
    this.stoneCycle?.stop();
    this.boardLayer.removeAll(true);
    this.overlayLayer.removeAll(true);
    this.pointerLayer.removeAll(true);
    this.stoneSprites.clear();
    this.forbiddenSprites = [];
    this.hintSprites = [];
    this.pointerCellKey = null;
    this.sequenceLabels = [];
    this.winSprites = [];

    const width = this.cameras.main.width;
    const height = this.cameras.main.height;
    const cellSize = Math.min(width / BOARD_SIZE, height / BOARD_SIZE);
    const boardHeight = BOARD_SIZE * cellSize;
    const originX = (width - (BOARD_SIZE - 1) * cellSize) / 2;
    const originY = (height - boardHeight) / 2 + cellSize / 2;

    this.currentCellSize = cellSize;
    this.board = new BoardRenderer(this, cellSize, originX, originY, this.boardLayer);
    this.board.drawBoard();
    this.pointer = this.board.createPointer(this.pointerLayer);
    this.syncBoardState(undefined, false);
  }

  private syncBoardState(previousState?: BoardSceneState, animateNewStones = true): void {
    if (!this.board) {
      return;
    }

    this.syncStoneSprites(previousState, animateNewStones);
    this.syncPointerBaseState();
    this.syncOverlaySprites();
  }

  private syncStoneSprites(previousState?: BoardSceneState, animateNewStones = true): void {
    if (!this.board) {
      return;
    }

    const renderVersion = this.renderVersion;
    const existingKeys = new Set(this.stoneSprites.keys());

    for (let row = 0; row < BOARD_SIZE; row += 1) {
      for (let col = 0; col < BOARD_SIZE; col += 1) {
        const cell = this.boardState.cells[row][col];
        const key = this.cellKey(row, col);

        if (cell === null) {
          continue;
        }

        existingKeys.delete(key);

        if (this.stoneSprites.has(key)) {
          continue;
        }

        const stone = this.board.placeStone(row, col, cell);
        this.stoneSprites.set(key, stone);

        const previousCell = previousState?.cells[row]?.[col] ?? null;
        const isNewStone = previousCell === null;

        if (shouldAnimatePlacedStone(isNewStone, animateNewStones, this.boardState.status)) {
          stone.play(STONE_ANIMS.FORM.key);
          stone.once(Phaser.Animations.Events.ANIMATION_COMPLETE, () => {
            if (
              renderVersion !== this.renderVersion ||
              !stone.active ||
              !stone.scene
            ) {
              return;
            }

            if (this.boardState.status === "playing") {
              this.stoneCycle?.start(stone);
            }
          });
        } else {
          stone.setFrame(STONE_ANIMS.STATIC.frame);
        }
      }
    }

    for (const key of existingKeys) {
      this.stoneSprites.get(key)?.destroy();
      this.stoneSprites.delete(key);
    }
  }

  private syncPointerBaseState(): void {
    if (!this.pointer) {
      return;
    }

    if (!this.boardState.interactive || this.boardState.status !== "playing") {
      this.clearTouchCandidate();
      this.hidePointer();
      return;
    }

    this.pointer
      .setScale(this.currentCellSize / FRAME_SIZE)
      .setTint(this.boardState.currentPlayer === 1 ? COLOR.STONE_BLACK : COLOR.STONE_WHITE);

    if (!this.boardState.mobileTouchPlacement) {
      this.clearTouchCandidate();
      return;
    }

    if (!this.touchCandidate || !this.board) {
      this.reportTouchCandidate(null);
      this.hidePointer();
      return;
    }

    const point = this.board.cellToPixel(this.touchCandidate.row, this.touchCandidate.col);
    const cellKey = this.cellKey(this.touchCandidate.row, this.touchCandidate.col);
    const restartPointerCycle = shouldRestartPointerCycle(
      this.pointerCellKey,
      cellKey,
      this.pointer.visible,
    );
    this.pointer
      .setPosition(point.x, point.y)
      .setVisible(true);
    this.pointerCellKey = cellKey;
    this.reportTouchCandidate(this.touchCandidate);

    if (restartPointerCycle) {
      this.pointerCycle?.start(this.pointer);
    }
  }

  private syncOverlaySprites(): void {
    if (!this.board || !this.overlayLayer) {
      return;
    }

    this.forbiddenSprites.forEach((sprite) => sprite.destroy());
    this.hintSprites.forEach((sprite) => sprite.destroy());
    this.sequenceLabels.forEach((label) => label.destroy());
    this.winSprites.forEach((sprite) => sprite.destroy());
    this.forbiddenSprites = [];
    this.hintSprites = [];
    this.sequenceLabels = [];
    this.winSprites = [];

    for (const cell of this.boardState.forbiddenMoves) {
      const point = this.board.cellToPixel(cell.row, cell.col);
      this.forbiddenSprites.push(
        this.createWarnSprite(point.x, point.y, COLOR.FORBIDDEN, WARNING_ANIMS.FORBIDDEN.key),
      );
    }

    for (const cell of this.boardState.winningMoves) {
      const point = this.board.cellToPixel(cell.row, cell.col);
      this.hintSprites.push(
        this.createWarnSprite(point.x, point.y, COLOR.WIN_MOVE, WARNING_ANIMS.POINTER.key),
      );
    }

    for (const cell of this.boardState.threatMoves) {
      const point = this.board.cellToPixel(cell.row, cell.col);
      this.hintSprites.push(
        this.createWarnSprite(point.x, point.y, COLOR.THREAT, WARNING_ANIMS.POINTER.key),
      );
    }

    if (this.boardState.showSequenceNumbers && this.boardState.status !== "playing") {
      const fontSize = Math.max(10, Math.round(this.currentCellSize * 0.25));

      for (const move of this.boardState.moves) {
        const cell = this.boardState.cells[move.row][move.col];
        if (cell === null) {
          continue;
        }

        const point = this.board.cellToPixel(move.row, move.col);
        const label = this.add.bitmapText(point.x, point.y, FONT_KEY, String(move.moveNumber), fontSize);
        label.setTint(cell === 0 ? COLOR.SEQ_ON_BLACK : COLOR.SEQ_ON_WHITE);
        label.setOrigin(0.5, 0.5);
        label.setDepth(3);
        this.overlayLayer.add(label);
        this.sequenceLabels.push(label);
      }
    }

    for (const cell of this.boardState.winningCells) {
      const point = this.board.cellToPixel(cell.row, cell.col);
      this.winSprites.push(
        this.createWarnSprite(point.x, point.y, COLOR.WIN_CELLS, WARNING_ANIMS.HOVER.key, 2.5),
      );
    }
  }

  private createWarnSprite(
    x: number,
    y: number,
    tint: number,
    animKey: string,
    depth = 0.5,
  ): Phaser.GameObjects.Sprite {
    const sprite = this.add.sprite(x, y, SPRITE.WARNING, 0);
    sprite.setScale(this.currentCellSize / FRAME_SIZE);
    sprite.setDepth(depth);
    sprite.setTint(tint);
    sprite.play({ key: animKey, repeat: -1 });
    this.overlayLayer?.add(sprite);
    return sprite;
  }

  private getPointerType(pointer: Phaser.Input.Pointer): string {
    return (pointer as { pointerType?: string }).pointerType ?? "mouse";
  }

  private reportTouchCandidate(candidate: CellPosition | null): void {
    const candidateKey = candidate ? this.cellKey(candidate.row, candidate.col) : null;
    const canPlace = canPlaceTouchCandidate(
      this.boardState.cells,
      this.boardState.forbiddenMoves,
      candidate,
    );

    if (
      candidateKey === this.reportedTouchCandidateKey &&
      canPlace === this.reportedTouchCanPlace
    ) {
      return;
    }

    this.reportedTouchCandidateKey = candidateKey;
    this.reportedTouchCanPlace = canPlace;
    this.boardState.onTouchCandidateChange(candidate, canPlace);
  }

  private setTouchCandidate(candidate: CellPosition | null): void {
    if (!this.boardState.mobileTouchPlacement) {
      return;
    }

    this.touchCandidate = candidate;

    if (!candidate || !this.board || !this.pointer) {
      this.hidePointer();
      this.reportTouchCandidate(null);
      return;
    }

    const point = this.board.cellToPixel(candidate.row, candidate.col);
    const cellKey = this.cellKey(candidate.row, candidate.col);
    const restartPointerCycle = shouldRestartPointerCycle(
      this.pointerCellKey,
      cellKey,
      this.pointer.visible,
    );
    this.pointer
      .setPosition(point.x, point.y)
      .setTint(this.boardState.currentPlayer === 1 ? COLOR.STONE_BLACK : COLOR.STONE_WHITE)
      .setVisible(true);
    this.pointerCellKey = cellKey;
    this.reportTouchCandidate(candidate);

    if (restartPointerCycle) {
      this.pointerCycle?.start(this.pointer);
    }
  }

  private clearTouchCandidate(): void {
    this.touchCandidate = null;
    this.touchDragOrigin = null;
    this.reportTouchCandidate(null);
  }

  private handlePointerMove(pointer: Phaser.Input.Pointer): void {
    if (!this.boardState.interactive || this.boardState.status !== "playing" || !this.board || !this.pointer) {
      this.hidePointer();
      return;
    }

    if (this.boardState.mobileTouchPlacement) {
      if (!this.touchDragOrigin) {
        return;
      }

      this.setTouchCandidate(
        moveTouchCandidateFromDrag(
          this.touchDragOrigin.candidate,
          pointer.x - this.touchDragOrigin.x,
          pointer.y - this.touchDragOrigin.y,
          this.currentCellSize,
        ),
      );
      return;
    }

    const cell = this.board.pixelToCell(pointer.x, pointer.y);

    if (!cell || this.boardState.cells[cell.row][cell.col] !== null) {
      this.hidePointer();
      return;
    }

    const point = this.board.cellToPixel(cell.row, cell.col);
    const cellKey = this.cellKey(cell.row, cell.col);
    const restartPointerCycle = shouldRestartPointerCycle(
      this.pointerCellKey,
      cellKey,
      this.pointer.visible,
    );
    this.pointer
      .setPosition(point.x, point.y)
      .setTint(this.boardState.currentPlayer === 1 ? COLOR.STONE_BLACK : COLOR.STONE_WHITE)
      .setVisible(true);
    this.pointerCellKey = cellKey;

    if (this.getPointerType(pointer) === "mouse" && restartPointerCycle) {
      this.pointerCycle?.start(this.pointer);
    }
  }

  private handlePointerDown(pointer: Phaser.Input.Pointer): void {
    if (
      !this.boardState.interactive ||
      this.boardState.status !== "playing" ||
      !this.board ||
      !this.pointer
    ) {
      return;
    }

    if (this.boardState.mobileTouchPlacement) {
      const originCandidate = this.touchCandidate ?? this.board.pixelToCell(pointer.x, pointer.y);
      if (!originCandidate) {
        return;
      }

      this.touchDragOrigin = {
        x: pointer.x,
        y: pointer.y,
        candidate: originCandidate,
      };
      this.setTouchCandidate(originCandidate);
      return;
    }

    if (this.getPointerType(pointer) !== "touch") {
      return;
    }

    const cell = this.board.pixelToCell(pointer.x, pointer.y);
    if (!cell || this.boardState.cells[cell.row][cell.col] !== null) {
      return;
    }

    const point = this.board.cellToPixel(cell.row, cell.col);
    this.pointer
      .setPosition(point.x, point.y)
      .setTint(this.boardState.currentPlayer === 1 ? COLOR.STONE_BLACK : COLOR.STONE_WHITE)
      .setVisible(true);
    this.pointerCellKey = this.cellKey(cell.row, cell.col);
  }

  private handlePointerUp(pointer: Phaser.Input.Pointer): void {
    if (!this.board) {
      return;
    }

    const cell = this.board.pixelToCell(pointer.x, pointer.y);
    if (this.boardState.status !== "playing") {
      if (cell) {
        this.boardState.onAdvanceRound();
      }
      return;
    }

    if (!this.boardState.interactive) {
      return;
    }

    if (this.boardState.mobileTouchPlacement) {
      this.touchDragOrigin = null;
      return;
    }

    if (this.getPointerType(pointer) === "touch") {
      if (!this.pointer?.visible) {
        return;
      }

      const pointerCell = this.board.pixelToCell(this.pointer.x, this.pointer.y);
      if (
        pointerCell &&
        canPlaceTouchCandidate(this.boardState.cells, this.boardState.forbiddenMoves, pointerCell)
      ) {
        this.boardState.onPlace(pointerCell.row, pointerCell.col);
      }
      this.hidePointer();
      return;
    }

    if (!cell || this.boardState.cells[cell.row][cell.col] !== null) {
      return;
    }

    this.boardState.onPlace(cell.row, cell.col);
  }

  private handlePointerOut(pointer: Phaser.Input.Pointer): void {
    if (this.boardState.mobileTouchPlacement) {
      return;
    }

    this.hidePointer();
  }

  private hidePointer(): void {
    this.pointerCycle?.stop();
    this.pointerCellKey = null;
    if (!this.pointer) {
      return;
    }

    this.pointer.stop();
    this.pointer.setVisible(false);
  }

  private cellKey(row: number, col: number): string {
    return `${row},${col}`;
  }
}
