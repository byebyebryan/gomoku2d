# Gomoku2D — Work Progress

## Done

**Cargo workspace** — `gomoku-core`, `gomoku-bot`, `gomoku-eval` (stub), `gomoku-cli`, `gomoku-wasm` (stub)

**gomoku-core**
- `Board` with `apply_move` / `undo_move` / `legal_moves` / `is_legal`
- Win detection: scans 4 directions from placed stone
- FEN serialization (`to_fen` / `from_fen`) for board state snapshots
- `Replay` struct — JSON in/out via serde, includes rules, player names, move list, result, duration
- `Variant` enum: `Freestyle` (default) and `Renju`; stored in `RuleConfig.variant` (serde default = freestyle, backward-compatible)
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

**gomoku-cli**
- `--black`/`--white` (`random`|`search`), `--depth`, `--time-ms`, `--replay <path>`, `--quiet`
- ASCII board printed before each move, move log, final result + elapsed time

---

## Up next

- [x] `gomoku-eval`: self-play runner — N games between two bots, win/loss/draw counts
- [x] `gomoku-eval`: basic Elo after a round-robin
- [x] `gomoku-web`: Phaser 3 + TypeScript + Vite project scaffold
- [x] `gomoku-web`: Phase B — static board renderer (grid, stones, pointer)
- [x] `gomoku-web`: Phase C — click-to-play human vs human (win detection, highlighting, reset)
- [x] `gomoku-wasm`: Phase E — wasm-pack bridge exposing Board + RandomBot to JS
- [x] `gomoku-web`: Phase E — game.ts refactored to use WasmBoard for all game state
- [ ] `gomoku-web`: Phase F — bot spectator (human vs bot, bot vs bot)

---

## Phase E details

**gomoku-wasm** (Rust):
- `WasmBoard` wraps `gomoku-core::Board` — `new()`, `applyMove()`, `isLegal()`, `cell()`, `currentPlayer()`, `result()`, `legalMoves()`, `undoLastMove()`, `toFen()`, `fromFen()`, `cloneBoard()`, `moveCount()`
- `WasmBot` wraps `RandomBot` or `SearchBot` — `createRandom()`, `createBaseline(depth)`, `chooseMove()`, `name()`
- `console_error_panic_hook` for Wasm panic messages
- `getrandom` js feature for `rand` crate in wasm32
- Built with `wasm-pack build --target web` — 52KB .wasm binary
- `SearchBot` exposed via `instant` crate polyfill for `std::time::Instant` (wasm-safe)

**gomoku-web integration**:
- `src/core/wasm_bridge.ts` — async `initWasm()` singleton, re-exports `WasmBoard`/`WasmBot`
- `src/main.ts` — awaits Wasm init before creating Phaser game
- `src/scenes/game.ts` — all game state via `WasmBoard`; `checkWin()` still in TS for winning cell highlighting
- Installed as npm `file:` dependency from `../gomoku-wasm/pkg/`
