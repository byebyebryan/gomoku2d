# Performance Tuning

Purpose: ongoing performance and optimization notes for `gomoku-bot-lab`.
This is the working document for:

- benchmark process
- fixed benchmark corpus
- current hotspot findings
- optimization backlog
- baseline snapshots before and after tuning passes

This is intentionally an evolving engineering note, not a product or design
doc.

## Goals

- measure performance changes before and after tuning
- use fixed, reviewable board positions instead of noisy ad hoc timing
- keep correctness checks alongside speed work
- make it easy to add new benchmark scenarios when a bug repro or hotspot
  justifies it

## Benchmarking rules

1. Always benchmark in release mode.
2. Use the checked-in fixed scenario corpus for comparisons.
3. Do not use pure random boards as the benchmark source.
4. Use self-play only to discover candidate positions, then manually promote
   them into the fixed corpus.
5. Compare medians and reported ranges, not single fastest runs.
6. Keep correctness verification with every tuning pass.

## Fixed scenario corpus

Source of truth: `gomoku-bot-lab/benchmarks/scenarios.rs`

Scenarios are stored as:

- stable `id`
- rule `variant`
- expected side to move
- human-readable move list (notation form)
- representative legal `probe_move`
- tags and purpose notes

Why this format:

- easy to review in PRs
- reproducible from scratch
- impossible-state random boards are avoided
- easy to extend from bug repros and self-play mining

### Initial curated set

| id | variant | to move | tags | purpose |
|---|---|---|---|---|
| `opening_sparse` | freestyle | Black | opening, sparse | early local opening around center |
| `early_local_fight` | freestyle | Black | opening, local-fight | compact early tactical cluster |
| `immediate_win` | freestyle | Black | tactical, immediate-win | direct win available now |
| `immediate_block` | freestyle | Black | tactical, immediate-block | forced defensive block |
| `attack_wins_race` | freestyle | Black | tactical, attack-vs-defense | take the direct win instead of blocking |
| `anti_blunder_open_three` | freestyle | White | tactical, anti-blunder | repro for the recent search safety fix |
| `create_open_four` | freestyle | Black | tactical, open-four | create a forcing open-four threat |
| `create_broken_three` | freestyle | Black | tactical, broken-three | choose a non-terminal shape-building move |
| `create_double_threat` | freestyle | Black | tactical, double-threat | create simultaneous immediate winning threats |
| `renju_forbidden_cross` | renju | Black | renju, forbidden | black to move with a forbidden tactical point |
| `midgame_medium` | freestyle | Black | midgame, medium-density | representative clustered midgame |
| `midgame_dense` | freestyle | Black | midgame, dense | denser midgame with larger frontier/eval cost |

### Search behavior cases

The corpus also defines `SEARCH_BEHAVIOR_CASES`, which pair scenarios with a
named lab config and expected moves. These are not performance measurements; they
are behavior anchors for the `v0.4` bot-discovery pass.

Current cases exercise the `balanced` lab config:

- immediate win
- immediate block
- open-three anti-blunder block
- attack-vs-defense race where winning now is better than blocking

The tactical scenario runner can also compare ad-hoc search configs while a
slice is under development. Treat those as diagnostic probes: if a config only
reduces counted nodes on already-passing forced-line scenarios, it is useful
evidence for the mechanism but not enough to become a product-facing preset.
Discarded experiments should be documented in the active v0.4 plan and removed
from the live lab spec surface. The broad `shape-eval` attempt fixed the
depth-2 broken-three diagnostic, but was discarded because it lost to simply
using `search-d3` and reduced effective depth under CPU budgets.

The next performance pass should start with measurement rather than another
tactical consumer: identify how much time `search-d3` spends in eval,
candidate generation, legality checks, safety-gate probes, and hidden tactical
work. Only keep a search change if it improves reached depth, average move time,
or tournament score under the same CPU budget.

## Benchmark suites

### Core

File: `gomoku-bot-lab/gomoku-core/benches/board_perf.rs`

Current measurements:

- `Board::clone()` on a fixed opening snapshot
- full `Board::cell()` scan on a fixed opening snapshot
- `Board::hash()` on a fixed opening snapshot
- `Board::to_fen()` on a fixed opening snapshot
- `immediate_winning_moves_for(current_player)`
- `has_multiple_immediate_winning_moves_for(current_player)`
- `apply_move()` followed by `undo_move()` on a representative legal move
- `forbidden_moves_for_current_player()` on Renju anchor positions
- candidate-set `is_legal()` filtering on Renju anchor positions

