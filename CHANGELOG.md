# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project loosely follows [Semantic Versioning](https://semver.org/).
Each version leads with a short note on the design intent of the release; the
bullets underneath record the concrete changes. Where both the web game and
the Rust bot lab see changes in a release, the bullets are split under **Web**
and **Bot lab** sub-headers. Repo-wide changes (docs, tooling, CI) live in
their own section.

## [Unreleased]

## [0.5.4] - 2026-07-23

**Theme: close the `0.5` reconciliation line on a deliberately reviewed baseline.**

`0.5.4` finishes the repository and product reconciliation that remained after
the `0.5.3` public-alpha checkpoint. It does not introduce a new gameplay,
bot-strength, or analyzer model. Instead, it makes ownership clearer, removes
inherited maintenance friction, validates the complete product loop, and leaves
Gomoku2D in a trustworthy state for either a pause or fresh `0.6` planning.

### Web and public surfaces

- Split the Lab report surface into focused loading, table, drilldown, board,
  and proof-frame owners while preserving its routes, data contracts, and
  presentation.
- Consolidated report publishing, removed retired report styles, and kept the
  curated tournament and analysis data unchanged.
- Refined document titles, internal navigation, profile empty/reset states,
  privacy wording, public-page copy, and the product-first README structure.
- Made bot-worker crashes recover once without stranding the current match and
  made replay-analysis caching fully best-effort when browser storage fails.
- Re-ran the desktop, mobile, keyboard, direct-route, replay-analysis, and
  browser product walkthrough without finding a release-blocking regression.

### Bot lab

- Separated tournament reporting, analysis batches, CLI commands, search
  orchestration, and tactical semantics into smaller modules behind their
  existing public contracts.
- Consolidated repeated replay-analysis tests around durable behavior,
  reducing the measured analysis/eval test runtime by 46.7 percent while
  retaining tactical, lethal, and Renju correctness gates.
- Refreshed Rust dependencies, replaced the unmaintained browser timing crate,
  and cleared the Rust dependency security audit.

### Docs and operations

- Archived the parked process-story material and completed reconciliation
  plans so `docs/working/` contains only active notes.
- Synced architecture, testing, roadmap, release-history, and contributor
  entrypoints with current code ownership and product language.
- Refreshed GitHub Actions, retained browser smoke as a local release gate, and
  completed the full Rust, Wasm, web, Firestore, production-build, dependency,
  and Playwright validation matrix.
- Closed the `0.5` line as Gomoku2D's maintainable public-product baseline;
  future online work starts from a separate `0.6` design checkpoint.

## [0.5.3] - 2026-07-22

**Theme: package Gomoku2D as a clean public alpha.**

`0.5.3` establishes a public-alpha checkpoint in the reconciliation line. It
does not add a new gameplay or analyzer model; it makes the existing product,
lab, documentation, media, and release workflow coherent enough to ship
without making a devlog or other long-form presentation work a prerequisite.

### Web and public surfaces

- Moved Privacy and Terms into the SPA and added direct links into individual
  Lab analysis entries.
- Refined the Visuals surface and refreshed gameplay, replay-analysis, Lab,
  visual-guide, and social-preview media for the current product.
- Reworked the root, web, and bot-lab READMEs around the shipped product loop,
  current architecture, and practical contributor entry points.
- Refreshed the web lockfile within existing version ranges, removing all known
  production dependency advisories before release.

### Bot lab and reports

- Centralized curated report artifacts under `reports/lab/`, recorded clean
  generation provenance, and refreshed the published bot and analysis data.
- Simplified report plumbing and generated-artifact boundaries without changing
  the promoted bots, tactical semantics, or replay-analysis model.

### Docs and operations

- Reorganized reference, working, and archive documentation so current
  contracts no longer compete with historical plans and experiment notes.
- Preserved private process-story extraction and curation material as an
  optional future writing resource rather than a release gate.
- Simplified Playwright handling in CI and Pages deployment, tightened release
  runbooks, and disabled the unsupported npm Dependabot updater for the local
  generated wasm dependency.
- Established the public-alpha checkpoint before the remaining repository and
  product reconciliation now tracked for `0.5.4`.

## [0.5.2] - 2026-05-27

**Theme: make Gomoku2D explain itself from inside the app.**

`0.5.2` continues the public-readiness line by adding concise in-product rules
and strategy surfaces, then tightening the supporting Lab, Visuals, and Replay
Analysis copy. The release does not change bot strength or analyzer semantics;
it makes existing capabilities more discoverable and easier to understand.

### Web

- Added an in-app Rules page covering Freestyle, Renju restrictions, and why
  real double-three / double-four checks matter.
- Added an in-app Guide page with board-rendered examples for immediate
  threats, imminent threats, counter threats, combos, forced sequences, and
  replay analysis.
- Reused the live board renderer for Guide examples so public explanations
  match the game board.
- Renamed public support routes to `/lab/` and `/visuals/`, keeping the nav
  focused on Rules, Guide, Lab, and Visuals.
- Polished the Lab and Visuals pages so static/supporting surfaces match the
  current product shell.
- Renamed finished-game and history entry points around Replay Analysis:
  `Analyze`, `Inspect`, and `Replay Analysis`.

### Docs and repo

- Synced README, public docs, and app-reference docs with the Rules / Guide /
  Replay Analysis / Lab / Visuals product shape.
- Kept generated report data unchanged; no bot tournament or replay-analysis
  report regeneration was required.

## [0.5.1] - 2026-05-25

**Theme: turn static lab artifacts into first-class public surfaces.**

`0.5.1` continues the `0.5` public-readiness line by moving the report and
asset surfaces out of generated one-off HTML and into the web app itself. The
release does not change bot strength or analyzer semantics. Instead, it makes
the project’s strongest lab outputs easier to ship, inspect, and explain.

### Web

- Replaced generated bot-report and replay-analysis HTML with a unified React
  `/lab/` viewer backed by compact published JSON data.
- Kept compact `/bot-report/report.json` and `/analysis-report/report.json`
  data artifacts while making `/lab/` the canonical public report surface.
- Rebuilt the asset preview as a React Visuals page with sprites, icons, color,
  typography, and UI examples in the same product shell.
- Aligned report, Visuals, privacy, and terms pages more closely with the
  game UI so static/supporting pages no longer feel detached from the product.

### Bot lab and reports

- Trimmed published report artifacts away from large generated HTML and debug
  dumps toward compact data exports consumed by the web app.
- Updated curated report docs around the preset-triangle analysis flow used by
  the public lab report.
- Preserved generated report JSON as no-diff artifacts while moving visual
  presentation responsibility to the frontend.

### Docs and repo

