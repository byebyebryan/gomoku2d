# Architecture

## Shape

Three components, one repo:

```
┌────────────────────────────┐      ┌────────────────────────────┐
│   gomoku-web (browser)     │      │   gomoku-backend (server)  │
│   ├─ React app shell (DOM) │◄────►│   ├─ HTTP API              │
│   └─ Phaser board (canvas) │      │   └─ Firestore client      │
│       ↑ shares core via    │      │       ↑ shares core via    │
│       wasm bridge          │      │       cargo path dep       │
└────────────────────────────┘      └────────────────────────────┘
                │                                  │
                └──────┐                  ┌────────┘
                       ▼                  ▼
             ┌─────────────────────────────────────┐
             │   gomoku-bot-lab (Rust workspace)   │
             │   core · rules · bots · eval · cli  │
             └─────────────────────────────────────┘
```

- **`gomoku-web`** is the product surface. Everything the player sees lives
  here.
- **`gomoku-bot-lab`** is the Rust workspace. Core rules, bots, evaluation
  tools, and the wasm bridge live together because they share logic. The web
  and (future) server both borrow from it.
- **`gomoku-backend`** (not built yet) is a Rust service that hosts matches,
  persists game state, and runs the features that need server trust. Starts
  as `gomoku-bot-lab/gomoku-api/` in the workspace, graduates to a top-level
  sibling when its deploy cadence diverges.

## Frontend stack

### The decision

- **React + TypeScript** — component model for the app shell.
- **Vite** — dev server and bundler.
- **React Router** — URL-driven screens (`/`, `/match/:id`, `/replays/:id`,
  `/puzzles`, `/profile`).
- **Zustand** — client state (current view, draft moves, UI toggles).
- **Firebase SDK directly** — sign-in, Firestore subscriptions, storage for
  cloud-backed features.
- **Local persistence** — local guest profile + guest history live in browser
  storage until the user chooses to sign in.
- **Phaser** — board rendering only. Receives a plain board state and emits
  intent events.

### Why this, not the alternatives

**Why React, not Svelte / Solid / vanilla?**
Biggest ecosystem for the pieces we'll want off the shelf — auth widgets,
routing, form controls, charting for post-game analysis. The project is
small enough that React's runtime cost doesn't matter. Solid would be nicer
ergonomically; Svelte would be lighter; neither pays for the lost ecosystem
leverage on a hobby-scope project.

**Why not Next.js?**
We don't need SSR, file-based routing, server components, or image
optimization. Next.js is optimized for content sites with some interactivity;
we're an app with some content. Vite + React Router is a better fit and
simpler to reason about.

**Why Zustand, not Redux / Jotai / Context?**
Redux Toolkit is fine but overshoots for a single-user app state this small.
Context triggers too-broad re-renders for live game state. Zustand is a
minimal store with selectors — the least machinery for the job.

**Why Firebase SDK directly, not TanStack Query?**
TanStack Query is the default answer for data-fetching these days, but it's
built around a *pull* model (fetch, cache, invalidate, refetch). Firestore
is *push* — you subscribe to a document or query via `onSnapshot` and the
server streams updates. Wrapping that in TanStack Query fights the grain;
you end up managing two caches. Small custom hooks over `onSnapshot` are
less code and closer to the truth.

**Why no CSS framework (Tailwind / MUI / etc.)?**
TBD. Leaning toward CSS Modules + a small hand-rolled design system (tokens
for color, spacing, type), since the UI is small and we want a distinctive
look. Tailwind is a reasonable alternative if the utility-first workflow
clicks; the decision can be deferred to the first screen build.

## The DOM/Phaser boundary

This is the load-bearing architectural decision for the FE rewrite.

**Phaser owns:** the 15×15 grid, stone sprites, hover preview, last-move
indicator, win-line animation. Anything that benefits from pixel-level
control or frame-based animation.

**DOM owns:** everything else. Player name cards, turn indicator, move
history list, clocks, resign/undo buttons, menus, modals, auth UI, lobby,
replay timeline, puzzle prompts.

**They communicate through a narrow interface:**

