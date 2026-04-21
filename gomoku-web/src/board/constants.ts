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
export const FONT_KEY = "pixel" as const;

const P = {
  WHITE:  0xffffff,
  DARK:   0x404040,
  DARKER: 0x303030,
  GREY:   0x888888,
  GOLD:   0xffcc66,
  BLACK:  0x000000,
  GREEN:       0x22aa44,
  GREEN_BRIGHT: 0x44dd44,
  RED:         0xcc2222,
  RED_BRIGHT:   0xff4444,
} as const;

function shade(base: number, f: number): number {
  const r = Math.min(255, Math.round((base >> 16 & 0xff) * f));
  const g = Math.min(255, Math.round((base >>  8 & 0xff) * f));
  const b = Math.min(255, Math.round((base       & 0xff) * f));
  return (r << 16) | (g << 8) | b;
}

export function lerpColor(a: number, b: number, t: number): number {
  const ar = (a >> 16) & 0xff, ag = (a >> 8) & 0xff, ab = a & 0xff;
  const br = (b >> 16) & 0xff, bg = (b >> 8) & 0xff, bb = b & 0xff;
  const r = Math.round(ar + (br - ar) * t);
  const g = Math.round(ag + (bg - ag) * t);
  const bl = Math.round(ab + (bb - ab) * t);
  return (r << 16) | (g << 8) | bl;
}

const BTN_SHADES: [number, number, number] = [1.0, 1.3, 0.7];

export const RED_BTN_TINTS:   [number, number, number] = BTN_SHADES.map(f => shade(P.RED,   f)) as [number, number, number];
export const GREEN_BTN_TINTS: [number, number, number] = BTN_SHADES.map(f => shade(P.GREEN, f)) as [number, number, number];

export const COLOR = {
  STONE_BLACK:  P.DARK,
  STONE_WHITE:  P.WHITE,

  TEXT_ON_BLACK:    P.WHITE,
  TEXT_ON_WHITE:    P.DARK,
  SUBTEXT:          P.GREY,

  SCORE_GAIN:   P.GREEN_BRIGHT,
  TIME_DELTA:   P.GOLD,

  PAGE_BG:       P.DARKER,

  BOARD_SURFACE: P.GOLD,
  GRID:          P.BLACK,

  TITLE:        P.GOLD,
  LABEL:        P.WHITE,

  BTN_LABEL:        P.WHITE,
  TOGGLE_LABEL:     P.WHITE,
  TOGGLE_NORMAL:    P.GREY,
  TOGGLE_HOVER:     shade(P.GREY, BTN_SHADES[1]),
  TOGGLE_SELECTED:  P.GREEN,

  FORBIDDEN:    P.RED_BRIGHT,
  THREAT:       P.RED_BRIGHT,
  WIN_MOVE:     P.GREEN_BRIGHT,
  WIN_CELLS:    P.GREEN_BRIGHT,

  SEQ_ON_BLACK: P.WHITE,
  SEQ_ON_WHITE: P.DARK,
} as const;
