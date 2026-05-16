import * as Phaser from "phaser";

import {
  BOARD_RENDER_DEPTHS,
  BOARD_RENDER_LAYER_ORDER,
  BOARD_SIZE,
  COLOR,
  FRAME_SIZE,
  HOVER_ANIMS,
  POINTER_ANIMS,
  SPRITE,
  STONE_ANIMS,
  TRANSFORM_ANIMS,
  WARNING_ANIMS,
} from "./constants";
import { BoardRenderer } from "./board_renderer";
import { SEQUENCE_FONT_FAMILY } from "./sequence_font";
import {
  canPlaceTouchCandidate,
  moveTouchCandidateFromPointerMove,
  pointerCueForCandidate,
  resetSpriteToFrame,
  sequenceNumberFontSize,
  sequenceNumberPosition,
  shouldAnimatePlacedStone,
  shouldRenderStandaloneForbiddenOverlay,
  shouldRestartPointerCycle,
  shouldSyncOverlaySprites,
  shouldStopStoneCycleBeforeStoneRemoval,
  shouldStopStoneIdleCycle,
  usesTouchCandidate,
  usesTouchpadDrag,
  warningAnimationForOverlay,
  warningSpriteForOverlay,
} from "./board_scene_logic";

import type { CellPosition, CellStone, MatchMove, MatchStatus } from "../game/types";
import type { BoardTouchControlMode, PointerCue } from "./board_scene_logic";

const STONE_IDLE_ANIMS = [
  STONE_ANIMS.IDLE_1,
  STONE_ANIMS.IDLE_2,
  STONE_ANIMS.IDLE_3,
  STONE_ANIMS.IDLE_4,
] as const;
const ASSET_URLS = {
  hover: new URL("../../assets/sprites/hover.png", import.meta.url).toString(),
  pointer: new URL("../../assets/sprites/pointer.png", import.meta.url).toString(),
  stone: new URL("../../assets/sprites/stone.png", import.meta.url).toString(),
  transform: new URL("../../assets/sprites/transform.png", import.meta.url).toString(),
  warning: new URL("../../assets/sprites/warning.png", import.meta.url).toString(),
} as const;

type AnimationRange = {
  end: number;
  frameRate: number;
  key: string;
  start: number;
};

type SpriteFrameRef = {
  frame: number;
  texture: string;
};

type SequenceStep =
  | { kind: "animation"; key: string }
  | { frame: SpriteFrameRef; kind: "delay"; max: number; min: number }
  | { keys: readonly string[]; kind: "randomAnimation" };

const STONE_STATIC_FRAME: SpriteFrameRef = { texture: SPRITE.STONE, frame: STONE_ANIMS.STATIC.frame };
const POINTER_STATIC_FRAME: SpriteFrameRef = { texture: SPRITE.POINTER, frame: POINTER_ANIMS.STATIC.frame };

const POINTER_NORMAL_STEPS: readonly SequenceStep[] = [
  { kind: "animation", key: POINTER_ANIMS.OPEN.key },
  { kind: "delay", frame: POINTER_STATIC_FRAME, min: 450, max: 1200 },
] as const;

const POINTER_PREFERRED_STEPS: readonly SequenceStep[] = [
  { kind: "animation", key: POINTER_ANIMS.PREFERRED.key },
  { kind: "delay", frame: POINTER_STATIC_FRAME, min: 450, max: 1200 },
] as const;

const POINTER_BLOCKED_STEPS: readonly SequenceStep[] = [
  { kind: "animation", key: POINTER_ANIMS.BLOCKED.key },
  { kind: "delay", frame: POINTER_STATIC_FRAME, min: 450, max: 1200 },
] as const;

const FORBIDDEN_STEPS: readonly SequenceStep[] = [
  { kind: "animation", key: WARNING_ANIMS.FORBIDDEN_OUT.key },
  { kind: "animation", key: WARNING_ANIMS.FORBIDDEN_IN.key },
] as const;

function cssColor(color: number): string {
  return `#${color.toString(16).padStart(6, "0")}`;
}

class RandomAnimationCycle {
  private readonly scene: Phaser.Scene;
  private readonly anims: readonly { key: string }[];
  private readonly delayMax: number;
  private readonly delayMin: number;
  private readonly resetFrame: SpriteFrameRef | null;
  private active = false;
  private sprite: Phaser.GameObjects.Sprite | null = null;
  private timer: Phaser.Time.TimerEvent | null = null;

