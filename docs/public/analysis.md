# Replay Analysis

Replay analysis answers a focused question:

> Where did this finished game become unavoidable?

It is not a full perfect-play solver. It follows the narrow tactical corridor
near the end of a decisive game, then marks the important moments.

## What The Labels Mean

- **Last escape:** the latest point where the losing side had a modeled way out.
- **Setup corridor:** the pressure sequence that led from that missed chance to
  the already-lost state.
- **Lethal sequence:** the final conversion after the defender no longer had
  coverage.
- **Mistake point:** the move that missed a required response or the last modeled
  escape.

## How To Read It

In a replay, analysis walks backward from the winning move. It highlights
threats, legal replies, forbidden Renju replies, and escape candidates. The
timeline separates normal play, the setup corridor, lethal onset, and the final
win.

The goal is educational: help a player see why the end collapsed and where a
different response mattered.

The full model and implementation contract live in
[`Game Analysis`](../reference/lab/game_analysis.md) and
[`Corridor Search`](../reference/lab/corridor_search.md).
