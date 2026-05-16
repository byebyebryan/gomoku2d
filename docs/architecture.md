# Architecture

## Shape

Three components, one repo:

```
┌────────────────────────────┐      ┌────────────────────────────┐
│   gomoku-web (browser)     │      │ gomoku-api / backend       │
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
- **`gomoku-api` / backend** is the future Rust service for the features that
  need cloud durability or server trust. It starts as
  `gomoku-bot-lab/gomoku-api/` so it can share workspace deps and graduate to a
  top-level `gomoku-backend/` only when deploy cadence or ownership justifies
  the split.

## Frontend stack

### The decision

- **React + TypeScript** — component model for the app shell.
- **Vite** — dev server and bundler.
- **React Router** — URL-driven screens for the current local-first app
  (`/`, `/match/local`, `/replay/:matchId`, `/profile`), with future
  cloud/online routes added later.
- **Zustand** — client state (current view, draft moves, UI toggles).
- **Firebase SDK directly** — sign-in, Firestore subscriptions, storage for
  cloud-backed features.
- **Local persistence** — local profile + local history live in browser
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
CSS Modules (`*.module.css`) plus a small hand-rolled token layer in
`global.css` for color, spacing, and type. The shell is small, the look is
deliberately distinctive, and the tokens are documented in `ui_design.md`.
A utility framework would add machinery and fight the retro/chunky visual
direction. Decided after the first screen builds landed.

## The DOM/Phaser boundary

This is the load-bearing architectural decision for the FE rewrite.

**Phaser owns:** the 15x15 grid, stone sprites, pointer, tactical warnings,
forbidden move overlays, result sequence numbers, and win-line hover. Anything
that benefits from pixel-level control or frame-based animation.

**DOM owns:** everything else. Status HUD, player identity/info, rule toggles,
result strips, replay transport/timeline, local record/history views, menus,
modals, auth UI, and any future cloud/online shell surfaces.

**They communicate through a narrow interface:**

```ts
// React → Phaser: simplified declarative board state
interface BoardProps {
  cells: Cell[][];
  counterThreatMoves: Move[];
  currentPlayer: 1 | 2;
  forbiddenMoves: Move[];
  lastMove?: Move;
  interactive: boolean;
  touchControlMode: 'none' | 'pointer' | 'touchpad';
  showSequenceNumbers: boolean;
  status: MatchStatus;
  imminentThreatMoves: Move[];
  threatMoves: Move[];
  winningMoves: Move[];
  winningCells: Move[];
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
- **Replay analysis:** `gomoku-analysis` owns bounded corridor traceback.
  `gomoku-eval` uses it for fixture/report generation; `gomoku-wasm` exposes a
  browser-safe analyzer bridge; `gomoku-web` converts saved matches to core
  replay JSON and renders the result.
- **Server (future):** `gomoku-api` depends on `gomoku-core` as a Cargo path
  dependency. Same Rust code, native target.
- **CLI / eval tools:** already using `gomoku-core` via path deps.

This means "is this move legal?" and "did this player win?" have exactly
one answer, regardless of where the question is asked. Tactical hint facts that
depend on rules semantics also live below the UI layer. The browser keeps a
rolling-frontier-backed `WasmBoard` and reads one threat snapshot for immediate
wins, immediate threats, imminent replies, counter-threat replies, and Renju
forbidden moves. The canonical winning line remains a separate result
visualization query. Rule-legality feedback such as Renju forbidden moves stays
always on; tactical assistance categories are filtered by device-local UI
preferences before they reach the board renderer.

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

### Casual / local match

```
user clicks cell
  → React dispatches place-move
    → gomoku-wasm applies move, returns new board
      → Zustand store updated
        → local history/profile persisted in browser storage
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
    → browser merges it into profiles/{uid}.match_history replay/summary tiers
      → browser writes the capped profile snapshot when the 5-minute sync gate is open
      → local cache stays in sync for quick resume/viewing
```

This path is still intentionally low-trust:

- gameplay stays client-side
- no per-move backend validation
- one coalesced profile snapshot write instead of one cloud document per match
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

- **Casual / free play** — no per-move backend validation; fine for local play,
  signed-in private bot history, and disposable local sessions.
- **Trusted / cloud-backed play** — backend validates every move. Used for
  ranked matches, server-owned online history, and any replay we intend to
  trust or share publicly.

That keeps the hot path cheap for throwaway play while making persistent/public
features trustworthy.

## Version sequence

The old v0.1-to-v0.2 migration is complete. React now owns the shell, Phaser is
the board renderer, and local play/history/replay are the working product
baseline. The `v0.3` backend-continuity line has since added optional Firebase
Auth, private cloud profile/history, and owner-scoped Firestore rules without
moving casual gameplay out of the browser.

From here, the version plan is:

- **P4 / `v0.4` — lab-powered product identity.** Use the Rust core/bot/eval
  tools for replay analysis, critical-moment tagging, puzzles, bot
  personalities, and related learning modes.
- **P5 / `v0.5` — presentation systems and skins.** Extend the frontend visual
  system so the product can support more than one tone without losing the
  board-first design.
- **P6 / `v0.6` — online product expansion.** Add the Cloud Run authority,
  direct challenge / PvP flows, trusted match persistence, ranked or matchmaking
  surfaces, and deliberate public/shareable artifacts.

Details are in `roadmap.md`. The architectural contract — what React owns, what
Phaser owns, how they talk, and where `gomoku-core` is authoritative — is the
part that needs to hold across all of it.
