# gomoku-bot-lab

The lab under the board: a Rust workspace for Gomoku2D's rules engine, bots,
replay analysis, evaluation harnesses, and WebAssembly bridge.

The web game imports the compiled `gomoku-wasm` package. Native crates own the
rules/search/analyzer contracts first, then expose stable pieces to the browser.

## Crates

| Crate | Role |
|---|---|
| `gomoku-core` | Board state, Freestyle/Renju rules, win detection, replay primitives |
| `gomoku-bot` | `Bot` trait, `SearchBot`, tactical facts, corridor proof helpers |
| `gomoku-analysis` | Shared replay-analysis model and bounded corridor traceback |
| `gomoku-eval` | Self-play arena, tournaments, reports, scenario runners |
| `gomoku-lab-support` | Shared scenario boards and fixtures |
| `gomoku-cli` | Native match runner with replay export |
| `gomoku-wasm` | wasm-pack bridge consumed by `gomoku-web` |

Dependency shape: `core` stays small; bot/analysis/eval/cli/wasm depend on it.
Shared fixtures live in `gomoku-lab-support` when multiple crates need the same
board states.

## Build And Test

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --workspace
```

## Run A Match

```sh
cargo run --release -p gomoku-cli -- --black search-d3 --white random
cargo run --release -p gomoku-cli -- --black search-d3 --white search-d1 --rule renju
cargo run --release -p gomoku-cli -- --black search-d3 --white random --quiet --replay outputs/game.json
```

Useful flags:

| Flag | Meaning |
|---|---|
| `--black`, `--white` | `random`, `search-dN`, or explicit lab specs |
| `--rule` | `freestyle` or `renju` |
| `--time-ms` | wall-clock search budget for search bots |
| `--replay` | write replay JSON |
| `--quiet` | suppress per-move board output |

## Current Model

- `SearchBot` is negamax with alpha-beta pruning, iterative deepening, TT,
  tactical ordering, and rolling threat facts.
- Product presets resolve to tested lab specs; presets are not parser concepts.
- Corridor search is primarily a replay-analysis foundation.
- Renju legality is owned by `gomoku-core` and validated through the promoted
  Renju corpus before consumers trust tactical effects.

Key docs:

- [`Bot Lab Code Overview`](../docs/reference/lab/code_overview.md)
- [`Search Bot`](../docs/reference/lab/search_bot.md)
- [`Tactical Shapes`](../docs/reference/lab/tactical_shapes.md)
- [`Lethal Threats`](../docs/reference/lab/lethal_threats.md)
- [`Corridor Search`](../docs/reference/lab/corridor_search.md)
- [`Game Analysis`](../docs/reference/lab/game_analysis.md)
- [`Renju Rules`](../docs/reference/lab/renju_rules.md)

## Eval Harness

`gomoku-eval` owns tournaments, scenario sweeps, lethal checks, and
replay-analysis report generation. Scratch output belongs in ignored
`outputs/`; curated published report JSON lives in `reports/` and
`analysis-reports/`.

Common diagnostics:

```sh
cargo run -p gomoku-eval -- tactical-scenarios
cargo run -p gomoku-eval -- lethal-scenarios
cargo run -p gomoku-eval -- renju-rules
```

Tournament and report commands live in
[`Tournament Eval`](../docs/reference/ops/tournament.md). Do not duplicate
long report commands here.

## Web Bridge

From the repo root:

```sh
wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
```

This produces `gomoku-bot-lab/gomoku-wasm/pkg/`, consumed by `gomoku-web` via a
local `file:` dependency.

## Adding A Bot

1. Add the bot under `gomoku-bot/src/`.
2. Implement `Bot`.
3. Register the parser/config path if it should be used by CLI/eval.
4. Add wasm/product exposure only after the lab config has a reason to ship.
