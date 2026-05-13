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
| `local_complete_open_four` | freestyle | Black | tactical, local, complete, open-four | complete an existing open four |
| `local_react_closed_four` | freestyle | Black | tactical, local, react, closed-four | answer the only completion square of an opponent closed four |
| `priority_complete_open_four_over_react_closed_four` | freestyle | Black | tactical, priority, complete, react | complete an open four instead of reacting to an opponent closed four |
| `priority_prevent_open_four_over_extend_three` | freestyle | White | tactical, priority, prevent, extend | prevent an opponent open three from becoming an open four |
| `priority_create_open_four_over_prevent_open_three` | freestyle | White | tactical, priority, counter-threat, open-four | create a four instead of immediately blocking an open three |
| `local_create_open_four` | freestyle | Black | tactical, local, create, open-four | diagnostic create case for `OpenFour` |
| `local_create_closed_four` | freestyle | Black | tactical, local, create, closed-four | diagnostic create case for `ClosedFour` |
| `local_create_broken_four` | freestyle | Black | tactical, local, create, broken-four | diagnostic create case for `BrokenFour` |
| `local_react_broken_four` | freestyle | White | tactical, local, react, broken-four | diagnostic reaction case for `BrokenFour` |
| `local_create_open_three` | freestyle | Black | tactical, local, create, open-three | diagnostic create case for `OpenThree` |
| `local_prevent_open_four_from_open_three` | freestyle | White | tactical, local, prevent, open-four, open-three | prevent an open three from becoming an open four |
| `local_create_closed_three` | freestyle | Black | tactical, local, create, closed-three | diagnostic create case for `ClosedThree` |
| `local_prevent_closed_four_from_closed_three` | freestyle | White | tactical, local, prevent, closed-four, closed-three | prevent a closed three from becoming a closed four |
| `local_create_broken_three` | freestyle | Black | tactical, local, create, broken-three | diagnostic create case for `BrokenThree` |
| `local_prevent_broken_four_from_broken_three` | freestyle | White | tactical, local, prevent, broken-four, broken-three | prevent a broken three from becoming a broken four |
| `combo_create_double_threat` | freestyle | Black | tactical, combo, double-threat | create simultaneous immediate winning threats |
| `renju_forbidden_cross` | renju | Black | renju, forbidden | black to move with a forbidden tactical point |
| `midgame_medium` | freestyle | Black | midgame, medium-density | representative clustered midgame |
| `midgame_dense` | freestyle | Black | midgame, dense | denser midgame with larger frontier/eval cost |

### Search behavior cases

The corpus also defines `SEARCH_BEHAVIOR_CASES`, which pair scenarios with a
named lab config and expected moves. These are not performance measurements; they
are behavior anchors for the `v0.4` bot-discovery pass.

Current cases exercise the `balanced` lab config:

- complete open four
- react to closed four
- prevent open four over extending a weaker line
- complete open four over reacting to the opponent's four

The tactical scenario runner can also compare ad-hoc search configs while a
slice is under development. Treat those as diagnostic probes: if a config only
reduces counted nodes on already-passing forced-corridor scenarios, it is useful
evidence for the mechanism but not enough to become a product-facing preset.
Discarded experiments should be documented in the active v0.4 plan and removed
from the live lab spec surface. The broad `shape-eval` attempt fixed the
depth-2 broken-three diagnostic, but was discarded because it lost to simply
using `search-d3` and reduced effective depth under CPU budgets.
The follow-up selective frontier local-threat eval was also discarded. It only
scored legal local-threat moves near the last four plies, which avoided the full
candidate scan, but the partial coverage made it inconsistent as a board-value
term. It improved a small number of tactical or short tournament samples while
remaining slower and unstable across D3/D5/D7 capped ablations.
The current `+pattern-eval` experiment is intentionally different: it stays
global, scans five-cell windows instead of recent plies, and filters
completion/extension squares through exact Renju legality. Early 16-game Renju
head-to-heads showed a possible strength signal, so we reran 64-game
comparisons. The stronger sample kept the D3 and D5-cap8 signal
(`search-d3+pattern-eval` over D3 by `49-0-15`, D5-cap8 pattern over D5-cap8 by
`39-0-25`) and made D7-cap8 slightly positive (`35-3-26`). The cost is still
real: D3 pattern averaged `326 ms` per move versus `39 ms` for default eval and
exhausted the `1000 ms` CPU budget on `7.9%` of moves. D7-cap8 pattern averaged
`581 ms` and exhausted budget on `40.9%` of moves. Keep it as an active lab axis
for now, not as a default.

