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

The project thesis behind that split lives in `project.md`.

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

Historical state is preserved in `docs/archive/progress_v0.1.md`.

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
- mobile touch-pad placement mode for cramped boards
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

Detailed working notes live in `docs/archive/v0_3_plan.md`. That file is an
ad-hoc planning artifact; this section remains the canonical roadmap.

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
- Firebase Auth popup/redirect handling: desktop tries popup first, mobile and
  embedded contexts use redirect, and popup-blocked/unsupported errors fall back
  to redirect without retrying intentional popup closes
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
- compact, versioned private match schema documented in `data_model.md`
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
- infra and free-tier tracking split into `backend_infra.md` and
  `backend_cost.md`

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

## P4 — `v0.4` Lab-Powered Product Identity (current)

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

### Current Sequencing

`0.4.0` is a foundation release, not the first visible analysis feature. The
initial bot experiments showed that style labels and tactical shortcuts are easy
to invent but hard to justify. The release therefore focuses on:

- explicit bot config and lab specs
- tactical scenario diagnostics
- multi-threaded tournament/report output
- search-pipeline metrics and ablation vocabulary
- behavior-preserving core/search hot-path optimizations
- written records of rejected tactical experiments

`0.4.1` uses that foundation for a narrower tactical ranking / move-ordering
pass. The target is not "make one depth-2 fixture pass"; it is better reached
depth, runtime, or match strength under the same budget.

The current `0.4.1` checkpoint has produced and published the first clean
8-entrant reference report: depth ladder, tactical-cap hard variants, and
pattern-eval ablations under the same Renju rule, centered opening suite, and
`1000 ms` CPU-per-move budget. The practical read is:

- `search-d1` is now a plausible easy/beginner lane because the safety gate
  covers hard local threats even at shallow depth.
- `search-d3` remains the stable default practice-bot baseline.
- `search-d5+tactical-cap-8` is the efficient hard-side candidate.
- `search-d7+tactical-cap-8` is stronger, but spends more budget.
- `+pattern-eval` is promising enough to keep as a lab axis, but too expensive
  and unsettled to promote as a default product preset.

That became the coherent `0.4.1` checkpoint: bot ladder, report, tactical
pipeline, and enough evidence to stop guessing at bot labels.

`0.4.2` stayed in the lab for one more pass before UI. The harness was strong
enough to justify a second measured pass, and product presets will be cleaner
if they come from evidence rather than from raw knobs. The intended order was:

- tune existing axes first: depth, child cap, candidate source, and pattern eval
- prototype bounded corridor search second, using only concrete local
  gain/defense replies with strict caps and explicit non-alpha-beta metrics
- treat style/character last; offensive/defensive labels should emerge from
  real budget allocation, not from ad hoc eval weights

The first `0.4.2` sweeps covered child-cap, pattern-eval, symmetric candidate
radius, and one asymmetric candidate source. They narrowed the bot question
rather than closing it: pattern eval is still the strongest signal but remains a
cost tradeoff, cap16 is not a general upgrade, cap4 is viable when paired with
tactical ordering, and `self2/opponent1` mainly looks useful as an efficiency
tweak for `D3 + pattern-eval`. No anchor promotion yet; save another bot sweep
for when we need to choose concrete product presets.

`0.4.2` then pivoted from "stronger bot right now" to something more important:
[`corridor search`](corridor_search.md) as a shared strategic model for
analysis, bot diagnostics, and future player education. The bot sweeps showed
that obvious tuning gains are getting expensive. A better next step is to
understand why a competent bot wins or loses, where the last escape existed,
and which forced sequence made the position collapse.

The current analyzer is still a lab artifact, not a replay-screen feature, but
it is the first serious version of that model. It uses bounded corridor proof to
explain the final forced sequence in finished tournament games, records proof
intervals instead of assuming one monotonic turning point, and separates ideal
play from human mistakes such as missed defenses, missed wins, possible
escapes, tactical errors, and strategic losses.

This matters beyond the interim report: corridor search gives replay analysis a
concrete foundation, formalizes the project's advanced-strategy vocabulary,
gives the lab a way to inspect bot behavior beyond Elo, can feed back into bot
search as focused forcing logic, and points toward a product identity where
Gomoku2D can teach the game it is playing.

The current public checkpoint is the curated top-two replay analysis report
published at `/analysis-report/`, generated from the current bot report. That is
the coherent `0.4.2` checkpoint: bot sweep evidence, a presentable corridor
analysis workbench, and a real strategic foundation for deciding what should
become player-facing later.

