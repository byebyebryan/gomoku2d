# Renju Rules Model

Purpose: define the Renju legality model Gomoku2D should implement before the
next core legality change.

The current core implementation uses a fast shape counter for overline,
double-four, and double-three. That is not precise enough for Renju. Under RIF,
an apparent three or four may stop existing if the continuation that would make
it meaningful is itself forbidden. This doc separates the rule semantics, the
implementation shape, and the validation plan so future tactical/search work can
depend on one rules oracle instead of local interpretations.

## Sources

Primary rule text:

- RIF international rules: <https://www.renju.net/rifrules/>
- RenjuNet advanced forbidden-move tutorial:
  <https://www.renju.net/advanced/>

Reference implementation candidates:

- Gomocup tournament information:
  <https://gomocup.org/detail-information/>
- Piskvork Renju fork: <https://github.com/wind23/piskvork_renju>
- SlowRenju: <https://github.com/wind23/SlowRenju>
- Rapfi: <https://github.com/dhbloo/rapfi>
- `renju_forbid`: <https://github.com/realjustice/renju_forbid>

Gomocup says its Renju tournament rule is based on international Renju rules,
with tournament-specific changes around openings, passing, and automatic draws.
Those changes do not affect forbidden-move legality.

The Piskvork Renju fork is useful because it is Gomocup-era code that accepts
`INFO rule 4` and implements recursive forbidden checks. SlowRenju is by the
same author, supports Renju directly, and uses the same visible `foulr` /
`A3r` / `B4` structure in its shape code. Rapfi is a newer strong open-source
engine with an incremental pattern-board Renju checker. `renju_forbid` is a
small MIT Go implementation that appears to be a port of the Piskvork-style
line logic.

Reference code should be treated as executable evidence, not source to copy.
Piskvork Renju, SlowRenju, and Rapfi are GPL-licensed, so copying code into
Gomoku2D is off-limits unless we deliberately change licensing. `renju_forbid`
is MIT, but it should still be validated rather than trusted blindly.

The inspected Piskvork path is:

- `renju/global.cpp`: exported `forbid(board, pos, size)` wrapper;
- `renju/Class_line4v.cpp`: `line4v::foulr`, the forbidden classifier;
- `renju/Class_line.cpp`: line-level five, overline, four, and three helpers.

`foulr` checks exact five first, then double-four, double-three, and overline.
Its double-three path asks whether apparent threes can actually become straight
fours, and calls back into forbidden checking for those imagined continuations.

The inspected SlowRenju path is:

- `Shape/line4v.cpp`: `line4v::foulr`, `line4v::A3r`, `line4v::B4`;
- `Shape/line.cpp`: line-level `A3`, `B4`, `A5`, and `A6`.

The inspected Rapfi path is:

- `Rapfi/game/board.cpp`: `Board::checkForbiddenPoint`;
- `Rapfi/game/board.h`: the public forbidden-point query;
- `Rapfi/game/pattern.cpp`: Renju pattern table generation.

Rapfi differs architecturally: it first marks possible forbidden points through
incremental pattern tables, then recursively rejects false forbidden points in
`checkForbiddenPoint`. That makes it a useful independent cross-check because it
is not simply the same line-counter code shape as Piskvork/SlowRenju.

## Semantics

Only Black has forbidden moves. White can win with a five or longer line and
does not have overline, double-four, or double-three restrictions.

For Black, the decision order should be:

1. If the move creates an exact five-in-a-row, it is a legal win.
2. Otherwise, if the move creates an overline, it is forbidden.
3. Otherwise, if the move creates more than one real four, it is forbidden.
4. Otherwise, if the move creates more than one real three after RIF 9.3
   filtering, it is forbidden.
5. Otherwise, it is legal.

This order follows the RIF wording that White wins if Black creates a forbidden
shape "without at the same time attaining five in a row." It also matches the
Piskvork Renju `foulr` structure, which checks exact five before double-four,
double-three, and overline.

### Real Four

A real four is not just any five-cell window with four Black stones and one
empty point. It must have at least one legal completion to an exact five.

Examples of apparent fours that should not count:

- the only completion creates an overline;
- the only completion is a forbidden double-four or double-three;
- every completion is blocked by edge or White stone.

The double-four rule counts real fours, not apparent shape windows.

### Straight Four

A straight four is an unbroken four with two different legal ways to add one
stone and make an exact five.

