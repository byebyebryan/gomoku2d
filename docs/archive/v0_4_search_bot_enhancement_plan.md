# `v0.4.x` Search Bot Enhancement Plan

Status: active retrospective and follow-up plan. This started as the `0.4.0`
bot-lab work loop; it now records which experiments failed, which foundation
pieces landed, and what the next measured search slice should be.

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
- Commit 9 broad bounded threat extension was rejected after focused analysis.
  It found forced lines by scanning the whole board at every leaf, which reduced
  counted nodes on some cases but increased wall/CPU cost and lost match
  strength.
- Commit 10 local threat facts landed in
  `84ea128 feat(bot): add local threat facts`.
- Commit 11 broad shape eval was rejected after focused analysis. It fixed one
  depth-2 diagnostic but lost to plain deeper baselines and reduced effective
  search depth under CPU budgets.
- Commit 12 search metrics, pipeline reporting, and explicit pipeline stages
  landed across the bot/report refactor commits.
- The first depth-oriented optimization pass landed as behavior-preserving
  core/search work: Renju forbidden precheck, virtual immediate-win probes,
  trusted apply, candidate-generation cleanup, bitboard board storage, and
  occupied-stone hot-path iteration.
- Commit 13 pattern static eval adds a lab-only static-eval branch. Unlike the
  rejected recent-frontier local-threat eval, it is globally consistent and
  Renju-aware, but ablations show it is expensive enough to remain experimental.
- `0.4.0` is therefore a bot-lab foundation release, not a product bot-settings
  release. `0.4.1` should start from this measured baseline instead of chasing a
  single tactical fixture.

## Goal

Evolve the existing `SearchBot` into a measurable experimental bot without
forking a separate `AdvancedSearchBot` yet. The current baseline must remain
reproducible, and experimental features should only become exposed config after
they show value in focused tests.

This is not a solver project. The near-term product goal is a fast, efficient
practice bot with enough real knobs to support interesting gameplay, plus a
foundation for later reverse search, replay analysis, and puzzle generation.
Depth remains the primary strength lever: all else equal, deeper search is still
the most reliable way to improve play. The hard constraint is the per-move time
budget and Gomoku's broad candidate space.

The revised target is therefore depth-oriented:

- optimize the baseline so the same budget reaches more real depth
- use cheap tactical shortcuts only when they extend effective depth without
  broad leaf scans
- trade breadth for depth through narrower candidates and stronger move ordering
  while preserving cheap immediate-threat safety

The working hypothesis is that search depth is still the lever that discovers
long-horizon plans and winning combinations. Vanilla alpha-beta can theoretically
find those plans if it reaches enough depth. In practice, the current candidate
breadth is too wide for that to happen consistently under a per-move budget.
Local threat detection should therefore serve search efficiency: preserve
tactical safety while making the search look at fewer, better ordered moves.

That means tactical work should be judged by effective depth, not by whether it
looks smarter in isolation. A useful local-shape primitive should help at least
one of these stages:

- **must-keep safety:** never prune immediate wins, forced blocks, forcing gains,
  or concrete defenses
- **move ordering:** search the tactically meaningful moves first so alpha-beta
  cuts more work
- **candidate staging:** try a narrower set first while retaining tactical
  must-keep moves outside the normal radius/cap
- **selective extension:** spend extra depth only when local facts provide a
  concrete forcing branch and reply set

Frontier/incremental threat state is a likely optimization, but it should not be
the next behavior dependency. Today "cheap local threat detection" still often
means scan-derived facts around candidates or reply sets. A true frontier model
would maintain candidate masks and threat facts alongside apply/undo, updating
only the cells and line windows affected by the last move. That could reduce
repeated candidate generation and annotation cost, but it also couples threat
correctness, Renju legality, and search recursion more tightly.

Decision: keep `Board` as the authoritative state for now, and make the next
tactical annotation stage cache-friendly rather than fully incremental. The
frontier model should be a lab-only optimization after the local threat fact
schema and consumers are stable. The trigger for that experiment is evidence
that annotation, candidate regeneration, or reply-set rescans are a top cost
after ordering/candidate-staging work lands.

Threat-space search is relevant because it gives Gomoku-specific tactical
language, not because the `0.4.x` search line should become a full
TSS/proof-number solver.

That creates two lanes:

- **Practice bot:** must stay responsive, configurable, and tunable for play
  style. It should use tactical knowledge when it helps, but it should not turn
  into a rigid solver that only optimizes proof quality.
- **Analysis tooling:** can become more solver-like later if replay analysis,
  reverse search, or puzzle generation needs proof-oriented machinery.

## Product Bot Strategy

The next direction should be framed as a tactical ladder, not as "make one
fixture pass" and not as "build the strongest solver."

### Layer 1: Local tactical competence

The basic bot should handle single local threats correctly:

