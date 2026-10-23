import type * as Phaser from "phaser";
import { describe, expect, it, vi } from "vitest";

import {
  BoardSpriteResolver,
  coloredAnimationKey,
  coloredTextureKey,
  multiplyTintRgbaPixels,
} from "./board_sprite_resolver";

type FakeAnimation = {
  key: string;
  toJSON: () => Phaser.Types.Animations.JSONAnimation;
};

type FakeListener = (...args: unknown[]) => void;

class FakeSprite {
  active = true;
  scene: Phaser.Scene | null = {} as Phaser.Scene;
  texture: { key: string };
  frame: string | number;
  plays: Phaser.Types.Animations.PlayAnimationConfig[] = [];
  anims: {
    currentAnim: Phaser.Animations.Animation | null;
    currentFrame: Phaser.Animations.AnimationFrame | null;
    isPlaying: boolean;
  } = { currentAnim: null, currentFrame: null, isPlaying: false };

  private readonly listeners = new Map<string, Set<FakeListener>>();

  constructor(
    texture: string,
    frame: string | number,
    private readonly getAnimation: (key: string) => FakeAnimation | undefined,
  ) {
    this.texture = { key: texture };
    this.frame = frame;
  }

  play(config: string | Phaser.Types.Animations.PlayAnimationConfig): this {
    const playConfig = typeof config === "string" ? { key: config } : config;
    const animation = this.getAnimation(playConfig.key as string);
    if (!animation) throw new Error(`Missing fake animation ${String(playConfig.key)}`);
    this.plays.push({ ...playConfig, key: animation.key });
    this.anims.currentAnim = animation as Phaser.Animations.Animation;
    const startFrame = playConfig.startFrame ?? 0;
    this.anims.currentFrame = { index: startFrame + 1 } as Phaser.Animations.AnimationFrame;
    this.anims.isPlaying = true;
    return this;
  }

  stop(): this {
    this.anims.isPlaying = false;
    return this;
  }

  setTexture(texture: string, frame: string | number = 0): this {
    this.texture.key = texture;
    this.frame = frame;
    return this;
  }

  setFrame(frame: string | number): this {
    this.frame = frame;
    return this;
  }

  on(event: string, listener: FakeListener): this {
    const listeners = this.listeners.get(event) ?? new Set<FakeListener>();
    listeners.add(listener);
    this.listeners.set(event, listeners);
    return this;
  }

  once(event: string, listener: FakeListener): this {
    const once = (...args: unknown[]): void => {
      this.off(event, once);
      listener(...args);
    };
    return this.on(event, once);
  }

  off(event: string, listener: FakeListener): this {
    this.listeners.get(event)?.delete(listener);
    return this;
  }

  emit(event: string, ...args: unknown[]): void {
    for (const listener of [...(this.listeners.get(event) ?? [])]) listener(...args);
  }

  listenerCount(event: string): number {
    return this.listeners.get(event)?.size ?? 0;
  }

  destroy(): void {
    this.active = false;
    this.scene = null;
    this.emit("destroy");
  }
}

function animationData(key: string, texture: string, frames: number[]): Phaser.Types.Animations.JSONAnimation {
  return {
    key,
    type: "frame",
    frames: frames.map((frame, index) => ({ key: texture, frame, duration: 50 + index })),
    frameRate: 12,
    duration: frames.length * 50,
    skipMissedFrames: true,
    delay: 0,
    repeat: 0,
    repeatDelay: 0,
    yoyo: false,
    showBeforeDelay: false,
    showOnStart: true,
    hideOnComplete: false,
  };
}

function fakeAnimation(data: Phaser.Types.Animations.JSONAnimation): FakeAnimation {
  return {
    key: data.key,
    toJSON: () => ({ ...data, frames: data.frames.map((frame) => ({ ...frame })) }),
  };
}

