# `v0.4.0` Search Bot Enhancement Plan

Status: ad-hoc implementation plan. This captures the current bot-lab work loop
so the commit boundaries and evaluation gates stay clear.

Current progress:

- Commit 1 and Commit 2 landed together in
  `29a88ca feat(bot): scaffold tactical search experiments`.
- Commit 3 tactical candidates was rejected after focused testing.
- Commit 4 tactical move ordering was rejected after focused testing.
- Commit 5 tactical eval was rejected after focused testing.
- Commit 6 tactical shape features landed in
  `263e734 feat(bot): add tactical shape analyzer labels`.
- Commit 7 tactical scenario diagnostics landed in
  `341f2fc feat(bot): add tactical scenario diagnostics`.
- Commit 8 forced-line search primitives landed in
  `dfc10c9 feat(bot): add forced-line search primitives`.
- Commit 9 broad bounded threat extension is being rejected after focused
  analysis. It found forced lines by scanning the whole board at every leaf,
  which reduced counted nodes on some cases but increased wall/CPU cost and lost
  match strength.

## Goal

Evolve the existing `SearchBot` into a measurable experimental bot without
forking a separate `AdvancedSearchBot` yet. The current baseline must remain
reproducible, and experimental features should only become exposed config after
they show value in focused tests.

This is not a solver project. The near-term product goal is a fast, efficient
practice bot with enough real knobs to support interesting gameplay, plus a
foundation for later reverse search, replay analysis, and puzzle generation.
Threat-space search is relevant because it gives Gomoku-specific tactical
language, not because `v0.4.0` should become a full TSS/proof-number solver.

That creates two lanes:

- **Practice bot:** must stay responsive, configurable, and tunable for play
  style. It should use tactical knowledge when it helps, but it should not turn
  into a rigid solver that only optimizes proof quality.
- **Analysis tooling:** can become more solver-like later if replay analysis,
  reverse search, or puzzle generation needs proof-oriented machinery.

## Design Direction

Keep one `SearchBot` implementation, but do not keep dead feature toggles in
`SearchBotConfig`. The first three shallow integration attempts were discarded,
so the next pass should improve the analyzer vocabulary before wiring new
behavior into search.

Decision: do not pivot the main bot to full threat-space search for `v0.4.0`.
Borrow the tactical model, not the whole architecture. TSS is valuable for
describing forcing threats and concrete replies, but a full dependency-tree
search is optimized for proof, not for a fast practice opponent with adjustable
style and difficulty.

Adopt threat-space-search terminology where it helps:

- **gain square:** the attacking move that creates a threat
- **cost/defense squares:** defender replies required to answer that threat
- **rest squares:** remaining squares that make the threat pattern possible

Do not pivot the primary bot to a full TSS engine yet. Full TSS is
solver-oriented and brings dependency trees, conflict checks, all-defenses
handling, and proof verification. Those can become analysis modules later. For
normal play, the main bot should stay alpha-beta based, with local threat facts
feeding candidate ordering, static eval, tactical diagnostics, and eventually
small forced-line modules.

Stable lab specs stay focused on reproducible baseline configs:

- `search-d3`
- `fast`
- `balanced`
- `deep`

This avoids duplicating the search loop while still allowing tournament reports
to compare meaningful variants once a feature has earned a config surface.

The current split is clearer now:

- **Forced-line search** is useful once a branch has immediate tactical forcing
  states, but it must derive those facts from local shapes instead of scanning
  the whole board at every leaf.
- **Shape labels** are useful for non-terminal shape creation: broken threes,
  open threes, and other moves that matter before immediate winning replies
  exist.
- **TSS-style local threat facts** sit between those two ideas. They inspect the
  four lines through the last/candidate move and return concrete tactical facts:
  severity/type, gain square, cost/defense squares, rest squares, and whether the
  shape is forcing.

In Gomoku terms, some depth problems are narrow but deep, while others start as
shape-recognition problems. A global depth increase is too expensive, but a
forced-line extension must exploit local shape structure to stay cheap.