The tactical scenario corpus is documented in
[`tactical_scenarios.md`](tactical_scenarios.md), including board prints,
roles, and expected moves. Hard safety-gate cases are
regression guards; diagnostic cases are not promotion gates on their own.
The shape terms used by those scenarios and the bot's local facts are defined in
[`tactical_shapes.md`](tactical_shapes.md).

The next performance pass should start with measurement rather than another
tactical consumer: identify how much time `search-d3` spends in eval,
candidate generation, legality checks, safety-gate probes, and hidden tactical
work. Only keep a search change if it improves reached depth, average move time,
or tournament score under the same CPU budget.

For the `0.4.1` bot direction, tactical facts were treated as a way to buy
effective depth. That remains the right benchmark framing for future bot work:
the measurement question is not "did this shape detector make one case pass?"
but "did local threat facts let the same budget search a narrower or better
ordered tree without losing tactical safety?" Candidate staging, move ordering,
and selective corridor extension should all report enough metrics to show
whether breadth was reduced and reached depth improved.

Do not use partial local-threat facts as static eval unless they can be made
globally consistent or incrementally maintained across the whole live board. A
partial leaf score can reward the last local fight while missing older threats
elsewhere; that is worse than the crude but consistent global line eval.
Partial/recent-frontier leaf scoring is retired for now. Tactical facts are
safer as ordering/filtering/extension hints because they change which branches
are searched first, not the final board-value semantics. The pattern-eval branch
is the current test of the "globally consistent" side of that rule. It should be
judged by match strength per CPU budget, not by one tactical scenario pass
count.

Do not optimize tactical facts into partial shortcuts before the full scan-vs-
rolling frontier contract is stable. The frontier experiment should have a clear
before/after target:

- same tactical behavior and scenario pass/fail set
- lower annotation, candidate generation, or reply-generation cost
- better reached depth or lower move time under the same CPU budget
- no hidden apply/undo correctness risk from stale threat facts

Now that local-threat annotation has explicit trace metrics, use those counters
to decide whether caching is worth the complexity before maintaining
updated/deleted shapes alongside board state.

## Optimization pass 7 snapshot

Date: `2026-05-04`

This pass measured the global pattern evaluator directly and then removed
obvious redundant work without changing its semantics:

- added a Criterion group for
  `pipeline/static_eval/pattern_eval/current_player`
- cached exact Renju-black legality checks inside one pattern-eval call
- scored black and white five-cell windows in one scan instead of scanning the
  board once per color

Direct static-eval benchmark:

| Benchmark | Before | After | Change |
|---|---:|---:|---:|
| `pipeline/static_eval/pattern_eval/current_player/midgame_dense` | `4.6769-4.7078 us` | `2.6536-2.6895 us` | about `-43%` |
| `pipeline/static_eval/pattern_eval/current_player/renju_forbidden_cross` | `14.351-14.397 us` | `7.3447-7.3995 us` | about `-49%` |

End-to-end effect:

- D3 pattern improved from `45-0-19`, `405 ms`, and `14.1%` budget exhaustion
  to `49-0-15`, `326 ms`, and `7.9%`.
- D5 cap8 pattern stayed positive at `39-0-25` and remained mostly within
  budget (`1.2%` exhausted).
- D7 cap8 pattern moved from neutral to slightly positive (`35-3-26`), but it
  still exhausted budget on `40.9%` of moves.

Conclusion: the optimization is a pure improvement for the experiment, but it
does not change the product decision. `+pattern-eval` remains a useful lab axis;
it is still too expensive to promote as a default.

## Optimization pass 8 snapshot

Date: `2026-05-04`

This pass kept the evaluator choice unchanged and optimized search plumbing:

- precomputed default-board candidate masks for radius `1`, `2`, and `3`
- generated candidates by OR-ing per-stone masks and subtracting occupied cells
  instead of rescanning each occupied stone's local square
- split the default TT-first move-ordering path from tactical ordering, so the
  normal search path no longer wraps every move in `OrderedMove`

Pattern eval stayed enabled only as a benchmark axis. It is not a default
product setting.

Direct pipeline benchmark:

| Benchmark | Before | After | Change |
|---|---:|---:|---:|
| `pipeline/candidate_moves/r2/early_local_fight` | `511.15-514.49 ns` | `95.973-96.935 ns` | about `-81%` |
| `pipeline/candidate_moves/r2/renju_forbidden_cross` | `426.41-429.35 ns` | `125.69-126.10 ns` | about `-70%` |
| `pipeline/candidate_moves/r2/midgame_dense` | `1.0915-1.1042 us` | `154.45-156.73 ns` | about `-86%` |