- take an immediate win
- block a single forced win
- complete or answer closed/broken fours
- recognize open threes and fours well enough to avoid obvious one-ply blunders

This is the minimum bar for a practice opponent. A bot that fails here feels
random, and human hint overlays become less trustworthy because the opponent
does not respect the same tactical language.

### Layer 2: Casual combo play

Assume both sides handle local threats correctly, but neither side has a deep
strategic plan. The game then becomes a casual PVE pattern: players probe,
extend lines, and eventually one side stumbles into a compound threat the other
side cannot fully answer. This is likely where the current product should feel
fun for a while, because it matches paper-Gomoku play better than solver-grade
perfection.

The bot should therefore become better at finding and not missing compound
threats, but without spending the whole move budget on proof machinery. Double
threats, four-over-three priority, and safe candidate narrowing matter more
than making depth 2 pass every quiet-shape diagnostic.

### Layer 3: Forced-chain steering

The advanced layer is not just "more local shapes." It is recognizing sequences
where one side makes forcing threats, expects the opponent to answer them, and
uses those forced replies to steer the board into a different, harder-to-block
threat.

This suggests a narrow forced-chain search:

- generate forcing gain moves from local threat facts
- derive concrete defense/cost replies from the created shape
- allow immediate counter-threat overrides when the defender can win or create a
  stronger forcing threat
- stop on win, fizzle, repeated/invalid line, or a small node/time budget

That is close to threat-space-search language, but it is intentionally not full
TSS yet. Full TSS requires dependency trees, rest-square conflicts, all-defenses
handling, and proof verification. Those are likely analysis features before
they are practice-bot features.

### Style knobs as budget allocation

Offensive and defensive styles should not be fake eval weights pasted on top of
the same search. The useful future distinction is budget allocation:

- **Offensive:** spend extra budget looking for own forcing chains and compound
  threats.
- **Defensive:** spend extra budget looking for opponent forcing chains and
  suppressing their setup moves.

That framing gives future UI knobs something real to expose. Until then, lab
presets should stay explicit search configs and should not be marketed as bot
personalities.

## Strategy Checkpoint

All behavior-changing experiments so far have failed their promotion gates:

- tactical candidates, tactical ordering, and tactical eval did not demonstrate
  enough value to keep as config surface
- broad bounded threat extension reduced counted alpha-beta nodes but made the
  bot slower because the hidden leaf-probe cost dominated
- broad shape eval fixed one depth-2 diagnostic but made deeper baselines weaker
  by spending too much work at every leaf

The shared failure mode is clear: each attempt added tactical work before proving
that the extra work increased effective depth or match strength. The next slice
must reverse that order. Start with measurement and baseline cost reduction, then
only add tactical shortcuts where the candidate set is already narrow or the
continuation moves are concrete.

Do not use `local_create_broken_three` as a pass/fail target. It remains useful
because it demonstrates why shallow search misses quiet shape-building moves,
but any fix that only makes depth 2 imitate depth 3 is not valuable unless it is
cheaper than reaching depth 3 normally.

## Design Direction

Keep one `SearchBot` implementation, but do not keep dead feature toggles in
`SearchBotConfig`. The shallow integration attempts and broad shape-eval pass
showed that passing one tactical fixture is the wrong goal. Tactical scenarios
are diagnostics; tournament strength, reached depth, budget stability, and
runtime explain whether a search change is worth keeping.

Decision: do not pivot the main bot to full threat-space search for the `0.4.x`
search line.
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
feeding tactical diagnostics, candidate ordering, and eventually small
forced-line modules. Static eval should only consume these facts if it can do so
locally or incrementally; broad leaf scans are explicitly out of scope.

Stable lab specs stay focused on reproducible baseline configs:

- `search-d3`
- `fast`
- `balanced`
- `deep`

Experimental suffixes can exist when they isolate a real pipeline axis.
`+pattern-eval` is the current example: it swaps only the static evaluator while
leaving candidate generation, safety, ordering, and child caps unchanged.

This avoids duplicating the search loop while still allowing tournament reports
to compare meaningful variants once a feature has earned a config surface.

The current split is clearer now:

- **Forced-line search** is useful once a branch has immediate tactical forcing
  states, but it must derive those facts from local shapes instead of scanning
  the whole board at every leaf.
- **Shape labels** are useful for explaining non-terminal shape creation:
  broken threes, open threes, and other moves that matter before immediate
  winning replies exist. They are not automatically worth scoring at every leaf.
- **TSS-style local threat facts** sit between those two ideas. They inspect the
  four lines through the last/candidate move and return concrete tactical facts:
  severity/type, gain square, cost/defense squares, rest squares, and whether the
  shape is forcing.

