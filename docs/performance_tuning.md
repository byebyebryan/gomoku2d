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
| `anti_blunder_open_three` | freestyle | White | tactical, anti-blunder | repro for the recent search safety fix |
| `renju_forbidden_cross` | renju | Black | renju, forbidden | black to move with a forbidden tactical point |
| `midgame_medium` | freestyle | Black | midgame, medium-density | representative clustered midgame |
| `midgame_dense` | freestyle | Black | midgame, dense | denser midgame with larger frontier/eval cost |

## Benchmark suites

### Core

File: `gomoku-bot-lab/gomoku-core/benches/board_perf.rs`

Current measurements:

- `immediate_winning_moves_for(current_player)`
- `apply_move()` followed by `undo_move()` on a representative legal move
- `forbidden_moves_for_current_player()` on Renju anchor positions

These cover the current quick-win candidates:

- `nearby_empty_moves()`
- `immediate_winning_moves_for()`
- core legality/apply/win path

### Search bot

File: `gomoku-bot-lab/gomoku-bot/benches/search_perf.rs`

Current measurement:

- `SearchBot::choose_move()` at depth `3`

Depth `3` is used because it matches the current browser-side practice bot
configuration in `gomoku-web`.

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

## Current hotspot findings

From code inspection before the first benchmark pass:

1. `gomoku-bot/src/search.rs:evaluate()`
   - full-board scan at every leaf
   - likely the biggest long-term search cost

2. `gomoku-bot/src/search.rs:candidate_moves()`
   - rescans the full board at each node

3. Root anti-blunder prefilter
   - adds extra board work before the main search

4. `gomoku-core/src/board.rs:nearby_empty_moves()`
   - currently uses `BTreeSet`

5. `gomoku-core/src/board.rs:immediate_winning_moves_for()`
   - currently clones a full board once per candidate move

## Current optimization backlog

### Quick wins

1. Rewrite `nearby_empty_moves()` to use a dense seen bitmap instead of
   `BTreeSet`
2. Rewrite `immediate_winning_moves_for()` to clone once and use
   `apply_move()` / `undo_move()` per candidate
3. Skip redundant `is_legal()` checks in search nodes where Renju-black
   forbidden logic is not relevant

### Larger future work

1. More incremental or localized evaluation
2. Incremental candidate frontier maintenance
3. Flat board storage instead of `Vec<Vec<Cell>>`

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
