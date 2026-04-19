# Gomoku2D Web FE Architecture Options Doc
**Date:** 2026-04-18  
**Purpose:** practical frontend architecture handoff for coding agents  
**Scope:** DOM app shell technology choices, state strategy, Phaser integration, and pivot options  
**Important stance:** this document is intentionally **not** a locked-in framework decision. It is a practical options-and-tradeoffs guide with a recommended starting point.

---

## 1. Executive summary

As more functionality moves out of Phaser, the project needs a clearer story for the **DOM-side frontend**.

The main question is not:
- “Should we use pure HTML/CSS/JS?”

The real question is:
- “What is the most practical modern web frontend stack for a game-like app shell that embeds Phaser for the board?”

### Recommended starting point

Start with a **light, client-first React stack**:

- **React** for DOM UI
- **React Router** for top-level screens
- **Zustand** for client/app-shell state
- **TanStack Query** for backend/server state
- **Phaser** embedded as a board renderer/interaction surface

### Why this is the recommended starting point

Because it is:
- practical
- low-friction
- modern
- easy to iterate on
- compatible with a static frontend deployment story
- good for UI complexity that is growing but not yet enterprise-scale

### Important principle

This is a **default starting point**, not a forever commitment.

The architecture should be designed so that:
- the DOM app shell can evolve
- state-management choices can pivot
- the Phaser boundary stays stable
- the backend integration layer remains replaceable

---

## 2. What “DOM” means in this project

In this project, “DOM” should be interpreted as:

> a modern component-based frontend application rendering to the browser DOM

Not:
- old-school hand-written jQuery-style app logic
- pure vanilla HTML pages with scattered scripts
- Phaser pretending to be a general UI framework

So when we say DOM-side UI, we mean:
- screen components
- panels
- lists
- forms
- cards
- navigation
- overlays
- async data-driven app surfaces

---

## 3. Architectural goal

The frontend should be split into three layers:

### A. DOM app shell
Owns:
- Home / Online / Replays / Profile
- Match shell/HUD
- overlays and menus
- forms/lists/navigation
- auth and room flows

### B. Shared controller/state layer
Owns:
- app state
- match shell state
- replay/review state
- backend data synchronization policy
- event wiring between DOM and Phaser

### C. Phaser board adapter
Owns:
- board rendering
- board input
- board-space overlays
- board animations

This layering should remain stable even if the exact React/state tool choices change.

---

## 4. Selection criteria

Any frontend stack should be judged on:

1. **Ease of building app-shell UI**
2. **Ease of embedding Phaser**
3. **Ease of handling async backend state**
4. **Ease of routing/navigation**
5. **Complexity overhead**
6. **Ability to pivot later**
7. **Static hosting friendliness**
8. **Compatibility with the project’s current scale**

---

## 5. Current practical frontend landscape

Today, the default “React app” story is not the old CRA + Redux stack.

### Important ecosystem realities
- Create React App is deprecated.
- React officially recommends starting with a framework or a modern build tool.
- Server state and client UI state are usually treated as separate problems now.
- Redux is no longer the default answer for every React app.

That means the real choices are now more like:
- framework vs lighter client app
- client state vs server state
- simple store vs heavy store
- route/data conventions vs custom setup

---

## 6. Recommended starting option

## Option A: Practical default

### Stack
- **Vite**
- **React**
- **React Router**
- **Zustand**
- **TanStack Query**
- **Phaser**

### Why this is the best starting point
This option is:
- light
- modern
- fast to build with
- easy to host statically
- easy to reason about
- good for an app with a growing shell but still modest scale

### Responsibilities

#### React
- component model
- screen composition
- layout
- overlays
- sidebar/HUD
- Home / Online / Replays / Profile
- DOM-side Match shell

#### React Router
- top-level app navigation
- route structure
- route transitions
- screen ownership

#### Zustand
- client-side state for:
  - current app mode
  - selected room/replay
  - review cursor
  - panel visibility
  - match shell state
  - Phaser bridge state if needed

#### TanStack Query
- async/server state for:
  - auth/session fetches
  - room data
  - replay list
  - profile data
  - leaderboard/history later
  - cache and refetch policy

#### Phaser
- board-only rendering and board input

### Main upside
You get a modern frontend without overcommitting to a heavier pattern too early.

### Main downside
You are choosing and wiring a few tools yourself rather than getting one “batteries-included” framework opinion.

---

## 7. Why not plain React + Redux by default

Redux is still valid, but it should not be treated as the automatic default.

