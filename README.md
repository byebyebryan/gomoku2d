# Gomoku2D

*An old favorite, built properly.*

Gomoku2D is a simple, fun web Gomoku with a retro feel and serious engineering
underneath: a modern frontend, a Rust/WebAssembly rules core, and a Rust bot lab
for game logic and AI experiments.

It is also a production experiment: one developer, an AI-centric workflow, and
a question more interesting than raw speed: how much of a real product team's
surface area can agents help cover without lowering the quality bar?

[![Gomoku2D hero capture — Local Match in progress on the pixel-art board](docs/assets/capture_v0_2_4_match_desktop.gif)](https://gomoku2d.byebyebryan.com/)

**Play in browser:** https://gomoku2d.byebyebryan.com/

**Pixel-art previews:** https://gomoku2d.byebyebryan.com/assets/

**Bot lab report:** https://gomoku2d.byebyebryan.com/bot-report/

**Replay analysis report:** https://gomoku2d.byebyebryan.com/analysis-report/

The answer so far is not "type less code and ship anything." It is closer to
running a tiny product team through agents: implementation, review, test
coverage, infrastructure, asset iteration, release notes, and design critique
all stay in the loop.

## What makes it different

- **Personal, but not casual.** Gomoku was a paper-and-pencil childhood
  favorite and one of my first game-dev targets. This version keeps that
  sentimental thread, but treats it like a real alpha product instead of a
  nostalgic weekend sketch.
- **Small surface, serious foundation.** React owns the app shell, Phaser
  renders the board, and Rust rules/bot logic ship to the browser through
  WebAssembly. The split keeps the UI light without trapping core game logic in
  the frontend.
- **A lab under the board.** The Rust workspace is where rules, bots,
  benchmarks, replay formats, and future analysis/puzzle features can be built
  natively before they reach the browser.
- **AI as production leverage.** The experiment is not whether agents can
  generate code quickly. It is whether one person can use agents to cover more
  of the product loop while still preserving taste, scope control, and review
  discipline.
- **Retro assets with a real workflow.** Sprites, icons, and fonts have source
  assets, manifests, and live preview pages, so the pixel-art style can be
  iterated deliberately instead of treated as decoration.

## What works today

- Start a practice match immediately, no account required.
- Play Freestyle or Renju against configurable Easy / Normal / Hard bots, with
  Renju forbidden-move feedback and mobile-friendly placement controls.
- Review local replays, scrub the timeline, and branch from a replay position
  into a fresh practice game.
- Let the browser analyzer mark the setup corridor and latest escape in
  finished decisive replays.
- Keep guest-local history by default, or sign in with Google for private
  cloud-backed history across browsers.
- Use the same board-first app on desktop and portrait mobile.

Lives in [`gomoku-web/`](gomoku-web/) — see its README for stack, local
development, and deploy/runtime details.

---

## The Bot Lab

[`gomoku-bot-lab/`](gomoku-bot-lab/) is the other half of the project: a Rust
workspace where the game can grow beyond "browser board plus bot." Rules,
replay format, and bot behavior live here first, can be tested and benchmarked
natively, and then ship to the web game through WebAssembly.

```
gomoku2d/
├── gomoku-web/         ← the game (React + Phaser board + TypeScript)
├── gomoku-bot-lab/     ← the Rust side
│   ├── gomoku-core/      rules + board
│   ├── gomoku-bot/       Bot trait + implementations
│   ├── gomoku-analysis/  setup-corridor replay analysis
│   ├── gomoku-eval/      self-play arena, tournaments, Elo
│   ├── gomoku-cli/       native match runner
│   └── gomoku-wasm/      wasm-pack bridge the game imports
└── docs/
```

| Crate | What it does |
|-------|--------------|
| `gomoku-core` | Board state, rules (Freestyle + Renju), win detection, FEN, replay JSON |
| `gomoku-bot` | `Bot` trait + implementations: `RandomBot`, `SearchBot` (negamax + α-β + iterative deepening + transposition table) |
| `gomoku-analysis` | Shared setup-corridor replay analyzer used by reports and the browser |
| `gomoku-cli` | Run one match: pick the bots, print the board, optionally save a replay |
| `gomoku-eval` | Run many matches: self-play arena, round-robin tournaments, Elo ratings |
| `gomoku-wasm` | `wasm-pack` bridge — exports the core + bots to the web game |

Build, CLI usage, replay format, and `SearchBot` notes live in
[`gomoku-bot-lab/README.md`](gomoku-bot-lab/README.md). The broader docs index
lives in [`docs/README.md`](docs/README.md).

---

## Current Status

The local-first `v0.2.x` product pass made the game feel complete without
cloud: board-first play, desktop/mobile layout, replay, profile, and local
history. The `v0.3` backend-continuity line added optional Google sign-in,
Firebase/Firestore plumbing, local-to-cloud profile promotion, private
cloud-backed history, schema/rules hardening, and Reset Profile without putting
sign-in in front of the game.

The `v0.4` lab-powered line is complete. Bot-lab reports, the static replay
analysis report, configurable bot settings, tactical hints, and the first
in-product replay-analysis surface are now live. The `v0.5` line is now about
public-release reconciliation: clean up the repo after the lab push, make the
reports feel first-class, explain the game and analyzer in-product, and package
the project for a stranger-facing alpha. For the longer-term sequencing, see
[`docs/reference/product/roadmap.md`](docs/reference/product/roadmap.md).

---

## Learn More

Start with the public docs:

- [`About`](docs/public/about.md) — project story and current product shape
- [`Rules And Renju`](docs/public/rules.md) — Freestyle, Renju, and forbidden moves
- [`Replay Analysis`](docs/public/analysis.md) — how to read setup corridors, lethal onset, and last escape
- [`Bot Lab`](docs/public/bot-lab.md) — presets, advanced controls, and published reports

Reference docs, working notes, runbooks, and archives are organized from
[`docs/README.md`](docs/README.md). The most useful technical entry points are
[`Architecture`](docs/reference/app/architecture.md),
[`Search Bot`](docs/reference/lab/search_bot.md),
[`Corridor Search`](docs/reference/lab/corridor_search.md),
[`Game Analysis`](docs/reference/lab/game_analysis.md), and
[`Release`](docs/reference/ops/release.md).

Live companion pages:

- [Asset previews](https://gomoku2d.byebyebryan.com/assets/)
- [Bot lab report](https://gomoku2d.byebyebryan.com/bot-report/)
- [Replay analysis report](https://gomoku2d.byebyebryan.com/analysis-report/)
