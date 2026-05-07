# Game Analysis

Purpose: define replay analysis for Gomoku2D before forced-line ideas become bot
logic or UI.

The analyzer is product-first. It should explain finished games and replay
moments in a way a player can understand. Bot improvements can reuse the same
machinery later, but the first goal is not a stronger practice bot or a full
solver.

## Product Goal

Given a finished match, the analyzer should answer:

- When did the winner have a forced win?
- What was the losing side's last real chance to escape?
- Which move was the decisive attack?
- Which move was the critical mistake?
- Was the loss strategic, accidental, or unclear within the analyzer limits?

The replay surface should produce concrete, bounded explanations:

- "Move 43: point of no return."
- "White's last chance was move 42."
- "Black had a forced line from here."
- "This looks like a missed defense, not a strategically lost position."

Do not overclaim. If the bounded analyzer cannot prove the position, it should
say so.

## Core Distinction

Replay analysis needs to keep three lines separate:

- Actual line: the moves that were really played.
- Ideal line: one proof line showing how the winner could force the result.
- Analysis model: the bounded rules used to decide whether a proof exists.

The actual game can diverge from the ideal line because human players and bots
can miss tactics. The product should explain that divergence instead of hiding
it behind a single "best move" label.

## Proof Model

The ideal-game layer asks two related, bounded questions:

- Corridor proof: from this prefix, can the eventual winner stay inside a narrow
  modeled forced line to the actual ending?
- Corridor exit: at a losing-side reply point, is there any legal reply that
  exits that detected corridor?

For replay analysis, "force" is intentionally model-bounded. The first product
goal is to explain the detected forced corridor near the end of the actual game,
not to prove that every alternate state is a game-theoretic loss under perfect
play. An escape reply can leave the detected corridor even if the defender might
still lose later.

This is proof-oriented analysis, not generic best-move analysis or full solver
analysis.

For a move list `m1..mn`:

- `P_k` is the board after move `k`.
- `winner` is the actual game winner.
- `side_to_move(P_k)` determines whether the root is an attacker node or a
  defender node.
- `corridor_status(P_k, winner, model)` returns `forced_win`, `escape_found`,
  or `unknown`.

Avoid a plain `not_proven` result. In a bounded analyzer, "not found" does not
mean "defensible." The useful distinction is:

- `forced_win`: the analyzer proved that the detected corridor reaches a win
  under the stated model and limits.
- `escape_found`: the defender has at least one model-valid move that exits the
  detected forced corridor, or the corridor reaches a neutral state with no
  active immediate/imminent threat and no forcing continuation.
- `unknown`: the analyzer hit depth, node, time, or model-scope limits.

Every result must carry its model and limits. A proof without those fields is
not product-safe.

## Threat Sequence Model

The analyzer should reason backward from the final winning move by separating
local tactical misses from already-forced positions.

Terminology, using Freestyle rules first:

- Immediate threat: a `ClosedFour` or `BrokenFour`. It threatens a direct win
  next turn and normally creates a single direct defensive reply.
- Imminent threat: an `OpenThree` or `BrokenThree`. It does not win next turn
  yet, but it creates a bounded reply set: direct defensive replies plus valid
  defender counter-threats. A closed three is latent material, not an active
  corridor threat.
- Lethal threat: an `OpenFour`. It is effectively terminal for this proof layer
  because the defender cannot cover both winning squares unless they have an
  immediate counter-win.
- Winning square: a legal empty point where the attacker can move and win
  immediately.
- Cost square: a local point that would answer the current threat if the
  defender can legally play it. This is the proof-side name for a shape
  `defense_square` from [`tactical_shapes.md`](tactical_shapes.md).
- Corridor reply: a named move that keeps play inside the threat corridor. For
  the defender this means answering the active threat or creating a valid
  counter-threat. For the attacker this means answering a defender threat or
  materializing a new immediate/imminent threat.
- Escape reply: a legal defender move that exits the detected forced corridor.
  It does not prove the defender survives the rest of the game; it only proves
  the current threat corridor no longer forces the result.
- Forced reply: a defender corridor reply that answers the current threat but
  still leaves the attacker a forced continuation.

The corridor is semantic, not a fixed-width search cap. A threat corridor exists
when active immediate/imminent threats force play into named local responses. It
can be wider for an imminent threat than for an immediate threat, and it can
include defensive counterplay, but it is still bounded by tactical shape
semantics rather than by all legal board moves.

Corridor state transitions:

- Enter a corridor when either side creates an immediate or imminent threat.
- Stay locked in the corridor while each reply creates or answers another
  immediate/imminent threat.