Focused `choose_move` benchmark:

| Benchmark | Result |
|---|---:|
| `balanced/early_local_fight` | about `-64%` |
| `balanced/renju_forbidden_cross` | about `-52%` |
| `balanced/midgame_dense` | about `-47%` |
| `deep/early_local_fight` | about `-63%` |
| `deep/renju_forbidden_cross` | about `-50%` |
| `deep/midgame_dense` | about `-62%` |

Direct pattern static-eval measurements drifted slightly in repeat runs despite
untouched scoring code. The final focused rerun was roughly `+1-3%` versus the
prior snapshot. Treat that as benchmark/code-layout noise to watch, not a
product concern: the end-to-end default search path improved substantially, and
pattern eval remains lab-only.

## Optimization pass 9 snapshot

Date: `2026-05-04`

This pass replaced clone/apply local-threat annotation with a virtual
board-after-move view. The tactical classifier now reads the hypothetical gain
stone through a small view object instead of cloning the full board and applying
the candidate move for every annotation.

Verification:

- added a parity test comparing virtual local-threat facts against the previous
  clone/apply reference on five, open-four, closed-four, and broken-three cases
- reran local-threat and tactical unit tests
- reran focused tactical scenarios for `search-d3`,
  `search-d5+tactical-first+child-cap-8`, and
  `search-d7+tactical-first+child-cap-8`; all hard safety gates passed

Focused `choose_move` benchmark, compared against the previous Criterion
snapshot:

| Benchmark | Change |
|---|---:|
| `fast/early_local_fight` | about `-72%` |
| `fast/renju_forbidden_cross` | about `-69%` |
| `fast/midgame_dense` | about `-79%` |
| `balanced/early_local_fight` | about `-3%` |
| `balanced/renju_forbidden_cross` | about `-6%` |
| `balanced/midgame_dense` | about `-4%` |
| `deep/early_local_fight` | about `-3%` |
| `deep/renju_forbidden_cross` | within noise, about `+1%` |
| `deep/midgame_dense` | about `-5%` |

Smoke tournament, compared against parent commit `b8d0fc2` with the same
`48`-match Renju run:

Command shape:

```sh
cargo run --release -p gomoku-eval -- tournament \
  --bots search-d1,search-d3,search-d5+tactical-first+child-cap-8,search-d7+tactical-first+child-cap-8 \
  --games-per-pair 8 \
  --opening-policy centered-suite \
  --opening-plies 4 \
  --rule renju \
  --search-cpu-time-ms 1000
```

| Bot | Before | After | Change |
|---|---:|---:|---:|
| `search-d1` | `7.62ms` | `6.74ms` | about `-12%` |
| `search-d3` | `59.52ms` | `60.16ms` | about `+1%` |
| `search-d5+tactical-first+child-cap-8` | `184.66ms` | `174.33ms` | about `-6%` |
| `search-d7+tactical-first+child-cap-8` | `419.92ms` | `397.60ms` | about `-5%` |

Tournament outcomes were unchanged: same `W-D-L`, same pairwise results, and
all `48` games ended naturally. Overall tournament wall time improved from
`30.457s` to `29.354s`, about `-4%`.

Interpretation: this is a quality-neutral hot-path cleanup worth keeping. The
largest win is in shallow configs where the local-threat safety gate dominates
runtime. Deeper configs still spend most time in search/eval, so the gain is
smaller. There is still no direct `pipeline/tactical_annotation` microbench; add
one if we need isolated annotation-only timing instead of end-to-end search and
tournament evidence.

## `0.4.1` reference tournament checkpoint

Date: `2026-05-04`

The latest clean curated report was generated from git commit `822045148556`
with `"git_dirty": false`. It used:

- Renju rules
- centered-suite openings with `4` opening plies
- `64` games per pair across `8` entrants, for `1792` total matches
- `1000 ms` Linux thread CPU time per move
- `120` max moves
- `22` worker threads on an AMD Ryzen 9 7900X host
- `908649 ms` total wall time

Standings:

| Rank | Bot | W-D-L | Avg depth | Avg move time | Budget hit |
|---:|---|---:|---:|---:|---:|
| 1 | `search-d7+tactical-cap-8+pattern-eval` | `303-6-139` | `5.50` | `520.0 ms` | `32%` |
| 2 | `search-d5+tactical-cap-8+pattern-eval` | `285-2-161` | `4.60` | `259.3 ms` | `1%` |
| 3 | `search-d7+tactical-cap-8` | `280-3-165` | `5.59` | `452.4 ms` | `26%` |
| 4 | `search-d3+pattern-eval` | `277-3-168` | `2.80` | `333.5 ms` | `10%` |
| 5 | `search-d5+tactical-cap-8` | `227-2-219` | `4.62` | `203.0 ms` | `1%` |
| 6 | `search-d5` | `218-9-221` | `3.82` | `790.8 ms` | `66%` |
| 7 | `search-d3` | `170-1-277` | `2.91` | `69.3 ms` | `0%` |
| 8 | `search-d1` | `17-4-427` | `1.00` | `7.8 ms` | `0%` |

