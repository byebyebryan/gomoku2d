# Roadmap

The order we're going to build in.

Versions are the planning spine. A phase is done when it produces a coherent
playable/product state, not when it checks every possible sub-task. Patch
releases can polish or harden a phase, but they should not blur phase intent.

This roadmap tracks two things at once:

- **Product story**: each phase should make Gomoku2D more credible as a real
  alpha/beta product.
- **Production story**: each phase should teach something useful about how much
  of a small product team's surface area AI agents can help one developer cover
  without lowering the quality bar.

The project thesis behind that split lives in [`project.md`](project.md).

## P1 — `v0.1` POC (done)

Validate that the basic stack idea works:

- Rust rules/core logic
- bot loop good enough to build against
- wasm bridge into a browser build
- Phaser-rendered browser game loop
- GitHub Pages deployment

What it proved:

- Rust + Wasm + web is a viable foundation for this project
- the bot lab can power the browser game without a rewrite
- a lightweight public deploy loop is enough for fast iteration

Production lens:

- prove whether an old native/game-logic idea can be revived into a browser
  product without throwing away the original technical core
- establish that repo docs, code review, and release notes can keep pace with
  rapid AI-centric iteration

Historical state is preserved in
[`progress_v0.1.md`](../../archive/progress_v0.1.md).

## P2 — `v0.2` FE Foundation And Local Play (done)

Move from proof-of-concept to a proper frontend product surface.

### Goals

- replace the single-scene Phaser app with a scalable FE stack
- put React in charge of the app shell, routes, and UI state
- reduce Phaser to a board-focused renderer
- establish durable DOM-shell and canvas visual languages
- make the local game feel complete without cloud
- support desktop and mobile intentionally

### Work In This Phase

- React shell and route structure
- local stores and browser persistence
- board owned by props/events instead of scene-owned app state
- Home, Match, Replay, and Profile as separate surfaces
- local profile and preferred-rule persistence
- local match history and replay viewer
- replay branching back into local practice
- board-first desktop and mobile match layouts
- mobile pointer/touchpad placement modes for cramped boards
- asset previews, screenshot/reference assets, release hygiene
- dependency/runtime maintenance after `v0.2.4` without cutting another product
  release

### Done When

A player can open the site and get a polished local experience:

- quick play from Home
- clean board-first match flow
- useful local replay without extra analysis surfaces
- local player record with persistent defaults and history
- mobile-web that feels intentionally designed
- consistent shell style that no longer feels like transitional scaffolding

### Out Of Scope

- sign-in
- Firestore/cloud persistence
- public replay links
- online matches
- replay analysis and puzzles

### Production Lens

- learn how far agents can help shape a frontend product, not just generate UI
  code
- establish screenshot review, asset preview, animation inventory, and release
  polish loops as first-class project practices
- validate the React/Phaser/Rust boundaries that let agents work in one layer
  without constantly breaking another

## P3 — `v0.3` BE Foundation And Cloud Continuity (done)

Add backend foundation without putting cloud in front of the local game.

Detailed working notes live in
[`v0_3_plan.md`](../../archive/v0_3_plan.md). That file is an ad-hoc planning
artifact; this section remains the canonical roadmap.

### Goals

- optional sign-in
- cloud-backed profile
- continuity across browsers/devices
- durable private history beyond one browser/device
- basic backend plumbing that later lab-powered and online features can build
  on

### Work In This Phase

- Firebase project/env setup and documentation
- Firebase Auth integration
- Profile sign-in/sign-out UI
- cloud profile create/load
- local profile-to-cloud promotion
- preferred settings/profile sync
- private cloud history for signed-in players
- cloud-saved private replay loading
- hardened Firestore rules for owner-scoped profile/history docs
- profile reset barrier and private-history clear path

### Final State

The backend-foundation and first continuity slices have reached the `v0.3.3`
wrap-up state:

- Firebase/GCP project, Firebase web app, and env-driven web bootstrap
- Firestore `(default)` in `us-central1`
- hardened owner-scoped Firestore rules for `profiles/{uid}` with embedded
  private match history
- Google Auth provider configured through the Firebase Auth / Identity Toolkit
  path
- Firebase Auth popup/redirect handling: top-level contexts, including mobile,
  try popup first; redirect is gated to hosts that can safely complete the
  Firebase Auth helper flow
- Profile sign-in/sign-out UI
- cloud profile create/load at `profiles/{uid}`
- local profile-to-cloud profile/settings promotion after sign-in
- capped private `match_history` replay, summary, and archived-stats tiers
  embedded directly in the cloud profile
- 5-minute coalesced profile/history sync lane for profile edits and finished
  signed-in casual matches
- per-user cloud-history cache and active-history resolution for Profile and
  Replay
- queued cloud-history reconciliation after live/local build races, so stale
  local sync errors clear when Firestore already contains the match
- signed-in Reset Profile flow with confirmation, reset barrier, embedded
  private-history clear, and per-device cache clear