- Exit the corridor when a side wins, or when all active immediate/imminent
  threats are neutralized and the attacker has no named forcing continuation.
- Return `unknown`, not `escape_found`, when a named corridor still exists but
  the analyzer cannot classify every corridor reply within its current limits.

The analyzer should not fall back to broad normal search. If the attacker has
latent closed threes but no active threat exists, the only attacker moves that
matter to corridor proof are moves that materialize a new immediate/imminent
threat. If the attacker instead plays a quiet non-forcing move while not
responding to an opponent threat, that is an accidental exit from the corridor in
the actual line, not evidence that the previous corridor still forces a win.

## Corridor-Only Replay Flow

The replay analyzer assumes a finished decisive game ends inside a threat
corridor. The final winning move provides the concrete terminal endpoint. The
goal is then not to solve the whole game, but to walk backward and find the
latest losing-side point where the final corridor could have been escaped.

The core loop is:

1. Start at the final winning move and identify the final threat corridor.
2. Walk backward along the actual replay through losing-side decision points.
3. At each losing-side turn, enumerate named corridor replies: direct defenses,
   immediate wins, and valid counter-threats.
4. For each losing-side alternative, follow only corridor continuations. The
   winning side may complete an immediate win, answer threats, or materialize
   new immediate/imminent threats, but it may not use broad quiet search to
   preserve a proof.
5. Stop when at least one losing-side reply exits the corridor. That reply marks
   the latest possible escape. If every named reply stays forced, continue
   walking backward. If a named corridor exists but cannot be classified, mark
   the boundary `unknown`.

This means the analyzer can usually skip winner-side decision points while
walking the actual spine: the question is what the losing side could have done
differently. In a normal alternating corridor, the backward scan advances by two
plies from one losing-side decision to the previous losing-side decision.

There is one guard: the actual winning-side move after each actual losing-side
reply must still be a valid corridor continuation. If the winner's actual move
does not complete an immediate win, answer an active defender threat, or
materialize a new immediate/imminent threat, the actual line accidentally exited
the corridor and any earlier forced interval ended there. Later play may enter
a new corridor, but it is no longer the same proof interval.

For losing-side alternatives, do not require the winner to repeat the actual
next move. The board has changed, so the winner may choose any named corridor
continuation from the alternate state. The restriction is not "actual move
only"; it is "corridor moves only."

Conceptually, this model should not need an arbitrary search depth limit. The
corridor itself bounds the search because every branch must be justified by an
active immediate/imminent threat. Implementation can still keep safety guards
for bugs, cycles in derived facts, or report runtime, but tripping such a guard
is `unknown`. A guard must never turn an active corridor into `escape`.

At the final winning move `PW`, inspect the previous prefix after `PW - 1`.
There is still at least one winning square open for the winner. The losing move
`PW - 1` falls into one of two broad cases:

- Single winning square at `PW - 2`: the loser could have blocked the only
  winning square and instead ignored it. This is an accidental/local missed
  defense unless the block itself was illegal, the loser had an immediate
  counter-win, or the block still entered another forced loss.
- Single winning square with no legal block: in Renju, White can create a
  single winning square whose natural Black block is forbidden. If Black has no
  immediate counter-win or other legal escape, that is already a forced terminal
  threat for Black, not an accidental missed block.
- Multiple winning squares at `PW - 2`: the loser can only cover one winning
  square, so the position is already forced unless the loser has a counter-win
  or another escape that changes the result first.

The forced cases are the strategically interesting ones. Walking backward one
round at a time, after a winning-side move `PX - 2`, ask:

> Which named corridor replies did the losing side have, and did any of them
> exit the active threat corridor?

This is deliberately narrower than "can the loser survive from here." The
analyzer only asks whether the current immediate/imminent threat corridor stays
alive.

For each corridor reply candidate, classify the follow-up explicitly:

- `forced`: the reply loses immediately, or the attacker can keep the game
  inside a named immediate/imminent corridor all the way to terminal win.
- `escape`: the reply neutralizes the active threats and no named attacker
  continuation can keep the corridor alive.
- `unknown`: a named corridor still exists, but at least one corridor reply or
  continuation cannot be classified within the model limits.

This distinction is critical for counter-threats. If the defender creates an
immediate/imminent counter-threat, the attacker may answer it. After that answer
the analyzer must inspect the new position:

- If the original attacker has a new active immediate/imminent threat and every
  named defender reply still loses, the counter-threat reply is `forced`.
- If the new position has an active immediate/imminent threat but some named
  reply is unproven, the counter-threat reply is `unknown`.
- If no active immediate/imminent threat remains and the attacker has no named
  forcing continuation, the counter-threat reply is `escape`.

