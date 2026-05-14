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
- The standalone `CorridorBot` bridge has been retired.
- The first live `SearchBot` integration, leaf quiescence through `+corridor-q`
  and `+corridor-qdN`, has also been retired because it had the wrong cost
  shape.
- Early gauntlet evidence showed leaf quiescence is the wrong cost shape: it
  probes too many depth-0 positions that ultimately fall back to static eval.
- The next candidate became default-off portal suffixes that spend corridor
  work only after a concrete move appears to enter or continue a forcing line.
  Later cleanup made the measurements honest, but the shape still did not
  survive cost/strength checks.

## Corridor Shortcut Design

The live-bot experiment treats narrow corridors as portals in the search space.

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

Portal permission should be side-specific and root-player-relative:

- `own`: allow shortcuts through corridors created by the bot's side.
- `opponent`: allow shortcuts through corridors created by the opponent's side.

This is intentionally asymmetric. It gives the lab a concrete mechanism for
offensive and defensive styles without inventing personality weights. A more
offensive candidate can spend extra corridor depth or budget on `own` forcing
lines. A more defensive candidate can spend extra depth or budget on `opponent`
forcing lines and escape checks. A balanced candidate enables both. The first
implementation should keep the knob explicit, for example:

```text
search-d5+corridor-own-dN-w3+corridor-opponent-dN-w3
```

The exact CLI suffix can change, but the model should stay side-specific:
enabled, max corridor depth, and max corridor width for each side.

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

Depth metrics must keep nominal and effective depth separate. A candidate named
`search-d3+...` still has a depth-`3` alpha-beta budget. Corridor following
should add explicit reach metrics instead of mutating that label:

- nominal alpha-beta depth,
- ordinary depth completed under budget,
- corridor extra plies followed,
- effective depth for the selected/principal branch,
- average and max effective depth across searched root moves.

The likely long-term performance fix is a rolling threat frontier. Full-board
local-threat scans are acceptable in the analyzer and early prototypes, but live
search eventually needs move-apply/move-undo updates that can cheaply answer:

- did the last move create a corridor entry?
- what active immediate/imminent threats exist near the frontier?
- what are the defender replies for the current corridor?
- why did the corridor exit?

Do not build that cache before the shortcut API is stable. The cache should
serve the live-search queries, not the other way around.

### Portal Spike Checkpoint

The first opt-in selective-extension suffixes landed default-off:

```text
search-dN+corridor-own-dX-wY
search-dN+corridor-opponent-dX-wY
```

They validate the plumbing and metrics, but not the current search behavior.
Focused tests found:

- `search-d3+corridor-own-d4-w3` lost decisively to base `search-d3` even when
  the per-move budget was raised from `1s` to `5s` and `10s`.
- `search-d3+corridor-own-d1-w3` was less bad but still spent most moves on the
  budget boundary.
- `search-d5+tactical-cap-8+corridor-own-d2-w3` won a tiny sample against base
  `search-d5+tactical-cap-8`, but it hit budget on every move, so it is not a
  promoted config.

The latest root-cause read was:

- Entry detection was not move-local enough. It could accept a move because the
  resulting board had an active threat that already existed.
- Accepted portal entries often hit depth or neutral exits and then resume
  normal alpha-beta many times.
- Ordinary alpha-beta node count understates the cost. The trace shows repeated
  threat scans, Renju legality checks, tactical annotations, corridor exits, and
  resumed searches.

Cleanup landed in this checkpoint:

- Require portal entry to be caused by the candidate move itself.
- Disable nested portal re-entry after a resume.
- Surface entry checks, accepted entries, acceptance rate, resume count, and
  exit reasons in the primary report.

Post-cleanup smoke checks:

- `search-d3+corridor-own-d1-w3` went `7-9` against `search-d3` over `16`
  games. It raised effective depth a little, but average search time was about
  `285 ms` versus `69 ms`, with `15.6%` budget exhaustion and more than `549k`
  resumed searches.
- `search-d3+corridor-own-d4-w3` went `6-10` against `search-d3`, with
  `86.4%` budget exhaustion and nominal depth collapse.
- `search-d5+tactical-cap-8+corridor-own-d2-w3` went `6-10` against
  `search-d5+tactical-cap-8`, with `80.1%` budget exhaustion.

