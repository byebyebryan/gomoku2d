# Game Analysis

Purpose: define how Gomoku2D applies corridor search to finished-game replay
analysis.

The broader corridor-search thesis, vocabulary, and bot/product implications
live in [`corridor_search.md`](corridor_search.md). This document is the replay
analyzer contract: how the model is applied to saved games, what the report
emits, and where the current implementation still has limits.

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
- "Black had a forced corridor from here."
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
  modeled corridor to the actual ending?
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

- `forced_win`: the analyzer proved that the detected corridor reaches a win
  under the stated model and limits.
- `escape_found`: the defender has at least one model-valid move that exits the
  detected forced corridor, or the corridor reaches a neutral state with no
  active immediate/imminent threat and no forcing continuation.
- `unknown`: the analyzer hit a model or replay guard before it could enumerate
  a concrete legal defender alternative.

Once the analyzer has a concrete legal defender reply, failure to prove that
reply is still forced should count as an escape from the detected corridor. Keep
the evidence honest by marking that branch as a `possible_escape`, not as a
`confirmed_escape`. This matches the product question: if the bounded
model cannot prove a position is still a forced loss for the losing side, it is
not an obvious forced loss worth presenting as decisive.

Every result must carry its model and limits. A proof without those fields is
not product-safe.

## Model Reference

Replay analysis uses the corridor-search model defined in
[`corridor_search.md`](corridor_search.md):

- threat categories: immediate, imminent, and lethal threats
- reply categories: corridor replies, forced replies, confirmed escapes, and
  possible escapes
- state transitions: enter, stay inside, win, or exit the corridor
- Renju overlay: raw tactical squares, legal corridor squares, and forbidden
  Black replies
- model limits: depth/guard cutoffs must not become proof of a forced win

This document only adds the replay-specific contract: start from a finished
game, walk the actual ending backward, and explain the final detected corridor
without claiming to solve every alternate future.

## Corridor-Only Replay Flow

The replay analyzer assumes a finished decisive game ends inside a threat
corridor. The final winning move provides the terminal endpoint. The goal is not
to solve the whole game; it is to walk backward and find the latest losing-side
decision that could have escaped the final detected corridor.

The core loop is:

1. Start at the final winning move and identify the final threat corridor.
2. Walk backward along the actual replay through losing-side decision points.
3. At each losing-side turn, enumerate named corridor replies: direct defenses,
   immediate wins, and valid counter-threats.
4. For each alternative, follow only corridor continuations. The winner may
   complete an immediate win, answer a threat, or materialize a new
   immediate/imminent threat; it may not use broad quiet search to preserve a
   proof.
5. Stop when at least one losing-side reply exits the corridor. That reply is
   the latest escape. If every named reply stays forced, keep walking backward.
   If a legal reply cannot be classified within the model budget, stop as a
   `possible_escape`.

This means the analyzer can usually skip winner-side decision points while
walking the actual spine. The question is what the losing side could have done
differently, not whether the winner had every possible quiet improvement.

There is one guard: the actual winning-side move after each actual losing-side
reply must still be a valid corridor continuation. If the winner's actual move
does not complete an immediate win, answer an active defender threat, or
materialize a new immediate/imminent threat, the actual line accidentally exited
the corridor. Later play may enter a new corridor, but it is no longer the same
proof interval.

When the latest escape boundary is before the active corridor has started, there
may be no immediate/imminent threat to mark yet. In that case the report should
mark the winner's next actual corridor-entry square as the losing side's escape
target. This means "deny this shown corridor," not "prove this move saves the
whole game."

For each corridor reply candidate, classify the follow-up explicitly:

- `forced`: the reply loses immediately, or the attacker can keep the game
  inside a named immediate/imminent corridor all the way to terminal win.
- `confirmed_escape`: the reply neutralizes the active threats and no named attacker
  continuation can keep the corridor alive.
- `possible_escape`: a legal reply is visible, but the analyzer cannot prove that
  the reply remains forced within the current model limits. This counts as an
  escape for root classification, but the report must show the proof limit.
- `unknown`: the model cannot enumerate a meaningful legal reply or hit a
  structural guard before a concrete alternative exists.

Transition labels follow the same proof-status rules everywhere:

- `escape_found -> forced_win` caused by the losing side: `missed_defense`.
- `escape_found -> forced_win` caused by the winning side: `strategic_loss`.
- `unknown -> forced_win`: `unclear`, even if the current forced sequence is
  proven.

## Replay Model Settings

The active replay analyzer exposes one corridor model:

- reply policy: `corridor_replies`
- proof budget: `--max-depth`, interpreted as corridor depth rather than broad
  minimax depth
- scan budget: `--max-scan-plies`, interpreted as how far backward through the
  finished replay the analyzer may look for the final corridor boundary

The report must keep those model settings visible. Product copy such as "forced
corridor" is acceptable only when paired with the model and limits. "Detected
forced corridor" or "last known escape" is safer when summarizing unresolved
branches.

Implementation-specific replay semantics:

- Principal line: one representative corridor continuation from the proof tree.
- If the previous prefix was still `unknown`, keep the root cause `unclear` even
  when the next prefix enters a proven forced interval.
- `possible_escape` branches count as escapes for root classification, while
  preserving their cutoff/limit evidence in proof details.
- Forbidden Renju squares should remain visible in proof frames as tactical
  evidence, but they are not playable replies for Black.

The general corridor state machine, Renju overlay, and model-limit invariants
are defined in [`corridor_search.md`](corridor_search.md).

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

Use a three-part classification:

- Loss category: player-facing severity based on proven forced-corridor length.
- Root detail: analyzer-facing reason the final forced interval exists.
- Tactical notes: local misses or conversion issues that happened along the
  actual line.

Loss categories:

- `mistake`: the proven forced-corridor span is shorter than `5` plies. This
  is a near-term miss such as failing to answer a four or a short three-threat
  conversion.
- `tactical_error`: the proven forced-corridor span is `5` to `8` plies. The
  loss was tactical, but it required seeing several forcing replies ahead.
- `strategic_loss`: the proven forced-corridor span is `9` plies or longer.
  The losing side's last viable escape was far enough back that the report
  should frame it as a deeper strategic miss.
- `unclear`: the bounded analyzer cannot prove enough to assign a severity.

Root-detail categories:

- `strategic_loss`: a move changes the position from `escape_found` to
  `forced_win` under the same model and limits. This means the move entered the
  detected forced corridor; it does not claim the previous position was a
  game-theoretic draw or win for the loser.
- `missed_defense`: the losing side had at least one escape move, but the
  actual move did not play one.
- `missed_win`: a player had an immediate or forced win, but played elsewhere
  and allowed the game to continue.
- `unclear`: the bounded analyzer cannot prove enough to identify a root detail.

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

The root analysis record should stay compact and product-facing first:

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

Branch evidence belongs behind drilldown UI/report details. The lab report
should preserve enough to explain forced sequences without forcing players to
read the proof tree:

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

Current `reply_classification` values:

- `blocked_but_forced`: the reply answered the current threat but all modeled
  continuations stay inside the detected forced corridor.
- `confirmed_escape`: the reply wins immediately, breaks the threat, or avoids
  the next forced continuation.
- `no_legal_block`: the only apparent cost squares are illegal for the defender
  and no immediate counter-win exists, so the threat remains forced even with a
  single winning square.
- `possible_escape`: the analyzer cannot prove a legal reply still loses within
  the current model.
- `unknown`: the analyzer cannot enumerate the reply cleanly enough to classify
  it.

`prefix_ply` and `actual_reply` are replay-context attribution fields. The lab
proof fills them while a proof branch is still following the actual replay line.
Once a proof branch diverges into a virtual continuation, nested evidence leaves
these fields empty instead of pretending it maps to a real move.

Proof result statuses:

- `forced_win`: detected corridor proven within the model and limits.
- `escape_found`: defender has at least one model-valid move that exits the
  detected forced corridor. This includes `possible_escape` branches, which are
  escapes for root classification but carry limit causes.
- `unknown`: the position exceeded analyzer scope before a concrete legal
  defender reply could be evaluated.

Limit-hit proof results should carry named causes. Current causes:

- `depth_cutoff`: corridor proof depth ran out.
- `reply_width_cutoff`: a named reply set exceeded the current audit width cap.
- `attacker_child_unknown`: at least one attacker child could not be resolved.
- `defender_reply_unknown`: at least one defender reply could not be resolved.
- `model_scope_unknown`: the selected proof model had no concrete reply or
  forcing set for the position.
- `outside_scan_window`: the previous prefix was outside the configured scan
  range.

Reports should also preserve `unclear_context` for unresolved rows: reason,
previous prefix, proof status, limit causes, principal-line notation, scan
range, and compact board snapshots. This is lab-debug evidence, not
player-facing explanation.

`model` must include at least:

- reply policy: currently `corridor_replies`, the single active model for
  named corridor replies.
- rule set: freestyle or Renju.
- limits: corridor depth, nodes, time, maximum proof branches, and
  `max_scan_plies`.

