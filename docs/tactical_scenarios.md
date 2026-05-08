# Tactical Scenario Corpus

Purpose: document the focused one-move tactical scenarios used by
`gomoku-eval tactical-scenarios`.

This corpus is not a tournament replacement. It answers narrower questions:

- Does a search config preserve obvious tactical safety?
- Does a proposed change regress known tactical shapes?
- What did the move cost in depth, nodes, safety probes, candidate generation,
  legality checks, and time?

For ranking or product-strength claims, use tournaments after the scenario sweep.

Source of truth:

- Scenario boards: `gomoku-bot-lab/benchmarks/scenarios.rs`
- Tactical case definitions: `gomoku-bot-lab/gomoku-eval/src/scenario.rs`
- Shape vocabulary: [`tactical_shapes.md`](tactical_shapes.md)

## Roles

`hard_safety_gate` cases are regression guards. A new safety gate should pass
these before it is compared in tournaments.

`diagnostic` cases are tactical probes. They are useful for understanding bot
behavior, but they are not promotion gates on their own. In particular, a local
broken-three creation case should not drive an optimization if depth 3 already
solves it cheaply.

## Expected Moves

Each active case defines an expected move set. A config passes the case only when
the chosen move is in that set.

Renju legality-only positions are intentionally not active tactical gates. They
belong in core/search legality coverage unless they test a real tactical
judgment beyond "do not play a forbidden move."

The expected move check is intentionally concrete and narrow. The surrounding
metadata explains what the case is trying to measure:

- `role`: whether a miss is a hard safety regression or a diagnostic miss.
- `layer`: whether the case is local, priority/race, or combo-oriented.
- `intent`: whether the bot is completing, creating, reacting, preventing,
  counter-threatening, or creating a double threat.
- `shape`: the primary expected-move shape when the case has one. Priority
  cases may contain other competing shapes on the board.

The JSON report uses schema version `3`. Each result records those metadata
fields, and the report includes grouped summaries by role, layer, and intent so
hard-gate failures do not get buried inside diagnostic misses.

## Tactical Layers

The corpus is split into three explicit layers:

- `local_*`: one localized threat fact in isolation. These cases cover complete,
  create, react, and prevent operations around a single shape.
- `priority_*`: two tactical ideas compete, and the bot must choose the higher
  value one. These cases are about ordering, not shape detection alone.
- `combo_*`: one move creates or resolves multiple connected threats.

The local layer uses the vocabulary in [`tactical_shapes.md`](tactical_shapes.md).
It is intentionally asymmetric. Reacting to an opponent `OpenFour` by blocking
one endpoint is not a meaningful local fixture because the opponent still has
the other completion. The useful open-four cases are completing one, creating
one, preventing one before it exists, or prioritizing a race against another
threat.

## How This Maps To Bot Strategy

The scenario corpus should drive the next search work, but only at the right
layer:

- `local_*` cases define the minimum tactical language. They are useful for
  regression tests and for validating local threat facts.
- `priority_*` cases test whether the bot chooses the stronger idea when two
  local tactics compete, such as completing a four before blocking or creating a
  four instead of answering a weaker three.
- `combo_*` cases test whether one move creates multiple problems for the
  opponent.

The next strategic target is above these isolated fixtures: bounded corridor
search. A corridor is a sequence where a gain move creates a local threat, the
defender's concrete reply set is known from the shape, and the attacker uses
that forced reply to steer toward another threat. The scenario corpus should
verify each building block, but a single diagnostic miss should not justify a
broad leaf scan or a product-facing config knob.

The corpus should also protect narrower search experiments. If future candidate
caps or staged candidate sets are tried, the local and priority cases are the
minimum check that tactical must-keep moves were not filtered out. Passing those
cases is still not enough to promote a change; it only means the breadth
reduction did not break the obvious tactical language before tournament testing.

Offensive and defensive bot styles should eventually map to which side's forced
chains receive extra budget. Offensive style searches own forcing continuations;
defensive style searches opponent forcing continuations and avoids letting those
lines start.

Exact board prints are included in the case list below.

| Case | Role | Layer | Intent | Shape | Side | Expected |
| --- | --- | --- | --- | --- | --- | --- |
| `local_complete_open_four` | hard | local | complete | `OpenFour` | Black | `G8` or `L8` |
| `local_react_closed_four` | hard | local | react | `ClosedFour` | Black | `E1` |
| `priority_complete_open_four_over_react_closed_four` | hard | priority | complete | `OpenFour` | Black | `G8` or `L8` |
| `priority_prevent_open_four_over_extend_three` | hard | priority | prevent | `OpenThree` | White | `G8` or `K8` |
| `priority_create_open_four_over_prevent_open_three` | diagnostic | priority | counter | `OpenFour` | White | `B4` or `F4` |
| `local_create_open_four` | diagnostic | local | create | `OpenFour` | Black | `G8` or `K8` |
| `local_create_closed_four` | diagnostic | local | create | `ClosedFour` | Black | `K8` |
| `local_create_broken_four` | diagnostic | local | create | `BrokenFour` | Black | `J8` or `K8` |
| `local_react_broken_four` | diagnostic | local | react | `BrokenFour` | White | `K8` |
| `local_create_open_three` | diagnostic | local | create | `OpenThree` | Black | `G8` or `J8` |
| `local_prevent_open_four_from_open_three` | diagnostic | local | prevent | `OpenThree` | White | `G8` or `K8` |
| `local_create_closed_three` | diagnostic | local | create | `ClosedThree` | Black | `J8` |
| `local_prevent_closed_four_from_closed_three` | diagnostic | local | prevent | `ClosedThree` | White | `K8` |
| `local_create_broken_three` | diagnostic | local | create | `BrokenThree` | Black | `I8` or `J8` |
| `local_prevent_broken_four_from_broken_three` | diagnostic | local | prevent | `BrokenThree` | White | `I8` |
| `combo_create_double_threat` | diagnostic | combo | double threat | - | Black | `J8` |