These cover the current quick-win candidates:

- `nearby_empty_moves()`
- `immediate_winning_moves_for()`
- core legality/apply/win path

### Search bot

File: `gomoku-bot-lab/gomoku-bot/benches/search_perf.rs`

Current measurement:

- `SearchBot::choose_move()` across the named baseline-search lab configs:
  `fast`, `balanced`, and `deep`

The `balanced` lab config uses depth `3` because it matches the current
browser-side practice bot configuration in `gomoku-web`. The `deep` lab config
matches the native CLI's historical depth-`5` default, and `fast` gives a cheap
comparison target.

## Commands

Scenario validity:

```sh
cargo test -p gomoku-core --test bench_scenarios
```

Core benchmark suite:

```sh
cargo bench -p gomoku-core --bench board_perf -- --noplot
```

Search benchmark suite:

```sh
cargo bench -p gomoku-bot --bench search_perf -- --noplot
```

Lab config and quick tournament smoke:

```sh
cargo run --release -p gomoku-cli -- --black balanced --white fast --quiet
cargo run --release -p gomoku-eval -- versus --bot-a fast --bot-b balanced --games 1
mkdir -p outputs
cargo run --release -p gomoku-eval -- tournament --bots search-d2,search-d3,search-d5 --games-per-pair 10 --opening-plies 4 --search-cpu-time-ms 100 --max-game-ms 10000 --seed 42 --report-json outputs/gomoku-tournament.json
cargo run --release -p gomoku-eval -- report-html --input outputs/gomoku-tournament.json --output outputs/gomoku-tournament.html --json-href gomoku-tournament.json
```

Curated ranking report, from `gomoku-bot-lab/`:

```sh
mkdir -p reports
cargo run --release -p gomoku-eval -- tournament --bots search-d2,search-d3,search-d5 --games-per-pair 64 --opening-plies 4 --search-cpu-time-ms 1000 --max-moves 120 --seed 48 --threads 22 --report-json reports/latest.json
cargo run --release -p gomoku-eval -- report-html --input reports/latest.json --output reports/index.html --json-href latest.json
```

`gomoku-eval` defaults to Renju so ranking tournaments are less dominated by
first-player advantage; pass `--rule freestyle` when validating freestyle product
behavior. Use an even `--games-per-pair` so each pair gets balanced color
coverage. Tournament games run multi-threaded by default and use seeded random
opening plies so deterministic bots do not replay one empty-board line forever.
For Linux ranking eval, prefer `--search-cpu-time-ms` over wall-clock
`--search-time-ms`; fixed-depth configs are still the cleanest reproducibility
baseline. The reusable JSON report is the source of truth for ranking analysis;
the HTML report is a derived view that can be regenerated without rerunning the
tournament. Keep scratch output under `gomoku-bot-lab/outputs/`; curated
reports under `gomoku-bot-lab/reports/` are copied into the public web build as
`/bot-report/`.

For release-quality reports, commit the bot/report implementation first, then
generate `reports/latest.json` and `reports/index.html` from a clean worktree
and commit those artifacts separately. The report records the git revision; if
the tree is dirty at tournament time, the HTML intentionally displays a
`_dirty` suffix and a development-run warning.

## Initial hotspot findings

From code inspection before the first benchmark pass:

1. `gomoku-bot/src/search.rs:evaluate()`
   - full-board scan at every leaf
   - likely the biggest long-term search cost

2. `gomoku-bot/src/search.rs:candidate_moves()`
   - rescans the full board at each node

3. Root safety gate
   - adds extra board work before the main search

4. `gomoku-core/src/board.rs:nearby_empty_moves()`
   - currently uses `BTreeSet`

5. `gomoku-core/src/board.rs:immediate_winning_moves_for()`
   - currently clones a full board once per candidate move

## Optimization backlog

### Completed

1. Rewrite `nearby_empty_moves()` to use a dense seen bitmap instead of
   `BTreeSet` (`2026-04-23`)
2. Rewrite `immediate_winning_moves_for()` to clone once and use
   `apply_move()` / `undo_move()` per candidate (`2026-04-23`)
