# v0.4.6 Replay Analysis Plan

Purpose: turn the corridor-search analyzer from a lab report artifact into a
first player-facing replay feature.

This is an ad-hoc working plan. Canonical concepts remain in
`docs/game_analysis.md`, `docs/corridor_search.md`, and `docs/game_visual.md`.

## Context

The `0.4.2` through `0.4.4` lab work made corridor search the project's most
important strategic model. It did not become a strong live-bot shortcut under
the current compute budget, but it did become useful for explaining finished
games:

- it identifies the final forced corridor near the ending;
- it finds the losing side's last known escape;
- it separates confirmed escapes from possible escapes;
- it exposes Renju forbidden replies as proof evidence;
- it gives the product a vocabulary for tactics beyond "the bot searched
  deeper."

`0.4.5` exposed bot controls and live tactical hints. `0.4.6` should now expose
the analyzer on the Replay page, not as another separate report site.

## Product Goal

When a player opens a decisive replay, the app should progressively answer:

- where the final forced corridor starts;
- which move was the point of no return;
- where the losing side last had an escape;
- what the key defensive or counter-threat options were;
- whether the conclusion is confirmed or only possible within the bounded
  model.

The feature should make replay more useful without turning the replay page into
a lab dashboard. The default player flow is:

1. Open a saved replay.
2. Analysis starts in the background for decisive games.
3. The player can keep scrubbing the replay while annotations arrive.
4. The UI offers a compact result summary and next/previous "analysis moment"
   navigation.
5. The board shows only the annotations for the currently selected replay
   frame.

## Scope

In scope for `0.4.6`:

- run replay analysis in the browser through Wasm;
- keep the work off the main UI thread with a web worker;
- make analysis progressive from the ending backward;
- annotate replay frames with corridor-analysis markers;
- add compact analysis status/result copy to the Replay page;
- reuse the existing analysis vocabulary: forced corridor, last escape,
  confirmed escape, possible escape, forbidden reply, immediate threat, imminent
  threat, counter-threat;
- keep live board overlays on the same split sprite vocabulary replay analysis
  will use.

Out of scope:

- public/shareable analysis links;
- cloud persistence of analysis results;
- full proof-tree browsing in product UI;
- live-game analysis;
- universal best-move recommendation;
- puzzle generation;
- another published static analysis-report layout pass.

## Architecture

The current replay analyzer lives in `gomoku-eval`, which is the wrong boundary
for browser use. `gomoku-eval` should remain the CLI/report shell. The reusable
analysis core should move into a small shared Rust layer that can be consumed by
both `gomoku-eval` and `gomoku-wasm`.

Recommended shape:

- `gomoku-core`: unchanged authority for board, rules, move codec, game result,
  and legality.
- `gomoku-bot`: tactical facts, rolling/scan `ThreatView`, corridor proof
  primitives, and search-bot logic.
- `gomoku-analysis`: new or equivalent shared module for replay-analysis
  state, bounded backward traceback, proof summaries, and product-safe result
  records. It depends on `gomoku-core` and `gomoku-bot`, but not on filesystem,
  HTML report rendering, tournament reports, or CLI output.
- `gomoku-eval`: consumes `gomoku-analysis` for fixture tests, batch analysis,
  and static report generation.
- `gomoku-wasm`: exposes a thin progressive analyzer API around
  `gomoku-analysis`.
- `gomoku-web`: owns worker orchestration and presentation only.

This keeps the same rule as the rest of the project: React and Phaser render
facts; they do not rediscover Gomoku/Renju strategy.

Implementation checkpoint:

- `gomoku-analysis` has been split out as the shared Rust analyzer crate.
- `gomoku-analysis` exposes `ReplayAnalysisSession`, a stepped backward
  traceback API that returns per-frame annotations and cumulative counters.
- `gomoku-eval` re-exports and consumes `gomoku-analysis` for the existing CLI
  and static report flows.
- `gomoku-wasm` exposes `WasmReplayAnalyzer.createFromReplayJson(...).step(...)`
  as a session-backed API, plus `WasmBoard.hashString()`.
- `gomoku-web` can convert a `SavedMatchV2` into exact core replay JSON and
  create the wasm analyzer from that saved match.
- `gomoku-web` has a cancellable replay-analysis worker/runner protocol for
  progress, completion, cancellation, and worker failure.
- Replay-route state, analysis copy, and board annotations are still the
  remaining product slice.

## Progressive Analyzer API

A single blocking `analyzeReplay()` call is the wrong browser API. The replay
page needs incremental updates so it can remain responsive and show progress.

Target shape:

```text
WasmReplayAnalyzer.createFromReplayJson(replay_json, options_json)
WasmReplayAnalyzer.step(max_work_units) -> AnalyzerStepResult
WasmReplayAnalyzer.dispose()
```

The bridge now advances a real Rust session. Each `step(max_work_units)` analyzes
one or more replay prefixes from the ending backward, returns frame annotations
for those prefixes, and withholds final analysis until the session resolves,
becomes unclear/unsupported, or reaches its scan bound.

The step result should include:

- status: `running`, `resolved`, `unclear`, or `unsupported`;
- current prefix/ply being analyzed;
- updated frame annotations keyed by replay ply;
- latest summary if known;
- counters for searched prefixes, branch roots, and proof nodes;
- model metadata: rule set, probe depth, traceback limit, analyzer version.

