# Systematic Code Review and Cleanup

Owner: Bryan. Baseline: `a0c32b2` on `main`. Scope: code ownership,
dependencies, duplication, lifecycle order, test reliability and maintenance
cost after the product reconciliation. Archived: selected repairs and cleanup
are complete; the [closeout](README.md) records the final runtime baseline.
This is a structural review of the
module boundaries and selected high-risk implementations, not a claim of
line-by-line formal verification or a new engine-strength evaluation.

## Decision

Retain the React/Phaser/Rust architecture and existing crate boundaries. Make
the bounded repairs below. The main opportunity is clearer ownership at the
history boundary and correct lifecycle ordering, rather than a framework or
engine replacement. Do not split files solely to meet a line-count target.

## Findings and Dispositions

| ID / priority | Finding | Result |
|---|---|---|
| C1 / P2 | Replay completion/error handlers called client callbacks before clearing the finished request. A callback starting another analysis cancelled the already-finished job and then lost the successor's callbacks. | Fixed: retire terminal state before callbacks; both completion/error reproductions failed before and pass after. Worker crashes already used the correct order. |
| C2 / P2 | Guest history imported retention, summary validation and archival statistics from `cloud_profile.ts`, which imports Firebase. Local code also needed a cloud-only conversion flag to avoid changing record trust. | Refactored: neutral `match/match_history.ts` owns shared history types, validation and aggregation. Local storage imports it directly. Cloud conversion stays in the cloud facade; existing cloud exports and stored schemas stay compatible. |
| C3 / P2 | `cloud_sync.ts` and `cloud_promotion.ts` independently constructed promoted histories, including reset filtering, retention, archive merging and trust conversion. Preflight also duplicated the profile-patch decision. | Refactored: both paths use `planPromotedMatchHistory`; preflight uses the same profile-patch decision as the write path. Snapshot/session guards remain at the orchestration boundary. |
| C4 / P2 | Three eval CLI commands used `create_dir_all(...).unwrap()` on the requested replay directory. A path occupied by a file caused a Rust panic (exit 101). | Fixed: shared output helper reports the path and filesystem error, using the existing controlled error exit (2). A valid directory is still created successfully. |
| C5 / P3 | The deep-linked report test asserted scrolling immediately after finding rendered content, although scrolling runs in `requestAnimationFrame`. The full suite reproduced the race. | Fixed: wait for the scheduled scroll assertion without weakening the expected behavior. |
| C6 / P3 | Static teaching diagrams still constructed a hidden 225-cell keyboard grid per board after becoming inert. The parent image already provides their text alternative. | Cleaned up: omit that unused semantic subtree for decorative boards; active match/replay boards keep their grids. The visual canvas and diagram labels stay in place. |
| C7 / P2 | Summary validation checked `opponent !== undefined` and then dereferenced it; a null opponent threw instead of rejecting the record during history hydration. | Fixed in the shared model: reject null opponents. The regression failed before and now rejects six malformed opponent forms without throwing. |

P2 here includes concrete maintenance hazards; C2/C3 are not claims that a
current player already lost data through those specific duplications.

## Coverage and Retained Design