- signed-in Delete Cloud path behind Reset Profile for deleting
  `profiles/{uid}`, clearing this device's cloud cache for that user, and
  signing out while leaving local browser history local
- compact, versioned private match schema documented in
  [`data_model.md`](../backend/data_model.md)
- local profile storage moved to the clean-break `local-profile.v3` key,
  aligned with the cloud replay/summary/archive retention tiers
- local play/history still working without Firebase config
- Google Auth Platform published to production for public sign-in
- static `/privacy/` and `/terms/` pages plus contact/deletion email for OAuth
  app readiness
- GitHub Pages publishes direct SPA entries for `/profile` and `/match/local`
  in addition to the fallback `404.html` route, reducing visible deep-link
  `404` noise for static app routes
- public-domain sign-in smoke and no-config fallback smoke completed for
  `0.3.0`
- local-build local-history promotion smoke completed for `0.3.1`: one 24-match
  local history imported exactly once before the later profile-snapshot pivot
- `0.3.2` private-history smoke completed across production and local builds:
  signed-in saves persisted, cloud history restored after refresh/sign-out,
  Reset Profile cleared cloud/local active history, old rows did not re-import,
  and post-reset saves worked normally
- Firestore rules tests cover owner scoping, profile update cooldowns,
  reset-barrier writes, embedded-history caps, owner-only profile deletes, and
  closed casual match subcollection writes
- infra and free-tier tracking split into
  [`backend_infra.md`](../ops/backend_infra.md) and
  [`backend_cost.md`](../ops/backend_cost.md)

Any future `0.3.x` work should be narrow hardening only: auth edge cases,
offline/sync polish, or cloud-history fixes exposed by real usage. Do not add
new product scope to this phase.

### Done When

Signing in extends the same local-first product without breaking it:

- local-only play remains complete
- local history can be promoted without duplicates
- signed-in matches save privately to cloud
- signed-in profile/history works across browsers
- no public artifacts are created implicitly

### Out Of Scope

- public replay sharing
- public profile URLs / username reservation unless required for auth polish
- live PvP
- matchmaking
- ranked/trusted matches
- Cloud Run match authority
- leaderboards
- replay analysis and puzzles

### Production Lens

- learn the practical boundary between local-first UX and cloud-backed
  persistence
- establish a safe secrets/config/docs workflow for GCP/Firebase work
- keep cloud setup, cost estimates, auth caveats, and release gates documented
  well enough that future agents can continue without guessing

## P4 — `v0.4` Lab-Powered Product Identity (complete)

Make the Rust lab visible as a product differentiator.

### Goals

- build a measured bot-lab foundation before exposing new player-facing bot
  controls
- make saved games useful after they end
- turn bot/search tooling into player-facing learning and challenge features
- make the Rust lab visible as a real product advantage
- give Gomoku2D a reason to exist beyond retro styling and basic play
- keep the lab-to-product loop tight: core/bot findings should become UI
  features without a rewrite

### Completed Outputs

The `0.4` line shipped the lab-powered identity in stages:

- explicit `search-*` bot specs, tactical scenario diagnostics, tournament
  reports, and search-pipeline metrics;
- measured bot presets and controlled advanced settings for depth, width,
  pattern scoring, corridor proof, tactical hints, and Renju feedback;
- rolling threat facts and pattern-frame caching as the default hot-path backend,
  with scan-backed modes kept as fallback/comparison tools;
- corridor search as the shared strategic model for replay analysis, bot
  diagnostics, and future player education;
- the in-product Replay analyzer: setup corridor, lethal onset, last escape,
  mistake-aware labels, and board evidence overlays;
- exact Renju forbidden-move checking backed by extracted/reference fixtures and
  practical performance filters;
- curated bot and analysis reports published as visible proof of the lab.

Detailed chronology and experiment evidence live in the archive and working
notes, especially [`v0_4_plan.md`](../../archive/v0_4_plan.md),
[`v0_4_search_bot_enhancement_plan.md`](../../archive/v0_4_search_bot_enhancement_plan.md),
[`v0_4_3_corridor_bot_plan.md`](../../archive/v0_4_3_corridor_bot_plan.md),
[`v0_4_4_frontier_plan.md`](../../archive/v0_4_4_frontier_plan.md), and
[`performance_tuning.md`](../../working/performance_tuning.md).

### Durable Lessons

- Direct bot-strength tuning had diminishing returns. Depth, tactical ordering,
  pattern scoring, and candidate caps matter, but most knobs only became useful
  after tournaments and reports made their tradeoffs visible.
- Corridor search was not broadly useful as a live bot shortcut under the current
  browser-scale compute budget, but it became the right foundation for replay
  explanation and shared tactical vocabulary.
- Rolling threat facts were worth doing because they unified the hot path behind
  a stable query contract while keeping scan-backed validation/fallback options.
- Renju legality is too subtle for shape shortcuts. The corrected recursive
  checker and corpus are core correctness work, not optional lab polish.
