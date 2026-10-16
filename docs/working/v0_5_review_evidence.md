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

## Interaction Refinement and Browser Coverage

Retain the existing visual direction. Refine the observed friction: preset
cards now explain their player intent; custom depth/width/scoring remain
available behind an explicit disclosure that opens when Custom is selected.
The finished game's header exposes Next round (swapping colors) through the
same input path as the canvas. Board instructions distinguish readonly and
finished states, and cell descriptions use readable overlay names. Match and
Replay link directly to the Guide. Renju copy explains exact-five priority,
valid directional continuations and the free opening without promising a
whole-position win from every forbidden shape.

Narrow portrait settings and disclosure expansion were inspected interactively
at 390 × 844. All 32 browser journeys passed in Chromium (14.9 s) and Firefox
(16.0 s), including keyboard next round and advanced disclosure. All 348 web
tests across 38 files passed. These remain agent/automation observations;
actual screen-reader and new-player acceptance are pending.

CI previously ran no browser journeys. The new lane runs the full suite on
Chromium/Firefox/WebKit, retains failure evidence, and reuses its preceding
production build. The web runner is pinned to Ubuntu 24.04 for supported browser
libraries. Local WebKit could not launch because this host lacks its required
ICU/XML/Flite libraries; CI must establish that engine's result. An earlier
all-engine attempt also had an expired local preview process; those connection
failures are infrastructure evidence, not app regressions. The two successful
engine runs used a freshly verified preview.


## Search and Exact Replay Boundaries

A supported 19 × 19 native board with a far-edge stone panicked in search:
SearchBot initialized a fixed 15 × 15 Zobrist table. It now sizes hashing from
the supplied board and invalidates transposition entries when board size,
win length, variant, or evaluation root side changes. A reused-versus-fresh
win-length probe also passes; it is an invariant check, not evidence of a
previously reproduced wrong score. Retain warm entries within one context.

The exact-u64 replay serializer replaced matching string values globally.
Player names equal to the seed or a move hash became numbers and failed native
deserialization. Two name cases failed before the repair; replacement now
matches complete numeric field tokens. Quoted field-like names are also covered
through the real Wasm analyzer. Cache revision 2 invalidates results predating
the rule correction; saved games remain intact. The cache revision must change
when rule/analyzer semantics change, even if the payload shape stays compatible.

Disposition: **fix/harden**. Native fmt/Clippy, all 347 workspace tests, and the
rebuilt production Wasm passed. The integrated web suite passed 358 tests across
38 files, including exact numeric names, old cache rejection and current Wasm.


## Browser Cost and Worker Ownership

An opt-in production profiling lane (`npm run profile:browser`) owns its preview
on port 8012 and records host/browser/build identity, resources, page CPU rate,
actual preset-worker samples and analysis/navigation memory. It stays outside
the default CI lane and has diagnostic time caps, not flaky timing assertions.
The workloads use legal published prefixes 4/12/20 from matches 65/577/1089,
and complete game 65 for analysis. They are examples, not a worst-case proof.

Measured on Ryzen 9 7900X / 24 logical CPUs, Chromium 156.0.8078.4, localhost
production preview, at 1280 × 900:

| Workload | Observation |
|---|---|
| Fresh Home / click to board state | 94 / 823 ms at normal page CPU; 324 / 513 ms at 6× page throttling |
| Warm Home / click to board state | 32 / 839 ms normal; 169 / 503 ms throttled |
| Local cold resources through board | 1,350,843 transferred / 3,040,912 decoded bytes; warm 7,800 / 649,617 |
| Bot worker ready | 5.8–8.7 ms |
| Easy first request, three positions | 9.6–10.9 ms |
| Normal first request | 93.6 / 483 / 228.3 ms |
| Hard first request | 238.4 / 2,250.3 / 370.8 ms |
| Main-thread 16 ms heartbeat during worker requests | maximum observed gap 16.0–16.3 ms |
| First analysis annotation / completion | 830 / 1,616 ms; repeat run 864 / 1,652 ms |
| Cached analysis, 30 SPA navigation cycles | 58–124 ms; zero workers/canvases after returning Home |
| Main JS heap after forced GC | 8.82 MB before; 10.48 / 11.07 / 11.35 / 11.59 MB after 5/10/20/30 cycles |

