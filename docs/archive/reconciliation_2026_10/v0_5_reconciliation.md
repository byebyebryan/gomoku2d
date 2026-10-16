# v0.5 Deep Reconciliation and Refinement Plan

Archived review plan. Status: reconciliation review completed with manual
acceptance deferred at Bryan's request. Integrated verification and the unattended acceptance follow-up
are recorded in the [evidence ledger](v0_5_review_evidence.md). Owner: Bryan.
`v0.5.4` remains the July release checkpoint; the next patch version and release
decision are separate from completing this review.

The [closeout](README.md) records the final code-review and UI decisions.
The scope and execution instructions below describe the completed review;
they do not reopen its workstreams or adopt the broader visual proposals.

## Purpose and Review Stance

Give the whole project a fresh, critical review after the pause and several
generations of agent-assisted development. Establish which product decisions,
implementation contracts, abstractions, tests, and visual choices still earn
their place. Turn the findings into a coherent, polished version of the existing
product before choosing the next feature cycle.

Judge both older work and today's repairs by behavior, clarity, player benefit,
and maintenance cost. Recheck the evidence behind earlier closeouts. Treat tests
and docs as claims to examine alongside the implementation and actual use.

The current local-first product, retro presentation, React/Phaser/Rust split,
and lab-powered replay identity are starting hypotheses to evaluate. Retain or
revise each with a reason. A substantial simplification or design refinement is
in scope when it improves an existing product contract. For a large replacement,
compare the smallest repair with the proposed replacement and its migration,
dependency, performance, and validation costs before implementing it.

## Baseline and Existing Evidence

- Development baseline: `ee2505c`, including the first October repair batch.
- The [October review](project_review_2026_10_15.md) is the initial assessment;
  its findings do not exhaust this pass.
- The [July closeout](../v0_5_4_reconciliation_plan.md) and
  [findings](../v0_5_4_findings.md) explain prior decisions and validation.
- Current code ownership maps are in the [web overview](../../reference/app/code_overview.md)
  and [lab overview](../../reference/lab/code_overview.md). Verify them against code.
- Keep the deployed release, development head, curated report provenance, and
  new review evidence distinct. Reuse valid baseline checks; repeat them when
  a relevant change or unresolved concern warrants it.

The first repair batch and its validation are retained below. Its fixes,
including the new keyboard grid and cloud guards, receive the same deeper
review as the rest of the project.

The [deep review evidence ledger](v0_5_review_evidence.md) records the current
product/contract map, module ownership and coverage, fresh-game walkthrough,
investigation priorities, and subsequent measurement/repair evidence.

## Sequence and Checkpoints

| Stage | Work | Exit evidence |
|---|---|---|
| 1. Reconstruct | Revisit the product thesis, release history, current behavior, and ownership; walk the main player journeys | Current product/contract map, coverage inventory, initial findings, and assumptions worth challenging |
| 2. Review deeply | Trace high-risk flows end to end; examine Rust, web, persistence, reports, tests, performance, and UX | Reproductions, measurements, source-backed correctness checks, and classified findings across every workstream |
| 3. Choose refinements | Resolve findings into a coherent target for the existing product; compare structural and visual alternatives where needed | Prioritized repair/refinement batches with intended behavior, acceptance, and explicit deferred opportunities |
| 4. Implement and polish | Land small, reviewable batches; recheck the complete affected journey after each | Reviewed code, appropriate regression coverage, before/after evidence, and updated contracts |
| 5. Verify and reconcile | Review the integrated result, run release gates, check live acceptance, and reconcile docs/media | Whole-product acceptance record and an explicit release-readiness decision |

Review product behavior and code together. A blocking defect can be fixed once
its contract and reproduction are understood; broad structural or visual changes
should follow the synthesis checkpoint. Keep correctness fixes, structural
refactors, and visual changes distinguishable in commits.

## Review Workstreams

### 1. Product Coherence and Player Journeys

Revisit the [product strategy](../../reference/product/product_strategy.md): who
benefits from this game, why they return, and what the lab contributes to play.
Trace first visit, setup, a complete fresh game, the finish, analysis, branching,
and returning to saved history. Include both a novice and a returning player.

- Check discoverability, terminology, default difficulty/hints, and whether the
  next useful action is understandable without reading repository docs.
