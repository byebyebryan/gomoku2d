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

Full CLI flag list and `SearchBot` strategy notes live in the [root README](../README.md#the-bot-lab) and [`docs/bot_baseline.md`](../docs/bot_baseline.md).

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
