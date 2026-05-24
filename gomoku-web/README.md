# gomoku-web

The browser product surface for Gomoku2D: a retro-feeling board game shell on
top of React, Phaser, and the Rust/WebAssembly core.

**Play:** https://gomoku2d.byebyebryan.com/

**Pixel-art previews:** https://gomoku2d.byebyebryan.com/assets/

**Lab report:** https://gomoku2d.byebyebryan.com/lab-report/

React owns the app shell: home, match, replay, profile, auth, and local/cloud
history. Phaser renders the board and nothing else. The rules engine and bot
are the same Rust code used by the native bot lab in this repo, compiled to
Wasm and called from JS. The bot runs in a Web Worker so it can think without
freezing the UI.

---

## Product Surface

This package owns the browser app:

- Start a match from Home with one click against the saved bot setup
- Tune rule, bot, hints, and touch controls from Settings; changes mid-game can
  queue for the next round or start a new game
- Live forbidden-move warnings when playing Black under Renju
- Optional tactical hints for immediate and imminent threats
- Undo the last turn during a live match
- Finish a match, open the replay, step through turns with browser-side corridor
  analysis, then branch off at any point to play the rest against the current bot
  setup without undoing before the branch point
- Local guest profile: display name and recent-match history —
  persisted in browser storage, no sign-in required
- Optional Google sign-in for private cloud-backed profile/history continuity
  across browsers
- Desktop and portrait/mobile layouts are intentional rather than collapsed —
  mobile uses a dedicated touch-placement flow instead of direct tap-to-place

---

## What gives it character

- Pixel art sprites with frame-by-frame animations — stones form and shatter,
  winning cells pulse, a hover pointer cycles through idle states
- Board-first layouts: slim HUD on match, transport deck on replay, no move
  list during live play
- Icon language for desktop actions and replay transport, kept monochrome and
  scoped so it doesn't become a separate skin
- Responsive: the board fits its available space on any viewport; portrait
  layouts are screen-specific rather than collapsed desktop

Design intent is split across:

- [`../docs/reference/app/app_design.md`](../docs/reference/app/app_design.md) — routes, flows, and screen contracts
- [`../docs/reference/app/ui_design.md`](../docs/reference/app/ui_design.md) — DOM shell visual language
- [`../docs/reference/app/game_visual.md`](../docs/reference/app/game_visual.md) — Phaser canvas visuals, sprite roles, and animation language

Source assets and local visual preview pages live in
[`assets/README.md`](assets/README.md). Published builds expose those previews
under `/assets/`.

Curated bot-lab reports live in [`../gomoku-bot-lab/reports/`](../gomoku-bot-lab/reports/)
and are copied into published builds under `/bot-report/` as the ranking/search
data source.

Curated replay-analysis reports live in
[`../gomoku-bot-lab/analysis-reports/`](../gomoku-bot-lab/analysis-reports/)
and are copied into published builds under `/analysis-report/` as the analysis
data source.

Production builds expect both curated report folders to contain `report.json`.
The React app renders the unified `/lab-report/` from those split data files.
For local/dev builds that intentionally skip reports, set `GOMOKU_ALLOW_MISSING_REPORTS=1`.

---

## Stack

| Layer | Tech |
|-------|------|
| App shell | React 19 |
| Routing | React Router 7 |
| Client state | Zustand 5 (vanilla stores + `useStore` selectors) |
| Board renderer | Phaser 4 (canvas, stateless view) |
| Language | TypeScript 6 |
| Build / dev server | Vite 8 (+ `vite-plugin-wasm`, `vite-plugin-top-level-await`) |
| Game logic + bot | Rust (`gomoku-core`, `gomoku-bot`) → `wasm-pack --target bundler` |
| Bot execution | Web Worker (off-thread) |
| Unit tests | Vitest + Testing Library |
| End-to-end smoke | Playwright |

Styling is CSS Modules (`*.module.css`) with a shared token layer in
`src/app/global.css`. No CSS framework.

---

## Source layout

