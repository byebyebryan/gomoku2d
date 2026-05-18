# Lethal Threats

Purpose: define the tactical layer above local immediate/imminent threats.

Local shape facts describe what exists on one line. Lethal-threat detection
answers a different question: does the defender have any legal reply that avoids
the attacker's terminal or already-known lethal continuation?

Source of truth in code:

- Terminal classifier: `gomoku-bot-lab/gomoku-bot/src/tactical.rs`
- Lethal scenario harness: `gomoku-bot-lab/gomoku-eval/src/lethal_scenario.rs`

Planned consumers:

- Corridor proof: `gomoku-bot-lab/gomoku-bot/src/corridor.rs`
- Replay analysis: `gomoku-bot-lab/gomoku-analysis/src/lib.rs`

## Layering

`Lethal` is not a `LocalThreatKind`.

The tactical layers are:

- local facts: `OpenFour`, `ClosedFour`, `BrokenFour`, `OpenThree`,
  `BrokenThree`, and related line-window facts;
- legal attacker continuations: which local facts provide legal terminal or
  threat-upgrade moves under the current rule set;
- legal defender coverage: whether one legal defender move removes all relevant
  attacker continuations;
- lethal result: coverage fails.

Shape names propose candidate evidence. They do not prove lethality by
themselves, especially under Renju.

## Terminal Coverage

The first classifier layer is terminal coverage:

```text
terminal lethal =
  defender is to move
  attacker has one or more legal immediate winning completions
  defender has no immediate winning move
  no legal defender reply removes all attacker immediate winning completions
```

Examples:

- Freestyle open four: `.XXXX.` is the common local lethal case because both
  endpoints are legal terminal completions and the defender cannot play both.
- Single blockable four: `OXXXX.` is not lethal when the open completion is a
  legal defender reply.
- Defender race: if the defender can win immediately, terminal coverage does
  not classify the attacker as lethal.
- Renju forbidden block: a White closed or broken four can become lethal if
  Black's only block is forbidden.
- Renju Black open four: not automatically lethal; illegal completions do not
  count as terminal targets.

The current API exposes both the proven result and the evidence:

```text
terminal_lethal_threat(board, attacker) -> Option<LethalThreat>
terminal_lethal_threat_analysis(board, attacker) -> TerminalLethalThreatAnalysis
```

The analysis form is what scenario reports and future UI/report surfaces should
use, because it explains non-lethal cases through terminal targets, defender
immediate wins, and legal covering replies.

## One-Step Lethal Coverage

The second classifier layer answers:

```text
one-step lethal =
  for every legal defender reply,
  attacker has a legal continuation that creates terminal lethal coverage
```

This is the first layer that can classify `4+3` and `3+3` positions. These are
usually the strategically interesting lethal threats because the position is
already lost before an open four appears on the board.

Conservative rule: if any legal defender reply avoids terminal coverage, or
creates an immediate defender win/counter-threat that the attacker cannot prove
through, the position is not proven lethal.

The current API mirrors terminal coverage:

```text
one_step_lethal_threat(board, attacker) -> Option<LethalThreat>
one_step_lethal_threat_analysis(board, attacker) -> OneStepLethalThreatAnalysis
lethal_threat(board, attacker) -> Option<LethalThreat>
```

`lethal_threat` checks terminal coverage first, then one-step coverage. The
one-step analysis records the defender replies considered, the attacker entries
that create terminal coverage after each reply, and any escaping replies.

## Renju Rules

Renju makes hardcoded shape shortcuts unsafe.

- Black attacker: every gain, completion, and next-lethal continuation must be
  legal. Exact-five wins are legal in the current core rules; overlines are
  forbidden. Double-three and double-four gain moves cannot be used as active
  lethal continuations.
- White attacker: Black forbidden replies shrink the defender's legal coverage
  set. A single White terminal target can be lethal if the only Black cover is
  forbidden.
- Black defender: forbidden blocks are proof evidence, not legal branches.
- White defender: White replies are freestyle-like, except White
  counter-threats may force Black toward forbidden answers.

The general rule stays the same: legal targets plus legal coverage decide the
position.

## Scenario Harness

`gomoku-eval lethal-scenarios` validates state classification directly. It is
separate from `gomoku-eval tactical-scenarios`, which tests bot move choice.

The current cases are:

- `lethal_freestyle_open_four`;
- `nonlethal_blockable_closed_four`;
- `nonlethal_defender_immediate_win`;
- `lethal_renju_forbidden_block`;
- `nonlethal_renju_black_open_four_overline_completion`;
- `lethal_freestyle_four_three`;
- `lethal_freestyle_double_open_three`;
- `nonlethal_crossed_broken_threes_shared_block`;
- `nonlethal_single_open_three`.

