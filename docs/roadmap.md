# Roadmap

The order we're going to build in.

Versions are the planning spine. A phase is done when it produces a coherent
playable/product state, not when it checks every possible sub-task. Patch
releases can polish or harden a phase, but they should not blur phase intent.

This roadmap tracks two things at once:

- **Product story**: each phase should make Gomoku2D more credible as a real
  alpha/beta product.
- **Process story**: each phase should teach something useful about building a
  product with AI agents as active collaborators.

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

Process lens:

- prove whether an old native/game-logic idea can be revived into a browser
  product without throwing away the original technical core
- establish that repo docs, code review, and release notes can keep pace with
  rapid AI-assisted iteration

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
- local guest profile and preferred-rule persistence
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

### Process Lens

- learn how far agents can help shape a frontend product, not just generate UI
  code
- establish screenshot review, asset preview, animation inventory, and release
  polish loops as first-class project practices
- validate the React/Phaser/Rust boundaries that let agents work in one layer
  without constantly breaking another

## P3 — `v0.3` BE Foundation And Cloud Continuity (current)

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
- guest-to-cloud promotion
- preferred settings/profile sync
- private cloud history for signed-in players
- cloud-saved private replay loading
- starter Firestore rules for owner-scoped profile/history docs

### Current State

The first backend-foundation slice is in place:

- Firebase/GCP project, Firebase web app, and env-driven web bootstrap
- Firestore `(default)` in `us-central1`
- owner-scoped Firestore rules for `profiles/{uid}` and private match docs
- Google Auth provider configured through the Firebase Auth / Identity Toolkit
  path
- Profile sign-in/sign-out UI
- cloud profile create/load at `profiles/{uid}`
- local guest play/history still working without Firebase config
- infra and free-tier tracking split into `backend_infra.md` and
  `backend_cost.md`

The remaining `v0.3` work is product continuity rather than raw setup:

- verify deployed-site sign-in after the next tagged deploy
- confirm production-build config gating and review Firebase/Firestore usage
  dashboards after the first cloud-profile smoke test
- publish the OAuth app from Testing to In production when public sign-in is
  intended
- import local guest profile/history into cloud state idempotently
- save future signed-in casual matches privately to Firestore
- load cloud-saved private history/replays from Profile

### Done When

Signing in extends the same local-first product without breaking it:

- guest-only play remains complete
- signed-in profile/history works across browsers
- local guest history can be promoted without duplicates
- future signed-in matches save privately to cloud
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

### Process Lens

- learn the practical boundary between local-first UX and cloud-backed
  persistence
- establish a safe secrets/config/docs workflow for GCP/Firebase work
- keep cloud setup, cost estimates, auth caveats, and release gates documented
  well enough that future agents can continue without guessing

## P4 — `v0.4` Lab-Powered Product Identity

Make the Rust lab visible as a product differentiator.

### Goals

- make saved games useful after they end
- turn bot/search tooling into player-facing learning and challenge features
- give Gomoku2D a reason to exist beyond retro styling and basic play
- keep the lab-to-product loop tight: core/bot findings should become UI
  features without a rewrite

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

### Process Lens

- test agent-assisted work across the hardest boundary in the repo: Rust search
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

### Process Lens

- test whether agents can extend a visual system without flattening it into
  generic UI
- learn how much design direction, asset tooling, and screenshot review are
  needed to keep AI-assisted frontend polish coherent
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

- local guest history
- signed-in private cloud history
- server-verified online/ranked history
- explicitly published public replays

### Process Lens

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
