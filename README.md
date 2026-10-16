# Gomoku2D

*An old favorite, built properly.*

Gomoku2D is a local-first browser Gomoku/Renju game built around one loop: play
a match, then work out why it ended that way. A retro board sits on top of a
Rust/WebAssembly rules core, configurable search bots, tactical hints, and
Replay Analysis that shows where a loss became forced.

[Play](https://gomoku2d.byebyebryan.com/) ·
[Rules](https://gomoku2d.byebyebryan.com/rules/) ·
[Guide](https://gomoku2d.byebyebryan.com/guide/) ·
[Lab](https://gomoku2d.byebyebryan.com/lab/) ·
[Visuals](https://gomoku2d.byebyebryan.com/visuals/) ·
[Docs](docs/README.md)

[![CI](https://github.com/byebyebryan/gomoku2d/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/byebyebryan/gomoku2d/actions/workflows/ci.yml)

![Gomoku2D gameplay with tactical hints, bot response, and a terminal winning frame](docs/assets/readme-gameplay.gif)

These captures show the prepared `0.5.5` build; see [media provenance](docs/assets/README.md)
and [Project status](#project-status).

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

Analysis is bounded: a confirmed escape clears the modeled forcing corridor;
a possible escape still needs verification. It is not a full-game solution.

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

## Run Locally

Install Node 24, Rust stable, and `wasm-pack`. From a fresh checkout:

```sh
git clone https://github.com/byebyebryan/gomoku2d.git
cd gomoku2d
wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
cd gomoku-web
npm ci
npm run dev
```

Open the local URL printed by Vite. Guest play needs no Firebase configuration.
The [web README](gomoku-web/README.md#local-development) covers optional cloud
setup and rebuilding Wasm; the [testing guide](docs/reference/ops/testing.md)
lists checks by area.

## Project Status

`0.5.5` packages the completed October reconciliation. See
[GitHub Releases](https://github.com/byebyebryan/gomoku2d/releases/latest) for the
currently published build and the [release preparation record](docs/archive/v0_5_5_release_preparation.md)
for this candidate. The review extended the `0.5.x` line with correctness,
persistence, recovery, accessibility, and maintenance fixes. The review is
complete, including the selected mobile Rules/Guide layout refinement; the
existing UI and copy remain the chosen direction.

These changes are recorded in [0.5.5](CHANGELOG.md#055---2026-10-16).
Pushing `main` runs CI; a release tag or explicit deployment is required to
publish the site. Real-account, physical-device, and human accessibility checks
remain explicitly deferred in the [review closeout](docs/archive/reconciliation_2026_10/README.md).
The [unattended follow-up](docs/archive/reconciliation_2026_10/v0_5_unattended_evidence.md)
adds two-client cloud-emulator acceptance and broader production Wasm evidence.
The next release and feature direction are separate decisions in the
[roadmap](docs/reference/product/roadmap.md).

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
