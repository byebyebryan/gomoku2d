# About Gomoku2D

Gomoku2D is a browser Gomoku game with a retro board, modern web shell, and a
Rust rules engine running through WebAssembly.

The starting point is personal: Gomoku was an old paper-and-pencil favorite and
one of the first games I tried to build. This version keeps that thread, but it
is built like a real alpha product instead of a weekend nostalgia sketch.

The project is also a production experiment. It asks how far one developer can
push a small but serious product with an AI-centric workflow while preserving
scope control, review discipline, design taste, and maintainable code.

## What Works Today

- Play immediately without an account.
- Choose Freestyle or Renju.
- Play against configurable Easy, Normal, and Hard bots.
- Analyze finished games to see where the loss became unavoidable.
- Branch from a replay position into a new game.
- Keep local history, or sign in for private cloud-backed history.

## Why It Is Different

Gomoku2D has a lab under the board. The same Rust workspace that powers the web
game also runs native bot tournaments, replay-analysis reports, tactical
scenarios, Renju validation, and performance checks.

That lab is not just for stronger bots. It is how the project explains games:
where a forced sequence started, where the defender last had an escape, and why
a move became decisive.

Next: [play the game](https://gomoku2d.byebyebryan.com/), read
[`Rules and Renju`](rules.md), or see how [`Replay Analysis`](analysis.md)
works.