In Gomoku terms, the useful opportunity is not "make depth 2 solve every shape."
It is identifying narrow but deep tactical branches and spending extra depth
only there. If a shape shortcut costs enough to reduce effective depth globally,
it is worse than simply using the existing deeper baseline.

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
the opponent. That is too shallow: the baseline search/root safety probe already
handles many immediate tactical cases, while adding another leaf-eval branch
increased complexity without producing a clear strength gain.

Learning:

- Do not keep an immediate-win/block-only `tactical_eval` toggle.
- Eval work should wait until the analyzer can describe richer shapes:
  open three, open four, closed four, broken three, double threat, and forcing
  reply.
- The next commit should be behavior-neutral shape detection with focused unit
  tests, not another direct search integration.

### Phase 6: Tactical Shape Features

Add richer analyzer output without changing move choice.

Initial target features:

- open four
- closed four
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
- closed four
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
- nodes, root safety-gate probe nodes, time, depth reached, budget exhaustion

Run the baseline configs first: `search-d2`, `search-d3`, and `search-d5`.
Only positions that expose a real baseline gap should drive new search logic.

Current implementation note: `gomoku-eval tactical-scenarios` runs the focused
one-move diagnostics across search configs and can write JSON. The initial
seven-case smoke run showed `search-d2` failing the broken-three creation case,
while `search-d3` and `search-d5` passed that initial set. This is useful
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
  the remaining miss is `local_create_broken_three`, which creates a
  broken-three shape rather than an immediate forced line.
- It does reduce work on already-solved forced cases. In one sweep,
  `local_create_open_four` dropped from depth 2 / 234 nodes to depth 1 / 53
  nodes, and `combo_create_double_threat` dropped from depth 2 / 312 nodes to
  depth 1 / 83 nodes.
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

- move ordering and cheap tactical safety for the practice bot
- future static eval only if facts are available locally or incrementally
- scenario diagnostics and tournament explanations in bot lab
- future replay analysis and reverse-search/puzzle features

Target facts:

- terminal five / win now
- open four with two winning endpoints
- closed/broken four with one forced block
- open three with extension/block endpoints
- closed/broken three as non-forced shape facts for eval/order
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

### Phase 10: Depth-Oriented Optimization

Before adding another tactical consumer, optimize the normal search path and
measure whether the same budget reaches more depth.

Candidate targets:

- reduce legality and winner-check overhead in hot search paths
- make candidate generation cheaper or cacheable without changing behavior
- improve transposition-table reuse and move-order feedback
- add better instrumentation for non-node tactical/probe work so reports show
  the real cost, not just alpha-beta nodes

Promotion gate:

- improves reached depth, average move time, or tournament score for `search-d3`
  under the same CPU budget
- keeps immediate win/block and existing behavior scenarios green
- does not add a product-facing config knob unless the improvement is stable

### Phase 11: Narrower Search And Move Ordering

Only after the baseline is measured and optimized, try trading breadth for depth.

The useful target is not "make `search-d2` pass `local_create_broken_three`." The
target is a narrower search that reaches deeper under the same budget while
avoiding obvious tactical mistakes. Cheap threat detection should protect
immediate wins, immediate losses, and concrete forcing replies; move ordering
should put likely forcing or high-value moves first without scanning every leaf.

Candidate experiments:

- root/child ordering from local threat facts
- candidate caps or staged candidates, with forced tactical moves always retained
- shallow tactical safety filters that are cheap enough to run at root or near
  root, not at every quiet leaf
- optional forced extensions only when local facts provide concrete reply lists

Promotion gate:

- improves d3 tournament ablation or reached-depth metrics, not just one scenario
- records tactical-probe metrics if any non-node work remains significant
- remains lab-only until it has both scenario and tournament evidence

If an experiment starts needing dependency trees, rest-square conflict
resolution, or all-defenses proof handling, stop and split it out as an analysis
module instead of burying it inside `SearchBot`.

### Phase 12: Global Pattern Static Eval

Status: active lab experiment.

This phase tests whether a globally consistent tactical-ish leaf evaluator can
beat the crude contiguous-run evaluator without repeating the failed local eval
mistakes.

Design constraints:

- scan stable five-cell windows across the whole board
- avoid move simulation, apply/undo, or candidate generation inside static eval
- filter only relevant empty completion/extension squares through exact core
  legality, so Renju black forbidden completions are not overvalued
- keep the default evaluator unchanged

Early evidence:

- Tactical scenarios: `search-d3+pattern-eval` passed `10/16` versus D3
  `11/16`, averaging `71 ms` per scenario versus `15 ms` and no longer
  exhausting the `1000 ms` CPU budget after removing per-window allocation.
- D3 64-game head-to-head: pattern won `45-0-19`, but averaged `405 ms` per
  move versus `44 ms` and exhausted budget on `14.1%` of moves.
- D5 cap8 64-game head-to-head: pattern won `39-0-25`, averaged `258 ms`
  versus `176 ms`, and barely exhausted budget (`0.6%`).