function createHarness() {
  const width = 32;
  const height = 16;
  const originalPixels = new Uint8ClampedArray(width * height * 4);
  originalPixels.set([200, 100, 50, 77, 99, 88, 77, 0]);
  const sourceLookups: string[] = [];
  const canvases: Array<{
    canvas: HTMLCanvasElement;
    data: Uint8ClampedArray | null;
    drawnImage: CanvasImageSource | null;
  }> = [];
  const textureMap = new Map<string, {
    key: string;
    source: Array<{ width: number; height: number }>;
    getSourceImage: () => HTMLImageElement | HTMLCanvasElement;
  }>();

  for (const key of ["stone", "pointer", "transform"]) {
    textureMap.set(key, {
      key,
      source: [{ width, height }],
      getSourceImage: () => {
        sourceLookups.push(key);
        return { id: key } as unknown as HTMLImageElement;
      },
    });
  }

  const spriteSheets: Array<{ key: string; frameHeight: number; frameWidth: number }> = [];
  const textures = {
    exists: (key: string) => textureMap.has(key),
    get: (key: string) => {
      const texture = textureMap.get(key);
      if (!texture) throw new Error(`Missing fake texture ${key}`);
      return texture;
    },
    addCanvas: (key: string, canvas: HTMLCanvasElement) => {
      const texture = {
        key,
        source: [{ width: canvas.width, height: canvas.height }],
        getSourceImage: () => canvas,
      };
      textureMap.set(key, texture);
      return texture;
    },
    addSpriteSheet: (key: string, texture: { key: string }, config: Phaser.Types.Textures.SpriteSheetConfig) => {
      spriteSheets.push({ key, frameHeight: config.frameHeight ?? config.frameWidth, frameWidth: config.frameWidth });
      return texture;
    },
    remove: (key: string) => {
      textureMap.delete(key);
      return textures;
    },
  };

  const animations = new Map<string, FakeAnimation>();
  const sourceAnimation = fakeAnimation(animationData("pointer-open", "pointer", [6, 7, 8, 9]));
  animations.set(sourceAnimation.key, sourceAnimation);
  const animationCreations: Phaser.Types.Animations.Animation[] = [];
  const anims = {
    exists: (key: string) => animations.has(key),
    get: (key: string) => animations.get(key),
    create: (config: Phaser.Types.Animations.Animation) => {
      animationCreations.push(config);
      const data = {
        ...animationData(config.key ?? "", "pointer", []),
        ...config,
        key: config.key ?? "",
        frames: (config.frames ?? []) as Phaser.Types.Animations.JSONAnimation["frames"],
      } as Phaser.Types.Animations.JSONAnimation;
      const animation = fakeAnimation(data);
      animations.set(animation.key, animation);
      return animation;
    },
  };

  const sprites: FakeSprite[] = [];
  const scene = {
    textures,
    anims,
    add: {
      sprite: (_x: number, _y: number, texture: string, frame: string | number) => {
        const sprite = new FakeSprite(texture, frame, (key) => animations.get(key));
        sprites.push(sprite);
        return sprite;
      },
    },
  };

  const canvasFactory = (canvasWidth: number, canvasHeight: number): HTMLCanvasElement => {
    const record: (typeof canvases)[number] = { canvas: null as never, data: null, drawnImage: null };
    const context = {
      drawImage: (image: CanvasImageSource) => { record.drawnImage = image; },
      getImageData: () => ({ data: new Uint8ClampedArray(originalPixels) } as ImageData),
      putImageData: (image: ImageData) => { record.data = new Uint8ClampedArray(image.data); },
    } as unknown as CanvasRenderingContext2D;
    const canvas = {
      width: canvasWidth,
      height: canvasHeight,
      getContext: () => context,
    } as unknown as HTMLCanvasElement;
    record.canvas = canvas;
    canvases.push(record);
    return canvas;
  };

  return {
    animationCreations,
    animations,
    canvases,
    canvasFactory,
    scene: scene as unknown as Phaser.Scene,
    sourceLookups,
    sourcePixels: originalPixels,
    sprites,
    spriteSheets,
  };
}

describe("board sprite tinting", () => {
  it("multiplies RGB channels while preserving alpha and the original pixel data", () => {
    const source = new Uint8ClampedArray([200, 100, 50, 77, 99, 88, 77, 0]);

    const tinted = multiplyTintRgbaPixels(source, 0x8040ff);

    expect([...tinted]).toEqual([100, 25, 50, 77, 50, 22, 77, 0]);
    expect([...source]).toEqual([200, 100, 50, 77, 99, 88, 77, 0]);
  });

  it("reuses the original white sheet and caches each full-sheet variant from the source", () => {
    const harness = createHarness();
    const resolver = new BoardSpriteResolver(harness.scene, harness.canvasFactory);

    expect(coloredTextureKey("stone", 0xffffff)).toBe("stone");
    expect(resolver.resolveTexture("stone", 0xffffff)).toBe("stone");

    const firstKey = resolver.resolveTexture("stone", 0x8040ff);
    expect(resolver.resolveTexture("stone", 0x8040ff)).toBe(firstKey);
    expect(harness.canvases).toHaveLength(1);
    expect(harness.canvases[0]?.canvas.width).toBe(32);
    expect(harness.canvases[0]?.canvas.height).toBe(16);
    expect([...harness.canvases[0]!.data!.slice(0, 8)]).toEqual([100, 25, 50, 77, 50, 22, 77, 0]);
    expect(harness.sourceLookups).toEqual(["stone"]);
    expect(harness.spriteSheets).toEqual([{ key: "", frameHeight: 16, frameWidth: 16 }]);

    const secondKey = resolver.resolveTexture("stone", 0x808080);
    expect(secondKey).not.toBe(firstKey);
    expect([...harness.canvases[1]!.data!.slice(0, 4)]).toEqual([100, 50, 25, 77]);
    expect(harness.sourceLookups).toEqual(["stone", "stone"]);

    const sameGameResolver = new BoardSpriteResolver(harness.scene, () => {
      throw new Error("cached variant should not create another canvas");
    });
    expect(sameGameResolver.resolveTexture("stone", 0x8040ff)).toBe(firstKey);
    expect(harness.canvases).toHaveLength(2);
    expect([...harness.sourcePixels.slice(0, 4)]).toEqual([200, 100, 50, 77]);
  });
});