For Black, both endpoint completions must be legal. If one endpoint is forbidden
or makes an overline, that line is not a straight four for double-three
evaluation.

### Real Three

A real three is a row of three stones where Black can add one stone, without
making a five at the same time, to make a legal straight four.

This is the source of the current bug class. A line that looks like an open
three in raw geometry is dead if every possible extension either fails to become
a straight four or lands on a forbidden point. The RenjuNet advanced tutorial
shows exactly this kind of dead-line example: apparent threes and fours can be
removed by forbidden continuations.

### Double-Three

A double-three is forbidden only when more than one real three remains after the
recursive RIF 9.3 test.

RIF 9.3 is recursive:

- imagine Black made the candidate move;
- for each apparent three, try the possible extensions that would make a
  straight four;
- ignore an extension if it also creates overline or double-four;
- if an extension creates another double-three, determine whether that
  double-three is itself forbidden by repeating the same test;
- count only threes that have at least one legal path to a straight four.

This means a double-three detector cannot be a single fixed-window shape count.
It needs either explicit recursion or a logically equivalent search over legal
continuations.

## Current Mismatch

The current core code appears to over-forbid at least one real-game position.

Fixture candidate:

- source: analysis report match `#1548`, before Black's actual reply in the
  contested frame
- side to move: Black
- candidate under question: `E6`
- current Gomoku2D core result: forbidden
- Piskvork Renju `forbid()` result: legal

Position before `E6`:

```text
Black: H8 F8 G8 G6 F9 D7 F7 E8 C6 D10 F6
White: H7 G9 I8 D9 F10 G10 E10 D8 B5 F11 F5
Candidate: Black E6
```

The raw geometry around `E6` contains a horizontal apparent four and two
apparent threes. Piskvork classifies the move as legal, which strongly suggests
at least one apparent three is dead after recursive continuation checks.

This should become the first regression fixture for the new oracle.

## Implementation Design

Keep `Board` as the public rules authority:

- `is_legal_for_color`
- `apply_move`
- `forbidden_moves_for_current_player`
- tactical and wasm threat views that query legal/effective facts

Internally, replace the current shape-count forbidden detector with a dedicated
Renju legality oracle. The exact type name is not important yet; the useful
boundary is:

```text
renju_forbidden_reason(board, black_move) -> Option<ForbiddenReason>
```

where:

```text
ForbiddenReason = Overline | DoubleFour | DoubleThree
```

The reason enum is useful for tests, reports, and future UI, even if the public
game move error continues to expose a single `Forbidden` result.

### Oracle Layers

The oracle should be built from small, testable layers:

1. `creates_exact_five(board, black_move)`
2. `creates_overline(board, black_move)`
3. `real_four_lines(board_after_move, origin)`
4. `legal_straight_four_extensions(board_after_move, three_line)`
5. `real_three_lines(board_after_move, origin)`
6. `forbidden_reason_inner(board, black_move, recursion_context)`

The important dependency direction:

- overline and exact five are direct line checks;
- real four asks whether completions are legal exact-five moves;
- real three asks whether extensions create legal straight fours;
- double-three asks real-three count, which may recurse when an extension itself
  creates another double-three.

### Recursion Guard

The recursive part needs an explicit guard. It should not rely on accidental call
depth.

Use a memo/stack key based on:

- board hash or compact stones around the tested lines;
- candidate move;
- recursion mode: root forbidden check versus "is this extension allowed for
  making a straight four?"

The guard should be diagnostic, not a normal semantic escape hatch. If the guard
ever changes an answer, that position should become a fixture and the oracle
should be fixed. In production, fail-open is less damaging than falsely marking
a legal move forbidden, but the target state is no unresolved recursion in
ordinary play.

### Performance Boundary

The existing `can_be_renju_forbidden_at` guard can stay as an optimization, but
it is not part of the rule semantics.

The exact oracle only needs to run for Black Renju candidates that pass a cheap
necessary condition. The previous optimization that a forbidden move needs at
least two nearby Black stones on one axis is still a good guard candidate, but
the validation suite must prove the guard never changes oracle output.

Do not expose guard choices as bot config. They are implementation details of
`exact_rules`.

## Consumers

The core legality oracle should be the single source of truth.

Search, corridor analysis, wasm hints, and report rendering can still use raw
tactical shape facts, but every Renju-active continuation must cross the oracle
before it receives tactical credit.

Rules for consumers:

