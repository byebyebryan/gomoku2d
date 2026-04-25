# Gomoku2D

*An old favorite, built properly.*

A retro Gomoku game built on a modern web and Rust stack.

[![Gomoku2D hero capture — Local Match in progress on the pixel-art board](docs/assets/capture_v0_2_4_match_desktop.gif)](https://dev.byebyebryan.com/gomoku2d/)

**Play in browser:** https://dev.byebyebryan.com/gomoku2d/

**Pixel-art previews:** https://dev.byebyebryan.com/gomoku2d/assets/

Gomoku2D aims for old-school play with better engineering: a crisp board-first
UI, local replay/history, and a browser build that feels fast and polished on
desktop and mobile without losing the feel of a small tabletop game.

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

The current line is the local-first `v0.2.x` product pass. The core play loop,
desktop/mobile shell, replay flow, and guest-local profile/history are in
place; the remaining `0.2.x` work is mostly hardening and non-UI fixes rather
than another broad UX rewrite.

For the longer-term sequencing — cloud continuity, published replays, online
play, and later lab-powered features — see [`docs/roadmap.md`](docs/roadmap.md).

---

## Learn more

The canonical design and schedule live in `docs/`:

- [`docs/product.md`](docs/product.md) — what we're building and why
- [`docs/architecture.md`](docs/architecture.md) — FE stack, DOM/Phaser boundary, core-sharing story
- [`docs/app_design.md`](docs/app_design.md) — current local-first routes, flows, and screen contracts
- [`docs/ui_design.md`](docs/ui_design.md) — DOM shell visual language and styling rules
- [`docs/game_visual.md`](docs/game_visual.md) — Phaser canvas, sprite, warning, and animation language
- [`gomoku-web/assets/README.md`](gomoku-web/assets/README.md) — source asset folders and local preview pages
- [Live asset previews](https://dev.byebyebryan.com/gomoku2d/assets/) — published sprite, icon, and font previews
- [`docs/ui_screenshot_review.md`](docs/ui_screenshot_review.md) — screenshot history and UI critique
- [`docs/backend.md`](docs/backend.md) — Firebase + Firestore + Cloud Run model for later cloud/online phases
- [`docs/roadmap.md`](docs/roadmap.md) — phased plan, with local-first `v0.2` before cloud/online
- [`docs/release.md`](docs/release.md) — local preview, release checks, tagging, and publish workflow
- [`docs/bot_baseline.md`](docs/bot_baseline.md) — current `SearchBot` strategy
- [`gomoku-web/README.md`](gomoku-web/README.md) — web game stack, local dev, deploy/runtime details
- [`gomoku-bot-lab/README.md`](gomoku-bot-lab/README.md) — bot-lab build/test, CLI usage, replay format, bot notes

Superseded exploratory docs and mock briefs are preserved under
[`docs/archive/`](docs/archive/).