Tactical scenarios still matter, but as diagnostics rather than match ranking.
They answer:

- Does the current baseline already solve this position?
- Does a proposed tactical change solve it with acceptable node/time cost?
- Does the change regress obvious safety cases?

If baseline depth 3 or depth 5 already solves a scenario, that scenario becomes
a regression guard, not a reason to add new logic.

## Phases

### Phase 1: Freeze Baseline Behavior

Lock down current `SearchBot` behavior before tactical changes affect move
choice.

- Keep `SearchBot::new(depth)` and `SearchBotConfig::custom_depth(depth)` as
  frozen baseline config.
- Add tests that baseline constructors preserve the current config.
- Ensure trace output records stable config fields so reports explain which
  knobs were active.
- Keep current web practice bot behavior unchanged.

### Phase 2: Add Experimental Config And Tactical Analyzer Skeleton

Add the scaffolding required for ablation tests without changing search results.

- Keep `SearchBotConfig` stable until an experiment proves useful.
- Keep lab spec parsing focused on stable baseline depth specs and named
  presets.
- Add an internal tactical analyzer skeleton.
- First analyzer fields:
  - legal move
  - immediate win
  - immediate block
- Do not wire analyzer output into candidate generation, ordering, or eval yet.

### Phase 3: Tactical Candidates

Decision: discarded for now.

Focused testing showed that immediate-win/block candidate expansion is
redundant with the current radius-2 baseline. A 16-game Renju comparison at
1000 ms CPU/move ended `search-d3` over `search-d3+candidates` by `9-7`, with
the candidates variant slightly higher average move time and budget exhaustion.

Learning:

- Do not keep a `tactical_candidates` toggle just to force-add immediate
  wins/blocks.
- Candidate expansion may become useful later, but only after the analyzer can
  identify richer shapes that radius-2 can miss in practice.
- Move ordering and eval can proceed without widening the candidate set.

### Phase 4: Tactical Move Ordering

Decision: discarded for now.

Immediate-win/block ordering was not strong enough to keep as a separate
toggle. A corrected implementation used tactical priority before TT tie-breaks,
but an 8-game Renju comparison at 1000 ms CPU/move ended `search-d3` over
`search-d3+ordering` by `5-3`. The ordering variant searched fewer nodes but
had higher average move time and slightly higher budget exhaustion.

Learning:

- Do not keep an immediate-win/block-only `tactical_move_ordering` toggle.
- The existing TT ordering is already useful; shallow tactical sorting can
  interfere without producing better play.
- Revisit ordering only after tactical eval/shape detection can classify richer
  threats.

### Phase 5: Tactical Eval

Decision: discarded for now.

The first pass only scored immediate winning moves for the current player and
the opponent. That is too shallow: the baseline search/root prefilter already
handles many immediate tactical cases, while adding another leaf-eval branch
increased complexity without producing a clear strength gain.

Learning:

- Do not keep an immediate-win/block-only `tactical_eval` toggle.
- Eval work should wait until the analyzer can describe richer shapes:
  open three, open four, blocked four, broken three, double threat, and forcing
  reply.
- The next commit should be behavior-neutral shape detection with focused unit
  tests, not another direct search integration.

### Phase 6: Tactical Shape Features

Add richer analyzer output without changing move choice.

Initial target features:

- open four
- blocked four
- open three
- broken three
- double threat

This should stay behind internal helpers and tests. Search integration comes
later only if the shape features are correct and readable.

Current implementation note: this slice adds labels to `analyze_tactical_move`
only. The search loop still uses the same candidates, ordering, and static eval
as the baseline.

### Phase 7: Tactical Scenario Diagnostics

Add a focused scenario runner before adding more bot behavior.

This is intentionally not a replacement for tournament eval. Tournament eval
answers "which bot scores better over many games?" Scenario diagnostics answer
"does this bot understand this specific tactical shape, and what did it cost?"