- raw Black gains/completions are useful diagnostics, not legal threats;
- forbidden Black squares remain proof evidence for analysis;
- White threats can become stronger because Black's natural reply is forbidden;
- no consumer should locally infer double-three/double-four by counting raw
  windows once the oracle exists.

## Validation Plan

### 1. Reference Harness

Create dev-only harnesses that can compare Gomoku2D against external Renju
checkers.

Preferred shape:

- keep external references cloned under `/tmp` or another external cache;
- compile tiny C++ wrappers for Piskvork Renju, SlowRenju, and Rapfi;
- call `renju_forbid` as a separate Go checker when useful;
- feed fixtures as JSON/CSV;
- write comparison output as a local artifact under `gomoku-bot-lab/outputs/`
  or a temporary path, not as a production dependency.

This avoids GPL contamination while giving us executable evidence for ambiguous
cases.

Use the references in priority order:

1. RIF/RenjuNet examples when they directly describe the shape.
2. Agreement between Piskvork Renju and SlowRenju.
3. Rapfi as an independent modern engine cross-check.
4. `renju_forbid` as a convenient MIT implementation and sanity check.

If references disagree, keep the position as an ambiguity fixture and decide it
from RIF/RenjuNet text rather than majority vote.

### 2. Golden Fixtures

Add focused core fixtures before broader fuzzing:

- exact five for Black is legal;
- Black overline without exact five is forbidden;
- simple double-four is forbidden;
- simple double-three with two real threes is forbidden;
- legal `4+3` is legal;
- apparent double-three with one dead three is legal;
- apparent double-four with one dead four is legal;
- White can play equivalent shapes legally;
- `#1548/E6` is legal, matching Piskvork.

Each fixture should store:

- board size;
- side to move;
- existing stones;
- candidate move;
- expected legality;
- expected reason when forbidden;
- source: `rif`, `renjunet_tutorial`, `piskvork`, `slowrenju`, `rapfi`,
  `renju_forbid`, or `project_regression`.

### 3. Differential Fuzz

Run random legal-ish midgame boards and compare candidate Black moves against
the external references.

Useful constraints:

- board size `15`;
- only compare empty Black candidate moves;
- generate both sparse and dense positions;
- include positions near edges;
- include positions with overline potential;
- reduce mismatches into small golden fixtures.

Classify every mismatch:

- Gomoku2D bug;
- external-reference bug;
- rule ambiguity that needs manual RIF interpretation;
- invalid generated position.

### 4. Integration Validation

After core legality changes:

- run `cargo test -p gomoku-core`;
- run tactical and lethal scenario suites;
- run replay-analysis smoke on the current published analysis sample;
- run wasm threat-view tests/build;
- inspect one report case with forbidden evidence, including `#1548`.

If legality changes affect many moves, regenerate:

- the anchor bot tournament data/report;
- the analysis data/report;
- any published static report artifacts.

### 5. Performance Validation

Measure before and after:

```sh
cargo bench -p gomoku-core --bench board_perf -- \
  "forbidden_moves/current_player/renju_forbidden_cross|candidate_legality/current_player/renju_forbidden_cross" \
  --noplot

cargo bench -p gomoku-bot --bench search_perf -- renju_forbidden_cross --noplot
```

If the exact oracle is materially slower, optimize only behind the same public
semantic boundary:

- cache per-move forbidden results inside one board/search frame;
- keep cheap necessary-condition guards;
- reuse line projections;
- expose diagnostics, not knobs.

## Implementation Slices

1. Land this design doc.
2. Add the Piskvork comparison harness and `#1548/E6` reference check.
3. Add core golden fixtures with current failures allowed or ignored only while
   the oracle is being replaced.
4. Replace the core forbidden detector with the oracle layers.
5. Run differential fuzz and promote mismatches into fixtures.
6. Review tactical/corridor/wasm consumers for any remaining raw-shape Renju
   assumptions.
7. Regenerate reports and publish once legality behavior is stable.

## Open Decisions

- Should Piskvork be the default behavior oracle when RIF text is ambiguous?
  Proposed answer: yes, unless RIF/RenjuNet examples clearly contradict it.
- Should exact-five override overline when both appear in different directions?
  Proposed answer: yes, matching RIF wording and Piskvork check order.
- Should the public API expose `ForbiddenReason` now?
  Proposed answer: expose internally first; add wasm/UI fields only when a
  product surface needs to explain the reason.
- How much recursion/memoization is necessary?
  Proposed answer: implement the guard from the start, then benchmark before
  adding specialized caches.
