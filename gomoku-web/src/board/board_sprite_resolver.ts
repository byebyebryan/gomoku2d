import type * as Phaser from "phaser";

import { FRAME_SIZE } from "./constants";

const ANIMATION_COMPLETE_EVENT = "animationcomplete";
const GAME_OBJECT_DESTROY_EVENT = "destroy";

export type BoardSpriteFrame = {
  frame: string | number;
  texture: string;
};

export type BoardAnimationOptions = Omit<Phaser.Types.Animations.PlayAnimationConfig, "key">;

type SpriteState = {
  animationKey: string | null;
  animationOptions: BoardAnimationOptions;
  completionDisposers: Set<() => void>;
  color: number;
  frame: string | number;
  texture: string;
};

type CanvasFactory = (width: number, height: number) => HTMLCanvasElement;

function createCanvas(width: number, height: number): HTMLCanvasElement {
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  return canvas;
}

function normalizedColor(color: number): number {
  return color & 0xffffff;
}

function colorSuffix(color: number): string {
  return normalizedColor(color).toString(16).padStart(6, "0");
}

export function coloredTextureKey(textureKey: string, color: number): string {
  const normalized = normalizedColor(color);
  return normalized === 0xffffff ? textureKey : `__board_tint__${textureKey}__${colorSuffix(normalized)}`;
}

export function coloredAnimationKey(animationKey: string, color: number): string {
  const normalized = normalizedColor(color);
  return normalized === 0xffffff ? animationKey : `__board_tint__${animationKey}__${colorSuffix(normalized)}`;
}

export function multiplyTintRgbaPixels(source: Uint8ClampedArray, color: number): Uint8ClampedArray {
  const tinted = new Uint8ClampedArray(source);
  const normalized = normalizedColor(color);
  const red = (normalized >> 16) & 0xff;
  const green = (normalized >> 8) & 0xff;
  const blue = normalized & 0xff;

  for (let index = 0; index < tinted.length; index += 4) {
    tinted[index] = Math.round((source[index]! * red) / 0xff);
    tinted[index + 1] = Math.round((source[index + 1]! * green) / 0xff);
    tinted[index + 2] = Math.round((source[index + 2]! * blue) / 0xff);
    tinted[index + 3] = source[index + 3]!;
  }

  return tinted;
}

/**
 * Resolves board-local tinted sheets and animations while keeping the source
 * assets as the only input to every tint operation. Phaser's texture and
 * animation managers own the caches, so each cache disappears with its game.
 */
export class BoardSpriteResolver {
  private readonly scene: Phaser.Scene;
  private readonly canvasFactory: CanvasFactory;
  private readonly spriteStates = new WeakMap<Phaser.GameObjects.Sprite, SpriteState>();

  constructor(scene: Phaser.Scene, canvasFactory: CanvasFactory = createCanvas) {
    this.scene = scene;
    this.canvasFactory = canvasFactory;
  }

  createSprite(
    x: number,
    y: number,
    texture: string,
    frame: number,
    color: number,
  ): Phaser.GameObjects.Sprite {
    const sprite = this.scene.add.sprite(x, y, this.resolveTexture(texture, color), frame);
    this.spriteStates.set(sprite, {
      animationKey: null,
      animationOptions: {},
      completionDisposers: new Set(),
      color: normalizedColor(color),
      frame,
      texture,
    });
    sprite.once(GAME_OBJECT_DESTROY_EVENT, () => this.clearCompletionListeners(sprite));
    return sprite;
  }

  resolveTexture(texture: string, color: number): string {
    const key = coloredTextureKey(texture, color);
    if (key === texture || this.scene.textures.exists(key)) {
      return key;
    }

    const sourceTexture = this.scene.textures.get(texture);
    const source = sourceTexture.getSourceImage();
    const sourceSize = sourceTexture.source[0];
    if (!sourceSize || sourceSize.width <= 0 || sourceSize.height <= 0) {
      throw new Error(`Cannot tint board texture "${texture}": its source has no dimensions.`);
    }

    const canvas = this.canvasFactory(sourceSize.width, sourceSize.height);
    const context = canvas.getContext("2d", { willReadFrequently: true });
    if (!context) {
      throw new Error(`Cannot tint board texture "${texture}": a 2D canvas context is unavailable.`);
    }

    context.drawImage(source as CanvasImageSource, 0, 0, sourceSize.width, sourceSize.height);
    const image = context.getImageData(0, 0, sourceSize.width, sourceSize.height);
    image.data.set(multiplyTintRgbaPixels(image.data, color));
    context.putImageData(image, 0, 0);

    const canvasTexture = this.scene.textures.addCanvas(key, canvas);
    if (!canvasTexture) {
      throw new Error(`Cannot cache tinted board texture "${key}".`);
    }

    const sheetTexture = this.scene.textures.addSpriteSheet("", canvasTexture, {
      frameHeight: FRAME_SIZE,
      frameWidth: FRAME_SIZE,
    });
    if (!sheetTexture) {
      this.scene.textures.remove(key);
      throw new Error(`Cannot add sprite-sheet frames to tinted board texture "${key}".`);
    }

    return key;
  }