Scenario categories:

- immediate win
- forced block
- open four
- blocked four
- open three
- broken three
- double threat
- tempting bad extension
- sparse long-range threat, only if radius-2 genuinely misses it

Each scenario should record:

- board position
- side to move
- expected move set initially; tactical-class assertions can be added later
- actual move
- pass/fail
- nodes, prefilter nodes, time, depth reached, budget exhaustion

Run the baseline configs first: `search-d2`, `search-d3`, and `search-d5`.
Only positions that expose a real baseline gap should drive new search logic.

Current implementation note: `gomoku-eval tactical-scenarios` runs the focused
one-move diagnostics across search configs and can write JSON. The initial
seven-case smoke run showed `search-d2` failing the broken-three creation case,
while `search-d3` and `search-d5` passed the current set. This is useful
diagnostic evidence, but the set is still too small to justify product-facing
bot presets.

### Phase 8: Broad Bounded Forced-Line Search

Decision: reject the broad integration.

The goal was reasonable: spend extra depth only when the position is in a
forcing tactical branch.

1. If the current player has an immediate win, prefer/return it immediately.
2. If the opponent has exactly one immediate win, treat the block as forced and
   extend that line.
3. If the opponent has multiple immediate wins and the current player cannot win
   now, treat the position as a near-forced loss.
4. If a move creates a forcing threat, extend that branch by a small bounded
   amount.
5. Stop extension with explicit caps: max extension depth, node/time deadline,
   and normal terminal checks.

Current implementation note: the first forced-line slice added classifiers for
the node's immediate tactical state and the threat state after a candidate move.
The node-state classifier distinguishes a legal forced block from an unblockable
immediate loss, which matters for Renju forbidden-move overlap cases.

The rejected integration pass added a default-off `threat_extension_depth` config
and lab-only `+threatN` spec suffix. The extension activated at depth-0 leaves
and called `immediate_winning_moves_for()` for both sides through
`classify_forced_line_state()`.

Measured behavior:

- `search-d2+threat1` does not improve the current scenario pass count because
  the remaining miss is `create_broken_three`, which creates a broken-three
  shape rather than an immediate forced line.
- It does reduce work on already-solved forced cases. In one sweep,
  `create_open_four` dropped from depth 2 / 234 nodes to depth 1 / 53 nodes, and
  `create_double_threat` dropped from depth 2 / 312 nodes to depth 1 / 83 nodes.
- It performs poorly in match ablation. In a 16-game d3 Renju head-to-head,
  `search-d3` beat `search-d3+threat1` by `11-5`. The extension cut counted
  nodes but increased average move time from `236.63 ms` to `480.53 ms` and
  budget exhaustion from `3.3%` to `26.1%`.

Root cause:

- `nodes` only counts negamax nodes.
- The extension adds expensive leaf work that is not reflected in `nodes`.
- `immediate_winning_moves_for()` scans nearby empty moves, clones/probes board
  state, and can invoke Renju forbidden logic. Doing that twice at every quiet
  leaf is too broad.

Learning:

- Whole-board immediate-win scans are useful diagnostics, not a cheap extension
  primitive.
- Real threat-space search should derive forced replies from the shape that was
  just created: inspect four lines through the last/candidate move and emit the
  concrete continuation moves directly.

### Phase 9: TSS-Style Local Threat Facts

Add local tactical facts before trying another forced-line integration.

This is related to the existing shape labels, but it is not the same thing.
Phase 6 labels answer "what kind of shape did this move create?" The next
primitive should answer "which concrete moves does this shape force?" using the
TSS vocabulary of gain, cost/defense, and rest squares.

This is the right next step because it improves the shared tactical vocabulary
without committing to a solver architecture. The same facts can support:

- move ordering and static eval for the practice bot
- scenario diagnostics and tournament explanations in bot lab
- future replay analysis and reverse-search/puzzle features

Target facts:

- terminal five / win now
- open four with two winning endpoints
- simple four with one forced block
- open three with extension/block endpoints
- broken three as a non-forced shape fact for eval/order
- no forcing shape

Rules:

- Inspect only the four lines through the last/candidate move.
- Return concrete move lists, not just booleans.
- Represent each fact in TSS-like terms: gain square, cost/defense squares, rest
  squares, severity, and forcing/non-forcing status.
- Avoid `immediate_winning_moves_for()` in the hot path.
- Keep Renju handling explicit: start with freestyle/local-shape facts if needed,
  then add legality filtering at the consumer boundary.

Non-goal:

- Do not build dependency-tree TSS, all-defenses search, or proof-number search
  in this slice.
- Do not make the default practice bot depend on proof-oriented tactical search.

### Phase 10: Shape-Aware Eval Or Ordering

This remains the next search-behavior slice after local threat facts exist.

Forced-line search did not solve the `create_broken_three` gap because that move
does not create an immediate winning reply. The Phase 6 analyzer already labels
`open_three`, `broken_three`, `open_four`, `blocked_four`, and `double_threat`;
the next experiment should use those labels to influence search without
pretending they are terminal forced lines.

Likely order:

1. Shape-aware ordering for pruning and cheaper discovery.
2. Shape-aware eval if scenario failures remain because leaf scores undervalue
   non-terminal shapes.
3. Candidate expansion only with concrete sparse-position evidence.

Initial usefulness target:

- `search-d2` should choose `I8` or `J8` in `create_broken_three` without
  requiring global depth 3.
- The change should not regress immediate win/block, open-four, double-threat,
  or Renju forbidden-overlap cases.
- If ordering alone reduces nodes but still chooses the wrong move, treat that as
  partial evidence and move to a small shape-aware eval experiment.

### Phase 11: TSS-Inspired Forced Extension

Only retry forced-line extension after Phase 9 exists.

The retry should not ask "does this leaf have immediate wins?" by scanning the
board. It should consume concrete continuation moves from the local threat facts:

- open four: treat as winning unless the ruleset creates a legal exception
- simple four: extend only the forced block
- open three: extend the small reply set and attacking continuations

Promotion gate:

- improves a targeted forced-line scenario or material runtime without hurting
  d3 tournament ablation
- records tactical-probe metrics if any non-node work remains significant
- remains lab-only until it has both scenario and tournament evidence

This is still not full TSS. It is a bounded tactical module plugged into the
alpha-beta bot. A broader TSS/proof module belongs later if replay analysis,
reverse search, or puzzle generation needs it.

If this retry starts needing dependency trees, rest-square conflict resolution,
or all-defenses proof handling, stop and split it out as an analysis module
instead of burying it inside `SearchBot`.

## Intended Commit Boundaries

### Commit 1: Config Plumbing And Baseline Guardrails

Includes:

- Baseline constructor/preset guardrails.
- Trace output for stable config fields.
- Lab spec parser tests for stable depth specs and named presets.
- Rejection tests for discarded feature suffixes.
- Tests for baseline defaults and parser behavior.

Expected behavior change: none.

### Commit 2: Tactical Analyzer Skeleton

Includes:

- Internal tactical analyzer type/helper.
- Tests for immediate win and immediate block detection.
- No integration with candidate generation, ordering, or eval.

Expected behavior change: none.

Completed in `29a88ca` together with Commit 1.

### Commit 3: Tactical Candidates

Decision: discarded.

Record the failed experiment in this doc and remove the config/code path rather
than carrying a toggle with no demonstrated value.

### Commit 4: Tactical Move Ordering

Decision: discarded.

Record the failed experiment in this doc and remove the config/code path rather
than carrying another shallow tactical toggle.

### Commit 5: Tactical Eval

Decision: discarded.

Record the failed experiment in this doc and remove the config/code path rather
than carrying a third shallow tactical toggle.

### Commit 6: Tactical Shape Features

Includes:

- Analyzer fields for richer tactical shapes.
- Curated board tests for open/blocked fours, open threes, broken threes, and
  double threats.
- No candidate generation, ordering, or eval integration.

Expected behavior change: none.

Completed in `263e734`.

### Commit 7: Tactical Scenario Diagnostics

Includes:

- Scenario data structure for tactical fixtures.
- Runner/report output for pass/fail, chosen move, expected move set, and
  search metrics.
- Initial scenario set covering a useful subset of the Phase 7 categories.
- Baseline run comparing `search-d2`, `search-d3`, and `search-d5`.

Expected behavior change: none.

Completed in `341f2fc`.

### Commit 8: Forced-Line Search Primitives

Includes:

- Internal helpers to classify immediate-win, forced-block, multi-threat, and
  creates-threat states.
- Focused unit tests for the forcing-state classifier.
- No integration with the normal search loop yet.

Expected behavior change: none.

Completed in `dfc10c9`.

### Commit 9: Broad Bounded Threat Extension

Decision: discard.

Record the failed experiment in this doc and remove the `threat_extension_depth`
config, `+threatN` parser support, and depth-0 broad extension code path.

Expected behavior change: none after cleanup.

### Commit 10: TSS-Style Local Threat Fact Primitive

Includes:

- Extend or replace the Phase 6 shape analyzer with a local fact helper that
  inspects four lines through a candidate move.
- Return gain, cost/defense, and rest squares for open fours, simple fours, and
  open threes.
- Keep broken three as a non-forced fact.
- Focused tests for each shape's concrete moves.
- No search integration yet.

Expected behavior change: none.

Current working slice: add this as a private, behavior-neutral helper in
`gomoku-bot/src/search.rs` first. The helper should return concrete facts for
terminal fives, open fours, simple fours, open threes, and broken threes, but it
must not affect candidate generation, move ordering, static eval, or search
depth until a later ablation commit consumes it.

### Commit 11: Shape-Aware Ordering/Eval Experiment

Includes:

- A focused red test or tactical sweep expectation for `create_broken_three`.
- First attempt should prefer ordering if it can be scoped narrowly and measured
  by node count plus move choice.
- If ordering does not change the chosen move, move to a small eval adjustment
  that scores Phase 6 shape labels at leaves.
- Remove or document failed sub-experiments before moving on.

Expected behavior change: only for experimental lab specs until scenario and
tournament evidence justify promotion.

### Commit 12: TSS-Inspired Forced Extension Retry

Includes:

- Consume local threat facts instead of whole-board immediate-win scans.
- Extend only concrete forced reply lists.
- Add metrics for any tactical-probe work outside the negamax node count.
- Run focused tactical sweep and d3 ablation before deciding whether to keep it.

Expected behavior change: only for experimental lab specs until proven.

## Evaluation Gates

Before moving from one behavioral commit to the next:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler`
- `npm --prefix gomoku-web run build`

For behavioral integration commits, also run at least a small ablation
tournament. After a feature survives focused testing, run a clean full
tournament report and publish/update the report only from a clean code commit.

Scenario diagnostics are required before the next behavioral integration commit.
The tactical scenario runner should be cheap enough to run during development,
while full tournament reports remain the slower release-quality check.

## Risks

- Tactical candidates can increase branching factor enough to erase strength
  gains.
- Tactical ordering can improve pruning but also bias the bot into shallow
  tactical tunnel vision.
- Tactical eval is the highest-risk phase because tuning can pass unit tests
  while making play feel worse.
- Shape detection can easily become a second rules engine. Keep the analyzer
  narrow, tested, and derived from existing core board APIs where possible.
- Forced-line search can become unbounded threat-space search by accident. Keep
  explicit extension caps and deadline checks.
- Scenario fixtures can become overfit. Treat them as diagnostic coverage, then
  confirm useful changes with tournament ablation.
- If toggles make `search.rs` too hard to reason about, revisit splitting into
  a separate bot or extracting modules before adding more features.
