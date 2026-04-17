// Sprite assets — 16x16 per frame (spritesheets) or 18x18 per frame (button).
// Loaded in boot scene via spritesheet configs.

export const SPRITE = {
  STONE: "stone",
  POINTER: "pointer",
  WARNING: "warning",
  BUTTON: "button",
} as const;

export const FRAME_SIZE = 16;
export const BUTTON_FRAME_SIZE = 18;

export const SPRITESHEET_CONFIG = {
  [SPRITE.STONE]: { url: "assets/sprites/stone.png", end: 33 },
  [SPRITE.POINTER]: { url: "assets/sprites/pointer.png", end: 18 },
  [SPRITE.WARNING]: { url: "assets/sprites/warning.png", end: 28 },
} as const;

// Animation definitions — frame ranges from assets/manifest.md
export const STONE_ANIMS = {
  FORM: { start: 25, end: 33, frameRate: 18, key: "stone-form" },
  STATIC: { frame: 0, key: "stone-static" },
  DESTROY: { start: 0, end: 3, frameRate: 12, key: "stone-destroy" },
  RELAX_1: { start: 0, end: 6, frameRate: 6, key: "stone-relax-1" },
  RELAX_2: { start: 6, end: 12, frameRate: 6, key: "stone-relax-2" },
  RELAX_3: { start: 12, end: 18, frameRate: 6, key: "stone-relax-3" },
  RELAX_4: { start: 18, end: 24, frameRate: 6, key: "stone-relax-4" },
} as const;

export const POINTER_ANIMS = {
  OUT: { start: 0, end: 4, frameRate: 12, key: "pointer-out" },
  IN: { start: 4, end: 8, frameRate: 12, key: "pointer-in" },
  FULL: { start: 8, end: 18, frameRate: 12, key: "pointer-full" },
} as const;

export const WARNING_ANIMS = {
  POINTER: { start: 0, end: 9, frameRate: 12, key: "warning-pointer" },
  HOVER: { start: 10, end: 16, frameRate: 12, key: "warning-hover" },
  FORBIDDEN: { start: 17, end: 28, frameRate: 10, key: "warning-forbidden" },
} as const;

// Board layout constants
export const BOARD_SIZE = 15;
export const WIN_LENGTH = 5;
