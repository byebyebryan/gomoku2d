# Gomoku2D

A browser Gomoku game, with a Rust bot lab powering the AI behind it.

**Play in browser:** http://dev.byebyebryan.com/gomoku2d/

The game itself is the main thing: a small, playful pixel-art take on five-in-a-row with animations, rule variants, and a local-first practice flow against the bot. The rules engine and bot opponent are written in Rust — they live in a separate workspace (`gomoku-bot-lab/`) where bot ideas can be tried out, benchmarked against each other, and compiled to WebAssembly for the browser. Anything that works in the lab ships to the game without a rewrite.

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

---

## The web game

A local-first single-player Gomoku in the browser. React owns the shell
(home, match, replay, profile); Phaser renders the board as a stateless view
driven by the shell's state.

Features:

- One-click `Play` from Home — match starts vs the Practice Bot, no setup
  flow
- Freestyle and Renju rule sets; mid-game switches queue for the next round
- Live forbidden-move warnings when playing Black under Renju
- Undo the last turn during a live match
- Local replay viewer with transport controls and timeline scrubbing; branch
  off mid-replay into a fresh practice game from any position
- Local guest profile: display name, preferred rule, recent-match history —
  persisted in browser storage, no sign-in required
- Intentional desktop and portrait/mobile layouts on every main screen, with
  a dedicated touch-placement flow on mobile instead of direct tap-to-place
- Pixel art sprites with frame-by-frame animations — stones form and shatter,
  winning cells pulse, idle pointer cycles
- Bot runs in a Web Worker so it can think without freezing the UI

Lives in [`gomoku-web/`](gomoku-web/) — see its README for stack and local dev.

---

## The bot lab

A Cargo workspace under [`gomoku-bot-lab/`](gomoku-bot-lab/) where bot ideas can
be tried out and measured.

| Crate | What it does |
|-------|--------------|
| `gomoku-core` | Board state, rules (Freestyle + Renju), win detection, FEN, replay JSON |
| `gomoku-bot` | `Bot` trait + implementations: `RandomBot`, `SearchBot` (negamax + α-β + iterative deepening + transposition table) |
| `gomoku-cli` | Run one match: pick the bots, print the board, optionally save a replay |
| `gomoku-eval` | Run many matches: self-play arena, round-robin tournaments, Elo ratings |
| `gomoku-wasm` | `wasm-pack` bridge — exports the core + bots to the web game |

Adding a new bot means writing one `impl Bot` and dropping it into the bot
registry. From there the CLI can play it, the eval framework can rate it
against the rest of the lineup, and the Wasm bridge can ship it to the browser.

```sh
cd gomoku-bot-lab

cargo build --release --workspace
cargo test  --workspace

# One match, default settings
cargo run --release -p gomoku-cli -- --black baseline --white random

# Time-budgeted (500ms per move) instead of fixed depth
cargo run --release -p gomoku-cli -- --black baseline --white random --time-ms 500

# Save a replay
cargo run --release -p gomoku-cli -- --black baseline --white random --quiet --replay /tmp/game.json
```

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--black` | `baseline` | Bot for Black: `random` or `baseline` |
| `--white` | `random`   | Bot for White: `random` or `baseline` |
| `--depth` | `5`        | Fixed baseline depth (ignored if `--time-ms` is set) |
| `--time-ms` | —        | Time budget per move in milliseconds |
| `--rule` | `freestyle` | Rule variant: `freestyle` or `renju` |
| `--replay` | —         | Write replay JSON to this path |
| `--quiet` | —          | Suppress per-move board printing |

### Current `SearchBot`

Negamax with alpha-beta pruning, iterative deepening, and a transposition table
keyed by incremental Zobrist hashing. Move candidates are pruned to cells within
2 steps of any existing stone. Static evaluation scores open and half-open runs
of 2–4 in all four directions. It beats `RandomBot` reliably and gives a casual
human a decent game; there's plenty of headroom for stronger ideas (threat
search, 4+3 combos, NN eval, …).

---

## Replay format

Both `gomoku-cli` and `gomoku-eval` write the same JSON. Any front end that
consumes it could replay a match.

```json
{
  "hash_algo": { "algorithm": "xorshift64", "seed": 16045690984833335166 },
  "rules": { "board_size": 15, "win_length": 5, "variant": "freestyle" },
  "black": "baseline",
  "white": "random",
  "moves": [
    { "mv": "H8", "time_ms": 120, "hash": 123456789 },
    { "mv": "D4", "time_ms": 5,   "hash": 987654321, "trace": { "depth": 3 } }
  ],
  "result": "black_wins",
  "duration_ms": 3520
}
```

---

## Rules

- 15×15 board, Black moves first
- Exactly 5 in a row (any direction) wins
- **Freestyle**: no placement restrictions
- **Renju**: Black is forbidden from double-three, double-four, and overline (6+);
  winning moves (exactly 5) are always allowed; White is unrestricted

---

## Web build & deploy

Production deploys go to GitHub Pages via a manually triggered workflow:

```sh
gh workflow run deploy.yml
```

The workflow builds the Wasm package, sets `GOMOKU_BASE_PATH=/gomoku2d/` for the
Vite build, and deploys `dist/` to Pages.

---

## Where this is going

The old Phaser-only web game was the `v0.1` snapshot. The current line is the
local-first `v0.2` product pass: React owns the shell, Phaser is reduced to
the board, and the feature focus is richer local play rather than immediate
online/backend bring-up.

Phase 1 (FE rewrite, runtime boundary) is done. Phase 2 is the local-first
polish pass around the desktop/mobile `v0.2.3` baseline plus the final
`v0.2.4` shell polish on top of it. That covers:

- a consistent DOM-shell visual language
- board-first match HUD and transport-first replay
- intentional portrait/mobile layouts on the main screens
- deeper local features — guest profile, local history, replay, rules defaults

Cloud sync, published replays, and online play still matter, but they are
later phases built on top of a stronger local product.

The canonical design and schedule live in `docs/`:

- [`docs/product.md`](docs/product.md) — what we're building and why
- [`docs/architecture.md`](docs/architecture.md) — FE stack, DOM/Phaser boundary, core-sharing story
- [`docs/design.md`](docs/design.md) — current local-first routes, flows, and screen contracts
- [`docs/visual_design.md`](docs/visual_design.md) — DOM shell visual language and styling rules
- [`docs/backend.md`](docs/backend.md) — Firebase + Firestore + Cloud Run model for later cloud/online phases
- [`docs/roadmap.md`](docs/roadmap.md) — phased plan, with local-first `v0.2` before cloud/online
- [`docs/bot_baseline.md`](docs/bot_baseline.md) — current `SearchBot` strategy
- [`gomoku-web/README.md`](gomoku-web/README.md) / [`gomoku-bot-lab/README.md`](gomoku-bot-lab/README.md) — package-level details

Superseded exploratory docs and mock briefs are preserved under
[`docs/archive/`](docs/archive/).
