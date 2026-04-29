# gomoku-web

The browser product surface for Gomoku2D: a retro-feeling board game shell on
top of React, Phaser, and the Rust/WebAssembly core.

**Play:** https://gomoku2d.byebyebryan.com/

**Pixel-art previews:** https://gomoku2d.byebyebryan.com/assets/

React owns the app shell: home, match, replay, profile, auth, and local/cloud
history. Phaser renders the board and nothing else. The rules engine and bot
are the same Rust code used by the native bot lab in this repo, compiled to
Wasm and called from JS. The bot runs in a Web Worker so it can think without
freezing the UI.

---

## What works today

Single-player, local-first:

- Start a match from Home with one click — opponent is the Practice Bot
- Switch between Freestyle and Renju rules; changes mid-game queue for the next
  round
- Live forbidden-move warnings when playing Black under Renju
- Undo the last turn during a live match
- Finish a match, open the replay, scrub move by move, then branch off at any
  point to play the rest against the bot yourself without undoing before the
  branch point
- Local guest profile: display name, preferred rule, recent-match history —
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

- [`../docs/app_design.md`](../docs/app_design.md) — routes, flows, and screen contracts
- [`../docs/ui_design.md`](../docs/ui_design.md) — DOM shell visual language
- [`../docs/game_visual.md`](../docs/game_visual.md) — Phaser canvas visuals, sprite roles, and animation language

Source assets and local visual preview pages live in
[`assets/README.md`](assets/README.md). Published builds expose those previews
under `/assets/`.

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
├── routes/         Home, LocalMatch, Profile, Replay
├── components/     Reusable UI (Board wrapper around Phaser)
├── board/          Phaser scene, renderer, board constants
├── cloud/          Firebase config/bootstrap for cloud-backed v0.3 surfaces
├── game/           Local match Zustand store + shared types
├── profile/        Local profile Zustand store (persisted to localStorage)
├── replay/         Replay frame derivation from saved matches
├── core/           Wasm bridge + bot worker protocol/runner
└── ui/             Icon component + icon registry
```

Routes:

- `/` — title screen, single `Play` CTA
- `/match/local` — live match vs Practice Bot
- `/replay/:matchId` — replay viewer for a saved match
- `/profile` — local player record, preferred rule, history
- `/privacy/` and `/terms/` — static info-page-template policy pages for the public app

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
any Firebase env vars. To enable the cloud-backed `v0.3` surfaces, copy the
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
config fetch commands live in [`../docs/backend_infra.md`](../docs/backend_infra.md).
CI and tag deploy builds include these public config values so the released app
can initialize Firebase.

Fetch registered web apps and then the selected app config with:

```sh
TOKEN=$(gcloud auth print-access-token)
curl -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  "https://firebase.googleapis.com/v1beta1/projects/gomoku2d/webApps"

APP_ID="1:892554744656:web:..."
curl -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  "https://firebase.googleapis.com/v1beta1/projects/gomoku2d/webApps/${APP_ID}/config"
```

```sh
npm run build              # production build (tsc + vite build + 404.html copy)
npm run preview            # serve the production build locally
npm test                   # vitest
npm run playtest:smoke     # playwright smoke run
```

The `postbuild` step copies `dist/index.html` to `dist/404.html` so GitHub
Pages serves the SPA on deep-linked routes like `/profile` and `/replay/:matchId`
instead of a 404.

---

## Deploy

Release and local-preview steps live in [`../docs/release.md`](../docs/release.md).

Production deploys to GitHub Pages only when a `v*` tag is pushed. Normal
commits to `main` run CI but do not publish the site.

The workflow builds the Wasm package, sets `GOMOKU_BASE_PATH=/` for the custom
domain Vite build, and deploys `dist/` to Pages.

---

## Where this fits

The game is the top-level product; the Rust side in `gomoku-bot-lab/` is a
supporting workspace. The bot you play against in the browser is the same code
you can pit against itself from the command line — `gomoku-wasm` exposes it to
JS and this package calls it through a Web Worker.

```
gomoku-web                     — this package
gomoku-bot-lab/gomoku-core     — board, rules, Renju enforcement, replay format
gomoku-bot-lab/gomoku-bot      — Bot trait + implementations (RandomBot, SearchBot, …)
gomoku-bot-lab/gomoku-eval     — self-play arena, tournaments, Elo
gomoku-bot-lab/gomoku-cli      — CLI match runner with replay export
gomoku-bot-lab/gomoku-wasm     — wasm-pack bridge: WasmBoard + WasmBot for JS
```

The local-first `v0.2` product pass is complete. `P1` proved Rust + Wasm +
browser play; `P2` landed the paired desktop/mobile shell in
`v0.2.3` and the final `v0.2.4` polish/reference set on top of it. The `v0.3`
backend-continuity line now has optional Firebase config, Google sign-in, cloud
profile create/load, local-to-cloud profile promotion, embedded private
cloud-backed history, cloud replay loading, auth fallback hardening, and reset
barrier hardening. Lab-powered product features, skins, published replays, and
online play stay sequenced in later phases — see
[`../docs/roadmap.md`](../docs/roadmap.md) for sequencing and
[`../docs/architecture.md`](../docs/architecture.md) for the runtime boundary.
