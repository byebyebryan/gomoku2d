// Sprite assets — 16x16 per frame.
// Loaded in boot scene via spritesheet configs.

export const SPRITE = {
  STONE: "stone",
  POINTER: "pointer",
  HOVER: "hover",
  WARNING: "warning",
  TRANSFORM: "transform",
} as const;

export const FRAME_SIZE = 16;

export const BOARD_RENDER_DEPTHS = {
  BOARD: 0,
  WARNING_SURFACE: 0.25,
  WARNING_BLOCKED: 0.25,
  POINTER: 0.5,
  STONE: 1,
  SEQUENCE_NUMBER: 1.5,
  WARNING_HOVER: 2,
  INPUT_ZONE: 3,
} as const;

export const BOARD_RENDER_LAYER_ORDER = [
  "BOARD",
  "WARNING",
  "POINTER",
  "STONE",
  "SEQUENCE_NUMBER",
  "HOVER",
] as const;

export const SPRITESHEET_CONFIG = {
  [SPRITE.STONE]: { url: "assets/sprites/stone.png", end: 23 },
  [SPRITE.POINTER]: { url: "assets/sprites/pointer.png", end: 19 },
  [SPRITE.HOVER]: { url: "assets/sprites/hover.png", end: 5 },
  [SPRITE.WARNING]: { url: "assets/sprites/warning.png", end: 29 },
  [SPRITE.TRANSFORM]: { url: "assets/sprites/transform.png", end: 9 },
} as const;

// Animation definitions — frame ranges from assets/manifest.md
export const STONE_ANIMS = {
  STATIC: { frame: 0, key: "stone-static" },
  DESTROY: { start: 0, end: 3, frameRate: 12, key: "stone-destroy" },
  IDLE_1: { start: 0, end: 5, frameRate: 6, key: "stone-idle-1" },
  IDLE_2: { start: 6, end: 11, frameRate: 6, key: "stone-idle-2" },
  IDLE_3: { start: 12, end: 17, frameRate: 6, key: "stone-idle-3" },
  IDLE_4: { start: 18, end: 23, frameRate: 6, key: "stone-idle-4" },
} as const;

export const POINTER_ANIMS = {
  STATIC: { frame: 0, key: "pointer-static" },
  BLOCKED: { start: 0, end: 5, frameRate: 12, key: "pointer-idle-blocked" },
  OPEN: { start: 6, end: 9, frameRate: 12, key: "pointer-idle-open" },
  PREFERRED: { start: 10, end: 19, frameRate: 12, key: "pointer-idle-preferred" },
} as const;

export const HOVER_ANIMS = {
  HOVER: { start: 0, end: 5, frameRate: 12, key: "hover" },
} as const;

export const WARNING_ANIMS = {
  WARNING: { start: 0, end: 5, frameRate: 12, key: "warning" },
  WARNING_ON_FORBIDDEN: { start: 6, end: 11, frameRate: 12, key: "warning-on-forbidden" },
  FORBIDDEN_OUT: { start: 12, end: 17, frameRate: 12, key: "forbidden-out" },
  FORBIDDEN_IN: { start: 18, end: 23, frameRate: 12, key: "forbidden-in" },
  HIGHLIGHT: { start: 24, end: 29, frameRate: 12, key: "highlight" },
} as const;

export const TRANSFORM_ANIMS = {
  FORM: { start: 0, end: 9, frameRate: 18, key: "transform-form" },
} as const;

// Board layout constants
export const BOARD_SIZE = 15;
export const WIN_LENGTH = 5;

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