Replay analysis has two scan modes:

- The CLI and default analysis options use `max_scan_plies=64`. Early-stop means
  short resolved corridors do not pay for the full cap, while long corridors
  still have enough room to resolve.
- Internally, `max_scan_plies=None` scans the full replay prefix history. This
  is reserved for curated fixtures and one-off diagnostics that want older notes
  such as conversion errors.
- With `max_scan_plies=N`, it walks backward from the final board at most `N`
  plies and stops early once the final forced corridor has a collected
  non-forced prefix.

The backward walk is a replay-analysis wrapper only. Each prefix proof still
uses the forward corridor solver, so the same corridor machinery can later be
called from bot search without replay-specific assumptions.

The proof tree can be stored separately from the summary so replay UI can show a
simple explanation first and expand into branch details later.

## Fixture Requirements

Fixtures should cover more than happy-path wins:

- terminal wins: single winning square, multiple winning squares, open four,
  Renju no-legal-block terminals
- escape behavior: confirmed escape, possible escape, immediate counter-win, and
  counter-threat escape
- replay imperfections: missed defense, missed win, conversion error, unknown
  gap, ongoing/draw
- corridor mechanics: short forced corridor, forced reply, forbidden Black defense,
  and model-limit cutoffs

Fixtures should print exact boards, expected labels, proof model, and limits.
They must fail if an implementation silently upgrades `unknown` into a forced
result. A `possible_escape` may contribute to `escape_found`, but it must remain
visible in evidence and report copy.

## UI Direction

First useful surface: replay.

Possible replay annotations:

- Mark the final winning line using the existing result-screen treatment.
- Mark the decisive attack move.
- Mark the losing side's last chance.
- Show the principal corridor continuation as a branch preview.
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
- `gomoku-bot`: shared tactical shape facts and corridor proof primitives while
  they remain lab-facing strategy logic.
- `gomoku-eval` or a new lab analysis module: bounded corridor search,
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

## Current Implementation

The `v0.4.2` implementation lives in `gomoku-eval` and is still a lab artifact:

- `gomoku_eval::analysis` defines the model/result types and bounded corridor
  proof walker.
- `analyze-replay` analyzes one replay JSON file.
- `analyze-replay-batch` analyzes a replay directory.
- `analyze-report-replays` samples compact tournament-report matches and
  reconstructs replay objects in memory.
- `analysis-fixtures` runs curated replay fixtures against the current model.
- `--include-proof-details` adds previous-prefix/final-start proof snapshots and
  visual decision frames for audit runs.

The curated public report lives under `gomoku-bot-lab/analysis-reports/` and is
published as `/analysis-report/`. It should be generated from
`gomoku-bot-lab/reports/latest.json` without explicit entrants, so it explains
the current published bot report's top-two matchup rather than an arbitrary
debug sample.

The current `v0.4.2` checkpoint:

- uses `reply_policy = corridor_replies`,
- uses corridor proof depth `4`,
- uses `max_scan_plies = 64`,
- resolves the current top-two 64-game sample with `64 analyzed / 64 total` and
  `0 failed`,
- classifies the sample as `3` mistakes, `25` tactical errors, `35` strategic
  losses, and `1` draw/ongoing game.

Historical implementation notes, older telemetry, rejected proof policies, and
debugging details are archived in
[`archive/v0_4_2_game_analysis_impl_notes.md`](archive/v0_4_2_game_analysis_impl_notes.md).

## Current Workflow

Use a small report-sampled run while tuning analyzer output:

```bash
cargo run --release -p gomoku-eval -- analyze-report-replays \
  --report reports/latest.json \
  --sample-size 8 \
  --report-json outputs/analysis/top2_smoke.json \
  --report-html outputs/analysis/top2_smoke.html \
  --max-depth 4 \
  --max-scan-plies 8
```

Use the release/checkpoint path only when the model or report output is ready
for review:

```bash
cargo run --release -p gomoku-eval -- analyze-report-replays \
  --report reports/latest.json \
  --sample-size 64 \
  --include-proof-details \
  --report-json analysis-reports/latest.json \
  --report-html analysis-reports/index.html \
  --max-depth 4
```

Keep scratch output under `gomoku-bot-lab/outputs/analysis/`. Commit only
analyzer code, docs, and deliberately curated report artifacts.

Next product decision: keep the analyzer in the lab until the proof-frame output
is clear enough for players, then choose between replay-screen annotation,
critical-moment tagging, or feeding corridor facts back into bot
ordering/narrow search.