Implementation budgets must not change those semantics. A reply set that is
"too broad for this implementation" is `unknown` if it is still generated from
named threat semantics. Only an actual neutralized corridor is `escape`.

If an `escape` reply exists, and it is the final escape before the forced
interval, it becomes the last chance. If the actual move did not choose one of
those replies, classify the transition with the same proof-status rules used
elsewhere:

- previous prefix `escape_found`, current prefix `forced_win`, transition move
  was by the losing side: `missed_defense`.
- previous prefix `escape_found`, current prefix `forced_win`, transition move
  was by the winning side: `strategic_loss`.
- previous prefix `unknown`, current prefix `forced_win`: `unclear`, even if the
  current forced sequence is proven.

If no `escape` reply exists and every named reply is `forced`, then `PX - 2` is
still inside the detected forced sequence. The defender may have legal cost
replies, but all model-visible replies stay inside the corridor rather than
escaping it. Keep walking backward.

If no `escape` reply exists but at least one named reply is `unknown`, stop at an
`unknown` boundary. Do not claim the earlier prefix is forced just because this
implementation failed to classify a corridor reply.

All winning squares, cost squares, and escape checks are rule-aware and
side-specific. A forbidden Renju point is not a legal winning square or escape
reply for Black, but the same point may still be a legal winning square for
White. If White can create a threat whose only natural Black answer is forbidden,
that single-square threat is terminal under the model.

This distinction is important for product copy:

- Accidental miss: "there was one block and it was missed."
- Forced sequence: "the block was forced, but the next threat was still
  unavoidable."
- Unknown: "the analyzer cannot determine whether the forced line started
  earlier."

## Corridor Search Semantics

The biggest design risk is pretending a tactical proof is stronger than it is.
Defender reply semantics must be explicit:

- Broad all-legal proof can remain a validation mode for tiny fixtures, but it
  is not the intended replay-analysis model.
- Corridor proof uses named threat semantics: immediate replies, imminent
  replies, immediate wins, and valid counter-threat replies.
- A fixed branch cap is an execution limit, not the definition of the corridor.
  If the named reply set exceeds a budget, the result is `unknown` unless the
  analyzer can still prove every named reply.

Search semantics:

- Attacker means the side we are proving a forced win for, even if that side is
  not currently to move.
- Attacker node: at least one named corridor move must lead to a win.
- Defender node: every named corridor reply must stay inside a forced corridor
  for the attacker.
- Escape: a named defender reply exits the detected forced corridor, or the
  attacker has no named corridor continuation after threats are neutralized.
- Unknown: a named corridor exists but cannot be fully classified inside model
  limits.
- Principal line: one representative forced line from the proof tree.

The reply set must be named and inspectable. At minimum, the implementation
should record whether the set includes immediate cost replies, imminent direct
defenses, defender immediate wins, counter-threats, and Renju-forbidden
cost-square handling. Leaving those out may be useful for a narrow experiment,
but it weakens the product claim.

Product copy must reflect the model. "Forced line" is acceptable when the report
also shows the model and limits. "Detected forced line" or "last known escape" is
safer than implying the loser had a proven long-term save.

## Renju Corridor Overlay

Renju should be modeled as a legality and threat-effect overlay on top of the
same corridor state machine, not as a separate proof model. The corridor still
enters on immediate/imminent threats, stays locked while named threats are
answered or materialized, and exits only when active threats are neutralized or a
side wins. What changes is whether a raw shape square is legal and effective for
the side that would play it.

The analyzer should carry raw and legal tactical facts separately:

- Raw threat square: a shape-derived gain, completion, or cost square before
  Renju legality is applied.
- Legal corridor square: a raw square that the side can legally play and that
  still has the expected tactical effect under Renju.
- Forbidden corridor square: a raw Black square rejected by Renju. This is proof
  evidence, not missing data.

Side-specific implications:

- Black attacker: a raw gain or completion only creates a corridor threat if it
  is legal for Black. Double-three, double-four, and overline can erase a raw
  freestyle threat before it enters the corridor.
- Black defender: forbidden cost squares are not valid replies. If every natural
  Black answer to a White threat is forbidden, the reply set is empty for rule
  reasons and the threat remains forced rather than unknown.
- White attacker: White can intentionally create threats whose natural Black
  replies are forbidden. The report should surface those forbidden costs because
  they explain why an apparently empty block square is unavailable.
- White defender: White reply generation is close to freestyle because White has
  no forbidden moves, but White counter-threats can still be strong specifically
  because they constrain Black into forbidden answer squares.