```
src/
├── app/            React entry (App.tsx, routes, global tokens)
├── routes/         Home, LocalMatch, Profile, Replay, Settings
├── components/     Reusable UI (Board wrapper around Phaser)
├── board/          Phaser scene, renderer, board constants
├── cloud/          Firebase config/bootstrap for cloud-backed profile/history
├── game/           Local match Zustand store + shared types
├── profile/        Local profile Zustand store (persisted to localStorage)
├── replay/         Replay frames, core conversion, and browser analysis runner
├── core/           Wasm bridge + bot worker protocol/runner
└── ui/             Icon component + icon registry
```

Routes:

- `/` — title screen, single `Play` CTA
- `/match/local` — live match vs the configured bot
- `/replay/:matchId` — replay viewer with browser-side corridor analysis for decisive saved matches
- `/profile` — local/cloud player record and history
- `/settings` — rule, bot, hint, and touch-control setup
- `/privacy/` and `/terms/` — static Privacy and Terms pages for the public app

---

## Local development

Prerequisites: Node 24 (see repo-root `.nvmrc`), Rust, `wasm-pack`.

```sh
# 1. Build the Wasm package (from repo root)
wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler

# 2. Run the dev server
cd gomoku-web
npm install
npm run dev
```

TypeScript changes hot-reload. After editing Rust, rebuild the Wasm package and
re-run `npm install` so Vite picks up the relinked `file:` dependency.

Firebase is optional during local development. Guest/local play works without
any Firebase env vars. To enable cloud-backed profile/history, copy the
example file and fill the public web-app config from Firebase:

```sh
cp .env.example .env.local
```

Required Vite env vars:

- `VITE_FIREBASE_API_KEY`
- `VITE_FIREBASE_AUTH_DOMAIN`
- `VITE_FIREBASE_PROJECT_ID`
- `VITE_FIREBASE_STORAGE_BUCKET`
- `VITE_FIREBASE_MESSAGING_SENDER_ID`
- `VITE_FIREBASE_APP_ID`

The current Firebase project is `gomoku2d`. Live setup details and API-based
config fetch commands live in [`../docs/reference/ops/backend_infra.md`](../docs/reference/ops/backend_infra.md).
CI and tag deploy builds include these public config values so the released app
can initialize Firebase.

```sh
npm run build              # production build + static report/asset routes
npm run preview            # serve the production build locally
npm test                   # vitest
npm run typecheck:scripts  # JS script type coverage
npm run playtest:smoke     # playwright smoke run
```

The `postbuild` step publishes asset previews, curated bot and analysis reports,
static SPA route entries, and the `404.html` fallback. Report publishing is
guarded so release builds fail if the curated artifacts are missing; set
`GOMOKU_ALLOW_MISSING_REPORTS=1` only for local/dev builds that intentionally
skip those pages.

---

## Deploy

Release and local-preview steps live in [`../docs/reference/ops/release.md`](../docs/reference/ops/release.md).

Production deploys to GitHub Pages only when a `v*` tag is pushed. Normal
commits to `main` run CI but do not publish the site.

The workflow builds the Wasm package, sets `GOMOKU_BASE_PATH=/` for the custom
domain Vite build, and deploys `dist/` to Pages.

---

## Where This Fits

The game is the top-level product; the Rust side in `gomoku-bot-lab/` is a
supporting workspace. The bot you play against in the browser is the same code
you can pit against itself from the command line — `gomoku-wasm` exposes it to
JS and this package calls it through a Web Worker.

```
gomoku-web                     — this package
gomoku-bot-lab/gomoku-core     — board, rules, Renju enforcement, replay format
gomoku-bot-lab/gomoku-bot      — Bot trait + implementations (RandomBot, SearchBot, …)
gomoku-bot-lab/gomoku-analysis — shared replay-analysis model and corridor traceback
gomoku-bot-lab/gomoku-eval     — self-play arena, tournaments, Elo
gomoku-bot-lab/gomoku-cli      — CLI match runner with replay export
gomoku-bot-lab/gomoku-wasm     — wasm-pack bridge: WasmBoard + WasmBot + replay analyzer for JS
```

For product sequencing, see
[`../docs/reference/product/roadmap.md`](../docs/reference/product/roadmap.md).
For the React/Phaser/Rust boundary, see
[`../docs/reference/app/architecture.md`](../docs/reference/app/architecture.md).
