# Lethal Threats

Purpose: define the position-level layer above local immediate and imminent
threats.

Local shape names suggest evidence. They do not prove lethality by themselves,
especially under Renju. A lethal threat is proven by coverage: the attacker has
legal terminal or known-lethal continuations, and the defender has no legal
single reply that covers them all.

Source of truth in code:

- tactical classifier: `gomoku-bot-lab/gomoku-bot/src/tactical/`
- scenario harness: `gomoku-bot-lab/gomoku-eval/src/lethal_scenario.rs`

## Layering

`Lethal` is not a `LocalThreatKind`.

The layers are:

1. local facts: open/closed/broken fours and threes;
2. legal attacker continuations under the active rule set;
3. legal defender coverage;
4. lethal result when coverage fails.

This is why a freestyle-looking shape can be non-lethal in Renju, and a single
White threat can become lethal if Black's only block is forbidden.

## Coverage Rules

Terminal lethal:

- attacker has one or more legal immediate winning completions;
- defender has no immediate win;
- no legal defender reply removes all attacker completions.

One-step lethal:

- after every legal defender reply, attacker has a legal continuation that
  creates terminal lethal coverage.

Typical freestyle lethal cases:

- open four;
- `4+4` when no one reply covers both fours;
- `4+3` when blocking the four still allows a legal open-four/lethal entry;
- `3+3` when every defense still leaves a legal lethal entry.

Typical non-lethal cases:

- a single blockable four;
- crossed threats with one shared covering reply;
- a defender immediate win;
- Renju Black continuations that are overline, double-four, or double-three.

## Renju Implications

Renju legality is part of coverage, not a post-filter:

- Black attacker continuations must be legal exact-five or legal threat entries.
- White attacker coverage may improve because Black replies can be forbidden.
- Forbidden Black defense squares are proof evidence and should be visible to
  analysis/report consumers.
- The only normal legal lethal fork Black can create in Renju is `4+3`; `4+4`
  and `3+3` are forbidden only when their branches are real.

Exact forbidden-move logic lives in [`renju_rules.md`](renju_rules.md).

## Replay Meaning

Lethal onset is the first frame where the losing side has no legal reply that
avoids the attacker's terminal or known-lethal continuation. Replay analysis
uses it to split the ending:

- setup corridor: how the loser was forced into the lethal state;
- lethal tail: conversion after the game was already effectively decided.

## Validation

Run the lethal scenario harness:

```sh
cd gomoku-bot-lab
cargo run --release -p gomoku-eval -- lethal-scenarios
cargo run --release -p gomoku-eval -- lethal-scenarios --show-boards
```

The doc should not embed every board dump. The CLI owns detailed scenario
rendering; this file owns the model.
