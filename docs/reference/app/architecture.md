# Architecture

## Shape

Current components, one repo:

```
┌────────────────────────────┐      ┌────────────────────────────┐
│   gomoku-web (browser)     │◄────►│   Firebase / Firestore     │
│   ├─ React app shell (DOM) │      │   Auth + profile/history   │
│   └─ Phaser board (canvas) │      │   sharing + replay library │
│       ↑ shares core via    │      └────────────────────────────┘
│       wasm bridge          │
└────────────────────────────┘
                │
                ▼
             ┌─────────────────────────────────────┐
             │   gomoku-bot-lab (Rust workspace)   │
             │   core · rules · bots · eval · cli  │
             └─────────────────────────────────────┘
```

- **`gomoku-web`** is the product surface. Everything the player sees lives
  here.
- **`gomoku-bot-lab`** is the Rust workspace. Core rules, bots, evaluation
  tools, and the wasm bridge live together because they share logic. The web
  app and native tools use the same rules.
- **Firebase Auth + Firestore** are the deployed backend today: optional Google
  sign-in, owner-scoped profile documents, private settings/history sync,
  public profiles/catalogs, a private replay library, and hardened rules.
- Multiplayer remains deferred. [Casual multiplayer research](../backend/p2p_multiplayer.md)
  prefers peer-to-peer moves with a small joining and signaling service.
  [Trusted server authority](../backend/future_cloud_run.md) is a parked design.

## Frontend stack

### The decision

- **React + TypeScript** — component model for the app shell.
- **Vite** — dev server and bundler.
- **React Router** — URL-driven screens for the current app (`/`,
  `/match/local`, `/replay/:matchId`, `/settings`, `/profile`, `/rules/`,
  `/guide/`, `/lab/`, `/visuals/`, `/dev/`, `/dev/process/`, `/dev/reviews/`,
  curated review case URLs, `/replay/shared/`, `/player/`, `/privacy/`, `/terms/`),
  with online-match routes still deferred.
- **Zustand** — client state (current view, draft moves, UI toggles).
- **Firebase SDK directly** — sign-in, explicit profile reads and checkpoint writes for
  cloud-backed features.
- **Local persistence** — local profile + local history remain in browser
  storage, with optional account continuity layered on top.
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
The private-history flow explicitly reads one owner profile and coalesces
settings and history into checkpoint writes. Public profile/catalog and private
library features use bounded reads and revision-checked writes. Small stores
own auth identity, cooldown, pending work, and reset barriers. The app loads
profiles on demand and checks the private profile snapshot in a transaction
before replacing it to reject stale cross-device writes. Revisit a subscription
abstraction when there is a live-data requirement.

**Why no CSS framework (Tailwind / MUI / etc.)?**
CSS Modules (`*.module.css`) plus a small hand-rolled token layer in
`global.css` for color, spacing, and type. The shell is small, the look is
deliberately distinctive, and the tokens are documented in
[`ui_design.md`](ui_design.md).
A utility framework would add machinery and fight the retro/chunky visual
direction. Decided after the first screen builds landed.

## Public developer boundary

`App.tsx` has no eager player/cloud imports. Developer references load a separate
lazy tree and own their frame/scrolling. `PlayerApp.tsx` retains existing game and
authentication-aware public-player/shared-replay behavior. A fresh developer
visit must not read/create local player state, initialize Firebase Auth, or load
Phaser/Wasm. Navigation out of the player unsubscribes controllers without
clearing singleton stores or cancelling pending promises; returning re-evaluates
normal synchronization. See [Developer area](developer_area.md) for route, source,
media and export contracts.

## The DOM/Phaser boundary

This is the load-bearing architectural decision for the FE rewrite.

**Phaser owns:** the 15x15 grid, stone sprites, pointer, tactical warnings,
forbidden move overlays, result sequence numbers, next-move hover targets, and
winning-line highlighters. Anything that benefits from pixel-level control or
frame-based animation.

**DOM owns:** everything else. Status HUD, player identity/info, rule toggles,
result strips, replay transport/timeline, local record/history views, menus,
modals, auth UI, and any future cloud/online shell surfaces.

**They communicate through a narrow interface:**

```ts
// React → Phaser: one declarative board view model
interface BoardViewModel {
  boardSize: number;
  forbiddenMoves: CellPosition[];
  interaction:
    | PlayInteraction
    | { kind: "readonly" }
    | { kind: "replay" };
  overlays: BoardOverlay[];
  position: {
    cells: CellStone[][];
    currentPlayer: 1 | 2;
    lastMove: CellPosition | null;
    moves: MatchMove[];
    showSequenceNumbers: boolean;
    status: MatchStatus;
  };
}

// Phaser → React: intent events
type BoardEvent =
  | { type: 'hover'; cell: Move }
  | { type: 'place'; cell: Move }
  | { type: 'unhover' };
```

