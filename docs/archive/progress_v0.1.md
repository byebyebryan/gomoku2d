# Gomoku2D — Work Progress

## Done

**Repo layout** — `gomoku-web/` (the game) and `gomoku-bot-lab/` (Cargo workspace: `gomoku-core`, `gomoku-bot`, `gomoku-eval`, `gomoku-cli`, `gomoku-wasm`)

**gomoku-core**
- `Board` with `apply_move` / `undo_move` / `legal_moves` / `is_legal`
- Win detection: scans 4 directions from placed stone
- FEN serialization (`to_fen` / `from_fen`) for board state snapshots
- `Replay` struct — JSON in/out via serde, includes rules, player names, move list, result, duration
- `Variant` enum: `Freestyle` (default) and `Renju`; stored in `RuleConfig.variant`
- Renju restrictions for Black: overline (6+) forbidden, double-four forbidden, double-three forbidden; winning moves (exactly 5) always allowed; White unrestricted
- `MoveError::Forbidden` for Renju violations; `is_legal` and `legal_moves` respect restrictions
- 18 unit tests (win detection, move errors, FEN round-trip, game-over guard, all Renju cases)

**gomoku-bot**
- `Bot` trait: `name() + choose_move(&Board) -> Move`
- `RandomBot` — uniform random over legal moves, seedable for tests
- `SearchBot` (`"baseline"`) — negamax + alpha-beta + iterative deepening + transposition table
  - Incremental Zobrist hashing (O(1) per node, not O(225))
  - Candidate move pruning: only cells within radius 2 of existing stones
  - Pattern eval: scores open/half-open runs of 2–4 in all 4 directions
  - `--depth` or `--time-ms` budget; exposes `last_info` (depth reached, nodes, score)
  - Strategy + known limitations: `docs/search_bot.md`
- 3 unit tests (legal move guarantee, finds immediate win, blocks opponent win)

**gomoku-eval**
- `Arena` — runs a single match between two bots with per-move timing and replay capture
- `Tournament` — round-robin across a bot lineup, configurable games per pairing
- `Elo` — standard Elo rating update from match results; initialized at 1500

**gomoku-cli**
- `--black`/`--white` (`random`|`search`), `--depth`, `--time-ms`, `--replay <path>`, `--quiet`
- ASCII board printed before each move, move log, final result + elapsed time

**gomoku-wasm**
- `WasmBoard` wraps `gomoku-core::Board` — `new()`, `createWithVariant()`, `applyMove()`, `isLegal()`, `cell()`, `currentPlayer()`, `result()`, `legalMoves()`, `undoLastMove()`, `toFen()`, `fromFen()`, `cloneBoard()`, `moveCount()`
- `WasmBot` wraps `RandomBot` or `SearchBot` — `createRandom()`, `createBaseline(depth)`, `chooseMove()`, `name()`
- Built with `wasm-pack build --target bundler` — Vite handles WASM loading via `vite-plugin-wasm`
- `console_error_panic_hook` for Wasm panic messages; `getrandom` js feature for `rand` crate

**gomoku-web** (Phaser 3 + TypeScript + Vite)
- Boot scene: preloads all spritesheets + bitmap font, registers all animations
- Board renderer: float `cellSize` fills screen edge-to-edge with no rounding gaps; depth edge fills to screen bottom
- All game state via `WasmBoard`; Renju rule enforcement via `createWithVariant("renju")`
- Human vs bot, bot vs bot, human vs human — bots use `WasmBot.createBaseline(3)`
- Bot moves run in a Web Worker to keep the UI thread responsive
- Settings panel: Freestyle / Renju toggle, per-player Human/Bot toggle, inline name editing for human players (click HUMAN button to type)
- Player profiles decoupled from color slots; profiles alternate color slots after each completed round
- Per-player move timers with live delta display; game timer; pending +1 win display
- Renju forbidden move overlays (red `warning_l` animation) on empty cells within radius 2 of existing stones — shown only when human black is to move
- Pointer idle animations: random cycle of out/in/full animations with static pauses; persists across cell transitions
- Stone idle animations: random relax-1/2/3/4 on the last-placed stone only; transfers on each new placement
- Win highlight: green `warning_l` animation on winning cells
- Result screen with full move sequence (all stones labeled with move order numbers in contrasting color)
- Round transition: player cards animate to swapped positions with background/text tint lerp over 500ms; board clears immediately at start
- Two-layer color system: primitive `P` palette → semantic `COLOR` map; `shade()` for hover/press tints via float factor (>1.0 brightens with channel clamp), `lerpColor()` for smooth transitions
- Responsive fixed canvas targets: 1200×900 (landscape, 4:3) and 900×1350 (portrait, 2:3), using `Phaser.Scale.FIT` + `CENTER_BOTH`
- Deployed to GitHub Pages via manually triggered `deploy.yml` workflow (Rust + Node cached between runs)