- Updated public-facing README links and status copy around the Visuals page and
  canonical lab report.
- Cleaned up stale asset-preview naming in docs and build scripts after the
  Visuals replacement.
- Tightened release-copy details without rerunning the curated tournament data.

## [0.5.0] - 2026-05-22

**Theme: reconcile the `0.4` lab work into a public-release foundation.**

`0.5.0` starts the polish line by turning the large `0.4.x` bot/analyzer push
into a cleaner, safer, and easier-to-explain product base. The release does not
try to add another major strategy concept. Instead, it tightens the web runtime,
clarifies the Rust lab architecture, refreshes the curated reports, and
organizes the docs around what is now worth showing publicly.

### Web

- Added a bounded transposition-table cap for browser search bots so stronger
  presets stay more predictable in long sessions.
- Added local replay-analysis result caching so repeat visits to analyzed
  replays do not redo the same wasm analysis work.
- Cleaned up the wasm bridge and frontend board/replay APIs so route code talks
  to typed app contracts instead of raw wasm payloads or scene internals.
- Refactored frontend lifecycle and board/replay plumbing after the `0.4.x`
  UI work, including safer no-config paths and tighter smoke coverage for play,
  replay, reports, and release builds.

### Bot lab and reports

- Split the search bot, tactical logic, replay-analysis bridge, eval CLI, report
  plumbing, and analysis-batch code into clearer module boundaries without
  changing the promoted anchor set.
- Kept lab search configs independent from the web memory cap so tournament and
  report runs remain controlled by lab-side specs.
- Added guards around curated report artifacts and progress output for analysis
  report generation, then refreshed the bot and analysis reports as the `0.5.0`
  baseline.
- Trimmed brittle/case-specific tests and moved coverage toward behavior
  conditions, corpora, and release smoke checks.

### Docs and repo

- Reorganized the docs by audience so public-facing context, reference material,
  working notes, runbooks, and archives are easier to navigate.
- Added web and bot-lab code/API overview maps to make the main app contracts,
  wasm boundary, bot pipeline, replay analyzer, and report stack easier to find.
- Polished stale roadmap and cleanup references around the `0.4` handoff into
  the `0.5` public-release/presentation line.
- Captured the project-level framing for `0.5`: clean the repo, make reports
  feel less like hidden dev artifacts, and prepare the strongest parts of the
  project for first public presentation.

## [0.4.8] - 2026-05-21

**Theme: wrap the `0.4` lab-powered line with mistake-aware replay analysis.**

`0.4.8` is the closing release for the `0.4` line. The earlier `0.4.x`
releases built the bot lab, rolling threat model, configurable bot surface, and
browser-side corridor analyzer. This release uses that foundation to make replay
analysis more human-facing: not just where the final forced line exists, but
what kind of losing-side failure led into it.

### Replay analysis

- Added compact failure classification on top of setup-corridor and lethal-onset
  evidence, including missed immediate wins, missed four replies, missed
  forcing-three replies, missed lethal prevention, missed setup-corridor escape,
  and unclear boundaries.
- Tightened corridor candidate handling around multi-threat positions so
  immediate threats, forcing-three replies, counter-threats, actual moves, and
  forbidden moves are shown and probed consistently in stepped UI analysis and
  static reports.
- Promoted lethal-onset shape data through the analysis bridge so reports and
  Replay status can describe common endings as open-four, 4+3, 4+4, or 3+3
  style failures instead of only showing raw proof-state labels.
- Refreshed the curated analysis report with failure steps, missed candidates,
  onset evidence, and cleaner proof marker styling.

### Web

- Updated Replay status copy so terminal, onset, last-escape, and failure frames
  read from the current player's perspective without overloading the board with
  extra controls.
- Added configurable evidence overlays that highlight the stones forming
  immediate, imminent, counter-threat, and winning-line annotations, using the
  same hint vocabulary as the analysis report.
- Reserved hover for "about to play here" targets and moved winning-line /
  evidence emphasis onto highlighter overlays.
- Tuned marker/highlighter styling, hover animation speed, and board hint
  weights so dense adjacent annotations stay readable.

### Bot lab, reports, and docs

- Extended tactical detection for combined non-forcing-three cases that together
  become actionable imminent threats.
- Cleaned up bot report tables by removing low-value Pool/Renju visible columns,
  replacing Pairwise best/worst with score, W-D-L, and shuffled Elo, and showing
  score versus each opponent in first-level pairwise expansion rows.
- Updated analyzer, tactic, visual, and roadmap docs around failure modes,
  evidence overlays, and the `0.4` line handoff to `0.5` presentation polish.

## [0.4.7] - 2026-05-19

**Theme: mark lethal-threat onset and harden Renju rule correctness.**

`0.4.7` started as the lethal-threat pass: mark the point where a replay becomes
effectively lost, so the analyzer can explain the setup corridor instead of
spending attention on the obvious final conversion. That forced combo threats
and Renju forbidden moves into the same model. The result is a hardening release:
the analyzer now has a clearer endpoint, and the rule engine has a more reliable
Renju legality foundation.

### Rules and analysis correctness

- Added lethal-threat detection for terminal coverage and one-step combo/fork
  threats, including open fours, four-three coverage, double-three coverage,
  shared-block rejects, and Renju forbidden-block cases.
- Split replay explanation into setup corridor, lethal onset, and lethal tail so
  the UI/report can show how the loser reached an already-lethal state instead
  of treating every final conversion ply as equally meaningful.
- Replaced the old shape-only Renju forbidden shortcut with a recursive
  legality-aware checker, then added extracted Renju.net advanced examples,
  handwritten fixtures, and external-reference validation notes.
- Added metrics and prefilters for the corrected Renju checker so the legality
  fix remains practical for bot search, hints, tournament reports, and replay
  analysis.
- Refreshed the curated bot and analysis reports after the Renju-rule and
  lethal-onset changes.

### Web

- Refined Replay status copy around lethal onset: terminal frames read as a won
  state, post-onset loser frames read as guaranteed loss, setup-corridor frames
  read as no viable escape, and last-escape frames use focused player-facing
  language.
- Updated the replay timeline to separate traceback progress, setup corridor,
  lethal tail, last escape, and lethal onset instead of treating the whole
  analyzed suffix as one red block.
- Made Replay turn navigation and board animation calmer by pacing the
  opponent's last-move idle animation and fixing animation listener ownership so
  stone-destroy callbacks cannot get stuck on the last frame.

### Repo and docs

- Promoted lethal-threat terminology and examples into `docs/lethal_threats.md`
  and aligned `docs/game_analysis.md` / `docs/corridor_search.md` around setup
  corridor versus lethal tail.
- Documented the Renju validation process and corpus so future legality changes
  can be tested against known hard examples instead of relying on intuition.
