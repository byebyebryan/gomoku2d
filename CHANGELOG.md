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

[Unreleased]: https://github.com/byebyebryan/gomoku2d/compare/v0.3.3...HEAD
[0.3.3]: https://github.com/byebyebryan/gomoku2d/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/byebyebryan/gomoku2d/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/byebyebryan/gomoku2d/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/byebyebryan/gomoku2d/compare/v0.2.4...v0.3.0
[0.2.4]: https://github.com/byebyebryan/gomoku2d/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/byebyebryan/gomoku2d/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/byebyebryan/gomoku2d/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/byebyebryan/gomoku2d/compare/v0.1...v0.2.1
[0.1]: https://github.com/byebyebryan/gomoku2d/releases/tag/v0.1