Single startup samples include automation overhead; the throttle comparison is
not evidence of a speedup or stable distribution. It applies to the page target,
not assumed worker throttling, and localhost is not a mobile network. Identical
position repeats benefit from warm transposition entries (Normal 5.5–8.9 ms,
Hard 5–191.6 ms); do not present them as new-position latency. Main heap growth
slows but these samples do not prove a plateau or measure worker/Wasm/GPU heap.
Physical lower-powered-device acceptance remains pending.

Disposition: **retain/refine** off-thread computation and lazy board loading.
Use provisional diagnostic expectations of Home/board within 2 s locally,
Normal new-position turns within 1 s, Hard within 3 s for this workload set,
and useful/full short-ending analysis within 2/5 s. These are investigation
triggers, not supported-device promises. Preserve visible thinking/progress and
usable cancellation; collect real-device distributions before release claims.

The lifecycle probes demonstrated two concrete resource issues. Repeated bot
crashes left three workers alive; the error handler now terminates the failed
thread before replacement (browser recovery leaves at most one). Replay cancel
could only queue a message behind a synchronous Wasm step; it now terminates
the active thread and creates another lazily for replacement work. Runner
regressions failed before the change; idle completed workers can still be reused.
The obsolete cooperative cancel protocol is removed. Disposition: **fix**.
All 33 Chromium journeys passed after the integrated repairs (14.4 s), including
touch, resize, analysis and repeated failure/retry.


## Local and Account Setup Lifecycle

Two stale-store cases reproduced durable data loss: after one tab recorded a
new ending or reset identity, a second tab saved its old snapshot over that
history/identity. The local persistence adapter now compares the last observed
raw document before writing. On a detected conflict, it preserves the newer
saved bytes, keeps this tab's subsequent work in memory, and displays a distinct
session-only notice. Both unit scenarios failed before; the two-page browser
scenario now passes. This is stale-writer detection, not an atomic multi-tab
merge; concurrent writing remains a single-active-tab contract. Do not reload a
session-only tab expecting unsynced edits to survive.

An account with Hard/Renju saved was overwritten by a fresh device's
Normal/Freestyle defaults during promotion. Establish the review's default
policy: restore saved cloud setup once per sign-in, then upload later local
changes. New cloud accounts initialize from the device setup. Restoration runs
before both controller and direct/background upload paths capture settings;
later cloud refreshes in the same session do not clobber local edits. The
failing sign-in reproduction and direct cold-load scenario now pass. Real OAuth
and cross-device round-trip acceptance remain pending.

The local session now owns profile/setup subscriptions. Resetting identity
terminates/disposes its old game so it cannot record into the replacement
profile. Saved setup changes stage the next game without changing a game in
progress; hint-only changes do not reset clocks or restart an empty board.
Three lifecycle cases failed before these repairs. This binding stays in the
lazy session module so cloud setup restoration does not load Wasm on Home.
Disposition: **fix/refactor**, retaining local-first identity and frozen current-
game settings. All 358 web tests and all 33 Chromium journeys passed together.


## Remaining Runtime Boundary Probes

A blocked lazy page download reproduced a blank app with no navigation. A route
error boundary now gives Reload and a full Home navigation; its browser scenario
failed before and passed after recovery was added. This handles a route/render
failure after the shell boots; it cannot recover a missing initial shell script
without the browser's own reload/network recovery.

Malformed retained cloud-history fields reproduced two crashes during profile
refresh. Hydration now validates cached/pending games and rebuilds pending sync
metadata, retaining valid pending records while ignoring malformed cache fields.
A null user entry already fell back safely and remains in the scenario table.

The analyzer's JSON options silently truncated values beyond wasm32 usize,
changing depth/scan semantics; two actual-Wasm cases failed before checked
conversion. A maximum native scan window also overflowed during construction;
it now uses saturating arithmetic. These are boundary repairs, not larger solver
budgets. Native fmt/Clippy, all 348 workspace tests, and RustSec audit passed.
The retained `proof_nodes` wire field counts principal-line steps rather than
visited search nodes; the progress label now says line steps. Do not use this
field as search-cost telemetry. Disposition: **fix/clarify**.

## Cloud Snapshot Conflicts

