# Bot: SearchBot

- **File:** `gomoku-bot-lab/gomoku-bot/src/search.rs`
- **Legacy name string:** `"baseline"`
- **Purpose:** Configurable alpha-beta search bot and the reference search
  implementation everything else gets compared against.

---

## Algorithm overview

Negamax with alpha-beta pruning and iterative deepening. The bot searches deeper
on each iteration, keeping the best result found so far, and cuts off when the
time budget is exhausted or a forced win/loss is detected. Time-budgeted search
checks the deadline inside the alpha-beta loop, so eval tournaments can compare
fixed-depth configs with a practical per-move cap.

```
for depth in 1..=max_depth:
    (score, move) = negamax(board, depth, -∞, +∞)
    if time_budget exceeded or abs(score) >= WIN:
        break
return best_move
```

Iterative deepening gives move ordering for free: the best move from depth N is tried first at depth N+1, which significantly improves alpha-beta cutoffs.

---

## Explicit config

`SearchBot` is built from `SearchBotConfig`. The compatibility constructors
still exist for legacy `baseline` specs and tests:

- `SearchBot::new(depth)` creates a custom fixed-depth search bot.
- `SearchBot::with_time(ms)` creates a custom time-budgeted search bot.

`gomoku-bot` intentionally exposes explicit engine knobs rather than owning
product presets:

| Config field | Meaning |
|---|---|
| `max_depth` | Fixed maximum iterative-deepening depth |
| `time_budget_ms` | Optional per-move wall-clock budget |
| `cpu_time_budget_ms` | Optional per-move Linux thread CPU-time budget |
| `candidate_radius` | Distance around existing stones used to generate candidate moves |
| `safety_gate` | Root safety gate: `opponent_reply_search_probe`, `opponent_reply_local_threat_probe`, or `none` |
| `move_ordering` | Alpha-beta move ordering: `tt_first_board_order` or lab-only `tactical_first` |
| `child_limit` | Optional lab-only cap on the ordered non-root child frontier searched by alpha-beta |

Search traces expose explicit pipeline stages: `candidate_source`,
`legality_gate`, tactical annotation counters, `safety_gate`, and
`move_ordering`. Today there is one candidate source family (`near_all_rN`), one
legality gate (`exact_rules`), one scan-based local-threat annotation object,
two optional safety gates (`opponent_reply_search_probe`,
`opponent_reply_local_threat_probe`, or `none`), two move-ordering modes
(`tt_first_board_order` default, `tactical_first` lab-only), and an optional
`child_limit` lab cap. Renju forbidden-move checks still use exact core rules,
but core first applies a cheap necessary-condition guard:
a forbidden candidate must have at least two black stones on one of the four
local axes before the exact detector runs.

That Renju guard is deliberately not exposed as a bot component. It is a
correctness-preserving core legality optimization, not a playing-style knob:
the exact legality result is unchanged, while the measured candidate-legality
hot path is cheaper.

The lab tools define temporary aliases over these fields for experiments:

| Alias | Max depth | Candidate source | Safety gate | Intent |
|---|---:|---|---|---|
| `fast` | 2 | `near_all_r2` | `opponent_reply_local_threat_probe` | cheap comparison target |
| `balanced` | 3 | `near_all_r2` | `opponent_reply_local_threat_probe` | current browser practice-bot depth |
| `deep` | 5 | `near_all_r2` | `opponent_reply_local_threat_probe` | current CLI default depth |

For lab-only ablations, append `+near-all-r1`, `+near-all-r2`, or
`+near-all-r3` to change candidate-source radius. Append `+no-safety`,
`+opponent-reply-search-probe`, or `+opponent-reply-local-threat-probe` to choose
the safety gate. Append `+tactical-first` to use local-threat facts for ordering
before alpha-beta visits candidate moves, for example
`search-d5+tactical-first`. Append `+child-cap-N` to limit the ordered non-root
child frontier after candidate generation, legality filtering, and move
ordering, for example `search-d5+tactical-first+child-cap-12`. Root still
considers every legal/safe candidate. Candidate radius and child cap are
intentionally separate: radius defines the discovery boundary, while child cap
tests whether ordering can keep useful deeper-node coverage while alpha-beta
searches fewer children. These switches measure one pipeline axis at a time;
defaults remain `near_all_r2`, `opponent_reply_local_threat_probe`,
`tt_first_board_order`, and no child cap.

These aliases are not durable product identity, and they are not character bots
yet. They exist so the lab can benchmark stable configs before deciding whether
UI presets like aggressive or defensive are real enough to expose.

Search traces include both the result and the config:

```json
{
  "config": {
    "max_depth": 3,
    "time_budget_ms": null,
    "cpu_time_budget_ms": null,
    "candidate_radius": 2,
    "candidate_source": "near_all_r2",
    "legality_gate": "exact_rules",
    "safety_gate": "opponent_reply_local_threat_probe",
    "move_ordering": "tt_first_board_order",
    "child_limit": null,
    "search_algorithm": "alpha_beta_id",
    "static_eval": "line_shape_eval"
  },
  "depth": 3,
  "nodes": 1234,
  "safety_nodes": 56,
  "total_nodes": 1290,
  "metrics": {
    "root_candidate_generations": 1,
    "search_candidate_generations": 80,
    "root_legality_checks": 20,
    "search_legality_checks": 400,
    "root_tactical_annotations": 56,
    "search_tactical_annotations": 0,
    "root_child_cap_hits": 0,
    "search_child_cap_hits": 0,
    "root_child_moves_before_total": 0,
    "search_child_moves_before_total": 0,
    "root_child_moves_after_total": 0,
    "search_child_moves_after_total": 0
  },
  "score": 200,
  "budget_exhausted": false
}
```

`nodes` counts alpha-beta search nodes. `safety_nodes` counts the optional root
safety-gate probe. For `opponent_reply_search_probe`, that is shallow
search-like reply work. For `opponent_reply_local_threat_probe`, it is inspected
root candidates and opponent replies classified through local threat facts, so
compare it as safety-gate work rather than as alpha-beta-equivalent nodes.
`total_nodes` is the aggregate used by eval reporting. Root/search candidate and
legality metrics are split so pipeline-stage costs can be compared
independently. Tactical annotation metrics count reusable local-threat
classification work separately from candidate generation and alpha-beta nodes.
Child-cap metrics count ordered non-root frontier size before and after the
optional `child_limit`; root cap metrics stay zero because root is intentionally
uncapped, and all cap metrics are zero for default uncapped configs.
Node budgets are not enforced yet; this is currently a trace and tournament
metric.

## `v0.4.0` experiment takeaways

The detailed experiment log lives in
[`archive/v0_4_search_bot_enhancement_plan.md`](archive/v0_4_search_bot_enhancement_plan.md).
The canonical lessons are:

- Keep one `SearchBot` implementation for now. A separate `AdvancedSearchBot`
  is not justified until a behavior-changing strategy survives evaluation.
- Failed experimental knobs should be removed instead of kept as dormant config
  fields. Dead toggles make future reports harder to interpret.
- Depth remains the most reliable strength lever. A tactical feature must prove
  that it improves reached depth, runtime, or tournament strength under the same
  budget; fixing one depth-2 fixture is not enough.
- Tactical candidates, immediate-win/block ordering, broad threat extension, and
  broad shape eval all failed their promotion gates. The common failure mode was
  hidden extra work that reduced effective depth or match strength.
- `local_create_broken_three` is a diagnostic, not a target. If depth 3 already
  solves a position cleanly, making depth 2 imitate it is only useful when it is
  cheaper than reaching depth 3 normally.
- TSS vocabulary is useful for facts such as gain, cost/defense, and rest
  squares, but the practice bot should not become a full threat-space-search
  solver in this line. Solver-like work belongs in later analysis modules if
  replay review or puzzles need proof-oriented machinery.

The current direction is depth-oriented: improve the normal search cost first,
then use tactical facts only for cheap safety, move ordering, or narrow forced
branches that improve reached depth under the same budget.

`child_limit` is currently a lab knob, not a default. Early tests show it is
most useful when paired with tactical ordering: pre-cleanup tests showed that a
cap without ordering dropped too much important coverage, while tactical-first
ordering with `child_limit` creates a real breadth-for-depth tradeoff. With root
uncapped, the D5 and D7 `tactical-first + child-cap-8` variants both beat
uncapped `search-d3` in a focused Renju tournament, and D7 beat D5. The clearest
same-depth signal so far is a 64-game Renju head-to-head where
`search-d5+tactical-first+child-cap-8` beat uncapped `search-d5` by `44-1-19`,
searched far fewer nodes, and reached more completed depth under the same
`1000 ms` CPU budget. A follow-up D9 `tactical-first + child-cap-4` variant
reached deeper on average than D7 cap8 but lost the head-to-head, suggesting
cap4 cuts too much breadth. That makes the cap a useful lab axis for
harder/slower search variants, but not yet a product default.

The key assumption is that depth remains the mechanism for seeing long play.
Non-tactical alpha-beta should find winning combinations if it can search deep
enough, but Gomoku's broad candidate set makes that unrealistic without better
breadth control. Local threat facts are therefore search-efficiency data, not a
replacement for search. They should let the bot keep tactically required moves,
order promising moves earlier, stage or cap quiet candidates more safely, and
extend only narrow forcing branches with concrete replies.

