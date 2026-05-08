# v0.4.3 Corridor-Bot Lab Plan

Purpose: keep `0.4.3` in the Rust lab for one more pass so corridor search can
be tested inside bot behavior before the product exposes bot settings.

## Why This Exists

`0.4.2` proved that corridor search is more than a report feature. It gives the
project a vocabulary for advanced play: immediate threats, imminent threats,
counter-threats, corridor entry, forced loss, confirmed escape, and possible
escape.

That makes the original `0.4.3` settings/UI slice premature. A settings surface
can only be honest if the exposed choices map to stable behavior. Right now we
know the analyzer can explain finished games, but we have not yet shown how the
same model changes live bot choices.

`0.4.3` should answer that in the lab first.

## Product Bet

The target is not "make the strongest possible Gomoku bot." The target is a bot
that becomes stronger in a way the product can explain:

- it sees forcing corridors earlier,
- it spends extra effort only where the branch is tactically narrow,
- it defends by finding escapes from known corridors,
- it produces losses and wins that the analyzer can explain afterward.

If that works, later UI can expose a meaningful practice ladder or style model.
If it does not, UI should stick to the already validated depth/tactical-cap
ladder and keep corridor analysis as a replay feature.

## Expected Work

- Stabilize shared corridor-search entry points so analyzer and bot experiments
  use the same tactical vocabulary and reply classification.
- Add lab-only bot modes for corridor-aware behavior, likely behind
  `SearchBotConfig` flags or lab aliases rather than product presets.
- Try corridor move ordering: prefer candidate moves that enter or continue
  promising forcing corridors.
- Try selective corridor extension: when alpha-beta reaches an active narrow
  corridor, extend through named corridor replies instead of stopping at normal
  depth.
- Try escape-aware defense: order or prefer defender replies that leave the
  active corridor instead of only blocking the immediate square.
- Capture corridor-specific cost metrics separately from normal search nodes so
  proof work cannot hide inside "fewer nodes searched."
- Refresh bot reports and replay-analysis reports from survivor candidates, not
  every intermediate experiment.

Current implementation note:

- `gomoku-bot::corridor` now owns the replay-independent corridor proof and
  defender-reply analysis entry points.
- `gomoku-eval` keeps replay-specific traceback/report shaping, but fresh
  alternate reply probes delegate to the bot-owned corridor engine.
- `CorridorBot` exposes two lab aliases: `corridor-random` and `corridor-d1`.
  They use shallow live corridor proof plus local candidate filtering, and are
  bridge probes rather than product presets.
- `corridor-d1` now receives the same depth-1 fallback search budget plumbing as
  normal search aliases. Fallback search traces stay visible, while corridor
  proof work is reported separately under `corridor.*` trace metrics.

## Corridor Reinforcement

Some `0.4.3` work may naturally be corridor-search reinforcement rather than bot
feature work. That is acceptable when it is exposed by bot integration and is
measurable:

- cheaper proof memoization,
- narrower transition enumeration,
- better Renju legality filtering before threat classification,
- clearer distinction between confirmed escape and possible escape,
- report metrics that explain whether corridor integration helped or just moved
  cost around.

Positive, behavior-preserving optimizations should land in place. Semantic
changes should stay behind lab aliases until they survive tournament and
analysis checks.

## Evaluation Plan

Use multiple signals. Elo alone is not enough for this slice.

- Tactical scenarios stay as preflight safety checks.
- Head-to-head and gauntlet runs measure strength and cost against current
  anchors.
- Analysis reports inspect how candidate bots win and lose, especially whether
  they reduce local mistakes, tactical errors, or long strategic losses.
- Search metrics must separate alpha-beta work from corridor proof work.

Useful comparisons:

- current default baseline (`search-d3`)
- current easy lane (`search-d1`)
- current hard-side candidates (`search-d5+tactical-cap-8`,
  `search-d7+tactical-cap-8`)
- pattern-eval variants only when the question involves strength/cost tradeoff
  against known anchors

## Non-Goals

- No player-facing settings page in `0.4.3`.
- No raw corridor-search knobs in the web UI.
- No full threat-space solver.
- No claim that corridor search proves the whole game-theoretic outcome.
- No product labels such as offensive/defensive unless they map to real budget
  allocation and survive evaluation.
- No rewrite of `SearchBot` into a separate advanced bot unless the integration
  becomes structurally impossible to keep clean.

## Acceptance

`0.4.3` is successful if it leaves us with one of these clear outcomes:

- corridor-aware search survives as a measurable lab candidate for later product
  presets,
- corridor-aware search is rejected or deferred with concrete cost/correctness
  evidence,
- or corridor-search reinforcement produces enough analyzer/bot stability that
  `0.4.4` can safely expose a simpler bot ladder.

In all cases, the next UI slice should have fewer raw knobs and clearer product
language than it would have had immediately after `0.4.2`.