  constructor(
    scene: Phaser.Scene,
    anims: readonly { key: string }[],
    delayMin: number,
    delayMax: number,
    resetFrame: SpriteFrameRef | null,
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
      resetSpriteToFrame(this.sprite, this.resetFrame);
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
          resetSpriteToFrame(this.sprite, this.resetFrame);
        }
        this.scheduleNext();
      });
    });
  }
}

class SequenceAnimationCycle {
  private readonly scene: Phaser.Scene;
  private readonly steps: readonly SequenceStep[];
  private active = false;
  private sprite: Phaser.GameObjects.Sprite | null = null;
  private stepIndex = 0;
  private timer: Phaser.Time.TimerEvent | null = null;

  constructor(scene: Phaser.Scene, steps: readonly SequenceStep[]) {
    this.scene = scene;
    this.steps = steps;
  }

  start(sprite: Phaser.GameObjects.Sprite): void {
    this.stop();
    this.active = true;
    this.sprite = sprite;
    this.stepIndex = 0;
    this.runNextStep();
  }

  stop(): void {
    this.active = false;
    this.timer?.destroy();
    this.timer = null;

    if (!this.sprite) {
      return;
    }

    this.sprite.removeAllListeners(Phaser.Animations.Events.ANIMATION_COMPLETE);
    this.sprite = null;
  }

  private runNextStep(): void {
    const sprite = this.sprite;
    if (!this.active || !sprite || !sprite.active || !sprite.scene || this.steps.length === 0) {
      return;
    }

    const step = this.steps[this.stepIndex];
    this.stepIndex = (this.stepIndex + 1) % this.steps.length;

    if (step.kind === "delay") {
      resetSpriteToFrame(sprite, step.frame);
      const delay = step.min + Math.random() * (step.max - step.min);
      this.timer = this.scene.time.delayedCall(delay, () => {
        this.runNextStep();
      });
      return;
    }

    if (step.kind === "randomAnimation") {
      const key = step.keys[Math.floor(Math.random() * step.keys.length)];
      sprite.play(key);
      sprite.once(Phaser.Animations.Events.ANIMATION_COMPLETE, () => {
        this.runNextStep();
      });
      return;
    }

    sprite.play(step.key);
    sprite.once(Phaser.Animations.Events.ANIMATION_COMPLETE, () => {
      this.runNextStep();
    });
  }
}

export interface BoardSceneState {
  cells: CellStone[][];
  counterThreatMoves: CellPosition[];
  currentPlayer: 1 | 2;
  forbiddenMoves: CellPosition[];
  imminentThreatMoves: CellPosition[];
  interactive: boolean;
  lastMove: CellPosition | null;
  moves: MatchMove[];
  onAdvanceRound: () => void;
  onPlace: (row: number, col: number) => void;
  onTouchCandidateChange: (candidate: CellPosition | null, canPlace: boolean) => void;
  touchControlMode: BoardTouchControlMode;
  touchCandidateResetVersion: number;
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
  counterThreatMoves: [],
  currentPlayer: 1,
  forbiddenMoves: [],
  interactive: false,
  lastMove: null,
  moves: [],
  onAdvanceRound: () => undefined,
  onPlace: () => undefined,
  onTouchCandidateChange: () => undefined,
  touchControlMode: "none",
  touchCandidateResetVersion: 0,
  showSequenceNumbers: false,
  status: "playing",
  imminentThreatMoves: [],
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
  private hoverLayer: Phaser.GameObjects.Container | null = null;
  private pointer: Phaser.GameObjects.Sprite | null = null;
  private pointerCellKey: string | null = null;
  private pointerCycle: SequenceAnimationCycle | null = null;
  private pointerLayer: Phaser.GameObjects.Container | null = null;
  private renderVersion = 0;
  private reportedTouchCandidateKey: string | null = null;
  private reportedTouchCanPlace = false;
  private root: Phaser.GameObjects.Container | null = null;
  private sequenceLayer: Phaser.GameObjects.Container | null = null;
  private sequenceLabels: Phaser.GameObjects.Text[] = [];
  private stoneCycle: RandomAnimationCycle | null = null;
  private stoneLayer: Phaser.GameObjects.Container | null = null;
  private stoneSprites: Map<string, Phaser.GameObjects.Sprite> = new Map();
  private touchCandidate: CellPosition | null = null;
  private touchDragOrigin: { x: number; y: number; candidate: CellPosition } | null = null;
  private forbiddenCycles: SequenceAnimationCycle[] = [];
  private warningLayer: Phaser.GameObjects.Container | null = null;
  private winSprites: Phaser.GameObjects.Sprite[] = [];

