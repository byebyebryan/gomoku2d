# Replay Analysis

Replay analysis answers a focused question:

> Where did this finished game become unavoidable?

It is not a full perfect-play solver. It follows the narrow tactical corridor
near the end of a decisive game, then marks the important moments.

## Terms

- **Lethal sequence:** the final forced line once the defender no longer has
  coverage.
- **Setup corridor:** the forcing sequence that led into the lethal state.
- **Last escape:** the latest point where the losing side had a modeled way out.
- **Mistake point:** the move that missed a required response or missed the last
  modeled escape.

## How To Read It

In a replay, analysis walks backward from the winning move. It highlights
threats, legal replies, forbidden Renju replies, and escape candidates. The
timeline separates normal play, the setup corridor, lethal onset, and the final
win.

The goal is educational: help a player see why the end collapsed and where a
different response mattered.

For the full model and implementation contract, see
[`Game Analysis`](../reference/lab/game_analysis.md) and
[`Corridor Search`](../reference/lab/corridor_search.md).
