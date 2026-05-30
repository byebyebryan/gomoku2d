# SearchBot

Purpose: document the current configurable alpha-beta bot contract.

Source of truth in code:

- engine: `gomoku-bot-lab/gomoku-bot/src/search/`
- lab spec parser: `gomoku-bot-lab/gomoku-bot/src/lab_spec.rs`
- product bridge: `gomoku-bot-lab/gomoku-wasm/`

## Algorithm

`SearchBot` is negamax with alpha-beta pruning, iterative deepening, a
transposition table, tactical ordering, and a rolling threat-view backend.

Tournaments can run fixed depth, wall-clock budget, or CPU-time budget. The web
game runs the bot in the browser and does not enforce the same tournament timer;
product settings instead restrict exposed depth/width choices so custom bots do
not become unreasonable for normal play.

## Current Config Axes

| Axis | Product/lab meaning |
|---|---|
| Depth | Maximum iterative-deepening depth from `search-dN`. |
| Candidate source | Empty cells near existing stones; normally `near_all_r2`. |
| Legality gate | Exact core legality, including Renju forbidden moves. |
| Safety gate | Root filter for current immediate/imminent obligations. |
| Move ordering | Tactical ordering over legal children. |
| Width | Optional child cap after ordering, e.g. `tactical-cap-8/16`. |
| Static eval | Default line-shape eval or `pattern-eval`. |
| Threat view | Default rolling facts, with scan/shadow modes for validation. |
| Corridor proof | Optional after-search proof over selected root candidates. |
| TT cap | Optional transposition-table entry limit. |

Implementation/validation axes such as scan threat view, shadow mode,
no-safety, and retired corridor portals should not be exposed as normal product
controls.

## Lab Specs And Presets

Lab specs are explicit parser strings such as:

- `search-d1`
- `search-d3+pattern-eval`
- `search-d5+tactical-cap-16+pattern-eval`
- `search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4`

The product exposes presets and narrow advanced controls rather than arbitrary
spec strings:

| UI preset | Backing spec |
|---|---|
| Easy | `search-d1` |
| Normal | `search-d3+pattern-eval` |
| Hard | `search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4` |

Advanced config persists product fields, not raw lab strings: depth, width,
pattern scoring, and corridor proof. If those meanings change, bump the config
version instead of silently changing saved settings.

## Rolling Threat View

Rolling threat facts are the default hot path. They provide shared tactical
queries for:

- root safety;
- move ordering;
- pattern evaluation;
- hints and wasm threat views;
- corridor/candidate proof consumers where applicable.

Scan-backed modes remain as fallback/validation tools. Shadow mode is useful
when changing rolling facts, but not as a product or report default.

## Corridor Proof Status

Corridor search was tested as a portal/leaf shortcut inside normal search and
did not justify its live-search cost. The current useful search-side integration
is limited corridor proof after normal root search, primarily as an experiment
and reportable axis.

Replay analysis remains the main product use of corridor search.

## Metrics

Search traces should keep stage-level counters rather than opaque totals:

- candidate generation and width;
- legality and Renju checks;
- safety-gate filtering;
- tactical annotation/order cost;
- alpha-beta nodes and TT use;
- reached depth and budget exhaustion;
- corridor-proof candidates/nodes when enabled.

Published reports should aggregate these into readable product/lab summaries.
Full scratch reports can carry richer diagnostics under ignored `outputs/`.

## Validation

Use the smallest check that protects the change:

```sh
cd gomoku-bot-lab
cargo test --workspace
cargo run -p gomoku-eval -- tactical-scenarios
cargo run -p gomoku-eval -- lethal-scenarios
```

For strength or runtime claims, run focused head-to-heads or the curated
tournament from [`../ops/tournament.md`](../ops/tournament.md).
