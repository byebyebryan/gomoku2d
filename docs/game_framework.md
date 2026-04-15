# Generic Game Framework Design Doc

**Date:** 2026-04-14  
**Purpose:** reusable architecture for native core + bot runtime + eval framework + multiple frontends  
**Audience:** coding agents and human developers

## 1. Executive summary

This document defines a reusable architecture for game and simulation projects that combine:

- a deterministic native core
- one or more bot or agent implementations
- benchmarking / evaluation / telemetry workflows
- one or more frontends for human interaction or visualization

The core design principle is:

> The **core** is the source of truth.  
> Bots, evaluation tooling, and frontends are layered around it.

This framework is intended for projects such as:

- board games
- arcade games
- physics sandboxes
- strategy games
- AI control problems
- replay / telemetry-heavy testbeds

---

## 2. When to use this architecture

Use this architecture when the project has some or all of these properties:

- logic and state matter more than engine/editor tooling
- bots or AI agents are first-class features
- benchmarking or repeated evaluation matters
- replay / trace artifacts are important
- multiple frontends may exist over time
- browser delivery is useful for the human-facing side
- native performance or concurrency matters for heavy workloads
- long-term maintainability matters more than the fastest prototype

This is especially useful when a small prototype grows into a more serious system.

---

## 3. High-level architecture

The recommended architecture has four main layers:

1. **Native core**
2. **Bot / agent runtime**
3. **Evaluation framework**
4. **Frontend(s)**

---

## 4. Layer responsibilities

## 4.1 Native core

The native core owns the domain rules and authoritative state transitions.

### Responsibilities

- state representation
- rules / simulation logic
- deterministic stepping
- scenario or level setup
- move / action validation
- terrain, board, or world generation
- physics or game rule enforcement
- observation / sensor generation
- serialization-friendly state structures
- replay / trace hooks

### Inputs and outputs

The core should consume:

- explicit action or command input
- configuration or scenario input
- deterministic seeds if applicable

The core should produce:

- authoritative next state
- events / outcomes
- observation views
- replay / trace data

### Important boundary

The core should **not** own:

- rendering
- window/input APIs
- browser code
- plotting code
- process management
- tournament orchestration
- UI-specific state

Useful mental model:

- **actions in**
- **state + events out**

---

## 4.2 Bot / agent runtime

The bot layer owns decision-making.

### Responsibilities

- control logic
- search / planning
- heuristics
- policy inference
- optimization routines
- bot-side configuration
- optional internal bot telemetry

### Contract

Bots should consume:

- an observation or state view
- optional timing or budget data
- optional configuration

Bots should produce:

- an action or command
- optional debug metadata

### Important boundary

Bots should not mutate core state directly.

The preferred interaction pattern is:

- `observe -> decide -> action`

---

## 4.3 Evaluation framework

The evaluation layer owns repeated execution, measurement, and analysis.

### Responsibilities

- benchmarking
- self-play
- bot-vs-bot arenas
- batch scenario runs
- tournament orchestration
- telemetry aggregation
- profiling
- plotting
- report generation
- regression testing
- replay / trace pack production

### Important boundary

The eval layer should not become the place where rules live.

It should orchestrate runs around the same core and bot interfaces used elsewhere.

---

## 4.4 Frontend(s)

Frontends own human interaction and presentation.

### Responsibilities

- human input
- rendering / presentation
- replay viewing
- scenario browsers
- debug overlays
- live match or sim inspection
- simple editing or sandbox controls if needed

### Multiple frontend support

The architecture should explicitly allow multiple frontends.

Examples:

- web frontend
- desktop frontend
- pygame frontend
- terminal/debug UI
- replay-only viewer

### Important boundary

Frontends should not embed ad hoc copies of rule logic.

They should consume:

- core state
- replay streams
- command interfaces
- optional embedded Wasm builds

---

## 5. Data contracts

The most important design work is the contract between layers.

Define these contracts clearly:

### 5.1 Action schema

Stable representation of what a player, human, or bot can do.

### 5.2 State schema

Stable authoritative representation of the world or game state.

### 5.3 Observation schema

