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
cargo run --release -p gomoku-cli -- --black balanced --white fast
cargo run --release -p gomoku-cli -- --black baseline --white random --time-ms 500
cargo run --release -p gomoku-cli -- --black baseline --white random --quiet --replay /tmp/game.json
```

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--black` | `baseline` | Bot for Black: `random`, `baseline`, `baseline-N`, or a lab alias (`fast`, `balanced`, `deep`) |
| `--white` | `random`   | Bot for White: `random`, `baseline`, `baseline-N`, or a lab alias (`fast`, `balanced`, `deep`) |
| `--depth` | `5`        | Fixed baseline depth for the plain `baseline` spec |
| `--time-ms` | —        | Time budget per move for search bots, including lab aliases |
| `--rule` | `freestyle` | Rule variant: `freestyle` or `renju` |
| `--replay` | —         | Write replay JSON to this path |
| `--quiet` | —          | Suppress per-move board printing |

### Current `SearchBot`

Negamax with alpha-beta pruning, iterative deepening, and a transposition table
keyed by incremental Zobrist hashing. Move candidates are pruned to cells within
2 steps of any existing stone. Static evaluation scores open and half-open runs
of 2–4 in all four directions. It reliably beats `RandomBot` and is intentionally
good enough for practice without trying to be a perfect Gomoku engine.

The lab now has named baseline-search aliases. `gomoku-bot` itself exposes
explicit `SearchBotConfig` fields; these names are lab conveniences over that
config, not canonical product presets.

| Alias | Config | Intent |
|---|---|---|
| `fast` | depth 2, `near_all_r2`, `opponent_reply_search_probe` | cheap comparison target |
| `balanced` | depth 3, `near_all_r2`, `opponent_reply_search_probe` | current browser practice-bot depth |
| `deep` | depth 5, `near_all_r2`, `opponent_reply_search_probe` | current CLI default depth |

Append `+near-all-r1`, `+near-all-r2`, or `+near-all-r3` to change the candidate
source radius. Append `+no-safety` to disable the root safety gate. For example,
`search-d3+near-all-r1+no-safety` keeps the depth-3 search but uses a radius-1
candidate source and no root safety gate. These are diagnostic switches, not
product presets.

Legacy specs still work: plain `baseline` uses `--depth`, `baseline-N` creates a
custom fixed-depth baseline bot, and `--time-ms` can cap search bots during CLI
games.

Failed search experiments are intentionally removed instead of kept as dead lab
suffixes. The broad shape-eval attempt fixed one depth-2 diagnostic but lost to
plain `search-d3`, so it is documented rather than exposed as a live lab spec.
Current notes live in
[`../docs/archive/v0_4_search_bot_enhancement_plan.md`](../docs/archive/v0_4_search_bot_enhancement_plan.md).

More detailed strategy notes live in [`../docs/bot_baseline.md`](../docs/bot_baseline.md).

### Eval harness

`gomoku-eval` runs head-to-head series, self-play, and multi-threaded
round-robin tournaments. Tournament games run in parallel, then results are
folded back in deterministic match order so replay names and sequential Elo
updates are reported consistently. CPU-time-budgeted searches can still vary
under scheduler load, so use repeated runs or fixed-depth eval for ranking
confidence. Eval defaults to Renju because bot rankings are easier to compare
when first-player advantage is constrained; pass `--rule freestyle` for
freestyle-specific product checks.

```sh
mkdir -p outputs
cargo run --release -p gomoku-eval -- tournament --bots fast,balanced,deep --games-per-pair 10 --opening-plies 4 --search-cpu-time-ms 100 --max-game-ms 10000 --seed 42 --report-json outputs/gomoku-tournament.json
cargo run --release -p gomoku-eval -- report-html --input outputs/gomoku-tournament.json --output outputs/gomoku-tournament.html --json-href gomoku-tournament.json
```

Use the larger curated-report run from `gomoku-bot-lab/` when publishing
bot-lab results:

