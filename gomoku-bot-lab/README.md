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
cargo run --release -p gomoku-cli -- --black search-d3 --white search-d1 --rule renju
cargo run --release -p gomoku-cli -- --black baseline --white random --time-ms 500
cargo run --release -p gomoku-cli -- --black baseline --white random --quiet --replay /tmp/game.json
```

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--black` | `baseline` | Bot for Black: `random`, `baseline`, `baseline-N`, or search aliases |
| `--white` | `random`   | Bot for White: `random`, `baseline`, `baseline-N`, or search aliases |
| `--depth` | `5`        | Fixed baseline depth for the plain `baseline` spec |
| `--time-ms` | —        | Time budget per move for search bots, including lab aliases |
| `--rule` | `freestyle` | Rule variant: `freestyle` or `renju` |
| `--replay` | —         | Write replay JSON to this path |
| `--quiet` | —          | Suppress per-move board printing |

### Current `SearchBot`

Negamax with alpha-beta pruning, iterative deepening, and a transposition table
keyed by incremental Zobrist hashing. Move candidates are pruned to empty cells
within two rows/columns of any existing stone (`near_all_r2`). Static evaluation
scores open and half-open runs of 2–4 in all four directions. It reliably beats
`RandomBot` and is intentionally good enough for practice without trying to be a
perfect Gomoku engine.

The lab primarily uses explicit search specs. `gomoku-bot` itself exposes
`SearchBotConfig`; the spec strings are lab conveniences over that config, not
canonical product presets.

| Spec | Config | Intent |
|---|---|---|
| `search-d1` | depth 1, `near_all_r2`, `current_obligation` | easy/beginner lane |
| `search-d3` | depth 3, `near_all_r2`, `current_obligation` | current default baseline |
| `search-d5` | depth 5, `near_all_r2`, `current_obligation` | uncapped depth reference |
| `search-d5+tactical-cap-8` | depth 5, tactical ordering, non-root child cap 8 | efficient hard-side candidate |
| `search-d7+tactical-cap-8` | depth 7, tactical ordering, non-root child cap 8 | stronger but slower hard-side candidate |

Append `+near-all-r1`, `+near-all-r2`, or `+near-all-r3` to change the symmetric
candidate source radius. Append `+near-self-rN-opponent-rM` to test asymmetric
current-player versus opponent-stone frontiers, for example
`+near-self-r2-opponent-r1`. Append `+null-cull` to remove generated cells that
cannot make five for either color in any direction; it is an experimental
candidate-filter axis, disabled by default. Append `+no-safety` to disable the
root safety gate. For example, `search-d3+near-all-r1+no-safety` keeps the
depth-3 search but uses a radius-1 candidate source and no root safety gate. The
default `current_obligation` safety gate only filters already-generated legal
root candidates against immediate wins, direct immediate blocks, and direct or
counter-four replies to imminent threats. Append `+tactical-full` to try the
older full local-threat move ordering before alpha-beta search.
Append `+child-cap-N` to cap the ordered non-root child frontier after candidate
generation, legality filtering, and move ordering. `+tactical-cap-N` is
shorthand for the current tactical ordering plus `+child-cap-N`: immediate
win/block checks stay global, then full tactical annotation runs only for moves
that can plausibly create local tactical shapes. `+tactical-full-cap-N` keeps
the older full-annotation path for direct comparisons. Root still considers
every legal/safe candidate; candidate source controls discovery, while child
cap controls how many ordered non-root children alpha-beta searches. Rolling
frontier is the default threat-view backend;
append `+scan-threat-view` to force the older scan-backed view for fallback
comparisons, or `+rolling-frontier-shadow` to run scan-vs-rolling parity checks.

Legacy specs still work: plain `baseline` uses `--depth`, `baseline-N` creates a
custom fixed-depth baseline bot, and the old `fast`/`balanced`/`deep` aliases
still parse for old scripts. New reports and gauntlets should use explicit
`search-*` specs.

### Current corridor integration

The earlier standalone `CorridorBot` bridge is retired. The first `SearchBot`
integration, the lab-only `+corridor-q` leaf-quiescence suffix, is also retired.
It proved the shared corridor engine can be called from search, but it spent
too much work probing depth-0 positions that usually fell back to static eval.

Failed search experiments are intentionally removed instead of kept as dead lab
suffixes. The broad shape-eval attempt fixed one depth-2 diagnostic but lost to
plain `search-d3`, so it is documented rather than exposed as a live lab spec.
The corridor leaf-quiescence result follows the same rule: keep the learning,
not the suffix. Current notes live in
[`../docs/archive/v0_4_search_bot_enhancement_plan.md`](../docs/archive/v0_4_search_bot_enhancement_plan.md).

More detailed strategy notes live in [`../docs/search_bot.md`](../docs/search_bot.md).
The `0.4.1` tactical-ladder work established the current bot baseline: local
threat competence first, casual combo play next, then bounded corridor ideas
only when they can be measured. Tactical facts are meant to buy effective depth
through safer narrowing, ordering, and selective extension, not to replace
alpha-beta search with broad shape scoring. At that checkpoint, the tactical
annotation stage recorded its own trace metrics and could be paired with the
lab-only child frontier cap to test whether better ordering bought effective
depth; the later `0.4.4` rolling-frontier pass promoted that caching model after
the metrics justified it.

