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
| `gomoku-cli` | Native match runner with replay export |
| `gomoku-wasm` | `wasm-pack` bridge exposing `WasmBoard`, `WasmBot`, and replay analysis to JS |

Dependency shape: `core` has zero deps; `bot` / `analysis` / `eval` / `cli` /
`wasm` all depend on `core`; `analysis`, `cli`, `eval`, and `wasm` depend on
`bot`; `eval` and `wasm` consume `analysis`.

## Build and test

```sh
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

- [`Search Bot`](../docs/reference/lab/search_bot.md): config axes, current
  specs, rolling frontier, tactical ordering, and retired suffixes.
- [`Corridor Search`](../docs/reference/lab/corridor_search.md): setup
  corridor, exits, proof boundaries, and search reuse.
- [`Game Analysis`](../docs/reference/lab/game_analysis.md): replay analyzer
  contract, failure modes, and product interpretation.
- [`Performance Tuning`](../docs/working/performance_tuning.md): current
  benchmark snapshots and rejected optimization paths.

### Eval harness

`gomoku-eval` runs head-to-head series, self-play, and multi-threaded bot
evaluation schedules. Full round-robin remains the release-quality coverage
mode, but focused tuning should usually start with `head-to-head` or `gauntlet`
so new knobs do not explode into every possible pairing. Tournament games run in
parallel, then results are folded back in deterministic match order so replay
names and sequential Elo updates are reported consistently. CPU-time-budgeted
searches can still vary under scheduler load, so use repeated runs or
fixed-depth eval for ranking confidence. Eval defaults to Renju because bot
rankings are easier to compare when first-player advantage is constrained; pass
`--rule freestyle` for freestyle-specific product checks.

Strict per-move budgets remain the default because they make anchor reports easy
to compare. For product-shaped checks, `--search-budget-mode pooled` uses the
same CPU budget as a per-game average: cheap moves bank CPU reserve, hard moves
can spend it, and `--search-cpu-reserve-ms` caps how much burst time a side can
carry. `--search-cpu-max-move-ms` can cap a single move independently from the
larger reserve pool.

```sh
mkdir -p outputs
cargo run --release -p gomoku-eval -- tournament --bots search-d1,search-d3,search-d5 --games-per-pair 10 --opening-policy centered-suite --opening-plies 4 --search-cpu-time-ms 100 --max-game-ms 10000 --seed 42 --report-json outputs/gomoku-tournament.json
cargo run --release -p gomoku-eval -- tournament --schedule head-to-head --bots search-d5+tactical-cap-8,search-d5+tactical-cap-8+pattern-eval --games-per-pair 64 --opening-policy centered-suite --opening-plies 4 --search-cpu-time-ms 1000 --report-json outputs/head-to-head.json
cargo run --release -p gomoku-eval -- tournament --schedule gauntlet --candidates search-d5+tactical-cap-4+pattern-eval,search-d7+tactical-cap-4+pattern-eval,search-d7+tactical-cap-16+pattern-eval --anchors search-d3,search-d5+tactical-cap-16+pattern-eval,search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4 --anchor-report reports/latest.json --games-per-pair 32 --opening-policy centered-suite --opening-plies 4 --search-cpu-time-ms 1000 --max-moves 120 --report-json outputs/sweep-a-gauntlet.json
cargo run --release -p gomoku-eval -- report-html --input outputs/gomoku-tournament.json --output outputs/gomoku-tournament.html --json-href gomoku-tournament.json
cargo run --release -p gomoku-eval -- analyze-replay-batch --replay-dir outputs/replays --report-json outputs/analysis-batch.json --report-html outputs/analysis-batch.html
cargo run --release -p gomoku-eval -- analyze-report-replays --report reports/latest.json --sample-size 8 --max-scan-plies 8 --report-json outputs/analysis/top2-smoke.json --report-html outputs/analysis/top2-smoke.html
cargo run --release -p gomoku-eval -- analyze-report-replays --report reports/latest.json --sample-size 64 --include-proof-details --report-json outputs/analysis/top2-audit.json --report-html outputs/analysis/top2-audit.html
```

Use the larger curated-report run from `gomoku-bot-lab/` when publishing
bot-lab results:

```sh
mkdir -p reports
cargo run --release -p gomoku-eval -- tournament \
  --bots search-d1,search-d3,search-d3+pattern-eval,search-d5+tactical-cap-16+pattern-eval,search-d7+tactical-cap-8+pattern-eval,search-d3+pattern-eval+corridor-proof-c16-d8-w4,search-d5+tactical-cap-16+pattern-eval+corridor-proof-c16-d8-w4,search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4 \
  --games-per-pair 64 \
  --opening-policy centered-suite \
  --opening-plies 4 \
  --search-cpu-time-ms 2000 \
  --search-budget-mode pooled \
  --search-cpu-reserve-ms 8000 \
  --search-cpu-max-move-ms 4000 \
  --max-moves 120 \
  --seed 63 \
  --threads 22 \
  --report-json reports/latest.json