The five-minute rules cooldown alone did not prevent a stale device from
replacing newer cloud content after the gate opened. Two production-backend
emulator reproductions overwrote another device's setup or ignored a newer
reset barrier. Both history and promotion now compare the current normalized
snapshot content and reset barrier inside a Firestore transaction before
replacement. A conflict or deleted document prevents the write and surfaces a
reload/retry error; local history/pending records remain available. A concurrent
change during the transaction causes the SDK to retry the comparison.

A second-review perturbation reversed map key enumeration without changing
content. Both positive production write cases then failed, exposing a false
conflict in the first comparison. Snapshot serialization now sorts map keys
recursively while preserving array order; the same positive cases retain this
check. The comparison excludes approximate client write timestamps and provider
metadata; it includes all user settings/history/name fields a normal snapshot
can replace and the reset barrier. This is optimistic conflict rejection,
not live collaborative merging. Current clients still use explicit reads and
checkpoint sync, not always-on profile listeners. Older deployed clients do not
acquire this protection until replaced. No Firestore schema/rules deployment
was made. Disposition: **fix**, keeping real-account acceptance conditional.

All 27 emulator scenarios passed, including stale settings/reset rejection
through both production write paths and successful current-snapshot writes.
These exercise the actual Firebase SDK with owner-scoped rules, extending the
previous injected-backend unit coverage. A transaction adds a document read
(and potentially retry reads); update cost estimates accordingly.


## Integrated Experience and Review Dispositions

Retain the board-first pixel identity and local-play → useful ending → alternative
loop. The observed fresh game produced an actionable confirmed defense and a
working branch. Default setup remains Normal/Freestyle with assistance; setup
copy now explains intent before exposing experimental knobs. This is a coherent
small product, with Lab/Visuals supporting its identity. A full interface/engine
replacement has no demonstrated acceptance or cost benefit in this review.