- Added a focused `v0.4.7` replay screenshot review for the updated timeline,
  status, and last-escape presentation.
- Set up mistake detection as the narrow remaining analyzer follow-up, which
  shipped in `0.4.8`; broader product cleanup moves to the `0.5` polish line.

## [0.4.6] - 2026-05-17

**Theme: bring corridor-search replay analysis into the playable web app.**

`0.4.6` turns the analysis report workbench into the first player-facing replay
analysis surface. The analyzer still has bounded-model limits, but saved replays
now explain the final forced corridor directly in the Replay page instead of
requiring a separate static report.

### Web

- Added browser-side replay analysis through a cancellable worker-backed Wasm
  session so saved decisive games are analyzed progressively from the ending
  backward.
- Added frame-aware replay status, analysis timeline fill, last-escape marker,
  and board-space annotations for forced losses, escapes, forbidden replies,
  immediate threats, imminent replies, counter-threats, and corridor entries.
- Reworked replay navigation around the finished-game review flow: the page
  opens at the final board, turn buttons step by whole turns, move buttons still
  step raw plies, and the current side's next actual move is shown as a hover
  target.
- Kept mobile replay board-first by hiding verbose analysis copy in portrait,
  removing unused top spacing, and keeping transport controls directly after the
  board instead of pinning them to the viewport.
- Replaced the overloaded warning overlay sheet with split caution,
  highlighter, and marker sprite roles, then aligned live hints and replay
  analysis overlays on the same board-space vocabulary.

### Bot lab and analysis

- Split the replay analyzer into the shared `gomoku-analysis` crate so
  `gomoku-eval` and `gomoku-wasm` consume the same corridor traceback engine.
- Added the stepped `ReplayAnalysisSession` API with per-frame annotations,
  progress counters, and final analysis output for both report generation and
  browser replay use.
- Tightened corridor reply candidate handling so report and web annotations use
  the same immediate-over-imminent tiering, actual-move filtering, forbidden
  markers, and legal alternative probes.

### Repo and docs

- Added the replay-analysis browser integration contract to
  `docs/game_analysis.md` and updated the `0.4.6` plan around the shipped
  player-facing surface.
- Refreshed the published analysis report after corridor-candidate fixes so the
  static report remains the reference artifact for browser annotation parity.
- Added focused `v0.4.6` replay screenshots to the screenshot-review record.

## [0.4.5] - 2026-05-16

**Theme: turn the bot lab into player-facing settings without exposing every
lab knob.**

`0.4.5` is the UI bridge that the earlier `0.4.x` lab releases were preparing
for. The app now has a dedicated Settings route for game and bot configuration,
tested bot presets for normal play, and a controlled advanced layer for players
who want to see the Rust search knobs directly. It also brings the tactical
vocabulary back into the live board through configurable defensive hints.

### Web

- Added a dedicated Settings route and moved rule/bot configuration out of
  Profile so Profile can stay focused on identity and match history.
- Added Easy / Normal / Hard bot presets plus a Custom bot layer for search
  depth, search width, pattern scoring, and corridor-proof controls.
- Persisted game, bot, hint, and touch-control settings through profile schema
  v5 so local and signed-in profiles keep the same product settings shape.
- Added mobile touch-control settings for touchpad-style cursor movement versus
  tap-to-move cursor placement.
- Added compact board-hint controls for immediate wins/blocks and imminent
  open/broken-three replies, with Renju forbidden-move feedback still always on.
- Added per-player game timers and active-move timing so slower hard/custom bot
  turns are visible instead of feeling like the app stalled.
- Kept replay "play from here" on the current saved rule while letting the
  current Settings bot drive the resumed game.

### Bot lab and rules

- Exposed the unified tactical threat snapshot through the Wasm bridge so the
  web board uses the same immediate/imminent/counter-threat facts as the lab.
- Tightened broken-three semantics, open-three reply coverage, immediate-over-
  imminent hint priority, and Renju-forbidden threat filtering after screenshot
  and analysis-report review exposed bad hint cases.
- Refreshed bot and analysis reports after the threat-model fixes so published
  lab artifacts match the current rules behavior.

### Repo and docs

- Updated `docs/ui_screenshot_review.md` with the `v0.4.5` desktop/mobile
  screenshot set, including the new Settings route and fixed-height mobile
  viewport captures.
- Updated roadmap, visual, data-model, backend, and `0.4.5` planning docs
  around profile-synced settings, compact hint controls, and the settings UI
  boundary.

## [0.4.4] - 2026-05-14

**Theme: replace hot tactical scans with a rolling threat frontier, then spend
the new headroom on measured bot quality.**

`0.4.4` is still a lab release, but it closes the loop opened by `0.4.3`. The
corridor-portal idea did not survive bounded search budgets, so this release
removes that dead path and focuses on the useful foundation it exposed: shared
threat facts, a scan/rolling `ThreatView` seam, explicit search-stage metrics,
and a faster default tactical pipeline. The result is not a player-facing
settings UI yet. It is a cleaner, stronger bot-lab baseline for deciding which
settings deserve to become product controls.

### Bot lab

- Promoted rolling frontier as the default threat-view backend for normal
  search, while keeping scan and rolling-shadow modes available for fallback and
  parity checks.
- Added rolling-backed pattern evaluation so `+pattern-eval` uses a cached
  pattern frame instead of repeated full-window scans in rolling mode.
- Simplified the root safety gate into a first-order current-obligation filter
  over already-generated legal candidates, with scan and rolling implementations
  behind the same `ThreatView` contract.
- Added pooled CPU-budget tournament mode so lab runs stay bounded while more
  closely matching the product expectation that a harder bot can spend longer
  on difficult moves.
- Kept candidate-proof corridor search as a lab axis, but removed the retired
  portal integration path and parser/report surface after repeated negative
  strength/cost results.
- Tightened tactical ordering around staged annotations, null-space culling,
  viability facts, and rolling-frontier metrics, then refreshed the anchor set
  around D1, D3, D5/D7 pattern variants, and corridor-proof variants.

### Reports

- Refreshed the published bot report with pooled-budget anchors, stage timing,
  rolling-frontier health, and cleaned-up search-cost presentation.
- Refreshed the published analysis report from the current top-two bot matchup
  so corridor-search examples match the latest tournament.

### Repo and docs

- Updated `docs/search_bot.md`, `docs/corridor_search.md`,
  `docs/performance_tuning.md`, `docs/tournament.md`, and the `0.4.4` archive
  plan around the rolling-frontier checkpoint and the failed portal line.
- Clarified that `0.4.5` is now the earliest likely UI/settings bridge, with
  `0.4.4` serving as the performance and threat-model foundation for that work.

