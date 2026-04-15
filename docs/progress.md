# Gomoku2D — Work Progress

## Done

**Cargo workspace** — `gomoku-core`, `gomoku-bot`, `gomoku-eval` (stub), `gomoku-cli`, `gomoku-wasm` (stub)

**gomoku-core**
- `Board` with `apply_move` / `undo_move` / `legal_moves` / `is_legal`
- Win detection: scans 4 directions from placed stone
- FEN serialization (`to_fen` / `from_fen`) for board state snapshots
- `Replay` struct — JSON in/out via serde, includes rules, player names, move list, result, duration
- 10 unit tests (win detection, move errors, FEN round-trip, game-over guard)

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

- [ ] `gomoku-eval`: self-play runner — N games between two bots, win/loss/draw counts
- [ ] `gomoku-eval`: basic Elo after a round-robin
- [ ] `gomoku-web`: PlayCanvas + TypeScript project scaffold
- [ ] `gomoku-wasm`: wasm-pack bridge exposing core to JS