The contrast trace found white labels at 1.39:1 on gold, 2.79:1 on green, 3.50:1
on teal and 3.73:1 on red. Colored controls now use #121212 ink while preserving
the fills; each pair exceeds 4.5:1. Dark-panel explanatory labels keep light ink.
This addresses [W3C's text contrast guidance](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html),
not a claim that the whole application is WCAG-certified. The board now honors
reduced motion for placement/removal, idle/pointer, warning, proof and replay
cues, using visible static frames and responding to preference changes. CSS
movement is reduced too. A real canvas test failed before and now stays pixel-
still for a full cue cycle, then animates again when the preference changes.

Interactive inspection covered desktop 1280 × 900, portrait 390 × 844 and
320 × 568, plus 812 × 375 short landscape. A temporary QA scenario checked all
ten main/public routes at 1280 × 720, 320 × 568, 640 × 450 and 812 × 375: 40
route/viewport combinations had one visible primary heading, no page-width
overflow and no runtime errors (28.6 s). The temporary scaffold was removed.
640 × 450 is a constrained layout probe, not a claim of actual browser zoom
acceptance. Portrait content scrolls instead of shrinking all controls onto one
screen. Current development [setup capture](assets/reconciliation-settings-2026-10-15.png)
records the selected visual direction. Actual zoom, screen-reader, touch-device
comfort and novice acceptance remain separate gates.

The native trace retains shared legality and tactical semantics below every
consumer. Scan/rolling tests protect optimization parity, while core/FEN
metamorphic checks, the independent Renju reference and legal replay corpus
challenge different assumptions. Corridor reply-width/depth cutoffs propagate
possible/unknown outcomes; confirmed escape means escape from the modeled
forcing corridor, not a global game-theoretic draw/win. Replay attribution
prefers immediate win/response, then prevention/escape and preserves confidence.
Retain this bounded model with its recorded limits; do not turn diagnostic
coordinate preferences into strength or best-move claims. Broad strength,
uncapped Renju continuation and general-solver research remain deferred.

The architecture keeps one rule core and typed payload boundaries. Session
subscriptions own live-match identity/setup, workers own cancellable compute,
and React owns accessible input and recovery. Splitting these into a replacement
framework, adding a query cache, or moving casual moves to a backend would add
migration and operational costs without repairing an observed player failure.
The existing owner map remains useful after the targeted repairs.

| Workstream / covered owners | Review disposition | Remaining limit or release effect |
|---|---|---|
| Product, Home, Match, saved endings, Replay, returning Profile | retain/refine existing complete loop | Actual new-player feedback before broad stranger-facing readiness |
| Core, tactical/frontier, search/corridor, analyzer, CLI/eval, Wasm, corpora | fix demonstrated rule/hash/number boundaries; retain bounded shared semantics | Depth-four Renju guard, modeled proof scope and diagnostic misses remain explicit; new strength baseline deferred |
| Shell/routes, board model/render/input, workers, reports/contracts | fix recovery/validation/resource ownership; retain React/Phaser/Rust split | Initial shell download still relies on browser network/reload; no formal whole-engine correctness claim |
| Identity/history, active-history, saved schema, auth/profile/promotion/history, Firestore | fix stale actions/writers and setup restoration; retain private client trust | One active local writing tab; no live collaborative merge; real OAuth and two-device round trip required |
| Settings/Profile, Replay presentation, Rules/Guide/Lab/Visuals/policies and UI/assets | refine intent, controls, labels, contrast and reduced motion | Actual screen reader, zoom, physical touch comfort and human visual acceptance pending |
| Browser resources/startup/search/analysis/cache | measure and fix demonstrated worker leaks/cancellation; retain lazy off-thread work | Sampled desktop budgets are diagnostic; physical lower-powered-device distributions required |
| Unit/scenario/corpus/Wasm/emulator/browser verification | keep independent/metamorphic/perturbation/fresh-flow checks; add three-engine CI | Automation is not real-account, physical-device or novice acceptance; keep opt-in profiling out of timing CI |
| Builds, dependencies, artifacts, media, docs and release/recovery | reconcile contracts/commands/provenance; refuse release auto-staging | No tag/deploy; old media/reports are labeled historical; final media and cloud acceptance before release |

## Operations Review

Verified current ownership and generation paths for Wasm, published report JSON,
visual manifests, static route entries, Pages 404 fallback, release notes/tags,
and separate Pages/Firestore deployment workflows. Public routes get copied
index entries; private dynamic replays use 404 fallback. Dependency locks remain
manual maintenance; production npm and RustSec audits passed. The five existing
Firebase CLI development advisories retain their documented upstream exception.

The release helper used to `git add -A` and commit unrelated work automatically.
It now requires a clean tree and pushed main, then only validates/tags. An
isolated fixture verified clean --check, dirty/untracked refusal without staging
or tags, and unpushed-commit refusal. No project tag was created. Release publication now marks hyphenated version
tags as prereleases, matching the helper's accepted prerelease syntax. Preview docs
now use a foreground, strict-port server instead of killing named sessions.
Recovery notes distinguish redeploying a prior web ref from restoring a recorded
Firestore ruleset pointer and require readback/behavior checks.

Active architecture docs incorrectly described current profile sync as push
subscriptions and committed v0.6 to online play. They now describe explicit
checkpoint reads/transactional writes and the later product checkpoint. Backend
cost estimates include transaction and preflight reads; official Firestore,
Firebase Auth and request-based Cloud Run quotas were rechecked on 2026-10-15.
This is a planning estimate, not billing/dashboard acceptance. All 131 active
relative documentation links resolved. July release media and June report
measurements stay historical; regenerate release media after version/UI
acceptance, and run new strength comparisons as a deliberate future baseline.


## Integrated Candidate Validation

- Native fmt/Clippy and 348 workspace tests passed (80.6 s for tests); RustSec
  audit passed against the current advisory database.
- The final current CLI run passed 12/12 mandatory tactical gates (21/36
  diagnostic hits, the same 15 classified misses), 9/9 lethal scenarios and
  30/30 Renju fixtures. Tactical run: 107.1 s.
- Rebuilt Wasm, web/tool typecheck, all 363 web tests in 38 files and production
  build passed. Production npm audit reports zero vulnerabilities.
- All 27 Firestore emulator tests passed, including actual SDK transactions
  under owner rules. Firestore rules/schema remain unchanged.
- All 36 local Chromium journeys passed after the preload repair (16.8 s), including page recovery,
  reduced-motion canvas stability/live preference change, keyboard completion,
  touch/resize, persistence failure/conflict, worker recovery and report proof
  confidence. CI runs these journeys in all three installed browser engines;
  verify its result against the exact pushed head before declaring publication
  complete.
- The 40 route/viewport probes passed. Profiling is opt-in and records the
  earlier measured bundle `a7771b19fb793893bbb3f31f9f0ac67464ae016f34327731f0bd971edb1303c4`
  (development work based on `346e83b`, dirty at measurement time); timings are
  not relabeled as measurements of every subsequent code revision.
- No release tag, deploy, production-account write or rules rollout was made.
  Real OAuth/two-device reset/settings/history, lower-powered physical-device,
  actual zoom, screen-reader and novice/human visual acceptance remain pending.

The engineering review has a result for each owner/workstream and closes its
reproduced high-priority failures. Keep the `0.5.x` phase open for the explicit
acceptance gates; choose the next patch/version and product direction afterward.


### Three-engine Recovery Finding

CI at `52626a9` passed 104/105 browser checks but WebKit could not retry a
blocked route chunk after returning Home. The trace showed the second page
never requested that JS URL again. This matches the cache poisoning described
in [WebKit 270357](https://bugs.webkit.org/show_bug.cgi?id=270357), whose fix
landed on 2026-10-10 and cannot be assumed present in shipped/tested browsers.
The error boundary itself worked; a plain reload did not clear the failed
modulepreload resource in that engine.

Disposition: **fix** the build behavior. Dynamic imports now fetch their own JS
graph without Vite-injected JS modulepreloads. CSS and initial HTML-entry
preloads remain. This avoids poisoned dynamic preload entries with a possible
network-waterfall tradeoff, rather than adding speculative import URL rewriting.
The unchanged recovery contract now probes both a route entry and a shared
Wasm bridge dependency. Both pass in the full 36-journey local Chromium suite;
typecheck/build passed. Verify both in all engines. Existing browser timing
samples predate this build choice; real-network/device performance remains a
release acceptance gate.


### Final Build Browser Profile

The [committed measurement JSON](assets/browser-profile-2026-10-15.json) records
all four profiling workloads after the WebKit build refinement: clean source
`c5bc3d2`, bundle SHA-256
`24ce131b55c9415c17ecbd5d127a8748b0fc66fb76ce128698fa09f13535b0b6`,
the same host/Chromium and local-preview conditions. All four scenarios passed
in 22.2 s. This is a current reproducible development sample, not a production
network or physical-device measurement.

| Final workload | Observation |
|---|---|
| Fresh / warm Home, normal CPU | 94 / 32 ms |
| Click to board state, normal CPU | 816 / 843 ms |
| Fresh / warm Home, 6× page CPU | 336 / 179 ms |
| Click to board state, 6× page CPU | 488 / 453 ms |
| Normal first worker requests | 100.5 / 505.2 / 232.8 ms |
| Hard first worker requests | 256.5 / 2,353.5 / 387.9 ms |
| Useful / complete short-ending analysis | 813 / 1,597 ms |
| Cached analysis, 30 cycles | 60–135 ms; zero workers/canvases after Home |
| Forced-GC page heap before / cycles 5/10/20/30 | 8.80 / 10.56 / 10.30 / 11.36 / 10.54 MB |

The samples remain within the provisional desktop investigation budgets. Heap
fluctuates within the later band rather than growing at every checkpoint; it
still excludes worker/Wasm/GPU heap. Identical-position TT repeats and page-only
CPU throttling retain the earlier interpretation limits. Dynamic preload changes
show no material local startup increase in this sample; test real-network/mobile
latency before making a broader performance promise.


## Unattended Acceptance and Manual Check Deferral

Bryan requested that all remaining manual checks be deferred, while exercising
whatever could run unattended. This decision supersedes earlier statements in
this ledger that manual acceptance must keep the reconciliation review open.
The review is complete with the deferrals below; a release remains a separate
decision. Deferred checks are not recorded as passed.

Installed Chrome connected through the ChatGPT browser extension and loaded the
local production build. Its native accessibility tree exposes the named Gomoku
grid, all 225 cells, keyboard instructions and a polite, atomic status region.
Keyboard placement triggered the normal bot reply (Move 2). This confirms browser
semantics and input behavior, not a real screen reader's speech or usability.

An independent axe-core 4.14.0 audit initially reproduced insufficient contrast
on Home links, match/profile labels, the selected preset description, Lab labels
and profile loss text. It also found focusable descendants inside static
Rules/Guide images. Repairs increase muted/loss text contrast and make teaching
boards inert under their existing text alternatives. Labeled diagram/visual
groups now have valid group roles. The obsolete test asserting the exact old
muted RGB value was removed; behavioral contrast checks replace it.

The final installed-Chrome audit covered ten routes at 1600 × 1000 and
390 × 844: **20 checks, zero reported violations and no page-width overflow**.
The [before/after JSON](assets/accessibility-audit-2026-10-15.json) records the
axe version, audited routes, viewport sizes, bundle identity and remaining
inconclusive checks. The browser suite now runs those 20 audits per engine and
attaches full violation/inconclusive details. It checks WCAG 2 A/AA, 2.1 A/AA
and best-practice rules; this is bounded automation, not WCAG certification.
The [Chrome setup capture](assets/acceptance-chrome-2026-10-15.jpg) records the
refined labels in their surrounding layout.

Typecheck, production build and production dependency audit passed. Local
Chromium and Firefox exercised all 56 journeys/audits per engine; two failures
were the obsolete exact-color assertion, and that affected case passed in both
after its removal. Local WebKit could not launch because this host lacks its
required ICU/XML/Flite libraries; the supported Ubuntu CI browser lane supplies
those dependencies and runs the full three-engine suite on the pushed head.

Remaining inconclusive findings include contrast over gradients and whether the
illustrative icon-only sample's visible glyph matches its accessible name.
They remain visible in audit output and deferred for visual/screen-reader
review. Static diagrams retain their descriptive labels; active match/replay
boards retain their navigable grids.

| Acceptance item | Unattended evidence | Current disposition |
|---|---|---|
| Real Google sign-in and two-device setup/history/reset continuity | Existing auth/session scenarios and real Firebase SDK transactions against Firestore emulator rules | Deferred; no designated disposable real-account round trip or production write performed |
| Physical lower-powered/mobile performance and touch comfort | Existing production profiling, page CPU throttling, responsive and simulated-touch journeys | Deferred; desktop simulation does not establish physical-device comfort, thermals or mobile network latency |
| Actual browser zoom | Existing constrained-layout probes; extension keyboard zoom attempt did not change viewport or device pixel ratio | Deferred; no actual browser-zoom result claimed |
| Screen-reader use | Installed Chrome's native grid/cell/status semantics, keyboard journeys, independent accessibility audits | Deferred; speech, reading order in real use and inconclusive checks remain unverified |
| New-player/human visual feedback | Earlier fresh game → analysis → branch walkthrough; browser captures | Deferred; agent observation is not novice/human acceptance |

No version/tag, deployment, Firestore rules rollout, or production-account write
was made. The next product checkpoint can proceed with these limits recorded.

## Rules and Guide Mobile Layout Follow-up

Bryan chose to retain the existing UI and copy after the visual proposal, and
selected only the Rules/Guide mobile layout work. Changes are confined to those
two route stylesheets at widths up to 720px: compact headers with visible
navigation labels, responsive diagram grids, and centered teaching boards that
stack when side-by-side examples would be too small. Desktop layout, page
content, fonts, colors, and the other application screens retain their existing
design.

The production build passed. The focused route and accessibility checks passed
in Chromium and Firefox (12 checks); local WebKit still cannot launch with this
host's missing system libraries. The supported Ubuntu CI lane supplies those
libraries and runs the existing full three-engine suite on the pushed head.

Installed Chrome checked both pages at 320×568, 360×640, 390×844, 430×932,
667×375, 720×480, 721×480, 768×1024, and 1280×900. All 18 layout checks showed
one navigation row, square diagrams, and no clipped links/diagrams or horizontal
page overflow. On phones the header is 42px tall; the smallest diagrams remain
226px wide at a 320px viewport and 260px wide at 360–430px.

Current captures: [Rules](assets/rules-mobile-2026-10-15.jpg) and
[Guide](assets/guide-mobile-2026-10-15.jpg). The broader proposal remains a
historical study, not an accepted implementation plan. Existing manual-check
deferrals remain in force; this follow-up makes no release or deployment.
