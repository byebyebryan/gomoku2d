# Bot: Baseline Search

- **File:** `gomoku-bot-lab/gomoku-bot/src/search.rs`
- **Name string:** `"baseline"`
- **Purpose:** Classic alpha-beta search bot — the reference implementation everything else gets compared against.

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

`SearchBot` is now built from `SearchBotConfig`. The compatibility constructors
still exist:

- `SearchBot::new(depth)` creates a custom fixed-depth baseline bot.
- `SearchBot::with_time(ms)` creates a custom time-budgeted baseline bot.

`gomoku-bot` intentionally exposes explicit engine knobs rather than owning
product presets:

| Config field | Meaning |
|---|---|
| `max_depth` | Fixed maximum iterative-deepening depth |
| `time_budget_ms` | Optional per-move wall-clock budget |
| `cpu_time_budget_ms` | Optional per-move Linux thread CPU-time budget |
| `candidate_radius` | Distance around existing stones used to generate candidate moves |
| `safety_gate` | Root safety gate: `opponent_reply_search_probe` or `none` |

Search traces expose explicit pipeline stages: `candidate_source`,
`legality_gate`, and `safety_gate`. Today there is one candidate source family
(`near_all_rN`), one legality gate (`exact_rules`), and one optional safety gate
(`opponent_reply_search_probe` or `none`). Renju forbidden-move checks still use
exact core rules, but core first applies a cheap candidate gate: only empty
cells within 2 spaces of a black stone can be forbidden.

The lab tools define temporary aliases over these fields for experiments:

| Alias | Max depth | Candidate source | Safety gate | Intent |
|---|---:|---|---|---|
| `fast` | 2 | `near_all_r2` | `opponent_reply_search_probe` | cheap comparison target |
| `balanced` | 3 | `near_all_r2` | `opponent_reply_search_probe` | current browser practice-bot depth |
| `deep` | 5 | `near_all_r2` | `opponent_reply_search_probe` | current CLI default depth |

For lab-only ablations, append `+near-all-r1`, `+near-all-r2`, or
`+near-all-r3` to change candidate-source radius. Append `+no-safety` or
`+opponent-reply-search-probe` to choose the safety gate, for example
`search-d5+near-all-r3+no-safety`. These switches measure one pipeline axis at a
time; defaults remain `near_all_r2` plus `opponent_reply_search_probe`.

These aliases are not core bot identity, and they are not character bots yet.
They exist so the lab can benchmark stable configs before deciding whether UI
presets like aggressive or defensive are real enough to expose.

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
    "safety_gate": "opponent_reply_search_probe",
    "move_ordering": "tt_first_board_order",
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
    "search_legality_checks": 400
  },
  "score": 200,
  "budget_exhausted": false
}
```

`nodes` counts alpha-beta search nodes. `safety_nodes` counts the optional root
safety-gate probe, currently `opponent_reply_search_probe`. `total_nodes` is
the aggregate used by eval reporting. Root/search candidate and legality metrics
are split so pipeline-stage costs can be compared independently. Node budgets
are not enforced yet; this is currently a trace and tournament metric.

Failed experimental knobs are removed instead of kept as dormant config fields.
The rejected broad threat-extension and broad shape-eval experiments are
documented in
[`archive/v0_4_search_bot_enhancement_plan.md`](archive/v0_4_search_bot_enhancement_plan.md).
The current direction is depth-oriented: improve the normal search cost first,
then use tactical facts only for cheap safety, move ordering, or narrow forced
branches that improve reached depth under the same budget.

For the next bot slice, `search-d3` is the primary optimization target. Tactical
scenarios remain diagnostics; a change should not be kept just because it fixes a
depth-2 fixture if it loses reached depth or tournament strength against the
current depth-3 baseline.

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

Rather than searching all 225 cells, only cells within Manhattan radius 2 of any existing stone are considered. This typically keeps the branching factor under 30 even in mid-game.

On an empty board, the first move is forced to the center.

**Known weakness:** radius 2 can miss long-range threats in sparse positions. Radius 3 would catch more but grows the branching factor.

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