## [0.4.3] - 2026-05-12

**Theme: unify corridor search with live bot experiments, then draw the line
before the next frontier rewrite.**

`0.4.3` stays in the Rust lab. It started from the question left by `0.4.2`:
can the replay analyzer's corridor model improve live bot search directly? The
answer for the current scan-backed implementation is "not yet." Portal-style
selective extension is semantically promising, but the current cost shape is
too expensive to promote as a stronger bot preset.

The value of the release is the foundation it leaves behind: shared tactical
threat semantics, move-local corridor entry detection, side-specific portal
controls, and enough instrumentation to make the next performance bottleneck
visible. That sets up the next serious lab pass around rolling threat-frontier
data instead of more blind knob turning.

### Bot lab

- Moved corridor logic closer to the bot side so replay analysis and live search
  share the same tactical vocabulary instead of carrying parallel detectors.
- Retired the standalone corridor-bot spike and the earlier leaf-quiescence
  integration shape after they proved useful for plumbing, but wrong for
  shipped behavior.
- Added opt-in corridor portal suffixes for live search experiments, including
  side-specific own/opponent controls, corridor depth and width bounds, resume
  handling, and portal cost metrics.
- Tightened portal semantics so entries are move-local, resumed normal searches
  cannot immediately re-enter portals, and resumed searches do not reuse or
  contaminate the parent transposition table.
- Unified tactical and corridor threat handling around a scan-backed
  `ThreatView` seam, keeping the current scanner as the reference while
  preparing for a future rolling threat frontier.
- Validated that scan-backed portals remain lab-only for now: focused smoke
  checks showed cleaner behavior and better observability, but not a promotable
  strength/cost tradeoff.

### Repo and docs

- Updated `docs/search_bot.md`, `docs/corridor_search.md`, roadmap notes, and
  the `v0.4.3` archive plan to frame corridor portals as infrastructure and
  evidence, not a new difficulty preset.
- Captured the rolling-frontier decision frame: exact apply/undo discipline,
  raw versus Renju-legal threat facts, shadow validation, and a conservative
  `0.4.4` boundary before any behavior switch.
- Extended bot-report metrics so future portal and frontier experiments can
  expose accepted entries, followed corridor plies, resume searches, and exit
  reasons instead of hiding the work inside ordinary alpha-beta cost.

## [0.4.2] - 2026-05-08

**Theme: establish corridor search as the strategic foundation for analysis and
future bots.**

`0.4.2` stays lab-facing, but it is one of the most important project
checkpoints so far. It started as another bot-tuning pass after the `0.4.1`
difficulty ladder, then exposed the real limit of knob turning: the current bot
is already competent enough that obvious strength gains are expensive and hard
to explain. The useful direction is not just "stronger bot"; it is a bot and
analysis layer that understands why games are won or lost.

That is what corridor search gives the project. It formalizes the vocabulary of
forced sequences, escapes, missed defenses, possible escapes, and corridor
entries; it gives the lab a way to inspect bot behavior; and it creates a
direct path toward replay education, critical-move tagging, and eventually
stronger bots that improve through explainable forcing logic instead of opaque
tuning.

### Bot lab

- Added batch gauntlet workflows so groups of candidate bot configs can be
  compared against anchored reference reports without rerunning a full
  round-robin for every sweep.
- Tested child-cap, pattern-eval, symmetric radius, and asymmetric
  candidate-source sweeps against the `0.4.1` anchors; no new anchor was
  promoted, but the results kept `+pattern-eval` and `self2/opponent1` as
  useful lab axes.
- Polished the bot report for larger lineups: clearer ranking/search tabs,
  candidate-vs-anchor labeling, expandable comparisons, cleaner run metadata,
  and a mobile-friendly table layout.

### Replay analysis

- Added replay-analysis fixtures, batch commands, JSON/HTML reports, and a
  published `/analysis-report/` surface generated from the current bot report's
  top-two head-to-head games.
- Implemented the first solid corridor-search foundation for finished replays:
  forced corridor detection, corridor-entry escapes, confirmed/possible escape
  outcomes, loss severity buckets, and proof details tied to the model limits.
- Turned replay analysis into a useful bot-lab diagnostic, not just a future UI
  idea: tournament games can now be inspected by forced sequence, last escape,
  and loss type instead of only by final score.
- Added visual proof frames with reusable board rendering, threat/counter-threat
  hints, forbidden Renju markers, actual-move rings, and compact per-turn
  explanation rows.
- Tightened tactical correctness around open-three defense squares, Renju
  forbidden blocks, illegal black threat filtering, actual replay move proof
  inheritance, and exclusion of non-corridor replay moves from model replies.

### Repo and docs

- Added `docs/game_analysis.md` as the canonical replay-analysis contract,
  including the current model, known limits, CLI workflows, and report shape.
- Added `docs/corridor_search.md` to keep the strategic corridor-search thesis
  visible instead of burying it inside replay report mechanics.
- Added analysis-report publishing to the web build and linked the report
  alongside the existing bot report and asset preview surfaces so the interim
  workbench can be reviewed like a real product artifact, not hidden as raw lab
  output.
- Updated roadmap, bot-lab, performance, tactical-shape, tournament, and release
  docs around the `0.4.2` checkpoint and the next likely `0.4.3` UI/settings
  bridge.

## [0.4.1] - 2026-05-04

**Theme: turn the bot-lab foundation into a measured difficulty ladder.**

`0.4.1` is still a lab-facing release, not the player-facing settings bridge.
It uses the `0.4.0` pipeline/reporting foundation to prove a stronger bot
ladder, refine report readability, and decide that `0.4.2` should stay
lab-first for one more bot exploration pass before exposing settings.

### Bot lab

- Realigned `0.4.1` around a measured tactical ladder rather than broad bot
  feature experiments: local threat safety, tactical-first ordering, child
  frontier caps, and pattern-eval ablations now have shared scenario and
  tournament vocabulary.
- Promoted the local-threat safety gate as the default safety path after it
  preserved hard tactical behavior with lower cost than the older shallow
  search-probe gate.
- Added reusable tactical annotation metrics and report fields so safety,
  ordering, and capped-child experiments expose generated candidate breadth
  separately from searched child breadth.
- Added `+tactical-cap-N` as the report-facing shorthand for
  `+tactical-first+child-cap-N`, keeping candidate radius and non-root child
  breadth as separate lab axes.
- Refreshed the clean 8-entrant Renju reference report with the depth ladder,
  tactical-cap variants, and pattern-eval variants under the centered opening
  suite.

### Repo and docs

- Polished the bot-report UI around the ranking table, expandable pairwise
  comparisons, search metrics, mobile layout, and navigation from the game and
  asset previews.