```ts
// React → Phaser: declarative board state
interface BoardProps {
  cells: Cell[][];
  lastMove?: Move;
  hoverable: boolean;
  highlights?: Move[];        // for puzzle hints, critical-move tagging
}

// Phaser → React: intent events
type BoardEvent =
  | { type: 'hover'; cell: Move }
  | { type: 'place'; cell: Move }
  | { type: 'unhover' };
```

A single `<Board>` React component wraps the Phaser instance, passes props
in, subscribes to events out. Phaser never reads from the global store or
Firestore directly.

This is a departure from v0.1, where Phaser scenes held game state and
drove the UI. The rewrite flips it: state lives in React/Zustand, Phaser
becomes a stateless view.

## Core sharing

One rules implementation, reused everywhere:

- **Browser:** `gomoku-bot-lab/gomoku-wasm` compiles `gomoku-core` to wasm.
  Web imports it via `gomoku-wasm = "file:../gomoku-bot-lab/gomoku-wasm/pkg"`
  in `package.json`. Vite's wasm plugin handles the load.
- **Server (future):** `gomoku-backend` depends on `gomoku-core` as a Cargo
  path dependency. Same Rust code, native target.
- **CLI / eval tools:** already using `gomoku-core` via path deps.

This means "is this move legal?" and "did this player win?" have exactly
one answer, regardless of where the question is asked.

## Data flow

### Offline bot match (works today)

```
user clicks cell
  → React dispatches place-move
    → gomoku-wasm applies move, returns new board
      → Zustand store updated
        → <Board> re-renders with new props
        → bot's turn: wasm bot picks a move, same loop
```

### Casual / guest match

```
user clicks cell
  → React dispatches place-move
    → gomoku-wasm applies move, returns new board
      → Zustand store updated
        → local guest history/profile persisted in browser storage
        → <Board> re-renders with new props
```

This lane is intentionally low-friction and low-trust:

- browser-authoritative
- local-first
- no per-move backend validation in the hot path

### Signed-in casual match history

```
user finishes a local bot/casual match while signed in
  → browser serializes compact replay + summary
    → browser writes one private history record to profiles/{uid}/matches/{id}
      → local cache stays in sync for quick resume/viewing
```

This path is still intentionally low-trust:

- gameplay stays client-side
- no per-move backend validation
- one cloud write on match end instead of syncing the whole live match
- good fit for bot matches and private history

Public sharing and ranked/trusted features do not rely on this path alone.

### Trusted / cloud-backed match (target)

```
user clicks cell
  → React POSTs move to Cloud Run
    → Cloud Run validates move against gomoku-core
      → Cloud Run writes authoritative match state to Firestore
        → Firestore pushes update to subscribed clients via onSnapshot
          → clients re-render from server-written state
```

Two trust levels exist on purpose:

- **Casual / free play** — no per-move backend validation; fine for guest play,
  signed-in private bot history, and disposable local sessions.
- **Trusted / cloud-backed play** — backend validates every move. Used for
  ranked matches, server-owned online history, and any replay we intend to
  trust or share publicly.

That keeps the hot path cheap for throwaway play while making persistent/public
features trustworthy.

## Migration plan

The v0.1 code under `gomoku-web/src/` (Phaser-only, scene-driven) needs to
be rewritten, not incrementally patched. Keeping the Phaser board code
(`board/`) is viable; the scenes and `main.ts` entry are not.

Rough sequence:

1. Bring up React + Vite in `gomoku-web/`, with React Router and Zustand
   wired but mostly empty.
2. Wrap the existing `board/` renderer in a `<Board>` React component that
   takes props and emits events. Delete the Phaser scenes.
3. Rebuild the match screen in DOM (player cards, turn indicator, history).
   Offline bot match working end-to-end in the new architecture.
4. Add local guest profile persistence, then Firebase sign-in and cloud-profile
   promotion.
5. Add private cloud history save at match end for signed-in casual play.
6. Add trusted cloud-backed online match flow.
7. Lab-powered features (puzzles, replay critical-move tagging).

Details are in `roadmap.md`. The architectural contract — what React owns,
what Phaser owns, how they talk — is the part that needs to hold across
all of it.
