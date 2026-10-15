# v0.5 Deep Review Evidence

This is the evidence ledger for the [active reconciliation plan](v0_5_reconciliation.md),
started 2026-10-15. Inspection is not certification. A mapped module remains
pending until its behavior, assumptions, consumers, and verification have a
recorded disposition. Generated dependencies are reviewed through their sources.

## Baselines

- Development and remote main: `ee2505c0541590027eb95c42de383217f77feef5`.
  Its [CI run](https://github.com/byebyebryan/gomoku2d/actions/runs/38816148039)
  passed. The expanded plan was the only outstanding work at loop entry.
- Public deployment remains the July `v0.5.4` release; the October repairs are
  development changes. No tag or deployment is part of this goal.
- Curated native tournament and analysis data retain their June provenance.
  They describe bounded native workloads, not current browser latency or
  general engine strength. The initial review contains the live configuration
  readback and artifact identities.
- Interactive walkthrough: production preview at `127.0.0.1:8001`, Chrome on the
  development host, normal viewport 1600 × 1345. No production account writes.

## Current Product Contract

The useful loop is immediate solo play, understandable tactical feedback, a
saved ending, inspection of the turning point, and trying an alternative.
Local identity/settings/history make returning useful without requiring login.
Optional Google/Firebase continuity is private client-uploaded data. It is not
ranked or independently verified evidence. Public Lab reports explain the
bounded model and provide inspectable examples; Visuals explains the authored
asset system. Future puzzles/sharing/online play are opportunities, not current
capabilities.

Retain this loop and the personal pixel presentation as the review's working
direction. The fresh-game walkthrough demonstrated that replay can identify a
concrete earlier defense and branch directly into it. An interface rewrite has
no demonstrated benefit. Prioritize making the existing loop legible, reliable,
responsive, and recoverable before adding more surfaces.

## Ownership and Coverage Inventory

Paths below are relative to the repository. Tests beside an owner belong to the
same review unit. Public routes are declared in `gomoku-web/src/app/App.tsx`.
The existing code overviews remain the detailed orientation maps.

| Unit | Sources / consumers | Contract and review target | Initial coverage |
|---|---|---|---|
| Shell / navigation | web `src/main.tsx`, `app/*`; publishing route script | Lazy routes, loading/error behavior, titles, navigation/focus, public paths | route map inspected |
| Home | `routes/HomeRoute*`, `app/links.ts` | Immediate play; access to setup/history/explanations | desktop walkthrough |
| Match | `routes/LocalMatchRoute*`, `game/*` | Move ownership, clocks, cancellation, undo, completion, round/setup semantics | fresh finish and branch observed; state trace started |
| Board model | `board/board_model.ts`, `game/types.ts` | Rust facts become presentation overlays without new rule logic | ownership inspected |
| Board rendering/input | `components/Board/*`, remaining `board/*` | Canvas/DOM alignment, pointer/touch/keyboard, animation and disposal | fresh keyboard play observed; lifecycle review pending |
| Local identity/history | `profile/local_profile_store*`, `profile_settings.ts`, `best_effort_storage.ts`, notice | Durable save, schema handling, retention, failure, reset and concurrent tabs | source and failure tests inspected |
| Active history | `profile/active_history*`, `use_active_history.ts`, `replay/use_replay_match.ts` | Identity, source/trust, deduplication and reset cutoffs | trace started |
| Saved record | `match/saved_match*`; local/cloud/replay consumers | Schema 2, compact cells, players/config/trust, structure versus legal game validation | source inspected |
| Auth / Firebase | `cloud/auth_store*`, `firebase*`, `CloudSessionController.tsx` | Optional config, session identity, listeners, account switches/reconnect | source trace started; actual OAuth pending |
| Cloud profile | `cloud/cloud_profile*`, promotion/sync files | Schema, ownership, cooldown, snapshot merge, stale completion and cross-device policy | source trace started |
| Cloud history | `cloud/cloud_history*`, `cloud_match*` | Pending/cache lifecycle, reset/delete coordination and account isolation | source trace started |
| Firestore policy | root rules/config and web rules tests | Private UID ownership, field limits/trust, cooldown/reset rules | prior emulator + live readback; deeper review pending |
| Bot bridge/config | web `core/bot_*`, `wasm_bridge.ts`; Rust wasm | Preset mapping, payload validation, worker queue/restart/error/resource ownership | source trace started |
| Replay conversion | web `replay/local_replay*`, `replay_analysis_core*`, winning-cell hook | Saved cells → legal core replay/exact hashes; frame/branch floor | fresh branch observed; source inspected |
| Analyzer runner/cache | `replay/replay_analysis_runner*`, worker, cache, hook | Progress/cancel/error/reuse; bounded valid persisted cache | source inspected; malformed cache and cancellation need probes |
| Replay presentation | `ReplayRoute*`, `replay_analysis_overlays*` | Confidence, useful decision, timeline, marker/keyboard explanations | fresh analyzed loss and confirmed alternative observed |
| Settings / Profile | `SettingsRoute*`, `ProfileRoute*` | Setup timing, selected controls, identity/history, destructive lifecycle | desktop walkthrough; async action guards need probes |
| Published reports | `reports/*`, `LabReportRoute*`, `AnalysisReportRoute.tsx`, `routes/lab-report/*` | Export schema → validator/type → lazy detail/proof/labels/provenance | source inspected; contract drift needs probes |
| Learning / assets / policies | Rules, Guide, AssetPreview, Policy routes; `ui/*`; assets/manifests | Rule/marker explanations, links, provenance, meaningful accessible content | previous initial walkthrough; full responsive pass pending |
| Core rules | Rust `gomoku-core/src/*` | Board/apply/undo/FEN/replay/hash and Renju forbidden model | pending deep review |
| Tactical facts | `gomoku-bot/src/tactical/*`, `frontier.rs`, `pattern.rs`, `viability.rs` | Threat/evidence correctness, rolling/scan parity, independent assumptions | pending deep review |
| Search / proof | bot `search/*`, `corridor.rs`, `lab_spec.rs`, `random.rs`, `lib.rs` | Candidate/safety/TT/apply-undo/budget/proof/preset semantics | pending deep review |
| Replay analysis model | `gomoku-analysis/src/*` | Onset/trace/failure/annotations/confidence, bounded proof and incremental session | pending deep review |
| Eval / native CLI | `gomoku-eval/src/*`, `gomoku-cli/src/main.rs` | Input parsing, arena/openings/budgets, summaries/export/provenance | pending deep review |
| Wasm exports | `gomoku-wasm/src/lib.rs` | Error/type/number translation, board/bot/analyzer lifetime | pending deep review |
| Fixtures / perf | `gomoku-lab-support/*`, corpora, external Renju material, benches | Independent references, hard versus diagnostic cases, workload representativeness | pending classification |
| Verification harness | colocated tests, web `playtests/*`, test configs, workflow CI | Meaningful assertions, real versus seeded journeys, browser matrix, cost | harness inspected; no browser projects currently configured |
| Build / operations | web scripts/config, root `scripts/*`, four workflows | Clean setup, reproducible artifacts/routes, release/deploy/recovery | prior maintenance; deeper review pending |
| Docs / artifacts / media | README, active refs, working/archive docs, reports/assets | Claims, commands, current versus historical provenance, generated boundaries | inventory linked; reconciliation pending |

All first-party web source modules and Rust source crates have an owner in this
map. At loop entry there were 125 tracked web source files, 80 Rust `src/*.rs`
files, four additional Rust benchmark/integration files, five web publishing/media
scripts, four root scripts, four workflows, and six browser-test/helper files.
Counts are an inventory check, not a review completion metric.

## Cross-boundary Traces

1. A move: Board input → `placeHumanMove` → Wasm `Board::apply_move` → snapshot,
   clocks and Rust threat facts → bot runner request → Rust search → guarded
   completion → same apply path. A finished position calls local recording.
2. A saved ending: compact schema 2 → local retained history/summaries → optional
   cloud promotion/snapshot under private UID → active-history selection → replay
   frames. The persisted schema validates structure; legal replay construction
   must validate game semantics separately.
3. Analysis: saved cells → Rust board replay with exact integer hash tokens →
   dedicated worker → incremental Rust session → step payload → annotations/cache
   → replay status/markers/timeline → branch seed → new local match. Cancellation
   messages are cooperative between steps; a long step deserves measurement.
4. Published evidence: native eval and analyzer → compact curated JSON and
   provenance → publishing script → report loader/validator → lazy row/proof
   rendering. Types, validators and the actual exported failure shape need to
   agree; a loader returning a typed object alone does not establish this.

## Fresh Journey Observation

Starting from Home, used keyboard activation to play black at A1, O15, A15, O1,
and A8 against Normal/Freestyle. The bot completed white's B3–F3 line in ten plies.
The ending was saved and Analyze opened it. Stepping backward identified the
open-four loss at ply 8 and a confirmed last escape at ply 6. Play From Here
created the six-ply branch with undo disabled at its floor. Playing B3 continued
the branch. Profile retained the original ending and reported one loss; Settings
remained reachable. This is real fresh runtime play, not a seeded fixture.

Observed refinement opportunities: raw search parameters dominate preset copy;
advanced controls are always exposed; match/replay markers have no nearby key;
readonly boards still announce placement instructions; round advancement lives
on the canvas while explicit keyboard controls expose only New Game. These are
concrete clarity/input questions, not evidence that the overall layout should be
replaced. Actual novice and screen-reader acceptance remain separate.

## Investigation Queue

| Priority | Question / contract | Evidence so far | Next acceptance |
|---|---|---|---|
| P1 | Does corrupt/newer local storage get silently overwritten? | Parser returns empty fallback; wrapper only catches I/O errors | Reproduce against real store; preserve unreadable bytes and visible session-only state |
| P1 | Can profile reset/delete completion affect a later account/session? | Route callbacks await stores then clear/reset/sign out without session guards | Deferred-operation route scenario with account/local-identity switch |
| P1 | Can malformed retained analyzer data crash replay? | Cache checks only shallow object shapes; consumers dereference nested facts | Perturb a valid cached result at consumed boundaries |
| P2 | Do report types/validators reflect native output? | Failure type says `ply`/candidate `role`; actual exporter uses `prefix_ply`/`roles`/`outcome`; optional consumed objects unvalidated | Curated artifact acceptance and mutated-payload rejection through real loaders |
| P2 | Does cancellation stop expensive work promptly? | Cooperative cancel cannot interrupt the current synchronous Wasm step | Browser workload + runner lifecycle probe; bound or terminate as justified |
| P2 | Does the local loop explain its controls and markers? | Fresh journey succeeds; guidance and round/advanced controls have discoverability gaps | Coherent small UI refinement after responsive walkthrough |

These are investigation priorities, not yet all confirmed defects. Rules/model,
browser cost, multi-tab/device behavior, and operational review continue after
the first contract repairs. Record each disposition before marking coverage done.