- Updated the `0.4` planning docs, search-bot notes, tournament notes, and
  performance notes around the current `0.4.1` checkpoint.
- Captured the next `0.4.x` sequencing: cut `0.4.1` as the bot-ladder/report
  checkpoint, keep `0.4.2` as one more lab-first bot exploration pass, and move
  the player-facing settings bridge to `0.4.3`.

## [0.4.0] - 2026-05-03

**Theme: turn `v0.4` toward bot-lab foundations before exposing player-facing
bot controls.**

The current `v0.4.0` candidate is intentionally not a settings/UI release. The
early bot-style experiments showed that "more tactical features" is not useful
unless the lab can prove better reached depth, runtime, or tournament strength.
This slice therefore hardens the measurement/reporting workflow, makes the
search pipeline explicit, records rejected experiments, and lands
behavior-preserving core/search optimizations that give later tactical ranking
work a cleaner base.

### Bot lab

- Added explicit `SearchBotConfig` plumbing, lab specs such as `search-d3`, and
  trace output for candidate source, legality gate, safety gate, search
  algorithm, static eval, node counts, and phase-split metrics.
- Added tactical scenario diagnostics for one-move correctness and cost probes
  without treating those fixtures as ranking substitutes.
- Added multi-threaded tournament JSON and HTML reports with CPU-time budgets,
  seeded openings, compact move lists, match trees, pipeline metrics, and clean
  provenance handling for public `/bot-report/` publishing.
- Recorded and removed rejected tactical candidate, ordering, broad threat
  extension, and broad shape-eval experiments instead of keeping dead config
  toggles.
- Split the search pipeline into explicit candidate source, exact legality,
  optional safety gate, move ordering, alpha-beta search, and static-eval
  stages so future ablations can isolate one variable at a time.

### Core and performance

- Added benchmark coverage for board storage, candidate generation, legality,
  immediate-win probes, pipeline stages, and fixed tactical/search scenarios.
- Tightened the Renju forbidden precheck so exact forbidden detection only runs
  after a cheap local necessary-condition guard.
- Replaced immediate-winning-move probes with virtual directional win checks
  while preserving exact legality handling.
- Switched core board storage to dual bitboards and routed hot bot eval and
  candidate paths through occupied-stone iteration.
- Kept positive behavior-preserving optimizations in place, while leaving
  tradeoff-heavy bot behavior changes behind lab specs and reports.

### Repo and docs

- Published the bot report alongside existing asset previews through GitHub
  Pages.
- Updated bot-lab README, search-bot notes, performance tuning notes, and
  `v0.4` planning docs around the pivot from product settings to measured bot
  foundation work.

## [0.3.3] - 2026-04-29

**Theme: wrap up `v0.3` by hardening cloud continuity instead of expanding
backend scope.**

`v0.3.3` grew larger than a normal patch because it closes the backend
foundation line: private history is now one cost-aware profile snapshot,
local/cloud profile shapes are aligned, auth handles more browser contexts, and
the operational docs describe the actual production path. The product remains
local-first and private-by-default; public replay sharing, online play, and
server-verified matches stay out of `v0.3`.

### Web and profile UX

- Polished Profile, Privacy, and Terms wording around local profile state, cloud
  sync, private history, experimental online features, and reset behavior.
- Softened the Reset Profile action, tightened confirmation copy, and kept the
  signed-in/signed-out reset scopes explicit.
- Added a signed-in Delete Cloud path behind Reset Profile so users can delete
  their Gomoku2D cloud profile, clear cloud history, and sign out while keeping
  local browser history local.
- Added compact profile-history sync badges for queued, syncing, synced, and
  retry states without adding another heavy status block.
- Added a `Show more` history control so the Profile page can keep 128 replay
  records without rendering an oversized list by default.
- Renamed the saved replay route from `/replays/local/:matchId` to
  `/replay/:matchId` as a clean URL break before replay URLs become
  user-facing.

### Cloud sync and schema

- Pivoted casual private cloud history from per-match documents to one embedded
  `profiles/{uid}.match_history` snapshot.
- Introduced profile schema v3 with `auth.providers`, `settings.default_rules`,
  `reset_at`, and mutually exclusive `replay_matches`, `summary_matches`, and
  `archived_stats` retention tiers.
- Aligned local history with the cloud shape under the clean-break
  `gomoku2d.local-profile.v3` key.
- Raised private replay retention to 128 full replay records, added a 1024-row
  lightweight summary tier, and rolled older records into archived aggregate
  stats.
- Changed signed-in profile/settings/history sync to a 5-minute coalesced
  profile-write lane, so rapid name edits, rule toggles, and multiple finished
  matches collapse into bounded snapshot writes.
- Avoided routine Firestore profile refresh writes when signed-in cloud profile
  fields are already current, and skipped no-op local-to-cloud promotion writes.
- Reconciled queued cloud-history sync against refreshed profile snapshots so
  live/local build races do not leave stale error badges after the data already
  reached Firestore.

### Auth, rules, and deploy

- Added Firebase Auth redirect fallback: desktop prefers popup, mobile or
  embedded contexts prefer redirect, popup-blocked/unsupported errors fall back
  to redirect, and intentional popup closes do not.
- Added direct GitHub Pages route entries for `/profile` and `/match/local` to
  avoid deep-linking through a visible `404` response where static routes are
  known.
- Hardened Firestore rules for profile schema v3, embedded-history caps,
  reset-barrier writes, 5-minute profile-update cooldowns, reset cooldown
  bypasses, owner-only profile deletes, and closed casual match subcollection
  writes.
- Updated Firestore index exemptions for the embedded history fields.
- Tightened release, backend infra, data-model, and cost docs around tag-only
  deploys, manual deploy smoke, Workload Identity rules deployment, and current
  Firestore free-tier math.

## [0.3.2] - 2026-04-28

**Theme: private cloud history becomes the active history surface.**

`v0.3.2` finishes the core cloud-continuity loop for `v0.3`: signed-in matches
save privately to Firestore, cloud history loads back into Profile/Replay, and
Reset Profile has a server-side barrier so old local rows cannot silently
return after a reset. The UX still abstracts away storage when sync is healthy:
guest users see local history, signed-in users see "my history", and public
replay sharing remains a later explicit publish feature.

### Web and cloud history

- Added direct `cloud_saved` writes for finished signed-in casual matches under
  `profiles/{uid}/matches/{match.id}` while still saving locally first.
- Added per-user cloud-history cache and active-history resolution so Profile
  and Replay can use one visible history surface across local pending rows,
  promoted guest imports, and cloud-saved matches.
- Added dedupe between direct `cloud_saved` IDs and deterministic
  `guest_import` IDs so promotion/retry paths do not duplicate the same local
  match.
