# Corridor Search

Purpose: define corridor search as the shared strategic model for replay
analysis, bot diagnostics, and future player-facing education.

`v0.4.2` started as another bot-tuning slice. The lab had a real harness by
then: configurable search specs, tactical scenarios, tournaments, gauntlets,
reports, and cost metrics. That made the next lesson visible. The current bot
is already competent enough that obvious strength gains are no longer cheap.
More depth, wider candidates, pattern eval, and child caps all matter, but they
quickly become tradeoffs between strength, runtime, and opacity.

Corridor search is the more important direction. It asks why a game collapsed:
what forced sequence existed, what replies were available, where the last escape
was, and whether the loss was a local mistake or a deeper strategic failure.
That makes it useful in three ways:

- It gives replay analysis a concrete foundation instead of a vague "AI review"
  promise.
- It gives the bot lab a way to inspect bot behavior instead of only comparing
  Elo, score, and average search cost.
- It gives future bots a path to become stronger in an explainable way: by
  recognizing and using forcing logic, not by opaque eval-weight tuning.

## Product Thesis

The long-term product value is not just "a Gomoku bot that wins." A more
distinctive Gomoku2D should carry an understanding of the game it is playing.
It should be able to show a player why a position became dangerous, which reply
would have escaped the final sequence, and how a forcing line developed.

That matters because casual Gomoku often lives between two levels:

- A basic player or bot wins when the opponent misses an obvious local threat.
- A stronger player wins by steering forced replies until a later threat becomes
  impossible to cover.

Corridor search targets that middle layer. It is not a full solver, but it can
model the forced tactical corridor where many real games are decided.

## Core Idea

A threat corridor is a bounded forcing sequence created by active tactical
threats. It is semantic, not a hard branch cap.

The corridor exists when one side creates an immediate or imminent threat that
must be answered. It stays active while replies create or answer more immediate
or imminent threats. It exits when a side wins, or when active threats are
neutralized and the attacker has no named forcing continuation.

The search should never fall back to broad quiet-move search. If a move does not
answer an active threat, win immediately, or create a new immediate/imminent
threat, it is outside the corridor.

Corridor state transitions:

- Enter a corridor when either side creates an immediate or imminent threat.
- Stay locked in the corridor while each reply creates or answers another
  immediate/imminent threat.
- Exit the corridor when a side wins, or when all active immediate/imminent
  threats are neutralized and the attacker has no named forcing continuation.
- Return a possible escape when a named legal defender reply exists but the
  model cannot prove that reply remains forced.
- Return unknown only when the model cannot enumerate a meaningful legal reply
  or hits a structural guard before a concrete alternative exists.

The attacker is the side we are proving a forced win for, even if that side is
not currently to move. At attacker nodes, one named corridor move is enough to
continue the proof. At defender nodes, every named corridor reply must remain
inside a forced corridor for the attacker to claim a forced win.

## Vocabulary

| Term | Meaning |
|------|---------|
| Immediate threat | A four threat that wins next turn unless answered, such as a closed four or broken four. |
| Imminent threat | A three threat that can become a four and creates a bounded reply set, such as an open three. |
| Lethal threat | A threat that is effectively terminal in this layer, such as an open four with two winning squares. |
| Corridor entry | The move that starts the active forced corridor being analyzed. |
| Corridor reply | A named move that keeps play inside the threat corridor by answering or creating an active threat. |
| Forced reply | A defender corridor reply that answers the current threat but still leaves the attacker a forced continuation. |
| Escape reply | A legal defender move that exits the detected corridor. It does not prove the defender survives the rest of the game. |
| Possible escape | A legal defender reply the bounded model cannot prove is still losing. Treat it as an escape from the current corridor, but keep the limit evidence. |
| Tactical error | A loss where the decisive corridor is visible within a short forced sequence. |
| Strategic loss | A loss where the decisive corridor reaches far enough back that the loser failed to anticipate a longer forcing plan. |

The shape vocabulary behind these terms lives in
[`tactical_shapes.md`](tactical_shapes.md). Replay-specific outcome labels and
report fields live in [`game_analysis.md`](game_analysis.md).

Implementation boundary: `gomoku-bot::tactical` owns raw local-threat facts and
`CorridorThreatPolicy` owns corridor-specific interpretation of those facts:
active-threat filtering, legal forcing continuations, defender reply generation,
and attacker corridor-move ranking. `gomoku-bot::corridor` owns proof recursion,
outcomes, diagnostics, and bridge-bot fallback behavior. It should not duplicate
shape definitions or reply-selection rules.

