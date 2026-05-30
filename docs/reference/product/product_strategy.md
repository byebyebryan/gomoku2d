# Product Strategy

Gomoku2D is both a product and a production experiment.

The product goal is a polished, opinionated browser Gomoku that is worth
showing. The production goal is to learn how far one developer can push an old,
sentimental project with an AI-centric workflow while still holding the work to
real product standards.

The tension matters. If the game is only a toy demo, the process lessons are
weak. If the process becomes the product, the game becomes incoherent. The
roadmap should therefore read like a real alpha/beta product path while keeping
the repo, docs, review loop, and release process healthy enough for repeated
agent-assisted work.

## Product Shape

Gomoku2D is a local-first web game with a retro board, a modern app shell, a
Rust/WebAssembly rules core, and a native Rust bot lab behind it.

The target product is not just a board with a bot. Finished games should remain
useful: replay them, branch from them, analyze where they turned, turn mistakes
into future puzzles, and practice against bots shaped by the same lab that
powers the game.

## Pillars

### Lab-powered identity

The bot lab is a product input, not scaffolding. The strongest features are the
ones that exist because the Rust core can run positions outside the live board:

- replay analysis that explains setup corridors, lethal onset, and last escape;
- bot reports that make search behavior visible instead of only comparing Elo;
- future puzzles and "save this game" challenges generated from real endings;
- configurable bots whose behavior maps to understandable search settings.

This is the clearest differentiator. The game should not compete by promising
the strongest possible Gomoku engine; it should compete by making its tactical
understanding visible and useful.

### Local-first on-ramp, cloud-backed depth

The first session should be disposable and frictionless. Cloud only matters when
the player asks for durable or shared state.

- First meaningful interaction creates a local profile on the device.
- Guests can play bot matches and keep local history without sign-in.
- Google sign-in promotes settings and private history into a cloud profile.
- Signed-in private history is cloud-backed continuity, not public evidence.
- Public/shared replays require an explicit publish step.
- Future ranked/trusted matches must be server-validated move by move.

Keep these trust lanes separate: local/free play, signed-in private continuity,
explicitly shared replay state, and future trusted online state.

### Simple surface, serious stack

React owns the shell, routes, settings, profile, replay, reports, and static
learning pages. Phaser owns the board. Rust owns rules, bots, analysis, and
native evaluation. The split keeps the product approachable without burying core
logic in the frontend.

The visual style should stay personal and inspectable: pixel-art board, clean
DOM surfaces, and a visible visual guide rather than decoration hidden in code.

### AI-assisted production discipline

The project is also a way to learn where agents help and where they fail. The
answer should come from product quality, not output volume.

Durable practices:

- use agents for exploration, implementation, review, reports, and docs;
- keep human control over taste, product judgment, and scope;
- treat docs, tests, screenshots, reports, and release notes as part of the
  product, not cleanup after the fact;
- prefer coherent releases over endless internal experiments.

## Non-goals

- Native mobile apps.
- Social feeds, chat timelines, or broad community mechanics.
- Ranked/esports depth before online play has a reason to exist.
- Monetization.
- Enterprise-grade anti-cheat.
- Feature expansion that hides the lab-powered identity instead of clarifying it.

## Success

- A stranger can open the URL, play immediately, understand the rules, finish a
  game, inspect the replay, and see that the analyzer is more than a passive
  move list.
- The repo clearly explains the product, the lab, and how they connect.
- At least one player-facing feature exists specifically because the bot lab
  exists. Replay analysis and lab reports already satisfy this; puzzles and
  save-this-game challenges are natural extensions.