- Added background pending-sync metadata, retry hooks, and visible sync failure
  states without blocking local play.
- Added signed-in Reset Profile with inline confirmation, cloud profile/default
  reset, bounded private-history delete, per-device cache clear, and local
  pending-sync clear.
- Kept guest-only and no-Firebase behavior intact: local profile/history/replay
  still work without cloud config.

### Rules, schema, and tests

- Added `history_reset_at` to the cloud profile reset model and made client
  promotion, direct sync, cloud loads, and active-history resolution ignore
  records at or before the barrier.
- Added `match_saved_at` to private cloud match documents so Firestore rules
  can compare match age against the reset barrier without parsing strings.
- Hardened Firestore rules for owner-only private match creates/deletes,
  request-time-only reset barrier writes, monotonic reset movement, and
  server-side rejection of stale post-reset match creates.
- Added emulator-backed Firestore rules tests and wired them into CI with Java
  setup.
- Added package overrides so the Firebase rules tooling keeps `npm audit`
  clean.

### Validation

- Local and production smoke tested signed-in save, reload, sign-out/sign-in,
  Reset Profile, post-reset save, and live/local cross-profile cloud sync.
- Manually deployed and verified the matching web build and Firestore rules
  before preparing the release.
- Refreshed backend infra, cost, data-model, roadmap, and completion-plan docs
  around the completed private-history slice.

## [0.3.1] - 2026-04-28

**Theme: private cloud continuity without a sign-in wall.**

`v0.3.1` makes sign-in feel like continuity instead of a separate account
mode. Local guest profiles and finished local matches can now promote into
private cloud state after sign-in, while local play remains complete without
cloud and no public artifacts are created implicitly.

### Web and cloud profile

- Added guest-to-cloud promotion: local profile display name,
  preferred rule, and finished local matches are copied to private cloud state
  after sign-in.
- Default `Guest` profiles now adopt the linked cloud display name on sign-in;
  custom local display names promote only while the cloud name still matches the
  provider default.
- Added deterministic `guest_import` match document IDs so retries skip already
  imported local matches instead of duplicating them.
- Added `docs/data_model.md`, moved local history to canonical
  `guest-profile.v2`, and tightened saved-match v1 around compact `move_cells`
  replay storage instead of verbose move objects.
- Added identity-bearing player records for saved matches, including owner UID
  snapshots for promoted humans and versioned practice-bot identity/config
  snapshots.
- Added saved-match validation for local history hydration and kept replay
  winning-line reconstruction backed by the shared core rules path.
- Updated Profile cloud copy to show background import progress, success, and
  failure while keeping local history on-device.
- Smoke-tested the local build promotion path with a 24-match guest history:
  Firestore imported exactly 24 private `guest_import` matches with matching
  `local_match_id`s and no extra/missing records.

### Schema and data model

- Versioned cloud profile documents with `schema_version: 1`; existing profiles
  receive the field on next sign-in via merge update.
- Prepared the `cloud_saved` document shape for future matches saved directly to
  cloud while signed in: no import metadata fields, `created_at` server
  timestamp instead of `imported_at`, human player carries `profile_uid` only
  (`local_profile_id: null`).
- Added `matchUserSide(match, { profileUid, localProfileId })` helper to resolve
  the user's side cross-device: prefers `profile_uid` (correct on any device for
  cloud records), falls back to `local_profile_id` for local-only records.
- Fixed display-name promotion so a custom local name only overwrites the cloud
  name when the cloud still holds the provider default — prevents a second device
  from silently overwriting a name chosen on the first.

### Infra

- Opened Firestore rules narrowly for owner-only private `guest_import` match
  creates and local display-name promotion; match updates/deletes remain closed.
- Tightened Firestore match validation for compact move-cell payloads.
- Extended Firestore rules to accept `cloud_saved` match creates alongside
  `guest_import`; both paths share common field validation via `validMatchCommon`.
- Kept source-specific match document keys and bot identity pinned in Firestore
  rules so persisted v1 data cannot drift ahead of the client decoder.

## [0.3.0] - 2026-04-27

**Theme: backend foundation without putting cloud in front of local play.**

`v0.3.0` opens the backend line with optional public Google sign-in and a
private cloud profile. The product remains local-first: guests can still play,
save local history, and replay matches without signing in. Cloud identity now
exists as the foundation for later guest promotion and private cloud history,
but those continuity features stay in follow-up `0.3.x` slices.

### Web and cloud profile

- Added env-gated Firebase browser bootstrap so cloud features initialize only
  when all required `VITE_FIREBASE_*` values are present.
- Added Google sign-in/sign-out on Profile and a cloud state badge that keeps
  local, unavailable, loading, error, and signed-in states visible.
- Added private cloud profile create/load at `profiles/{uid}`, seeded from the
  Google provider and updated with provider metadata, preferred rule, and login
  timestamps.
- Kept guest/local history separate from cloud identity; signed-in Profile copy
  explicitly says local history remains local until promotion ships.
- Verified the no-config production build path: cloud sign-in is disabled,
  no Auth/Firestore requests are made, and Home/Local Match still work.

### Public app readiness

- Moved the public app to `https://gomoku2d.byebyebryan.com/` with GitHub Pages
  serving from the custom domain root.
- Added crawlable static `/privacy/` and `/terms/` pages using the same
  retro/info-page surface language as the app shell.
- Linked policy pages from the raw home HTML and the React-rendered Home screen
  so OAuth crawlers and users can both find them.
- Added the public contact/deletion address `gomoku2d@byebyebryan.com`.
- Published the Google Auth Platform app to production, kept OAuth scopes to
  basic identity/profile/email sign-in, and intentionally left the OAuth logo
  blank to avoid unnecessary brand-verification work.

### Infra and docs

- Documented the live Firebase/GCP setup in `backend_infra.md`: project IDs,
  enabled APIs, Auth domains, OAuth access gate, Firebase web config, rules
  deployment, and smoke-test checks.
- Split backend cost/headroom tracking into `backend_cost.md` with explicit
  Firestore/Auth free-tier assumptions and guardrails for future `0.3.x` work.
- Deployed hardened profile-only Firestore rules for `profiles/{uid}`, with
  private match writes kept closed until the cloud-history slice ships.
- Refined the roadmap around the project thesis: `v0.3` is backend continuity,
  `v0.4` should make the Rust lab visible as product identity, and later
  online/trusted-match work remains out of scope.
- Updated release and deployment docs for the custom-domain Pages workflow and
  manual release-candidate deploys.

## [0.2.4] - 2026-04-24

**Theme: wrap up `v0.2` with polish, release hygiene, and better iteration
loops.**

