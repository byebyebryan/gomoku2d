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
- Move-local entry checks and nested re-entry guards reduced obvious
  over-acceptance.
- Rank/top-N gates, static exits, and proof-only mode were tested as additional
  tuning controls, then removed because they added parser/report surface without
  producing a candidate bot direction.

Those changes made the measurements honest, but they did not produce a useful
bot knob:

- Resume-from-exit portals distorted scores and multiplied normal searches from
  corridor exits.
- Proof-only portals were safer than resume-from-exit portals, but still paid
  hundreds of branch probes per move for too few terminal proofs.
- A `32` game D3 proof-only head-to-head at `1s/move` lost `13-19` to base
  `search-d3` while costing about `176 ms/move` versus `60 ms/move`.
- A `32` game D5+tactical-cap8 proof-only head-to-head lost `15-17` to base
  `search-d5+tactical-cap-8` while costing about `175 ms/move` versus
  `116 ms/move`.

The conclusion is that portal search is not useful in this bounded-compute
shape. The current parser no longer accepts portal suffixes. Treat
`+corridor-own-dN-wM`, `+corridor-opponent-dN-wM`, and the later rank/top-N,
proof-only, and static-exit controls as historical report evidence only. Do not
include portal variants in anchor tournaments, difficulty ladders, settings UI,
or product copy.

The durable output from the portal work is not a stronger bot. It is the shared
`ThreatView` seam, unified search/corridor tactical facts, and report metrics
that make corridor cost visible. Future bot work should focus on cheaper threat
facts, better ordering, and explicit analysis-first corridor proofing rather
than adding more portal tuning knobs.

### Retired Leaf Corridor Extension Candidate

After rejecting portals, the next bot-facing corridor experiment did not revive
the portal model. It tested leaf corridor extension: run normal iterative
deepening first, then use any remaining budget on a corridor-enhanced pass that
only extends non-quiet leaf nodes.

The goal is narrower than portal search. Portal search asked whether a child
move could shortcut into a deeper score. Leaf extension asks whether normal
search stopped in the middle of an active tactical corridor. That makes it a
better fit for horizon-effect repair:

1. Run the normal `SearchBot` search exactly as today and keep its best move as
   the safe fallback.
2. If the normal search reaches its configured depth before the compute budget
   expires, run a corridor-enhanced pass with the same root candidates and
   baseline principal-variation ordering.
3. At `depth == 0`, classify the leaf as quiet or non-quiet.
4. Quiet leaves use normal static eval.
5. Non-quiet leaves follow only named corridor moves up to max corridor depth,
   terminal state, corridor exit, or deadline.
6. Terminal corridor results can override immediately. Non-terminal exits must
   not replace the normal-search result in the current terminal-only variant.
7. If the enhanced pass does not finish a comparable root search, keep the
   normal-search result and use partial corridor data only for diagnostics and
   future ordering.

This pass needs a separate transposition-table namespace, or a fresh table,
because changing leaf evaluation changes the meaning of cached scores. Reusing
normal-search TT entries for cutoffs would mix scores produced by different
evaluators. Reusing the normal principal variation for move ordering is safe and
desirable.

Partial results should be tiered:

- Proven terminal win for a root branch: safe decisive override.
- Completed enhanced root search without a terminal win: diagnostics only.
- Incomplete corridor probes: useful for metrics and next-pass ordering only.
- Mixed enhanced/static scores from only some leaves: unsafe as a direct move
  decision because explored leaves received deeper tactical treatment than
  unexplored leaves.

Leaf extension also needs explicit ordering so it spends leftover budget on
meaningful corridors first:

- Prioritize leaves on or near the normal-search principal variation.
- Prioritize leaves with immediate wins or immediate defensive obligations.
- Then prioritize imminent-threat leaves with narrow reply sets.
- Then prioritize leaves whose static score is close enough to affect the root
  choice.
- Inside a corridor, order immediate wins, forced blocks, imminent-threat
  replies, counter-threats, local threat rank, then stable board order.

The first lab shape should be explicit and conservative, for example
`search-dN+leaf-corridor-dM-w3`: normal depth `N`, max leaf corridor depth `M`,
and reply width `3`. Required metrics:

- leaf corridor checks,
- active leaf hits,
- terminal hits,
- static exit evals,
- max-depth exits,
- deadline exits,
- corridor nodes,
- average extra plies,
- enhanced pass completion rate,
- terminal exits reached inside corridor probes,
- terminal root candidates split by winning versus losing proof,
- decisive winning terminal overrides,
- budget exhausted rate.

The first decision to test is not "does this make the bot stronger?" It is
"does this cheaply identify and repair horizon-effect leaves often enough to
justify a second pass?" If not, keep corridor search as analysis infrastructure
and do not add another bot knob.

