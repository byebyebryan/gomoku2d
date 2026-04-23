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

**Theme: final `v0.2.4` polish pass + repo hygiene.**

`v0.2.4` is a deliberately narrow polish pass on top of the paired
desktop/mobile `v0.2.3` baseline. It is not a redesign; the working rule is
*board first, status second, controls third, meta last, no extra chrome.*
After it lands, the DOM shell is considered effectively frozen for the rest of
the `0.2.x` line — remaining work goes to non-UI fixes and stability. The bot
lab sees a small anti-blunder fix and a workspace-wide `rustfmt`.

### Design (web)

- Quieter small labels globally (eyebrows, section labels, mini field labels,
  table headers) so labels organize information instead of competing with
  values.
- Tightened spacing rhythm around the shared `4 / 8 / 12 / 16 / 24 / 32` scale
  to remove the last "assembled section by section" feel from desktop Match,
  Replay, and Profile.
- Standardized icon and button alignment (size, icon-to-label gap, left
  padding, vertical centering) so top nav and action buttons read as one
  system.
- Compressed the desktop Match right rail so it reads more like a HUD and
  less like a utility sidebar.
- Tightened the desktop Replay transport group into a single playback-console
  object, with `Play From Here` clearly secondary.
- Re-hierarchied desktop Profile: stats as hero, identity/settings demoted,
  history softened from a data-table feel into a player-record ledger.
- Dropped the redundant `Replay timeline` subtitle from the replay transport
  zone.
- Locked the icon system to a uniform `24px` render, matching the authored
  pack's native scale.

### Added

**Web**

- Favicon assets.

### Changed

**Web**

- Copy polish on screen labels; renamed the local opponent to `Practice Bot`.

**Bot lab**

- Hardened the baseline bot's anti-blunder search.
- Formatted the Rust workspace with `rustfmt`.

### Docs

- Refreshed canonical doc set (`product.md`, `architecture.md`, `design.md`,
  `backend.md`, `roadmap.md`, `visual_design.md`, `visual_review.md`, root
  `README.md`, `gomoku-web/README.md`) to match the shipped v0.2 product and
  align release framing around the upcoming UI freeze.

### Tooling

- Added `LICENSE` (MIT), `CHANGELOG.md`, CI workflow (rustfmt / clippy /
  cargo test / tsc / vitest / production build), `.editorconfig`,
  `rust-toolchain.toml` pinning `stable` + `rustfmt` + `clippy` +
  `wasm32-unknown-unknown`, Node version pin (`.nvmrc` + `engines`),
  Dependabot config (cargo + npm + github-actions, with npm ecosystem groups
  for React / Vite / testing), PR template.
- Committed `gomoku-bot-lab/Cargo.lock` for reproducible builds of the
  workspace binaries.

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
  `architecture.md`, `design.md`, `backend.md`, `roadmap.md`.
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

[Unreleased]: https://github.com/byebyebryan/gomoku2d/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/byebyebryan/gomoku2d/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/byebyebryan/gomoku2d/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/byebyebryan/gomoku2d/compare/v0.1...v0.2.1
[0.1]: https://github.com/byebyebryan/gomoku2d/releases/tag/v0.1
