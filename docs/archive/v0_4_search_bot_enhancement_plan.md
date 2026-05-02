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
- Commit 9 bounded threat extension is in progress. Focused investigation shows
  it is useful for already-forced immediate threat lines, but it does not solve
  non-terminal shape creation such as broken threes.

## Goal

Evolve the existing `SearchBot` into a measurable experimental bot without
forking a separate `AdvancedSearchBot` yet. The current baseline must remain
reproducible, and experimental features should only become exposed config after
they show value in focused tests.

## Design Direction

Keep one `SearchBot` implementation, but do not keep dead feature toggles in
`SearchBotConfig`. The first three shallow integration attempts were discarded,
so the next pass should improve the analyzer vocabulary before wiring new
behavior into search.

Stable lab specs stay focused on reproducible baseline configs:

- `search-d3`
- `fast`
- `balanced`
- `deep`

This avoids duplicating the search loop while still allowing tournament reports
to compare meaningful variants once a feature has earned a config surface.

The current split is clearer now:

- **Forced-line search** is useful once a branch has immediate tactical forcing
  states: win now, one legal forced block, opponent multi-threat, or unblockable
  loss.
- **Shape-aware ordering/eval** is needed for non-terminal shape creation:
  broken threes, open threes, and other moves that matter before immediate
  winning replies exist.

In Gomoku terms, some depth problems are narrow but deep, while others start as
shape-recognition problems. A global depth increase is too expensive, but a
forced-line extension alone cannot make the bot choose a quiet shape-building
move if the leaf classifier still sees the position as `Quiet`.

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

### Phase 8: Bounded Forced-Line Search

Add selective extension for forcing tactical branches.

Target behavior:

1. If the current player has an immediate win, prefer/return it immediately.
2. If the opponent has exactly one immediate win, treat the block as forced and
   extend that line.
3. If the opponent has multiple immediate wins and the current player cannot win
   now, treat the position as a near-forced loss.
4. If a move creates a forcing threat, extend that branch by a small bounded
   amount.
5. Stop extension with explicit caps: max extension depth, node/time deadline,
   and normal terminal checks.

This should be implemented as a tactical extension around search, not as an
unbounded "keep searching threats forever" mode.

Initial limits:

- `threat_extension_depth`: small, likely 1-2 until proven useful
- reuse existing wall/CPU deadline checks
- no product-facing config until focused scenarios and tournament ablation show
  value

Current implementation note: the first forced-line slice added classifiers for
the node's immediate tactical state and the threat state after a candidate move.
The node-state classifier distinguishes a legal forced block from an unblockable
immediate loss, which matters for Renju forbidden-move overlap cases.

The first integration pass adds a default-off `threat_extension_depth` config and
lab-only `+threatN` spec suffix. The extension only activates at depth-0 leaves
and only follows forced tactical states: immediate win, legal forced block,
opponent multi-threat, or unblockable immediate loss.

Measured behavior from the focused tactical sweep:

- `search-d2+threat1` does not improve the current scenario pass count because
  the remaining miss is `create_broken_three`, which creates a broken-three
  shape rather than an immediate forced line.
- It does reduce work on already-solved forced cases. In one sweep,
  `create_open_four` dropped from depth 2 / 234 nodes to depth 1 / 53 nodes, and
  `create_double_threat` dropped from depth 2 / 312 nodes to depth 1 / 83 nodes.
- This is useful evidence for the extension mechanism, but not enough to promote
  `+threatN` to a stable bot preset or player-facing knob.

### Phase 9: Shape-Aware Eval Or Ordering

This is now the next likely behavior slice.

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

### Commit 9: Bounded Threat Extension

Includes:

- Search integration behind internal experimental config or lab-only path.
- Extension caps for depth and existing wall/CPU deadlines.
- Scenario comparison against baseline.
- No tournament ablation unless the tactical sweep shows either a pass-count
  improvement or a clear strength hypothesis. Node reduction on already-passing
  scenarios is useful, but not enough by itself.

Expected behavior change: only for the experimental forced-line config.

Current working slice. Baseline configs should remain unchanged.

### Commit 10: Decision And Cleanup

Includes:

- Decide whether the bounded threat extension is worth keeping as a lab-only,
  default-off diagnostic feature. The current evidence supports "narrow but
  real", not "ready to productize".
- If kept, document its boundary: immediate forced lines only, not shape
  creation.
- If discarded, keep the learning and remove the config/code path rather than
  carrying a dead toggle.
- Update this doc with the decision and next integration target.

Expected behavior change: baseline configs stay unchanged either way.

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