- Examine whether replay analysis helps someone make a different decision,
  including how uncertain findings are explained.
- Separate observed friction from taste preferences and future feature ideas.
  Identify product/taste decisions that need Bryan's judgment with concrete
  alternatives. Where feasible, include an actual new player's walkthrough;
  agent walkthroughs establish different evidence.

Output: a concise current-product contract, journey findings, and priorities
for refinement. Revisit Home, Settings complexity, and public Lab placement on
their merits rather than inheriting their earlier disposition.

### 2. Rules, Tactics, Bots, and Analyzer Correctness

Review `gomoku-core`, `gomoku-bot`, and `gomoku-analysis` from rule definitions
through player-visible explanations. Trace legality, win detection, apply/undo,
FEN/replay parsing, tactical facts, candidate filtering, corridor proof, lethal
onset, failure attribution, and confidence propagation.

- Check Renju recursion guards and unresolved behavior against the documented
  rule model, corpus provenance, and authoritative external references.
- Challenge shared assumptions in scan/rolling parity and existing fixtures;
  two implementations or tests can agree while sharing the same mistake.
- Examine pruning, caches/TT state, budgets, cancellation boundaries, and
  differences between native and Wasm execution.
- Classify the existing non-gating tactical misses. Establish whether each is a
  known model limit, fixture problem, correctness defect, or strength limitation.

Output: justified semantic contracts, counterexamples where found, and explicit
model limits. Correctness repairs are in scope; an open-ended strength/solver
research program remains a separate decision.

### 3. Architecture, State, and Data Contracts

Inventory first-party crates, modules, routes, stores, workers, schemas,
serializers, and publishing scripts. Record their owners, consumers, and review
coverage. Deeply trace the boundaries carrying state or game facts.

- Follow a move and a completed match through core, Wasm, worker, store, saved
  record, cloud snapshot, replay conversion, analyzer, report, and presentation.
- Compare exported JSON with actual consumers, TypeScript/Rust types, runtime
  validators, fixtures, and versioning. Look for parallel representations that
  have drifted or duplicated semantic decisions.
- Examine async ownership, request/session identity, cancellation, disposal,
  memory/resource lifetime, error propagation, and recovery after partial work.
- Review speculative abstractions, dead paths, config axes, hidden fallbacks,
  and mixed responsibilities. Simplify when a concrete consumer or maintenance
  benefit supports it; file size alone does not establish a problem.

Output: verified ownership and contract maps, targeted simplifications, and
regressions for broken boundaries. Review generated/vendor artifacts through
their source, reproducibility, and consumers rather than editing them directly.

### 4. Local Persistence and Cloud Continuity

Trace local identity, settings, replay/summary retention, promotion, private
cloud history, cooldowns, pending work, and account/profile lifecycle.

- Exercise sign-out/account switches, resets/deletion, delayed/out-of-order
  operations, repeated requests, multiple tabs/devices, and reconnects.
- Cover denied/full/corrupt storage, unavailable cloud configuration, network
  failure, partial sync, reload, and recovery. Recheck the recent memory fallback
  and its durable-save messaging.
- Verify private ownership and trust distinctions through client validation,
  Firestore rules, emulator scenarios, and read-only live configuration checks.
- Use disposable test data for destructive lifecycle checks. Perform a real
  account and cross-device round trip before claiming that level of acceptance.

Output: a persistence/lifecycle state matrix, reproducible races or data-loss
risks, and clearly separated emulator, deployed-config, and real-account evidence.

### 5. Interaction, Visual Refinement, and Accessibility

Review the complete app as an authored experience: Home, Match, Replay,
Settings, Profile, Rules, Guide, Lab, Visuals, and policy/navigation surfaces.

- Examine visual hierarchy, typography/readability, density, spacing, contrast,
  control affordances, board markers, animation, copy, loading, empty, pending,
  success, failure, and destructive-action states.
- Check desktop, small laptop, narrow/short portrait, touch, landscape,
  resizing, and browser zoom. Define the supported viewport/browser floor.
- Complete play and replay through keyboard input; examine focus order,
  announcements, disabled/selected state, coordinates, reduced motion, and an
  actual screen reader. Check that the DOM/canvas combination stays coherent.