  constructor() {
    super({ key: "BoardScene" });
  }

  preload(): void {
    this.preloadSpritesheet(SPRITE.STONE, ASSET_URLS.stone);
    this.preloadSpritesheet(SPRITE.POINTER, ASSET_URLS.pointer);
    this.preloadSpritesheet(SPRITE.HOVER, ASSET_URLS.hover);
    this.preloadSpritesheet(SPRITE.WARNING, ASSET_URLS.warning);
    this.preloadSpritesheet(SPRITE.TRANSFORM, ASSET_URLS.transform);
  }

  create(): void {
    this.ensureAnimations();
    this.stoneCycle = new RandomAnimationCycle(this, STONE_IDLE_ANIMS, 700, 2200, STONE_STATIC_FRAME);
    this.createSceneGraph();
    this.scale.on(Phaser.Scale.Events.RESIZE, this.renderBoard, this);
    this.input.on("pointermove", this.handlePointerMove, this);
    this.input.on("pointerdown", this.handlePointerDown, this);
    this.input.on("pointerup", this.handlePointerUp, this);
    this.input.on("pointerout", this.handlePointerOut, this);
    this.renderBoard();
  }

  shutdown(): void {
    this.stopPointerCycle();
    this.stoneCycle?.stop();
    this.stopForbiddenCycles();
    this.scale?.off?.(Phaser.Scale.Events.RESIZE, this.renderBoard, this);
    this.input?.off?.("pointermove", this.handlePointerMove, this);
    this.input?.off?.("pointerdown", this.handlePointerDown, this);
    this.input?.off?.("pointerup", this.handlePointerUp, this);
    this.input?.off?.("pointerout", this.handlePointerOut, this);
    this.root?.destroy(true);
    this.root = null;
    this.board = null;
    this.boardLayer = null;
    this.hoverLayer = null;
    this.pointer = null;
    this.pointerCellKey = null;
    this.pointerLayer = null;
    this.sequenceLayer = null;
    this.stoneLayer = null;
    this.warningLayer = null;
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
    this.ensureRangeAnimation(SPRITE.TRANSFORM, TRANSFORM_ANIMS.FORM);
    this.ensureRangeAnimation(SPRITE.STONE, STONE_ANIMS.DESTROY);

    for (const idle of STONE_IDLE_ANIMS) {
      this.ensureRangeAnimation(SPRITE.STONE, idle);
    }

    for (const anim of [
      POINTER_ANIMS.BLOCKED,
      POINTER_ANIMS.OPEN,
      POINTER_ANIMS.PREFERRED,
    ]) {
      this.ensureRangeAnimation(SPRITE.POINTER, anim);
    }

    this.ensureRangeAnimation(SPRITE.HOVER, HOVER_ANIMS.HOVER);

    for (const anim of [
      WARNING_ANIMS.WARNING,
      WARNING_ANIMS.WARNING_ON_FORBIDDEN,
      WARNING_ANIMS.FORBIDDEN_OUT,
      WARNING_ANIMS.FORBIDDEN_IN,
      WARNING_ANIMS.HIGHLIGHT,
    ]) {
      this.ensureRangeAnimation(SPRITE.WARNING, anim);
    }
  }

  private ensureRangeAnimation(sprite: string, anim: AnimationRange, repeat = 0): void {
    if (this.anims.exists(anim.key)) {
      return;
    }

    this.anims.create({
      key: anim.key,
      frames: this.anims.generateFrameNumbers(sprite, {
        start: anim.start,
        end: anim.end,
      }),
      frameRate: anim.frameRate,
      repeat,
    });
  }

  private createSceneGraph(): void {
    this.root = this.add.container(0, 0);
    this.boardLayer = this.add.container(0, 0);
    this.warningLayer = this.add.container(0, 0);
    this.pointerLayer = this.add.container(0, 0);
    this.stoneLayer = this.add.container(0, 0);
    this.sequenceLayer = this.add.container(0, 0);
    this.hoverLayer = this.add.container(0, 0);
    const layers = {
      BOARD: this.boardLayer,
      HOVER: this.hoverLayer,
      POINTER: this.pointerLayer,
      SEQUENCE_NUMBER: this.sequenceLayer,
      STONE: this.stoneLayer,
      WARNING: this.warningLayer,
    } satisfies Record<(typeof BOARD_RENDER_LAYER_ORDER)[number], Phaser.GameObjects.Container>;
    this.root.add(BOARD_RENDER_LAYER_ORDER.map((layer) => layers[layer]));
  }

