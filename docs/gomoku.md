# Gomoku-Specific Design Addendum

**Date:** 2026-04-14  
**Purpose:** Gomoku-specific guidance to sit on top of the generic game framework doc  
**Audience:** coding agents and human developers

## 1. Scope

This document captures the Gomoku-specific decisions, constraints, and recommendations for using Gomoku as a pilot project for the broader framework.

The purpose is **not** primarily to build the strongest possible Gomoku or Renju engine.

The purpose is to:

- validate the native core / bot / eval / web FE architecture
- provide a clean sandbox for experimenting with multiple bot styles
- support bot-vs-bot arena play
- enable replay viewing and web delivery
- optionally support Gomocup compatibility
- create a smaller, cleaner proving ground before a more complex project such as Pylander

---

## 2. Why Gomoku is a good pilot

Compared with a richer real-time sim, Gomoku gives:

- discrete deterministic rules
- trivial serialization
- easy replay format
- simple state and action contracts
- easy self-play and tournaments
- fast iteration on bot logic
- low frontend complexity
- easy browser delivery

This makes it excellent for validating:

- core boundaries
- bot boundaries
- arena/eval workflows
- replay tooling
- multiple bot types under one harness
- optional Wasm packaging

What it does **not** validate fully:

- continuous-time stepping
- physics/control loops
- high-frequency real-time rendering
- complex sensor pipelines
- the hardest throughput questions from larger simulations

---

## 3. Recommended Gomoku architecture mapping

Use the generic framework, with these Gomoku-specific mappings.

## 3.1 Gomoku core

The core should own:

- board state
- rules
- move legality
- turn progression
- win/draw detection
- board serialization
- replay state
- opening rule / variant configuration

The core should not own:

- search
- engine protocol adapters
- browser UI
- tournament orchestration
- rating/Elo computation

## 3.2 Gomoku bot layer

Bots should consume:

- board state or observation
- current player
- optional timing budget
- optional rule configuration

Bots should produce:

- move selection
- optional debug info such as score, PV, or search stats

Keep the bot contract narrow and explicit.

## 3.3 Gomoku eval layer

The eval layer should own:

- self-play
- tournaments
- Elo / win rate stats
- opening suites
- timing stats
- replay export
- regression packs
- bot-vs-bot arena orchestration

## 3.4 Gomoku frontend

The frontend should own:

- board rendering
- click input
- move list
- replay viewer
- bot-vs-bot spectator view
- arena result visualization

---

## 4. Game rules and variant handling

One of the most important early design choices is rule-set handling.

Support variants as explicit configuration, not scattered conditionals.

Possible rule/variant dimensions:

- board size
- freestyle vs standard Gomoku
- Renju-like restrictions
- swap rules
- opening rule variants
- time controls

Recommended approach:

- keep a base rules engine
- encode variant behavior in a configuration object or variant enum
- make replays include the rule configuration

This matters because “Gomoku is solved” is too coarse a statement. Practical play strength and balance vary a lot by variant and opening rule.

---

## 5. State of Gomoku bot strategy

Your older mental model is still basically right.

The strongest practical approaches are still built around:

- minimax / negamax
- PVS / negascout style search
- transposition tables
- Zobrist hashing
- localized candidate generation
- tactical threat-space logic
- pattern-heavy evaluation

The key difficulty remains the large branching factor. Strong play still relies heavily on:

- narrowing candidate moves
- focusing on tactical hotspots
- handling forcing sequences well
- using transposition efficiently

So for this pilot, classic search remains the best foundation bot style.

---

## 6. “Solved game” perspective

Gomoku is often described as solved or strategically exhausted in the unrestricted sense, especially because first-player advantage is very strong.

That does **not** make it useless as a pilot project.

What still makes it valuable:

- practical engine design
- rule/variant handling
- evaluation harnesses
- self-play infrastructure
- browser delivery
- replay tooling
- RL experimentation
- cross-bot tournaments

So treat “solved” as a note about game-theory status, not a reason to avoid the project.

---

## 7. Bot roadmap for the pilot

## 7.1 Bot v1: baseline search bot

Build this first.

Recommended features:

- deterministic move generation
- negamax / PVS / negascout
- iterative deepening
- Zobrist hashing
- transposition table
- candidate move pruning
- tactical pattern evaluation
- replay-friendly stats output

This is the debugging anchor and strength baseline.

## 7.2 Bot v2: stronger conventional bot

Possible additions:

- better move ordering
- opening handling
- stronger tactical threat resolution
- improved time management
- variant-aware search tuning

