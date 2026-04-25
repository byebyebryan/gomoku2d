# Roadmap

The order we're going to build in.

Versions are the planning spine. A phase is done when it produces a coherent
playable/product state, not when it checks every possible sub-task. Patch
releases can polish or harden a phase, but they should not blur phase intent.

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

## P3 — `v0.3` BE Foundation And Cloud Continuity (current)

Add backend foundation without putting cloud in front of the local game.

Detailed working notes live in `docs/archive/v0_3_plan.md`. That file is an
ad-hoc planning artifact; this section remains the canonical roadmap.

### Goals

- optional sign-in
- cloud-backed profile
- continuity across browsers/devices
- durable private history beyond one browser/device
- basic backend plumbing that later online features can build on

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

## P4 — `v0.4` Online Product Expansion

Turn the cloud foundation into player-facing online features.

### Goals

- make online human play real
- introduce trusted server-backed match records
- add public/shareable surfaces deliberately
- keep casual and trusted/ranked lanes distinct

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
- shareable profile or match links if they earn their way in

### Done When

Two people can reliably play a full online game, and the app can distinguish:

- local guest history
- signed-in private cloud history
- server-verified online/ranked history
- explicitly published public replays

## P5 — `v0.5` Lab-Powered Features

Use the Rust bot lab as a visible product differentiator.

### Goals

- make saved games more useful after they end
- turn bot/search tooling into learning and replay features
- add features that would not exist without the Rust core/bot lab

### Possible Work

- replay analysis
- critical-moment tagging
- better-move suggestions
- puzzle generation from real or curated games
- "save this game" positions from losing replays
- stronger bot endpoint if needed for analysis

### Done When

At least one player-facing feature clearly exists because the bot lab can run
positions and searches outside the live board.

## Non-Goals For Now

Called out so they do not quietly creep back into the near-term plan:

- native mobile apps
- chat/social feeds
- SSR/server-rendered app shell
- monetization
- ranked/esports depth before basic online play works
- forcing cloud or online into the default local flow

## Tracking

Keep progress lightweight:

- the repo history and deployed build tell most of the story
- use this file to keep phase intent and sequencing clear
- keep detailed temporary milestone notes in `docs/archive/`
- archive outdated exploratory docs instead of patching every one into
  permanence
