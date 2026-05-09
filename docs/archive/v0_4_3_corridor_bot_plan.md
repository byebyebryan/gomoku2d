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
- Pivot the next integration from broad leaf quiescence to corridor shortcuts:
  follow narrow forcing lines from child moves, then return terminal score or
  resume normal search at corridor exit.
- Refresh bot reports and replay-analysis reports from survivor candidates, not
  every intermediate experiment.

Current implementation note:

- `gomoku-bot::corridor` now owns the replay-independent corridor proof and
  defender-reply analysis entry points.
- `gomoku-eval` keeps replay-specific traceback/report shaping, but fresh
  alternate reply probes delegate to the bot-owned corridor engine.
- The standalone `CorridorBot` bridge has been retired. The first live
  integration is `SearchBot` leaf quiescence through the lab-only
  `+corridor-q` suffix, with explicit `+corridor-qdN` sweeps for deeper proof.
- Corridor proof work is reported separately through search metrics such as
  `corridor_leaf_probes`, `corridor_search_nodes`, and
  `corridor_static_fallbacks`.
- Early gauntlet evidence showed leaf quiescence is the wrong cost shape: it
  probes too many depth-0 positions that ultimately fall back to static eval.
  The next candidate should spend corridor work only after a concrete move
  appears to enter or continue a forcing line.

## Corridor Shortcut Design

The next live-bot experiment should treat narrow corridors as portals in the
search space.

Normal alpha-beta still owns candidate generation, legality filtering, safety
gates, move ordering, child caps, iterative deepening, and time-budget handling.
After alpha-beta applies a child move, the corridor layer gets one cheap local
entry test:

- If the child move does not create an immediate or imminent threat, recurse
  normally.
- If the move creates a corridor and the defender reply set is at most `3`, run
  the corridor follower.
- If the corridor reaches a terminal win/loss, return that terminal score.
- If the corridor neutralizes or opens wider than `3` replies, return an exit
  board and resume normal alpha-beta from that board.
- If the corridor hits a safety ply guard, treat it conservatively as an exit,
  not as a proven win.

Width is the key guard. The starting cap is `3` because that covers the local
branching factor of broken and half-open three responses while excluding broad
positions that should stay in normal search. Maximum corridor ply remains a
diagnostic guard, not the primary definition of a corridor.

This also clarifies the implementation boundary. Replay analysis asks "can this
reply be proven to stay in the forced corridor?" Live search asks "can this move
shortcut through a narrow forcing line and return a useful terminal or exit
state?" Those share tactical facts and corridor transitions, but they should not
share the analyzer's need to prove every visible alternate reply.

The desired corridor result shape for live search is transition-oriented:

- `NotCorridor`
- `Terminal(score)`
- `Exit { board, plies_followed, reason }`

Only terminal results should directly override alpha-beta score. Exit results
should resume normal search from the transformed board, with metrics recording
the effective extra plies searched through the portal.

The likely long-term performance fix is a rolling threat frontier. Full-board
local-threat scans are acceptable in the analyzer and early prototypes, but live
search eventually needs move-apply/move-undo updates that can cheaply answer:

- did the last move create a corridor entry?
- what active immediate/imminent threats exist near the frontier?
- what are the defender replies for the current corridor?
- why did the corridor exit?

Do not build that cache before the shortcut API is stable. The cache should
serve the live-search queries, not the other way around.

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
- Shortcut metrics should include corridor entries seen, accepted entries,
  width-rejected entries, followed corridor plies, terminal exits, width exits,
  neutral exits, guard exits, resumed normal-search states, and effective extra
  ply gained.

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
