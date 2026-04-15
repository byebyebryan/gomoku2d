# Web Frontend Plan (Phaser 3)

**Date:** 2026-04-15
**Purpose:** concrete implementation plan for `gomoku-web`
**Tech:** Phaser 3 + TypeScript + Vite

---

## 1. Why Phaser

| Concern | Phaser | PlayCanvas |
|---------|--------|------------|
| 2D spritesheet animation | Built-in, first-class | Supported but 3D-centric API |
| Click/hover input | `setInteractive()` per sprite | Raycasting against 3D entities |
| Physics (Pylander future) | Arcade built-in, Matter.js plugin | No built-in, bring your own |
| Game loop / scene mgmt | Built-in Phaser scenes | Built-in but 3D-oriented |
| Bundle size | ~1MB | ~1MB |
| Community / examples | 50k+ stars, massive example library | Smaller, professional |

Phaser is the stronger 2D tool. PlayCanvas is the stronger 3D tool.
Both Gomoku and Pylander are 2D. Switch to PlayCanvas if a future
project genuinely needs 3D.

---

## 2. Project structure

```
gomoku-web/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── public/
│   └── assets/
│       └── sprites/          ← symlink or copy from /assets/sprites
├── src/
│   ├── main.ts               ← Phaser game config + boot
│   ├── scenes/
│   │   ├── boot.ts           ← asset preloading
│   │   ├── menu.ts           ← main menu
│   │   ├── game.ts           ← active gameplay
│   │   └── replay.ts         ← replay viewer
│   ├── board/
│   │   ├── board_renderer.ts ← grid, stones, pointer, warning
│   │   └── animations.ts     ← stone/pointer/warning anim defs
│   ├── core/
│   │   ├── game_state.ts     ← local board state (pre-Wasm)
│   │   └── wasm_bridge.ts    ← Wasm integration (Phase 5)
│   └── replay/
│       └── replay_player.ts  ← step through replay JSON
```

---

## 3. Implementation phases

### Phase A: Scaffold

- Vite + Phaser 3 + TypeScript project
- Phaser game config with scene list
- Boot scene that loads spritesheet assets
- Verify sprites render (place a stone on screen)

**Done when:** `npm run dev` shows a Phaser canvas with one stone sprite.

### Phase B: Static board renderer

- Draw 15×15 grid using `gomoku_grid_half_line.png` frames
  - Edge/corner cells render subset of directions (see manifest)
  - Internal cells render all 4 half-lines
- Place black/white stones using `gomoku_stone.png` frame 0 (static)
- Z-sorting: grid → pointer → stone (see manifest for formula)
- Board centered on canvas

**Done when:** Empty board renders correctly with proper grid lines.

### Phase C: Click-to-play (human vs human)

- Click on empty cell → place stone for current player
- Cell hover shows pointer sprite (`gomoku_pointer.png`)
- Stone placement plays `stone-form` animation
- Win detection highlights winning stones with `gomoku_warning_l.png`
- Basic turn indicator (whose move)
- Reset / new game button

**Done when:** Two humans can play a full game on the same screen.

### Phase D: Replay viewer

- Load replay JSON (paste or file picker)
- Replay player controls: step forward, step back, play/pause, speed
- Board renders each move in sequence
- Stone animations play on placement
- Move list sidebar (optional)

**Done when:** Any replay JSON from `gomoku-cli` or `gomoku-eval` can be
loaded and stepped through.

### Phase E: Wasm bridge

- Build `gomoku-wasm` with `wasm-pack`
- Replace local TypeScript game state with Wasm core calls
- Validate: browser games produce identical state to native core
- Bot moves via Wasm (RandomBot first, then SearchBot)

**Done when:** Browser play uses the same Rust core via Wasm.

### Phase F: Bot spectator

- Human vs bot (human picks color)
- Bot vs bot (both automated, with move-by-move replay)
- Spectator mode: watch eval tournament replays
- Thinking time display per move
- Match result summary

**Done when:** Can watch a bot vs bot game in the browser.

---

## 4. Sprite asset mapping

All sprites are 16×16 per frame, horizontal strips. Phaser spritesheet
config:

```typescript
// gomoku_stone.png: 544×16 = 34 frames
{ key: 'stone', url: 'sprites/gomoku_stone.png', frameConfig: { frameWidth: 16, frameHeight: 16 } }

// gomoku_pointer.png: 304×16 = 19 frames
{ key: 'pointer', url: 'sprites/gomoku_pointer.png', frameConfig: { frameWidth: 16, frameHeight: 16 } }

// etc.
```

Animation definitions from manifest:

```typescript
// stone-form: frames 25-33, 18fps
// stone-destroy: frames 0-3, 12fps
// stone-relax-1: frames 0-6, 6fps
// stone-relax-2: frames 6-12, 6fps
// stone-relax-3: frames 12-18, 6fps
// stone-relax-4: frames 18-24, 6fps
```

Phaser animation creation:

```typescript
this.anims.create({
  key: 'stone-form',
  frames: this.anims.generateFrameNumbers('stone', { start: 25, end: 33 }),
  frameRate: 18,
});
```

---

## 5. Board layout (from manifest)

```
Board origin: centered at canvas center
Cell [x,y] pixel position: center + (x - 7) * cellSize, center + (y - 7) * cellSize
Cell [0,0] = top-left, Cell [14,14] = bottom-right
Z-sorting: base = -y * 10, grid +1, pointer +4, warning +3, stone +6
```

---

## 6. Canvas sizing

Gomoku is a 15×15 grid of 16px cells. At 1x scale that's 240×240px —
tiny. Recommended: scale up to fit viewport with a minimum cell size of
32-48px.

```
cellSize = Math.floor(Math.min(viewportWidth, viewportHeight) * 0.8 / 15)
```

---

## 7. Dependencies

```json
{
  "dependencies": {
    "phaser": "^3.80"
  },
  "devDependencies": {
    "typescript": "^5",
    "vite": "^5"
  }
}
```

No other runtime deps. Wasm integration adds `gomoku-wasm` pkg later.

---

## 8. What this validates for the framework

Building gomoku-web with Phaser validates:

- Phaser as the 2D frontend layer of the framework
- Sprite asset pipeline (spritesheets → Phaser animations)
- Scene management pattern (boot → menu → game/replay)
- Local state management (and later Wasm bridge)
- Replay JSON as a cross-frontend contract
- Click input → core action → state update → re-render cycle

What Pylander would reuse:
- Same Phaser + Vite scaffold
- Same scene pattern
- Same game loop (but with Arcade physics)
- Same asset pipeline
- Proven that the frontend layer is swappable