3. Skip redundant `is_legal()` checks in search nodes where Renju-black
   forbidden logic is not relevant (`2026-04-23`)
4. Add `has_multiple_immediate_winning_moves_for()` so the root safety gate can
   stop after two immediate wins (`2026-04-23`)
5. Let `apply_move()` be the immediate-win legality gate instead of calling
   `is_legal_for()` first and repeating Renju checks (`2026-04-23`)
6. Add benchmark-corpus search tests for legal output plus immediate
   win/block anchors (`2026-04-23`)
7. Tighten the Renju forbidden precheck from "near any black stone" to "two
   black stones on one local axis" before the exact forbidden detector
   (`2026-05-03`)
8. Replace immediate-win probe apply/undo with virtual directional run checks
   (`2026-05-03`)
9. Replace `Board`'s `Vec<Vec<Cell>>` storage with dual bitboards and route bot
   eval/candidate hot loops through occupied-stone iteration (`2026-05-03`)

### Future work

1. More incremental or localized evaluation
2. Incremental candidate frontier maintenance
3. Bitboard-aware helpers for any remaining full-cell-scan callers that become
   hot under profiling

## Baseline snapshot

Date: `2026-04-23`

Context:

- local workstation snapshot only; rerun before treating numbers as stable
- commands used:

```sh
cargo test -p gomoku-core --test bench_scenarios
cargo bench -p gomoku-core --bench board_perf -- --noplot
cargo bench -p gomoku-bot --bench search_perf -- --noplot
```

### Core anchors

| Benchmark | Time |
|---|---|
| `immediate_winning_moves/current_player/opening_sparse` | `21.37–21.55 µs` |
| `immediate_winning_moves/current_player/anti_blunder_open_three` | `28.57–28.68 µs` |
| `immediate_winning_moves/current_player/renju_forbidden_cross` | `78.08–78.77 µs` |
| `immediate_winning_moves/current_player/midgame_dense` | `44.35–44.61 µs` |
| `apply_move_then_undo/opening_sparse` | `294.30–300.25 ns` |
| `apply_move_then_undo/renju_forbidden_cross` | `546.99–581.20 ns` |
| `apply_move_then_undo/midgame_dense` | `369.95–400.62 ns` |
| `forbidden_moves/current_player/renju_forbidden_cross` | `28.47–28.84 µs` |

### Search anchors

All numbers below are `SearchBot::choose_move()` at depth `3`.

| Benchmark | Time |
|---|---|
| `opening_sparse` | `55.39–56.62 ms` |
| `early_local_fight` | `73.35–75.03 ms` |
| `immediate_win` | `13.91–14.01 ms` |
| `immediate_block` | `13.73–13.85 ms` |
| `anti_blunder_open_three` | `91.13–92.77 ms` |
| `renju_forbidden_cross` | `140.83–143.39 ms` |
| `midgame_medium` | `139.78–142.50 ms` |
| `midgame_dense` | `214.87–228.09 ms` |

### Notes

- The search baseline already shows the expected pattern:
  - tactical forced positions are cheap
  - denser midgames and Renju legality pressure are much more expensive
- `renju_forbidden_cross` is notably heavier than a similarly sized freestyle
  tactical position, which supports the current suspicion that legality and
  nearby-win scanning deserve the first quick-pass optimization work.

## Optimization pass 1 snapshot

Date: `2026-04-23`

Changes:

- `nearby_empty_moves()` now uses a dense seen bitmap and emits row-major moves.
- `immediate_winning_moves_for()` now clones the board once, then probes with
  `apply_move()` / `undo_move()`.
- search nodes now skip pre-`apply_move()` legality checks except for Renju
  black, where forbidden-move filtering is required.

Commands used:

```sh
cargo test --workspace
cargo bench -p gomoku-core --bench board_perf -- --noplot
cargo bench -p gomoku-bot --bench search_perf -- --noplot
```

### Core anchors after pass 1

