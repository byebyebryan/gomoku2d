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
- Which losing-side move became critical?
- Which neutral root cause explains the final corridor within analyzer limits?

The replay surface should produce concrete, bounded explanations:

- "Move 43: point of no return."
- "White's last chance was move 42."
- "Black had a setup corridor from here."
- "This looks like a missed defense, not a general position evaluation."

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
goal is to explain the setup corridor near the end of the actual game, not to
prove that every alternate state is a game-theoretic loss under perfect play. An
escape reply can leave the detected corridor even if the defender might still
lose later.

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
  detected setup corridor, or the corridor reaches a neutral state with no
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

Replay analysis consumes the same `CorridorThreatPolicy` as bot-owned corridor
search. That policy converts raw tactical facts into active threats and named
defender replies; the replay analyzer adds replay context, backward traceback,
root-cause labels, and report rendering.

This document only adds the replay-specific contract: start from a finished
game, walk the actual ending backward, and explain the setup corridor
without claiming to solve every alternate future.

## Corridor-Only Replay Flow

The replay analyzer assumes a finished decisive game ends inside a threat
corridor. The final winning move provides the terminal endpoint, but the
interesting explanation should usually stop earlier: at the point where the
loser first faced a lethal threat and the rest of the game became conversion.
The goal is not to solve the whole game; it is to walk backward and find the
latest losing-side decision that could have escaped the setup corridor before
lethal onset.

That gives the analysis useful boundaries and spans:

- Terminal move: the actual five was played.
- Lethal onset: the earlier frame where the loser no longer had a legal reply
  that avoids the attacker's terminal or known-lethal continuation.
- Cause boundary: the earlier frame where the loser still had an escape, or
  where the setup corridor began.
- Setup corridor: the cause boundary through lethal onset. This is the
  player-facing corridor: how the loser was forced into the already-lost state.
- Lethal tail: lethal onset through terminal move. This proves conversion, but
  is usually less explanatory than the setup corridor.

The lethal tail is often obvious. The setup corridor is where the analyzer
should spend explanation effort: did the loser get locked into the lethal state
by forcing replies, or did they simply miss an earlier reply?

Implementation note: `final_forced_interval` remains the full proof suffix from
forced start through terminal because the recursive proof model needs that
complete evidence. Reports and replay UI derive `setup_corridor` as
`final_forced_interval.start_ply..lethal_onset.prefix_ply` when lethal onset is
known.

The core loop is:

1. Start at the final winning move and identify the full final forced interval.
2. Walk backward along the actual replay through losing-side decision points.
3. At each losing-side turn, enumerate named corridor replies: direct defenses,
   immediate wins, and valid counter-threats.
4. For each alternative, follow only corridor continuations. The winner may
   complete an immediate win, answer a threat, or materialize a new
   immediate/imminent/lethal threat; it may not use broad quiet search to
   preserve a proof.
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

When lethal detection is available, a winning-side actual move that creates a
proven lethal threat also counts as a valid corridor continuation and can become
the analysis endpoint for the final conversion segment.

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

Reply candidates use the same contract in the static report and browser replay
UI. Generate immediate replies first, generate imminent replies only if there is
no immediate tier, render every surviving candidate box, then filter out actual
and forbidden moves before proof search. The report list contains only legal
non-actual alternatives; actual and forbidden candidates stay on the board as
markers because they explain why no branch was searched.

Transition labels follow the same proof-status rules everywhere:

- `escape_found -> forced_win` caused by the losing side: `missed_defense`.
- `escape_found -> forced_win` caused by the winning side: `corridor_entry`.
- `unknown -> forced_win`: `unclear`, even if the current forced sequence is
  proven.

## Replay Model Settings

The active replay analyzer exposes one corridor model:

- reply policy: `corridor_replies`
- proof budget: `--max-depth`, interpreted as corridor depth rather than broad
  minimax depth
- scan budget: `--max-scan-plies`, interpreted as how far backward through the
  finished replay the analyzer may look for the final corridor boundary

The report must keep those model settings visible. Product copy should prefer
"setup corridor" for the player-facing cause span and reserve "full forced
interval" for implementation/proof evidence. "Detected setup corridor" or "last
known escape" is safer when summarizing unresolved branches.

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