`v0.2.4` is the wrap-up release for the local-first `0.2.x` line. It does not
try to redesign the app. Instead, it closes the loop on the paired
desktop/mobile shell from `v0.2.3`, hardens the Practice Bot without turning it
into a stronger opponent, and puts better workflows in place for future bot,
asset, and release work.

After `v0.2.4`, the DOM shell is considered effectively frozen for the rest of
`0.2.x`; future work should be bug fixes, bot/core iteration, or explicitly
new feature phases.

### Web polish

- Refined the outer DOM shell one more time: quieter labels, tighter spacing,
  consistent icon/button alignment, denser desktop Match and Replay rails, and
  a Profile screen that reads more like a local record page than a settings
  form.
- Kept the mobile layouts from `v0.2.3`, but made cramped Match and Replay
  viewports safer by allowing page flow/scroll instead of letting bottom
  controls collide with the board.
- Polished player-facing copy, renamed the local opponent to `Practice Bot`,
  and added a Home-screen version label sourced from the web package version.
- Added favicon assets, refreshed the README hero GIF, regenerated the Open
  Graph image, and captured the final `v0.2.4` desktop/mobile screenshot set.

### Game visuals and asset workflow

- Reworked canvas warning language so winning/threat cells, forbidden Renju
  cells, and threat-plus-forbidden overlaps can be shown without fighting the
  pointer.
- Refined the sprite animation set and z-order rules for pointer, warnings,
  stones, winning-line hover, and result sequence numbers.
- Switched result-screen sequence labels from the old bitmap-font path to the
  TTF pixel-font path with stable desktop/mobile sizing.
- Split DOM-shell styling and canvas/game visuals into separate docs:
  `ui_design.md` for the shell and `game_visual.md` for Phaser board-space
  rendering.
- Added published preview pages for sprites, icons, and fonts so asset changes
  can be inspected quickly, shared visually, and used as a small showcase for
  the retro pixel-art style.

### Bot lab and core workflow

- Hardened the baseline Practice Bot against obvious anti-blunder failures,
  especially immediate-win and immediate-block cases.
- Added a bot/core performance benchmark harness with a fixed, reviewable
  scenario corpus so future tuning work can be measured instead of guessed.
- Optimized nearby-move generation, immediate-winning-move scans, and the
  anti-blunder prefilter so the safety fix does not carry unnecessary
  slowdown.
- Added search-bot regression coverage over the benchmark corpus and formatted
  the Rust workspace with `rustfmt`.
- Moved winning-line detection into `gomoku-core` / `gomoku-wasm`, reducing
  duplicated game-rule logic in the web UI.

### Gameplay fixes

- Replay branching now preserves an undo floor, so a resumed match cannot undo
  earlier than the replay position it started from.
- Mobile Match keeps the touch-placement flow from `v0.2.3`, while preserving
  the board-first layout and making failure cases safer on short screens.

### Project and release hygiene

- Refreshed the canonical doc set (`product.md`, `architecture.md`,
  `app_design.md`, `backend.md`, `roadmap.md`, `ui_design.md`,
  `game_visual.md`, `ui_screenshot_review.md`, root `README.md`,
  `gomoku-web/README.md`) around the `0.2.x` freeze.
- Added the ongoing bot/core performance tuning note with benchmark rules,
  fixed scenarios, baseline timings, and optimization-pass snapshots.
- Added `LICENSE` (MIT), `.editorconfig`, CI workflow, `rust-toolchain.toml`,
  Node version pin, Dependabot config, PR template, release automation,
  release backfill tooling, smoke-test coverage, and committed
  `gomoku-bot-lab/Cargo.lock` for reproducible workspace binaries.
- Batched safe Dependabot updates and fixed the resulting Rust CI lint issues.

## [0.2.3] - 2026-04-22

**Theme: paired desktop/mobile system, not a desktop-first shell with a
narrow fallback.**

Before `v0.2.3` the mobile experience was desktop screens collapsing downward.
This release introduces intentional portrait layouts on every main screen and
a dedicated touch-placement control model for mobile Match, because direct
tap-to-place on a 15×15 grid was too cramped. It also introduces the shell's
icon language — monochrome, narrow in scope, sized for the authored pack.
Bot lab unchanged.

### Design (web)

- Match, Replay, and Profile now have intentional portrait layouts instead of
  reading like desktop screens collapsing downward.
- Introduced a cohesive icon language for desktop actions and replay transport,
  kept monochrome and narrow to avoid becoming a separate app skin.
- Denser, clearer replay transport controls that keep the board as the hero.
- Local Match on portrait uses a dedicated touch-placement flow rather than
  direct tap-to-place, so the player can aim before committing.

### Added (web)

- Dedicated mobile touch-placement flow for local matches.
- Mobile-specific layouts for Local Match, Replay, and Profile.
- Desktop icon pack for actions and replay transport.
- Pending-rule note on mobile Match when a rule switch is queued for the next
  round.

### Docs

- Refreshed visual reference captures (v0.2.3 desktop + mobile set).
- Aligned `v0.2.3` release framing and visual baseline across the design docs.

## [0.2.2] - 2026-04-22

**Theme: flatten the shell and clarify button roles.**

`v0.2.1` proved the structure but still felt slightly app-like and over-boxed.
`v0.2.2` removes unnecessary nested panel frames, strengthens palette contrast,
and gives buttons narrow, consistent jobs (primary/secondary/tertiary/danger).
On the gameplay side, it deepens the local play loop with two features the
local-first product needed to feel complete: undo during a live match, and
resuming a local match from a replay frame. Bot lab unchanged.

### Design (web)

- Flatter shell with fewer unnecessary boxes and repeated panel frames.
- Stronger palette contrast and clearer button-role language.
- Clearer live-match and replay HUD language — less dashboard, more HUD.
- Stronger record-screen treatment on Profile: history as ledger rows rather
  than boxed cards.

### Added (web)

- Undo the last turn during a live match.
- Resume a local match from a replay frame (branch into a new practice game
  from any position in the replay).

### Changed (web)

- Restyled the local shell, profile layout, and replay sidebar to the flatter
  `v0.2.2` baseline.
- Improved portrait shell scrolling so scroll ownership stays inside panes
  that need it.
- Fixed profile history gutter spacing.

### Docs

- Aligned `v0.2.2` roadmap and design docs.

## [0.2.1] - 2026-04-22

**Theme: first practical DOM-shell baseline — the React rewrite lands, and
the product direction is reframed around local-first play.**