- D7 cap8 64-game head-to-head: pattern was neutral at `32-0-32`, averaged
  `571 ms` versus `433 ms`, and exhausted budget on `39.3%` of moves.

Interpretation:

- The D3 and D5-cap8 match-strength signal is real enough to keep exploring.
- The D7-cap8 result says pattern eval does not automatically scale to deeper
  capped search; at that point the extra eval cost can erase the benefit.
- The cost is too high to make it a default.
- The next useful step is taxonomy/cost tuning: keep the global semantics, but
  reduce redundant window work or make high-severity windows dominate without
  overcounting weak two-stone patterns.

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
- Curated board tests for open/closed fours, open threes, broken threes, and
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
- Return gain, cost/defense, and rest squares for open fours, closed/broken
  fours, and open threes.
- Keep closed/broken three as non-forced facts.
- Focused tests for each shape's concrete moves.
- No search integration yet.

Expected behavior change: none.

Completed in `84ea128`. The helper remains private and behavior-neutral: it
returns concrete facts for terminal fives, open fours, closed/broken fours,
open/closed threes, and broken threes, but it does not affect candidate
generation, move ordering, static eval, or search depth until a later ablation
commit consumes it.

### Commit 11: Broad Shape Eval Experiment

Decision: discard.

Root ordering alone did not fix `search-d2` on `local_create_broken_three`, and
broad leaf shape eval fixed that one d2 diagnostic by scanning both players'
local candidate threats at evaluation leaves. That was the wrong target and the
wrong cost profile.

Measured result:

- `search-d2+shape-eval` fixed `local_create_broken_three`, but plain `search-d3`
  already fixed it cheaply.
- In 64-game Renju ablations at `1000 ms` CPU/move, `search-d2+shape-eval`
  beat `search-d2` but remained slower and weaker than deeper baselines.
- `search-d3` beat `search-d3+shape-eval` by `36.5-27.5` at `1000 ms` and
  `22.0-10.0` at `2000 ms`.
- `search-d3+shape-eval` reduced effective depth: about `1.88` reached depth at
  `1000 ms` and `2.03` at `2000 ms`, versus plain `search-d3` around `2.9`.

Learning: shape information is useful, but not as broad leaf scoring. The next
search behavior pass should optimize for effective depth under budget, then try
narrower candidate ordering/search only if it improves d3 tournament or reached
depth metrics.

Expected behavior change: none after cleanup.

### Commit 11b: Selective Frontier Local-Threat Eval Experiment

Decision: discard.

After rejecting broad local-threat leaf eval, a narrower experiment scored only
legal local-threat moves near the last four plies with radius one. The intent was
to test whether recent-frontier coverage could retain the useful tactical signal
without scanning every generated candidate at every static-eval leaf.

Measured result:

- Tactical sweep at `1000 ms` CPU/move: `search-d3` stayed `11/16`, while
  `search-d5+tactical-first+child-cap-8` improved from `11/16` to `12/16`.
- D3 16-game Renju ablation: the frontier eval variant beat baseline `11-0-5`,
  but average move time rose from roughly `45 ms` to `394 ms`, with budget hits.
- D5 cap8 16-game Renju ablation: baseline beat the frontier eval variant
  `9-1-6`.
- D7 cap8 64-game Renju ablation: the frontier eval variant was effectively
  neutral at `33.0-31.0`, but slower and more budget constrained.

Learning: local-threat board-value scoring has an awkward split. A global scan
has the right semantic coverage but costs too much at leaf nodes. A partial
recent-frontier score is cheaper, but it can overvalue the latest local fight
while ignoring older live threats elsewhere. Static eval must remain globally
consistent. Local threat facts should stay in move ordering, safety gates,
must-keep child caps, or narrow forced-line extensions until a whole-board
incremental threat model is justified by metrics.

Expected behavior change: none after cleanup.

### Commit 12: Depth-Oriented Search Improvement

Includes:

- Add enough instrumentation to explain where `search-d3` spends time under a
  CPU budget: eval calls, candidate generation, legality checks, root
  safety-gate probe work, average candidate count, reached depth, and budget
  exhaustion.
- Pick one behavior-neutral baseline optimization from that evidence before
  adding another tactical feature.
- Preserve cheap tactical safety for immediate wins/losses.
- Prefer changes that let `search-d3` reach more depth or spend less time under
  the same CPU budget.
- If tactical/probe work is added later, report it separately from alpha-beta
  nodes so hidden cost cannot masquerade as a node reduction.
- Run focused tactical sweep and d3 ablation before deciding whether to keep any
  behavior-changing experiment.

Expected behavior change: only for experimental lab specs until proven, unless
the change is a behavior-neutral optimization.

Progress:

- Added behavior-neutral `SearchMetrics` counters to `SearchBot` traces:
  eval calls, phase-split candidate generations, total/max candidate moves,
  phase-split Renju legality checks, illegal skips, TT hits/cutoffs, and beta
  cutoffs.
