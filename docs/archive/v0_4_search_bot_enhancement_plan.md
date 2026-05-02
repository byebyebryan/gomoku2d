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
- Commit 7 tactical scenario diagnostics is in progress.
- Next planning focus: bounded forced-line search.

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

The next likely improvement is not another shallow tactical toggle. It is a
bounded forced-line search path: spend extra depth only when the position is in
a forcing tactical branch. In Gomoku terms, the depth problem is often narrow
but deep. A plain depth-3 search can miss a lethal open-three/double-threat
sequence because it spends depth on unrelated replies, while a global depth
increase is too expensive. Forced-line search should make the search selectively
deeper where the opponent's useful replies are constrained.

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

- `max_threat_extension_depth`: small, likely 2-4
- reuse existing wall/CPU deadline checks
- no product-facing config until focused scenarios and tournament ablation show
  value

### Phase 9: Shape-Aware Eval Or Ordering

Defer this until forced-line behavior is measured.

If forced-line search solves the horizon issue, eval/order may only need to make
the existing search cheaper. If it does not, shape-aware eval can score richer
non-terminal positions using the Phase 6 analyzer labels.

Likely order:

1. Shape-aware ordering for pruning only.
2. Shape-aware eval if scenario failures remain at leaves.
3. Candidate expansion only with concrete sparse-position evidence.

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

Current working slice. No search behavior changes should be included here.

### Commit 8: Forced-Line Search Primitives

Includes:

- Internal helpers to classify immediate-win, forced-block, multi-threat, and
  creates-threat states.
- Focused unit tests for the forcing-state classifier.
- No integration with the normal search loop yet.

Expected behavior change: none.

### Commit 9: Bounded Threat Extension

Includes:

- Search integration behind internal experimental config or lab-only path.
- Extension caps for depth and existing wall/CPU deadlines.
- Scenario comparison against baseline.
- Small tournament ablation only if scenarios show improvement.

Expected behavior change: only for the experimental forced-line config.

### Commit 10: Decision And Cleanup

Includes:

- Keep the forced-line path only if it improves targeted scenarios without
  unacceptable tournament/runtime regression.
- Remove failed code paths rather than carrying dead toggles.
- Update this doc with the decision and next integration target.

Expected behavior change: depends on whether Commit 9 survives evaluation.

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