A single `<Board model={...}>` React component wraps the Phaser instance,
passes a `BoardViewModel` in, and subscribes to intent events out. Phaser never
reads from the global store or Firestore directly.

The React wrapper also mirrors the board as an accessible DOM grid. Keyboard
placement uses the same `BoardViewModel` intent callback as pointer placement;
legality remains in the game store and Rust core. Arrow keys explore cells,
Home/End navigate a row, and Enter/Space place a stone on the human turn.

This is a departure from v0.1, where Phaser scenes held game state and
drove the UI. The rewrite flips it: state lives in React/Zustand, Phaser
becomes a stateless view.

## Core sharing

One rules implementation, reused everywhere:

- **Browser:** `gomoku-bot-lab/gomoku-wasm` compiles `gomoku-core` to wasm.
  Web imports it via `gomoku-wasm = "file:../gomoku-bot-lab/gomoku-wasm/pkg"`
  in `package.json`. Vite's wasm plugin handles the load. Structured bridge
  payloads cross the wasm boundary as JSON strings and are parsed in
  `gomoku-web/src/core/wasm_bridge.ts`; React, Phaser, and stores should not
  cast raw wasm objects directly.
- **Replay analysis:** `gomoku-analysis` owns bounded corridor traceback.
  `gomoku-eval` uses it for fixture/report generation; `gomoku-wasm` exposes a
  session-backed browser analyzer bridge; `gomoku-web` converts saved matches
  to core replay JSON and schedules progressive analysis in a cancellable web
  worker before rendering annotations.
- **CLI / eval tools:** already using `gomoku-core` via path deps.

This means "is this move legal?" and "did this player win?" have exactly
one answer, regardless of where the question is asked. Tactical hint facts that
depend on rules semantics also live below the UI layer. The browser keeps a
rolling-frontier-backed `WasmBoard` and reads one typed threat snapshot for
immediate wins, immediate threats, imminent replies, counter-threat replies, and
Renju forbidden moves. The canonical winning line remains a separate result
visualization query. Rule-legality feedback such as Renju forbidden moves stays
always on; tactical assistance categories are filtered by saved profile UI
preferences before they reach the board renderer.

## Data flow

### Offline bot match (works today)

```
user clicks cell
  → React dispatches place-move
    → gomoku-wasm applies move, returns JSON result
      → Zustand store updated
        → <Board> re-renders with new props
        → bot's turn: worker runs the wasm bot, guarded result uses the same loop
```

### Casual / local match

```
local game finishes
  → game store emits one finished match
    → local profile creates compact schema-2 replay
      → retained replay/summary/stat tiers persist in browser storage
        → Profile and Replay can inspect the ending
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

Published casual replays contain client-supplied results and follow a separate
opt-in [sharing contract](../backend/sharing_data.md).

### Deferred online match designs

The [casual multiplayer proposal](../backend/p2p_multiplayer.md) keeps validation
in both browsers and uses the backend for joining and signaling. The
[trusted match sketch](../backend/future_cloud_run.md#trusted-match-sketch)
describes how a future Cloud Run service could reuse `gomoku-core` to validate
moves and own ranked results. Adoption of either design is tracked in the
[roadmap](../product/roadmap.md#deferred-multiplayer).

## Version sequence

The old v0.1-to-v0.2 migration is complete. React now owns the shell, Phaser is
the board renderer, and local play/history/replay are the working product
baseline. The `v0.3` backend-continuity line has since added optional Firebase
Auth, private cloud profile/history, and owner-scoped Firestore rules without
moving casual gameplay out of the browser.

The `v0.4` lab-powered loop and `v0.5` reconciliation are implemented. The
October `0.5.x` review completed deeper correctness, lifecycle, usability,
and maintenance work. The [closeout](../../archive/reconciliation_2026_10/README.md)
records the retained architecture, targeted repairs, and acceptance deferrals.

The published v0.6 releases added self-contained replay links, a private replay
library, and opt-in public profiles and catalogs. v0.6.4 added private titles,
notes and openings for saved replays and revision-safe edits to public listings.
Shared and public routes initialize authentication independently of private
history. Completed continuations use normal game history, promotion and
aggregate totals. An optional saved-match origin preserves the starting ply and
source fingerprint through cloud sync and portable export. Earlier preview
continuations appear in their owner's My games. Private study copies stay in
Saved replays. See the [portable format](portable_replay.md),
[sharing data](../backend/sharing_data.md), and the [v0.6.4 release record](../../archive/v0_6_4_release.md).
The live signed-in public-sharing round trip remains pending. Multiplayer
remains deferred with direct P2P as the preferred gameplay path.

Details are in [`Roadmap`](../product/roadmap.md). The architectural contract
— what React owns, what Phaser owns, how they talk, and where `gomoku-core` is
authoritative — is the part that needs to hold across all of it.