Interpretation:

- `search-d1` is a credible easy lane only because the local-threat safety gate
  handles hard immediate threat cases; it is not competitive in the reference
  ladder.
- `search-d3` remains the clean default baseline: low cost, stable behavior,
  and no budget pressure.
- `search-d5+tactical-cap-8` is the efficient hard-side line-eval candidate.
  It outranks uncapped D5 while using far less time and budget.
- `search-d7+tactical-cap-8` is stronger than D5 cap8 but pays for it with
  materially higher budget pressure.
- Pattern eval improves raw tournament score, especially at D5 cap8, but it is
  still a strength-versus-budget tradeoff. Keep it as a lab axis until the cost
  side is better understood.

This is a good release checkpoint for `0.4.1`: the lab has a current shared
baseline and enough evidence to avoid another broad tactical experiment. The
next behavior slice should be bounded corridor search using concrete local
gain/defense replies, measured as a bot primitive rather than exposed as a UI
knob.

## `0.4.2` sweep A gauntlet checkpoint

Date: `2026-05-05`

The first `0.4.2` sweep used the `0.4.1` clean reference report as its anchor
source and tested child-cap / pattern-eval candidates against those anchors.
It used:

- Renju rules
- centered-suite openings with `4` opening plies
- `8` candidates x `8` anchors x `32` games, for `2048` total matches
- `1000 ms` Linux thread CPU time per move
- `120` max moves
- `22` worker threads on an AMD Ryzen 9 7900X host
- `1230537 ms` total wall time

This is a screening run, not a full 16-entrant round robin. Candidate rows
measure candidate-vs-anchor results. Anchor rows measure anchor-vs-candidate
results and should be read together with the embedded reference anchor report.

Candidate screen:

| Candidate | W-D-L | Score | Avg depth | Avg move time | Budget exhausted | Breadth |
|---|---:|---:|---:|---:|---:|---:|
| `search-d5+tactical-cap-16+pattern-eval` | `158-5-93` | `62.7%` | `4.53` | `436.7 ms` | `10.6%` | `16.0 / pre 89.6` |
| `search-d7+tactical-cap-4+pattern-eval` | `157-3-96` | `61.9%` | `5.72` | `408.7 ms` | `19.3%` | `7.5 / pre 89.1` |
| `search-d5+tactical-cap-4+pattern-eval` | `155-4-97` | `61.3%` | `4.61` | `200.3 ms` | `0.5%` | `8.1 / pre 90.9` |
| `search-d7+tactical-cap-16+pattern-eval` | `144-7-105` | `57.6%` | `5.03` | `738.0 ms` | `56.0%` | `16.0 / pre 88.4` |
| `search-d7+tactical-cap-4` | `143-2-111` | `56.3%` | `5.71` | `359.9 ms` | `15.8%` | `7.9 / pre 87.3` |
| `search-d5+tactical-cap-4` | `142-2-112` | `55.9%` | `4.56` | `142.3 ms` | `0.3%` | `7.7 / pre 84.2` |
| `search-d7+tactical-cap-16` | `126-1-129` | `49.4%` | `5.29` | `662.1 ms` | `44.2%` | `16.0 / pre 85.7` |
| `search-d5+tactical-cap-16` | `115-1-140` | `45.1%` | `4.58` | `337.7 ms` | `5.7%` | `16.1 / pre 86.3` |

Interpretation:

- Pattern eval remains a real strength signal, but still not a free default.
  The strongest candidate scores are all pattern-eval variants, and the costs
  range from acceptable (`D5 cap4 pattern`) to too heavy (`D7 cap16 pattern`).
- `tactical-cap-16` is not a clear upgrade. The line-eval cap16 candidates
  underperformed and cost more. Pattern cap16 can score well at D5, but it is
  slower and lost its direct anchor comparison against `D5 cap8 pattern`.
- `tactical-cap-4` is more interesting than expected. It often searches about
  as narrowly as cap8 in practice because the safety/order gates already trim
  weak branches; `D5 cap4 pattern` is the cleanest fast hard-side candidate,
  and `D7 cap4 pattern` is the most interesting deeper candidate.
