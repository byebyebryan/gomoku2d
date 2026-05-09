# Tactical Shape Vocabulary

Purpose: define the tactical shape terms used by `SearchBot`, tactical
scenarios, and future tactical eval work.

This doc is intentionally local and practical. It does not try to be a complete
Gomoku/Renju theory reference. It defines the shared facts our bot can detect
cheaply around one candidate move or an existing corridor threat.

Source of truth in code:

- Shared tactical shape facts: `gomoku-bot-lab/gomoku-bot/src/tactical.rs`
- Search consumer: `gomoku-bot-lab/gomoku-bot/src/search.rs`
- Corridor consumer: `gomoku-bot-lab/gomoku-bot/src/corridor.rs`
- Replay-analysis consumer tests: `gomoku-bot-lab/gomoku-eval/src/analysis.rs`
- Scenario boards: `gomoku-bot-lab/benchmarks/scenarios.rs`
- Focused scenario runner: `gomoku-bot-lab/gomoku-eval/src/scenario.rs`

## Model

A tactical shape fact is move-centric and policy-neutral:

- `player`: the side creating the shape.
- `kind`: the shape class.
- `origin`: either the candidate move that creates the shape or the existing
  run stone that anchors an already-live corridor threat.
- `defense_squares`: points the opponent should consider answering.
- `rest_squares`: points the attacker needs later to turn a weaker shape into a
  stronger forcing shape.

Create means "play the origin square and create this shape" when the origin is
a candidate move. Existing corridor facts use the origin only as a stable anchor
for dedupe and diagnostics; the important tactical data is still the defense
and rest squares.

Defense squares are local shape facts, not always a scenario role. Depending on
timing, the useful tactical question may be to prevent a stronger shape before
it exists, or to react to a forcing shape after it exists. For open fours, no
single normal reaction is sufficient because the attacker has multiple winning
completions.

The code keeps this split explicit:

- Raw detector: one line-window evaluator produces `LocalThreatFact` for both
  after-move annotations and existing-board corridor threats.
- `SearchThreatPolicy`: ranks facts for alpha-beta ordering and decides which
  tactical facts are "must keep" under child caps. Broken threes are material,
  but not must-keep forcing moves.
- `CorridorThreatPolicy`: filters raw facts into active corridor threats by
  checking legal forcing continuations, then owns defender reply generation and
  attacker corridor move ranking.

This is intentional. SearchBot remains a broad alpha-beta engine; corridor
search remains a narrow proof model. They share facts and tactical policy
helpers, not proof recursion.

## Shape Definitions

Notation:

- `X`: stone for the player creating the shape.
- `O`: opponent stone, board edge, or rule-forbidden blocker.
- `.`: empty legal point.
- `_`: any point outside the five-cell local window.

| Shape | Local pattern | Create meaning | Defense / cost squares | Forcing |
| --- | --- | --- | --- | --- |
| `Five` | `XXXXX` | Wins immediately. | None. | Yes |
| `OpenFour` | `.XXXX.` | Creates two immediate winning completions. | Both ends are completions; one block is not enough. | Yes |
| `ClosedFour` | `OXXXX.` or `.XXXXO` | Creates one immediate winning completion from a contiguous four. | The single open completion. | Yes |
| `BrokenFour` | `XX.XX`, `X.XXX`, or `XXX.X` | Creates one immediate winning completion through an internal gap. | The gap/completion square. | Yes |
| `OpenThree` | `..XXX..`, `O.XXX..`, or `..XXX.O` | Creates a two-ended three that can become an open or closed four. | For `..XXX..`, the two adjacent ends. For asymmetrical `O.XXX..` / `..XXX.O`, the two adjacent ends plus the far outer square on the two-space side. Direct adjacent replies are listed before the far outer reply. `O.XXX.O` is boxed and not an active corridor threat. | Yes, but can lose to stronger counter-threats |
| `ClosedThree` | `OXXX.` or `.XXXO` | Creates a one-ended contiguous three. | The single open end. | No |
| `BrokenThree` | Any unblocked five-cell window with exactly three `X`, two `.`, and at least one gap, such as `XX.X.`, `X.XX.`, `X.X.X`, or `X..XX` | Creates a non-contiguous three where at least one empty rest square would create an immediate four-completion threat. | Rest/defense squares are the empty points in that five-cell window that would create a closed or broken four if the attacker played there next. | Yes for corridor reply generation; non-forcing for search ordering |

## Priority

The current local priority is intentionally coarse:

1. `Five`
2. `OpenFour`
3. `ClosedFour` / `BrokenFour`
4. `OpenThree`
5. `BrokenThree`
6. `ClosedThree`

This priority is not one global truth. It is interpreted by consumer policy:

- Search policy ranks closed and broken threes as low-priority tactical
  material and does not let them punch through child caps as must-keep moves.
- Corridor policy ignores closed threes but treats broken threes as active
  imminent threats when their rest squares have legal forcing continuations.
- Renju filtering is policy-aware. Raw facts preserve shape squares; search
  credit and corridor activity only use legal/effective Black continuations.

## Local Scenario Roles

Local tactical scenario fixtures use three roles:

- Create test: applying `gain_square` creates the expected `kind`.
- Prevent test: the side to move occupies a point before the opponent can
  upgrade an existing weaker shape into a stronger one.
- React test: the side to move answers a forcing shape after the opponent has
  already created it.

The roles are intentionally not symmetric. `ClosedFour` and `BrokenFour` have
clear react fixtures because one completion square must be answered. `OpenFour`
does not have a normal react fixture: blocking one endpoint still leaves the
other endpoint as a win unless the current player can win immediately or create
a stronger counter-threat.

Rest tests remain useful for weaker shapes because each `rest_square` is an
attacker continuation square that can turn the shape into a four. For corridor
analysis, `BrokenThree` belongs with imminent threats when those rest squares
exist. For search ordering, broken threes are still non-forcing material so
they do not override immediate safety. Closed threes remain latent material,
not active corridor threats.

Examples:

- `OpenFour`: create `K8` from `H8 I8 J8` creates completions `G8` and `L8`.
- `ClosedFour`: create `K8` from `O G8, X H8 I8 J8` creates completion `L8`.
- `BrokenFour`: create `J8` from `H8 I8 L8` creates completion `K8`.
- `OpenThree`: create `J8` from `H8 I8` creates defense squares `G8` and `K8`.
- Asymmetrical `OpenThree`: with `O G8, X I8 J8 K8`, `H8`, `L8`, and
  `M8` are all valid defenses; playing `M8` keeps either Black extension to a
  closed four instead of an open four.
- Boxed three: with `O G8 M8, X I8 J8 K8`, `H8` and `L8` are open but the
  outer sides are both blocked. This is not an active corridor threat because
  either continuation only creates a closed four that can be answered next.
- `ClosedThree`: create `J8` from `O G8, X H8 I8` creates defense square `K8`.
- `BrokenThree`: create `J8` from `H8 K8` creates rest/defense squares `G8`,
  `I8`, and `L8`; each attacker continuation creates a four-completion threat.
- Split `BrokenThree`: create `L8` from `H8 J8` creates rest/defense squares
  `I8` and `K8`.
- Two-gap `BrokenThree`: create `L8` from `H8 K8` creates rest/defense squares
  `I8` and `J8`.

## Renju

Renju does not change the vocabulary, but it changes whether raw shape squares
are legal and tactically effective:

- A Black gain square can be forbidden, so a raw freestyle shape may fail to
  become a corridor threat.
- A Black completion square can be forbidden, so a raw immediate win may not be
  a legal win.
- A Black defense square can be forbidden, so a White threat may have fewer
  legal Black replies than its raw shape suggests.
- White can sometimes create threats whose natural Black answers are forbidden.

Those are tactical judgments, not simple legality checks. A Renju-aware shape
fact should preserve both the raw square and its legality result. Silent filtering
is risky because "the only block is forbidden" is proof evidence for the
analyzer and useful explanation for the report.

Corridor-facing shape facts should eventually distinguish:

- raw gain/completion/cost squares derived from line shape,
- legal corridor squares the side can actually play,
- forbidden Black squares with a reason such as overline, double three, or
  double four.

Renju scenarios should cover both sides of the asymmetry:

- Black raw attack that is invalid because the gain or completion is forbidden.
- Black defense that fails because every natural answer is forbidden.
- White attack that is stronger because it places Black's answer on a forbidden
  square.
- White defense/counter-threat that remains freestyle-like for White but changes
  Black's reply set.

## Current Limits

- Facts are local to lines through the candidate move or an occupied origin
  stone.
- Search and corridor now share one tactical module/source of truth. Their
  forcing semantics intentionally differ: search treats broken threes as
  non-forcing ordering material, while corridor search treats broken threes as
  active imminent threats when they have legal forcing continuations.
- Closed threes are non-forcing diagnostics for both consumers.
- The practice bot should consume these facts only where they improve reached
  depth, runtime, or tournament strength under the same budget.
