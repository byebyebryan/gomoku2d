# Web Frontend Plan (Phaser 3)

**Last updated:** 2026-04-17
**Status:** All original phases complete and shipped to GitHub Pages.

---

## Tech choice: Phaser 3

| Concern | Phaser | PlayCanvas |
|---------|--------|------------|
| 2D spritesheet animation | Built-in, first-class | Supported but 3D-centric API |
| Click/hover input | `setInteractive()` per sprite | Raycasting against 3D entities |
| Physics (Pylander future) | Arcade built-in, Matter.js plugin | No built-in |
| Game loop / scene management | Built-in Phaser scenes | Built-in but 3D-oriented |
| Bundle size | ~1MB | ~1MB |
| Community / examples | 50k+ stars, massive example library | Smaller, professional |

Phaser is the stronger 2D tool. Original plan listed PlayCanvas — switched before
implementation started. Both Gomoku and Pylander are 2D; revisit if a future project
genuinely needs 3D.

---

## Actual project structure

```
gomoku-web/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── index.html
├── public/
│   └── assets/sprites/       ← spritesheets + bitmap font
├── src/
│   ├── main.ts               ← Phaser game config + Wasm init
│   ├── layout.ts             ← viewport → canvas size (portrait/landscape)
│   ├── scenes/
│   │   ├── boot.ts           ← asset preloading + animation registration
│   │   └── game.ts           ← all gameplay, settings, result screen
│   ├── board/
│   │   ├── board_renderer.ts ← grid, depth edge, stone/pointer/warning sprites
│   │   ├── ui.ts             ← PlayerCard, ActionButton, ToggleButton, InfoBox
│   │   └── constants.ts      ← SPRITE/ANIM keys, P palette, COLOR map, lerpColor
│   └── core/
│       ├── wasm_bridge.ts    ← WasmBoard + WasmBot JS wrappers, Wasm init
│       └── bot_worker.ts     ← Web Worker that runs bot choose_move off-thread
```

No `menu.ts` or `replay.ts` — single game scene handles everything.

---

## Implementation phases (retrospective)

### Phase A: Scaffold ✓
Vite + Phaser 3 + TypeScript. Boot scene loads spritesheets. Verified sprites render.

### Phase B: Static board renderer ✓
15×15 grid with proper edge/corner/internal frame selection. Float `cellSize` — no
rounding gaps at any viewport size. Depth edge fills to screen bottom. Stone tinting
for black/white.

### Phase C: Human vs human ✓
Click-to-play, pointer hover animation, `stone-form` on placement, win highlight,
turn indicator, new game.

### Phase D: Replay viewer — deferred
The result screen shows all moves with sequence numbers on each stone. This covers
the core use case (reviewing who played what). Step-through prev/next controls can
be added minimally when needed — the `moveOrder` map already has all the data.

### Phase E: Wasm bridge ✓
`gomoku-wasm` built with `wasm-pack --target bundler`. `WasmBoard` and `WasmBot`
wrap Rust core. Vite loads `.wasm` via `vite-plugin-wasm`. Rust core drives all
game state in browser.

### Phase F: Bot play ✓ (and beyond original scope)
- Human vs bot, bot vs bot, human vs human
- Bot runs in a Web Worker (keeps UI thread responsive)
- Settings panel: Freestyle/Renju toggle, per-player Human/Bot toggle, inline name
  editing
- Player profiles decouple from color slots; profiles alternate color slots each round
- Per-player move timers with live delta; game timer; pending +1 win display
- Renju forbidden move overlays on relevant empty cells
- Stone + pointer idle animations
- Result screen with full move sequence labels
- Round transition: cards animate to swapped positions with tint lerp (500ms)
- Two-layer color system: `P` primitives → `COLOR` semantic roles; `shade()` and
  `lerpColor()` for dynamic tints
- Responsive 1200×900 / 900×1350 canvas with `Scale.FIT + CENTER_BOTH`
- Deployed to GitHub Pages

---

## What this validated for the framework

- Phaser as the 2D frontend layer
- Sprite asset pipeline: spritesheets → Phaser animations
- `wasm-pack --target bundler` + Vite plugin for Rust/Wasm integration
- Web Worker pattern for off-thread bot computation
- Single-scene design is practical for a game of this scope
- Replay JSON (`gomoku-cli`/`gomoku-eval`) is a solid cross-frontend contract
- Pylander reuse: same Phaser + Vite scaffold, same scene pattern, same asset
  pipeline, same Wasm bridge approach

---

## What's next

1. **Bot strength selector** — expose depth (or preset labels) in settings panel
2. **Stronger bot eval** — threat detection, 4+3 combo recognition, better
   positional scoring; validated against current baseline via `gomoku-eval` arena
3. **Replay step-through** (deferred) — minimal when prioritized: result screen
   already shows sequence, just needs prev/next navigation
