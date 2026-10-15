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

## Contract Repair Batch 1

The first three queue items reproduced against the development baseline:
four corrupt/newer-storage cases overwrote the original bytes, delayed reset
replaced the next local identity, delayed deletion called the next session's
sign-out, and two malformed nested cache cases were accepted. Eight regression
cases failed before the repairs.

Disposition: **fix**. Put local document parsing inside the best-effort boundary,
reject unsupported versions/root/identity shapes, and keep original storage
untouched after a read/validation failure. Guard foreground reset/delete
follow-up actions with the initiating auth object and, for local reset, local
identity. Share nested analyzer payload validation between Wasm and cache
readers; validate annotation roles, sides, coordinates, intervals, summary and
failure objects before presentation. The step envelope and native summary have
independent schema versions; current real-Wasm coverage exercises both.

Validation: typecheck including scripts/tools passed; 334 web tests across
35 files passed; production build passed; all 32 Chromium browser scenarios
passed (13.8 seconds), including denied/quota/corrupt/newer storage, live bot
recovery, keyboard finish/replay/branch, and published Lab confidence. No Rust,
Firestore schema/rules, curated report data, release tag, or deployment changed.
Actual account and physical-device acceptance remain outstanding.

## Published Report Contract Review

Confirmed and repaired drift between the native export and the analysis type:
failure uses `prefix_ply` and candidate `roles`/`outcome`, not `ply`/`role`.
The type now shares the analyzer failure projection, preserving optional fields
for older published projections. Missing confidence is presented as unconfirmed.

Nine mutated-artifact cases reproduced acceptance of malformed consumed fields:
failure notation, onset labels, reversed intervals, out-of-board cells, count
mismatch, duplicate tournament cells, provenance objects and inconsistent board
dimensions. Both loaders now reject these at their boundary; the two real curated
artifacts still load. Analysis publication is the existing implicit 15 × 15
format: removed the viewer's speculative board-size inference, which could not
recover a different board's encoding from the largest move cell.

A browser run exposed duplicate `h1` headings in the intermediate Lab loading
state. Loading/error panels now use a distinct secondary heading. Disposition:
**fix/refactor**, retaining the lazy report structure and historical data.
Typecheck and all 345 web tests (37 files) passed after this batch. Production
build and both affected published-report browser scenarios passed. Curated JSON
is unchanged.

## Core Rules and Corpus Review, First Boundary

Confirmed a FEN boundary defect: imports contain stones but intentionally have
no undo history; draw detection used history length. A nine-ply 3 × 3 regression
compares every played prefix with its FEN import through apply, hash and undo.
The final imported move incorrectly stayed ongoing. Draw detection now counts
occupied bitset cells. Invalid zero/impossible/overflowing FEN dimensions return
errors before allocation. FEN remains a position format: it does not reconstruct
past moves or infer historical wins from arbitrary imported diagrams.

Native fmt/Clippy and 342 workspace tests passed with the two new core
regressions. A subsequent symmetry test checks all 29 Renju reference labels
under all eight rotations/reflections, including actual application; all 232
probes passed. The actual rebuilt Wasm bridge also passed imported draw/undo and
invalid-dimension checks. Broader final gates follow the integrated candidate.

The [federation definitions and rules 9.1–9.3](https://gomoku.renju.net/rifrules/)
support the core's exact-five priority, unrestricted White overlines, and
recursive legal-continuation checks. The 23 external diagram labels previously
agreed with the separate Piskvork checker; symmetry broadens geometry coverage,
not rule completeness. The depth-four continuation guard remains an explicit
bounded approximation; no counterexample beyond that guard has been established
in this review. Retain it pending focused reference work, and do not claim full
tournament adjudication. Public teaching copy needs to distinguish shape
legality from a whole-position forced-win proof and describe the free opening.

The earlier 15 diagnostic misses are all non-forcing shape objectives under
simple-evaluation `search-d2/d3/d5`, not the web presets and not forced best-move
proofs. The four mandatory immediate-safety cases passed under each config.
Classify the misses as search/evaluation preferences or limitations; preserve
them as diagnostics rather than tuning the engine until it matches coordinate
lists. No demonstrated illegal move or immediate-safety failure was found there.

| Diagnostic objective | Misses across d2/d3/d5 | Observed alternatives |
|---|---:|---|
| Create closed four | 3 | I7 instead of K8 |
| Create broken four | 3 | G8 or H7 instead of J8/K8 |
| Create open three | 1 | d5 H7 instead of G8/J8 |
| Create closed three | 3 | H7 instead of J8 |
| Prevent closed-three upgrade | 2 | d3 G7 / d5 H7 instead of K8; upgrade is not an immediate win |
| Create broken three | 3 | G7 or H7 instead of I8/J8 |

Disposition: **fix** the demonstrated FEN boundary; **retain** the reference
corpus and diagnostic distinction; **defer** strength tuning to the later product
checkpoint. Continue reviewing tactics/proof limits, state/TT ownership and
analysis attribution; this section does not certify the whole engine.

### Directional Renju Counterexample

The subsequent source trace found a demonstrated false-forbidden outside the old
29 cases: four completion accepted an exact five in any direction. In the new
corpus position, H8 was incorrectly rejected as double-four because I8 completes
a vertical five while making a horizontal six. A horizontal four must complete
to five horizontally. The pinned Piskvork checker returned legal (`reason=0`),
matching the federation definitions and contradicting the old core.

Disposition: **fix**. Completion checks now carry their direction, including both
endpoints of a straight four. The new core test failed before the repair and
passed after it. The corpus now has 30 cases / 240 symmetry probes. Native
fmt/Clippy and 344 workspace tests passed; rebuilt Wasm has a corresponding
board/hint/application regression. A separate compatibility check replayed all
1,792 published tournament games legally under the repaired rules (0.41 s).
That check is retained because future rule changes must make compatibility
decisions explicit.

June report data remain historical reference evidence with recorded revisions;
their measured outcomes are not relabeled as current-engine benchmarks. The Lab
now says this near its tabs. Retaining them preserves experiment provenance;
fresh strength comparisons belong to the next deliberate evaluation baseline.
The correction does not justify copying new results over historical evidence.
