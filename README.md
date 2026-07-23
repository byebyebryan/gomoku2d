# Gomoku2D

*An old favorite, built properly.*

Gomoku2D is a local-first browser Gomoku/Renju game built around a simple loop:
play a match, then inspect what happened. It pairs a retro board with a
Rust/WebAssembly rules core, configurable bots, tactical hints, and Replay
Analysis that shows where a loss became forced.

**Live site:** https://gomoku2d.byebyebryan.com/

![Gomoku2D gameplay with tactical hints, bot response, and a terminal winning frame](docs/assets/readme-gameplay.gif)

## Play

- Start immediately against Easy, Normal, or Hard bots; no account is required.
- Choose Freestyle or Renju, including recursive Renju forbidden-move checks.
- Tune bot depth, width, scoring, and proof options through advanced settings.
- Enable hints for immediate threats, imminent threats, counter threats, and
  the stones behind them.
- Play on desktop or portrait mobile with dedicated touch controls.
- Keep history locally, or sign in with Google for private cloud continuity.

The target is not the strongest possible Gomoku engine. It is a competent,
inspectable opponent whose decisions connect to the same tactical language the
rest of the product uses.

## Analyze

A finished game is more than a move list. Replay Analysis starts from the ending
and walks backward through the losing side's decisions:

- **Combo onset:** the lethal threat that settled the result.
- **Setup corridor:** the forced sequence that led into that threat.
- **Last escape:** the final position where another move could still avoid the
  loss.
- **Failure step:** the missed response, prevention, or escape when the analyzer
  can classify one.

The replay can still be scrubbed normally, and any eligible frame can become a
new practice game.

![Replay Analysis walks backward through the last escape, setup corridor, and lethal onset.](docs/assets/readme-analysis.gif)

## Inspect The System

The Rust lab is not a separate benchmark toy. It produces the browser bot, the
Replay Analysis model, and the tactical vocabulary shared by the game. Its
published report exposes rankings, search telemetry, and analyzed games rather
than reducing the result to a single strength number.

![The Lab report drills into bot rankings, search telemetry, and replay analysis examples.](docs/assets/readme-lab.gif)

The Visuals guide publishes the other half of the system: pixel sprites, icons,
source sheets, design tokens, and the layering rules used on the board.

![The Visuals guide shows the pixel-art style system, icons, sprites, and source sheets.](docs/assets/readme-visuals.gif)

## How It Was Built

Gomoku2D is also a one-developer, agent-assisted production experiment. Agents
expanded what was practical across implementation, analysis, review, docs,
reports, and release work. The human still owns the parts that define the
project: taste, scope, domain reasoning, and technical judgment.

## Repository Map

```text
gomoku2d/
├── gomoku-web/         browser app: React shell, Phaser board, wasm bridge
├── gomoku-bot-lab/     Rust lab: rules, bots, analyzer, eval, CLI, wasm
├── reports/lab/        curated JSON artifacts rendered by /lab/
├── docs/               reference docs, working notes, and archives
└── scripts/            release and process-story helpers
```

The browser game lives in [`gomoku-web/`](gomoku-web/). The Rust rules, bots,
analyzer, and evaluation harness live in
[`gomoku-bot-lab/`](gomoku-bot-lab/). Player-facing explanations are published
inside the app; technical references start at
[`docs/README.md`](docs/README.md).

## Explore The Project

- [`gomoku-web/README.md`](gomoku-web/README.md): web app architecture, local
  development, build, and deploy notes.
- [`gomoku-bot-lab/README.md`](gomoku-bot-lab/README.md): Rust workspace,
  native commands, eval harness, and wasm bridge.
- [`docs/README.md`](docs/README.md): current product, architecture, lab, and
  operations references.
- [`docs/reference/product/roadmap.md`](docs/reference/product/roadmap.md):
  current sequencing.
- [`CHANGELOG.md`](CHANGELOG.md): release history and intent.