  private renderBoard(): void {
    this.renderVersion += 1;
    if (
      !this.boardLayer ||
      !this.warningLayer ||
      !this.pointerLayer ||
      !this.stoneLayer ||
      !this.sequenceLayer ||
      !this.hoverLayer
    ) {
      return;
    }

    this.stopPointerCycle();
    this.stoneCycle?.stop();
    this.stopForbiddenCycles();
    this.boardLayer.removeAll(true);
    this.warningLayer.removeAll(true);
    this.pointerLayer.removeAll(true);
    this.stoneLayer.removeAll(true);
    this.sequenceLayer.removeAll(true);
    this.hoverLayer.removeAll(true);
    this.stoneSprites.clear();
    this.forbiddenSprites = [];
    this.forbiddenCycles = [];
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
    if (!this.board || !this.stoneLayer) {
      return;
    }

    this.syncStoneSprites(previousState, animateNewStones);
    this.syncPointerBaseState();
    if (shouldSyncOverlaySprites(previousState, this.boardState)) {
      this.syncOverlaySprites();
    }
  }

  private syncStoneSprites(previousState?: BoardSceneState, animateNewStones = true): void {
    if (!this.board || !this.stoneLayer) {
      return;
    }

    const stoneLayer = this.stoneLayer;
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

        const stone = this.board.placeStone(row, col, cell, stoneLayer);
        this.stoneSprites.set(key, stone);

        const previousCell = previousState?.cells[row]?.[col] ?? null;
        const isNewStone = previousCell === null;

        if (shouldAnimatePlacedStone(isNewStone, animateNewStones, this.boardState.status)) {
          stone.play(TRANSFORM_ANIMS.FORM.key);
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

    if (shouldStopStoneCycleBeforeStoneRemoval(existingKeys.size)) {
      this.stoneCycle?.stop();
    }

    for (const key of existingKeys) {
      const stone = this.stoneSprites.get(key);
      if (stone?.active && stone.scene) {
        stone.removeAllListeners(Phaser.Animations.Events.ANIMATION_COMPLETE);
        stone.play(STONE_ANIMS.DESTROY.key);
        stone.once(Phaser.Animations.Events.ANIMATION_COMPLETE, () => {
          stone.destroy();
        });
      } else {
        stone?.destroy();
      }
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

    if (!usesTouchCandidate(this.boardState.touchControlMode)) {
      this.clearTouchCandidate();
      return;
    }

    if (!this.touchCandidate || !this.board) {
      this.reportTouchCandidate(null);
      this.hidePointer();
      return;
    }

    this.showPointerAtCandidate(this.touchCandidate, true, true);
  }

  private syncOverlaySprites(): void {
    if (!this.board || !this.warningLayer || !this.sequenceLayer || !this.hoverLayer) {
      return;
    }

    this.stopForbiddenCycles();
    this.forbiddenSprites.forEach((sprite) => sprite.destroy());
    this.hintSprites.forEach((sprite) => sprite.destroy());
    this.sequenceLabels.forEach((label) => label.destroy());
    this.winSprites.forEach((sprite) => sprite.destroy());
    this.forbiddenSprites = [];
    this.hintSprites = [];
    this.sequenceLabels = [];
    this.winSprites = [];

    const forbiddenKeys = new Set(
      this.boardState.forbiddenMoves.map((cell) => this.cellKey(cell.row, cell.col)),
    );

    for (const cell of this.boardState.forbiddenMoves) {
      if (!shouldRenderStandaloneForbiddenOverlay(cell, this.boardState.threatMoves)) {
        continue;
      }

      const point = this.board.cellToPixel(cell.row, cell.col);
      const sprite = this.createForbiddenSprite(point.x, point.y);
      this.forbiddenSprites.push(sprite);
    }

    for (const cell of this.boardState.winningMoves) {
      const point = this.board.cellToPixel(cell.row, cell.col);
      this.hintSprites.push(
        this.createWarnSprite(point.x, point.y, COLOR.WIN_MOVE, warningAnimationForOverlay("winningMove")),
      );
    }

    for (const cell of this.boardState.threatMoves) {
      const point = this.board.cellToPixel(cell.row, cell.col);
      const isForbidden = forbiddenKeys.has(this.cellKey(cell.row, cell.col));
      this.hintSprites.push(
        this.createWarnSprite(
          point.x,
          point.y,
          COLOR.THREAT,
          warningAnimationForOverlay("threatMove", isForbidden),
        ),
      );
    }

    for (const cell of this.boardState.imminentThreatMoves) {
      const point = this.board.cellToPixel(cell.row, cell.col);
      this.hintSprites.push(
        this.createWarnSprite(
          point.x,
          point.y,
          COLOR.IMMINENT_THREAT,
          warningAnimationForOverlay("imminentThreatMove"),
        ),
      );
    }

    for (const cell of this.boardState.counterThreatMoves) {
      const point = this.board.cellToPixel(cell.row, cell.col);
      this.hintSprites.push(
        this.createWarnSprite(
          point.x,
          point.y,
          COLOR.COUNTER_THREAT,
          warningAnimationForOverlay("counterThreatMove"),
        ),
      );
    }

    if (this.boardState.showSequenceNumbers && this.boardState.status !== "playing") {
      for (const move of this.boardState.moves) {
        const cell = this.boardState.cells[move.row][move.col];
        if (cell === null) {
          continue;
        }

        const point = this.board.cellToPixel(move.row, move.col);
        const position = sequenceNumberPosition(point.x, point.y);
        const label = this.add.text(position.x, position.y, String(move.moveNumber), {
          color: cssColor(cell === 0 ? COLOR.SEQ_ON_BLACK : COLOR.SEQ_ON_WHITE),
          fontFamily: SEQUENCE_FONT_FAMILY,
          fontSize: `${sequenceNumberFontSize(this.currentCellSize)}px`,
        });
        label.setOrigin(0.5, 0.5);
        label.setDepth(BOARD_RENDER_DEPTHS.SEQUENCE_NUMBER);
        this.sequenceLayer.add(label);
        this.sequenceLabels.push(label);
      }
    }

    for (const cell of this.boardState.winningCells) {
      const point = this.board.cellToPixel(cell.row, cell.col);
      this.winSprites.push(
        this.createWarnSprite(
          point.x,
          point.y,
          COLOR.WIN_CELLS,
          warningAnimationForOverlay("winningLine"),
          BOARD_RENDER_DEPTHS.WARNING_HOVER,
          warningSpriteForOverlay("winningLine"),
        ),
      );
    }
  }

  private createWarnSprite(
    x: number,
    y: number,
    tint: number,
    animKey: string,
    depth: number = BOARD_RENDER_DEPTHS.WARNING_SURFACE,
    texture: string = SPRITE.WARNING,
  ): Phaser.GameObjects.Sprite {
    const sprite = this.add.sprite(x, y, texture, 0);
    sprite.setScale(this.currentCellSize / FRAME_SIZE);
    sprite.setDepth(depth);
    sprite.setTint(tint);
    sprite.play({ key: animKey, repeat: -1 });
    const layer = texture === SPRITE.HOVER ? this.hoverLayer : this.warningLayer;
    layer?.add(sprite);
    return sprite;
  }

  private createForbiddenSprite(x: number, y: number): Phaser.GameObjects.Sprite {
    const sprite = this.add.sprite(x, y, SPRITE.WARNING, WARNING_ANIMS.FORBIDDEN_OUT.start);
    sprite.setScale(this.currentCellSize / FRAME_SIZE);
    sprite.setDepth(BOARD_RENDER_DEPTHS.WARNING_BLOCKED);
    sprite.setTint(COLOR.FORBIDDEN);
    this.warningLayer?.add(sprite);

    const cycle = new SequenceAnimationCycle(this, FORBIDDEN_STEPS);
    cycle.start(sprite);
    this.forbiddenCycles.push(cycle);
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
    if (!usesTouchCandidate(this.boardState.touchControlMode)) {
      return;
    }

    this.touchCandidate = candidate;

    if (!candidate || !this.board || !this.pointer) {
      this.hidePointer();
      this.reportTouchCandidate(null);
      return;
    }

    this.showPointerAtCandidate(candidate, true, true);
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

    if (usesTouchCandidate(this.boardState.touchControlMode)) {
      const pointerCell = this.board.pixelToCell(pointer.x, pointer.y);

      if (usesTouchpadDrag(this.boardState.touchControlMode) && !this.touchDragOrigin) {
        return;
      }

      const origin = this.touchDragOrigin?.candidate ?? pointerCell;
      if (!origin) {
        return;
      }

      this.setTouchCandidate(
        moveTouchCandidateFromPointerMove(
          this.boardState.touchControlMode,
          origin,
          pointerCell,
          pointer.x - (this.touchDragOrigin?.x ?? pointer.x),
          pointer.y - (this.touchDragOrigin?.y ?? pointer.y),
          this.currentCellSize,
        ),
      );
      return;
    }

    this.showPointerAtCandidate(
      this.board.pixelToCell(pointer.x, pointer.y),
      false,
      this.getPointerType(pointer) === "mouse",
    );
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

    if (usesTouchCandidate(this.boardState.touchControlMode)) {
      const tappedCandidate = this.board.pixelToCell(pointer.x, pointer.y);

      if (!usesTouchpadDrag(this.boardState.touchControlMode)) {
        this.setTouchCandidate(tappedCandidate);
        return;
      }

      const originCandidate = this.touchCandidate ?? tappedCandidate;
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

    this.showPointerAtCandidate(this.board.pixelToCell(pointer.x, pointer.y), false, false);
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

    if (usesTouchCandidate(this.boardState.touchControlMode)) {
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

    if (
      !cell ||
      !canPlaceTouchCandidate(this.boardState.cells, this.boardState.forbiddenMoves, cell)
    ) {
      return;
    }

    this.boardState.onPlace(cell.row, cell.col);
  }

  private handlePointerOut(pointer: Phaser.Input.Pointer): void {
    if (usesTouchCandidate(this.boardState.touchControlMode)) {
      return;
    }

    this.hidePointer();
  }

  private showPointerAtCandidate(
    candidate: CellPosition | null,
    showBlockedOccupied: boolean,
    animateCycle: boolean,
  ): void {
    if (!candidate || !this.board || !this.pointer) {
      this.hidePointer();
      this.reportTouchCandidate(null);
      return;
    }

    const cue = pointerCueForCandidate(
      this.boardState.cells,
      this.boardState.forbiddenMoves,
      [
        ...this.boardState.winningMoves,
        ...this.boardState.threatMoves,
        ...this.boardState.imminentThreatMoves,
        ...this.boardState.counterThreatMoves,
      ],
      candidate,
      showBlockedOccupied,
    );

    if (cue === "hidden") {
      this.hidePointer();
      return;
    }

    const point = this.board.cellToPixel(candidate.row, candidate.col);
    const cellKey = `${this.cellKey(candidate.row, candidate.col)}:${cue}`;
    const restartPointerCycle = shouldRestartPointerCycle(
      this.pointerCellKey,
      cellKey,
      this.pointer.visible,
    );

    this.pointer
      .setPosition(point.x, point.y)
      .setScale(this.currentCellSize / FRAME_SIZE)
      .setTint(this.boardState.currentPlayer === 1 ? COLOR.STONE_BLACK : COLOR.STONE_WHITE)
      .setVisible(true);
    this.pointerCellKey = cellKey;
    if (usesTouchCandidate(this.boardState.touchControlMode)) {
      this.reportTouchCandidate(candidate);
    }

    if (!animateCycle) {
      this.stopPointerCycle();
      resetSpriteToFrame(this.pointer, POINTER_STATIC_FRAME);
      return;
    }

    if (restartPointerCycle) {
      this.startPointerCycle(cue);
    }
  }

  private startPointerCycle(cue: Exclude<PointerCue, "hidden">): void {
    if (!this.pointer) {
      return;
    }

    this.stopPointerCycle();
    this.pointerCycle = new SequenceAnimationCycle(
      this,
      this.pointerStepsForCue(cue),
    );
    this.pointerCycle.start(this.pointer);
  }

  private pointerStepsForCue(cue: Exclude<PointerCue, "hidden">): readonly SequenceStep[] {
    switch (cue) {
      case "blocked":
        return POINTER_BLOCKED_STEPS;
      case "preferred":
        return POINTER_PREFERRED_STEPS;
      case "normal":
        return POINTER_NORMAL_STEPS;
    }
  }

  private stopPointerCycle(): void {
    this.pointerCycle?.stop();
    this.pointerCycle = null;
  }

  private stopForbiddenCycles(): void {
    this.forbiddenCycles.forEach((cycle) => {
      cycle.stop();
    });
    this.forbiddenCycles = [];
  }

  private hidePointer(): void {
    this.stopPointerCycle();
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
