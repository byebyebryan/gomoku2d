# Bot Lab

The Bot Lab is the native Rust side of Gomoku2D. It runs the same rules and bot
logic that ship to the browser, but with stronger tooling:

- command-line games;
- bot tournaments;
- tactical scenario reports;
- replay-analysis reports;
- performance benchmarks;
- Renju legality validation.

## Product Bots

The web game exposes tested presets first:

- **Easy:** a shallow practice bot.
- **Normal:** the default learning opponent.
- **Hard:** a stronger bot that can spend several seconds on difficult moves.

Advanced settings expose a controlled version of the lab knobs, such as search
depth, width, pattern scoring, and corridor proof. The point is not to hide the
engine. Gomoku2D is strongest when the bot is explainable and tunable, not just
opaque.

## Reports

The published bot report compares tested bot configurations under one rule set,
opening policy, and budget model. The analysis report samples games from the
top matchup and shows how the replay analyzer explains decisive endings.

See the live reports:

- [Bot report](https://gomoku2d.byebyebryan.com/bot-report/)
- [Replay analysis report](https://gomoku2d.byebyebryan.com/analysis-report/)

For implementation details, see [`Search Bot`](../reference/lab/search_bot.md)
and the [`gomoku-bot-lab` README](../../gomoku-bot-lab/README.md).
