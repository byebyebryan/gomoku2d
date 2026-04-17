# Gomoku2D — Sprite Asset Manifest

All sprites are pixel-art style. Spritesheets use 16×16 per frame; button uses 18×18 per frame.

---

## Published sprites (copied to `public/assets/sprites/`)

| File | Size | Description |
|------|------|-------------|
| `stone.png` | 544×16 | Stone animations — 34 frames |
| `pointer.png` | 304×16 | Cursor animations — 19 frames |
| `warning.png` | 464×16 | Warning overlays — 29 frames (0–9 legacy pointer, 10–16 win, 17–28 forbidden) |
| `button.png` | 72×18 | Button 9-slice — 4 frames (0=up, 1=hover, 2=down, 3=pointer) |
| `PixelOperator8-Bold.png` | — | Bitmap font atlas |
| `PixelOperator8-Bold.fnt` | — | Bitmap font descriptor |

---


---

## Spritesheet frame layout

### `stone.png` (16×16 per frame)

| Frames | Animation | FPS | Notes |
|--------|-----------|-----|-------|
| 0–3 | `stone-destroy` | 12 | |
| 0–6 | `stone-relax-1` | 6 | Overlaps destroy range |
| 6–12 | `stone-relax-2` | 6 | |
| 12–18 | `stone-relax-3` | 6 | |
| 18–24 | `stone-relax-4` | 6 | |
| 25–33 | `stone-form` | 18 | Placement anim |
| 0 | `stone-static` | — | Resting frame |

### `pointer.png` (16×16 per frame)

| Frames | Animation | FPS |
|--------|-----------|-----|
| 0–4 | `pointer-out` | 12 |
| 4–8 | `pointer-in` | 12 |
| 8–18 | `pointer-full` | 12 |
| 8 | static idle frame | — |

### `warning.png` (16×16 per frame)

| Frames | Animation | FPS | Notes |
|--------|-----------|-----|-------|
| 0–9 | `warning-pointer` | 12 | Loops; used for human move hints (green = winning, red = losing) |
| 10–16 | `warning-hover` | 12 | Loops; used for win highlights (green tint) |
| 17–28 | `warning-forbidden` | 10 | Loops; used for forbidden move overlays (red tint) |

### `button.png` (18×18 per frame, 9-slice borders: 8px each side)

| Frame | State |
|-------|-------|
| 0 | Up / normal |
| 1 | Hover |
| 2 | Down / pressed / disabled |
| 3 | Pointer / active / selected |