- Compare current and proposed layouts at consistent viewports using the
  [screenshot procedure](../../working/ui_screenshot_review.md), then exercise the interactions.
  If a systemic design issue emerges, prototype one representative screen
  before widening the change.

Output: a prioritized UX critique, selected visual direction, and before/after
interaction evidence. Include refinement of good-but-unfinished surfaces, with
an explicit player benefit for each change.

### 6. Browser Performance and Resource Behavior

Measure the shipped web path on a documented desktop and representative lower-
powered device. Use CPU throttling as diagnostic evidence, keeping it distinct
from physical-device results and native tournament timings.

- Measure cold/warm startup, transferred assets, Wasm/worker startup, preset
  move latency and worst cases, and UI responsiveness while computation runs.
- Measure time to useful analysis, completion/progress, cancellation, cache
  behavior, and repeated match/replay/navigation memory growth.
- Set explicit usability budgets after establishing the baseline; record
  device, build, workload, latency distribution, and outliers. Optimize measured
  bottlenecks while preserving relevant correctness gates.

Output: a small reproducible browser workload set, supported-device expectations,
and an agreed response to expensive work: bound it, cancel it, communicate it,
or reduce its cost. Native self-play strength remains a separate measurement.

### 7. Test Quality and Verification Cost

Review whether coverage protects intended behavior and can fail for the right
reason. Audit assertions alongside the implementation, particularly tests that
were written from the same assumptions as the code under review.

- Cover cross-boundary and failure-state behavior; check cold and warm paths,
  no-config/offline operation, interrupted work, malformed inputs, and real
  completed-game flows alongside seeded replay fixtures.
- Use focused scenarios, independent references, and property/metamorphic or
  perturbation checks where they answer a concrete correctness question.
- Consolidate duplicated executions, brittle implementation assertions, and
  temporary scaffolding. Keep unique slow coverage and measure its cost.
- Validate promised browser engines; distinguish automated Chromium/Firefox/
  WebKit checks from real mobile-browser acceptance.

Output: a test/acceptance map and a practical local/CI/release command matrix.
Add tests for meaningful behavior, and use visual inspection for visual changes
that do not warrant an automated contract.

### 8. Repository, Documentation, and Release Operations

Check clean-checkout setup, generated Wasm/artifacts, dependencies/overrides,
workflows, release scripts, public routing, live configuration, rollback, and
operating-cost assumptions. Review current audit exceptions against upstream
evidence when doing the dependency lane.

- Reconcile code, active references, working notes, archives, README/public
  claims, versions, and runnable commands. Expose unresolved contracts instead
  of smoothing them over with prose.
- Trace curated report JSON through generation, provenance, schema, publication,
  and UI. Regenerate affected artifacts only when their behavior/data changes.
- Review media, asset provenance, dead/generated files, and ignored outputs;
  keep historical evidence where it remains useful.
- Check release/deploy ownership, production smoke, recovery, and the boundary
  between code publication and actual deployment/account acceptance.

Output: a reproducible contributor/release path, current contracts and public
claims, and explicit operating limits. A tagged release follows a readiness
decision rather than serving as a deadline for the review.

## Execution and Finding Discipline

Use this document as the single entrypoint. Keep concise decisions and findings
here; link detailed repros, measurements, or design comparisons only when needed.

For each material finding, record the affected contract and source, trigger or
observation, impact/priority, evidence level, disposition, intended change,
acceptance, and remaining uncertainty. Use `fix`, `refactor`, `refine`, `retain`,
or `defer`. Retained complexity needs a reason; deferred work needs a destination
and an explicit effect on release readiness. Avoid a quota of findings or edits.

Prioritize data/identity loss, incorrect game facts, and broken player flows;
then clarity, usability/performance, and maintenance friction. Tie visual polish
to a chosen experience rather than accumulating unrelated cosmetic edits.

Execute in bounded loops, each with a concrete question, named ownership/scope,
evidence to gather, a reviewable result, and a stopping condition. Update the
coverage and finding tables before widening scope. Keep unrelated concurrent
work intact and make validated changes in small commits under the current
direct-maintenance workflow. A useful alternate implementation can be explored
in isolation before replacing the working path.

Each implementation batch ends with an affected-journey check and a second
review of its assumptions, failure behavior, tests, and diff. Report automated,
browser-observed, deployed, physical-device, and real-user acceptance separately.

