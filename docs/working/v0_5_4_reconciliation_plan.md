# v0.5.4 Reconciliation Closeout Plan

Status: active.

Purpose: finish the repository and product reconciliation that was declared
complete too early at `v0.5.3`, while preserving that release as a valid public
alpha checkpoint.

## Context

`v0.5.3` was substantial work, not a failed release. It centralized report
artifacts, reduced stale documentation, refreshed public media and READMEs,
tightened release operations, and shipped from a fully validated commit.

The rushed decision was the phase boundary. A resumed review found meaningful
cleanup, ownership, test, tooling, and product-polish questions that should be
settled before the project starts the broader `v0.6` online design line.

This follow-up must remain bounded. "Clean everything" is not a release plan;
each finding needs a concrete contract, benefit, and validation path.

## Objectives

- Understand the current product and repository as a whole after the pause.
- Reduce clear ownership and maintenance friction without behavior churn.
- Remove dead paths and stale vocabulary, or document why retained fallbacks
  still exist.
- Make test and release cost intentional rather than inherited.
- Review the shipped product as a new player would experience it.
- Reconcile the public story and release-facing artifacts around the playable
  product before presenting the production experiment.
- End with a trustworthy baseline for fresh `v0.6` planning.

## Workstream 1: Baseline And Finding Register

Start read-only. Record findings before broad edits.

- Inventory crate, module, route, worker, persistence, report, script, and doc
  ownership.
- Inspect large files for mixed responsibilities; do not split files based on
  line count alone.
- Trace optional and fallback paths from configuration through real consumers.
- Profile test and scenario runtime before deleting or reorganizing coverage.
- Audit tracked/generated artifacts, ignored outputs, dependencies, CI, and
  release workflow duplication.
- Walk every public route on desktop and mobile, including loading, empty,
  failure, offline/no-config, and persisted-state behavior where applicable.

Classify every material finding as:

- `fix`: incorrect, stale, or unnecessarily costly;
- `refactor`: valid behavior with unclear ownership;
- `retain`: intentional complexity with a documented reason;
- `defer`: valuable work outside `0.5.4`, with the destination recorded.

## Workstream 2: Rust And Lab Ownership

Priority review targets:

- tournament report schema, aggregation, publishing, and tests currently
  concentrated in `gomoku-eval/src/report/mod.rs`;
- search orchestration versus state, candidates, evaluation, timing, metrics,
  and proof ownership;
- tactical detection versus rolling-frontier maintenance and scan parity;
- eval CLI dispatch, analysis batching, scenarios, and fixtures;
- analyzer tests and helpers that can move toward smaller behavior corpora.

Expected direction:

- split modules only where responsibilities and dependencies become clearer;
- preserve the scan implementation when it remains a useful correctness oracle
  or fallback, otherwise remove the full path rather than hiding it;
- keep corridor proof distinct from retired corridor-portal experiments;
- remove retired lab syntax instead of maintaining compatibility for
  unpublished experiment names;
- pair behavior-neutral refactors with corpora, parity checks, and benchmarks.

## Workstream 3: Web And Report Ownership

Priority review targets:

- split report loading, tabs, tables, drilldowns, board rendering, and analysis
  proof presentation where the current route modules mix those concerns;
- divide shared report styling by stable surface rather than one growing CSS
  module;
- consolidate duplicated report/static publishing scripts when one explicit
  artifact pipeline is clearer;
- review wasm bridge, worker, replay cache, local profile, cloud profile, and
  saved-match boundaries for duplicated schema translation;
- remove stale product vocabulary from fixtures and helpers when the old term
  is not part of a compatibility contract.

Do not redesign the UI merely to justify refactoring it. Product changes should
come from the walkthrough findings.

## Workstream 4: Tests, Dependencies, And Operations

- Measure the Rust workspace, tactical scenario, wasm, web, rules, and browser
  lanes separately.
- Consolidate narrow regression cases into durable scenario or contract
  coverage where possible.
- Keep slow tests when they protect unique behavior; move diagnostics out of
  hard gates when they are not pass/fail contracts.
- Review the open Cargo and GitHub Actions dependency updates as one controlled
  batch.
- Reconcile duplicated CI/deploy setup only when the shared mechanism remains
  easy to run and debug.
- Decide whether a small browser smoke belongs in automated release gates or
  remains a documented manual production check.
- Keep production dependency audit clean and document development-tool-only
  advisories separately.

## Workstream 5: Docs And Artifacts

- Keep the roadmap short and the active code/API references current.
- Move the parked process-story bundle out of `docs/working/`, or reduce it to
  one working entrypoint plus clearly archived source material.
- Check links, command examples, version language, terminology, and ownership
  maps after code changes.
- Keep curated reports and current public media; remove historical binaries
  only when they no longer provide useful evidence, not merely to improve a
  file-count metric.
- Regenerate reports only if bot, analyzer, report schema, or source tournament
  behavior changes.

## Workstream 6: Product Refinement

Run a fresh-player walkthrough rather than reviewing isolated screenshots:

- first visit and one-click play;
- Easy, Normal, Hard, and advanced bot settings;
- Freestyle and Renju play, hints, clocks, touch input, and match completion;
- replay entry, progressive analysis, navigation, evidence, branching, and
  cached return visits;
- local profile/history, cloud sign-in/sync, reset/delete, and failure states;
- Rules, Guide, Lab, Visuals, Privacy, Terms, and direct/deep links;
- desktop, narrow mobile, keyboard/focus, overflow, loading, and error states.

Fix concrete issues and copy drift. Avoid broad visual redesign unless a
systemic problem is demonstrated across multiple surfaces.

The public presentation pass should remain product-first:

- keep Home minimal and let completed games lead naturally into Replay
  Analysis;
- structure the root README around Play, Analyze, and the inspectable system;
- keep the agent-assisted process visible but secondary to the product;
- refresh social metadata and media only when the current artifact no longer
  represents the shipped surface.

## Sequence

1. Produce and prioritize the finding register.
2. Land low-risk dead-path, vocabulary, docs, and tooling cleanup.
3. Refactor Rust and web ownership in reviewable behavior-neutral commits.
4. Profile and reconcile tests, dependencies, CI, and release operations.
5. Run the complete product walkthrough and patch its findings.
6. Re-sync docs, refresh only affected artifacts/media, and run release gates.
7. Cut `v0.5.4`, then start `v0.6` from a new design checkpoint.

## Boundaries

Not part of `v0.5.4`:

- online play, matchmaking, trusted server authority, or public replay sharing;
- another broad bot-strength or corridor-search research phase;
- new analyzer strategy concepts or puzzle generation;
- a theme/skin system or broad product redesign;
- publishing the optional process story or devlog;
- breaking persisted player data without a separately reviewed reason.

## Release Bar

- The finding register has no unclassified material item.
- Refactors preserve behavior unless a documented defect is being fixed.
- Current fallbacks and compatibility paths have explicit reasons to exist.
- `docs/working/` contains active work rather than parked archives.
- Formatting, lint, Rust tests, tactical/lethal/Renju corpora, wasm build, web
  typecheck/tests, Firestore rules, production build, and browser smoke pass.
- Product walkthrough findings are fixed or explicitly deferred.
- Published reports remain valid and clean; regenerated artifacts carry clean
  provenance when regeneration was required.
- `v0.5.4` can honestly close the `0.5` reconciliation line.