```sh
mkdir -p reports
cargo run --release -p gomoku-eval -- tournament --bots fast,balanced,deep --games-per-pair 64 --opening-plies 4 --search-cpu-time-ms 1000 --max-moves 120 --seed 42 --report-json reports/latest.json
cargo run --release -p gomoku-eval -- report-html --input reports/latest.json --output reports/index.html --json-href latest.json
```

Useful eval flags:

| Flag | Description |
|------|-------------|
| `--rule` | Rule variant: `renju` by default, or `freestyle` |
| `--search-time-ms` | Applies a per-move budget to search bots, including lab aliases |
| `--search-cpu-time-ms` | Applies a Linux thread CPU-time budget to search bots |
| `--max-game-ms` | Records a still-running game as a draw after this wall-clock cap |
| `--max-moves` | Records a still-running game as a draw after this move count |
| `--seed` | Base seed for reproducible random bots and tournament openings |
| `--opening-plies` | Tournament-only seeded random opening moves before bots take over; defaults to `4` |
| `--threads` | Tournament worker count; defaults to available CPU parallelism minus 2 |
| `--games-per-pair` | Tournament games per bot pair; use an even number for color balance |
| `--replay-dir` | Writes replay JSON for each eval game |
| `--report-json` | Writes a compact tournament report with summary stats and `cell_index_v1` move lists |
| `report-html --json-href` | Adds the raw JSON link shown in the rendered HTML |

Seeded openings make deterministic bots see varied positions. Wall-clock budgets
are practical but noisy under multi-threaded load. CPU-time budgets are better
for Linux ranking eval, while fixed-depth configs remain the most reproducible
option. Tournament reports include pairwise records, color splits, shuffled-order
Elo averages, depth/budget stats, and compact `move_cells` using the same
`row * 15 + col` codec as saved web matches.

Scratch reports should stay in ignored `outputs/`. Curated reports for the
public site live in [`reports/`](reports/); the web build copies that folder to
`/bot-report/`. To publish a selected run, write the JSON to
`reports/latest.json` and render the HTML to `reports/index.html`.

Curated reports should be generated as a follow-up artifact commit after the
bot/report code is already committed. Run `git status --short` first; a dirty
worktree is intentionally captured as a `_dirty` git revision and shown as a
development-run warning in the report. That warning is fine for scratch output
under `outputs/`, but avoid publishing it as the canonical `/bot-report/`.

### Tactical scenario diagnostics

Tournament reports answer "which config scores better over many games?"
Tactical scenarios answer a narrower question: "does this config choose the
expected one-move tactical response in this position, and what did it cost?"

Run the baseline tactical sweep from `gomoku-bot-lab/`:

```sh
cargo run -p gomoku-eval -- tactical-scenarios --bots search-d2,search-d3,search-d5 --search-cpu-time-ms 1000
```

The command reports pass/fail, chosen move, expected move set, depth reached,
nodes, root safety-gate probe nodes (`prefilter_nodes` in the current JSON
schema), time, and budget exhaustion. To capture reusable JSON:

```sh
mkdir -p outputs
cargo run -p gomoku-eval -- tactical-scenarios --bots search-d2,search-d3,search-d5 --search-cpu-time-ms 1000 --report-json outputs/tactical-scenarios.json
```

Treat this as diagnostic coverage, not a ranking system. If a baseline config
already passes a scenario, that fixture becomes a regression guard. New search
logic should be driven by scenarios that expose real gaps, then confirmed with
tournament ablation. The rejected broad threat-extension and broad shape-eval
experiments are recorded in the v0.4 search plan rather than exposed as current
lab specs.

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
    {
      "mv": "D4",
      "time_ms": 5,
      "hash": 987654321,
      "trace": {
        "config": {
          "max_depth": 3,
          "time_budget_ms": null,
          "cpu_time_budget_ms": null,
          "candidate_radius": 2,
          "root_prefilter": true
        },
        "depth": 3,
        "nodes": 42,
        "prefilter_nodes": 4,
        "total_nodes": 46,
        "budget_exhausted": false,
        "score": 100
      }
    }
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

## Performance tuning

Benchmark process, fixed scenario corpus, and baseline snapshots live in
[`../docs/performance_tuning.md`](../docs/performance_tuning.md).

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
