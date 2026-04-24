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
| `hover.png` | 96x16 | 6 cols x 1 row | Winning-line hover overlay |
| `pointer.png` | 160x32 | 10 cols x 2 rows | Touch/mouse pointer idle cues |
| `stone.png` | 96x64 | 6 cols x 4 rows | Stone destroy and idle loops |
| `warning.png` | 96x80 | 6 cols x 5 rows | Tactical warning and forbidden overlays |
| `transform.png` | 160x16 | 10 cols x 1 row | Form transform used by stone placement |

## Frame Layout

Frame numbers are row-major.

### `hover.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-5 | `hover` | 12 | Winning line, tinted green |

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

### `warning.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-5 | `warning` | 12 | Winning move green, threat move red |
| 6-11 | `warning-on-forbidden` | 12 | Threat move red when the cell is also forbidden |
| 12-17 | `forbidden-out` | 12 | Forbidden move loop |
| 18-23 | `forbidden-in` | 12 | Forbidden move loop |
| 24-29 | `highlight` | 12 | Reserved; not used by runtime yet |

### `transform.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-9 | `transform-form` | 18 | Stone placement |