## Browser Integration Contract

The browser uses the same analyzer as the static report path; it does not
reimplement corridor search in React or Phaser.

- `gomoku-web` converts a saved match into exact `gomoku-core` replay JSON using
  `WasmBoard` hashes.
- `gomoku-wasm` owns `WasmReplayAnalyzer.createFromReplayJson(...).step(...)`.
- The wasm bridge returns structured analysis and board payloads as JSON
  strings. `gomoku-web/src/core/wasm_bridge.ts` is the TypeScript parsing
  boundary; route, store, and renderer code should consume typed helpers.
- `gomoku-analysis` owns `ReplayAnalysisSession`, which walks backward from the
  ending and returns frame annotations before final analysis is ready.
- `gomoku-web` runs the analyzer inside a cancellable web worker. Replay
  playback and route changes must remain responsive while analysis progresses.

The step result schema intentionally separates progress from final summary:

- `status`: `running`, `resolved`, `unclear`, `unsupported`, or `error`;
- `annotations`: per-ply highlights and markers produced during this step;
- `current_ply`: the next prefix to analyze, or `null` when finished;
- `analysis`: final `GameAnalysis` only when the analyzer is done;
- `counters`: searched prefixes, branch roots, and proof nodes.

Route/UI code should merge annotations by `ply` and render only the current
replay frame's markers. Analysis output is transient product state in `0.4.6`;
do not store it in local or cloud profile documents.

The replay UI may simplify raw proof roles for readability: `possible_escape`
and `confirmed_escape` both display as escape (`E`), forbidden replies reuse the
forbidden/caution visual, immediate loss uses the existing warning marker, and
unknown markers can stay hidden. Reports/debug data should keep the raw roles.

Replay navigation uses one surface instead of a separate analysis mode. The page
opens on the finished board, turn controls step backward or forward by two plies
to preserve side-to-move perspective, and the slider remains available for raw
move scrubbing. Each frame should show the opponent's last actual move as the
focused stone and the current side's next actual move as the hover target; loser
side analysis overlays then add alternate replies and `L` / `E` outcomes.

The replay timeline is an analysis surface, not a normal media progress bar.
The base track stays neutral, the setup corridor fills in red from the latest
escape/cause boundary through lethal onset, and the latest escape is shown as a
green point marker. The deck should label this area as `Status`, using the
analyzer's progressive state and counters rather than repeating the final match
result.

Once analysis resolves, the status should describe the current frame rather than
only the whole replay. The terminal frame can show `Black won` or `White won`
with setup-corridor length when known. Winner-side frames inside the setup
corridor can read `Black can force a win`; loser-side frames can read `White is
locked in`; the latest escape frame can read `White's last escape`. Frames after
lethal onset should read as guaranteed conversion, while frames before the setup
corridor should fall back to normal turn language such as `Black to move`.

## Backward Walk

For a finished game, walk backward from the final move and test prefixes. Do not
assume forced-win state is monotonic across the actual game.

A player can:

- create a forced win,
- miss the conversion and release it,
- regain a forced win later after another reply miss.

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
- `critical_loser_ply`: losing-side move that allows the decisive attack or misses
  the last escape.

Important labels:

- Final win: the actual ending move and winning line.
- Full forced interval: a contiguous range of prefixes where the winner has a
  proof through terminal.
- Setup corridor: the replay-facing subset from forced start through lethal
  onset.
- Point of no return: the start of the setup corridor.
- Last chance: the final escape opportunity before that interval.
- Decisive attack: the winner's forcing move.
- Critical loser move: the losing-side move that made the attack possible or failed to
  escape it.

The decisive attack and critical loser move are related but not always the same
move. A strong attack may be the winner's achievement, while the losing-side
boundary may be the previous move.

## Human Imperfection Layer

Real games, especially human games, are not ideal games. Analysis should
classify the actual line separately from the ideal proof.

Keep the layers separate:

- Corridor evidence: where the setup corridor starts, where lethal onset begins,
  and which replies were considered at each losing-side decision.
- Root detail: analyzer-facing reason the final forced interval exists. This is
  not a player-facing severity grade.
