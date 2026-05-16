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
| `caution.png` | 96x48 | 6 cols x 3 rows | Staged replacement for tactical caution and forbidden overlays |
| `highlighter.png` | 96x48 | 6 cols x 3 rows | Staged board-cell highlighter variants for hints and replay analysis |
| `hover.png` | 96x16 | 6 cols x 1 row | Winning-line hover overlay |
| `marker.png` | 96x96 | 6 cols x 6 rows | Staged proof/result marker variants for replay analysis |
| `pointer.png` | 160x32 | 10 cols x 2 rows | Touch/mouse pointer idle cues |
| `stone.png` | 96x64 | 6 cols x 4 rows | Stone destroy and idle loops |
| `warning.png` | 96x80 | 6 cols x 5 rows | Tactical warning and forbidden overlays |
| `transform.png` | 160x16 | 10 cols x 1 row | Form transform used by stone placement |

## Frame Layout

Frame numbers are row-major.

### `caution.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-5 | `caution` | 12 | Staged tactical caution loop for winning/threat cells |
| 6-11 | `forbidden-out` | 12 | Staged forbidden move loop half |
| 12-17 | `forbidden-in` | 12 | Staged forbidden move loop half |

### `highlighter.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-5 | `highlight-soft` | 12 | Staged soft board-cell highlight |
| 6-11 | `highlight-outline` | 12 | Staged outline board-cell highlight |
| 12-17 | `highlight-strong` | 12 | Staged stronger board-cell highlight |

### `hover.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-5 | `hover` | 12 | Winning line, tinted green |

### `marker.png`

| Frames | Animation | FPS | Runtime use |
|--------|-----------|-----|-------------|
| 0-5 | `marker-1` | 12 | Staged replay-analysis proof/result marker variant |
| 6-11 | `marker-2` | 12 | Staged replay-analysis proof/result marker variant |
| 12-17 | `marker-3` | 12 | Staged replay-analysis proof/result marker variant |
| 18-23 | `marker-4` | 12 | Staged replay-analysis proof/result marker variant |
| 24-29 | `marker-5` | 12 | Staged replay-analysis proof/result marker variant |
| 30-35 | `marker-6` | 12 | Staged replay-analysis proof/result marker variant |

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
