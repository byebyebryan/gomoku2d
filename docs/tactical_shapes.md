# Tactical Shape Vocabulary

Purpose: define the tactical shape terms used by `SearchBot`, tactical
scenarios, and future tactical eval work.

This doc is intentionally local and practical. It does not try to be a complete
Gomoku/Renju theory reference. It defines the facts our bot can detect cheaply
around one candidate move.

Source of truth in code:

- Shape facts: `gomoku-bot-lab/gomoku-bot/src/search.rs`
- Scenario boards: `gomoku-bot-lab/benchmarks/scenarios.rs`
- Focused scenario runner: `gomoku-bot-lab/gomoku-eval/src/scenario.rs`

## Model

A tactical shape fact is move-centric:

- `player`: the side creating the shape.
- `kind`: the shape class.
- `gain_square`: the move that creates the shape.
- `defense_squares`: points the opponent should consider answering.
- `rest_squares`: points the attacker needs later to turn a weaker shape into a
  stronger forcing shape.

Offense means "play the gain square and create this shape."

Defense means "play one of the defense squares to answer this shape." For
weaker shapes, defense may only be a useful interruption rather than a forced
move. For open fours, no single defense is sufficient because the attacker has
multiple winning completions.

## Shape Definitions

Notation:

- `X`: stone for the player creating the shape.
- `O`: opponent stone, board edge, or rule-forbidden blocker.
- `.`: empty legal point.
- `_`: any point outside the five-cell local window.

| Shape | Local pattern | Offense meaning | Defense / cost squares | Forcing |
| --- | --- | --- | --- | --- |
| `Five` | `XXXXX` | Wins immediately. | None. | Yes |
| `OpenFour` | `.XXXX.` | Creates two immediate winning completions. | Both ends are completions; one block is not enough. | Yes |
| `ClosedFour` | `OXXXX.` or `.XXXXO` | Creates one immediate winning completion from a contiguous four. | The single open completion. | Yes |
| `BrokenFour` | `XX.XX`, `X.XXX`, or `XXX.X` | Creates one immediate winning completion through an internal gap. | The gap/completion square. | Yes |
| `OpenThree` | `.XXX.` | Creates a two-ended three that can become an open or closed four. | The two ends. | Yes, but can lose to stronger counter-threats |
| `ClosedThree` | `OXXX.` or `.XXXO` | Creates a one-ended contiguous three. | The single open end. | No |
| `BrokenThree` | `XX.X`, `X.XX`, or equivalent local gap shape | Creates a non-contiguous three that can become a four after a rest move. | Current baseline mirrors `rest_squares`; richer cost analysis can refine this later. | No |

## Priority

The current local priority is intentionally coarse:

1. `Five`
2. `OpenFour`
3. `ClosedFour` / `BrokenFour`
4. `OpenThree`
5. `ClosedThree` / `BrokenThree`

This priority is not yet the static eval. It is a shared vocabulary for
diagnostics, move ordering, safety gates, and later evaluation experiments.

## Offense / Defense Pairs

Every shape should have paired offensive and defensive tests:

- Offense test: applying `gain_square` creates the expected `kind`.
- Defense test: each `defense_square` is a plausible opponent answer to that
  offensive fact.
- Rest test: each `rest_square` is an attacker continuation square for weaker
  shapes.

Examples:

- `OpenFour`: offense `K8` from `H8 I8 J8` creates completions `G8` and `L8`.
- `ClosedFour`: offense `K8` from `O G8, X H8 I8 J8` creates completion `L8`.
- `BrokenFour`: offense `J8` from `H8 I8 L8` creates completion `K8`.
- `OpenThree`: offense `J8` from `H8 I8` creates defense squares `G8` and `K8`.
- `ClosedThree`: offense `J8` from `O G8, X H8 I8` creates defense square `K8`.
- `BrokenThree`: offense `J8` from `H8 K8` creates rest/defense square `I8`.

## Renju

Renju does not change the vocabulary, but it changes legality:

- A Black gain square can be forbidden.
- A Black defense or completion square can be forbidden.
- White can sometimes create threats whose natural Black answer is forbidden.

Those are tactical judgments, not simple legality checks. They should become
separate Renju tactical scenarios only after the freestyle shape vocabulary is
stable.

## Current Limits

- Facts are local to lines through the candidate move.
- Closed and broken threes are non-forcing diagnostics today.
- Broken-three defense currently mirrors rest squares; a future TSS-style pass
  can split cost and rest more precisely.
- The practice bot should consume these facts only where they improve reached
  depth, runtime, or tournament strength under the same budget.