---

## Up next

### Bot picker — lab → game bridge

The README frames this project as a bot lab plus a web game joined by a shared
core. In practice the bridge is thin: the web game hardcodes
`WasmBot.createBaseline(3)` behind a HUMAN/BOT toggle. Adding a new bot to
`gomoku-bot` doesn't surface anywhere a player can see it.

**Goal:** per-slot picker in settings with a small list of bot presets. Making
adding a preset a one-line change.

**Non-goals (for now):** auto-generated UI from a Rust-side bot registry
(worth doing once there are more than ~3 bots); free-form depth/time sliders
(presets stay friendlier); localStorage persistence (in-memory is enough).

**UI.** Replace each slot's HUMAN/BOT toggle with a row of options:

```
Player 1: [ HUMAN ] [ RANDOM ] [ EASY ] [ MEDIUM ] [ STRONG ]
Player 2: [ HUMAN ] [ RANDOM ] [ EASY ] [ MEDIUM ] [ STRONG ]
```

HUMAN keeps the inline name editor. Other options select a bot preset; the
player card name shows the preset label (e.g. `MEDIUM BOT`) with no editor.
Fall back to a dropdown-style picker in portrait if the row gets too wide.

**Presets.** One place in TypeScript:

```ts
// gomoku-web/src/core/bot_presets.ts
export const BOT_PRESETS: BotPreset[] = [
  { id: "random", label: "RANDOM", spec: { kind: "random" } },
  { id: "easy",   label: "EASY",   spec: { kind: "baseline", depth: 2 } },
  { id: "medium", label: "MEDIUM", spec: { kind: "baseline", depth: 3 } },
  { id: "strong", label: "STRONG", spec: { kind: "baseline", depth: 5 } },
];
```

Depths are placeholders — calibrate them via `gomoku-eval` before shipping.

**Profile interaction.** Preset lives with the profile, not the color slot —
if Player 1 is STRONG and plays Black this round, they're still STRONG when
they play White next round. `botRunner.configure` is called with
`[blackProfile.spec, whiteProfile.spec]` on each round start.

**Changes.**
- `bot_protocol.ts`: extend `BotSpec` with `| { kind: "random" }`
- `bot_worker.ts`: `buildBot()` switch over `kind`
- `bot_presets.ts` (new): the preset list above
- `ui.ts`: replace HUMAN/BOT toggle row with preset picker; bot name display
- `game.ts`: track per-profile preset, reconfigure worker on round start
- No Rust changes needed — `createRandom()` and `createBaseline(depth)` already
  exist

**Calibration.** Before shipping, run a round-robin with `gomoku-eval` between
RandomBot and Baseline at depths 2–5. Check Elo spread and response-time at
each depth on the dev machine; if STRONG takes >3s per move, downgrade or
switch to a time budget. Doubles as a worked example of the eval pipeline.

**Order of work.**
1. Extend `BotSpec` + `bot_worker.ts` to handle `kind: "random"`
2. Add `bot_presets.ts`
3. Wire the preset picker into `PlayerCard` settings UI
4. Update `game.ts` to track per-profile preset and reconfigure on round start
5. Run calibration sweep, set final depths
6. Move this item to Done

### Other

- `gomoku-bot`: stronger eval — threat detection, 4+3 combos, better positional
  scoring (feeds into bot picker calibration — a stronger baseline means better
  spacing between preset levels).
- `gomoku-web`: replay step-through viewer — deferred. Result screen move
  sequence already covers the core need; adding prev/next controls is minimal
  when prioritized.

### Future hooks (not scheduled)

- Rust-side bot registry exposing `wasm_bots()` → drives the web picker
  automatically. Worth it once there are 4+ bots.
- `WasmBot.createBaselineTimed(ms)` so STRONG is bounded by response time, not
  depth.
- Bot trace overlay in dev mode (depth/nodes/score from `last_info`).