`0.4.3` stayed in the lab for one more corridor-search pass before UI plumbing.
Corridor search is now too central to the advanced-strategy story to expose only
as an analyzer report and then immediately jump to settings. The checkpoint
answered the first live-bot integration question: scan-backed corridor portals
are useful plumbing and instrumentation, but not yet a promotable strength
feature. The durable output is unified tactical/corridor threat semantics,
move-local portal entry detection, asymmetric own/opponent portal controls, and
metrics that make the next cost problem visible.

That release remains lab-first:

- reuse corridor-search vocabulary and tactical facts inside bot experiments
  without turning the bot into a full solver
- measure strength, search cost, and loss categories against the current
  published anchors
- reinforce or optimize corridor search where the bot integration exposes
  obvious cost or correctness gaps
- treat offensive/defensive behavior as side-specific corridor budget allocation,
  not as arbitrary eval personality weights
- keep UI, settings, and product preset work out of scope unless the lab result
  is clear enough to name

`0.4.4` is the rolling-frontier lab pass, not the UI bridge. Corridor portals
need cheap, reliable local threat facts before they can be a practical bot
primitive. The `0.4.3` scan-backed portal suffixes now provide move-local entry
semantics and useful cost metrics, but focused smoke checks are still slower,
weaker, and budget-bound. The durable `0.4.3` result is therefore the
scan-backed `ThreatView` seam and unified threat vocabulary, not a promoted
corridor bot.

The `0.4.4` checkpoint treats rolling frontier as a correctness-sensitive cache
architecture:

- keep `Board` as the authority for stones, turn, result, and exact legality;
- normalize tactical facts so scan-backed and cached views can be compared
  exactly;
- add a rolling `ThreatView` implementation that updates alongside apply/undo;
- validate it against `ScanThreatView` on tactical fixtures, random sequences,
  and Renju forbidden cases;
- run it in shadow mode before promoting hot-path behavior;
- promote rolling-backed tactical ordering and current-obligation safety as the
  default threat-view backend after focused parity and smoke metrics are clean;
- keep scan-backed threat view as an explicit fallback/comparison suffix;
- treat relaxed/no-budget scan-vs-rolling runs as the semantic parity check, and
  normal-budget runs as budget-interaction/cost evidence.

The working plan lives in `docs/archive/v0_4_4_frontier_plan.md`.

`0.4.5` becomes the earliest likely UI bridge for bot controls/settings. Expose
only knobs that have survived lab evidence. A reasonable product-facing starting
point is still an easy/default/hard ladder backed by reports, but corridor-aware
bot behavior may change what those labels mean. Keep raw pattern-eval,
child-cap, corridor-search, and shadow/scan diagnostic knobs lab-only until they
become product language. Profile should not become a dumping ground for
bot/debug preferences.

Later `0.4.x` slices can compete based on which lab-powered product surface
feels strongest.

### Possible Work

- replay analysis with critical-moment tagging
- better-move suggestions and opponent best-reply previews
- generated puzzles from real games or curated positions
- "save this game" challenges from losing positions
- bot personalities or customizable practice bot settings
- benchmark-backed bot presets that feel meaningfully different
- stronger bot endpoint only if browser-side wasm is not enough for the chosen
  product surface

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

## P5 — `v0.5` Presentation Systems And Skins

Use the frontend foundation to broaden the product's visual range without
losing the board-first design.

### Goals

- support distinct visual skins or theme sets
- keep the default retro pixel-art identity, but add room for a more serious or
  quieter presentation
- make the product feel more intentional and less locked to one aesthetic
- improve showcase/onboarding surfaces once lab-powered features exist

### Possible Work

- theme/skin tokens that cover DOM shell, board colors, sprites, and previews
- one alternate "serious board" skin, if it earns its way in
- theme-aware asset preview pages
- home/profile/replay copy and layout polish around the stronger product story
- screenshot and release-asset refresh for the new product identity

### Done When

The app can change visual tone without becoming a different product, and the
skin system proves the FE stack can support more than one presentation layer.

### Production Lens

- test whether agents can extend a visual system without flattening it into
  generic UI
- learn how much design direction, asset tooling, and screenshot review are
  needed to keep AI-centric frontend polish coherent
- keep theme work constrained so it supports the product story instead of
  becoming an endless cosmetics pass

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
- keep detailed temporary milestone notes in `docs/archive/`
- archive outdated exploratory docs instead of patching every one into
  permanence
