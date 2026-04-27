# v0.3 Plan

Status: ad-hoc milestone plan. This file is a working contract for the `v0.3`
line, not a canonical product spec. Keep `docs/roadmap.md`, `docs/backend.md`,
`docs/backend_infra.md`, `docs/backend_cost.md`, and `docs/app_design.md` as
the long-lived sources of truth.

## Frame

`v0.2` made Gomoku2D feel like a complete local-first game: board-first match,
local replay/history, guest profile, mobile control model, polished shell, and
asset/release workflows.

`v0.3` should not reopen the shell or jump straight to public/social features.
It is the **backend foundation** phase: optional sign-in, cloud profile, private
history, and enough backend plumbing for later online features, while preserving
the local-first product from `v0.2`.

## Product Goal

A player can:

- open the site and play as a guest exactly as in `v0.2`
- build local match history without signing in
- sign in from Profile when they decide continuity is worth it
- promote their local guest profile and finished local matches into cloud state
- continue playing with future finished matches saved privately to cloud
- open their cloud-backed history/replays from another browser

The app should still make complete sense if the player never signs in.

## Explicit Non-Goals

- no sign-in wall before play
- no public replay sharing
- no username reservation or public profile URL unless required for auth polish
- no live online matches
- no trusted match authority / Cloud Run hot path
- no leaderboards
- no puzzles
- no replay analysis

Those belong to later phases after private cloud continuity feels solid.

## Scope

### 1. Firebase Preflight

- decide dev/prod Firebase project shape
- document required `VITE_FIREBASE_*` env vars
- add Firebase SDK dependencies using modular imports only
- add a thin Firebase client module in `gomoku-web`
- add starter Firestore rules for owner-scoped profile/history docs
- keep config public and secrets out of the repo

Initial state as of the first `v0.3` pass:

- use one Firebase/GCP project for now: `gomoku2d`
- Firebase project and web app are initialized
- Firebase/Auth/Firestore APIs are enabled
- Auth config is initialized; Google provider is enabled with a Google Auth
  Platform Web OAuth client
- web Firebase config is env-driven and optional at runtime
- Firestore is created as the default Native-mode database in `us-central1`
- Firestore rules are in repo and deployed to the `cloud.firestore` release
- live setup tracking lives in `docs/backend_infra.md`
- backend cost tracking lives in `docs/backend_cost.md`

Checkpoint after the first implementation slice:

- Auth state is wired through the web app and remains inert when Firebase env
  vars are absent.
- Profile can sign in/out with Google and create/load the cloud profile at
  `profiles/{uid}`.
- The first live `profiles/{uid}` document has been observed in Firestore via
  the REST API.
- Local guest profile/history remain the visible record; the UI explicitly
  says local history stays local until promotion ships.
- Local popup-auth headers and SSH port-forwarding guidance are documented.
- The public access gate is now explicit: an `External` OAuth app in `Testing`
  only works for configured test users until it is published to production.
- Public access has since been flipped to `In production`, with the OAuth logo
  intentionally left blank to avoid brand verification for this first cloud
  profile slice.
- The production domain, policy pages, contact email, public sign-in smoke, and
  no-config fallback smoke are recorded in `docs/backend_infra.md` and
  `docs/archive/v0_3_completion_plan.md`.

### 2. Auth State Layer

- add auth-state subscription
- expose sign-in and sign-out actions
- support Google sign-in first
- add GitHub sign-in only if it stays small
- keep auth UI on Profile; do not alter Home/Match entry friction
- handle popup-blocked, canceled, offline, and loading states cleanly

### 3. Cloud Profile

- create or load `profiles/{uid}` after sign-in
- seed display name/avatar/provider metadata from Firebase provider data
- preserve app-owned display name separately from provider display name
- sync preferred rule/settings if still useful
- make Profile clearly show local-only vs cloud-backed state

### 4. Guest-To-Cloud Promotion

- import local guest profile data once after first sign-in
- import finished local matches into `profiles/{uid}/matches/{id}`
- use stable local-origin IDs so retrying promotion is idempotent
- do not delete local guest history until the cloud import is confirmed
- make duplicate prevention testable

### 5. Private Cloud History

- save newly finished signed-in matches to `profiles/{uid}/matches/{id}`
- load signed-in match history on Profile
- keep local guest history and cloud history conceptually separate
- route cloud replay viewing without introducing public replay URLs
- label records enough to distinguish local-only and cloud-backed origins

### 6. Hardening

- offline/error state review
- bundle-size check after Firebase imports
- smoke tests for guest play still working
- tests for promotion dedupe and history serialization where practical
- manual desktop/mobile Profile review

## Suggested Release Slices

### `0.3.0` — Auth + Cloud Profile

- Firebase config plumbing
- auth state store
- Profile sign-in/sign-out
- cloud profile create/load
- no history promotion yet

This is implemented and validated. Remaining work before tagging is release
mechanics: version/changelog/docs prep, checks, `v0.3.0` tag, and release
deploy verification.

### `0.3.1` — Guest Promotion

- import local guest profile + finished local matches
- idempotent local-origin IDs
- clearer Profile copy for local vs cloud-backed state

### `0.3.2` — Private Cloud History

- save future signed-in matches to cloud
- load cloud match history on Profile
- replay cloud-saved private matches

The exact split can change, but avoid bundling auth, import, and history sync
into one hard-to-debug release.

## Acceptance Criteria For The `0.3` Line

- Guest-only play remains as fast and complete as `v0.2`.
- Signing in is optional and discoverable from Profile.
- A signed-in player can keep identity, preferred settings, and private match
  history across browsers.
- Guest promotion is idempotent and does not duplicate imported matches.
- Public artifacts are not created implicitly.
- The docs keep local guest history, cloud private history, and future public
  replay sharing as separate concepts.
- Later lab-powered and online work has a clear backend/auth/history foundation
  to build on.

## Risks

- Firebase SDK bundle growth can be noticeable next to the already-large Phaser
  chunk. Use modular imports and check build output after the first integration.
- Auth popups have browser-specific failure modes. Profile needs clear fallback
  copy and retry states.
- Promotion bugs are easy to hide until real local history exists. Stable IDs
  and retry-safe writes matter more than fancy UI.
- Cloud history can blur with future public replay sharing. Keep the `v0.3`
  language private-only unless explicitly entering a later publish/share phase.
