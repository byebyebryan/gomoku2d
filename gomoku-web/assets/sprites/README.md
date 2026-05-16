# Gomoku2D Sprite Assets

All runtime board spritesheets use 16x16 frames. The source files in this
folder are the authoritative copies; matching files under
`public/assets/sprites/` must stay in sync for deployed asset URLs.

For the meaning of each board-space visual role, see
`../../../docs/game_visual.md`. This README is only the frame inventory.
For a visual inspection page, open [preview.html](./preview.html).

## Published Sprites

| File | Size | Layout | Description |
|------|------|--------|-------------|
| `caution.png` | 96x48 | 6 cols x 3 rows | Tactical caution and forbidden overlays |
| `highlighter.png` | 96x48 | 6 cols x 3 rows | Board-cell highlight variants for hints and replay analysis |
| `hover.png` | 96x16 | 6 cols x 1 row | Winning-line hover overlay |
| `marker.png` | 96x96 | 6 cols x 6 rows | Warning and proof/result marker variants |
| `pointer.png` | 160x32 | 10 cols x 2 rows | Touch/mouse pointer idle cues |
| `stone.png` | 96x64 | 6 cols x 4 rows | Stone destroy and idle loops |
| `transform.png` | 160x16 | 10 cols x 1 row | Form transform used by stone placement |

## Z-Order

`caution.png`, `highlighter.png`, and `marker.png` are board-surface features.
They sit above the board/grid and below pointer, stone, sequence, and hover
layers.

Surface sub-order:

1. marker / caution
2. highlighter
3. board / grid

## Color Language

| Role | Tint |
|---|---|
| `highlight-strong` | Red for immediate threat/loss, green for immediate win; preview uses red |
| `highlight-soft` | Pink for imminent threat, purple for counter-threat; preview uses pink |
| `highlight-entry` | Per-side corridor-entry context; preview uses white |
| `marker-warning` | Red for immediate loss/threat, green for immediate win; preview uses red |
| `marker-question` | Gray |
| `marker-L` | Red |
| `marker-F` | Red |
| `marker-E` | Green |
| `marker-P` | Teal |

## Frame Layout

Frame numbers are row-major.

### `caution.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-5 | `caution-forbidden-warning` | 12 | Combined forbidden + warning caution loop |
| 6-11 | `caution-forbidden-out` | 12 | Forbidden move loop half |
| 12-17 | `caution-forbidden-in` | 12 | Forbidden move loop half |

### `highlighter.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-5 | `highlight-strong` | 12 | Strong board-cell highlight |
| 6-11 | `highlight-soft` | 12 | Subtle board-cell highlight |
| 12-17 | `highlight-entry` | 12 | Corridor-entry or critical-point highlight |

### `hover.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-5 | `hover` | 12 | Winning line, tinted green |

### `marker.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-5 | `marker-warning` | 12 | Immediate-loss marker |
| 6-11 | `marker-question` | 12 | Unknown marker |
| 12-17 | `marker-L` | 12 | Forced-loss marker |
| 18-23 | `marker-F` | 12 | Forbidden-reply marker |
| 24-29 | `marker-E` | 12 | Confirmed-escape marker |
| 30-35 | `marker-P` | 12 | Possible-escape marker |

### `pointer.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-5 | `pointer-idle-blocked` | 12 | Blocked pointer cue |
| 6-9 | `pointer-idle-open` | 12 | Normal open-cell pointer cue |
| 10-19 | `pointer-idle-preferred` | 12 | Preferred pointer cue for winning or threat cells |
| 0 | static | - | Rest frame between pointer loop beats |

### `stone.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-3 | `stone-destroy` | 12 | Stone removal |
| 0-5 | `stone-idle-1` | 6 | Last placed stone idle variant |
| 6-11 | `stone-idle-2` | 6 | Last placed stone idle variant |
| 12-17 | `stone-idle-3` | 6 | Last placed stone idle variant |
| 18-23 | `stone-idle-4` | 6 | Last placed stone idle variant |
| 0 | static | - | Normal resting stone frame |

### `transform.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-9 | `transform-form` | 18 | Stone placement |