Stable view presented to bots or frontends.

This may be:

- full state
- partial state
- sensor data
- filtered or role-specific state

### 5.4 Replay / trace schema

Stable structure for:

- seeds
- initial setup
- actions over time
- state snapshots or deltas
- outcomes
- metrics

### 5.5 Metrics schema

Stable format for analytics and evaluation outputs.

Treat these contracts as first-class artifacts.

---

## 6. Determinism guidance

For projects involving benchmarking, self-play, or agent development, determinism is highly valuable.

Aim for:

- deterministic seeds
- stable stepping order
- reproducible outcomes
- versioned replay formats
- controlled floating-point assumptions when relevant

Why this matters:

- debugging
- regression testing
- bot evaluation
- cross-implementation comparisons
- agent iteration

---

## 7. Frontend strategy

A common failure mode is letting the first frontend become the whole architecture.

Assume from the beginning that there may be multiple frontends with different roles:

- one for rapid prototyping
- one for browser delivery
- one for debugging
- one for replay inspection

Examples of frontend roles:

- **prototype frontend** for fast iteration
- **web frontend** for sharing and accessibility
- **debug frontend** for rich inspection
- **lightweight replay viewer** for quick analysis

---

## 8. Browser and Wasm strategy

## 8.1 Browser as presentation layer

For many projects, the browser is best treated as:

- visualization layer
- replay viewer
- shareable demo surface
- remote-control surface
- lightweight interactive sandbox

This often works better than forcing the entire heavy execution pipeline into the browser.

## 8.2 Wasm as optional deployment target

Wasm is useful for selected components:

- deterministic core
- lightweight bots
- local sandbox play
- replay stepping
- browser-safe demos

Wasm is less suitable for:

- very heavy benchmark pipelines
- complex native process orchestration
- unrestricted native integrations
- thread-heavy systems unless browser constraints are acceptable

Recommended posture:

- treat Wasm as an **optional target**
- do not assume the entire system must run in-browser

---

## 9. Repository strategy

When transitioning from an existing prototype, there are two broad options:

- migrate in place
- start a new repo

### Prefer a new repo when

- the existing project is tightly coupled to old tooling/runtime choices
- the new architecture changes the project center significantly
- you need clean contracts and modularity
- the existing repo is valuable as a stable reference/oracle
- you want to avoid mixed assumptions

### Prefer in-place migration when

- the architecture change is incremental
- the current repo already has clean boundaries
- preserving one continuous codebase matters more than cleanliness

Recommended general rule:

If the future system is fundamentally a different architecture, use a **new repo** and keep the old one as:

- reference implementation
- behavior oracle
- prototype shell
- regression target

---

## 10. Generic implementation phases

## Phase 1: define the architecture skeleton

- create the new repo
- define module boundaries
- define state/action/observation/replay contracts
- create minimal runnable vertical slice

## Phase 2: build the core

- implement authoritative state
- implement rule/step logic
- add serialization
- add deterministic seeds
- add replay hooks

## Phase 3: build the first bots/agents

- random or trivial baseline
- deterministic baseline bot
- one stronger reference bot

## Phase 4: build evaluation tooling

- single-run CLI
- batch runner
- arena/tournament runner
- metrics output
- replay export
- basic reporting

## Phase 5: build the main frontend

- render state
- consume replay data
- support human input if needed
- inspect outcomes

## Phase 6: add optional deployment targets

- Wasm build
- alternate frontend
- compatibility adapters
- remote runner integration

---

## 11. Generic module layout

Reusable module layout:

- `project-core`
- `project-bot`
- `project-eval`
- `project-cli`
- `project-web`
- `project-adapter-*`
- optional `project-wasm`

### Suggested responsibilities

#### `project-core`

- authoritative state
- rules / simulation
- scenario setup
- replay schema
- serialization helpers

#### `project-bot`

- bot interface
- built-in bots
- search / planning / policy logic

#### `project-eval`

- batch runs
- arena / self-play
- benchmarks
- metrics and reports

#### `project-cli`

- developer entrypoints
- run one match / scenario
- inspect replays
- export artifacts

