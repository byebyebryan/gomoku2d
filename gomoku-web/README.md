# gomoku-web

The browser app for Gomoku2D. React owns application state and DOM surfaces,
Phaser owns the board canvas, and Rust/Wasm supplies rules, bot moves, tactical
facts, and replay analysis.

**Live app:** https://gomoku2d.byebyebryan.com/

[Dev site](https://gomoku2d.byebyebryan.com/dev/)

The developer-area changes are local 0.6.3 source until release;
[the contract](../docs/reference/app/developer_area.md) records their scope.

## Responsibilities

- Home, local match, replay, profile, settings, rules, guide, developer references,
  privacy, and terms routes.
- Local-first guest play with browser-persisted profile/settings/history.
- Optional Google sign-in for private cloud-backed profile/history continuity.
- Self-contained links to finished games, with explicit local replay saves and
  account-library import/sync.
- Continuations from shared replays that enter ordinary My games and totals
  while retaining their inherited-move provenance.
- Opt-in public player profiles with explicitly shared dated statistics and a
  paged catalog of games the player chose to publish.
- Guide help for replay actions and two static bot-generated examples in the
  existing shared viewer, without sign-in or a new backend.
- Configurable bot presets and advanced web-safe bot controls.
- Freestyle/Renju play, Renju forbidden-move hints, tactical hints, and mobile
  touch placement.
- Replay Analysis in the browser: progressive wasm analysis, timeline markers,
  board overlays, local result caching, and "Play From Here" branching.
- [Bot lab](https://gomoku2d.byebyebryan.com/lab/) at `/lab/`, rendered from curated
  JSON artifacts under `../reports/lab/`.
- [Visual design](https://gomoku2d.byebyebryan.com/visuals/) at `/visuals/` for
  source icons, sprites, fonts and design tokens, and
  [UI design](https://gomoku2d.byebyebryan.com/dev/reviews/core-screens-062/)
  starting at `/dev/reviews/core-screens-062/`, with UI history and design reviews
  alongside the current screens.

## Core Boundary

The UI presents game facts; it does not invent them. Rule legality, tactical
semantics, bot decisions, and replay-analysis annotations come from Rust
through the wasm bridge. TypeScript validates those payloads and turns them
into product state, copy, and visuals.

## Runtime Shape

```text
React routes/stores
  -> BoardViewModel
    -> Phaser board scene
  -> wasm bridge
    -> gomoku-core / gomoku-bot / gomoku-analysis
  -> Web Workers
    -> bot search and replay analysis off the UI thread
```

The active match and replay flows intentionally stay local-first. Cloud is a
continuity layer for signed-in private history, not a requirement for playing.

Production builds target `esnext` and use native top-level await to load Wasm
in the app and ES workers. The build needs `vite-plugin-wasm`; it does not need
a separate top-level-await transform.

Dynamic imports load their JavaScript graph directly, with CSS preloaded. This
avoids WebKit retaining failed JavaScript modulepreloads across a reload; the
route recovery checks cover retrying both route and shared dependency failures.

The support baseline is a modern browser with WebAssembly, ES module workers,
and native top-level await. CI exercises its installed Chromium, Firefox and
WebKit versions. The October layout review covers widths from 320 CSS pixels;
actual mobile-device, zoom and screen-reader acceptance are explicitly deferred
in the reconciliation plan. Browser journeys include independent axe-core audits
of ten routes at desktop and portrait sizes; inconclusive checks are attached
to test results and do not count as human acceptance.

## Source Layout

```text
src/
├── app/          React entry, router, global CSS tokens
├── routes/       product screens and page-level copy
├── components/   reusable UI, including the Board wrapper
├── board/        Phaser scene, renderer, overlays, input mapping
├── core/         wasm bridge, bot worker protocol, bot runner
├── game/         local match session/store, clocks, hints, undo, save
├── profile/      local profile/settings/history persistence
├── cloud/        Firebase auth/profile/history sync
├── match/        saved-match schema, shared history model and helpers
├── replay/       replay frames, wasm analyzer runner, overlays, cache
└── ui/           icon registry and shared UI helpers
```

Static/publishing scripts live in `scripts/`. Source art lives in `assets/` and
is published into the build by the postbuild scripts.

## Local Development

Prerequisites: Node 24, Rust, and `wasm-pack`.

From the repo root, build the wasm package once:

```sh
wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
```

Then run the web app:

```sh
cd gomoku-web
npm ci
npm run dev
```

After editing Rust code used by wasm, rebuild `gomoku-wasm` and rerun
`npm install` in `gomoku-web` so Vite picks up the relinked local dependency.

Firebase is optional for local development. Guest play works without env vars.
To test cloud-backed profile/history, copy `.env.example` to `.env.local` and
fill the public Firebase web-app config:

```sh
cp .env.example .env.local
```

Required Vite vars:

- `VITE_FIREBASE_API_KEY`
- `VITE_FIREBASE_AUTH_DOMAIN`
- `VITE_FIREBASE_PROJECT_ID`
- `VITE_FIREBASE_STORAGE_BUCKET`
- `VITE_FIREBASE_MESSAGING_SENDER_ID`
- `VITE_FIREBASE_APP_ID`

The current Firebase project is `gomoku2d`. Infra notes live in
[`../docs/reference/ops/backend_infra.md`](../docs/reference/ops/backend_infra.md).

## Common Commands

```sh
npm run typecheck          # TS plus checked JS scripts
npm test                   # Vitest
npm run test:rules         # Firestore rules through Firebase emulators
npm run test:cloud         # isolated two-client Auth/Firestore acceptance
npm run build              # production build plus postbuild publishing
npm run preview            # serve dist/ locally
npm run playtest:smoke     # focused automated Playwright smoke
npm run playtest:engines   # full Chromium, Firefox and WebKit journeys
npm run profile:browser    # opt-in production browser cost/resource samples
npm run media:readme       # regenerate README/showcase media from preview
```

For only the profile/sharing/replay assets, use the reviewed local demo and an
explicitly owned loopback preview:

```sh
GOMOKU_MEDIA_BASE_URL=http://127.0.0.1:8013 GOMOKU_MEDIA_PUBLIC_PROFILE_ID=v061-demo-player npm run media:sharing
```

This command captures existing UI; it does not seed or modify cloud records.
See [media provenance](../docs/assets/README.md) and the
[example corpus contract](../docs/reference/app/replay_examples.md).

The `postbuild` step publishes visual assets, curated lab reports, SPA route
entries, and `404.html`. Production builds expect these report artifacts:

- `../reports/lab/bot-report.json` -> `/bot-report/report.json`
- `../reports/lab/analysis-report.json` -> `/analysis-report/report.json`

Set `GOMOKU_ALLOW_MISSING_REPORTS=1` only for local/dev builds that
intentionally skip report pages.

## Deploy

Release and local-preview details live in
[`../docs/reference/ops/release.md`](../docs/reference/ops/release.md).

Production deploys to GitHub Pages when a `v*` tag is pushed. Normal commits to
`main` run CI but do not publish the site. The deploy workflow builds wasm,
sets `GOMOKU_BASE_PATH=/` for the custom domain, and publishes `dist/`.

## Reference Docs

- [`../docs/reference/app/code_overview.md`](../docs/reference/app/code_overview.md)
- [`../docs/reference/app/architecture.md`](../docs/reference/app/architecture.md)
- [`../docs/reference/app/app_design.md`](../docs/reference/app/app_design.md)
- [`../docs/reference/app/ui_design.md`](../docs/reference/app/ui_design.md)
- [`../docs/reference/app/game_visual.md`](../docs/reference/app/game_visual.md)
- [`assets/README.md`](assets/README.md)
