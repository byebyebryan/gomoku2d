# v0.5.4 Reconciliation Findings

Status: completed by `v0.5.4` on 2026-07-23.

This register turned the v0.5.4 reconciliation plan into concrete, reviewable
work. A finding was complete only after its implementation and validation were
recorded here; this archived copy records the final closeout state.

## Baseline

- Starting commit: `c79c67b` (`main`, clean, aligned with `origin/main`).
- `gomoku-analysis` tests: 43 passed in 59.06 seconds.
- `gomoku-eval` tests: 99 passed in 57.52 seconds.
- Current published reports remain the authority and are not regenerated for
  behavior-neutral refactors.

## Rust And Lab

| Finding | Class | Decision | Status |
|---|---|---|---|
| Tournament report schema, aggregation, provenance, and tests share one large module. | refactor | Split ownership behind the existing `gomoku_eval::report` facade without changing JSON. | complete |
| Analysis batch execution, publication projection, proof frames, and tests share one large module. | refactor | Split runner, publication, and proof-frame ownership without changing output. | complete |
| Eval CLI options, tournament planning, analysis planning, output, and dispatch are concentrated in one module. | refactor | Extract command-owned modules while preserving the CLI contract. | complete |
| Search safety, ordering, threat adapters, engine traversal, and tests remain concentrated in `search/mod.rs`. | refactor | Extract cohesive internal modules behind the current search API. | complete |
| Tactical types, scan view, replies, lethal logic, shape recognition, evidence, and tests share one module. | refactor | Split semantic layers while preserving tactical behavior and re-exports. | complete |
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
| Lab and analysis routes mix loading, navigation, tables, drilldowns, board rendering, and help content. | refactor | Split stable components while preserving `/lab/`, query parameters, JSON endpoints, and appearance. | complete |
| Report presentation shares one oversized CSS module. | refactor | Give analysis proof frames their own style owner; retain coupled responsive table/shell rules together and remove retired selectors. | complete |
| Bot and analysis report publishing use duplicate wrapper scripts. | fix | Replace them with one declarative report publisher and preserve output paths. | complete |
| Browser smoke previously caused expensive Playwright installation in CI. | retain | Keep browser smoke as a documented local release gate; do not restore browser downloads to CI. | confirmed |
| A hard bot-worker failure recreated an unconfigured worker and could strand the current match. | fix | Reconfigure and replay one pending request after a worker crash; reject repeated failures rather than looping. | complete |
| Replay-analysis cache reads could throw when browser storage operations were blocked. | fix | Keep reads, writes, cleanup, and profile-reset clearing best-effort at the storage boundary. | complete |

## Docs, Dependencies, And Operations

| Finding | Class | Decision | Status |
|---|---|---|---|
| The parked process-story source bundle occupies `docs/working/`. | fix | Move it to an indexed archive without rewriting or publishing it. | complete |
| Cargo patch updates and GitHub Actions major updates are open. | fix | Apply Cargo and Actions updates as separate commits after structural work. | complete |
| CI and deploy duplicate some setup but remain readable and serve different purposes. | retain | Do not introduce a shared composite action in this loop. | confirmed |
| npm Dependabot cannot model the local Wasm package dependency safely. | retain | Keep npm updates manual and production audit clean. | confirmed |
| Tactical scenario docs invoke performance-sensitive D5 search in debug mode. | fix | Run tactical and lethal scenario commands with release binaries in active docs and runbooks. | complete |
| Rust dependency audit found a patched Crossbeam advisory and the direct unmaintained `instant` timing crate. | fix | Upgrade Crossbeam and replace `instant` with the maintained Wasm-compatible `web-time` crate. | complete |

## Product Walkthrough

| Finding | Class | Decision | Status |
|---|---|---|---|
| The shipped product has not had a fresh-player walkthrough after the pause. | fix | Exercise public routes, game settings, replay analysis, profiles, responsive layout, keyboard access, and failure states. | complete |
| A real Google sign-in and cloud-sync round trip requires live OAuth credentials and an interactive account. | defer | Keep Firestore rules and no-config fallback automated; repeat the live account flow during release review. | deferred |
| Broad visual redesign would hide whether reconciliation preserved behavior. | defer | Limit v0.5.4 to demonstrated defects and copy drift; reconsider larger product work in v0.6 planning. | confirmed |

## Public Presentation