- `D7 cap16` is not attractive right now. Both line and pattern variants spend
  too much budget without producing a convincing screening result.
- Do not promote a new product preset from this gauntlet alone. The next useful
  check is a smaller focused run among the survivors: `D5 cap4`, `D5 cap4
  pattern`, `D5 cap8 pattern`, `D7 cap4`, `D7 cap4 pattern`, and the current
  `D7 cap8` / `D7 cap8 pattern` anchors.

## `0.4.2` sweep B/C candidate-source checkpoint

Date: `2026-05-05`

The next sweep focused on candidate-source breadth rather than child cap. A
radius gauntlet confirmed the expected tradeoff: symmetric `near-all-r3` is too
expensive to be a good default direction, while symmetric `near-all-r1` is too
limiting as a general source. That pushed the sweep toward asymmetric candidate
sources, especially `near-self-r2-opponent-r1`: keep radius 2 around the side to
move, but trim opponent-stone expansion to radius 1.

The full asymmetric gauntlet used:

- Renju rules
- centered-suite openings with `4` opening plies
- `6` candidates x `8` anchors x `32` games, for `1536` total matches
- `1000 ms` Linux thread CPU time per move
- `120` max moves
- `22` worker threads on an AMD Ryzen 9 7900X host
- `632271 ms` total wall time

The report was generated from a dirty workspace because the asymmetric-source
implementation was under review. Treat the numbers as screening evidence, not
publishable anchor data.

Candidate screen:

| Candidate | W-D-L | Score | Avg depth | Avg move time | Budget exhausted | Breadth |
|---|---:|---:|---:|---:|---:|---:|
| `search-d3+near-self-r2-opponent-r1+pattern-eval` | `166-2-88` | `65.2%` | `2.87` | `175.7 ms` | `1.8%` | `67.8` |
| `search-d7+tactical-cap-8+near-self-r2-opponent-r1+pattern-eval` | `165-5-86` | `65.4%` | `5.70` | `430.2 ms` | `19.2%` | `9.0 / pre 72.3` |
| `search-d5+tactical-cap-8+near-self-r2-opponent-r1+pattern-eval` | `161-1-94` | `63.1%` | `4.59` | `170.1 ms` | `0.2%` | `9.2 / pre 72.5` |
| `search-d7+tactical-cap-8+near-self-r2-opponent-r1` | `139-4-113` | `55.1%` | `5.71` | `412.2 ms` | `18.8%` | `9.1 / pre 71.1` |
| `search-d5+tactical-cap-8+near-self-r2-opponent-r1` | `123-1-132` | `48.2%` | `4.63` | `143.8 ms` | `0.1%` | `9.4 / pre 70.2` |
| `search-d3+near-self-r2-opponent-r1` | `100-0-156` | `39.1%` | `2.91` | `41.2 ms` | `0.1%` | `68.4` |

Interpretation:

- The most interesting result is `D3 + self2/opponent1 + pattern-eval`, not a
  deeper capped variant. It tied `D3 + pattern-eval` directly (`16-0-16`) while
  cutting average move time materially in this gauntlet schedule.
- The impact on tactical-cap variants is less clean. `self2/opponent1` shrinks
  the pre-ordering frontier, but tactical ordering plus the child cap still does
  most of the useful pruning. The extra config axis is not yet justified for a
  product preset.
- Plain `self2/opponent1` without pattern eval should not be promoted. It helps
  some same-family line-eval comparisons but underperforms the active
  pattern-eval anchors.
- Do not promote new anchors from this checkpoint. Keep asymmetric candidate
  sources as a lab axis and run a clean survivor comparison only if we need to
  choose between efficient pattern-eval variants.

## `0.4.4` rolling frontier checkpoint

Date: `2026-05-12`

Rolling frontier now has two feature modes:

- `Full` keeps tactical annotations and per-origin move facts for corridor
  portal entry checks.
- `TacticalOnly` keeps tactical annotations and dirty-axis tracking, but skips
  move-fact maintenance. Normal search uses this mode when corridor portals are
  disabled.

Search metrics now split rolling-frontier cost into delta capture, move-fact
maintenance, dirty annotation marking, clean annotation lookup, dirty annotation
recompute, and fallback lookup. The split matters because the earlier rolling
versions looked bad in aggregate but mixed several different costs together.

Focused Renju smoke, `3` entrants x `8` games per pair, centered-suite
openings, `4` opening plies, `1000 ms/move`, `120` max moves:

| Config | Avg move time | Scan query time | Frontier query time | Frontier update time | Shadow mismatches |
|---|---:|---:|---:|---:|---:|
| `search-d3+tactical-cap-8` | `71.9 ms` | `27.2s` | `0s` | `0s` | `0` |
| `search-d3+tactical-cap-8+rolling-frontier-shadow` | `100.5 ms` | `27.2s` | `12.2s` | `0.47s` | `0` |
| `search-d3+tactical-cap-8+rolling-frontier` | `59.1 ms` | `7.4s` | `13.0s` | `0.47s` | `0` |

The same smoke with `+no-safety` isolates the search-annotation path:

| Config | Avg move time | Scan query time | Frontier query time | Frontier update time | Shadow mismatches |
|---|---:|---:|---:|---:|---:|
| `search-d3+tactical-cap-8+no-safety` | `29.7 ms` | `4.5s` | `0s` | `0s` | `0` |
| `search-d3+tactical-cap-8+no-safety+rolling-frontier-shadow` | `45.6 ms` | `4.5s` | `2.6s` | `0.14s` | `0` |
| `search-d3+tactical-cap-8+no-safety+rolling-frontier` | `21.9 ms` | `0s` | `2.8s` | `0.14s` | `0` |

Interpretation:

- Tactical-only rolling is now a net speed win for normal search in these
  smokes: about `18%` faster with the safety gate and about `26%` faster without
  it.
- Shadow mode is expectedly slower because it runs both scan and rolling.
- Move-fact update counts are `0` in tactical-only rolling, which confirms
  normal search no longer pays corridor-specific maintenance cost when portals
  are off.
- Root safety and any still-scan-backed queries remain visible in the scan
  counters, so larger anchor runs should still be used before promoting rolling
  as a default.

Follow-up memo checkpoint:

- Directly writing dirty recomputes back into the rolling frontier is unsafe
  unless undo restores previous raw annotations, so dirty recompute results are
  memoized in `SearchState` by board hash, player, and move instead.
- D3 safety-enabled paired smoke improved to rolling `39.9 ms/move` versus scan
  `49.3 ms/move`.
- D3 no-safety paired smoke improved to rolling `18.8 ms/move` versus scan
  `26.7 ms/move`.
- D5 cap8 paired smoke was roughly tied on time, rolling `159.5 ms/move` versus
  scan `159.7 ms/move`, but rolling searched more nodes and had no budget
  exhaustion in that small sample.
- The root safety gate has since been simplified to a `current_obligation`
  filter over already-generated legal root candidates. It now supports scan,
  rolling, and rolling-shadow threat views directly; rolling uses a root-only
  full frontier because current obligations need active existing-threat facts.
  The older opponent-reply probes are retired rather than kept as lab suffixes.
- Corridor portal continuation now uses the selected threat view for
  materialized attacker entries and defender replies. In non-shadow rolling mode,
  this keeps the search-owned threat-view scan counter at zero; shadow mode still
  runs both scan and rolling answers for parity.

Current-obligation safety checkpoint:

| Config | Avg move time | Scan query time | Frontier query time | Frontier update/rebuild time | Shadow mismatches |
|---|---:|---:|---:|---:|---:|
| `search-d3+tactical-cap-8` | `21.5 ms` | `4.68s` | `0s` | `0s` | `0` |
| `search-d3+tactical-cap-8+rolling-frontier-shadow` | `32.0 ms` | `4.52s` | `2.35s` | `0.40s` | `0` |
| `search-d3+tactical-cap-8+rolling-frontier` | `15.5 ms` | `0s` | `2.59s` | `0.41s` | `0` |

Interpretation:

- The simplified safety gate removed the remaining scan-backed root tax from
  rolling mode in this smoke.
- Rolling mode is now a clear focused-smoke speed win for D3 cap8, but this is
  still a lab suffix and not a product/default promotion.
- Scan-vs-rolling controls are now the correct promotion gate. D7 cap8 is the
  important cautionary case: at `1000 ms/move`, scan won `35-29` while both
  sides were budget-sensitive; at no per-move CPU budget, the same full `64`
  game centered-suite sample was exactly `31-2-31` with matching average
  nodes/depth and faster rolling time. That points to budget interaction, not a
  rolling semantic regression.