Initial smoke result: the first static-exit implementation did not meet that
bar. Against plain `search-d3` with a 1s/move budget,
`search-d3+leaf-corridor-d1-w3` lost `1-7` while spending about `749 ms/move`
and exhausting budget on about `53%` of moves;
`search-d3+leaf-corridor-d2-w3` and `search-d3+leaf-corridor-d4-w3` both split
`4-4` while spending about `922-937 ms/move` and exhausting budget on about
`90-93%` of moves. A relaxed 5s smoke for `d2-w3` showed a small `3-1` signal,
but still spent about `4.3 s/move` and exhausted budget on about `70%` of
moves. The current shape is therefore diagnostic only: it extends too many
leaves and mostly converts leftover budget into depth/static exits rather than
cheap decisive proofs.

The follow-up terminal-only conversion fixed the obvious correctness risk:
non-terminal/static corridor work no longer overrides the normal-search move.
That made the shallow smoke safer, but not cheaper. `d1-w3` moved from `1-7`
to `4-4` against plain `search-d3`, while still spending about `759 ms/move`
and exhausting budget on about `60%` of moves. It reached `13,031` terminal
corridor exits, but produced no winning root overrides in the first smoke.
`d2-w3` also split `4-4`, spending about `921 ms/move` and exhausting budget
on about `89%` of moves; it reached `4,454` terminal corridor exits and found
`2` winning terminal root candidates, both of which became root overrides.
That confirms the terminal-only path can surface real decisive proofs, but it
is still too expensive and too rare to promote without better leaf selection.

Replacing attacker-side apply/undo materialization with cached candidate
potential was a clear cost improvement, but not enough to promote the feature.
`d1-w3` still split `4-4`, dropped to about `628 ms/move`, exhausted budget on
about `33%` of moves, reached `27,695` terminal corridor exits, and produced no
root overrides. `d2-w3` still split `4-4`, dropped to about `817 ms/move`,
exhausted budget on about `66%` of moves, reached `20,866` terminal corridor
exits, and produced `7` winning terminal root overrides. All `7` were move
confirmations, not move changes, so the proof pass still mostly confirms wins
the normal search already chose.

### Candidate-Proof Corridor Pass

The next refinement changes the role of corridor search again. Normal search
should rank moves first. Corridor search should then act as a proof pass over a
small selected set of normal-search candidates, not as another evaluator that
re-enters the whole root search.

The contract is:

1. Run normal iterative deepening with corridor proof disabled.
2. Only start proof if normal search completes the configured max depth and the
   shared wall/CPU deadline still has budget remaining.
3. Capture the deepest completed root candidates and scores.
4. Prove the normal best move first, then selected candidates from the
   normal-search ranking.
5. For each candidate, apply the candidate root move and run corridor proof from
   that resulting position.
6. Proof returns only `ProvenWin`, `ProvenLoss`, or `Unknown`.
7. `Unknown` never outranks normal-search score by itself.

This keeps normal search safe: if proof times out, exits the corridor, or cannot
prove a terminal result, the bot keeps the normal-search answer. Proven terminal
information can still change the move:

- If normal best is `ProvenWin`, keep it and count a confirmation.
- If another checked candidate is `ProvenWin` while normal best is not, switch
  to it and count a move change.
- If normal best is `ProvenLoss`, switch to the best checked candidate that is
  not proven loss.

The current lab spelling is `+corridor-proof-cN-dM-wW`: `cN` sets the root
candidate cap, `dM` sets max corridor proof depth, and `wW` sets max reply
width. The current baseline shape is `+corridor-proof-c16-d8-w4`. This spelling
intentionally means "prove the top `N` normal-search candidates regardless of
score gap." Score-gap filtering is intentionally not a config axis because it
lets normal-search confidence suppress the proof pass that is supposed to
challenge normal-search ranking.

Reports display the current baseline suffix as `Corridor Proof` to keep the
UI readable. The full `c16-d8-w4` spelling remains in commands and raw report
JSON so experiments stay reproducible.

The older split suffixes `+leaf-corridor-dM-wW` and `+leaf-proof-cN` remain
historical/parser compatibility names for old reports. New experiments should
use the single `+corridor-proof-cN-dM-wW` suffix so the report label reads as
one concept.

The goal is to spend leftover budget on decisive questions instead of
multiplying broad search work. This does not make corridor proof a product knob;
it remains a lab-only experiment until it produces move changes at acceptable
cost.

Initial sweeps changed the status of this path from purely diagnostic to
promising. The old default shape, `d4-w3` with three close-scoring candidates,
mostly confirmed rank-1 normal-search wins. Widening candidates at proof depth
`4` checked deeper ranks but still produced no move changes. The useful jump
came from proof depth `8` and a power-of-two candidate cap:

- `search-d3+corridor-proof-c16-d8-w3` beat `search-d3` by `42-22` in a
  64-game head-to-head.
- It averaged about `91 ms/move` against base D3's `51 ms/move`, with `0.1%`
  budget exhaustion under a `1000 ms/move` CPU budget.
- It checked `4363` proof candidates, found `167` proven wins and `87` proven
  losses, produced `31` proof-driven move changes, and confirmed `136` normal
  best moves.
