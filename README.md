# gomoku2d

A Gomoku engine and game framework sandbox — built to validate a reusable architecture for native core + bot + eval + web frontend before applying it to larger projects.

**Stack:** Rust (Cargo workspace) · PlayCanvas + TypeScript (web, Phase 2) · wasm-pack (Phase 2)

---

## Workspace

| Crate | Status | Description |
|-------|--------|-------------|
| `gomoku-core` | ✅ | Board state, rules, win detection, FEN serialization, replay/JSON schema |
| `gomoku-bot` | ✅ | `Bot` trait, `RandomBot`, `SearchBot` (negamax + α-β + iterative deepening + TT) |
| `gomoku-cli` | ✅ | Match runner — bots, ASCII board, replay export |
| `gomoku-eval` | 🔜 | Self-play, tournaments, Elo (Phase 2) |
| `gomoku-web` | 🔜 | PlayCanvas + TypeScript frontend (Phase 2) |
| `gomoku-wasm` | 🔜 | wasm-pack bridge exposing core to JS (Phase 2) |

---

## Quick start

```sh
# Build everything
cargo build --release --workspace

# Run tests
cargo test --workspace

# Search bot vs random (default depth 5)
cargo run --release -p gomoku-cli -- --black baseline --white random

# Random vs random
cargo run --release -p gomoku-cli -- --black random --white random

# Search vs baseline
cargo run --release -p gomoku-cli -- --black baseline --white baseline

# Quiet output + save replay
cargo run --release -p gomoku-cli -- --black baseline --white random --quiet --replay /tmp/game.json

# Time-budgeted baseline (500ms per move)
cargo run --release -p gomoku-cli -- --black baseline --white random --time-ms 500
```

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--black` | `baseline` | Bot for Black: `random` or `baseline` |
| `--white` | `random` | Bot for White: `random` or `baseline` |
| `--depth` | `5` | Fixed baseline depth (ignored if `--time-ms` is set) |
| `--time-ms` | — | Time budget per move in milliseconds |
| `--replay` | — | Write replay JSON to this path |
| `--rule` | `freestyle` | Rule variant: `freestyle` or `renju` |
| `--quiet` | — | Suppress per-move board printing |

---

## Rules

- 15×15 board
- First to get 5 in a row (horizontal, vertical, or diagonal) wins
- Black always goes first
- Standard Gomoku — no Renju restrictions

---

## SearchBot

Negamax with alpha-beta pruning, iterative deepening, and a transposition table keyed by incremental Zobrist hashing. Move candidates are pruned to cells within 2 steps of any existing stone. Static evaluation scores open and half-open runs of 2–4 in all four directions.

---

## Replay format

```json
{
  "hash_algo": {
    "algorithm": "xorshift64",
    "seed": 16045690984833335166
  },
  "rules": { "board_size": 15, "win_length": 5, "variant": "freestyle" },
  "black": "baseline",
  "white": "random",
  "moves": [
    { "mv": "H8", "time_ms": 120, "hash": 123456789 },
    { "mv": "D4", "time_ms": 5, "hash": 987654321, "trace": { "depth": 3 } }
  ],
  "result": "black_wins",
  "duration_ms": 3520
}
```

---

## Project goals

See [`docs/gomoku.md`](docs/gomoku.md) for the full design rationale and [`docs/game_framework.md`](docs/game_framework.md) for the generic architecture this project validates. Progress is tracked in [`docs/progress.md`](docs/progress.md).