| Finding | Class | Decision | Status |
|---|---|---|---|
| The root README gives the production experiment equal weight before establishing the playable product, then repeats capabilities across Highlights and Features. | fix | Lead with Play, Replay Analysis, and the inspectable Lab/Visuals system; keep the agent-assisted process as supporting context. | complete |
| Social metadata describes only a pixel-art board and Rust bot, omitting the product's strongest differentiator. | fix | Name tactical hints, configurable bots, and Replay Analysis while retaining the established title and social image. | complete |
| A new Profile leaves the Match History area empty without explaining how it becomes useful. | fix | Add one compact empty state with a Play action; do not add onboarding panels elsewhere. | complete |
| SPA navigation can leave stale document titles on Home, Match, Settings, and Profile. | fix | Give every route one shared title contract and cover the product flow in browser smoke. | complete |
| Lab and Visuals use document links for internal navigation and reload the application. | fix | Use router links while preserving routes and report/manifest state contracts. | complete |
| Profile reset omitted cached replay analyses, while the privacy page did not disclose all persisted settings and caches. | fix | Clear replay analyses with local profile data, preserve them for cloud-only deletion, and align the privacy and confirmation copy with actual storage behavior. | complete |
| Supporting copy drifts between player language, implementation notes, and generic tutorial prose. | fix | Preserve the chosen product vocabulary and home voice; rewrite only connective copy across Settings, Profile, Rules, Guide, Lab, and Visuals. | complete |
| The current README GIFs and social image already show the shipped gameplay, analyzer, Lab, and Visuals surfaces accurately. | retain | Keep the binaries unchanged; recapture only when a visible source surface changes. | confirmed |
| Replay Analysis is intentionally reached through a finished match or saved history rather than a bundled demo. | retain | Keep Home minimal and improve discovery through product flow, Guide copy, README structure, metadata, and media. | confirmed |

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

## Product Walkthrough Result

- The production build passed all 23 Playwright flows covering play, settings,
  profile/history, replay analysis and branching, reports, visuals, rules,
  guide, desktop layout, portrait layout, and touch input.
- Direct navigation across 10 public routes at desktop and mobile widths found
  no page errors, missing headings, failed responses, or horizontal overflow.
- Keyboard tab traversal reached every visible interactive control on nine
  public shell/report/settings surfaces.
- The walkthrough found one stale Playwright locator: profile history now uses
  `Inspect`, not the retired `Replay` action label. The shared helper was fixed;
  no shipped product defect required a UI change.
- Cloud-disabled behavior and Firestore authorization remain automated. A live
  Google sign-in/sync round trip stays as a manual release-review check.

## Public Presentation Result

- Desktop and portrait-mobile captures were compared for Home, Match, Settings,
  Profile, Rules, Guide, Lab, and Visuals. The established visual hierarchy held
  across the matrix; no systemic redesign problem was found.
- The current seeded Replay Analysis animation was reviewed through terminal,
  onset, setup-corridor, and last-escape frames.
- The Profile empty state was reviewed at `1440x1000` and `390x844`; it explains
  the next step without displacing the record summary or mobile controls.
- Existing README GIFs and the Open Graph image remain current. Their tracked
  files were left unchanged instead of producing equivalent binary outputs.
- Profile reset now clears cached replay analyses together with local games and
  settings. Cloud-only deletion still preserves local data, and the policy copy
  states both boundaries explicitly.
- The editorial pass retained the home line, compact match/replay status, and
  technical analysis terms. Helper copy now addresses the player directly,
  explains only non-obvious mechanics, and avoids stale report terminology.
- Curated bot and analysis JSON remain authoritative and unchanged because this
  pass did not alter bot, analyzer, report schema, or source tournament behavior.

## Validation Result

- Rust format and workspace Clippy passed; 340 workspace tests passed.
- Tactical hard gates passed `12/12`; lethal scenarios passed `9/9`; Renju
  fixtures passed `29/29`; the external Renju reference check completed.
- The Wasm package rebuilt successfully against the refreshed lockfile.
- Web typecheck, 302 unit tests, 23 Firestore rules tests, production build,
  production dependency audit, and all 23 browser tests passed.
- The 20-route responsive audit and nine-route keyboard audit passed.
- Curated bot and analysis report sources were not regenerated or changed.
- Rust and npm production dependency audits completed with no vulnerabilities
  or unmaintained runtime dependencies.
