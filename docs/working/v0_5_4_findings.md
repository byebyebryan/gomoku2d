# v0.5.4 Reconciliation Findings

Status: active.

This register turns the v0.5.4 reconciliation plan into concrete, reviewable
work. A finding is complete only after its implementation and validation are
recorded here. The version remains open until a separate release-preparation
pass.

## Baseline

- Starting commit: `c79c67b` (`main`, clean, aligned with `origin/main`).
- `gomoku-analysis` tests: 43 passed in 59.06 seconds.
- `gomoku-eval` tests: 99 passed in 57.52 seconds.
- Current published reports remain the authority and are not regenerated for
  behavior-neutral refactors.

## Rust And Lab

| Finding | Class | Decision | Status |
|---|---|---|---|
| Tournament report schema, aggregation, provenance, and tests share one large module. | refactor | Split ownership behind the existing `gomoku_eval::report` facade without changing JSON. | planned |
| Analysis batch execution, publication projection, proof frames, and tests share one large module. | refactor | Split runner, publication, and proof-frame ownership without changing output. | planned |
| Eval CLI options, tournament planning, analysis planning, output, and dispatch are concentrated in one module. | refactor | Extract command-owned modules while preserving the CLI contract. | planned |
| Search safety, ordering, threat adapters, engine traversal, and tests remain concentrated in `search/mod.rs`. | refactor | Extract cohesive internal modules behind the current search API. | planned |
| Tactical types, scan view, replies, lethal logic, shape recognition, evidence, and tests share one module. | refactor | Split semantic layers while preserving tactical behavior and re-exports. | planned |
| Scan threat view remains reachable from diagnostics, shadow parity, and rolling fallbacks. | retain | Keep scan, rolling, and shadow modes; document scan as the correctness oracle and fallback. | confirmed |
| Corridor proof is still a product/lab configuration and report metric. | retain | Keep corridor proof distinct from retired corridor-portal and leaf-extension experiments. | confirmed |
| Parser tests enumerate unpublished experiment suffixes that are already intentionally unsupported. | fix | Retain generic unknown-suffix rejection instead of compatibility tests for each retired spelling. | complete |

## Tests

| Finding | Class | Decision | Status |
|---|---|---|---|
| Replay-analysis behavior is repeated across long one-off tests and report projection tests. | refactor | Replace repeated whole-replay executions with focused session, proof, and marker contracts; keep analyzer-specific expectations with their owners. | complete |
| Analysis and eval dominate local Rust test time. | fix | Consolidate duplicate executions, measure before/after, and preserve every unique behavior contract. | complete |
| Tactical, lethal, and Renju corpora protect distinct game contracts. | retain | Keep them as hard gates; do not trade correctness for suite speed. | confirmed |

## Web And Reports

| Finding | Class | Decision | Status |
|---|---|---|---|
| Lab and analysis routes mix loading, navigation, tables, drilldowns, board rendering, and help content. | refactor | Split stable components while preserving `/lab/`, query parameters, JSON endpoints, and appearance. | planned |
| Report presentation shares one oversized CSS module. | refactor | Split styles by report shell, tournament tables, and analysis proof boards. | planned |
| Bot and analysis report publishing use duplicate wrapper scripts. | fix | Replace them with one declarative report publisher and preserve output paths. | planned |
| Browser smoke previously caused expensive Playwright installation in CI. | retain | Keep browser smoke as a documented local release gate; do not restore browser downloads to CI. | confirmed |

## Docs, Dependencies, And Operations

| Finding | Class | Decision | Status |
|---|---|---|---|
| The parked process-story source bundle occupies `docs/working/`. | fix | Move it to an indexed archive without rewriting or publishing it. | planned |
| Cargo patch updates and GitHub Actions major updates are open. | fix | Apply Cargo and Actions updates as separate commits after structural work. | planned |
| CI and deploy duplicate some setup but remain readable and serve different purposes. | retain | Do not introduce a shared composite action in this loop. | confirmed |
| npm Dependabot cannot model the local Wasm package dependency safely. | retain | Keep npm updates manual and production audit clean. | confirmed |

## Product Walkthrough

| Finding | Class | Decision | Status |
|---|---|---|---|
| The shipped product has not had a fresh-player walkthrough after the pause. | fix | Exercise public routes, game settings, replay analysis, profiles, responsive layout, keyboard access, and failure states. | planned |
| Broad visual redesign would hide whether reconciliation preserved behavior. | defer | Limit v0.5.4 to demonstrated defects and copy drift; reconsider larger product work in v0.6 planning. | confirmed |

## Loop Boundaries

- No bot-strength tuning, analyzer semantics, online play, theme work, or
  process-story publication.
- No persisted-data, Wasm, CLI, report-schema, or public-route break.
- No version bump, report regeneration, push, tag, release, or deployment before
  the review checkpoint.

## Test Runtime Result

Warm-cache local measurements after consolidation:

- `gomoku-analysis`: 42 tests in 42.96 seconds, down from 59.06 seconds.
- `gomoku-eval`: 98 tests in 19.19 seconds, down from 57.52 seconds.
- Combined: 62.15 seconds, down from 116.58 seconds (46.7 percent).

The removed cases repeated expensive full-replay work already protected by
session parity, corridor proof, and report-marker contracts. Their replacements
test the decision boundary directly rather than preserving historical replay
IDs as implementation fixtures.