| Area / inspected owners | Assessment | Disposition |
|---|---|---|
| App routing/recovery and static publishing | Explicit lazy route imports and route-specific loading copy are easy to follow. Publishing has small separate asset/report/SPA owners. | Retain; a generic route factory would save little and obscure loading/recovery context. |
| Saved matches, profile persistence and history | Saved-match schema is already shared. The history model was misplaced in a Firebase module. | C2; retain best-effort storage and durable-write conflict checks. |
| Auth/profile/history stores, setup restoration and promotion | Per-operation IDs, exact source snapshots and account-session checks protect different invalidation boundaries. Promotion history was duplicated. | C3; do not replace distinct guards with a generic async-store framework. |
| Match store/session/clocks/completion | Active-game state and selected next-game setup have different lifetimes. Session owns subscriptions and optional cloud follow-up; store owns board/compute state. | Retain; merging those lifetimes would undo recent correctness repairs. |
| Ready worker queue, bot runner and replay runner | Worker readiness/restart is already shared. Bot retry and progressive replay callbacks are different protocols. | C1; retain the common queue and separate runners. |
| Board model/scene/renderer/overlay logic | React input, Phaser rendering and pure visual mappings have useful owners. The import scan's scene/overlay loop is a type-only `BoardSceneState` reference. | Retain; no runtime import cycle found in that loop. Do not introduce a generic scene framework. |
| Profile/Settings/Replay and report UI | Several routes are large, but most code is route-specific presentation or selectors. Report panels already separate proof, results and explanatory content. | Retain; see conditional follow-ups below. |
| Wasm bridge, replay cache and published report validators | Nested validation is shared where runtime/cache contracts agree; historical published projections intentionally permit different fields. | Retain explicit boundary validation; avoid a global loose JSON validator or merging historical and live contracts. |
| Rust core and Renju | Rule/FEN/replay ownership remains in core; large board files include substantial inline tests. | Retain; this pass makes no rule-model changes. Earlier independent/corpus checks remain the relevant correctness evidence. |
| Rust tactical/frontier/search and analyzer | Existing search, tactical and analysis submodules have meaningful responsibility splits. Scan and rolling implementations provide independent parity evidence. | Retain; preserve independent references and bounded proof semantics. |
| Eval CLI, report aggregation and proof publication | CLI dispatch already separates options/output/analysis/tournament helpers; report aggregation has a large explicit telemetry mapping. | C4; retain report schema/provenance and bounded publication interfaces. |
| Tests, build and release operations | TypeScript unused checks, Clippy, scenario/corpus/backend tests and three-engine browser CI provide useful layers. A real scheduling race remained in a report test. Release tooling already refuses unrelated dirty work. | C5; retain the staged checks and release safeguards. |

## Conditional Follow-ups

These are maintenance options tied to concrete future changes, not work needed
to complete this pass:

1. **Search context:** `search/engine.rs` forwards many invariant arguments
   through root search and negamax and suppresses `too_many_arguments`. When
   adding another search policy, group immutable policy and mutable search
   resources into a small internal context. Preserve move order, deadlines,
   metrics and TT behavior with deterministic parity checks. A mechanical
   rewrite of the hot path today would add verification cost without repairing
   a demonstrated player failure.
2. **Report telemetry mapping:** `report/aggregate.rs` repeats a large field
   mapping in record/add/finish. Before adding more counters, consider a typed
   declaration/table that distinguishes sums, maxima and denominators. Do not
   silently alter historic JSON keys, units or averaging rules to shorten code.
3. **Profile presentation selectors:** if Profile gains another history mode,
   move its pure identity/result/statistics/copy helpers into a profile view
   model before adding more branches to the route. A single-consumer helper
   split currently offers limited benefit compared with the shared-history fix.

Small repeated predicates (`isObject`, nullable fields, result labels) do not
justify a new universal utilities layer. Archival process-story extraction and
historical artifacts were outside this runtime cleanup; no archival material
was deleted.

## Verification

- Two replay callback-order regression cases: failed before, passed after.
- Guest history isolation: records and reloads a legally finished nine-ply game
  while Firebase imports are set to fail; local-only trust is preserved.
- Temporary comparison against baseline `a0c32b2`: 64 history cases across
  sizes 0/1/127/128/129/1024/1025/1153, duplicate IDs, two identities, reset
  filtering and both trust-conversion modes. Replay and summary/archive outputs
  matched. Temporary baseline/test scaffolding was removed.
- Web typecheck, all 367 tests across 39 files, production build and 27 real-SDK
  Firestore emulator tests passed.
- Eval CLI reproduction: invalid replay directory exited 101 with a panic
  before; after the repair it exits 2 with a contextual error. A valid directory
  succeeds and is created.
- Rust fmt/Clippy and 21 eval CLI tests passed. All 56 production Chromium
  journeys passed before the final decorative-markup cleanup; affected teaching,
  keyboard and reduced-motion journeys were rechecked afterward. The configured
  CI runs the full native and three-engine browser gates on the pushed head.

The [manual acceptance deferrals](v0_5_review_evidence.md#unattended-acceptance-and-manual-check-deferral)
remain in force. This pass changes no persisted schema, Firestore rules, engine
policy, published historical reports, version or deployment.