  resolveAnimation(animationKey: string, color: number): string {
    const key = coloredAnimationKey(animationKey, color);
    if (key === animationKey || this.scene.anims.exists(key)) {
      return key;
    }

    const sourceAnimation = this.scene.anims.get(animationKey);
    if (!sourceAnimation) {
      throw new Error(`Cannot tint board animation "${animationKey}": the source animation is missing.`);
    }

    const sourceData = sourceAnimation.toJSON();
    const frames = sourceData.frames.map((frame) => ({
      ...frame,
      key: this.resolveTexture(frame.key, color),
    }));
    const animation = this.scene.anims.create({
      ...sourceData,
      frames,
      key,
    });
    if (!animation) {
      throw new Error(`Cannot cache tinted board animation "${key}".`);
    }

    return key;
  }

  play(
    sprite: Phaser.GameObjects.Sprite,
    animationKey: string,
    options: BoardAnimationOptions = {},
    startFrame?: number,
  ): Phaser.GameObjects.Sprite {
    const state = this.stateFor(sprite);
    return this.startAnimation(sprite, state, animationKey, options, startFrame, false);
  }

  private startAnimation(
    sprite: Phaser.GameObjects.Sprite,
    state: SpriteState,
    animationKey: string,
    options: BoardAnimationOptions,
    startFrame: number | undefined,
    preserveCompletionListeners: boolean,
  ): Phaser.GameObjects.Sprite {
    if (!preserveCompletionListeners) {
      this.clearCompletionListeners(sprite);
    }
    const resolvedKey = this.resolveAnimation(animationKey, state.color);
    state.animationKey = animationKey;
    state.animationOptions = { ...options };
    sprite.play({ ...options, key: resolvedKey, ...(startFrame === undefined ? {} : { startFrame }) });
    return sprite;
  }

  setColor(sprite: Phaser.GameObjects.Sprite, color: number): Phaser.GameObjects.Sprite {
    const state = this.stateFor(sprite);
    const nextColor = normalizedColor(color);
    if (state.color === nextColor) {
      return sprite;
    }

    const animationKey = state.animationKey;
    const displayedFrameIndex = sprite.anims.currentFrame?.index;
    const currentFrameIndex = displayedFrameIndex === undefined ? 0 : Math.max(0, displayedFrameIndex - 1);
    const continueAnimation = animationKey !== null && sprite.anims.isPlaying;
    state.color = nextColor;

    if (continueAnimation && animationKey !== null) {
      this.startAnimation(sprite, state, animationKey, state.animationOptions, currentFrameIndex, true);
    } else {
      this.reset(sprite, { texture: state.texture, frame: state.frame });
    }

    return sprite;
  }

  reset(sprite: Phaser.GameObjects.Sprite, frame: BoardSpriteFrame): Phaser.GameObjects.Sprite {
    const state = this.stateFor(sprite);
    this.clearCompletionListeners(sprite);
    if (sprite.active === false || !sprite.scene) {
      state.animationKey = null;
      state.animationOptions = {};
      return sprite;
    }

    const texture = this.resolveTexture(frame.texture, state.color);
    sprite.stop();
    if (sprite.texture.key === texture) {
      sprite.setFrame(frame.frame);
    } else {
      sprite.setTexture(texture, frame.frame);
    }
    state.animationKey = null;
    state.animationOptions = {};
    state.frame = frame.frame;
    state.texture = frame.texture;
    return sprite;
  }

  listenForComplete(
    sprite: Phaser.GameObjects.Sprite,
    animationKey: string,
    callback: () => void,
  ): () => void {
    const spriteState = this.stateFor(sprite);
    const event = ANIMATION_COMPLETE_EVENT;
    let active = true;
    let dispose: () => void = () => {};
    const listener = (animation: Phaser.Animations.Animation): void => {
      if (
        spriteState.animationKey !== animationKey ||
        animation.key !== this.resolveAnimation(animationKey, spriteState.color)
      ) {
        return;
      }

      dispose();
      spriteState.animationKey = null;
      spriteState.animationOptions = {};
      callback();
    };
    dispose = (): void => {
      if (!active) return;
      active = false;
      sprite.off(event, listener);
      spriteState.completionDisposers.delete(dispose);
    };
    sprite.on(event, listener);
    spriteState.completionDisposers.add(dispose);
    return dispose;
  }

  private clearCompletionListeners(sprite: Phaser.GameObjects.Sprite): void {
    const state = this.spriteStates.get(sprite);
    if (!state) return;
    for (const dispose of [...state.completionDisposers]) {
      dispose();
    }
  }

  private stateFor(sprite: Phaser.GameObjects.Sprite): SpriteState {
    const state = this.spriteStates.get(sprite);
    if (!state) {
      throw new Error("Board sprite was not created by this resolver.");
    }
    return state;
  }
}