Renju legality must not be applied as a silent early filter that erases proof
evidence. Each corridor square should carry enough annotation for report and
debug output, for example `{ role, side, raw_square, legal, forbidden_reason }`.
The exact shape can change in implementation, but the invariant should hold:
rule-forbidden replies are visible as tactical facts and are excluded only when
deciding which replies are playable.

## Forced Extensions

The proof model is corridor-bounded rather than depth-bounded. The extension
path only follows named immediate/imminent threats and their named replies.
Conceptually, it tests whether a reply is a true corridor exit or merely a
forced reply. From there:

- attacker extension moves are limited to legal moves that answer a defender
  threat or create a new immediate/imminent threat,
- defender extension replies are limited to named replies to the active threat
  plus immediate wins or valid counter-threats,
- a defender reply that neutralizes all active threats and leaves no named
  attacker continuation is an escape,
- a defender reply that leaves an active corridor but cannot be classified by
  the current implementation is unknown,
- tripping an implementation guard inside an active corridor returns `unknown`.

This handles chained threats such as "closed four, forced block, create open
three, forced replies, create broken four" without pretending the analyzer has
searched every quiet alternative.
If the previous prefix was still `unknown`, the analyzer should keep the root
cause `unclear` even when the next prefix enters a proven forced interval.

The current lab CLI still exposes depth and forced-extension budgets because the
implementation is not yet a pure corridor prover. Treat those fields as safety
and diagnostic controls, not as the intended product proof semantics.

The current MVP detects winning squares through `immediate_winning_moves_for`.
It does not yet expose a first-class threat inventory with named shapes, cost
squares, or all defender reply classifications. That inventory is the next
design step before broader forced-line claims.

The next useful inventory is not a hybrid all-legal/tactical model. It is a
first-class corridor model: enumerate active immediate/imminent threats,
generate named replies and counter-threats, and classify whether the corridor
continues, exits, wins, or remains unknown. It is still model-bounded proof, not
a general TSS solver.

The current implementation is behind the desired corridor model in one known
place: `BrokenThree` may still be treated as diagnostic-only because its cost
and rest-square semantics are not yet clean enough. Design-wise, `BrokenThree`
belongs with imminent threats; implementation should promote it only when the
reply generator can name the relevant defensive replies and continuations
without falling back to broad search. Rest-square dependency graphs and
multi-threat combinations belong in later TSS-style work once the basic corridor
facts are validated on report samples.

## Backward Walk

For a finished game, walk backward from the final move and test prefixes. Do not
assume forced-win state is monotonic across the actual game.

A player can:

- create a forced win,
- miss the conversion and release it,
- regain a forced win later after another mistake.

The analyzer should therefore record proof intervals rather than only one point:

- `proof_start_ply`: first prefix in an interval where the winner has a proven
  forced win under the model.
- `proof_end_ply`: last prefix in the forced interval. The next prefix, if any,
  is an escape or unknown boundary.
- `unknown_gap`: a prefix inside the scanned range where the analyzer cannot
  prove either forced win or escape.
- `last_chance_ply`: last losing-side turn before a proof interval where an
  escape exists.
- `decisive_attack_ply`: winner move that creates or enters the proof interval.
- `critical_mistake_ply`: losing move that allows the decisive attack or misses
  the last escape.

Important labels:

- Final win: the actual ending move and winning line.
- Forced interval: a contiguous range of prefixes where the winner has a proof.
- Point of no return: the start of the final unreleased forced interval.
- Last chance: the final escape opportunity before that interval.
- Decisive attack: the winner's forcing move.
- Critical mistake: the losing move that made the attack possible or failed to
  escape it.

The decisive attack and critical mistake are related but not always the same
move. A strong attack may be the winner's achievement, while the root mistake
may be the losing side's previous move.

## Human Imperfection Layer

Real games, especially human games, are not ideal games. Analysis should
classify the actual line separately from the ideal proof.

Use a two-part classification:

- Root cause: the main reason the final forced interval exists.
- Tactical notes: local misses or conversion issues that happened along the
  actual line.

Root-cause categories:

- `strategic_loss`: a move changes the position from `escape_found` to
  `forced_win` under the same model and limits. This means the move entered the
  detected forced corridor; it does not claim the previous position was a
  game-theoretic draw or win for the loser.
- `missed_defense`: the losing side had at least one escape move, but the
  actual move did not play one.
- `missed_win`: a player had an immediate or forced win, but played elsewhere
  and allowed the game to continue.
- `unclear`: the bounded analyzer cannot prove enough to identify a root cause.

If the previous prefix is `unknown`, do not label the transition as a strategic
loss. The correct root cause is `unclear`, optionally with a tactical note that
the move entered a proven forced interval.