Each JSON result includes `board_ascii`, and the CLI can print boards directly:

```sh
cargo run -p gomoku-eval -- lethal-scenarios --show-boards
```

Board legend:

- `B`: black stone
- `W`: white stone
- `.`: empty point
- Coordinates use `A1` through `O15`.
- Boards are shown with row 15 at the top, matching printed-board convention.

### lethal_freestyle_open_four

- Rule: freestyle
- Attacker: Black
- Defender to move: White
- Expected: lethal, terminal targets `G8` and `L8`

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
 3  W . . . . . . . . . . . . . .  3
 2  W . . . . . . . . . . . . . .  2
 1  W . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### nonlethal_blockable_closed_four

- Rule: freestyle
- Attacker: Black
- Defender to move: White
- Expected: non-lethal, terminal target `L8`, covering reply `L8`

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
 2  W . . . . . . . . . . . . . .  2
 1  W . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### nonlethal_defender_immediate_win

- Rule: freestyle
- Attacker: White
- Defender to move: Black
- Expected: non-lethal because Black can win immediately at `B5`

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . W W W W . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . B . . . . . . . . . . . . .  4
 3  . B . . . . . . . . . . . . .  3
 2  . B . . . . . . . . . . . . .  2
 1  . B . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### lethal_renju_forbidden_block

- Rule: Renju
- Attacker: White
- Defender to move: Black
- Expected: lethal, terminal target `G10`, no legal Black covering reply

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . W . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . B W B W . . . . . .  9
 8  . . . . . . W B B B . . . . .  8
 7  . . . B W W W W B . . . . . .  7
 6  . . . . . . B . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### nonlethal_renju_black_open_four_overline_completion

- Rule: Renju
- Attacker: Black
- Defender to move: White
- Expected: non-lethal, target `G8`, covering reply `G8`; `L8` is not a legal
  Black terminal target because it would overline.

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . B B B B . B . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . W . W . W . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### lethal_freestyle_four_three

- Rule: freestyle
- Attacker: Black
- Defender to move: White
- Expected: one-step lethal. The four and three cross at `I8`. White can cover
  the immediate `L8` target, but Black can then play `I6` or `I10` to create
  terminal open-four coverage on the vertical line.

```text
    A B C D E F G H I J K L M N O
15  W . . . . . . . . . . . . . W  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . B . . . . . .  9
 8  . . . . . . W B B B B . . . .  8
 7  . . . . . . . . B . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . . . . . . . . . . . . . W  1
    A B C D E F G H I J K L M N O
```

### lethal_freestyle_double_open_three

- Rule: freestyle
- Attacker: Black
- Defender to move: White
- Expected: one-step lethal. Two open threes cross at `I8`. Every direct reply
  to one open three lets Black turn the other open three into terminal
  open-four coverage.

```text
    A B C D E F G H I J K L M N O
15  W . . . . . . . . . . . . . W  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . B . . . . . .  9
 8  . . . . . . . B B B . . . . .  8
 7  . . . . . . . . B . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . . . . . . . . . . . . . W  1
    A B C D E F G H I J K L M N O
```

### nonlethal_crossed_broken_threes_shared_block

- Rule: freestyle
- Attacker: Black
- Defender to move: White
- Expected: non-lethal. Two broken threes cross at the open `I8` point; White
  can play that shared crossing point to block both threats.

```text
    A B C D E F G H I J K L M N O
15  W . . . . . . . . . . . . . W  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . B . . . . . .  9
 8  . . . . . . B B . B . . . . .  8
 7  . . . . . . . . B . . . . . .  7
 6  . . . . . . . . B . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . W . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . . . . . . . . . . . . . W  1
    A B C D E F G H I J K L M N O
```

### nonlethal_single_open_three

- Rule: freestyle
- Attacker: Black
- Defender to move: White
- Expected: non-lethal. White can reply at `G8` or `K8`, and both replies
  escape terminal coverage.

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

Future slices should wire the classifier into replay analysis first, then
consider search integration after reports prove the evidence is reliable.

## Search And Analysis Use

Replay analysis is the first intended consumer. The useful replay boundaries
are:

- terminal move: the actual five;
- lethal onset: the earliest frame in the final suffix where the loser has no
  legal reply avoiding terminal or known-lethal continuation;
- cause boundary: the earlier last escape or forced-corridor entry.

Search integration remains experimental. A proven lethal state can eventually
act like a terminal tactical leaf, but the classifier should be validated in
reports first because false positives would be damaging inside bot search.