- Extended tactical scenario JSON/CLI output and tournament report JSON so
  future optimization runs can compare non-node work directly.
- Hardened the tactical scenario corpus with explicit roles, category-level
  semantic validation, and a dedicated board-print doc.
- Split tactical scenarios into `local_*`, `priority_*`, and `combo_*` layers.
  Local fixtures cover create/prevent/react/complete versions of the shape
  vocabulary. The taxonomy is intentionally asymmetric; open-four reaction is
  not a normal block fixture because one endpoint block still loses to the other
  endpoint.
- Kept Renju legality-only fixtures out of active tactical hard gates; future
  Renju tactical cases should test threat judgment around forbidden points, not
  simple "do not play illegal moves" behavior.
- Kept new tournament report metrics additive/defaulted so existing committed
  report JSON can still be rendered.
- Wrote ignored raw baseline reports under `gomoku-bot-lab/outputs/`.

Focused tactical baseline:

```sh
cargo run --release -p gomoku-eval -- tactical-scenarios \
  --bots search-d2,search-d3,search-d5 \
  --search-cpu-time-ms 1000 \
  --report-json outputs/tactical_baseline_search_metrics.json
```

- Baseline snapshot before the tactical-layer fixture expansion: `8` cases: `4`
  hard safety-gate cases and `4` diagnostics.
- `search-d2`: `6 / 8` passed; all hard safety gates passed, but it misses
  `priority_create_open_four_over_prevent_open_three` and
  `local_create_broken_three`.
- `search-d3`: `7 / 8` passed; all hard safety gates passed, but it still takes
  the conservative block in `priority_create_open_four_over_prevent_open_three`.
- `search-d5`: `7 / 8` passed; all hard safety gates passed, but it still takes
  the conservative block in `priority_create_open_four_over_prevent_open_three`.
- `priority_create_open_four_over_prevent_open_three` is intentionally
  diagnostic: it captures a real tactical gap where creating an open four should
  override blocking an open three, but it should not fail the current baseline
  safety gate.

Renju tournament baseline:

```sh
cargo run --release -p gomoku-eval -- tournament \
  --bots search-d2,search-d3,search-d5 \
  --games-per-pair 64 \
  --opening-plies 4 \
  --search-cpu-time-ms 1000 \
  --max-moves 120 \
  --seed 48 \
  --threads 22 \
  --report-json outputs/tournament_baseline_search_metrics.json
```

Wall clock: `168.49s`; shell-reported CPU utilization: `1875%`.

| Bot | W-D-L | Avg ms | Avg nodes | Avg depth | Budget hit | Avg eval | Avg candidate generations | Avg candidate moves | Avg legality checks |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `search-d5` | `81-1-46` | `862.91` | `137240` | `3.60` | `76.9%` | `120165` | `9317` | `93.9` | `70176` |
| `search-d3` | `80-5-43` | `290.00` | `27117` | `2.89` | `10.1%` | `20595` | `651` | `94.7` | `15327` |
| `search-d2` | `25-6-97` | `192.09` | `6361` | `1.99` | `11.4%` | `1032` | `140` | `92.8` | `3568` |

Immediate reading:

- Nominal depth `5` is only marginally ahead of depth `3` in this run, and
  `search-d3` split `32-0-32` against `search-d5`.
- Candidate width is consistently broad at roughly `93-95` moves per generated
  candidate set. The cost explosion is from repeated candidate generation,
  static evaluation, and Renju legality checks rather than a single unusually
  wide branch.
- The next optimization slice should be behavior-neutral and should target
  cheaper candidate generation / legality filtering / board scanning before any
  new tactical feature is added.

Root safety-gate ablation:

```sh
cargo run --release -p gomoku-eval -- tactical-scenarios \
  --bots search-d3,search-d3+no-safety,search-d5,search-d5+no-safety \
  --search-cpu-time-ms 1000 \
  --report-json outputs/tactical_safety_ablation.json

cargo run --release -p gomoku-eval -- tournament \
  --bots search-d3,search-d3+no-safety \
  --games-per-pair 64 \
  --opening-plies 4 \
  --search-cpu-time-ms 1000 \
  --max-moves 120 \
  --seed 65 \
  --threads 22 \
  --report-json outputs/tournament_safety_d3_ablation_64.json

cargo run --release -p gomoku-eval -- tournament \
  --bots search-d5,search-d5+no-safety \
  --games-per-pair 64 \
  --opening-plies 4 \
  --search-cpu-time-ms 1000 \
  --max-moves 120 \
  --seed 65 \
  --threads 22 \
  --report-json outputs/tournament_safety_d5_ablation_64.json
```

These measurements compare the default root safety gate with `+no-safety`,
which disables the root safety gate while keeping the candidate source and
legality gate unchanged. They predate the current tactical corpus revision that
keeps Renju legality-only cases out of active hard gates.