| Benchmark | Time | Baseline |
|---|---|---|
| `immediate_winning_moves/current_player/opening_sparse` | `2.4769–2.5033 µs` | `21.37–21.55 µs` |
| `immediate_winning_moves/current_player/anti_blunder_open_three` | `3.1904–3.2312 µs` | `28.57–28.68 µs` |
| `immediate_winning_moves/current_player/renju_forbidden_cross` | `50.854–51.433 µs` | `78.08–78.77 µs` |
| `immediate_winning_moves/current_player/midgame_dense` | `4.4690–4.5294 µs` | `44.35–44.61 µs` |
| `apply_move_then_undo/opening_sparse` | `307.20–325.54 ns` | `294.30–300.25 ns` |
| `apply_move_then_undo/renju_forbidden_cross` | `524.85–575.04 ns` | `546.99–581.20 ns` |
| `apply_move_then_undo/midgame_dense` | `365.02–392.08 ns` | `369.95–400.62 ns` |
| `forbidden_moves/current_player/renju_forbidden_cross` | `24.032–24.272 µs` | `28.47–28.84 µs` |

### Search anchors after pass 1

All numbers below are `SearchBot::choose_move()` at depth `3`.

| Benchmark | Time | Baseline |
|---|---|---|
| `opening_sparse` | `13.717–13.854 ms` | `55.39–56.62 ms` |
| `early_local_fight` | `13.614–13.729 ms` | `73.35–75.03 ms` |
| `immediate_win` | `1.5889–1.5966 ms` | `13.91–14.01 ms` |
| `immediate_block` | `1.9394–1.9676 ms` | `13.73–13.85 ms` |
| `anti_blunder_open_three` | `14.215–14.304 ms` | `91.13–92.77 ms` |
| `renju_forbidden_cross` | `18.819–18.928 ms` | `140.83–143.39 ms` |
| `midgame_medium` | `23.464–23.832 ms` | `139.78–142.50 ms` |
| `midgame_dense` | `33.215–33.394 ms` | `214.87–228.09 ms` |

### Notes

- The biggest win came from removing per-candidate board clones in immediate
  win scanning. This also reduced the root safety-gate cost.
- Freestyle immediate-win scans improved by roughly an order of magnitude on
  the fixed anchors. Renju immediate-win scans improved less because forbidden
  checks remain the dominant cost there.
- `apply_move_then_undo` stayed effectively flat, which is expected because
  this pass did not change the move application path.

## Optimization pass 2 snapshot

Date: `2026-04-23`

Changes:

- `Board::has_multiple_immediate_winning_moves_for()` scans nearby candidates
  directly and returns as soon as it finds two wins.
- `SearchBot` uses that helper in the opponent-reply safety gate instead of
  collecting every immediate winning move and checking `len() >= 2`.
- `immediate_winning_moves_for()` now uses the same probe path and lets
  `apply_move()` reject illegal candidates, avoiding duplicate Renju forbidden
  checks.
- Bot tests now assert all fixed benchmark scenarios produce legal moves, and
  the immediate-win / immediate-block anchors keep their expected behavior.

Commands used:

```sh
cargo test --workspace
cargo bench -p gomoku-core --bench board_perf -- --noplot
cargo bench -p gomoku-bot --bench search_perf -- --noplot
```

### Core anchors after pass 2

| Benchmark | Time | Pass 1 |
|---|---|---|
| `immediate_winning_moves/current_player/opening_sparse` | `2.4295–2.4505 µs` | `2.4769–2.5033 µs` |
| `immediate_winning_moves/current_player/anti_blunder_open_three` | `3.1551–3.1626 µs` | `3.1904–3.2312 µs` |
| `immediate_winning_moves/current_player/renju_forbidden_cross` | `26.990–27.135 µs` | `50.854–51.433 µs` |
| `immediate_winning_moves/current_player/midgame_dense` | `4.4511–4.4783 µs` | `4.4690–4.5294 µs` |
| `has_multiple_immediate_winning_moves/current_player/opening_sparse` | `2.2625–2.2687 µs` | new benchmark |
| `has_multiple_immediate_winning_moves/current_player/immediate_win` | `1.5401–1.5536 µs` | new benchmark |
| `has_multiple_immediate_winning_moves/current_player/anti_blunder_open_three` | `2.8538–2.8745 µs` | new benchmark |
| `has_multiple_immediate_winning_moves/current_player/renju_forbidden_cross` | `26.573–26.837 µs` | new benchmark |
| `has_multiple_immediate_winning_moves/current_player/midgame_dense` | `4.1642–4.1895 µs` | new benchmark |
| `forbidden_moves/current_player/renju_forbidden_cross` | `24.403–24.581 µs` | `24.032–24.272 µs` |

### Search anchors after pass 2

All numbers below are `SearchBot::choose_move()` at depth `3`.

