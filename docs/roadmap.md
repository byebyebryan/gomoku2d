# Roadmap

The order we're going to build in.

This project is now explicitly sequenced around a local-first `v0.2`, not an
online-first pivot. The FE stack transition is still important, but it is now a
means to a richer local product rather than a straight runway into backend
work.

Phases are not time-boxed. A phase is done when it produces a coherent playable
state, not when it checks every possible sub-task.

## Phase 0 — Snapshot (`v0.1`, done)

Offline browser Gomoku with a Phaser-driven frontend and Rust+wasm game/core
integration.

What it proved:

- the Rust core + wasm bridge works in a real browser bundle
- the existing bot loop is good enough to build on
- the deploy pipeline is solid enough for fast iteration

Historical state is preserved in `docs/archive/progress_v0.1.md`.

## Phase 1 — FE foundation (done)

Establish the new runtime boundary:

- React shell
- Phaser reduced to a board-focused renderer
- local routes
- local stores
- board owned by props/events instead of a scene-driven app shell

This phase is effectively complete. Remaining work from here on should feed the
`v0.2` product pass instead of reopening the old "rewrite first, features
later" framing.

## Phase 2 — `v0.2` local-first product pass (current)

This is the current focus.

### Goals

- make the FE stack feel native instead of transitional
- establish a durable DOM-shell visual language
- simplify the live match UI around a board-first HUD
- deepen the local play loop with replay, records, and persistent defaults
- ship a public build that feels coherent as a local game

### Work in this phase

- rewrite the shell around the ongoing visual guide in `ui_design.md`
- simplify Home, Match, Replay, and Profile around their current roles
- introduce a compact icon language where it improves density without replacing
  plain-language CTAs
- keep chronology out of live match UI
- keep replay transport-first and remove move-list dependence there too
- allow a replay frame to branch into a new local practice game
- treat `Profile` as the player's local record screen, not a settings dump
- keep local profile, local history, replay, and rules switching polished
- give Match, Replay, and Profile intentional portrait/mobile layouts instead
  of simple desktop collapse behavior
- support a mobile-specific local play control model when direct tap placement is
  too cramped for the board
- make the shell resilient to future board-theme swaps without redesigning it

### Done when

A player can open the site and get a polished local experience:

- quick play from Home
- clean board-first match flow
- local replay that is useful without extra analysis surfaces
- local player record with persistent defaults and history
- mobile-web that feels intentionally designed on the key local screens, not
  just minimally responsive
- a consistent shell style that no longer feels like transitional scaffolding

Near-term release framing:

- `v0.2.3` established the paired desktop/mobile UI baseline for the
  local-first shell
- `v0.2.4` landed the final small polish and hardening pass on top of that
  baseline
- with `v0.2.4` in, the DOM shell is considered effectively frozen for the
  rest of `0.2.x`
- remaining `0.2.x` work should focus on non-UI fixes and stability, not
  reopening the mobile layout or control-model work

### Out of scope

- sign-in
- Firestore
- published replay links
- online matches
- analysis and puzzles

## Phase 3 — Cloud-backed continuity

Cloud comes back only after the local product is stable.

### Goals

- optional sign-in
- continuity across devices
- durable private history beyond one browser/device

### Work in this phase

- guest-to-cloud promotion
- cloud-backed profile sync
- private cloud history for signed-in players
- rules/settings sync if it still feels worthwhile

### Done when

Signing in extends the same local-first product without breaking it. A player
can keep their identity and history across browsers, but the app still makes
sense without cloud.

## Phase 4 — Shared replays and public identity

Only after private local/cloud history already feels good.

### Work in this phase

- explicit replay publish flow
- public replay pages
- lightweight public identity / username surfaces if needed

### Done when

Sharing a replay is deliberate and useful, without collapsing private history
and public artifacts into the same thing.

## Phase 5 — Online play

The big step after the app is already strong as a local product.

### Work in this phase

- trusted match authority
- direct challenge flow first
- live online match state
- verified match persistence

### Done when

Two people can reliably play a full online game without the app feeling like a
separate product from the local experience.

## Phase 6 — Lab-powered features

Only worth doing after the main play surfaces are already solid.

Possible work:

- replay analysis
- critical-moment tagging
- puzzle generation

These remain intentionally opportunistic. They should earn their way in by
making replay and learning more interesting, not by expanding scope for its own
sake.

## Non-goals for now

Called out so they do not quietly creep back into the near-term plan:

- native mobile apps
- chat/social systems
- SSR/server-rendered app shell
- monetization
- forcing cloud or online into the default local flow

## Tracking

Keep progress lightweight.

- the repo history and deployed build tell most of the story
- use this file to keep phase intent and sequencing clear
- archive outdated exploratory docs instead of patching every one into
  permanence
