# gomoku-bot-lab

The lab under the board.

This Rust workspace owns the game rules, Renju legality, bot search, tactical
facts, replay-analysis model, tournament/eval harness, native CLI, and
WebAssembly bridge consumed by `gomoku-web`.

The browser app is the product surface. The lab is where game understanding is
made explicit before it is shipped to the browser.

## Why It Exists

Gomoku2D is not only a board with a bot. The distinctive features come from
being able to run positions outside the live browser game:

- configurable search bots backed by native tournament results;
- tactical hints from the same threat model used by search and analysis;
- Replay Analysis that walks backward through lethal onset, setup corridor, and
  last escape;
- curated Lab reports that make bot behavior and analyzer output inspectable;
- future puzzles or lessons generated from real replay positions.

That is why rules/search/analysis live here first and cross into the web app
through wasm only after the contract is stable enough to expose.

## Workspace Map

| Crate | Role |
|---|---|
| `gomoku-core` | Board state, Freestyle/Renju rules, win detection, replay primitives |
| `gomoku-bot` | `Bot` trait, `SearchBot`, tactical facts, rolling threat view, corridor proof helpers |
| `gomoku-analysis` | Replay traceback, lethal onset, setup corridor, failure classification, annotations |
| `gomoku-eval` | Tournaments, scenario checks, report JSON generation, analysis batches |
| `gomoku-lab-support` | Shared scenario boards and fixtures |
| `gomoku-cli` | Native match runner and replay exporter |
| `gomoku-wasm` | `wasm-pack` bridge consumed by `gomoku-web` |

Dependency rule: `gomoku-core` owns rules and replay foundations. Consumers
should depend on it rather than reimplementing board, legality, win, or replay
logic.

## Current Model

- `SearchBot` uses negamax with alpha-beta pruning, iterative deepening,
  transposition table, tactical ordering, rolling threat facts, pattern
  evaluation, and optional root-level corridor proof.
- Product bot presets resolve to tested lab specs. Presets are product labels;
  the lab parser still works with explicit specs such as
  `search-d7+tactical-cap-8+pattern-eval`.
- Corridor search is primarily the replay-analysis foundation. The bot uses a
  narrower proof pass where it has tested value.
- Renju legality is owned by `gomoku-core` and checked against promoted corpus
  fixtures before search, analysis, or wasm hints trust the result.
- Scan-backed threat logic remains a correctness reference; hot paths should
  prefer rolling threat facts when the behavior is equivalent.

## Build And Test

From `gomoku-bot-lab/`:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --workspace
```

Focused diagnostics:

```sh
cargo run --release -p gomoku-eval -- tactical-scenarios
cargo run --release -p gomoku-eval -- lethal-scenarios
cargo run -p gomoku-eval -- renju-rules
```

Use scenario/corpus tests for behavior categories. Avoid adding one-off replay
debug tests unless the replay is promoted into a curated fixture with a clear
reason.

## Run A Native Match

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

## Eval And Reports

`gomoku-eval` owns tournaments, scenario sweeps, lethal checks, Renju corpus
checks, bot report generation, and replay-analysis report generation.

Scratch output belongs under ignored `outputs/`. Curated published report data
belongs at repo root:

- `../reports/lab/bot-report.json`
- `../reports/lab/analysis-report.json`

The web build copies those files into `dist/` and renders them through the
unified `/lab/` route. Canonical tournament/report commands live in
[`../docs/reference/ops/tournament.md`](../docs/reference/ops/tournament.md);
do not duplicate long command lines here.

## WebAssembly Bridge

From the repo root:

```sh
wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
```

This produces `gomoku-bot-lab/gomoku-wasm/pkg/`, consumed by `gomoku-web` via a
local `file:` dependency.

`gomoku-wasm` should translate data across the JS boundary only. It should not
own independent rules, bot, or analyzer logic. Any wasm payload change must be
updated on the TypeScript validation side in `gomoku-web/src/core/wasm_bridge.ts`
or the replay analysis protocol files.

## Adding Or Shipping A Bot Change

1. Change the Rust config/search code in `gomoku-bot`.
2. Validate with focused tests/scenarios and a small eval before broad runs.
3. Add parser support only if the config needs CLI/eval access.
4. Add wasm/product exposure only when the lab config has a reason to ship.
5. Refresh curated reports if anchor behavior changes.

## Deeper References

- [`../docs/reference/lab/code_overview.md`](../docs/reference/lab/code_overview.md)
- [`../docs/reference/lab/search_bot.md`](../docs/reference/lab/search_bot.md)
- [`../docs/reference/lab/tactical_shapes.md`](../docs/reference/lab/tactical_shapes.md)
- [`../docs/reference/lab/lethal_threats.md`](../docs/reference/lab/lethal_threats.md)
- [`../docs/reference/lab/corridor_search.md`](../docs/reference/lab/corridor_search.md)
- [`../docs/reference/lab/game_analysis.md`](../docs/reference/lab/game_analysis.md)
- [`../docs/reference/lab/renju_rules.md`](../docs/reference/lab/renju_rules.md)
- [`../docs/reference/ops/tournament.md`](../docs/reference/ops/tournament.md)
