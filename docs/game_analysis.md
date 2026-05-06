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

The ideal-game layer asks one narrow question from a board prefix:

> Can the eventual winner force a win from this position?

This is proof-oriented analysis, not generic best-move analysis.

For a move list `m1..mn`:

- `P_k` is the board after move `k`.
- `winner` is the actual game winner.
- `side_to_move(P_k)` determines whether the root is an attacker node or a
  defender node.
- `forced_win(P_k, winner, model)` returns `forced_win`, `escape_found`, or
  `unknown`.

Avoid a plain `not_proven` result. In a bounded analyzer, "not found" does not
mean "defensible." The useful distinction is:

- `forced_win`: the analyzer proved a win under the stated model and limits.
- `escape_found`: the defender has at least one model-valid escape.
- `unknown`: the analyzer hit depth, node, time, or model-scope limits.

Every result must carry its model and limits. A proof without those fields is
not product-safe.

## Threat Sequence Model

The analyzer should reason backward from the final winning move by separating
local tactical misses from already-forced positions.

Terminology:

- Winning square: a legal empty point where the attacker can move and win
  immediately.
- Cost square: a local point that would answer the current threat if the
  defender can legally play it. This is the proof-side name for a shape
  `defense_square` from [`tactical_shapes.md`](tactical_shapes.md). Cost squares
  must be split into legal cost replies and illegal cost squares before they are
  used in a proof.
- Escape reply: a legal defender move that either wins immediately for the
  defender, or answers the current threat and avoids entering another proven
  forced win.
- Forced reply: a defender move that answers the current threat but still leaves
  the attacker a forced continuation.

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

> Is there any legal losing-side reply `PX - 1` that either wins immediately for
> the loser, or prevents the next immediate win `PX` and avoids another forced
> win?

If such a reply exists, it is an escape reply. If it is the final escape before
the forced interval, it becomes the last chance. If the actual move did not
choose one of those replies, classify the transition with the same proof-status
rules used elsewhere:

- previous prefix `escape_found`, current prefix `forced_win`, transition move
  was by the losing side: `missed_defense`.
- previous prefix `escape_found`, current prefix `forced_win`, transition move
  was by the winning side: `strategic_loss`.
- previous prefix `unknown`, current prefix `forced_win`: `unclear`, even if the
  current forced sequence is proven.

If no escape reply exists, then `PX - 2` is still inside the forced sequence.
The defender may have legal cost replies, but all of them are forced replies
rather than escapes. Keep walking backward until the analyzer finds an escape
reply or hits an `unknown` boundary.

All winning squares, cost squares, and escape checks are rule-aware and
side-specific. A forbidden Renju point is not a legal winning square or escape
reply for Black, but the same point may still be a legal winning square for
White. If White can create a threat whose only natural Black answer is forbidden,
that single-square threat is terminal under the model.

This distinction is important for product copy:

- Accidental miss: "there was one block and it was missed."
- Forced sequence: "the block was forced, but the next threat was still
  unavoidable."
- Unknown: "the analyzer cannot prove whether an escape existed earlier."

## Reply-Set Bounds

The biggest design risk is pretending a tactical proof is stronger than it is.
Defender reply semantics must be explicit:

- `all_legal_defense`: the defender may choose any legal move. This is the
  strongest proof style, but it is much more expensive.
- `tactical_defense`: the defender only replies with legal local cost replies,
  defender immediate wins, and other explicitly named tactical escapes derived
  from the current forcing shape. This is useful for forced-chain exploration,
  but it is a model-bounded proof.
- `hybrid_defense`: use tactical replies when a concrete forcing shape defines
  them, and fall back to a wider legal set when no concrete reply set exists.

Search semantics:

- Attacker means the side we are proving a forced win for, even if that side is
  not currently to move.
- Attacker node: at least one legal forcing move must lead to a win.
- Defender node: every move in the selected reply set must still lose.
- Escape: a defender move in the selected reply set that breaks the proof.
- Principal line: one representative forced line from the proof tree.