| Run | Bot | Result | Avg ms | Avg depth | Budget hit | Search nodes | Safety probe nodes |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Tactical | `search-d3` | `7 / 7` pass | `19.9` | n/a | `0` | `3999` | `2411` |
| Tactical | `search-d3+no-safety` | `6 / 7` pass | `15.4` | n/a | `0` | `5943` | `0` |
| Tactical | `search-d5` | `7 / 7` pass | `442.0` | n/a | `3` | `163623` | `2411` |
| Tactical | `search-d5+no-safety` | `6 / 7` pass | `435.4` | n/a | `3` | `165490` | `0` |
| D3 64-game | `search-d3` | `29-0-35` | `213.1` | `2.86` | `7.2%` | `16430` | `4657` |
| D3 64-game | `search-d3+no-safety` | `35-0-29` | `145.8` | `2.85` | `2.5%` | `29130` | `0` |
| D5 64-game | `search-d5` | `33-1-30` | `817.8` | `3.76` | `69.3%` | `146497` | `4804` |
| D5 64-game | `search-d5+no-safety` | `30-1-33` | `898.3` | `3.50` | `87.3%` | `191405` | `0` |

Immediate reading:

- No-safety is a useful control, not a clear replacement. It fails the
  earlier `block_open_three` tactical case, now named
  `priority_prevent_open_four_over_extend_three`.
- The then-current search-probe safety gate was buying real tactical safety and
  helped D5 preserve reached depth under the `1000 ms` CPU budget.
- The then-current search-probe safety gate could still be counterproductive
  for shallower D3 match play: it added about `4.7k` hidden nodes per searched
  move in the D3 ablation, while the no-safety variant won that 64-game
  head-to-head.
- The next likely target is a cheaper, more meaningful root filter: keep
  immediate win/block safety, avoid broad whole-board opponent rescans, and use
  local threat facts around the candidate move to reject only obvious blunders
  when a safe alternative exists.

Pipeline reset:

The root safety-gate ablation exposed a design problem: the earlier root
pipeline bundled candidate generation, Renju legality filtering, opponent reply
generation, tactical detection, and root candidate deletion. Treating that bundle
as the primary ablation made results muddy because it changed several dimensions
at once.

Use this per-move pipeline vocabulary going forward:

```text
board state
-> move source / candidate selection
-> rules legality gate
-> tactical annotation
-> optional safety gate
-> move ordering
-> alpha-beta search
-> static eval
```

Stage definitions:

- **Move source / candidate selection** controls breadth only. The useful lab
  axis is `near_all_r1`, `near_all_r2`, and `near_all_r3`: empty cells within
  radius `1`, `2`, or `3` of any existing stone. Future experiments may also
  try owner-aware sources such as `near_own_r2` or `near_opp_r2`. Whole-board
  move generation is conceptually pure but probably too far from a useful bot
  baseline, so keep it as a possible diagnostic, not the main baseline axis.
- **Rules legality gate** must be separate from candidate selection and tactical
  safety. Freestyle legality is just bounds, empty cell, and ongoing game. Renju
  forbidden legality is only relevant for Black.
- **Renju forbidden discovery** can be tighter than search candidate selection.
  Any black forbidden move must be within Chebyshev `r2` of an existing black
  stone. White stones can block patterns during the exact forbidden check, but
  white stones do not need to seed the possible-forbidden set.
- **Exact Renju forbidden check** may still inspect farther along the four board
  directions for overline, double-four, and double-three windows. The cheap part
  is deciding whether a candidate needs that exact check at all.
- **Tactical annotation** computes local threat facts into reusable move
  annotations without deleting moves by itself. The current stage is scan-based,
  records trace metrics, and is consumed by the local-threat safety gate.
- **Safety gate** is where a move may be removed. The promoted implementation is
  `opponent_reply_local_threat_probe`: it applies each root candidate, scans
  legal opponent replies, and classifies each reply through local threat facts
  instead of running a full immediate-winning-move scan after every reply. The
  older `opponent_reply_search_probe` remains available for comparison. Neither
  should be treated as the baseline candidate selector.
- **Move ordering** consumes tactical facts in the lab-only `tactical_first`
  mode to improve alpha-beta pruning without changing the legal candidate set.
- **Child frontier cap** is a lab-only post-ordering limit on how many non-root
  children alpha-beta searches. It does not change candidate discovery radius,
  and it does not cap root move choice after the root safety gate. In
  `tactical_first` mode, forcing tactical moves are kept even when they fall
  beyond the nominal cap.

Current SearchBot profile:

| Stage | Current name | Notes |
| --- | --- | --- |
| Candidate source | `near_all_r2` | Empty cells within radius 2 of any existing stone |
| Legality gate | `exact_rules` | Calls the rules engine; Renju black uses exact forbidden checks |
| Tactical annotator | `local_threat_annotation` | Scan-based move annotation for local threat facts; trace metrics split root/search annotation work |
| Safety gate | `opponent_reply_local_threat_probe` | Explicit `SafetyGate` config chooses `none`, `opponent_reply_search_probe`, or `opponent_reply_local_threat_probe` |
| Move ordering | `tt_first_board_order` default, `tactical_first` lab-only | Default keeps transposition-table move first, then stable generated order; lab mode ranks immediate wins, blocks, and local forcing shapes first |
| Child frontier | no cap default, `child_cap_N` lab-only | Optional non-root cap after ordering; root stays uncapped, and tactical ordering preserves must-keep tactical moves |
| Search | `alpha_beta_id` | Alpha-beta with iterative deepening and transposition table |
| Static eval | `line_shape_eval` | Scores open and half-open line runs |

Implication for the next implementation slice:

- Keep splitting the code and metrics around these stages so ablations isolate
  one dimension at a time. Candidate source, legality gate, and safety gate are
  now explicit code stages. Tactical annotation is explicit enough for the
  safety gate and can be reused by ordering/reporting without hiding scans
  inside eval leaves. Move ordering now has a default-off tactical lab mode for
  evaluation, and child frontier capping is available as the next breadth-vs-
  depth experiment without changing candidate radius.
- Keep the older product behavior available as `near_all_r2 + exact_rules +
  opponent_reply_search_probe`, but default to the cheaper local-threat safety
  gate after promotion.
- Add clean lab specs for each stage rather than using one bundled root-stage
  switch as the baseline. The current implemented suffixes are `+near-all-r1`,
  `+near-all-r2`, `+near-all-r3`, `+no-safety`,
  `+opponent-reply-search-probe`,
  `+opponent-reply-local-threat-probe`, `+tactical-first`, and `+child-cap-N`.
- Optimize Renju legality by exact-checking only black candidates within `r2` of
  black stones, regardless of whether search candidate selection uses `r1`,
  `r2`, or `r3`.
- After the pipeline split, compare candidate radius, legality cost, current
  safety probe, and any cheap local safety gate independently.

Initial `0.4.1` local safety-gate experiment:

- Added `+opponent-reply-local-threat-probe` as a default-off lab suffix, then
  promoted it as the default safety gate after focused tactical and tournament
  evidence.
- It still scans legal opponent replies at the root, but classifies each reply
  with local threat facts instead of running a full immediate-winning-move scan
  after every reply.
- In the 16-case tactical sweep, `search-d3` and
  `search-d3+opponent-reply-local-threat-probe` had the same pass/fail pattern:
  all hard safety gates passed, with the same diagnostic misses.
- In a 64-game Renju head-to-head at `1000 ms` CPU/move, seed `52`, opening
  plies `4`, max moves `120`, and `22` threads, the result was score-neutral:
  `32-0-32`. Average move time improved from `147.93 ms` to `77.54 ms`;
  average reached depth stayed `2.90`; budget hits improved from `0.9%` to
  `0.4%`.
- A d2/d3/d5 tactical sweep showed no differences in pass/fail, chosen move,
  reached depth, or budget exhaustion between the old and new safety gates.

## Revised `0.4.1` Action Plan

The next work should avoid another broad tactical integration. Use the tactical
ladder above to keep each slice honest.

1. **Realign tactical eval around the tactical ladder.**
   Keep the existing one-move expected-move checks, but make the report schema
   explicit about `role`, `layer`, `intent`, and optional `shape`. Summarize
   results by role/layer/intent so hard safety failures, local diagnostics,
   priority races, and combo probes are not collapsed into one flat pass/fail
   count.
2. **Finish the local safety-gate decision.**
   Completed: `opponent_reply_local_threat_probe` is promoted as the cheaper
   default safety gate. Keep `opponent_reply_search_probe` as an explicit lab
   suffix for comparison and regression investigation.
3. **Extract tactical annotation as a real pipeline stage.**
   Completed for local-threat facts: move annotations are reusable by safety,
   ordering, reports, and future forced-chain code. Do not hide future scans
   inside eval leaves. Keep the API cache-friendly: explicit inputs, stable fact
   structs, clear candidate/reply ownership, and metrics for annotation work.
4. **Try ordering before eval.**
   In progress: `+tactical-first` uses local facts to rank immediate
   completions, forced blocks, and forcing shapes ahead of quiet candidates.
   Promotion still requires better reached depth or tournament score, not only
   prettier tactical-scenario results.
   Initial evidence is mixed but worth continuing: a focused D3 tactical sweep
   kept all hard safety cases green and improved one diagnostic case
   (`local_create_broken_four`), but added annotation time. A 16-game D3 Renju
   head-to-head was score-neutral (`8-0-8`) with fewer alpha-beta nodes but
   slightly slower moves. An 8-game D5 Renju smoke was positive (`5-0-3`) with
   far fewer alpha-beta nodes, but high budget pressure means this is not yet
   promotion evidence.
