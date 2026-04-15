# Bot: Baseline Search

**File:** `gomoku-bot/src/search.rs`  
**Name string:** `"baseline"`  
**Purpose:** Classic alpha-beta search bot — the reference implementation everything else gets compared against.

---

## Algorithm overview

Negamax with alpha-beta pruning and iterative deepening. The bot searches deeper on each iteration, keeping the best result found so far, and cuts off when the time budget is exhausted or a forced win/loss is detected.

```
for depth in 1..=max_depth:
    (score, move) = negamax(board, depth, -∞, +∞)
    if time_budget exceeded or abs(score) >= WIN:
        break
return best_move
```

Iterative deepening gives move ordering for free: the best move from depth N is tried first at depth N+1, which significantly improves alpha-beta cutoffs.

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