cargo run --release -p gomoku-eval -- report-html --input reports/latest.json --output reports/index.html --json-href latest.json
```

Useful eval flags:

| Flag | Description |
|------|-------------|
| `--schedule` | Tournament pairing workflow: `round-robin` by default, `head-to-head`, or `gauntlet` |
| `--bots` | Bot list for `round-robin`; exactly two bots for `head-to-head` |
| `--candidate` | Single candidate bot for `gauntlet` |
| `--candidates` | Comma-separated candidate bots for batch `gauntlet`; plays candidates against anchors only |
| `--anchors` | Comma-separated anchor bots for `gauntlet` |
| `--anchor-report` | Optional full round-robin report used as the gauntlet rating reference, usually `reports/latest.json`; validates rule/opening/budget/cap compatibility |
| `--rule` | Rule variant: `renju` by default, or `freestyle` |
| `--search-time-ms` | Applies a per-move budget to search bots, including lab aliases |
| `--search-cpu-time-ms` | Applies a Linux thread CPU-time budget to search bots |
| `--search-budget-mode` | Budget policy: `strict` per move by default, or `pooled` CPU reserve mode |
| `--search-cpu-reserve-ms` | CPU reserve cap for pooled mode; defaults to `4000` |
| `--search-cpu-max-move-ms` | Optional max CPU-time budget for one pooled move; useful when the reserve pool should be larger than the allowed spike |
| `--max-game-ms` | Records a still-running game as a draw after this wall-clock cap |
| `--max-moves` | Records a still-running game as a draw after this move count |
| `--seed` | Base seed for reproducible random bots and tournament opening-suite rotation |
| `--opening-policy` | Tournament opening policy; defaults to `centered-suite`; `random-legal` keeps the older whole-board random opening mode |
| `--opening-plies` | Tournament-only opening moves before bots take over; defaults to `4` |
| `--threads` | Tournament worker count; defaults to available CPU parallelism minus 2 |
| `--games-per-pair` | Tournament games per bot pair; use an even number for color balance |
| `--replay-dir` | Writes replay JSON for each eval game |
| `--report-json` | Writes a compact tournament report with summary stats and `cell_index_v1` move lists |
| `report-html --json-href` | Adds the raw JSON link shown in the rendered HTML |
| `analyze-replay-batch --replay-dir` | Analyzes saved replay JSON files and writes grouped analysis JSON/HTML reports |
| `analyze-report-replays --report` | Samples compact tournament report matches, reconstructs replay objects in memory, and writes grouped analysis JSON/HTML reports |

The default centered opening suite gives every bot pair the same local 4-ply
openings, with both color assignments, so rankings are less dominated by random
whole-board stones. Wall-clock budgets are practical but noisy under
multi-threaded load. CPU-time budgets are better for Linux ranking eval, pooled
CPU budgets are closer to hard-bot product use, and fixed-depth configs remain
the most reproducible option. Tournament reports
include pairwise records, color splits, shuffled-order Elo averages,
depth/budget stats, opening IDs, generated candidate width, post-ordering child
width, and compact `move_cells` using the same `row * 15 + col` codec as saved
web matches. The tournament harness and opening suite are documented in
[`../docs/reference/ops/tournament.md`](../docs/reference/ops/tournament.md).

For replay analysis iteration, prefer `analyze-report-replays --sample-size 8`
against an existing tournament report before running a full matchup. The
stratified sample is deterministic and tries to include both entrants, both
colors where available, a draw or max-move game, and short/long games. Replay
analysis now uses corridor-exit semantics: it follows the actual ending
corridor and asks whether model-valid defender replies can leave it, rather than
trying to prove every alternate state as a game-theoretic loss. The batch report
includes `unclear_reason`, final forced-interval presence, prefix counts,
per-entry elapsed time, limit-cause counts, and `unclear_context` drilldown.
Replay analysis defaults to a `64`-ply backward scan cap; short resolved
corridors stop early, and smoke runs can still override with
`--max-scan-plies 8`.
The strategic model is documented in
[`../docs/reference/lab/corridor_search.md`](../docs/reference/lab/corridor_search.md); the replay-specific
contract lives in [`../docs/reference/lab/game_analysis.md`](../docs/reference/lab/game_analysis.md).
Report rows intentionally avoid a category/severity label. The header keeps
only compact total, unclear, and error counts; expanded rows focus on proof
diagnostics such as reply probes, searched nodes, search time, unclear context,
and visual decision frames. The analyzer no longer assigns severity from
forced-corridor length. Add
`--include-proof-details` when auditing decisive replay labels; it records the
previous-prefix and final-forced-start proof snapshots plus visual HTML
decision frames for pre-move states from the winning ply backward through the
final forced interval, without changing the default compact report shape. These
frames separate reply role from reply outcome: outer hints show immediate or
imminent defensive candidates, offensive counter-threat candidates, and actual
replay moves, while marker characters show whether that reply is a confirmed
escape, possible escape, immediate loss, unknown, or forced loss. Keep
proof-detail audits at the base corridor depth until corridor search has better
pruning, memoization, or a narrower transition model.

Scratch reports should stay in ignored `outputs/`. Curated reports for the
public site live in [`reports/`](reports/); the web build copies that folder to
`/bot-report/`. To publish a selected run, write the JSON to
`reports/latest.json` and render the HTML to `reports/index.html`.

Curated replay-analysis reports for the public site live in
[`analysis-reports/`](analysis-reports/); the web build copies that folder to
`/analysis-report/`. The published analysis report should sample the head-to-head
games between the current top two standings in `reports/latest.json`; omit
explicit `--entrant-a` / `--entrant-b` so the CLI uses that default.

Curated reports should be generated as a follow-up artifact commit after the
bot/report code is already committed. Run `git status --short` first; a dirty
worktree is intentionally captured as a `_dirty` git revision and shown as a
development-run warning in the report. That warning is fine for scratch output
under `outputs/`, but avoid publishing it as the canonical `/bot-report/`.
The latest curated full round-robin report also acts as the anchor-rating source
for focused gauntlet runs; do not maintain a separate anchor cache unless the
published report workflow stops being enough.

### Tactical scenario diagnostics

Tournament reports answer "which config scores better over many games?"
Tactical scenarios answer a narrower question: "does this config choose the
expected one-move tactical response in this position, and what did it cost?"
The current corpus is documented in
[`../docs/reference/corpora/tactical_scenarios.md`](../docs/reference/corpora/tactical_scenarios.md), including
exact board prints, hard safety-gate cases, diagnostic cases, expected moves,
and the role/layer/intent/shape metadata used by reports.
The shared shape terms behind those cases live in
[`../docs/reference/lab/tactical_shapes.md`](../docs/reference/lab/tactical_shapes.md).

Run the baseline tactical sweep from `gomoku-bot-lab/`:

```sh
cargo run -p gomoku-eval -- tactical-scenarios --bots search-d1,search-d3,search-d5,search-d5+tactical-cap-8,search-d7+tactical-cap-8 --search-cpu-time-ms 1000
```

The command reports `PASS`/`FAIL` for hard safety gates and `HIT`/`MISS` for
diagnostic probes, followed by rule variant, side to move, case role, chosen
move, expected move sets, layer, intent, shape, depth reached, nodes, root
safety-gate work (`safety_nodes`), root/search candidate and legality costs,
time, and budget exhaustion. To capture reusable JSON:

```sh
mkdir -p outputs
cargo run -p gomoku-eval -- tactical-scenarios --bots search-d1,search-d3,search-d5,search-d5+tactical-cap-8,search-d7+tactical-cap-8 --search-cpu-time-ms 1000 --report-json outputs/tactical-scenarios.json
```

Treat this as diagnostic coverage, not a ranking system. Hard-gate failures are
regressions; diagnostic misses are active behavior gaps. New search logic should
be driven by diagnostics that expose real gaps, then confirmed with tournament
ablation. The rejected broad threat-extension and broad shape-eval experiments
are recorded in the v0.4 search plan rather than exposed as current lab specs.

### Lethal scenario diagnostics

Lethal scenarios answer a different question from tactical scenarios:
"does this position already leave the defender without legal coverage?" They
validate the shared lethal classifier directly rather than asking a bot to pick
a move. The model and current case list live in
[`../docs/reference/lab/lethal_threats.md`](../docs/reference/lab/lethal_threats.md).

Run the lethal safety harness from `gomoku-bot-lab/`:

```sh
cargo run -p gomoku-eval -- lethal-scenarios
```

Add `--show-boards` when reviewing the exact fixture positions in the terminal.
The JSON report always includes each case's `board_ascii` field.

To capture reusable JSON:

```sh
mkdir -p outputs
cargo run -p gomoku-eval -- lethal-scenarios --report-json outputs/lethal-scenarios.json
```

The current harness covers terminal coverage and one-step coverage: freestyle
open four, blockable single four, defender immediate-win race, Renju forbidden
block, Renju Black illegal-completion caveat, crossed `4+3`, crossed `3+3`, a
non-lethal crossed broken-three pair with a shared block, and a non-lethal
single open three. Replay analysis should consume this harness before lethal
classification is used in bot search.

## Replay format

Both `gomoku-cli` and `gomoku-eval` write the same JSON. The web game consumes
that replay format directly.

```json
{
  "hash_algo": { "algorithm": "xorshift64", "seed": 16045690984833335166 },
  "rules": { "board_size": 15, "win_length": 5, "variant": "freestyle" },
  "black": "search-d3",
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
          "candidate_source": "near_all_r2",
          "null_cell_culling": "disabled",
          "legality_gate": "exact_rules",
          "safety_gate": "current_obligation",
          "move_ordering": "tt_first_board_order",
          "child_limit": null,
          "search_algorithm": "alpha_beta_id",
          "static_eval": "line_shape_eval"
        },
        "depth": 3,
        "nodes": 42,
        "safety_nodes": 4,
        "total_nodes": 46,
        "metrics": {
          "root_candidate_generations": 1,
          "search_candidate_generations": 12,
          "null_cell_cull_checks": 0,
          "null_cells_culled": 0,
          "root_legality_checks": 4,
          "search_legality_checks": 80,
          "root_tactical_annotations": 4,
          "search_tactical_annotations": 0,
          "child_limit_applications": 0,
          "search_child_limit_applications": 0,
          "child_cap_hits": 0
        },
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