## 7.3 Bot v3: interesting experiments

After the architecture is validated, experiment with:

- RL self-play bot
- hybrid search + learned evaluation
- policy/value guidance
- LLM-assisted tooling or move priors
- intentionally weird or biased bots for harness testing

---

## 8. RL and LLM applicability

## 8.1 RL

RL is a good fit here as an **experiment track** because Gomoku gives:

- simple state representation
- fast self-play
- clear win/loss signal
- low environment complexity
- easy arena integration

Recommended stance:

- not the first bot
- very good as a second-stage experiment once the core and baseline search bot are stable

## 8.2 LLMs

LLMs are not the best default runtime brain for Gomoku move selection.

They are more useful for:

- code generation
- test generation
- replay analysis
- heuristic brainstorming
- move-prior experiments
- developer tooling

Treat LLM-based play as an optional experiment, not the main engine plan.

---

## 9. Gomocup compatibility

## 9.1 What Gomocup expects

Gomocup engines are generally native executables that communicate with a manager over stdin/stdout using a line-based protocol.

This is best treated as an **adapter concern**, not part of the core design.

## 9.2 Recommended architecture stance

Support Gomocup via a thin adapter module, for example:

- `gomoku-gomocup-adapter`

This adapter should wrap:

- one or more internal bots
- the core rules engine
- protocol parsing/formatting

Do not make Gomocup protocol assumptions part of the core interfaces.

---

## 10. Web FE and Gomocup bots

A pure browser frontend cannot directly run arbitrary native Gomocup binaries.

Recommended approach:

- web frontend for UI and inspection
- native backend runner for arbitrary external Gomocup bots
- optional Wasm builds for your own browser-safe bots

This suggests a hybrid model:

- **browser** for rendering, replays, local lightweight demos
- **native runner** for full external bot compatibility

---

## 11. Suggested Gomoku module layout

Suggested modules:

- `gomoku-core`
- `gomoku-bot`
- `gomoku-eval`
- `gomoku-cli`
- `gomoku-web`
- `gomoku-gomocup-adapter`
- optional `gomoku-wasm`

### Responsibilities

#### `gomoku-core`

- board state
- rules
- move generation
- legality checks
- variant configuration
- replay schema

#### `gomoku-bot`

- bot trait/interface
- random bot
- baseline search bot
- stronger search bot
- optional RL bot

#### `gomoku-eval`

- self-play runner
- tournaments
- opening suites
- Elo / win rate stats
- timing metrics
- replay export

#### `gomoku-cli`

- run single match
- run tournament
- inspect positions
- export replays

#### `gomoku-web`

- board renderer
- replay viewer
- click-to-play
- spectator mode
- arena result views

#### `gomoku-gomocup-adapter`

- stdin/stdout bridge
- protocol-compatible wrapper around internal bots

#### `gomoku-wasm`

- optional browser build of core and selected bots

---

## 12. Replay and logging recommendations

Replays are especially important for Gomoku because they are so cheap and useful.

Each replay should include:

- rule variant
- board size
- seed if relevant
- player identities
- moves in order
- result
- timing stats if available
- optional engine metadata

This makes replays useful for:

- debugging
- arena inspection
- regression testing
- frontend playback
- bot comparison

---

## 13. Implementation order

## Phase 1

- build deterministic core
- build random bot
- build baseline search bot
- define replay format
- build CLI single-match runner

## Phase 2

- build tournament / eval harness
- add timing and metrics
- build minimal web replay/spectator frontend

## Phase 3

- add stronger search features
- add Gomocup adapter
- optionally add Wasm build

## Phase 4

- experiment with RL
- experiment with hybrid/LLM-assisted tooling
- expand arena and analysis tools

---

## 14. Success criteria

This pilot is successful if it proves:

- one clean native core
- one explicit bot contract
- one useful arena/eval layer
- one usable web frontend
- replays/logs as first-class artifacts
- multiple bots can be pitted against each other cleanly
- optional Gomocup compatibility can be added without polluting the core

It does **not** need to prove:

- strongest possible competitive engine
- final ML stack
- full solution to every future Pylander concern

---

## 15. Final recommendation

Use Gomoku as an architecture and bot-experiment sandbox.

Build:

- a clean deterministic core
- a conventional search baseline
- a solid replay/eval harness
- a web frontend for viewing and interaction
- optional Gomocup compatibility as an adapter
- optional RL as a later experiment

This gives a strong proving ground for the broader framework while keeping the problem small enough to move quickly.