## Small Examples

These examples use a one-line slice rather than a full board:

- `X` is the side creating the threat.
- `O` is the defender.
- `_` is an empty playable point.
- `#` is a blocked edge or opponent stone.

### Immediate Threat

```text
XXXX_
```

`X` has four in a row and can win at the empty end. The defender has one direct
job: play the empty point now, unless the defender has an immediate counter-win.
This is an immediate corridor: the next reply is narrow and easy to explain.

If the defender ignores it, the loss is usually a local mistake. If the defender
plays the block but `X` still has another forcing continuation elsewhere, the
block is a forced reply, not an escape.

### Lethal Threat

```text
_XXXX_
```

`X` has an open four. There are two winning squares and the defender can cover
only one. In this layer, that is effectively terminal unless the defender wins
immediately first. The corridor does not need broad search to explain the
position.

### Imminent Threat

```text
__XXX__
```

`X` does not win next move yet, but the shape can become a four from either
side. The defender has a small named reply set instead of one forced square:
direct defenses around the shape, plus possible counter-threats if the defender
can create something at least as urgent.

This is where corridor search becomes more useful than a simple "block the
four" rule. There may be multiple plausible replies, but they are still
tactical replies, not all legal board moves.

### Asymmetric Open Three

```text
O_XXX__
```

One outer side is blocked, but the two-space side still matters. The defender
must consider the adjacent reply and the far outer reply on the open side. This
case is easy to miss if open-three defense is modeled as "only the two adjacent
ends."

The boxed version is different:

```text
O_XXX_O
```

Both outer sides are blocked. This is not an active corridor threat because it
cannot become an open four in the same forcing sense.

### Escape Versus Forced Reply

Suppose `X` creates an imminent threat and `O` has three named replies:

```text
O replies: A, B, C
```

- If `A` neutralizes all active threats and `X` has no named forcing
  continuation, `A` is a confirmed escape from this corridor.
- If `B` blocks the visible threat but allows `X` to create a new immediate
  threat, `B` is a forced reply.
- If `C` is legal but the bounded model cannot prove whether it stays forced,
  `C` is a possible escape. The analyzer should stop there for replay
  classification instead of pretending the corridor is proven.

This distinction is why corridor analysis is useful for player education: it can
separate "you missed the only block" from "you had options, but all visible
options still stayed inside the forcing line."

### Renju Forbidden Reply

Under Renju, Black may have a natural-looking block that is illegal because it
creates a forbidden double-three, double-four, or overline.

```text
White threat -> natural Black block at G10
G10 is forbidden for Black
```

For corridor search, that square is not missing data. It is proof evidence. The
report should show why the square matters tactically and mark it as forbidden
for Black. If every natural Black reply is forbidden, the threat may remain
forced even with only one visible winning square.

## Replay Analysis Role

For finished games, corridor search works backward from the winning move. The
question is not "what is the best move in this position?" The question is:

> Where was the latest losing-side decision that could have escaped the final
> forced corridor?

The analyzer follows the actual ending, checks losing-side decision points, and
classifies alternate corridor replies as forced loss, confirmed escape, possible
escape, forbidden, immediate loss, or unknown. The published report is an
interim workbench for this model, not the final in-game replay UI.

That interim report is still important. It makes the model visible enough to
review the board states, proof frames, and markings before the concept is baked
into a player-facing screen.

## Bot Diagnostics Role

Tournament score tells us which bot won. Corridor analysis can tell us more:

- Did the loser miss a short forced defense?
- Did the winner create a longer forcing corridor?
- Did a bot appear strong because the opponent made a local mistake?
- Did a bot lose despite reasonable search cost because it failed to see a
  forcing plan?

This is why corridor search belongs in the lab before it belongs in the UI. It
lets bot changes be judged by the kinds of wins and losses they create, not just
by aggregate score.

## Bot Role

`0.4.3` and `0.4.4` tested whether corridor search could become a live
`SearchBot` shortcut before the product exposes bot settings. The result is
negative for the current compute budget. Corridor search remains important for
replay analysis, shared tactical vocabulary, and test fixtures, but corridor
portals are not a candidate bot feature.

### Retired Corridor Portal Experiment

The portal idea treated a narrow corridor as a shortcut inside alpha-beta:
after a child move, the bot checked whether that move entered an immediate or
imminent threat corridor, followed the bounded replies, and tried to use the
result as a deeper child score.