Tactical notes:

- `accidental_blunder`: the actual move allows a simpler or immediate tactic
  that was locally avoidable.
- `conversion_error`: the winning side had a forced win, played a move that
  released it, then later won after another mistake.
- `strong_attack`: the decisive move created a forcing line even though the
  previous position was not clearly lost.

This split avoids overlap. For example, one move can be a `missed_defense` root
cause and also carry an `accidental_blunder` note if it missed an obvious local
block.

"Accidental" and "strategic" are product labels, not judgments of player skill.
They mean "local tactical oversight" versus "the position became lost under the
analysis model."

The replay UI should prefer concrete language over blame:

- "Missed defense" is clearer than "bad move."
- "Last chance" is clearer than "mistake" when the proof is narrow.
- "Unclear" is better than pretending the analyzer solved the position.

## Output Shape

Initial analysis output can be a compact record:

```text
GameAnalysis
  schema_version
  rule_set
  winner
  loser
  final_move
  final_winning_line
  model
  limits
  final_forced_interval
  last_chance_ply
  decisive_attack_ply
  critical_mistake_ply
  root_cause
  tactical_notes
  principal_line
  unclear_context
  proof_summary
```

To explain forced sequences, summaries also need branch evidence. The compact UI
can hide this at first, but the lab report should preserve it:

```text
ThreatSequenceEvidence
  prefix_ply
  attacker
  defender
  winning_squares
  raw_cost_squares
  legal_cost_squares
  illegal_cost_squares
  defender_immediate_wins
  actual_reply
  reply_classification
  escape_replies
  forced_replies
  next_forcing_move
  proof_status
  limit_hit
```

`reply_classification` should be one of:

- `ignored_single_win`: one winning square existed and the actual reply did not
  answer it.
- `blocked_but_forced`: the reply answered the current threat but all modeled
  continuations stay inside the detected forced corridor.
- `escaped`: the reply wins immediately, breaks the threat, or avoids the next
  forced continuation.
- `no_legal_block`: the only apparent cost squares are illegal for the defender
  and no immediate counter-win exists, so the threat remains forced even with a
  single winning square.
- `unknown`: the analyzer cannot classify the reply within the current model.

`prefix_ply` and `actual_reply` are replay-context attribution fields. The lab
proof fills them while a proof branch is still following the actual replay line.
Once a proof branch diverges into a virtual continuation, nested evidence leaves
these fields empty instead of pretending it maps to a real move.

Proof result records should use explicit status:

- `forced_win`: detected corridor proven within the model and limits.
- `escape_found`: defender has at least one model-valid move that exits the
  detected forced corridor.
- `unknown`: search was cut off or the position exceeded analyzer scope.

Unknown proof results should also carry named causes. Current lab causes are:

- `depth_cutoff`: normal proof depth ran out.
- `forced_extension_cutoff`: narrow forced-extension budget ran out.
- `attacker_child_unknown`: at least one attacker child could not be resolved.
- `defender_reply_unknown`: at least one defender reply could not be resolved.
- `model_scope_unknown`: the selected proof model had no concrete reply or
  forcing set for the position.
- `outside_scan_window`: the previous prefix was not part of the scanned range.

For `unclear` results, reports should preserve enough context to drive the next
debugging pass without rerunning the whole tournament:

```text
UnclearContext
  reason
  previous_prefix_ply
  final_forced_interval
  previous_proof_status
  previous_proof_limit_hit
  previous_limit_causes
  previous_side_to_move
  winner
  principal_line_notation
  scan_start_ply
  scan_end_ply
  snapshots
```

Snapshots are compact board rows at the previous prefix and the final forced
interval start. They are lab-debug evidence, not a player-facing explanation.

`model` should include at least:

- defense policy: `all_legal_defense`, `tactical_defense`, or
  `hybrid_defense`.
- tactical reply coverage: legal cost replies only, legal cost replies plus
  defender immediate wins, counter-threats, forbidden-cost handling, or another
  named set.
- attacker move policy: all legal moves, local forcing moves, or another named
  candidate source.
- rule set: freestyle or Renju.
- limits: depth, nodes, time, maximum proof branches, and maximum backward
  window.

The proof tree can be stored separately from the summary so replay UI can show a
simple explanation first and expand into branch details later.

## Fixture Requirements

The first analyzer fixtures should include more than happy-path wins:

- Immediate final win with one obvious last block.
- Single-winning-square miss: loser ignores the only block and loses
  immediately.
- Multiple-winning-square terminal: loser blocks one winning square but another
  winning square remains.
- Renju single-square terminal: White has one winning square, Black has no
  immediate counter-win, and Black's only block is forbidden.