- The strongest product direction is explainable play: configurable bots,
  reports, and replay analysis that can show where a finished game turned.

That makes `0.4` a coherent foundation: configurable bots, measured reports,
rolling threat facts, replay traceback, lethal onset, Renju correctness, and
mistake-aware explanations now all exist. The remaining work is no longer to
prove the lab direction. `0.5` should turn these capabilities into a cleaner
presentation system, stronger onboarding, and more intentional player education
without expanding the analyzer scope by default.

### Done When

At least one player-facing feature clearly exists because the bot lab can run
positions and searches outside the live board. A stranger should be able to see
why this is not just another Gomoku board with a bot.

### Production Lens

- test agentic work across the hardest boundary in the repo: Rust search
  logic, wasm/API plumbing, and explanatory UI
- develop a repeatable workflow for turning bot-lab experiments into polished
  product features
- learn how to present AI/search output in a way that helps players without
  making the UI feel like a debug dashboard

## P5 — `v0.5` Public Release Reconciliation (current)

Turn the `0.4` lab-powered foundation into a cleaner, more understandable
public alpha. The goal is not to add another broad research line; it is to make
the existing product story legible to strangers, clean up the repo after the
heavy lab work, and package the project for first public release.

### Goals

- reconcile code, tests, docs, and generated artifacts after the `0.4` lab line
- make the bot and replay-analysis reports feel like first-class product pages
  instead of developer artifacts
- explain Gomoku, Renju, bot settings, and replay analysis from inside the app
- refresh showcase/onboarding surfaces around the current product loop:
  play instantly, review the ending, see where the game turned, and branch from
  the replay
- prepare README, screenshots, social assets, itch/dev-log copy, and release QA
  for a first public-facing alpha
- choose which 0.4 follow-ups belong in the public product now: better-move
  suggestions, puzzle generation, bot personalities, or a stronger server-side
  bot endpoint only if browser-side wasm is not enough

### Possible Work

- mark current generated report artifacts explicitly so they do not dominate
  source-language stats or review diffs
- move bot/analysis report presentation toward web-owned viewer components over
  structured report data
- slim or split committed report data if the current `latest.json` artifacts are
  too heavy for normal review after the viewer rewrite starts
- productize the existing concise docs for About, Rules, Analysis, and Bot Lab
  inside the app
- polish Home, Replay, report pages, and README copy around the lab-under-the-
  board story
- refresh hero capture, screenshots, Open Graph image, and public release notes
- theme/skin work only if it supports the public story and does not become the
  main scope

### Done When

The project is ready to show to strangers as a coherent public alpha:

- the repo is clean enough that generated artifacts do not obscure source code;
- report pages are understandable product surfaces, not lab dump pages;
- the app explains its unusual features without requiring docs archaeology;
- README/home/release assets reflect the current `0.4` capabilities;
- public-release smoke covers play, settings, replay analysis, reports,
  sign-in, mobile, and no-config fallback.

### Production Lens

- test whether agents can help reconcile a research-heavy line into a clean
  public-facing product
- keep build process, repo hygiene, docs, and release artifacts at the same
  quality bar as user-facing features
- learn how to present lab/search output as product education rather than debug
  UI
- keep visual polish constrained so it supports the product story instead of
  becoming an endless cosmetics pass

The working plan lives in
[`v0_5_public_release_plan.md`](../../working/v0_5_public_release_plan.md).

## P6 — `v0.6` Online Product Expansion

Turn the cloud foundation and stronger product identity into player-facing
online features.

### Goals

- make online human play real
- introduce trusted server-backed match records
- add public/shareable surfaces deliberately
- keep casual, private cloud, and trusted/ranked lanes distinct
- make sharing carry interesting moments, not just raw game records

### Possible Work

- direct challenge flow
- live PvP match state
- trusted match authority
- verified match persistence
- matchmaking if direct challenge proves too limited
- ranked mode and rating/leaderboard surfaces
- explicit replay publish flow
- public replay pages
- lightweight public identity / username surfaces
- game streaming or spectator-style viewing if it supports the product story

### Done When

Two people can reliably play a full online game, and the app can distinguish:

- local history
- signed-in private cloud history
- server-verified online/ranked history
- explicitly published public replays

### Production Lens

- test backend authority, realtime state, and public sharing only after the app
  has distinctive product moments worth preserving
- keep online work grounded in trust boundaries rather than generic social
  feature creep

## Non-Goals For Now

Called out so they do not quietly creep back into the near-term plan:

- native mobile apps
- chat/social feeds
- SSR/server-rendered app shell
- monetization
- ranked/esports depth before basic online play has a reason to exist
- forcing cloud or online into the default local flow

## Tracking

Keep progress lightweight:

- the repo history and deployed build tell most of the story
- use this file to keep phase intent and sequencing clear
- keep detailed temporary milestone notes in [`archive`](../../archive/)
- archive outdated exploratory docs instead of patching every one into
  permanence