The implementation improved over several passes:

- `+corridor-q` proved the shared corridor module could be called from search,
  but leaf quiescence asked too often and too late.
- `+corridor-own-dN-wM` and `+corridor-opponent-dN-wM` moved the probe to child
  moves and added side-specific portal controls.
- Move-local entry checks, nested re-entry guards, ordered-rank gates, and
  top-N gates reduced obvious over-acceptance.
- `+corridor-proof-only` removed resume churn by accepting only terminal
  corridor proofs and falling back to the original child search on non-terminal
  exits.

Those changes made the measurements honest, but they did not produce a useful
bot knob:

- Resume-from-exit portals distorted scores and multiplied normal searches from
  corridor exits.
- Proof-only portals were safer, but still paid hundreds of branch probes and
  fallbacks per move for too few terminal proofs.
- A `32` game D3 proof-only head-to-head at `1s/move` lost `13-19` to base
  `search-d3` while costing about `176 ms/move` versus `60 ms/move`.
- A `32` game D5+tactical-cap8 proof-only head-to-head lost `15-17` to base
  `search-d5+tactical-cap-8` while costing about `175 ms/move` versus
  `116 ms/move`.

The conclusion is that portal search is not useful in this bounded-compute
shape. Keep the implementation disabled by default and lab-only so old reports
can be reproduced and future ideas can be compared against the failed shape, but
do not include portal variants in anchor tournaments, difficulty ladders,
settings UI, or product copy. If a future experiment revisits this direction,
start from `+corridor-proof-only`; the resume and static-exit modes are
historical controls, not recommended candidates.

The durable output from the portal work is not a stronger bot. It is the shared
`ThreatView` seam, unified search/corridor tactical facts, and report metrics
that make corridor cost visible. Future bot work should focus on cheaper threat
facts, better ordering, and explicit analysis-first corridor proofing rather
than adding more portal tuning knobs.

### Rolling Threat Frontier

Longer term, the cheap local question should be backed by a rolling frontier
model for threat facts. Full-board threat scans are acceptable for the analyzer
and early lab prototypes, but they are the wrong shape for a hot search path as
more search stages depend on threat detection.

The frontier should not replace `Board` or core legality. It should be a derived
index that stays synchronized with apply/undo and answers corridor/search
queries cheaply:

- did the last move create, materialize, or continue a corridor entry?
- what active immediate/imminent threats exist for each side?
- what are the legal defender replies for the current corridor?
- which local facts changed because of the last applied or undone move?
- did the corridor exit because threats were neutralized, became too wide, or
  became illegal under Renju?

The main design shift is from "scan the board to rediscover facts" to "update
the small set of facts whose relevant lines changed." Shape facts are axis-local:
a move affects facts on the four Gomoku axes crossing that move. Renju tactical
annotations add one broader dependency for Black continuation effectiveness, so
the current safe frontier refreshes those annotations globally until that logic
is split into its own cache. In the successful shape, corridor search becomes
close to free because it asks for the already-known active threat and its
already-known reply set.

Important tradeoffs:

- Correctness risk is high. A stale or missing threat fact can make the bot miss
  a forced loss, invent a fake corridor, or mishandle a Renju forbidden reply.
- Renju makes the cache more delicate. Raw shape facts, legal continuations,
  immediate-win checks, and forbidden black squares must stay separate because
  legality can change the tactical meaning of a square.
- Undo must be exact. Search applies and undoes thousands of moves; frontier
  updates need stack discipline at least as strong as board history.
- The cache has to serve the search API, not the analyzer UI. If we build it
  around report frames, it will become too broad for the hot path.
- Memory overhead is acceptable; semantic drift is not. Favor explicit,
  testable facts over clever compact encodings until the model is stable.

The safe migration path is incremental:

1. Define a `ThreatView` interface for the exact queries used by search and
   corridor search.
2. Implement that interface with the current scan-backed logic first.
3. Add differential tests that compare scan-backed answers after random
   apply/undo sequences, tactical fixtures, and Renju forbidden fixtures.
4. Add a rolling implementation behind a lab/shadow mode and compare it against
   the scan-backed view during tests and selected eval runs.
5. Validate normal-search tactical ordering through the same seam first, because
   it is easier to compare against scan behavior than corridor recursion.
6. Switch corridor entry/reply queries to the rolling view only after normal
   search parity and the portal semantics are already stable.

