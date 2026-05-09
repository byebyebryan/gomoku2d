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

`0.4.3` should test corridor search inside bot logic before the product exposes
new bot settings. The integration should stay focused:

- prefer moves that enter promising forcing corridors,
- extend search through narrow forced corridors,
- order defensive replies that escape known corridors,
- avoid illegal or fake Renju threats before they distort evaluation,
- expose offensive/defensive style as budget allocation over forcing and
  prevention, not as arbitrary eval weights.

The goal is not to bolt a full threat-space solver onto the current bot. The
goal is a practical forcing layer that improves strength, explanation, and
player education together.

### Corridor As Search Shortcut

The preferred live-search model is corridor search as a selective shortcut
inside alpha-beta, not as broad leaf evaluation.

Think of a narrow corridor as a portal in the search space. Normal alpha-beta
still generates, orders, and searches candidate moves. After a child move is
applied, the bot can ask a cheap local question: did this move enter a forcing
corridor? If not, search continues normally with one ply of depth spent. If yes,
and the reply set is narrow, the bot can follow the corridor without spending
the usual depth budget. The corridor either reaches a terminal win/loss or exits
into an unclear position where normal search resumes.

That makes corridor search a selective extension:

- enter only after a concrete immediate or imminent threat is detected,
- follow only while the branch remains tactically narrow,
- return terminal score if the corridor proves a win or loss,
- resume normal alpha-beta from the exit board if the corridor neutralizes,
- fall back to normal search when the corridor is too wide to justify special
  handling.

Initial lab tuning should use corridor reply width as the primary guard. A cap
of `3` is the first target because it matches the expected maximum branching
factor for a local imminent threat such as a broken or half-open three. A
separate maximum corridor ply limit is still useful as a safety guard, but it
should not be the main cost-control lever. Width determines whether the branch
is a corridor; depth only prevents pathological cases from running forever.

This is intentionally different from the retired `+corridor-q` leaf quiescence.
Leaf quiescence asked too often and too late: many depth-0 positions needed a
corridor probe only to fall back to static eval. The shortcut model spends
corridor work only after move generation has produced a candidate that appears
to enter or continue a forcing line.

The lab report should make that cost shape visible. At minimum, selective
extension needs metrics for entries seen, entries accepted, entries rejected by
width, corridor plies followed, terminal exits, width exits, neutral exits,
safety-guard exits, resumed normal-search states, and effective extra ply gained.
Those metrics should stay separate from ordinary alpha-beta nodes so a corridor
candidate cannot look cheaper by moving work into an unreported bucket.

This work may also reinforce corridor search itself. Bot integration will put
more pressure on proof cost, transition enumeration, memoization, and Renju
legality filtering than replay reports alone. Those optimizations belong in the
lab when they make corridor-aware behavior cheaper or clearer without changing
the model's honesty about `possible_escape` and `unknown`.

Longer term, the cheap local question should probably be backed by a rolling
frontier model for threat facts. Full-board threat scans are acceptable for the
current analyzer and early lab prototypes, but they are the wrong shape for a
hot search path. The frontier should track threat facts affected by applied and
undone moves, then answer queries such as "did the last move enter a corridor?"
and "what are the current defender replies?" without rescanning the whole board.
That optimization should follow the shortcut interface, not precede it, so the
cache is built around the queries the bot actually needs.

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
- retired `SearchBot` corridor quiescence evidence from the former
  `+corridor-q` suffix,
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
- the retired `+corridor-q` leaf integration was too expensive for live play and
  is no longer a lab suffix.
- The next corridor-bot direction is selective extension through narrow
  corridors, with initial reply width cap `3`, not broader proof at every leaf.

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