## Coverage Register

The October review produced the dispositions below. Detailed ownership traces,
reproductions, samples and integrated results live in the evidence ledger.
The manual checks below were subsequently deferred by Bryan; they no longer
hold this review open. Review completion does not authorize a release.

| Workstream | Deep review | Evidence and unresolved items |
|---|---|---|
| Product and journeys | reviewed; retain/refine | Fresh complete game, analyzed defense, branch and returning history; actual new-player acceptance deferred |
| Rules, bots, analyzer | reviewed; fix/retain limits | FEN, directional Renju, hash context and numeric boundaries fixed; independent corpus and historical replay legality checked; bounded proof/continuation limits retained |
| Architecture and contracts | reviewed; targeted repairs | Ownership map, consumed payload validators, exact replay integers, cancellable workers and route recovery; retain shared core and current framework split |
| Persistence and cloud | reviewed; fix lifecycle/conflicts | Corrupt storage, stale foreground actions/tabs, sign-in setup and transactional cloud writes covered; single local writing tab; real account/device acceptance deferred |
| UX, visuals, accessibility | reviewed; refine | Presets/disclosure, keyboard round/input, guidance, contrast and reduced motion; route/viewport probes and independent accessibility audits; actual zoom/screen reader/physical touch acceptance deferred |
| Browser performance | measured; bounded expectations | Production workload harness, desktop startup/preset/analysis and 30-cycle memory evidence; provisional diagnostic budgets; physical slow-device acceptance deferred |
| Test quality | reviewed; strengthen useful boundaries | Independent/metamorphic/perturbation/production-backend checks; all three browser engines in CI; timing profile stays opt-in |
| Repo, docs, operations | reviewed; reconcile | Manual maintenance, artifact provenance, active docs/links, release no-auto-staging and recovery; historical media retained until next release acceptance |

## First Execution Loop

Completed entry loop: product/contract reconstruction and a whole-journey critique.

1. Pin the development/deployment baselines and build the module/route/contract
   coverage inventory using the existing ownership maps.
2. Walk a fresh game through finish, replay, and branching, then settings,
   history, and the public explanatory surfaces on desktop and mobile.
3. Compare intended behavior, observed behavior, and the contracts/tests that
   claim to establish it. Include the recent fixes in that comparison.
4. Record a prioritized first finding set and the decisions it raises. Select
   the first deep-trace targets and refinement batches before broad editing.

Deliverable: a current product/contract map, populated coverage inventory, and
an evidence-backed execution order. Subsequent loops and their review outcomes are recorded in the evidence ledger.

## Initial Finding Register

| Finding | Class | Acceptance | Status |
|---|---|---|---|
| Stale cloud flushes can patch a different current profile | fix | Operation-specific results; account/reset/overlap regressions; guarded follow-up history actions | implemented; automated checks passed |
| Storage failures escape local profile actions | fix | Play, settings, and finished replays remain usable in memory; persistence failure is visible | implemented; denied/quota browser cases passed |
| Replay copy discards analyzer confidence | fix | Possible, confirmed, and unresolved outcomes retain their meaning in replay, reports, and Guide | implemented; unit and published-Lab browser checks passed |
| Bot failures leave an unexplained blocked turn | fix | Failed-turn state and explicit retry; cancellation/new-game guards remain correct | implemented; repeated-worker-crash recovery passed on desktop/mobile |
| Board input and selected settings lack accessible semantics | fix | Keyboard board interaction and semantic settings state, with browser verification | implemented; keyboard/browser checks passed; real screen-reader acceptance pending |
| Shared Rust semantics and React/Phaser ownership | retain | Recheck ownership, correctness oracles, and costs in the deeper review | reviewed and retained; targeted repairs and cleanup complete |
| New online product scope and open-ended solver/bot-strength research | defer | Fresh product/design checkpoint after reconciliation; demonstrated correctness fixes remain in scope | deferred |
| Real OAuth and cross-device cloud continuity | acceptance | Real-account evidence is required to claim that level of acceptance | explicitly deferred by Bryan; not passed |

## October Repair Batch

Cloud loads and promotions now return operation-specific results. Profile
patches require their source snapshot, and follow-up history actions check the
current session. The same protection covers the separate background-history
patch path found during implementation.