- The next durable checkpoint should be a clean scan-vs-rolling matrix for D3
  cap8, D5 cap8, D7 cap8, and one pattern-eval lane, with normal-budget and
  relaxed/no-budget controls plus `--fail-on-shadow-mismatch` shadow smokes.

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
cargo run --release -p gomoku-eval -- tournament --bots search-d1,search-d3,search-d5 --games-per-pair 10 --opening-policy centered-suite --opening-plies 4 --search-cpu-time-ms 100 --max-game-ms 10000 --seed 42 --report-json outputs/gomoku-tournament.json
cargo run --release -p gomoku-eval -- report-html --input outputs/gomoku-tournament.json --output outputs/gomoku-tournament.html --json-href gomoku-tournament.json
```

Curated ranking report, from `gomoku-bot-lab/`:

```sh
mkdir -p reports
cargo run --release -p gomoku-eval -- tournament \
  --bots search-d1,search-d3,search-d5,search-d5+tactical-cap-8,search-d7+tactical-cap-8,search-d3+pattern-eval,search-d5+tactical-cap-8+pattern-eval,search-d7+tactical-cap-8+pattern-eval \
  --games-per-pair 64 \
  --opening-policy centered-suite \
  --opening-plies 4 \
  --search-cpu-time-ms 1000 \
  --max-moves 120 \
  --seed 63 \
  --threads 22 \
  --report-json reports/latest.json