#### `project-web`

- browser UI
- replay viewer
- spectator mode
- human interaction UI

#### `project-adapter-*`

- protocol bridges
- external engine compatibility
- legacy system interop

#### `project-wasm`

- optional browser-target build of selected pieces

---

## 12. Bot experimentation guidance

This architecture is designed to support multiple bot approaches side by side.

Examples:

- rule-based bot
- search-based bot
- heuristic bot
- RL bot
- hybrid search + learned bot
- remote/service-backed bot
- LLM-assisted tooling around bots

Good practice:

Always keep at least one:

- simple baseline bot
- deterministic reference bot

---

## 13. RL and LLM positioning

## 13.1 RL

RL is often a good fit when:

- self-play is easy
- episodes are cheap
- rewards are clear
- experimentation is part of the goal

But RL should usually come **after** the core, replay, and baseline bot infrastructure are stable.

## 13.2 LLMs

LLMs are usually more useful as development tools than as the main runtime brain.

Useful roles include:

- code generation
- refactoring assistance
- test generation
- replay analysis
- heuristic brainstorming
- debugging explanations
- data labeling or summarization

They can also be explored as agent components, but should not be assumed to be the best primary decision engine for structured games or simulations.

---

## 14. Adapter philosophy

If the project needs to interoperate with an external ecosystem or protocol, keep that integration in a dedicated adapter layer.

Examples:

- tournament protocol
- engine protocol
- legacy save format
- remote service wrapper

Rule:

Adapters should wrap the architecture.  
They should not define it.

---

## 15. Performance guidance

Before optimizing, decide where the real work happens.

### Native is often best for

- heavy search
- large batch evaluation
- simulation throughput
- profiling and benchmarking
- process orchestration
- concurrency-heavy workloads

### Browser/Wasm is often best for

- lightweight local simulation
- demos
- replays
- interaction
- visualization

Do not force everything into the same runtime if the use cases are different.

---

## 16. Testing guidance

At minimum, build tests for:

- deterministic step behavior
- serialization round-trips
- replay integrity
- rule edge cases
- bot contract compliance
- eval runner stability

Helpful additional tests:

- golden replays
- scenario regression packs
- cross-frontend state consistency
- adapter compatibility tests

---

## 17. Non-goals for initial bootstrap

Avoid trying to solve everything at once.

Common non-goals for v1:

- strongest possible bot/AI
- maximal frontend polish
- every deployment target
- every compatibility adapter
- full ML pipeline
- perfect performance tuning
- final data dashboard system

Start with a small vertical slice that proves the architecture.

---

## 18. Guidance for coding agents

When implementing this architecture:

1. Keep the core deterministic where practical.
2. Do not let frontend code own rules.
3. Do not let bots mutate core state directly.
4. Keep replay/log artifacts first-class.
5. Prefer explicit schemas over ad hoc structures.
6. Keep adapters at the boundary.
7. Support multiple frontends by design.
8. Treat Wasm as optional unless explicitly required.
9. Build one minimal vertical slice before expanding scope.
10. Preserve at least one simple baseline bot and one baseline frontend.

---

## 19. Template decision checklist

Before implementation, answer these questions.

### Core

- What is the authoritative state?
- What is one simulation/game step?
- What is the action schema?
- What is the observation schema?
- What is deterministic and what is not?

### Bots

- Do bots see full state or partial observations?
- What timing/budget interface exists?
- How are bot results logged?

### Eval

- What metrics matter?
- What artifacts are saved?
- What is the basic batch-run workflow?

### Frontend

- Is the first frontend live-play, replay-first, or both?
- Does the frontend run local logic, remote logic, or both?

### Deployment

- What must be native?
- What should be browser-accessible?
- Is Wasm optional or required?

---

## 20. Final recommendation

For any new game/sim/agent project:

- design the **core first**
- make the **bot interface explicit**
- make **replays and metrics first-class**
- keep the **frontend replaceable**
- use **native execution** for heavy workloads
- use the **browser** for presentation and lightweight interaction
- treat **Wasm** as an optional bridge where it adds value