| Benchmark | Time | Pass 1 |
|---|---|---|
| `opening_sparse` | `13.180–13.311 ms` | `13.717–13.854 ms` |
| `early_local_fight` | `13.148–13.245 ms` | `13.614–13.729 ms` |
| `immediate_win` | `1.4407–1.4486 ms` | `1.5889–1.5966 ms` |
| `immediate_block` | `1.7686–1.7827 ms` | `1.9394–1.9676 ms` |
| `anti_blunder_open_three` | `13.194–13.431 ms` | `14.215–14.304 ms` |
| `renju_forbidden_cross` | `17.489–17.690 ms` | `18.819–18.928 ms` |
| `midgame_medium` | `22.643–22.935 ms` | `23.464–23.832 ms` |
| `midgame_dense` | `32.766–33.130 ms` | `33.215–33.394 ms` |

### Notes

- The large core win is Renju immediate-win scanning, because the duplicate
  forbidden check was removed.
- The opponent-reply safety gate now uses a purpose-built boolean query, so it no
  longer allocates a full winning-move list when it only needs to know whether
  two replies exist.
- Search improved modestly across the fixed corpus. The pass is still a quick
  win, not a replacement for the larger future work around localized eval or
  incremental candidate frontiers.

## Optimization pass 3 snapshot

Date: `2026-05-03`

Changes:

- `Board::can_be_renju_forbidden_at()` now uses a directional local guard:
  a candidate must have at least two black stones on one of the four axes before
  the exact Renju forbidden detector runs.
- The exact forbidden detector is unchanged. The guard only rejects impossible
  forbidden candidates earlier.
- `board_perf` now includes a candidate-set legality benchmark to measure the
  path used by bot root/search candidate filtering.

Commands used:

```sh
cargo test -p gomoku-core renju_forbidden_guard_rejects_single_nearby_black_stone
cargo test -p gomoku-core optimized_renju_forbidden_moves_match_full_scan
cargo test -p gomoku-core --test bench_scenarios
cargo bench -p gomoku-core --bench board_perf -- "forbidden_moves/current_player/renju_forbidden_cross|candidate_legality/current_player/renju_forbidden_cross" --noplot
cargo bench -p gomoku-bot --bench search_perf -- renju_forbidden_cross --noplot
```

### Targeted core anchors after pass 3

| Benchmark | Time | Local baseline before pass 3 |
|---|---|---|
| `forbidden_moves/current_player/renju_forbidden_cross` | `7.4991-7.5844 µs` | `7.4633-7.5145 µs` |
| `candidate_legality/current_player/renju_forbidden_cross` | `4.5258-4.5532 µs` | `7.6663-7.7254 µs` |

### Targeted search anchors after pass 3

| Benchmark | Time | Criterion change |
|---|---|---|
| `fast/renju_forbidden_cross` | `15.371-15.556 ms` | `-4.26% to -1.55%` |
| `balanced/renju_forbidden_cross` | `20.034-20.225 ms` | `-6.35% to -5.05%` |
| `deep/renju_forbidden_cross` | `549.30-552.28 ms` | `-4.51% to -3.93%` |

### Notes

- The full forbidden-list benchmark is effectively flat. That path already
  starts from black-nearby candidates, so the stricter guard does not buy much.
- The candidate legality benchmark improves by roughly 41%, which is the more
  relevant hot path for search candidate filtering.
- The search benchmark shows a modest but measurable improvement on the Renju
  legality-pressure scenario.
- Learning: keep this as an in-place core legality optimization, not a new bot
  component. It preserves exact rules behavior and has no meaningful product or
  tuning tradeoff; exposing it as config would add surface area without helping
  evaluation.

## Optimization pass 4 snapshot

Date: `2026-05-03`

Changes:

- `Board::immediate_winning_moves_for()` and
  `Board::has_multiple_immediate_winning_moves_for()` now use a virtual
  directional win probe instead of cloning a board and applying/undoing each
  candidate move.
- The probe still calls exact legality first, so Renju forbidden moves remain
  excluded.
- `gomoku-core/tests/bench_scenarios.rs` now compares the optimized immediate
  winning move list against a full apply/undo scan for every benchmark scenario
  and both colors.

Commands used:

```sh
cargo test -p gomoku-core immediate
cargo test -p gomoku-core --test bench_scenarios
cargo bench -p gomoku-core --bench board_perf -- "immediate_winning_moves/current_player|has_multiple_immediate_winning_moves/current_player" --noplot
cargo bench -p gomoku-bot --bench search_perf -- "balanced/(create_double_threat|renju_forbidden_cross|midgame_dense)" --noplot
```

### Targeted core anchors after pass 4

| Benchmark | Result |
|---|---|
| `immediate_winning_moves/current_player/*` | `~23-32%` faster on freestyle anchors |
| `immediate_winning_moves/current_player/renju_forbidden_cross` | `~9%` faster |
| `has_multiple_immediate_winning_moves/current_player/*` | `~22-33%` faster on freestyle anchors |
| `has_multiple_immediate_winning_moves/current_player/renju_forbidden_cross` | `~10%` faster |

### Targeted search anchors after pass 4

| Benchmark | Time | Criterion change |
|---|---|---|
| `balanced/create_double_threat` | `50.600-50.826 ms` | `-10.04% to -9.39%` |
| `balanced/renju_forbidden_cross` | `17.142-17.221 ms` | `-15.14% to -14.44%` |
| `balanced/midgame_dense` | `36.110-36.173 ms` | `-9.98% to -9.00%` |

### Notes

- This is another in-place core optimization rather than a bot component. It
  preserves exact move legality and winning semantics while removing repeated
  board mutation from a hot query used by UI hints and the search safety gate.
- The end-to-end search improvement is larger on safety-heavy positions because
  `opponent_reply_search_probe` calls
  `has_multiple_immediate_winning_moves_for()` many times.

## Optimization pass 5 snapshot

Date: `2026-05-03`

Changes:

- `Board` now stores stones in two compact `u64` bitsets instead of
  `Vec<Vec<Cell>>`.
- `Color` now uses `repr(u8)`, keeping `Cell = Option<Color>` compact.
- `Board::for_each_occupied()` exposes efficient occupied-stone iteration for
  callers that do not need to scan empty cells.
- `SearchBot` static eval and candidate generation now use occupied-stone
  iteration. This is required for the bitboard storage change to be a net
  search win: naive bitboards made full `cell()` scans slower.

Commands used:

```sh
cargo test -p gomoku-core occupied_cells_visit_each_stone_with_color
cargo test -p gomoku-core -p gomoku-bot
cargo bench -p gomoku-core --bench board_perf -- "board_clone/opening_sparse|board_cell_scan/opening_sparse|board_hash/opening_sparse|board_to_fen/opening_sparse" --noplot
cargo bench -p gomoku-bot --bench pipeline_perf -- "pipeline/static_eval/current_player/midgame_dense|pipeline/candidate_moves/r2/midgame_dense" --noplot
cargo bench -p gomoku-bot --bench search_perf -- balanced/midgame_dense --noplot
```

### Targeted core anchors after pass 5

| Benchmark | Time | Local pre-bitboard anchor |
|---|---|---|
| `board_clone/opening_sparse` | `23.741-23.953 ns` | `~129 ns` |
| `board_cell_scan/opening_sparse` | `126.62-128.78 ns` | `~96 ns` |
| `board_hash/opening_sparse` | `565.34-566.08 ns` | `~598 ns` |
| `board_to_fen/opening_sparse` | `271.95-272.84 ns` | `~304 ns` |

### Targeted bot anchors after pass 5

| Benchmark | Time | Criterion change |
|---|---|---|
| `pipeline/static_eval/current_player/midgame_dense` | `574.90-580.76 ns` | `-42.32% to -41.77%` |
| `pipeline/candidate_moves/r2/midgame_dense` | `1.0457-1.0528 µs` | no significant change after occupied-iteration fix |
| `balanced/midgame_dense` | `28.253-28.413 ms` | `-12.04% to -11.40%` |

### Notes

- Compact storage is a clear win for clone-heavy search paths and serialized
  board utilities, but `Board::cell()` now costs two bit checks. Avoid
  full-board `cell()` scans in hot loops; iterate occupied bits instead.
- The search improvement came only after routing eval and candidate generation
  through `Board::for_each_occupied()`. A storage-only bitboard conversion
  regressed end-to-end search because empty-cell scans became more expensive.
- Keep bitboard details inside core for now. Bot code should depend on semantic
  helpers (`is_empty`, `has_color`, `for_each_occupied`) rather than accessing
  raw storage.
