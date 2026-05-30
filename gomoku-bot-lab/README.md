# gomoku-bot-lab

The lab under the board: a Rust workspace where Gomoku2D's rules engine, bots,
replay format, benchmarks, and WebAssembly bridge live.

The web game (`../gomoku-web/`) imports the compiled `gomoku-wasm` artifact and
stays deliberately ignorant of the native lab internals. That keeps the browser
surface simple while letting core game logic and AI experiments evolve here
first.

## Crates

| Crate | What it does |
|-------|--------------|
| `gomoku-core` | Board state, rules (Freestyle + Renju), win detection, FEN, replay JSON |
| `gomoku-bot` | `Bot` trait + implementations: `RandomBot`, `SearchBot`, plus corridor proof helpers |
| `gomoku-analysis` | Shared replay-analysis model and bounded corridor traceback |
| `gomoku-eval` | Self-play arena, round-robin tournaments, Elo |
| `gomoku-lab-support` | Shared scenario boards and fixtures for tests, reports, and perf work |
| `gomoku-cli` | Native match runner with replay export |
| `gomoku-wasm` | `wasm-pack` bridge exposing `WasmBoard`, `WasmBot`, and replay analysis to JS |

Dependency shape: `core` is intentionally small and only carries serde-style
data dependencies; `bot` / `analysis` / `eval` / `cli` / `wasm` all depend on
`core`; `analysis`, `cli`, `eval`, and `wasm` depend on `bot`; `eval`, `wasm`,
and tests consume `lab-support` where shared scenario fixtures are useful.

## Build and test

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --workspace
cargo test  --workspace
```

## Run a match

```sh
cargo run --release -p gomoku-cli -- --black search-d3 --white random
cargo run --release -p gomoku-cli -- --black search-d3 --white search-d1 --rule renju
cargo run --release -p gomoku-cli -- --black search-d5 --white random --time-ms 500
cargo run --release -p gomoku-cli -- --black search-d3 --white random --quiet --replay /tmp/game.json
```

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--black` | `search-d3` | Bot for Black: `random`, `search-dN`, lab specs, or legacy `baseline` aliases |
| `--white` | `random`   | Bot for White: `random`, `search-dN`, lab specs, or legacy `baseline` aliases |
| `--depth` | `5`        | Fixed depth for the legacy plain `baseline` spec |
| `--time-ms` | —        | Time budget per move for search bots, including lab aliases |
| `--rule` | `freestyle` | Rule variant: `freestyle` or `renju` |
| `--replay` | —         | Write replay JSON to this path |
| `--quiet` | —          | Suppress per-move board printing |

### Current search and analysis model

The lab primarily uses explicit `search-*` specs over `SearchBotConfig`.
`gomoku-bot` owns the engine and config; product presets resolve to tested
specs but are not parser concepts.

Current defaults:

- `SearchBot` is negamax with alpha-beta pruning, iterative deepening, a
  transposition table, tactical ordering, and a rolling threat-view backend.
- The web game exposes Easy / Normal / Hard plus narrow advanced controls.
- Corridor search is a replay-analysis foundation first. The failed live
  portal experiments remain historical evidence, not current bot surface.
- Renju legality lives in `gomoku-core` and is validated through the Renju
  corpus before it is trusted by bot/search/report code.

Detailed references:

- [`Bot Lab Code Overview`](../docs/reference/lab/code_overview.md): crate
  map, API boundaries, and common change paths.
- [`Search Bot`](../docs/reference/lab/search_bot.md): config axes, current
  specs, rolling frontier, tactical ordering, and retired suffixes.
- [`Corridor Search`](../docs/reference/lab/corridor_search.md): setup
  corridor, exits, proof boundaries, and search reuse.
- [`Game Analysis`](../docs/reference/lab/game_analysis.md): replay analyzer
  contract, failure modes, and product interpretation.
- [`Performance Tuning`](../docs/working/performance_tuning.md): current
  benchmark snapshots and rejected optimization paths.

### Eval harness

`gomoku-eval` runs focused head-to-heads, candidate gauntlets, full round-robin
tournaments, tactical scenario sweeps, lethal scenario checks, and replay-analysis
reports. Keep scratch output in ignored `outputs/`; curated published reports
live in `reports/` and `analysis-reports/`.

Common commands from `gomoku-bot-lab/`:

```sh
# quick focused comparison
cargo run --release -p gomoku-eval -- tournament \
  --schedule head-to-head \
  --bots search-d3,search-d3+pattern-eval \
  --games-per-pair 16 \
  --opening-policy centered-suite \
  --opening-plies 4 \
  --search-cpu-time-ms 1000 \
  --report-json outputs/head-to-head.json

# export a compact published report from a full tournament report
cargo run --release -p gomoku-eval -- report-json \
  --input outputs/head-to-head.json \
  --output outputs/head-to-head-published.json

# replay-analysis smoke from a compact published report with replay cells
cargo run --release -p gomoku-eval -- analyze-report-replays \
  --report outputs/head-to-head-published.json \
  --sample-size 8 \
  --report-json outputs/analysis/top2-smoke.json

# tactical and lethal diagnostic harnesses
cargo run -p gomoku-eval -- tactical-scenarios
cargo run -p gomoku-eval -- lethal-scenarios
```

Runbook and model docs:

- [`Tournament Eval`](../docs/reference/ops/tournament.md): schedules,
  opening suite, report process, and published-report command.
- [`Tactical Scenario Corpus`](../docs/reference/corpora/tactical_scenarios.md):
  focused one-move tactical fixtures and expected moves.
- [`Lethal Threats`](../docs/reference/lab/lethal_threats.md): lethal
  classifier semantics and scenario harness.
- [`Game Analysis`](../docs/reference/lab/game_analysis.md): replay-analysis
  contract and report interpretation.

## Replay format

Both `gomoku-cli` and `gomoku-eval` write the same replay JSON. The web game
consumes that format directly.

```json
{
  "rules": { "board_size": 15, "win_length": 5, "variant": "freestyle" },
  "black": "search-d3",
  "white": "random",
  "moves": [
    { "mv": "H8", "time_ms": 120, "hash": 123456789 }
  ],
  "result": "black_wins",
  "duration_ms": 3520
}
```

Bot-produced moves may include a `trace` object with search config, node,
budget, and metric counters. Compact web-saved matches use `move_cells`
(`row * 15 + col`) for storage and report reconstruction.

## Build the web bridge

From the repo root:

```sh
wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
```

This produces `gomoku-bot-lab/gomoku-wasm/pkg/`, which `gomoku-web/` consumes
via a `file:` dep.

## Performance tuning

Benchmark process, fixed scenario corpus, and baseline snapshots live in
[`../docs/working/performance_tuning.md`](../docs/working/performance_tuning.md).

Run the current harnesses with:

```sh
cargo test -p gomoku-core --test bench_scenarios
cargo bench -p gomoku-core --bench board_perf -- --noplot
cargo bench -p gomoku-bot --bench search_perf -- --noplot
```

## Adding a new bot

1. Add a module under `gomoku-bot/src/`
2. `impl Bot for YourBot`
3. Register it in the relevant bot or lab-alias registry
4. The CLI can play it immediately; `gomoku-eval` can rate it; `gomoku-wasm`
   can ship it to the browser once surfaced through `WasmBot`
