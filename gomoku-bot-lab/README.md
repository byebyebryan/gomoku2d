# gomoku-bot-lab

The Rust side of gomoku2d — a Cargo workspace where the rules engine and the
bots live, and where bot ideas can be tried out and measured. The web game
(`../gomoku-web/`) imports the compiled `gomoku-wasm` artifact and has no
other knowledge of what's in here.

## Crates

| Crate | What it does |
|-------|--------------|
| `gomoku-core` | Board state, rules (Freestyle + Renju), win detection, FEN, replay JSON |
| `gomoku-bot` | `Bot` trait + implementations: `RandomBot`, `SearchBot` |
| `gomoku-eval` | Self-play arena, round-robin tournaments, Elo |
| `gomoku-cli` | Native match runner with replay export |
| `gomoku-wasm` | `wasm-pack` bridge exposing `WasmBoard` + `WasmBot` to JS |

Dependency shape: `core` has zero deps; `bot` / `eval` / `cli` / `wasm` all
depend on `core`; `cli` / `eval` / `wasm` depend on `bot`.

## Build and test

```sh
cargo build --release --workspace
cargo test  --workspace
```

## Run a match

```sh
cargo run --release -p gomoku-cli -- --black baseline --white random
cargo run --release -p gomoku-cli -- --black baseline --white random --time-ms 500
cargo run --release -p gomoku-cli -- --black baseline --white random --quiet --replay /tmp/game.json
```

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--black` | `baseline` | Bot for Black: `random` or `baseline` |
| `--white` | `random`   | Bot for White: `random` or `baseline` |
| `--depth` | `5`        | Fixed baseline depth (ignored if `--time-ms` is set) |
| `--time-ms` | —        | Time budget per move in milliseconds |
| `--rule` | `freestyle` | Rule variant: `freestyle` or `renju` |
| `--replay` | —         | Write replay JSON to this path |
| `--quiet` | —          | Suppress per-move board printing |

### Current `SearchBot`

Negamax with alpha-beta pruning, iterative deepening, and a transposition table
keyed by incremental Zobrist hashing. Move candidates are pruned to cells within
2 steps of any existing stone. Static evaluation scores open and half-open runs
of 2–4 in all four directions. It reliably beats `RandomBot` and is intentionally
good enough for practice without trying to be a perfect Gomoku engine.

More detailed strategy notes live in [`../docs/bot_baseline.md`](../docs/bot_baseline.md).

## Replay format

Both `gomoku-cli` and `gomoku-eval` write the same JSON. The web game consumes
that replay format directly.

```json
{
  "hash_algo": { "algorithm": "xorshift64", "seed": 16045690984833335166 },
  "rules": { "board_size": 15, "win_length": 5, "variant": "freestyle" },
  "black": "baseline",
  "white": "random",
  "moves": [
    { "mv": "H8", "time_ms": 120, "hash": 123456789 },
    { "mv": "D4", "time_ms": 5,   "hash": 987654321, "trace": { "depth": 3 } }
  ],
  "result": "black_wins",
  "duration_ms": 3520
}
```

## Build the web bridge

From the repo root:

```sh
wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
```

This produces `gomoku-bot-lab/gomoku-wasm/pkg/`, which `gomoku-web/` consumes
via a `file:` dep.

## Adding a new bot

1. Add a module under `gomoku-bot/src/`
2. `impl Bot for YourBot`
3. Register it in the bot registry
4. The CLI can play it immediately; `gomoku-eval` can rate it; `gomoku-wasm`
   can ship it to the browser once surfaced through `WasmBot`
