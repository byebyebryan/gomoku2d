# Gomoku2D — Asset & Design Manifest

Extracted from Unity project. All sprites are pixel-art style at 16x16 per frame.

---

## Sprites

### Single Sprites (18x18, 9-slice buttons)

| File | Description | Borders (L,T,R,B) |
|------|-------------|-------------------|
| `button_0.png` | Default/normal state | 6,8,6,8 |
| `button_1.png` | Hover state | 6,8,6,8 |
| `button_2.png` | Pressed/disabled state | 6,8,6,8 |
| `button_3.png` | Active/selected state | 6,8,6,8 |

### Spritesheets (all 16x16 per frame, horizontal strip)

| File | Total Size | Frames | Description |
|------|-----------|--------|-------------|
| `Gomoku Stone.png` | 544x16 | 34 | Stone show/idle/hide animations |
| `Gomoku Pointer.png` | 304x16 | 19 | Cursor hover/pulse animations |
| `Gomoku Grid Half Line.png` | 304x16 | 19 | Board grid line segments |
| `Gomoku Warning L.png` | 160x16 | 10 | Win highlight (large overlay) |
| `Gomoku Warning H.png` | 112x16 | 7 | Win highlight (horizontal) |
| `Gomoku Numbers.png` | 160x16 | 10 | Digits 0-9 |

### Font

| File | Description |
|------|-------------|
| `Minecraft.ttf` | Pixel font, designed for size 16, no anti-aliasing |

---

## Frame Slicing

All spritesheets are horizontal strips. Frame extraction:
```
frame_index → srcRect = (index * 16, 0, 16, 16)
```

---

## Animation Definitions

### Stone Animations (from `Gomoku Stone.png`)

| Animation | Frames | Duration (s) | FPS | Trigger |
|-----------|--------|-------------|-----|---------|
| `stone-form` | 25–33 (9 frames) | 0.444 | 18 | On placement |
| `stone-static` | 0 (1 frame) | — | — | Resting state |
| `stone-destroy` | 0–3 (4 frames) | 0.25 | 12 | On removal |
| `stone-relax-1` | 0–6 (7 frames) | 1.0 | 6 | Random idle |
| `stone-relax-2` | 6–12 (7 frames) | 1.0 | 6 | Random idle |
| `stone-relax-3` | 12–18 (7 frames) | 1.0 | 6 | Random idle |
| `stone-relax-4` | 18–24 (7 frames) | 1.0 | 6 | Random idle |

**Frame layout on the sheet:**
```
Frames 0–3:   destroy sequence
Frames 0–6:   relax-1 (overlaps with destroy)
Frames 6–12:  relax-2
Frames 12–18: relax-3
Frames 18–24: relax-4
Frames 25–33: form (placement) sequence
```

**Stone Animator States:**
```
hide (default) ──[show trigger]──► show (stone-form anim)
                                      │
                                 [complete] ▼
                                    idle (stone-static)
                                      │
                              [idle trigger]──► relax-1/2/3/4 (random pick)
                                      │              │
                                      │         [complete]──► back to idle
                                      │
                              [hide trigger]──► hide (stone-destroy anim)
```

**Idle behavior:** After a random interval (5–10 seconds), one of relax-1 through relax-4 is picked randomly and plays once, then returns to static.

### Pointer Animations (from `Gomoku Pointer.png`)

| Animation | Frames | Duration (s) | FPS | Trigger |
|-----------|--------|-------------|-----|---------|
| `pointer-relax-out` | 0–4 (5 frames) | 0.333 | 12 | On pointer show/reset |
| `pointer-relax-in` | 4–8 (5 frames) | 0.333 | 12 | On hover enter (empty cell) |
| `pointer-relax-full` | 8–18 (11 frames) | 0.833 | 12 | Continuous idle pulse |

**Pointer behavior:**
- Hovering empty cell → `pointer-relax-in` then loops `pointer-relax-full`
- Hovering occupied cell → same anims but bool `non_empty_cell = true` (different visual state)
- Leaving cell → soft reset after delay, returns to off-screen

### Warning Animation (from `Gomoku Warning L.png`)

| Animation | Frames | Duration (s) | FPS | Trigger |
|-----------|--------|-------------|-----|---------|
| `warning-surface` | 0–9 (10 frames) | 0.75 | 12 | On win, loops |

---

## Grid Line Frames

`Gomoku Grid Half Line.png` has 19 frames representing different grid line segments (half-lines between two adjacent cells). Each Cell draws up to 4 half-lines (up, right, down, left) rotated appropriately:

| Direction | Rotation (Z) | Scale |
|-----------|-------------|-------|
| Up | 0° | (1, 1, 1) |
| Right | -90° | (-1, 1, 1) |
| Down | 180° | (1, 1, 1) |
| Left | 90° | (-1, 1, 1) |

Only drawn if neighbor exists in that direction (edge/corner cells skip).

---

## Game Design Notes

### Rules
- **Board**: 15×15 grid
- **Win**: 5+ stones in a row (horizontal, vertical, or diagonal)
- **Turns**: Black always goes first, then alternates
- **No restrictions**: Standard Gomoku (no renju rules, no swap)

### Layout & Coordinates
```
Board origin: centered at (0, 0) in world space
Cell [x,y] world position: (x - 7.0, y - 7.0, 0)
Cell [0,0] = top-left at (-7, -7)
Cell [14,14] = bottom-right at (7, 7)
```

### Z-Sorting (render order, higher = in front)
```
Layer formula: base = -y * 10

surface/edge:   base + 0
grid lines:     base + 1
pointer:        base + 4
warning:        base + 3
stone:          base + 6
```
Bottom rows (higher y) have lower sort order → appear behind top rows.

### Colors
| Element | RGB |
|---------|-----|
| Empty cell (background) | (127, 255, 145) — mint green |
| White stone | (255, 255, 255) |
| Black stone | (64, 64, 64) — dark gray, not pure black |
| Cell color flip formula | `reverse = 3 - (int)color` |

### Cascade/Ripple Animation
When the board shows or hides, cells animate in a wave pattern:
- Starts from cell [0,0] (top-left)
- Each cell, after its own animation starts, waits `cell_delay_before_wake_neighbors` **frames** (not seconds), then triggers neighbors at direction 0 (up) and direction 2 (right)
- This creates a diagonal cascade wave across the board

### Win Animation
- Winning stones lift up by 0.25 units
- Warning surface sprite activates and loops
- Clicking anywhere after win triggers game reset
- On reset after game end, player colors swap (Black↔White)

### Player Profiles
- Up to 5 profiles stored in XML
- Each profile tracks: name, total games/wins/losses, last 10 games, current session games
- Save/load via XML serialization

### UI Screens
1. **Splash** → auto-transitions to MainMenu after timer
2. **MainMenu** → player selection, profile editing, start button
3. **InGame** → active gameplay, minimal UI (player cards, menu/reset buttons)