- Short closed-four forced line.
- Open-four line where one block is insufficient.
- Tactical-defense proof where only legal cost replies are considered.
- Tactical-defense failure where a counter-win or counter-threat escapes the
  legal local cost-reply set.
- Escape reply: defender has a move that both answers the current threat and
  prevents the next forced continuation.
- Forced reply: defender answers the current threat, but every legal cost reply
  still loses to the next forcing move.
- All-legal-defense proof for a tiny position where exhaustive defense is cheap.
- Conversion error: winner has a forced line, releases it, then wins later.
- Missed defense: loser has an escape but plays elsewhere.
- Missed win: player ignores an immediate or forced win.
- Forced chain: defender blocks one immediate threat, attacker creates the next
  immediate threat, and the analyzer proves the continuation without widening
  the whole search.
- Unknown: position exceeds proof limits and must not be labeled strategic.
- Unknown gap: an earlier forced interval cannot be connected safely to the
  final forced interval.
- Renju legality edge: forbidden Black defense squares never count as escapes.

These fixtures should print exact boards, expected labels, proof model, and
limits. They should fail if an implementation silently upgrades `unknown` into
`strategic_loss`.

## UI Direction

First useful surface: replay.

Possible replay annotations:

- Mark the final winning line using the existing result-screen treatment.
- Mark the decisive attack move.
- Mark the losing side's last chance.
- Show the principal forced line as a branch preview.
- Let the user step from the actual replay into the ideal continuation.

Keep this separate from general hints. A replay analyzer explains what happened;
it should not constantly interrupt live play.

## Engine Boundary

The analyzer should start in the Rust lab where reports and CLI inspection are
cheap. Once behavior is stable, expose a compact version through core/wasm for
the web replay UI.

Likely layering:

- `gomoku-core`: board, rules, legality, winning-line checks, compact move
  codecs, and any generic line/shape facts that are not bot-specific.
- `gomoku-eval` or a new lab analysis module: bounded forced-line search,
  proof-tree generation, and analyzer reports while the behavior is
  experimental.
- `gomoku-wasm`: stable summary API for web replay after the model is validated.
- `gomoku-web`: presentation only.

Avoid coupling product analysis directly to `SearchBot` internals. SearchBot can
reuse proven analysis facts later, but replay analysis should not inherit
tournament-only knobs or bot-specific evaluation shortcuts by accident.

Do not let the Phaser scene or React route own analysis rules.

## Non-Goals

- Full Gomoku/Renju solving.
- Universal best-move recommendation for every position.
- Ranking player skill.
- Public shareable analysis as part of the first slice.
- Running expensive analysis automatically on every local move.
- Treating bot tournament strength as the only success metric.

## Implementation Slices

1. Lock terminology, proof statuses, model bounds, and output shape in this doc.
2. Add finished-game prefix fixtures covering immediate wins, short forced
   lines, conversion errors, missed defenses, missed wins, unknown results, and
   Renju legality edges.
3. Build a CLI/lab analyzer that finds final win, proof intervals, last chance,
   and a bounded principal line for simple finished games.
4. Add proof-tree output and report rendering for debugging.
5. Add batch replay analysis for tournament replay directories.
6. Use analyzer summaries to annotate replay.
7. Feed proven, cheap forced-line facts back into bot ordering or narrow search
   only after the analyzer behavior is trustworthy.

## Lab MVP

The first lab implementation lives in `gomoku-eval` and is intentionally narrow:

- `gomoku_eval::analysis` defines the model/result types and bounded proof
  walker.
- `gomoku-eval analyze-replay --input <replay.json>` emits JSON analysis.
- `gomoku-eval analyze-replay-batch --replay-dir <dir>` analyzes a replay
  directory and emits grouped JSON/HTML reports for tournament smoke runs.
- `gomoku-eval analyze-report-replays --report <report.json>` samples compact
  tournament-report matches, reconstructs temporary replay objects in memory,
  and analyzes them without first writing replay JSON files.
- `gomoku-eval analysis-fixtures` runs curated replay fixtures and prints
  expected-vs-actual labels for the current analysis model.
- The current replay analyzer uses corridor-exit semantics for proof summaries:
  attacker nodes follow actual corridor moves or immediate wins, while defender
  nodes classify only model-valid corridor exits before calling a prefix forced.
  The older broad `prove_forced_win` path remains as a reference/validation
  helper for small positions.
- The current proof engine handles immediate wins, single-threat escapes,
  open-four style unavoidable immediate wins, one narrow forced-chain extension,
  defender immediate-win escapes, Renju forbidden-block terminals, proof
  intervals, conversion notes, missed defenses, missed wins, ongoing/draw
  summaries, and explicit `unknown` states.
