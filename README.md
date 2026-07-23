# Gomoku2D

*An old favorite, built properly.*

Gomoku2D is a local-first browser Gomoku/Renju game built around one loop: play
a match, then work out why it ended that way. A retro board sits on top of a
Rust/WebAssembly rules core, configurable search bots, tactical hints, and
Replay Analysis that shows where a loss became forced.

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

The aim is not the strongest possible Gomoku engine. It is a competent,
inspectable opponent whose decisions connect to the same tactical language used
by hints, replays, and the lab.

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

Gomoku2D began as an unfinished project from nearly a decade ago and returned
as a one-developer, agent-assisted production experiment. Agents made its
breadth practical across implementation, analysis, review, docs, reports, and
release work. The human still owns the decisions that define the project:
taste, scope, domain reasoning, and technical judgment.

## Inside The Repository

```text
gomoku2d/
├── gomoku-web/         browser app: React shell, Phaser board, wasm bridge
├── gomoku-bot-lab/     Rust lab: rules, bots, analyzer, eval, CLI, wasm
├── reports/lab/        curated JSON artifacts rendered by /lab/
├── docs/               reference docs, working notes, and archives
└── scripts/            release and process-story helpers
```

Start in [`gomoku-web/`](gomoku-web/) for product UI, browser state, board
rendering, and cloud continuity. Start in
[`gomoku-bot-lab/`](gomoku-bot-lab/) for rules, bots, tactical facts, replay
analysis, evaluation, and the wasm bridge. Shared game facts originate in Rust;
the web app presents them rather than maintaining a second rules model.

## Project References

- [`gomoku-web/README.md`](gomoku-web/README.md): web app architecture, local
  development, build, and deploy notes.
- [`gomoku-bot-lab/README.md`](gomoku-bot-lab/README.md): Rust workspace,
  native commands, eval harness, and wasm bridge.
- [`docs/README.md`](docs/README.md): current product, architecture, lab, and
  operations references.
- [`docs/reference/product/roadmap.md`](docs/reference/product/roadmap.md):
  current sequencing.
- [`CHANGELOG.md`](CHANGELOG.md): release history and intent.
- [`LICENSE`](LICENSE): MIT license.
