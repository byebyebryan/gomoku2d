# Gomoku2D — Work Progress

## Done

**Cargo workspace** — `gomoku-core`, `gomoku-bot`, `gomoku-eval`, `gomoku-cli`, `gomoku-wasm`, `gomoku-web`

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
  - Strategy + known limitations: `docs/bot_baseline.md`
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
- Settings panel: Freestyle / Renju toggle, per-player Human/Bot toggle, inline name editing for human players (click HUMAN button to type)
- Player profiles decoupled from color slots; color slots swap each game (loser opens as black)
- Per-player move timers with live delta display; game timer; pending +1 win display
- Renju forbidden move overlays (red `warning_l` animation) on empty cells within radius 2 of existing stones — shown only when human black is to move
- Pointer idle animations: random cycle of out/in/full animations with static pauses; persists across cell transitions
- Stone idle animations: random relax-1/2/3/4 on the last-placed stone only; transfers on each new placement
- Win highlight: green `warning_l` animation on winning cells
- Responsive fixed canvas targets: 1200×900 (landscape, 4:3) and 900×1350 (portrait, 2:3), using `Phaser.Scale.FIT` + `CENTER_BOTH`

---

## Up next

- `gomoku-web`: replay viewer — load replay JSON, step through moves
- `gomoku-web`: stronger bot option (depth 5+) selectable in settings