The `0.4.1` reference report compared the depth ladder, tactical-cap variants,
and pattern-eval variants. Its product read was conservative: D1 was an easy
lane, D3 remained the default baseline, D5 tactical-cap was the efficient
hard-side candidate, D7 tactical-cap was stronger but slower, and pattern eval
was still lab-only because the score gain came with real compute cost.

The `0.4.2` lab pass kept that restraint rather than jumping straight to UI
settings. It swept existing knobs with head-to-heads and gauntlets, then pivoted
toward corridor search as the more useful foundation: explain why bots win or
lose, identify final forced sequences, and use that evidence before promoting
more product settings.

The `0.4.2` sweeps still matter: pattern eval remains the strongest lab signal,
cap16 is not a clear upgrade, cap4 is a viable narrowing point when paired with
tactical ordering, and asymmetric `self2/opponent1` candidate discovery is most
interesting as an efficiency tweak for `D3 + pattern-eval`. Treat these as lab
candidates, not product presets yet. Corridor-search strategy is documented in
[`../docs/corridor_search.md`](../docs/corridor_search.md).

The `0.4.3` lab slice tested corridor search inside bot behavior before
exposing more web settings. The result is useful evidence but not a candidate
bot direction. `+corridor-own-dN-wM`, `+corridor-opponent-dN-wM`,
remain as disabled lab suffixes for base comparison, but focused checks stayed
slower or weaker than the base anchors. The rank/top-N, proof-only, and static
exit controls were removed from the current parser and remain only as historical
report evidence. Keep corridor portals out of anchors, sweeps, difficulty
ladders, and UI work.

The durable output was the scan-backed `ThreatView` seam, unified tactical
facts, and honest portal metrics. Those pieces set up the `0.4.4`
rolling-frontier pass to replace hot-path scans behind a stable query contract
without promoting the failed portal shape.
The working plan lives in
[`../docs/archive/v0_4_3_corridor_bot_plan.md`](../docs/archive/v0_4_3_corridor_bot_plan.md).

`0.4.4` promotes the rolling-frontier pass after focused scan-vs-rolling and
shadow parity checks. Scan remains available as an explicit fallback behind the
same `ThreatView` seam, while shadow remains validation-only. The working plan lives in
[`../docs/archive/v0_4_4_frontier_plan.md`](../docs/archive/v0_4_4_frontier_plan.md).

Current frontier suffixes are intentionally narrow:

- `+rolling-frontier-shadow`: compare rolling-backed tactical annotations,
  current-obligation safety, and any remaining corridor diagnostic queries
  against scan-backed answers, report shadow mismatch counts, and record
  scan-vs-frontier update/query timing; behavior stays scan-backed.
- `+rolling-frontier`: explicitly use the default rolling-backed tactical
  annotations, indexed immediate-win checks, root win/block checks, and any
  remaining corridor diagnostic queries. Current-obligation safety also uses a
  root-only full frontier in this mode.
- `+scan-threat-view`: force the scan-backed threat view for fallback and
  comparison runs.

Search now threads an optional frontier through recursive apply/undo with the
board and hash. The default rolling mode enables it for normal search, while
scan disables it and shadow runs both paths for parity. Immediate wins now have a
dedicated per-player rolling index, so `TacticalOnly` mode can answer win/block
queries without maintaining full corridor move facts. Current smoke data reached
zero shadow mismatches, and focused promotion smokes make rolling faster than
scan while preserving relaxed-budget parity.
Non-shadow rolling search should keep threat-view scan counters at zero; scan
queries are expected only in scan mode, rolling-shadow comparison, or explicit
fallback diagnostics.

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
carry.

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
  --search-cpu-time-ms 1000 \
  --search-budget-mode pooled \
  --search-cpu-reserve-ms 4000 \
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
[`../docs/tournament.md`](../docs/tournament.md).

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
[`../docs/corridor_search.md`](../docs/corridor_search.md); the replay-specific
contract lives in [`../docs/game_analysis.md`](../docs/game_analysis.md).
Report rows lead with loss-category severity: `mistake` for forced-corridor
spans shorter than `5` plies, `tactical_error` for spans from `5` to `8` plies,
and `strategic_loss` for spans `9` plies or longer. The root detail remains as
row detail so an `unclear` result can still distinguish corridor-depth cutoffs,
defender-reply unknowns, model-scope unknowns, scan-cap cutoffs, games with
no final forced interval, and the board prefixes that need inspection. Add
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
[`../docs/tactical_scenarios.md`](../docs/tactical_scenarios.md), including
exact board prints, hard safety-gate cases, diagnostic cases, expected moves,
and the role/layer/intent/shape metadata used by reports.
The shared shape terms behind those cases live in
[`../docs/tactical_shapes.md`](../docs/tactical_shapes.md).

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