For `tactical_defense`, the reply set must be named and inspectable. At minimum,
the implementation should record whether the set includes only legal local cost
replies or also defender immediate wins, counter-threats, and Renju-forbidden
cost-square handling. Leaving those out may be useful for a narrow experiment,
but it weakens the product claim.

Product copy must reflect the model. "Forced line" is acceptable for
`all_legal_defense`; "forced line within tactical replies" is safer for
`tactical_defense` until the behavior is validated.

## Forced Extensions

The lab analyzer now has a narrow forced-extension budget in addition to normal
proof depth. This is not general extra depth. The extension path is only entered
after a defender answers an immediate attacker threat and clears the direct
winning move. Conceptually, it tests whether that cost-square reply is a true
escape or merely a forced reply. From there:

- attacker extension moves are limited to legal moves that create a new
  immediate winning threat,
- defender extension replies are limited to legal attacker cost replies plus
  defender immediate wins,
- a defender reply that breaks the forced continuation remains an escape,
- exhausting the extension budget returns `unknown`.

This handles simple chained threats such as "closed four, forced block, create
open four" without pretending the analyzer has searched every quiet alternative.
If the previous prefix was still `unknown`, the analyzer should keep the root
cause `unclear` even when the next prefix enters a proven forced interval.

The current MVP detects winning squares through `immediate_winning_moves_for`.
It does not yet expose a first-class threat inventory with named shapes, cost
squares, or all defender reply classifications. That inventory is the next
design step before broader forced-line claims.

One Renju-specific implementation trap: a tactical reply helper may derive raw
attacker cost squares, intersect them with defender-legal moves, and end up with
no defender reply because the only natural Black block is forbidden. That case
must be classified as `no_legal_block` / forced terminal, not silently downgraded
to `unknown`.

The next slice should stay at the immediate-winning-square / four-level threat
layer: `Five`, `OpenFour`, `ClosedFour`, and `BrokenFour`. Do not expand this
slice into open-three rest-square dependency search yet. Open-three and weaker
shape chains require richer TSS-style dependency handling and should wait until
the four-level model is explicit and fixture-backed.

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
  `forced_win` under the same model and limits.
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
  continuations still lose.
- `escaped`: the reply wins immediately, breaks the threat, or avoids the next
  forced continuation.
- `no_legal_block`: the only apparent cost squares are illegal for the defender
  and no immediate counter-win exists, so the threat remains forced even with a
  single winning square.
- `unknown`: the analyzer cannot classify the reply within the current model.

Proof result records should use explicit status:

- `forced_win`: proven within the model and limits.
- `escape_found`: defender has at least one model-valid escape.
- `unknown`: search was cut off or the position exceeded analyzer scope.

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
5. Use analyzer summaries to annotate replay.
6. Feed proven, cheap forced-line facts back into bot ordering or narrow search
   only after the analyzer behavior is trustworthy.

## Lab MVP

The first lab implementation lives in `gomoku-eval` and is intentionally narrow:

- `gomoku_eval::analysis` defines the model/result types and bounded proof
  walker.
- `gomoku-eval analyze-replay --input <replay.json>` emits JSON analysis.
- `gomoku-eval analysis-fixtures` runs curated replay fixtures and prints
  expected-vs-actual labels for the current analysis model.
- The current proof engine handles immediate wins, single-threat escapes,
  open-four style unavoidable immediate wins, one narrow forced-chain extension,
  proof intervals, conversion notes, missed defenses, missed wins, ongoing/draw
  summaries, and explicit `unknown` states.
- The fixture report currently covers missed defense, delayed conversion,
  losing-side missed win, shallow-model unknown guard, closed-four to open-four
  forced-chain continuation, and ongoing replay behavior.
- Tactical-defense mode is present as a model flag and immediate-threat reply
  subset, but it is not yet a full threat-space search.

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
```

This is still a lab artifact. Do not expose it in the web replay UI until the
fixture set and report output make the limits obvious enough for players.