### When Redux/RTK is strong
- very large app-wide state graph
- many contributors
- desire for strict state transitions
- complex tooling/debugging needs
- lots of global non-server state

### Why it may be overkill right now
For Gomoku2D:
- much of the complexity is UI shell + backend data
- server state should be separated anyway
- a large central reducer architecture may add more ceremony than value initially

### Practical view
If state complexity grows significantly later, pivoting from Zustand to Redux Toolkit is possible.

So:
- **do not start with Redux just because it is famous**
- use it when the problem actually becomes Redux-shaped

---

## 8. Alternative options

## Option B: React Router framework-first
### Stack
- **React Router framework mode**
- **React**
- **TanStack Query**
- **Zustand**
- **Phaser**

### Upsides
- stronger route/data conventions
- more structured app organization
- code splitting and richer route model
- still compatible with SPA/static deployment modes

### Downsides
- slightly more framework opinion up front
- less minimal than a plain Vite setup
- may be more structure than needed at the current stage

### Good fit when
- routing and page/data boundaries are becoming central
- the app shell starts to feel like a real product app, not just a game wrapper

---

## Option C: Next.js-style full-stack React
### Stack
- **Next.js**
- **React**
- state choices as needed
- **Phaser** embedded client-side

### Upsides
- strong ecosystem
- SSR/RSC/full-stack options
- broad deployment and app architecture patterns
- good if the project becomes much more web-product-like

### Downsides
- heavier than necessary for the current problem
- more framework concepts than the project may need
- Phaser embedding can be more awkward than in a straightforward client-side app
- may be overkill if the frontend remains mostly client-driven

### Good fit when
- the app becomes heavily web-product-oriented
- SEO/server rendering/admin surfaces become much more important
- there is a genuine need for more full-stack React patterns

---

## Option D: Vanilla/minimal DOM shell
### Stack
- custom JS/TS
- router of choice or no real router
- direct DOM manipulation
- Phaser

### Upsides
- minimal dependencies
- total control
- low conceptual overhead for very tiny projects

### Downsides
- scales poorly as UI complexity rises
- more custom plumbing
- harder to keep consistent
- easier to accumulate UI debt

### Good fit when
- project is still tiny
- shell is extremely small
- long-term scalability is not a concern

### Recommendation
Not recommended for Gomoku2D now that online/replay/profile features are becoming real.

---

## 9. Recommended pivot strategy

This project should optimize for **practicality and easy pivoting**, not theoretical purity.

### Recommended strategy
Start with:
- React
- React Router
- Zustand
- TanStack Query
- Phaser

But keep the architecture stable enough that you can later pivot:
- Zustand -> Redux Toolkit
- Vite app -> framework-first app
- client-only routing -> richer route/data conventions
- light shell -> more productized shell

### Key rule
The most important thing to stabilize is **the boundary**, not the exact library choice.

That means:
- stable DOM vs Phaser ownership
- stable state ownership model
- stable event contract
- stable screen structure

Those matter more than whether the store is Zustand or Redux.

---

## 10. State model recommendation

Split state into categories.

## 10.1 Client UI state
Examples:
- current screen
- panel/menu visibility
- selected replay
- selected room
- review cursor/index
- local shell preferences
- DOM-only interaction state

### Recommended tool
- Zustand

## 10.2 Server/async state
Examples:
- auth/session profile
- room lists
- invite status
- replay list/history
- profile data
- backend-driven match metadata

### Recommended tool
- TanStack Query

## 10.3 Board render/input state
Examples:
- board position
- hover cell
- selected cell
- last move marker
- board animation state

### Ownership
- authoritative state in shared/controller layer
- Phaser receives render/input props and emits events

### Important rule
Do not store board truth only inside Phaser scene internals.

---

## 11. Suggested frontend module boundaries

### `app/`
- app bootstrap
- route config
- providers
- shell layout

### `screens/`
- Home
- Match
- Online
- Replays
- Profile

### `components/`
- buttons
- panels
- player cards
- badges
- rows
- overlays

### `state/`
- Zustand stores
- UI state
- match shell state
- review state
- room selection state

### `queries/`
- TanStack Query hooks
- backend fetch logic
- cache keys
- mutations

### `phaser/`
- board adapter
- phaser bootstrapping
- board scene
- event bridge
- render model mapping

### `controllers/` or `services/`
- shared event orchestration
- match controller
- replay controller
- online controller

The exact folder structure can vary, but these conceptual boundaries are useful.

---

## 12. Route structure recommendation

A likely route model:

- `/` -> Home
- `/match` -> Match
- `/online` -> Online
- `/replays` -> Replay library
- `/replays/:id` -> Review mode
- `/profile` -> Profile
- optional `/room/:id` -> Online room landing
- optional `/match/:id` -> specific match/replay state

### Why routes matter
As more functionality moves into DOM, route clarity becomes important for:
- shareability
- replay URLs
- room links
- app structure
- easier mental model

---

## 13. Phaser integration recommendation

Phaser should be treated as an embedded board module.

### Recommended pattern
- React component owns a container element
- Phaser mounts into that container
- React/controller passes render model and mode flags down
- Phaser emits events upward through a narrow bridge

### Good React -> Phaser inputs
- board position
- visual mode
- hover/selection state
- review index
- animation commands
- theme/tile size if needed

### Good Phaser -> React/controller outputs
- intersection hovered
- intersection clicked
- animation complete
- board ready

### Important rule
Do not let random DOM components call deep Phaser scene methods directly.

Use a controlled bridge.

---

## 14. Match shell recommendation

The Match screen should be a DOM page composed of:

- board container
- sidebar/HUD
- context panel
- action stack
- overlays

Phaser renders only inside the board container.

This is a key architecture clarification.

### DOM owns
- player cards
- timers
- turn indicators
- menu button
- reset/resign/share
- result panel
- replay controls

### Phaser owns
- stones
- markers
- board hover
- last move highlight
- move numbers on board
- board-local effects

---

## 15. Practical implementation order

## Phase 1
- set up DOM app shell with chosen stack
- define top-level screens
- keep current Phaser board integration minimal

## Phase 2
- move match HUD and controls into DOM
- establish board adapter boundary
- define shared controller/state model

## Phase 3
- add server-state layer with TanStack Query
- implement Home / Online / Replays / Profile shells
- connect initial backend data flows

## Phase 4
- refine board bridge
- polish route structure
- add replay URLs / room flows / profile flows

## Phase 5
- reassess if state complexity now justifies Redux Toolkit or a stronger framework model

This keeps the initial migration practical.

---

## 16. Pivot triggers

These are signs you may want to pivot tools later.

## Pivot from Zustand to Redux Toolkit if:
- client state becomes very large and highly interdependent
- many contributors are touching the same state graph
- debug tooling and stricter patterns become essential
- many state transitions need explicit orchestration

## Pivot from Vite/light React shell to a stronger framework if:
- route/data complexity becomes dominant
- server-side rendering becomes strategically important
- the app shell becomes much broader than the game itself
- you need stronger built-in framework conventions

## Do not pivot just because
- a library is fashionable
- “real apps use X”
- the current setup is not theoretically perfect

Pivot when the problem clearly changes shape.

---

## 17. Risks and tradeoffs

## Risk 1: too many libraries too early
### Mitigation
Only adopt libraries with clear ownership:
- React = components
- React Router = navigation
- Zustand = client state
- TanStack Query = server state
- Phaser = board

## Risk 2: duplicated state between query/store/phaser
### Mitigation
Document ownership clearly.
Do not store the same thing as source-of-truth in multiple places.

## Risk 3: overengineering before features exist
### Mitigation
Prefer the practical default stack.
Keep boundaries stronger than abstractions.

## Risk 4: framework churn
### Mitigation
Optimize for stable architecture and narrow contracts, not library permanence.

---

## 18. Coding-agent rules

1. Default new app UI work to React/DOM components.
2. Keep Phaser limited to board-space rendering and interaction.
3. Use Zustand only for client-side shell/UI state.
4. Use TanStack Query for backend/server data flows.
5. Do not treat Redux as mandatory unless complexity proves it.
6. Keep the React <-> Phaser bridge narrow and explicit.
7. Prefer route/page clarity over dumping more panels into Match.
8. Favor a practical implementation that can pivot later over a theoretically perfect stack chosen too early.

---

## 19. Current recommended choice

### Start here
- **Vite**
- **React**
- **React Router**
- **Zustand**
- **TanStack Query**
- **Phaser**

### Why
This is the best balance of:
- speed
- flexibility
- modern practice
- low complexity
- easy future evolution

It is a practical choice, not an ideological one.

---

## 20. Final recommendation

The web FE story for Gomoku2D should be:

- **React-based DOM app shell**
- **Phaser board adapter**
- **split client state from server state**
- **choose a light practical stack first**
- **pivot only when the real problem changes**

The most important thing to lock down is not a specific framework religion.

It is:
- screen ownership
- state ownership
- Phaser boundary
- route structure
- event flow

If those are stable, the library choices can evolve safely.