Steps 1 and 2 are now the `0.4.3` cleanup boundary, but only for the minimal
search-facing seam. `gomoku-bot::tactical` exposes a `ThreatView` contract and a
`ScanThreatView` reference backed by the existing scanner for the queries the
current portal code actually uses:

- current active immediate/imminent corridor threats,
- whether a specific move creates or materializes a local corridor entry,
- defender replies to one active threat,
- attacker move rank for tactical ordering,
- search-ordering tactical annotation for a candidate before it is played.

Rolling frontier is still likely the next structural step, but not the next
blind rewrite. First keep normal search and the portal asking the same local
questions through the same seam; then make those questions incremental.

#### Rolling Frontier Drilldown

The design should be interrogated as a cache architecture, not as a bot feature.
The clean split is:

- `Board`: authoritative stones, turn, result, rule config, apply/undo, and
  exact legality.
- `ThreatView`: read-only query contract used by search and corridor logic.
- `ScanThreatView`: current scan-backed reference implementation.
- `RollingThreatFrontier`: optional derived index that implements the same
  contract by updating facts on apply/undo.

The frontier should own normalized tactical facts, not gameplay state:

```text
ThreatFact {
  side,
  kind,
  line_id,
  origin,
  gain_squares,
  cost_squares,
  rest_squares,
  legal_gain_squares,
  legal_cost_squares,
  forbidden_black_squares
}
```

The first implementation should favor explicit facts over compact bit-packing.
Compact storage can come later if profiling proves memory or cache locality is
the bottleneck. The more important invariant is that raw shape facts, legal
continuations, and forbidden Black squares are represented separately.

Update model:

1. Search applies a move to `Board`.
2. The frontier receives the same move and records an undo delta.
3. It invalidates facts along the four axes crossing the move, conservatively
   covering cells up to four steps away.
4. It rebuilds raw shape facts for affected anchors.
5. It reapplies rule/effect filtering, including Renju forbidden handling.
6. Undo pops the exact frontier delta and restores the previous fact sets.

That invalidation window is intentionally conservative. It is acceptable to
rebuild more local facts than strictly necessary. It is not acceptable to miss a
fact or keep a stale legal/forbidden classification.

The current `ThreatView` is intentionally smaller than the final frontier API.
The future rolling contract should grow only when a consumer needs more detail:

- `move_threat_delta(board, mv)`: what threat facts are created, materialized,
  continued, neutralized, or made illegal by this exact move?
- `search_annotation_for_move(mv)`: search-ordering facts for a candidate
  before it is played.
- `active_threats(side)`: current immediate/imminent corridor threats for one
  side.
- `corridor_replies(attacker)`: legal defender replies to the current active
  corridor threat.
- `legal_forcing_continuations(attacker, fact)`: legal gain/completion moves
  for one fact.

Anything needed only for HTML reports should stay outside the hot frontier API.
The analyzer can keep using scan-backed or batch-oriented helpers until the
search-facing frontier is proven.

Validation strategy:

- Differential unit tests compare `ScanThreatView` and `RollingThreatFrontier`
  on every tactical fixture.
- Random apply/undo tests apply legal moves, compare views after each move, then
  undo back to the start and compare again.
- Renju-specific tests cover forbidden attacker continuations, forbidden
  defender cost squares, and White threats whose natural Black replies are
  forbidden.
- Shadow eval runs compute both views while only scan-backed results affect
  behavior, then fail fast on the first mismatch with a board dump and changed
  fact list.
- Only after normal-search shadow mode is clean should corridor entry/reply
  paths become the next rolling consumer.

The main release-boundary implication is that this is too large for the same
checkpoint as the current portal cleanup. `0.4.3` finishes move-local portal
semantics, report metrics, and the scan-backed `ThreatView` seam. `0.4.4`
should be the rolling-frontier lab pass if the seam proves stable.

The `0.4.4` working plan lives in
[`archive/v0_4_4_frontier_plan.md`](archive/v0_4_4_frontier_plan.md).
Player-facing bot settings should wait until a later `0.4.x` slice, likely
`0.4.5`, so the UI exposes product language instead of unresolved cache/search
knobs.

## Renju Overlay

Renju is a legality and threat-effect overlay on the same corridor model, not a
separate proof model.

The analyzer should carry raw and legal tactical facts separately:

- Raw threat square: a shape-derived gain, completion, or cost square before
  Renju legality is applied.
- Legal corridor square: a raw square that the side can legally play and that
  still has the expected tactical effect under Renju.