- Tactical notes: local misses or conversion issues that happened along the
  actual line.

Do not infer mistake severity directly from setup-corridor length. We may
reintroduce mistake/tactical/strategic labels later, but they should be based on
the type of missed opportunity and player-facing explanation, not a simple span
threshold.

`0.4.7` intentionally stops short of full mistake detection. Lethal onset gives
the analyzer a better boundary first: after onset, the loser is already in a
guaranteed-loss state under the model, so a later reply should not be called the
mistake just because it fails to save the game. The useful question for a
follow-up is whether an earlier actual move ignored a concrete response or
escape candidate before the position became lethal.

The `0.4.8` mistake layer uses response semantics, not corridor length:

- Before lethal onset, if a player faces immediate or imminent threats and plays
  outside the highest-priority response candidate set, classify that as a missed
  response.
- At the last losing-side decision before lethal onset, if a viable prevention
  move exists and the actual move is not a missed response, classify that as
  missed lethal prevention.
- If the viable escape/prevention point is earlier in the setup corridor, or the
  model finds no viable late prevention at all, classify the loss as a missed
  escape from the setup corridor.
- If proof is bounded or unknown, label cautiously as a possible mistake or
  unclear boundary rather than overclaiming.

`0.4.8` implements this as a derived failure layer on top of the existing proof
tree. It does not run a second broad search. The analyzer first records the
setup corridor and lethal onset, then classifies the latest losing-side failure
before onset:

- `missed_immediate_win`: the losing side had an immediate win and played
  elsewhere.
- `missed_immediate_response`: the losing side ignored a legal response to a
  four threat.
- `missed_imminent_response`: the losing side ignored a legal response or
  counter-threat against a forcing three threat.
- `missed_lethal_prevention`: immediately before lethal onset, the losing side
  had a viable way to avoid onset but chose another non-obvious losing line.
- `missed_escape`: the losing side failed to escape the setup corridor earlier,
  or the model found no viable late prevention before onset.
- `unclear`: the proof boundary is unknown or outside the scan window.

Root-detail categories:

- `corridor_entry`: a move changes the position from `escape_found` to
  `forced_win` under the same model and limits. This means the move entered the
  detected setup corridor; it does not claim the previous position was a
  game-theoretic draw or win for the loser.
- `missed_defense`: the losing side had at least one escape move, but the
  actual move did not play one.
- `missed_win`: a player had an immediate or forced win, but played elsewhere
  and allowed the game to continue.
- `unclear`: the bounded analyzer cannot prove enough to identify a root detail.

If the previous prefix is `unknown`, do not label the transition as a corridor
entry. The correct root cause is `unclear`, optionally with a tactical note that
the move entered a proven forced interval.

Tactical notes:

- `conversion_error`: the winning side had a forced win, played a move that
  released it, then later won after another reply miss.
- `strong_attack`: the decisive move created a forcing line even though the
  previous position was not clearly lost.

This split avoids overlap. For example, a `missed_defense` root cause already
explains the missed local block, so it should not also receive a separate
mistake-style tactical note.

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
  lethal_onset
  failure
  limits
  final_forced_interval
  last_chance_ply
  decisive_attack_ply
  critical_loser_ply
  root_cause
  tactical_notes
  principal_line
  unclear_context
  proof_summary
```

`FailureAnalysis` is intentionally compact:

```text
FailureAnalysis
  mode
  side
  prefix_ply
  actual_move
  missed_candidates
  prevented_onset_ply
  confidence