- The fixture report currently covers missed defense, delayed conversion,
  losing-side missed win, shallow open-four corridor detection, closed-four to
  open-four forced-chain continuation, defender counter-win escape, Renju
  no-legal-block terminal behavior, and ongoing replay behavior.
- Tactical-defense mode exposes legal cost replies, defender immediate wins,
  and forbidden cost squares in branch evidence, but it is still not a full
  threat-space search.
- Batch analysis reports include `unclear_context` and limit-cause counts
  for unresolved entries: previous prefix status, proof-limit flag, named limit
  causes, principal-line notation, and compact board snapshots. This is meant
  to make proof-limit, model-scope, and scan-window failures inspectable before
  adding more search.
- Batch analysis reports can opt into `--include-proof-details` for decisive
  replay audits. This adds previous-prefix and final-forced-start proof
  snapshots, reply classification, principal-line notation, compact board
  snapshots, and visual decision frames. The visual frames render pre-move
  decision states backward from the winning ply through the final forced
  interval, then include a `before forced run / after ply N` boundary frame.
  They mark the side to move, the actual replay move for each ply, immediate
  win-now squares, opponent-win-next losing squares, and defender reply
  outcomes for the audited position. Defender replies use two visual layers:
  the outer hint explains why the square is shown: bright green for an
  immediate win, bright red for an immediate losing square, pink for a defensive
  reply to an imminent threat, blue for an offensive counter-threat reply, and
  an actual-move ring for the replay move. The marker character explains what
  happens if the defender plays it (`L` forced loss, `E` escape, `!` immediate
  loss, `?` unknown). Proof branch evidence such as
  aggregate cost squares, forbidden costs, and principal-line moves stays in the
  textual proof snapshots with explicit attacker/side-to-move labels, so the
  board does not imply nested branch moves are current gameplay hints. Keep it
  off for normal smoke runs; turn it on when reviewing why a `strategic_loss`,
  `missed_defense`, or decisive `unclear` label was assigned.
- After the corridor-exit pivot, a top-two report smoke run against
  `search-d7+tactical-cap-8+pattern-eval` vs
  `search-d5+tactical-cap-8+pattern-eval` passed with `8 analyzed / 8 total`
  and `0 failed` in about `3.9s` total elapsed time. It classified the decisive
  sample as `6` strategic losses and `1` missed defense, with `1` draw/ongoing
  entry and no decisive-game `unclear` roots.
- A follow-up 64-game top-two implementation snapshot passed with `64 analyzed /
  64 total` and `0 failed` in about `49s` total elapsed time. It classified the
  decisive sample as `54` strategic losses, `5` missed defenses, and `4`
  unclear proof-limit entries, with `1` draw/ongoing entry. All decisive games
  found a final forced interval; only the `4` unclear decisive games carried
  limit causes (`forced_extension_cutoff`, `attacker_child_unknown`, and
  `defender_reply_unknown`). Treat this as current telemetry, not validation of
  the corrected corridor-only model or product-ready copy: `strategic_loss`
  still means "entered the detected forced corridor" rather than a full
  game-theoretic loss proof.
- The same implementation snapshot with `--include-proof-details` produced
  proof details for all `63` decisive entries and skipped the single
  draw/ongoing entry. This is the current audit path for checking whether the
  reported root transition, actual forced interval, and board prefixes are
  plausible before changing the proof model. The visual decision frames now also
  show local defender reply outcomes. For the first top-two sample, the ply-14
  frame marks `G4`, `G7`, and `G9` as imminent-defense replies that all end in
  forced loss, with `G7` additionally marked as the actual replay move. It also
  marks offensive counter-threat replies separately. `I11` still loses because
  Black answers at `I10` and re-enters the narrow forced line. `I10` should also
  be a forced loss: after Black answers at `I11`, White is still locked in the
  named `G4/G7/G9` corridor, and those replies should be provable as forced
  losses.
  If the implementation reports `I10` as `escape` or `unknown`, that is an
  implementation gap in corridor proof, not the intended model. This is the
  intended bridge between "the user sees multiple plausible choices" and "which
  of those choices are proven escapes, forced losses, or still outside the
  model."
- Before the corridor-exit pivot, the 64-game sampled checkpoint passed with
  `64 analyzed / 64 total`
  and `0 failed`: `63` proof-limit hits and `1` draw/ongoing game. Bounded
  scan expansion removed the previous scan-window cutoffs. Under
  `all_legal_defense`, `61` decisive games hit `depth_cutoff` plus
  `attacker_child_unknown` and `defender_reply_unknown`; the other `2` hit
  `forced_extension_cutoff` plus the same child/reply unknowns.