The profile and cloud-history cache share a best-effort storage boundary that
supports synchronous and asynchronous storage. After a failure, the store stays
in memory for the session; it does not overwrite data it could not read. The
local profile exposes a visible save-status notice outside its persisted state.

Replay uses E for confirmed and P for possible escapes; status/failure text,
published Lab decisions, and the Guide preserve that distinction. Bot failures
now expose an explicit retry while retaining the current position. A DOM grid
mirrors the canvas for keyboard navigation and cell descriptions, with one Tab
stop, visible focus, and placement through the existing game-store callback.

Validation on 2026-10-15:

- Web typecheck, including scripts and Playwright tools: passed.
- Web unit/contract/scenario tests: 326 passed across 35 files.
- Firestore emulator rules: 23 passed.
- Production build and all browser flows: 30 passed, including seven new
  reconciliation scenarios for keyboard completion, semantic settings, denied
  and quota storage failures, desktop/mobile worker recovery, and report confidence.
- Rendered keyboard-focus, storage-fallback, and desktop/mobile retry states
  inspected from the browser test screenshots.
- Curated report data, Rust behavior, and persisted schemas remain unchanged.

Batch-time status, superseded by the final readiness decision below:
no release tag or deployment was made. The phase remained open: real-account
continuity/reset checks and actual screen-reader acceptance have not been
substituted with automated evidence. Representative slow-device performance
remains a separate acceptance question from the native tournament measurements.

## Completion and Release Readiness

- Every workstream has a review result and coverage record. Material findings
  are resolved, deliberately retained, or deferred with their release impact.
- Product intent, core rules/tactical semantics, persisted data, runtime
  contracts, and player-facing explanations agree, including known limits.
- Critical correctness, identity/data-loss, and broken-flow findings are closed
  before release readiness is claimed.
- The full local loop is coherent and polished across the declared input,
  viewport, browser, and device support matrix, with uncertainty/recovery clear.
- Representative browser performance has measured expectations and an explicit
  disposition for workloads outside them.
- Appropriate Rust/corpus/Wasm/web/rules/build/browser/audit gates pass on the
  candidate; relevant real-account, screen-reader, and physical-device evidence
  is recorded separately. Explicitly deferred acceptance carries its uncertainty
  into the release decision without keeping the engineering review open.
- Current docs, public copy, reports, and media describe the resulting product.
- A final integrated review evaluates the experience and maintenance burden,
  with a concrete reason to close this pass beyond a green test suite.

The pass refines the existing product. Online implementation, public sharing,
puzzles, monetization, and open-ended engine research remain future product
choices. A larger redesign or replacement must earn its place through the
review above. Choose the next release and `v0.6` direction after reconciliation.


## Readiness Decision After Engineering Review

Close the reconciliation review with explicit acceptance deferrals. Bryan asked
to defer all manual checks and exercise whatever could run unattended. The
follow-up used installed Chrome through the browser extension, checked native
accessibility semantics, and added repeatable accessibility audits to the
browser suite. Its contrast and static-diagram findings were repaired.

Real Google OAuth/two-device continuity/reset, physical lower-powered/mobile
performance and touch comfort, actual browser zoom, screen-reader speech/use,
and novice/human visual feedback are deferred, not passed. Gradient contrast and
the illustrative icon's accessible-name review remain inconclusive automation
results. The [unattended acceptance record](v0_5_review_evidence.md#unattended-acceptance-and-manual-check-deferral)
records coverage and limits. These items do not block moving on from this review.

No new version, release tag, deployment, or production-account write is chosen
here. Choose a patch/version, refresh release media and make a release decision
with the deferrals visible; rerun affected automated gates on that candidate.
The existing corpora and bounded engine limits remain disclosed. A fresh strength
baseline and online/puzzle/sharing choices belong to the next product checkpoint.

## Subsequent Code and UI Review

The [systematic code review](code_review_2026_10_15.md) completed the selected
callback-order, shared-history, promotion, CLI-error, and diagram/test cleanup.
Bryan retained the existing UI and copy after reviewing the visual alternatives;
only the [Rules/Guide mobile refinement](v0_5_review_evidence.md#rules-and-guide-mobile-layout-follow-up)
was selected and implemented. Other mock proposals are not an accepted backlog.
The [closeout](README.md) records final verification and publication state.