describe("board sprite animation resolution", () => {
  it("maps every animation frame to one cached coloured sheet without changing timing", () => {
    const harness = createHarness();
    const resolver = new BoardSpriteResolver(harness.scene, harness.canvasFactory);
    const tint = 0x404040;
    const textureKey = coloredTextureKey("pointer", tint);
    const animationKey = coloredAnimationKey("pointer-open", tint);

    expect(resolver.resolveAnimation("pointer-open", tint)).toBe(animationKey);
    expect(resolver.resolveAnimation("pointer-open", tint)).toBe(animationKey);
    expect(harness.animationCreations).toHaveLength(1);
    expect(harness.animationCreations[0]).toMatchObject({
      frameRate: 12,
      key: animationKey,
      repeat: 0,
      frames: [
        { key: textureKey, frame: 6, duration: 50 },
        { key: textureKey, frame: 7, duration: 51 },
        { key: textureKey, frame: 8, duration: 52 },
        { key: textureKey, frame: 9, duration: 53 },
      ],
    });
    expect(harness.canvases).toHaveLength(1);
  });

  it("keeps a pointer on its current last frame when colour changes and completes the new animation", () => {
    const harness = createHarness();
    const resolver = new BoardSpriteResolver(harness.scene, harness.canvasFactory);
    const sprite = resolver.createSprite(0, 0, "pointer", 0, 0x404040) as unknown as FakeSprite;
    const completed = vi.fn();

    resolver.play(sprite as unknown as Phaser.GameObjects.Sprite, "pointer-open");
    resolver.listenForComplete(sprite as unknown as Phaser.GameObjects.Sprite, "pointer-open", completed);
    sprite.anims.currentFrame = { index: 4 } as Phaser.Animations.AnimationFrame;

    resolver.setColor(sprite as unknown as Phaser.GameObjects.Sprite, 0xffffff);

    expect(sprite.plays[sprite.plays.length - 1]).toMatchObject({ key: "pointer-open", startFrame: 3 });
    expect(sprite.anims.currentFrame?.index).toBe(4);
    sprite.emit("animationcomplete", {
      key: coloredAnimationKey("pointer-open", 0x404040),
    });
    expect(completed).not.toHaveBeenCalled();
    expect(sprite.listenerCount("animationcomplete")).toBe(1);

    sprite.emit("animationcomplete", { key: "pointer-open" });
    expect(completed).toHaveBeenCalledTimes(1);
    expect(sprite.listenerCount("animationcomplete")).toBe(0);
  });

  it("keeps reset frames coloured and removes completion listeners on reset or destruction", () => {
    const harness = createHarness();
    const resolver = new BoardSpriteResolver(harness.scene, harness.canvasFactory);
    const sprite = resolver.createSprite(0, 0, "pointer", 0, 0x404040) as unknown as FakeSprite;
    const completed = vi.fn();
    const textureKey = coloredTextureKey("pointer", 0x404040);

    resolver.play(sprite as unknown as Phaser.GameObjects.Sprite, "pointer-open");
    resolver.listenForComplete(sprite as unknown as Phaser.GameObjects.Sprite, "pointer-open", completed);
    resolver.reset(sprite as unknown as Phaser.GameObjects.Sprite, { texture: "pointer", frame: 2 });
    expect(sprite.texture.key).toBe(textureKey);
    expect(sprite.frame).toBe(2);
    expect(sprite.listenerCount("animationcomplete")).toBe(0);
    sprite.emit("animationcomplete", { key: coloredAnimationKey("pointer-open", 0x404040) });
    expect(completed).not.toHaveBeenCalled();

    resolver.play(sprite as unknown as Phaser.GameObjects.Sprite, "pointer-open");
    resolver.listenForComplete(sprite as unknown as Phaser.GameObjects.Sprite, "pointer-open", completed);
    sprite.destroy();
    expect(sprite.listenerCount("animationcomplete")).toBe(0);
    expect(() => resolver.reset(sprite as unknown as Phaser.GameObjects.Sprite, { texture: "pointer", frame: 0 })).not.toThrow();
    expect(completed).not.toHaveBeenCalled();
  });
});