## Board Legend

- `B`: black stone
- `W`: white stone
- `.`: empty point
- Coordinates use `A1` through `O15`.
- Boards are shown with row 15 at the top, matching printed-board convention.

## Cases

### local_complete_open_four

- Role: `hard_safety_gate`
- Rule: freestyle
- Side to move: Black
- Expected: `G8` or `L8`
- Intent: complete the current open four immediately.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B B B B . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . W . W . W . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### priority_complete_open_four_over_react_closed_four

- Role: `hard_safety_gate`
- Rule: freestyle
- Side to move: Black
- Expected: `G8` or `L8`
- Intent: complete Black's open four instead of reacting to White's closed four
  at `E1`.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B B B B . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W W W W . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### priority_prevent_open_four_over_extend_three

- Role: `hard_safety_gate`
- Rule: freestyle
- Side to move: White
- Expected: `G8` or `K8`
- Intent: prevent Black from replying with `G8` or `K8`, which would create a
  forcing open-four threat, instead of extending White's weaker diagonal line.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B B B . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . W . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . W . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### priority_create_open_four_over_prevent_open_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: White
- Expected: `B4` or `F4`
- Intent: show the counter-threat exception to local open-three prevention.
  Black has an open-three style threat on row 8, but White can create an open
  four on row 4 and therefore does not have to block immediately.
- Current status: diagnostic only. The current `SearchBot` still prefers the
  conservative block, so this should drive future tactical work rather than fail
  the baseline safety gate.

```text
    A B C D E F G H I J K L M N O
15  B . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B B B . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . W W W . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### local_create_open_four

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `G8` or `K8`
- Intent: local create fixture for `OpenFour`; Black can create two
  immediate completions from the row-8 three.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B B B . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . W . W . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### local_create_closed_four

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `K8`
- Intent: local create fixture for `ClosedFour`; Black can create a
  contiguous four with one open completion.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . W B B B . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . W . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### local_react_closed_four

- Role: `hard_safety_gate`
- Rule: freestyle
- Side to move: Black
- Expected: `E1`
- Intent: local react fixture for `ClosedFour`; Black can occupy the only open
  completion after the threat exists.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B . . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . B  3
 2  . . . . . . . . . . . . . . B  2
 1  W W W W . . . . . . . . . . B  1
    A B C D E F G H I J K L M N O
```

### local_create_broken_four

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `J8` or `K8`
- Intent: local create fixture for `BrokenFour`; Black can create a
  one-gap four with one internal completion.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B B . . B . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . W . W . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### local_react_broken_four

- Role: `diagnostic`
- Rule: freestyle
- Side to move: White
- Expected: `K8`
- Intent: local react fixture for `BrokenFour`; White can occupy the internal
  completion after the threat exists.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B B B . B . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . W . W . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### local_create_open_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `G8` or `J8`
- Intent: local create fixture for `OpenThree`; Black can create a two-ended
  three.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B B . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . W . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### local_prevent_open_four_from_open_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: White
- Expected: `G8` or `K8`
- Intent: local prevent fixture; White can occupy either extension square before
  Black turns the open three into an open four.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B B B . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . W . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### local_create_closed_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `J8`
- Intent: local create fixture for `ClosedThree`; Black can create a
  one-ended contiguous three.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . W B B . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### local_prevent_closed_four_from_closed_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: White
- Expected: `K8`
- Intent: local prevent fixture; White can occupy the one open end before Black
  turns the closed three into a closed four.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . W B B B . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### local_create_broken_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `I8` or `J8`
- Intent: local create fixture for `BrokenThree`; Black can create a
  non-contiguous three with an internal rest square.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B . . B . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . W . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### local_prevent_broken_four_from_broken_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: White
- Expected: `I8`
- Intent: local prevent fixture; White can occupy the current rest square before
  Black turns the broken three into a broken four.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B . B B . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . W . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### combo_create_double_threat

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `J8`
- Intent: combo fixture; create simultaneous horizontal and vertical immediate
  winning replies.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . B . . . . .  10
 9  . . . . . . . . . B . . . . .  9
 8  . . . . . . B B B . . . . . .  8
 7  . . . . . . . . . B . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  W . W . W . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . W . W . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

## Future Renju Tactical Cases

The current active tactical sweep stays freestyle because the obvious Renju
fixtures were legality checks in disguise.

Better future Renju tactical cases should test judgment that only matters under
Renju constraints:

- Black should avoid valuing a threat whose required completion or defense point
  is forbidden for Black.
- White should be able to create threats whose required Black defense lands on a
  forbidden point.

Those cases need exact board proofs before becoming hard gates.