```

`missed_candidates` are concrete legal alternatives with their tactical roles
and outcome under the existing corridor proof. `possible` confidence means the
candidate escaped the bounded model but was not fully proven safe.

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
  continuations stay inside the detected setup corridor or lethal tail.
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
  detected setup corridor. This includes `possible_escape` branches, which are
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
  plies and stops early once the full final forced interval has a collected
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
- corridor mechanics: short setup corridor, forced reply, forbidden Black
  defense, and model-limit cutoffs

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

The analyzer started in the Rust lab where reports and CLI inspection were
cheap. It now has a shared boundary that can feed both lab reports and the web
replay UI without moving analysis rules into React or Phaser.

Current layering:

- `gomoku-core`: board, rules, legality, winning-line checks, compact move
  codecs, and any generic line/shape facts that are not bot-specific.
- `gomoku-bot`: shared tactical shape facts and corridor proof primitives while
  they remain lab-facing strategy logic.
- `gomoku-analysis`: bounded corridor traceback, proof summaries, and
  product-safe analysis result records.
- `gomoku-eval`: CLI/report shell, fixture runner, and HTML/JSON report
  rendering around `gomoku-analysis`.
- `gomoku-wasm`: browser bridge for `WasmReplayAnalyzer` and exact replay hash
  helpers needed by web-side replay conversion.
- `gomoku-web`: saved-match-to-core-replay adapter, worker/UI orchestration, and
  presentation only.

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

The analyzer model now lives in `gomoku-analysis`, with `gomoku-eval` kept as
the lab/report shell and `gomoku-wasm` as the browser bridge:

- `gomoku_analysis` defines the model/result types and bounded corridor proof
  walker.
- `analyze-replay` analyzes one replay JSON file.
- `analyze-replay-batch` analyzes a replay directory.
- `analyze-report-replays` samples compact tournament-report matches and
  reconstructs replay objects in memory.
- `analysis-fixtures` runs curated replay fixtures against the current model.
- `--include-proof-details` adds previous-prefix/final-start proof snapshots and
  visual decision frames for audit runs.
- `WasmReplayAnalyzer.createFromReplayJson(...).step(...)` exposes the same
  model to browser code through the JSON bridge.
- `ReplayAnalysisRunner` runs that wasm analyzer in a web worker and streams
  step results back to the replay page, so route-level callers receive the same
  progressive shape as the lab report path.
- `gomoku-web/src/replay/replay_analysis_core.ts` converts `SavedMatchV2`
  records into core replay JSON with exact wasm-generated position hashes, then
  constructs the analyzer.

The curated public report lives under `gomoku-bot-lab/analysis-reports/` and is
published as `/analysis-report/`. It should be generated from the full/debug
tournament report used to produce the compact bot report, because the published
bot report intentionally omits replay cells. The curated sample explains the
in-game Easy/Normal/Hard preset triangle rather than an arbitrary debug matchup.

The current analyzer checkpoint:

- reads replay cells from the full tournament report under `outputs/`,
- publishes compact analysis JSON with `selector = "Preset triangle"`,
- uses `reply_policy = corridor_replies`,
- uses corridor proof depth `4`,
- uses `max_scan_plies = 64`,
- generated the current curated 192-game preset-triangle report with `192/192` analyzed
  and `0` failed entries,
- writes the curated batch report under
  `gomoku-bot-lab/analysis-reports/`,
- powers replay-screen highlights and markers from saved replay data without
  persisting analysis results in local/cloud profile schema.

The internal `GameAnalysis` result is not the same shape as the published batch
report JSON. The batch report wraps many replay analyses with source provenance,
summary counts, diagnostics, and rendered proof details; the browser replay UI
uses the step-wise wasm analyzer instead of loading that published batch JSON.

Historical implementation notes, older telemetry, rejected proof policies, and
debugging details are archived in
[`../../archive/v0_4_2_game_analysis_impl_notes.md`](../../archive/v0_4_2_game_analysis_impl_notes.md).

## Current Workflow

Use a small report-sampled run while tuning analyzer output:

```bash
cargo run --release -p gomoku-eval -- analyze-report-replays \
  --report outputs/full-tournament-report.json \
  --sample-size 8 \
  --report-json outputs/analysis/top2_smoke.json \
  --max-depth 4 \
  --max-scan-plies 8
```

Use the release/checkpoint path only when the model or report output is ready
for review:

```bash
cargo run --release -p gomoku-eval -- analyze-report-replays \
  --report outputs/full-tournament-report.json \
  --selector preset-triangle \
  --published-report-json analysis-reports/report.json \
  --max-depth 4 \
  --max-scan-plies 64
```

Keep scratch output under `gomoku-bot-lab/outputs/analysis/`. Commit only
analyzer code, docs, and deliberately curated report artifacts.

Keep analysis derived from saved replay data; do not persist it in local/cloud
profile schema unless repeated runtime cost becomes a real product problem.
