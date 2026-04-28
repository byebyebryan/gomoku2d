# Gomoku2D

*An old favorite, built properly.*

A retro Gomoku game built on a modern web and Rust stack.

[![Gomoku2D hero capture — Local Match in progress on the pixel-art board](docs/assets/capture_v0_2_4_match_desktop.gif)](https://gomoku2d.byebyebryan.com/)

**Play in browser:** https://gomoku2d.byebyebryan.com/

**Pixel-art previews:** https://gomoku2d.byebyebryan.com/assets/

Gomoku2D aims for old-school play with better engineering: a crisp board-first
UI, local replay/history, and a browser build that feels fast and polished on
desktop and mobile without losing the feel of a small tabletop game.

It is also a process lab: a serious attempt to learn how far a veteran engineer
can push an old, sentimental project with current AI coding agents while still
holding the work to real product standards. The product has to be good enough
for the process lessons to matter.

Right now the focus is quick solo play against the Practice Bot. Matches start
fast, the board stays central, and the surrounding UI is there to support the
game instead of competing with it.

Features:

- One-click `Play` from Home — match starts vs the Practice Bot, no setup
  flow
- Freestyle and Renju rule sets; mid-game switches queue for the next round
- Live forbidden-move warnings when playing Black under Renju
- Undo the last turn during a live match
- Local replay viewer with transport controls and timeline scrubbing; branch
  off mid-replay into a fresh practice game without undoing before the branch
  point
- Local guest profile: display name, preferred rule, recent-match history —
  persisted in browser storage, no sign-in required
- Intentional desktop and portrait/mobile layouts on every main screen, with
  a dedicated touch-placement flow on mobile instead of direct tap-to-place
- Pixel art sprites with frame-by-frame animations — stones form and shatter,
  winning cells pulse, idle pointer cycles

Under the hood, React owns the shell, Phaser renders the board, and the Rust
rules and bot code ship to the browser through WebAssembly. The bot runs in a
Web Worker so it can think without freezing the UI.

Lives in [`gomoku-web/`](gomoku-web/) — see its README for stack, local
development, and deploy/runtime details.

---

## The bot lab

A Cargo workspace under [`gomoku-bot-lab/`](gomoku-bot-lab/) where the game
logic can be developed seriously without making the browser app carry all that
weight. Rules, replay format, and bot behavior live here first, can be tested
and benchmarked natively, and then ship to the web game through WebAssembly.

```
gomoku2d/
├── gomoku-web/         ← the game (React + Phaser board + TypeScript)
├── gomoku-bot-lab/     ← the Rust side
│   ├── gomoku-core/      rules + board
│   ├── gomoku-bot/       Bot trait + implementations
│   ├── gomoku-eval/      self-play arena, tournaments, Elo
│   ├── gomoku-cli/       native match runner
│   └── gomoku-wasm/      wasm-pack bridge the game imports
└── docs/
```

| Crate | What it does |
|-------|--------------|
| `gomoku-core` | Board state, rules (Freestyle + Renju), win detection, FEN, replay JSON |
| `gomoku-bot` | `Bot` trait + implementations: `RandomBot`, `SearchBot` (negamax + α-β + iterative deepening + transposition table) |
| `gomoku-cli` | Run one match: pick the bots, print the board, optionally save a replay |
| `gomoku-eval` | Run many matches: self-play arena, round-robin tournaments, Elo ratings |
| `gomoku-wasm` | `wasm-pack` bridge — exports the core + bots to the web game |

Build, CLI usage, replay format, and `SearchBot` notes live in
[`gomoku-bot-lab/README.md`](gomoku-bot-lab/README.md) and
[`docs/bot_baseline.md`](docs/bot_baseline.md).

---

## Current status

The local-first `v0.2.x` product pass is complete. The core play loop,
desktop/mobile shell, replay flow, and guest-local profile/history are in
place. The `v0.3` backend-continuity line now has Firebase, Firestore rules,
Google sign-in, cloud profile plumbing, local guest-history promotion, private
signed-in match saves, cloud history loading, and Reset Profile hardening in
place without putting sign-in in front of the local game.

For the longer-term sequencing from cloud continuity to lab-powered features,
skins, and later online play, see [`docs/roadmap.md`](docs/roadmap.md).

---

## Learn more

The canonical design and schedule live in `docs/`:

- [`docs/project.md`](docs/project.md) — the product/process thesis and project tenets
- [`docs/product.md`](docs/product.md) — what we're building and why
- [`docs/architecture.md`](docs/architecture.md) — FE stack, DOM/Phaser boundary, core-sharing story
- [`docs/app_design.md`](docs/app_design.md) — current local-first routes, flows, and screen contracts
- [`docs/ui_design.md`](docs/ui_design.md) — DOM shell visual language and styling rules
- [`docs/game_visual.md`](docs/game_visual.md) — Phaser canvas, sprite, warning, and animation language
- [`gomoku-web/assets/README.md`](gomoku-web/assets/README.md) — source asset folders and local preview pages
- [Live asset previews](https://gomoku2d.byebyebryan.com/assets/) — published sprite, icon, and font previews
- [`docs/ui_screenshot_review.md`](docs/ui_screenshot_review.md) — screenshot history and UI critique
- [`docs/backend.md`](docs/backend.md) — Firebase + Firestore + Cloud Run model for cloud, lab-powered, and online phases
- [`docs/backend_infra.md`](docs/backend_infra.md) — live Firebase/GCP setup, rules deployment, and env checklist
- [`docs/backend_cost.md`](docs/backend_cost.md) — backend free-tier assumptions, estimates, and headroom tracking
- [`docs/roadmap.md`](docs/roadmap.md) — version-based phase plan from POC to cloud, lab-powered identity, skins, and online features
- [`docs/release.md`](docs/release.md) — local preview, release checks, tagging, and publish workflow
- [`docs/bot_baseline.md`](docs/bot_baseline.md) — current `SearchBot` strategy
- [`gomoku-web/README.md`](gomoku-web/README.md) — web game stack, local dev, deploy/runtime details
- [`gomoku-bot-lab/README.md`](gomoku-bot-lab/README.md) — bot-lab build/test, CLI usage, replay format, bot notes

Superseded exploratory docs and mock briefs are preserved under
[`docs/archive/`](docs/archive/).
