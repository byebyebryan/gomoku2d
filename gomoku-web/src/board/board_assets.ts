import * as Phaser from "phaser";

import {
  CAUTION_ANIMS,
  FRAME_SIZE,
  HIGHLIGHTER_ANIMS,
  HOVER_ANIMS,
  MARKER_ANIMS,
  POINTER_ANIMS,
  SPRITE,
  STONE_ANIMS,
  TRANSFORM_ANIMS,
} from "./constants";

type AnimationRange = {
  end: number;
  frameRate: number;
  key: string;
  start: number;
};

const ASSET_URLS = {
  caution: new URL("../../assets/sprites/caution.png", import.meta.url).toString(),
  highlighter: new URL("../../assets/sprites/highlighter.png", import.meta.url).toString(),
  hover: new URL("../../assets/sprites/hover.png", import.meta.url).toString(),
  marker: new URL("../../assets/sprites/marker.png", import.meta.url).toString(),
  pointer: new URL("../../assets/sprites/pointer.png", import.meta.url).toString(),
  stone: new URL("../../assets/sprites/stone.png", import.meta.url).toString(),
  transform: new URL("../../assets/sprites/transform.png", import.meta.url).toString(),
} as const;

export const STONE_IDLE_ANIMS = [
  STONE_ANIMS.IDLE_1,
  STONE_ANIMS.IDLE_2,
  STONE_ANIMS.IDLE_3,
  STONE_ANIMS.IDLE_4,
] as const;

export function preloadBoardAssets(scene: Phaser.Scene): void {
  preloadSpritesheet(scene, SPRITE.CAUTION, ASSET_URLS.caution);
  preloadSpritesheet(scene, SPRITE.HIGHLIGHTER, ASSET_URLS.highlighter);
  preloadSpritesheet(scene, SPRITE.STONE, ASSET_URLS.stone);
  preloadSpritesheet(scene, SPRITE.POINTER, ASSET_URLS.pointer);
  preloadSpritesheet(scene, SPRITE.HOVER, ASSET_URLS.hover);
  preloadSpritesheet(scene, SPRITE.MARKER, ASSET_URLS.marker);
  preloadSpritesheet(scene, SPRITE.TRANSFORM, ASSET_URLS.transform);
}

export function ensureBoardAnimations(scene: Phaser.Scene): void {
  ensureRangeAnimation(scene, SPRITE.TRANSFORM, TRANSFORM_ANIMS.FORM);
  ensureRangeAnimation(scene, SPRITE.STONE, STONE_ANIMS.DESTROY);

  for (const idle of STONE_IDLE_ANIMS) {
    ensureRangeAnimation(scene, SPRITE.STONE, idle);
  }

  for (const anim of [
    POINTER_ANIMS.BLOCKED,
    POINTER_ANIMS.OPEN,
    POINTER_ANIMS.PREFERRED,
  ]) {
    ensureRangeAnimation(scene, SPRITE.POINTER, anim);
  }

  ensureRangeAnimation(scene, SPRITE.HOVER, HOVER_ANIMS.HOVER);

  for (const anim of [
    CAUTION_ANIMS.FORBIDDEN_WARNING,
    CAUTION_ANIMS.FORBIDDEN_OUT,
    CAUTION_ANIMS.FORBIDDEN_IN,
  ]) {
    ensureRangeAnimation(scene, SPRITE.CAUTION, anim);
  }

  for (const anim of [
    HIGHLIGHTER_ANIMS.STRONG,
    HIGHLIGHTER_ANIMS.SOFT,
    HIGHLIGHTER_ANIMS.ENTRY,
  ]) {
    ensureRangeAnimation(scene, SPRITE.HIGHLIGHTER, anim);
  }

  for (const anim of [
    MARKER_ANIMS.WARNING,
    MARKER_ANIMS.QUESTION,
    MARKER_ANIMS.L,
    MARKER_ANIMS.F,
    MARKER_ANIMS.E,
    MARKER_ANIMS.P,
  ]) {
    ensureRangeAnimation(scene, SPRITE.MARKER, anim);
  }
}

function preloadSpritesheet(scene: Phaser.Scene, key: string, url: string): void {
  if (scene.textures.exists(key)) {
    return;
  }

  scene.load.spritesheet(key, url, {
    frameHeight: FRAME_SIZE,
    frameWidth: FRAME_SIZE,
  });
}

function ensureRangeAnimation(scene: Phaser.Scene, sprite: string, anim: AnimationRange, repeat = 0): void {
  if (scene.anims.exists(anim.key)) {
    return;
  }

  scene.anims.create({
    key: anim.key,
    frames: scene.anims.generateFrameNumbers(sprite, {
      start: anim.start,
      end: anim.end,
    }),
    frameRate: anim.frameRate,
    repeat,
  });
}