`v0.2.1` is the hinge release. It replaces the `v0.1` scene-driven Phaser app
with a React shell that owns the UI (home, match, replay, profile) while
Phaser is reduced to a stateless board-only renderer receiving props and
emitting intent events. It also reframes the project: instead of pushing
straight into an online/Firebase backend, the focus moves to a richer
local-first `v0.2` built around a local guest profile on first interaction,
with cloud sync, published replays, and online play deferred to later phases.
The bot lab is unchanged in code — the same `gomoku-wasm` build now sits
behind a React wrapper instead of a Phaser scene.

### Design (web)

- Proper DOM-shell structure with clear screen separation between Home,
  Match, Replay, and Profile.
- Stronger foundation for local profile and replay features — the shell can
  grow without turning Phaser back into the app.
- More scalable spacing, scrolling, and panel ownership.
- Established retro/tactile visual direction and a shared token layer for
  color, spacing, and typography.

### Added (web)

- React + Vite + React Router + Zustand application shell.
- Routes: Home (`/`), Local Match (`/match/local`), Profile (`/profile`),
  Replay (`/replays/local/:matchId`).
- Local guest profile persisted in browser storage (display name, preferred
  rule, recent-match history) — no sign-in required.
- Local replay viewer with transport controls.
- Local rules selection (Freestyle / Renju) tied to the guest-profile
  default.

### Changed (web)

- **Runtime boundary:** React owns state and routing; Phaser renders the
  board as a stateless view driven by a narrow `BoardProps` / `BoardEvent`
  interface.
- Refined local web shell palette and state handling.
- Confined app scrolling to internal panes so the viewport stays fixed.
- Tightened board sizing and route entry so the board doesn't jump on
  navigation.

### Repository

- Rust crates grouped under `gomoku-bot-lab/`; the web game reframed as the
  top-level product.

### Docs

- **Product direction reframed** around a guest-first, local-first `v0.2`;
  cloud sign-in, published replays, and online play deferred to later phases.
- New canonical doc set written as the new north star: `product.md`,
  `architecture.md`, `app_design.md`, `backend.md`, `roadmap.md`.
- Pre-pivot exploratory docs moved to `docs/archive/` (progress log,
  Phaser-era FE plan, online backend design, FE gap analysis, UI/UX
  exploration notes).

## [0.1] - 2026-04-18

**Theme: first playable snapshot — stand up the Rust core and bot lab, then
prove they work end-to-end in a browser.**

`v0.1` established both halves of the repo. The **Rust bot lab** was built
from scratch: a Cargo workspace with a zero-dep rules core, a `Bot` trait with
random and search-based implementations, a CLI match runner, a self-play and
Elo evaluation framework, and a `wasm-pack` bridge for the browser. The
**web game** was a single Phaser scene on top of that bridge — the right
shape to prove the Rust core + Wasm path worked in a real browser bundle,
the wrong shape to grow from. Gameplay, settings, and shell concerns blurred
together in one canvas-driven surface. That lesson drove the `v0.2.1` rewrite.

### Added

**Bot lab** (new workspace)

- Cargo workspace: `gomoku-core`, `gomoku-bot`, `gomoku-cli`, `gomoku-eval`,
  `gomoku-wasm`.
- `gomoku-core`: 15×15 board, Freestyle + Renju rules (including Black's
  double-three / double-four / overline restrictions), win detection, FEN,
  replay JSON, tactical-hint + forbidden-move analysis.
- `gomoku-bot`: `Bot` trait, `RandomBot`, and `SearchBot` (negamax with
  alpha-beta, iterative deepening, transposition table keyed by incremental
  Zobrist hash, candidate pruning to stones within radius 2, open/half-open
  run evaluation).
- `gomoku-cli`: native match runner with replay export; flags for bot
  selection, depth, time budget, rule variant.
- `gomoku-eval`: self-play arena, round-robin tournaments, Elo ratings.
- `gomoku-wasm`: `wasm-pack` bridge exposing `WasmBoard` and `WasmBot` to
  JS, with Renju variant support via `createWithVariant`.
- Self-describing replay JSON with per-move metadata and display notation,
  shared by `gomoku-cli` and `gomoku-eval`.

**Web** (Phaser single-scene app)

- Phaser 3 + TypeScript + Vite scaffold.
- 15×15 Gomoku play with Freestyle and Renju rule variants.
- Human-vs-human, human-vs-bot, and bot-vs-bot modes with per-player
  Human/Bot toggles and inline name editing.
- Renju forbidden-move warnings for the human Black player.
- Per-player and total-game move timers, pause-on-settings.
- Stone placement, win detection, reset; forming/shattering animations.
- Pointer and last-placed-stone idle animations.
- Player-card swap animation on new round.
- Move sequence numbers on stones after game end.
- Bot execution in a Web Worker (stateful, ready-signal handshake).
- Pixel bitmap font and sprite/asset consolidation under
  `gomoku-web/assets/`.

**Infrastructure**

- GitHub Pages deploy workflow (manual trigger) for the web build.

### Design (web)

- Strong retro tone and board dominance from move one.
- Chunky, high-contrast controls; UI that feels game-like before it feels
  app-like.

### Known limits (addressed in v0.2.1)

- Everything lived inside one Phaser scene; gameplay, settings, and shell
  concerns blurred together.
- Expressive UI language, but not scalable beyond one canvas.

[Unreleased]: https://github.com/byebyebryan/gomoku2d/compare/v0.5.4...HEAD
[0.5.4]: https://github.com/byebyebryan/gomoku2d/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/byebyebryan/gomoku2d/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/byebyebryan/gomoku2d/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/byebyebryan/gomoku2d/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/byebyebryan/gomoku2d/compare/v0.4.8...v0.5.0
[0.4.8]: https://github.com/byebyebryan/gomoku2d/compare/v0.4.7...v0.4.8
[0.4.7]: https://github.com/byebyebryan/gomoku2d/compare/v0.4.6...v0.4.7
[0.4.6]: https://github.com/byebyebryan/gomoku2d/compare/v0.4.5...v0.4.6
[0.4.5]: https://github.com/byebyebryan/gomoku2d/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/byebyebryan/gomoku2d/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/byebyebryan/gomoku2d/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/byebyebryan/gomoku2d/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/byebyebryan/gomoku2d/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/byebyebryan/gomoku2d/compare/v0.3.3...v0.4.0
[0.3.3]: https://github.com/byebyebryan/gomoku2d/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/byebyebryan/gomoku2d/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/byebyebryan/gomoku2d/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/byebyebryan/gomoku2d/compare/v0.2.4...v0.3.0
[0.2.4]: https://github.com/byebyebryan/gomoku2d/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/byebyebryan/gomoku2d/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/byebyebryan/gomoku2d/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/byebyebryan/gomoku2d/compare/v0.1...v0.2.1
[0.1]: https://github.com/byebyebryan/gomoku2d/releases/tag/v0.1