Tactical annotation stays scan-based but cache-friendly. `Board` remains the
source of truth; search-side annotation computes local facts into a reusable move
annotation and now feeds both safety and the lab-only `tactical_first` ordering
mode. It can also pair with `child_limit` to test whether ordered tactical
coverage lets alpha-beta search fewer children without changing candidate
discovery. A full frontier model, where a `SearchPosition` tracks changed
candidate masks and threat facts through apply/undo, is a later optimization
experiment. It should wait until the fact schema and consumers are stable and
metrics show annotation or candidate regeneration is worth making incremental.

For `v0.4.1`, the strategic target is a practice bot that climbs a tactical
ladder:

1. Local competence: never miss obvious immediate wins, single forced blocks,
   or clear four-shape reactions.
2. Casual combo play: recognize compound threats and priority races that casual
   human players often discover through probing.
3. Forced-chain steering: eventually spend bounded extra depth on narrow lines
   where local threat facts provide the gain move and concrete defender replies.

This keeps the bot aligned with the product. It should become more interesting
and configurable, not just more solver-like. Offensive and defensive styles
should eventually mean different budget allocation: own forced-chain search
versus opponent forced-chain prevention.

Positive search optimizations should land in place when they preserve exact
behavior and improve measured hot paths. They should become configurable only
when they represent a real tradeoff: strength versus speed, breadth versus
depth, style, safety, or explainability.

For the next bot slice, `search-d3` is the primary optimization target. Tactical
scenarios remain diagnostics; a change should not be kept just because it fixes
a depth-2 fixture if it loses reached depth or tournament strength against the
current depth-3 baseline.

The focused tactical scenario corpus is documented in
[`tactical_scenarios.md`](tactical_scenarios.md). It is layered into `local_*`,
`priority_*`, and `combo_*` cases, with explicit role, layer, intent, and shape
metadata in the JSON report. Use the hard safety-gate cases as regression
guards before tournament ablations; use diagnostic cases to understand behavior
and cost, not as standalone promotion gates.

The tactical shape vocabulary is documented in
[`tactical_shapes.md`](tactical_shapes.md). Shape facts are move-centric records
with a `kind`, `gain_square`, `defense_squares`, and `rest_squares`; this keeps
create, prevent, react, and future eval work tied to the same definitions.

---

## Transposition table

Each position is keyed by a Zobrist hash (64-bit). The table stores:

| Field | Description |
|-------|-------------|
| `depth` | Depth at which this entry was searched |
| `score` | Score found |
| `flag` | `Exact`, `LowerBound`, or `UpperBound` |
| `best_move` | Best move found at this node (used for move ordering) |

On each node, if a TT entry exists at sufficient depth, we return early or tighten the alpha-beta window. The TT move is always tried first in the child loop.

### Zobrist hashing

Hash is computed incrementally — O(1) per node rather than O(board_size²). Each `(row, col, color)` triple has a precomputed random 64-bit value. The turn bit is XORed separately. When making a move, the child hash is:

```
child_hash = parent_hash ^ piece(row, col, color) ^ turn_bit
```

---

## Candidate move generation

Rather than searching all 225 cells, only empty cells within two rows/columns of
any existing stone are considered (`near_all_r2`). This is a square/Chebyshev
radius. The current tournament metrics show a typical generated candidate set
around 90 moves in developed Renju positions; earlier small-position estimates
are no longer a reliable planning number.

On an empty board, the first move is forced to the center.

**Known weakness:** radius 2 can miss long-range threats in sparse positions.
Radius 3 would catch more but grows the branching factor. Candidate radius is
now an explicit lab axis (`near_all_r1`, `near_all_r2`, `near_all_r3`) so future
experiments can trade breadth for reached depth deliberately.

---

## Static evaluation

Called at leaf nodes (depth 0) or terminal positions.

Terminal positions return ±2,000,000 immediately.

For non-terminal positions, the eval scores runs of consecutive same-color stones in all 4 directions (horizontal, vertical, diagonal ↘, diagonal ↗) for both sides and returns `my_score - opponent_score`.

### Run scoring

Each run is characterised by its **length** (2–4) and the number of **open ends** (0 = blocked both sides, 1 = half-open, 2 = fully open). Blocked runs (0 open ends) are ignored. The base values:

| Run length | Base score |
|------------|-----------|
| 4 | 10,000 |
| 3 | 1,000 |
| 2 | 100 |

Score per run = `base × open_ends_count`. A fully open four (score 20,000) is treated as near-forcing. An open three (2,000) is a serious threat.

**Known weakness:** the eval doesn't model threat interactions — two simultaneous open threes (a "double-three") aren't scored higher than their sum. A stronger eval would detect these compound threats explicitly.

---

## Known limitations / future work

- No dedicated threat-space search — misses forcing sequences that require looking ahead at threats specifically
- Eval doesn't detect double-threat patterns (double-three, four+three)
- Candidate radius 2 may miss some long-range setups
- No opening book — always searches from scratch on move 1
- TT grows unbounded (no eviction); for longer matches this could be addressed with a fixed-size table and age-based replacement
