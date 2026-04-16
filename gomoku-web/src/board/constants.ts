// Sprite assets are all 16x16 per frame, horizontal strips.
// Loaded in boot scene via spritesheet configs.

export const SPRITE = {
  STONE: "stone",
  POINTER: "pointer",
  GRID: "grid",
  WARNING_L: "warning_l",
  WARNING_H: "warning_h",
  NUMBERS: "numbers",
} as const;

export const FRAME_SIZE = 16;

export const SPRITESHEET_CONFIG = {
  [SPRITE.STONE]: { url: "assets/sprites/gomoku_stone.png", end: 33 },
  [SPRITE.POINTER]: { url: "assets/sprites/gomoku_pointer.png", end: 18 },
  [SPRITE.GRID]: { url: "assets/sprites/gomoku_grid_half_line.png", end: 18 },
  [SPRITE.WARNING_L]: { url: "assets/sprites/gomoku_warning_l.png", end: 9 },
  [SPRITE.WARNING_H]: { url: "assets/sprites/gomoku_warning_h.png", end: 6 },
  [SPRITE.NUMBERS]: { url: "assets/sprites/gomoku_numbers.png", end: 9 },
  // button_0..3 are individual 18x18 images, not spritesheets
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
  SURFACE: { start: 0, end: 9, frameRate: 12, key: "warning-surface" },
} as const;

// Board layout constants
export const BOARD_SIZE = 15;
export const WIN_LENGTH = 5;