- A tactical-defense 64-game comparison also produced `63` proof-limit hits and
  `1` draw/ongoing game. It was faster, but `61` decisive games became
  `model_scope_unknown`, meaning the narrow tactical model did not have a
  concrete reply/forcing set for the previous prefix. This is useful evidence
  for future model design, not a product-safe proof.
- A hybrid-defense local-threat smoke run against the same top-two matchup,
  sampled at `8` games with depth `2`, forced extensions `4`, and backward
  window `8`, found `2` missed defenses, `5` unclear proof-limit entries, and
  `1` draw/ongoing game. The useful signal is that bounded local-threat replies
  can resolve some real report samples, not only synthetic fixtures. The risk is
  runtime: the slowest sampled entry took about `55s`, and summed per-entry time
  was about `95s`. The next analyzer slice should add tighter proof budgets,
  memoization, or better activation telemetry before widening shape coverage.
- A stricter double-threat-only trigger was fast but did not improve the sampled
  report, while a broader one-or-two-threat trigger improved coverage but became
  expensive. Keep both facts in mind before treating local-threat replies as a
  default product model.
- Adding `BrokenFour` facts and diagnostic `BrokenThree` facts did not change
  the same 8-game smoke result: still `2` missed defenses, `5` unclear
  proof-limit entries, and `1` draw/ongoing game, with roughly the same runtime.
  Temporarily treating `BrokenThree` as forcing was much slower and was narrowed
  back to diagnostic-only before checkpointing.
- Raising all-legal depth is not the next practical move. The 8-game smoke at
  `all_legal_defense`, depth `3`, forced extensions `4` still left `7`
  unresolved entries and took roughly `190s` wall-clock / `626s` summed
  per-entry time, versus about `2.4s` / `7.4s` for depth `2`.
- Increasing forced extensions alone did not help the smoke matrix. The dominant
  issue is normal proof depth plus defender breadth, not forced-extension
  budget.

Current next target: inspect the visual decision-frame audit output for the
top-two 64-game run, especially the `4` decisive `unclear` entries and any
surprising `strategic_loss` labels. Use remaining failures or suspicious labels
to decide whether the next slice should improve named local exits, forced-chain
evidence, or report readability. Do not expose replay analysis in the web UI
yet, and do not try to solve remaining unknowns by simply raising all-legal
depth or widening shape coverage.

Example:

```bash
cargo run -p gomoku-eval -- analyze-replay \
  --input outputs/replays/match_001.json \
  --output outputs/analysis_001.json \
  --defense-policy all-legal-defense \
  --max-depth 2 \
  --max-forced-extensions 4

cargo run -p gomoku-eval -- analysis-fixtures \
  --report-json outputs/analysis_fixtures.json \
  --report-html outputs/analysis_fixtures.html \
  --defense-policy all-legal-defense \
  --max-depth 2 \
  --max-forced-extensions 4

cargo run -p gomoku-eval -- analyze-replay-batch \
  --replay-dir outputs/replays \
  --report-json outputs/analysis_batch.json \
  --report-html outputs/analysis_batch.html \
  --defense-policy all-legal-defense \
  --max-depth 2 \
  --max-forced-extensions 4 \
  --max-backward-window 24

cargo run --release -p gomoku-eval -- analyze-report-replays \
  --report reports/latest.json \
  --entrant-a search-d7+tactical-cap-8+pattern-eval \
  --entrant-b search-d5+tactical-cap-8+pattern-eval \
  --sample-size 8 \
  --report-json outputs/analysis/top2_smoke.json \
  --report-html outputs/analysis/top2_smoke.html \
  --defense-policy all-legal-defense \
  --max-depth 2 \
  --max-forced-extensions 4 \
  --max-backward-window 8
```

Use the report-sampled 8-game smoke path while tuning analyzer output or proof
logic. It covers both entrants, color assignments where available, draws or
max-move games, and short/long games deterministically. Run a full 64-game
head-to-head analysis only for checkpoint reports. `--max-backward-window 8`
is the practical default for iteration; `24` is reserved for focused samples or
longer runs until the proof model becomes narrower. Add
`--include-proof-details` when the goal is auditability rather than a compact
summary report.

Keep generated analysis JSON/HTML under `gomoku-bot-lab/outputs/analysis/`
while iterating. These files are ignored scratch artifacts; commit only the
analyzer code, docs, and any deliberately curated reports.

This is still a lab artifact. Do not expose it in the web replay UI until the
fixture set and report output make the limits obvious enough for players.