Verdict: the cleanup fixed semantics and observability, but not viability. Keep
the suffixes default-off as lab plumbing, skip a larger promotion sweep for the
current scan-backed implementation, and move the next serious performance
question to the `0.4.4` rolling-frontier pass.

Remaining cleanup before deeper frontier work:

- Reuse tactical annotation data from move ordering where possible if it falls
  out naturally from a small refactor.
- Do not run another broad portal sweep until entry/reply facts are cheaper.

### Rolling Frontier Decision Frame

Rolling frontier is still likely the next structural step, but it should be
treated as a correctness-sensitive cache, not as a tactical feature by itself.

Target abstraction: a scan-backed `ThreatView` first, then a rolling
implementation later. The minimal scan-backed contract is now part of the
`0.4.3` cleanup for the current search-facing queries:

- whether a specific move creates or materializes a local corridor threat,
- the active immediate/imminent threats by side,
- legal defender replies for the current corridor threat,
- attacker move rank for tactical ordering.

The richer rolling-frontier contract should be added later only where a
consumer needs it. Likely future queries include raw versus Renju-legal threat
squares, move deltas, and corridor exit reasons.

The rolling version should update only facts whose local lines changed after an
apply/undo. It must preserve the raw/legal/forbidden split because Renju can
turn a natural-looking black reply into proof evidence for the opponent.

Main risks:

- stale incremental facts silently corrupt bot choices,
- undo stack bugs break search reproducibility,
- Renju legality can be cached at the wrong semantic layer,
- a report-shaped cache can become too broad for live search,
- frontier speed can mask incorrect portal semantics if the entry model is not
  fixed first.

De-risking sequence:

1. Keep current scan logic as the reference implementation.
2. Define the exact `ThreatView` query contract and route existing wrappers to
   that contract.
3. Add differential tests over random apply/undo sequences, tactical fixtures,
   and Renju forbidden fixtures.
4. Run a rolling frontier in shadow mode and assert it matches scan-backed
   answers before it affects bot behavior.
5. Switch only the hot portal entry/reply path once the shadow mode is stable.

### Release Boundary

Rolling frontier is large enough to stand on its own. It should not be bundled
into `0.4.3` unless the implementation is limited to the scan-backed seam needed
to de-risk the next release.

Recommended split:

- `0.4.3`: finish the current corridor portal pass. Tighten move-local portal
  semantics, make the shortcut metrics honest, introduce the scan-backed
  `ThreatView` seam, and defer the current portal model as lab-only evidence.
- `0.4.4`: run the rolling frontier lab pass. Build the incremental threat
  cache behind the same query contract, validate it against the scan-backed
  reference in fixtures and shadow runs, then switch only the hot portal
  entry/reply path if it proves equivalent and faster.
- `0.4.5` or later: expose player-facing settings/bot controls once the lab has
  stable product language. Avoid turning settings into a raw debug console for
  unresolved search experiments.

This keeps `0.4.3` focused on whether corridor portals are the right behavior,
and keeps `0.4.4` focused on whether the necessary threat-detection machinery
can be made correct and cheap.

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
- Reports should show both nominal depth and effective depth. The expected
  improvement is not that a `d3` bot becomes labeled as a deeper bot; it is that
  its measured effective depth rises inside narrow corridors without losing the
  cost profile of the nominal depth.
- Asymmetric portal experiments should report `own` and `opponent` shortcut
  counts separately so offensive and defensive candidates can be compared by
  actual budget allocation, not just label.

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
  `0.4.4` can focus on rolling-frontier performance instead of basic corridor
  correctness.

In all cases, the eventual UI slice should have fewer raw knobs and clearer
product language than it would have had immediately after `0.4.2`.

## Final Outcome

The accepted outcome is rejection/deferment of corridor portals as a live bot
feature under the current bounded search budget. The portal work remains useful
as lab evidence and as pressure that produced shared tactical facts,
move-local corridor semantics, the `ThreatView` seam, and honest cost metrics.
It should not be promoted into anchors, product presets, or UI settings.

The least misleading diagnostic shape we tried was proof-only portal search:
use only terminal corridor proofs and fall back to the original child search for
non-terminal exits. Even that shape stayed slower and did not outperform the
base anchors in focused head-to-head runs. Treat proof-only, rank/top-N, and
resume/static portal modes as historical controls only.