- Proven wins were not only rank-1 confirmations: average proven-win rank was
  `1.62`, with a max rank of `14`.

The broader 8-anchor gauntlet showed the limits of corridor proof by itself.
`search-d3+corridor-proof-c16-d8-w3` beat `search-d1` and `search-d3`, but
lost to every stronger anchor and finished `211-301` overall at about
`115 ms/move`. Adding pattern eval made the shape competitive with the lower
anchor tier:

- `search-d3+pattern-eval+corridor-proof-c16-d8-w3` finished `302-210` across
  the same 8-anchor gauntlet.
- It beat `search-d3+pattern-eval` by `36-28`, beat plain `search-d5` by
  `40-24`, tied `search-d5+tactical-cap-8` at `32-32`, and stayed close to
  `search-d7+tactical-cap-8` at `30-34`.
- The cost rose to about `264 ms/move` with `7.6%` budget exhaustion.

Direct same-config head-to-heads are the cleanest read so far:

| Base config | Corridor result | Cost read |
| --- | ---: | --- |
| `search-d1` | `43-21` | strong uplift, still cheap enough |
| `search-d3` | `42-22` | strong uplift, about 2x move cost |
| `search-d5` | `32-32` | no benefit; uncapped D5 is already budget-bound |
| `search-d5+tactical-cap-8` | `30-34` | slight regression |
| `search-d7+tactical-cap-8` | `36-28` | uplift, but already budget-heavy |
| `search-d3+pattern-eval` | `37-27` | useful uplift |
| `search-d5+tactical-cap-8+pattern-eval` | `38-26` | best current signal |
| `search-d7+tactical-cap-8+pattern-eval` | `34-28-2` | uplift, but high budget pressure |

That suggests the candidate-proof model can catch moves normal search misses
when it has enough depth and a broad enough root candidate set, especially when
paired with pattern eval. Proof depth `16` and `32` were much more expensive in
small smokes and started hitting the budget hard.

The reply-width sweep locked the current baseline to `w4`, mostly for cleaner
power-of-two config semantics rather than a large strength gap. In a 640-game
D5+tactical-cap8+pattern round-robin over base, `w3`, `w4`, `w5`, and `w8`,
all proof widths beat base. `w4` and `w5` both beat base `40-24`; `w3` and
`w8` beat base `38-26`. Direct width-vs-width pairings were effectively flat,
with most pairs splitting `32-32`; `w5` edged `w3` by `33-31`. `w8` added no
meaningful strength and cost more than the narrower lanes. Use
`D5 tactical-cap-8 + pattern-eval + corridor-proof-c16-d8-w4` was the first
useful candidate-proof branch, not plain non-pattern corridor proof. The latest
anchor refresh promotes that D5 proof lane to tactical-cap-16 after focused
head-to-head checks showed the wider cap was a better D5 control.

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
6. Switch corridor proof and analysis queries to the rolling view only after
   normal search parity is stable.

Steps 1 and 2 became the `0.4.3` cleanup boundary, but only for the minimal
search-facing seam. `gomoku-bot::tactical` exposes a `ThreatView` contract and a
`ScanThreatView` reference backed by the existing scanner for the queries search,
corridor proof, and analysis actually use:

- current active immediate/imminent corridor threats,
- defender replies to one active threat,
- attacker move rank for tactical ordering,
- search-ordering tactical annotation for a candidate before it is played.

This seam became the bridge into the `0.4.4` rolling-frontier pass. Normal
search and corridor proof now ask the same local questions through
`ThreatView`; rolling mode answers the hot normal-search queries by default,
while scan remains the fallback and comparison path.

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

The release-boundary implication has already played out: `0.4.3` finished the
shared tactical vocabulary, corridor cost metrics, and the scan-backed
`ThreatView` seam; `0.4.4` promotes rolling frontier as the default
normal-search backend after parity checks and focused scan-vs-rolling runs.

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
or defense hint from one side's perspective and forbidden evidence from Black's
perspective. The hint explains why the square matters; the forbidden/caution
visual explains why Black cannot use it. Internal reports may still expose the
raw `forbidden` role explicitly, while the product replay UI should reuse the
existing forbidden visual language.

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

## Current `v0.4.4` Checkpoint

The current checkpoint provides:

- a corridor-based replay analyzer,
- bot-owned corridor proof entry points under `gomoku-bot::corridor`,
- shared local-threat facts and search/corridor policy views under
  `gomoku-bot::tactical` consumed by both `SearchBot` and corridor search,
- rolling-backed `ThreatView` as the default normal-search backend, with
  scan-backed `ThreatView` retained for fallback and comparisons,
- retired `SearchBot` corridor quiescence evidence from the former
  `+corridor-q` suffix,
- retired `SearchBot` portal evidence from former suffixes such as
  `+corridor-own-dN-wM` and `+corridor-opponent-dN-wM`, which are no longer
  accepted by the current parser,
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
