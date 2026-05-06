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

## Reply-Set Bounds

The biggest design risk is pretending a tactical proof is stronger than it is.
Defender reply semantics must be explicit:

- `all_legal_defense`: the defender may choose any legal move. This is the
  strongest proof style, but it is much more expensive.
- `tactical_defense`: the defender only replies with local cost/escape squares
  derived from the current forcing shape. This is useful for forced-chain
  exploration, but it is a model-bounded proof.
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
the implementation should record whether the set includes only local cost
squares or also defender immediate wins, counter-threats, and Renju-forbidden
escape handling. Leaving those out may be useful for a narrow experiment, but it
weakens the product claim.

Product copy must reflect the model. "Forced line" is acceptable for
`all_legal_defense`; "forced line within tactical replies" is safer for
`tactical_defense` until the behavior is validated.

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
- `proof_end_ply`: first later prefix where an escape is found, or the final
  move if it stays forced.
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

Proof result records should use explicit status:

- `forced_win`: proven within the model and limits.
- `escape_found`: defender has at least one model-valid escape.
- `unknown`: search was cut off or the position exceeded analyzer scope.

`model` should include at least:

- defense policy: `all_legal_defense`, `tactical_defense`, or
  `hybrid_defense`.
- tactical reply coverage: cost squares only, cost squares plus immediate wins,
  counter-threats, forbidden escapes, or another named set.
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
- Short closed-four forced line.
- Open-four line where one block is insufficient.
- Tactical-defense proof where only cost squares are considered.
- Tactical-defense failure where a counter-win or counter-threat escapes the
  local cost-square reply set.
- All-legal-defense proof for a tiny position where exhaustive defense is cheap.
- Conversion error: winner has a forced line, releases it, then wins later.
- Missed defense: loser has an escape but plays elsewhere.
- Missed win: player ignores an immediate or forced win.
- Unknown: position exceeds proof limits and must not be labeled strategic.
- Unknown gap: an earlier forced interval cannot be connected safely to the
  final forced interval.
- Renju legality edge: a natural Black defense is forbidden.

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
