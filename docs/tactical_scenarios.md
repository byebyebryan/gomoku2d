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

`diagnostic` cases are shape probes. They are useful for understanding bot
behavior, but they are not promotion gates on their own. In particular,
`create_broken_three` should not drive an optimization if depth 3 already solves
it cheaply.

## Expected Moves

Each active case defines an expected move set. A config passes the case only when
the chosen move is in that set.

Renju legality-only positions are intentionally not active tactical gates. They
belong in core/search legality coverage unless they test a real tactical
judgment beyond "do not play a forbidden move."

## Shape Pair Fixtures

Shape pair fixtures are diagnostic cases for the vocabulary in
[`tactical_shapes.md`](tactical_shapes.md). They come in offense/defense pairs:

- Offense: the side to move can play the `gain_square` that creates the shape.
- Defense: the side to move can occupy a defense, completion, or rest square for
  the opponent's existing shape. For `OpenFour`, this records either completion
  square even though one block is not enough to stop the threat.

They are not promotion gates yet. They exist so future ordering/eval experiments
can measure whether a change understands the same shape language. Exact board
prints are included in the case list below.

| Case | Shape | Stance | Side | Expected |
| --- | --- | --- | --- | --- |
| `shape_offense_open_four` | `OpenFour` | offense | Black | `G8` or `K8` |
| `shape_defense_open_four` | `OpenFour` | defense | White | `G8` or `L8` |
| `shape_offense_closed_four` | `ClosedFour` | offense | Black | `K8` |
| `shape_defense_closed_four` | `ClosedFour` | defense | White | `L8` |
| `shape_offense_broken_four` | `BrokenFour` | offense | Black | `J8` or `K8` |
| `shape_defense_broken_four` | `BrokenFour` | defense | White | `K8` |
| `shape_offense_open_three` | `OpenThree` | offense | Black | `G8` or `J8` |
| `shape_defense_open_three` | `OpenThree` | defense | White | `G8` or `K8` |
| `shape_offense_closed_three` | `ClosedThree` | offense | Black | `J8` |
| `shape_defense_closed_three` | `ClosedThree` | defense | White | `K8` |
| `shape_offense_broken_three` | `BrokenThree` | offense | Black | `I8` or `J8` |
| `shape_defense_broken_three` | `BrokenThree` | defense | White | `I8` |

## Board Legend

- `B`: black stone
- `W`: white stone
- `.`: empty point
- Coordinates use `A1` through `O15`.
- Boards are shown with row 15 at the top, matching printed-board convention.

## Cases

### take_immediate_win

- Role: `hard_safety_gate`
- Rule: freestyle
- Side to move: Black
- Expected: `G8` or `L8`
- Intent: finish the current open four immediately.

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

### block_immediate_loss

- Role: `hard_safety_gate`
- Rule: freestyle
- Side to move: Black
- Expected: `E1`
- Intent: block White's immediate horizontal win.

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

### win_race_before_blocking

- Role: `hard_safety_gate`
- Rule: freestyle
- Side to move: Black
- Expected: `G8` or `L8`
- Intent: choose the immediate win instead of blocking White at `E1`.

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

### prevent_open_three_reply

- Role: `hard_safety_gate`
- Rule: freestyle
- Side to move: White
- Expected: `G8` or `K8`
- Intent: prevent Black from replying with `G8` or `K8`, which would create a
  forcing open-four threat.

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

### counter_open_three_with_four

- Role: `diagnostic`
- Rule: freestyle
- Side to move: White
- Expected: `B4` or `F4`
- Intent: show the counter-threat exception to `prevent_open_three_reply`.
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

### create_open_four

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `G8` or `K8`
- Intent: create an open four with two immediate winning replies.

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

### shape_offense_open_four

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `G8` or `K8`
- Intent: paired offensive fixture for `OpenFour`; Black can create two
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

### shape_defense_open_four

- Role: `diagnostic`
- Rule: freestyle
- Side to move: White
- Expected: `G8` or `L8`
- Intent: paired defensive fixture for `OpenFour`; either endpoint is a
  completion square, even though one block cannot fully stop an open four.

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
 1  W . W . W . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### shape_offense_closed_four

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `K8`
- Intent: paired offensive fixture for `ClosedFour`; Black can create a
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

### shape_defense_closed_four

- Role: `diagnostic`
- Rule: freestyle
- Side to move: White
- Expected: `L8`
- Intent: paired defensive fixture for `ClosedFour`; White can occupy the only
  open completion.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . W B B B B . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . W . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### shape_offense_broken_four

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `J8` or `K8`
- Intent: paired offensive fixture for `BrokenFour`; Black can create a
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

### shape_defense_broken_four

- Role: `diagnostic`
- Rule: freestyle
- Side to move: White
- Expected: `K8`
- Intent: paired defensive fixture for `BrokenFour`; White can occupy the
  internal completion.

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

### shape_offense_open_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `G8` or `J8`
- Intent: paired offensive fixture for `OpenThree`; Black can create a
  two-ended three.

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

### shape_defense_open_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: White
- Expected: `G8` or `K8`
- Intent: paired defensive fixture for `OpenThree`; White can interrupt either
  extension square before Black creates an open four.

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

### shape_offense_closed_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `J8`
- Intent: paired offensive fixture for `ClosedThree`; Black can create a
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

### shape_defense_closed_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: White
- Expected: `K8`
- Intent: paired defensive fixture for `ClosedThree`; White can occupy the one
  open end before Black extends.

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

### shape_offense_broken_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `I8` or `J8`
- Intent: paired offensive fixture for `BrokenThree`; Black can create a
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

### shape_defense_broken_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: White
- Expected: `I8`
- Intent: paired defensive fixture for `BrokenThree`; White can occupy the
  current rest square.

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

### create_broken_three

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `I8` or `J8`
- Intent: create a non-terminal broken-three shape. This is a diagnostic, not a
  promotion target.

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

### create_double_threat

- Role: `diagnostic`
- Rule: freestyle
- Side to move: Black
- Expected: `J8`
- Intent: create simultaneous horizontal and vertical immediate winning
  replies.

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