- Forbidden corridor square: a raw Black square rejected by Renju. This is proof
  evidence, not missing data.

Side-specific implications:

- Black attacker: a raw gain or completion only creates a corridor threat if it
  is legal for Black. Forbidden continuations do not create active threat
  strength.
- Black defender: forbidden cost squares are not valid replies. If every natural
  Black answer to a White threat is forbidden, the threat remains forced for
  rule reasons rather than becoming unknown.
- White attacker: White can create threats whose natural Black answers are
  forbidden. The report should surface those forbidden costs because they
  explain why an apparently obvious block is unavailable.
- White defender: White has no forbidden moves, but White counter-threats can be
  strong specifically because they may force Black toward forbidden answers.

The presentation rule follows the same split. A square can carry a normal threat
or defense hint from one side's perspective and an `F` marker from Black's
perspective. The hint explains why the square matters; `F` explains why Black
cannot use it.

## Model Limits

Corridor search is model-bounded. That is a feature, not a defect.

The model should expose its limits rather than overclaim:

- `forced_win`: the detected corridor was proven under the stated model and
  limits.
- `confirmed_escape`: a defender reply exits the active corridor.
- `possible_escape`: a legal defender reply exists, but the model cannot prove
  that reply is still forced.
- `unknown`: the model could not enumerate a meaningful legal reply or hit a
  structural guard before finding a concrete alternative.

The current implementation uses a corridor depth budget as a safety and
diagnostic control. Conceptually, the corridor itself should bound the search:
every branch must be justified by an active immediate or imminent threat. In
practice, depth and guard limits are still useful while the shape model,
memoization, and pruning are evolving.

The important invariant: a guard or cutoff must never turn an active corridor
into a proven forced win. Once a legal defender reply exists, failure to prove
the reply is still losing is evidence for a possible escape from the current
corridor.

## Non-Goals

- Corridor search is not a full game-theoretic Gomoku/Renju solver.
- It is not generic broad best-move analysis.
- It is not a replacement for the existing alpha-beta bot.
- It should not overclaim certainty without exposing model limits.
- It should not become player-facing until the lab reports are stable enough to
  explain real games cleanly.

## Current `v0.4.3` Checkpoint

The current checkpoint provides:

- a corridor-based replay analyzer,
- bot-owned corridor proof entry points under `gomoku-bot::corridor`,
- shared local-threat facts and search/corridor policy views under
  `gomoku-bot::tactical` consumed by both `SearchBot` and corridor search,
- a scan-backed `ThreatView` seam for future rolling-frontier replacement,
- retired `SearchBot` corridor quiescence evidence from the former
  `+corridor-q` suffix,
- retired default-off `SearchBot` portal suffixes kept only for lab
  reproduction, including `+corridor-own-dN-wM`,
  `+corridor-opponent-dN-wM`, and the safer diagnostic
  `+corridor-proof-only`,
- proof-detail JSON and HTML report generation,
- visual proof frames with board rendering and semantic markers,
- Renju-aware handling for forbidden black replies and illegal black threats,
- published `/analysis-report/` artifacts generated from the current bot
  report's top-two matchup,
- docs that separate strategic model, replay implementation, tactical shapes,
  and bot-tuning evidence.

Known limits:

- The analyzer is still model-bounded and intentionally conservative.
- `possible_escape` is common and acceptable; it means the current model cannot
  prove the branch remains in the forced corridor.
- The report is a lab artifact, not a polished replay-screen feature.
- The retired `+corridor-q` leaf integration was too expensive for live play and
  is no longer a lab suffix.
- The first selective-extension implementation is measurable but not promoted
  as a strength candidate; even after move-local/resume cleanup, smoke runs are
  still weaker and much more budget-bound than base anchors.

## Related Docs

- [`game_analysis.md`](game_analysis.md) — replay analyzer mechanics, report
  schema, CLI workflow, and known implementation limits.
- [`tactical_shapes.md`](tactical_shapes.md) — local tactical shape vocabulary
  and shape facts.
- [`search_bot.md`](search_bot.md) — current `SearchBot` pipeline and tuning
  takeaways.
- [`performance_tuning.md`](performance_tuning.md) — benchmark and tournament
  evidence behind the current bot-lab direction.
- [`archive/v0_4_3_corridor_bot_plan.md`](archive/v0_4_3_corridor_bot_plan.md)
  — working plan for the corridor-aware bot lab pass.