5. **Try ordered child frontier caps before forced-chain search.**
   In progress: `+child-cap-N` keeps candidate radius fixed but limits the
   ordered non-root frontier that alpha-beta searches after move ordering. Root
   remains uncapped after safety filtering. This tests whether tactical ordering
   can provide useful deeper-node coverage with fewer searched children and
   better effective depth. Promotion requires tactical safety staying green plus
   better reached depth, runtime, or tournament score under the same CPU budget.
   Initial evidence before the root/child split: child caps without tactical
   ordering were reckless (`search-d5+child-cap-12` went `0-1-47` in a 4-bot
   focused tournament). That negative-control result also predates the later
   legality-before-cap cleanup, so rerun it before treating it as current
   evidence. Tactical ordering carried the cap:
   `search-d5+tactical-first+child-cap-8` matched uncapped D5's `12/16`
   tactical-scenario pass count while cutting average scenario time from about
   `472 ms` to `14 ms`, and all hard safety cases stayed green.
   After correcting the implementation so root stays uncapped and only non-root
   children are limited, the breadth-for-depth hypothesis became stronger. In a
   64-games-per-pair Renju tournament, `search-d5+tactical-first+child-cap-8`
   beat uncapped `search-d3` by `47-1-16`; `search-d7+tactical-first+child-cap-8`
   beat uncapped `search-d3` by `48-0-16`; and D7 cap8 beat D5 cap8 by
   `38-1-25`. A direct same-depth D5 head-to-head was the strongest evidence:
   `search-d5+tactical-first+child-cap-8` beat uncapped `search-d5` by
   `44-1-19`, while averaging about `13k` nodes and `304 ms` per move versus
   uncapped D5's about `309k` nodes and `797 ms` per move under the same
   `1000 ms` CPU budget. A follow-up four-bot run tried
   `search-d9+tactical-first+child-cap-4`: it beat D3 by `43-0-21` and D5 cap8
   by `38-0-26`, but lost to D7 cap8 by `26-1-37`. It reached deeper on average
   than D7 cap8 but likely cut too much breadth. Treat child caps as a promising
   depth/difficulty lab axis, with the next question being whether cap size or
   depth can compete with uncapped D5 under the same CPU budget.
   On the easy side, a tactical sweep showed that even `search-d1` passes all
   `4/4` hard safety-gate cases; it passed `13/16` total tactical cases, while
   D2 passed `12/16` and D3 passed `11/16`. A 64-games-per-pair Renju tournament
   across D1/D2/D3 still showed a clear strength ladder: D1 lost to D2 by
   `16-4-44`, D1 lost to D3 by `13-0-51`, and D2 lost to D3 by `11-2-51`.
   A later 64-games-per-pair Renju ladder run across D1, D3,
   `search-d5+tactical-first+child-cap-8`, and
   `search-d7+tactical-first+child-cap-8` initially confirmed the intended
   product shape, but also exposed a harness problem: random opening plies were
   whole-board legal moves, and opening seeds were tied to bot-pair indices. That
   made wide and focused tournaments compare different opening suites and allowed
   color-dominated random openings to hide strength differences.

   The harness now defaults to a shared `centered-suite` opening policy: 32
   deterministic 4-ply, center-local Renju-safe templates, each played with both
   color assignments in a 64-games-per-pair run. With that suite, the D1/D3/D5/D7
   ladder results were D1/D3 `3-0-61`, D1/D5 cap8 `3-0-61`, D1/D7 cap8 `3-0-61`,
   D3/D5 cap8 `23-0-41`, D3/D7 cap8 `15-0-49`, and D5 cap8/D7 cap8 `26-1-37`.
   D7 cap8 was the stronger hard-side bot in this suite, while D5 cap8 remained
   much cheaper. Treat D5 cap8 as the efficient hard bot and D7 cap8 as the slower
   hard-side variant.
6. **Prototype bounded forced-chain search as lab-only.**
   Start at root or near-root, only when local facts provide concrete gain and
   defense squares. Keep strict caps and record all non-alpha-beta work in
   traces. This is the first slice that can meaningfully support future
   offensive/defensive styles.
7. **Defer full incremental frontier state until metrics justify it.**
   A `SearchPosition` / `FrontierState` wrapper can maintain candidate masks,
   tactical annotations, and dirty cells around apply/undo, but only after the
   scan-based annotation semantics are stable. Run this as a scan-vs-frontier
   performance experiment if annotation or candidate regeneration remains a hot
   cost after ordering/staging work.
8. **Defer full TSS and product bot personalities.**
   If the prototype needs dependency trees, all-defenses proof, or rest-square
   conflict resolution to be correct, split it into analysis tooling instead of
   forcing it into `SearchBot`.

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