The worker schedules chunks with small time slices. Route changes and match
changes cancel the analyzer. Worker or Wasm failure must degrade to "Analysis
unavailable" while replay playback continues normally.

## Replay UI

The Replay page should absorb analysis in place.

Board panel:

- keep the board as the primary surface;
- render annotations for the currently selected frame only;
- do not show every proof branch at once;
- keep final winning-line treatment unchanged.

Replay deck:

- add a compact analysis section below result/match metadata or near playback;
- show status while running: `Analyzing from the final move...`;
- show result when available: `Forced corridor: move 43-51`,
  `Last escape: move 42`, `Possible escape found`;
- show model copy only as secondary detail: `Corridor search, depth 4,
  traceback 64`.

Timeline/playback:

- add small markers for analyzed moments, the forced interval, and last escape;
- add next/previous analysis moment controls;
- keep normal scrubbing usable while analysis is running.

Mobile:

- keep board-first layout;
- avoid a large proof panel above the board;
- collapse analysis copy into a short row/card below playback controls;
- keep next/previous analysis moment buttons reachable without forcing a long
  scroll.

## Annotation Language

The replay UI should use the analysis-report vocabulary, but with fewer visible
markers.

Cell context outlines:

- immediate win: green;
- immediate threat: red;
- imminent threat reply: pink;
- counter-threat: purple;
- corridor entry / deny this corridor: white or gold.

Proof markers:

- `L`: forced loss;
- `F`: forbidden Renju reply;
- `E`: confirmed escape;
- `P`: possible escape;
- `?`: unknown;
- `!`: immediate loss if ignored;

Avoid showing forced-loss letters as a default product marker. `L` exists for
detail/report surfaces, but a move on the actual losing line already led to the
shown result; the useful information is where the player had alternatives.

## Sprite Intake

New source sprites were added under `gomoku-web/assets/sprites/`:

| File | Size | Layout | Intended role |
|---|---:|---|---|
| `caution.png` | 96x48 | 6 cols x 3 rows | Live tactical caution and forbidden-style loops |
| `highlighter.png` | 96x48 | 6 cols x 3 rows | Board-cell highlight outlines for replay/hint contexts |
| `marker.png` | 96x96 | 6 cols x 6 rows | Warning and proof/result markers |

These replace the overloaded legacy board-overlay sheet. The old sheet mixed
marker-shaped frames, caution/forbidden frames, and a reserved highlighter row;
the split is cleaner:

- `caution` should own forbidden and forbidden-warning caution loops;
- `highlighter` should own board-cell context highlights;
- `marker` should own warning and symbolic proof/result markers.

Caution animation names should distinguish the combined and standalone
forbidden surfaces:

- `caution-forbidden-warning`
- `caution-forbidden-out`
- `caution-forbidden-in`

Marker animation names should follow the proof letters:

- `marker-warning`
- `marker-question`
- `marker-L`
- `marker-F`
- `marker-E`
- `marker-P`

Color contract:

- `highlight-strong`: immediate threat/loss/win; red for danger, green for win.
- `highlight-soft`: imminent or counter-threat; pink for imminent, purple for
  counter-threat.
- `highlight-entry`: corridor entry or critical-point context; per-side or
  neutral, with white as the preview default.
- `marker-warning`: red for immediate loss/threat, green for immediate win.
- `marker-question`: gray.
- `marker-L`: red.
- `marker-F`: red.
- `marker-E`: green.
- `marker-P`: teal.

Implementation rules:

- keep source and `public/assets/sprites/` copies in sync;
- update the sprite README and preview when frame layout changes;
- add `SPRITE.CAUTION`, `SPRITE.HIGHLIGHTER`, and `SPRITE.MARKER`;
- keep live hints on `SPRITE.MARKER` / `SPRITE.CAUTION` without changing hint
  semantics;
- use the new highlighter/marker sheets for replay analysis;
- keep the legacy overlay sheet removed once no runtime or preview references
  remain.

## Data And Caching

Do not add cloud/profile persistence in the first slice. Analysis is derived
from the saved replay and can be recomputed.

Short-lived client caching is acceptable:

- key by match id, match move hash, analyzer version, rule set, and model
  options;
- store in memory first;
- session storage can be considered only if repeat analysis latency is painful.

Do not store analysis into local/cloud profile schema in `0.4.6`.

## Testing And Validation

Rust:

- move existing analysis fixtures to the shared analyzer boundary;
- keep current report/fixture parity in `gomoku-eval`;
- add Wasm-safe tests for result serialization shape where practical.

Web:

- worker protocol tests for progress, cancellation, failure, and unsupported
  replay states;
- Replay route tests for running/resolved/error states;
- Board/scene tests for new annotation sprites and z-order;
- no-regression tests for existing live hint priority.

Manual:

- analyze a decisive local replay;
- scrub while analysis is running;
- verify mobile replay layout remains board-first;
- compare at least one known tournament replay against the static analysis
  report for matching summary fields.

## Release Story

`0.4.6` should be framed as the first player-facing version of corridor-search
analysis:

- `0.4.2` proved the analyzer in reports;
- `0.4.5` exposed bot configuration and tactical hints;
- `0.4.6` makes saved replays explain themselves.

Keep the claim bounded. This is not a perfect solver. It is a model-backed
explanation of the final forced corridor and the last visible escape.