cargo run --release -p gomoku-eval -- report-html --input reports/latest.json --output reports/index.html --json-href latest.json
```

`gomoku-eval` defaults to Renju so ranking tournaments are less dominated by
first-player advantage; pass `--rule freestyle` when validating freestyle product
behavior. Use an even `--games-per-pair` so each pair gets balanced color
coverage. Tournament games run multi-threaded by default and use a seeded
centered opening suite so deterministic bots see varied local positions without
random whole-board scatter. For Linux ranking eval, prefer
`--search-cpu-time-ms` over wall-clock `--search-time-ms`; fixed-depth configs
are still the cleanest reproducibility baseline. The reusable JSON report is the
source of truth for ranking analysis; the HTML report is a derived view that can
be regenerated without rerunning the
tournament. Keep scratch output under `gomoku-bot-lab/outputs/`; curated
reports under `gomoku-bot-lab/reports/` are copied into the public web build as
`/bot-report/`.

Curated replay-analysis reports under `gomoku-bot-lab/analysis-reports/` are
copied into the public web build as `/analysis-report/`. Treat that report as a
companion to the published bot report: it should sample the head-to-head games
between the top two standings in `reports/latest.json`.

For release-quality reports, commit the bot/report implementation first, then
generate `reports/latest.json` and `reports/index.html` from a clean worktree
and commit those artifacts separately. The report records the git revision; if
the tree is dirty at tournament time, the HTML intentionally displays a
`_dirty` suffix and a development-run warning.

Renderer-only report polish is different: keep the clean `latest.json`, rerender
`reports/index.html`, and commit the HTML/renderer change without rerunning the
long tournament. The report JSON provenance should continue to identify the
bot/eval code that produced the match data.

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
10. Clean up remaining bitboard-era hot-path callers: reuse Zobrist tables for
   root hashes, classify virtual cells without `Board::cell()`, and iterate
   occupied stones when generating nearby moves (`2026-05-03`)
11. Precompute radius candidate masks and generate candidates from occupied
    masks instead of scanning local squares around every stone (`2026-05-04`)
12. Replace clone/apply local-threat annotation with a virtual board-after-move
    view while preserving the same tactical facts (`2026-05-04`)
13. Split rolling frontier into full and tactical-only feature modes so normal
    search can cache tactical annotations without maintaining corridor
    move-fact indexes while portals are disabled (`2026-05-12`)
14. Route corridor continuation/reply queries through `ThreatView` and remove
    the pre-move attacker-rank scan surface from the rolling trait
    (`2026-05-12`)

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
| `immediate_winning_moves/current_player/priority_prevent_open_four_over_extend_three` | `28.57–28.68 µs` |
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
| `local_complete_open_four` | `13.91–14.01 ms` |
| `local_react_closed_four` | `13.73–13.85 ms` |
| `priority_prevent_open_four_over_extend_three` | `91.13–92.77 ms` |
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
| `immediate_winning_moves/current_player/priority_prevent_open_four_over_extend_three` | `3.1904–3.2312 µs` | `28.57–28.68 µs` |
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
| `local_complete_open_four` | `1.5889–1.5966 ms` | `13.91–14.01 ms` |
| `local_react_closed_four` | `1.9394–1.9676 ms` | `13.73–13.85 ms` |
| `priority_prevent_open_four_over_extend_three` | `14.215–14.304 ms` | `91.13–92.77 ms` |
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
| `immediate_winning_moves/current_player/priority_prevent_open_four_over_extend_three` | `3.1551–3.1626 µs` | `3.1904–3.2312 µs` |
| `immediate_winning_moves/current_player/renju_forbidden_cross` | `26.990–27.135 µs` | `50.854–51.433 µs` |
| `immediate_winning_moves/current_player/midgame_dense` | `4.4511–4.4783 µs` | `4.4690–4.5294 µs` |
| `has_multiple_immediate_winning_moves/current_player/opening_sparse` | `2.2625–2.2687 µs` | new benchmark |
| `has_multiple_immediate_winning_moves/current_player/local_complete_open_four` | `1.5401–1.5536 µs` | new benchmark |
| `has_multiple_immediate_winning_moves/current_player/priority_prevent_open_four_over_extend_three` | `2.8538–2.8745 µs` | new benchmark |
| `has_multiple_immediate_winning_moves/current_player/renju_forbidden_cross` | `26.573–26.837 µs` | new benchmark |
| `has_multiple_immediate_winning_moves/current_player/midgame_dense` | `4.1642–4.1895 µs` | new benchmark |
| `forbidden_moves/current_player/renju_forbidden_cross` | `24.403–24.581 µs` | `24.032–24.272 µs` |

### Search anchors after pass 2

All numbers below are `SearchBot::choose_move()` at depth `3`.

| Benchmark | Time | Pass 1 |
|---|---|---|
| `opening_sparse` | `13.180–13.311 ms` | `13.717–13.854 ms` |
| `early_local_fight` | `13.148–13.245 ms` | `13.614–13.729 ms` |
| `local_complete_open_four` | `1.4407–1.4486 ms` | `1.5889–1.5966 ms` |
| `local_react_closed_four` | `1.7686–1.7827 ms` | `1.9394–1.9676 ms` |
| `priority_prevent_open_four_over_extend_three` | `13.194–13.431 ms` | `14.215–14.304 ms` |
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
cargo bench -p gomoku-bot --bench search_perf -- "balanced/(combo_create_double_threat|renju_forbidden_cross|midgame_dense)" --noplot
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
| `balanced/combo_create_double_threat` | `50.600-50.826 ms` | `-10.04% to -9.39%` |
| `balanced/renju_forbidden_cross` | `17.142-17.221 ms` | `-15.14% to -14.44%` |
| `balanced/midgame_dense` | `36.110-36.173 ms` | `-9.98% to -9.00%` |

### Notes

- This is another in-place core optimization rather than a bot component. It
  preserves exact move legality and winning semantics while removing repeated
  board mutation from a hot query used by UI hints and the search safety gate.
- The end-to-end search improvement is larger on safety-heavy positions because
  the older `opponent_reply_search_probe` calls
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

## Optimization pass 6 snapshot

Date: `2026-05-03`

Changes:

- Added `Board::hash_with(&ZobristTable)` so root search hashing can reuse the
  searcher's existing table instead of rebuilding hash state through a private
  bot-side scan.
- Routed virtual-cell classification through bitboard helpers instead of
  `Board::cell()`.
- Rewrote `nearby_empty_moves()` and `nearby_empty_moves_for_color()` to iterate
  occupied stones directly instead of scanning the full board for anchor cells.
- Kept `has_multiple_immediate_winning_moves_for()` on the direct scan path
  after the occupied-anchor rewrite regressed dense midgame benchmarks. That
  helper scans possible replies and exits early after two wins, so the simpler
  direct path remains better there.

Commands used:

```sh
cargo test -p gomoku-core -p gomoku-bot
cargo bench -p gomoku-core --bench board_perf -- "board_hash/opening_sparse|immediate_winning_moves/current_player/midgame_dense|has_multiple_immediate_winning_moves/current_player/midgame_dense|candidate_legality/current_player/renju_forbidden_cross" --noplot
cargo bench -p gomoku-bot --bench search_perf -- "balanced/(renju_forbidden_cross|midgame_dense)" --noplot
```

### Targeted anchors after pass 6

| Benchmark | Result |
|---|---|
| `candidate_legality/current_player/renju_forbidden_cross` | small improvement in targeted runs after removing a root hash scan |
| `balanced/renju_forbidden_cross` | effectively neutral in targeted runs |
| `balanced/midgame_dense` | roughly `2%` faster in the targeted occupied-anchor run |

### Notes

- This was a cleanup pass, not a new search strategy. The goal was to stop
  paying accidental bitboard-era adapter costs in callers that are already on
  the hot path.
- The direct-scan result for
  `has_multiple_immediate_winning_moves_for()` is a useful constraint: occupied
  iteration is not automatically faster when a helper's dominant shape is
  "scan candidate replies and return early."
- Future bitboard follow-ups should be profile-driven. The raw storage change
  has already paid off; the remaining wins are likely from higher-level search
  structure, candidate ordering, or localized eval rather than more storage
  plumbing.
