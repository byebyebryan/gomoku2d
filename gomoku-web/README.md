# gomoku-web

A small Gomoku game for the browser, with pixel art and a few playful touches.

**Play:** https://byebyebryan.github.io/gomoku2d/

Built with Phaser 3 + TypeScript + Vite. The rules engine and bot opponent are
the same Rust code used by the native bot lab in this repo, compiled to Wasm and
called from JS. The bot runs in a Web Worker so it can think without freezing
the UI.

---

## What you can do

- Play human vs human, human vs bot, or bot vs bot — any combination per side
- Switch between Freestyle and Renju rules from the settings panel
- Edit player names inline; win counts persist across rounds; the two profiles
  alternate color slots after each completed round
- Get live forbidden-move warnings when playing Black under Renju
- Watch the move-by-move sequence on the result screen between rounds

---

## A bit of polish

It's a simple game, but the presentation gets a little extra love:

- Pixel art sprites with frame-by-frame animations — stones form and shatter,
  the hover pointer cycles through idle states, winning cells pulse green
- Round-end transition: the two player cards swap positions while their
  background and text colors lerp through each other
- Idle "relax" animations on the most recently placed stone
- Per-player move timer with a live delta display, plus a total game timer
- Responsive canvas: 1200×900 (landscape) or 900×1350 (portrait), scaled to fit
  any viewport

---

## Stack

| Layer | Tech |
|-------|------|
| Renderer / game loop | Phaser 3.87 |
| Language | TypeScript 5 |
| Build | Vite 6 |
| Game logic + bot | Rust (`gomoku-core`, `gomoku-bot`) → `wasm-pack --target bundler` |
| Bot execution | Web Worker (off-thread) |

---

## Local development

Prerequisites: Node, Rust, `wasm-pack`.

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

```sh
npm run build     # production build
npm run preview   # serve the production build locally
```

---

## Deploy

Production deploys to GitHub Pages via a manually triggered workflow at the repo
root (`.github/workflows/deploy.yml`):

```sh
gh workflow run deploy.yml
```

---

## Where this fits

The game is the top-level product; the Rust side lives in `gomoku-bot-lab/` as
a supporting workspace. The bot you play against in the browser is the same
code you can pit against itself from the command line — `gomoku-wasm` exposes
it to JS and this package calls it through a Web Worker.

What's here now is the `v0.1` snapshot: offline single-player, Phaser-driven
end to end. The next phase rewrites the shell in React and reduces Phaser to
a board-only renderer — see [`../docs/architecture.md`](../docs/architecture.md)
for the target and [`../docs/roadmap.md`](../docs/roadmap.md) for sequencing.

```
gomoku-web                     — this package
gomoku-bot-lab/gomoku-core     — board, rules, Renju enforcement, replay format
gomoku-bot-lab/gomoku-bot      — Bot trait + implementations (RandomBot, SearchBot, …)
gomoku-bot-lab/gomoku-eval     — self-play arena, tournaments, Elo
gomoku-bot-lab/gomoku-cli      — CLI match runner with replay export
gomoku-bot-lab/gomoku-wasm     — wasm-pack bridge: WasmBoard + WasmBot for JS
```
