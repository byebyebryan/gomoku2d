# Design

Scope: information architecture, screen-level flows, visual language, and
the component families we'll build to. Paired with `architecture.md` (which
decides the runtime) — this one decides what the player sees.

Unless explicitly called out otherwise, this document describes the **target
product shape**, not the exact surface area of the next milestone. Early phases
ship a subset; `roadmap.md` is the source of truth for rollout order.

## Information architecture

Six top-level destinations in the target product:

| Route | Purpose |
|---|---|
| `/` | Home — play now, resume in-progress games, recent activity |
| `/match/:id` | Live match (vs. bot or human) |
| `/online` | Lobby — direct challenges first; matchmaking and spectate later |
| `/replays` · `/replays/:id` | Replay library and viewer |
| `/puzzles` · `/puzzles/:id` | Puzzle list and solver |
| `/profile` | Account, stats, settings |

Guests land on `/` with no cloud account yet. On the first meaningful
interaction, the app creates a **local guest profile** on the device so bot
play and local history work immediately. Signing in with Google or GitHub later
promotes that local state to a cloud-backed profile.

Not every route needs to work for guests. The design assumption is:

- offline/casual play works as a guest
- cloud-backed features gate sign-in at the point they become relevant

## Screen inventory

### Home (`/`)

Three sections, stacked:

1. **Play now** — two big buttons: "Play bot" (opens difficulty picker) and
   "Play online" (routes to `/online`). Always visible.
2. **In progress** — cards for any unfinished games. Tapping resumes the
   match. Empty state explains the section if there's nothing yet.
3. **Recently finished** — last 3-5 games with result, opponent, date.
   Tapping opens the replay. "See all" routes to `/replays`.

No feed, no social graph. Home is an action hub, not a timeline.

### Match (`/match/:id`)

The board is the center of gravity; everything else is a frame.

```
┌──────────────────────────────────────────────┐
│ [opponent card]              [resign] [menu] │
│                                              │
│             ┌────────────────┐               │
│             │                │               │
│             │     BOARD      │               │
│             │   (Phaser)     │               │
│             │                │               │
│             └────────────────┘               │
│                                              │
│ [you card]          [move history / clock]   │
└──────────────────────────────────────────────┘
```

- **Player cards** (top/bottom) — name, avatar, color stone, clock if timed,
  turn indicator (subtle glow on the player to move).
- **Move history** — compact list, scrollable. Click a move to peek at the
  position (ghost preview on the board).
- **Menu** — offer draw, request undo (bot matches only, with caveat — see
  below), settings, quit.

**On undo:** in bot matches it's available but rate-limited and visibly
costs rating (or is off entirely on higher difficulty). In human matches
it requires opponent consent. The button exists because it's expected in
casual play; the design discourages it from becoming the default.

### Online lobby (`/online`)

Target state has three tabs:

1. **Quick play** — "find me a match" with optional rating band filter.
   Shows a queue timer while matching.
2. **Friends** — list of accepted friends, presence indicator, "challenge"
   button. Challenges are links, not push notifications.
3. **Spectate** — high-rated or recently-featured games.

Friends list exists for direct challenges, not socializing. No feed, no DMs.

Phase rollout note:

- First shipped slice is **direct challenge by link**
- Quick play / rating-band matching comes later
- Friends and spectate only exist if later phases justify them

### Replay viewer (`/replays/:id`)

The board reused, with a timeline scrubber underneath:

- **Timeline** — horizontal bar, one tick per move, with heatmap colors for
  evaluation swing (lab-tagged critical moves stand out).
- **Playback controls** — prev / next / play-pause / jump to critical move.
- **Analysis panel** (right side, collapsible) — for the selected move:
  what was played, what the bot's top suggestion was, score delta. Comes
  from the lab running the position post-match.
- **"Try from here"** — branch into a live match against a bot from this
  position. Drives "can you save this game?" mode.

### Puzzles (`/puzzles`)

A puzzle is a position + "find the win in N" or "defend against the threat."

- **List view** — cards with difficulty, theme (e.g. "open four," "double
  threat"), attempts/solved state.
- **Solver view** — board + prompt. You make moves, the puzzle-bot responds
  with its best reply. Solved when the puzzle's win condition triggers.
  Failed when you play a losing move or run out of tries.

Puzzles come from the lab: it scans real games for positions with forced
outcomes, verifies a human could reasonably find the win, and tags them by
theme.

### Profile (`/profile`)

- **Identity** — local guest state or signed-in profile, avatar, display name,
  sign in / link to Google or GitHub, sign out.
- **Stats** — games played, win rate by color, rating graph over time,
  bot difficulty distribution.
- **Settings** — board theme, sound, notifications (if any), accessibility
  toggles.

Stats are lightweight summaries, not a dashboard. If someone wants depth,
they go to replays.

## Visual language

### Style pillar: hybrid pixel + modern shell

The board stays pixel-art. The app shell is clean modern UI. They share a
palette and typographic scale so it doesn't feel like two games.

**Board (Phaser):**
- Pixel-art wood grain, crisp nearest-neighbor scaling.
- Black/white stone sprites carried over from v0.1.
- Hover preview as a translucent pixel stone.
- Win line as an animated pixel-art highlight (not a CSS overlay).

**Shell (DOM):**
- Clean, generous whitespace, modern type (system font stack or a single
  sans like Inter).
- Buttons and cards with soft borders and subtle shadows — not skeuomorphic,
  not flat-to-the-point-of-ambiguous.
- Subtle pixel-accent touches at the seams: section dividers with a pixel
  texture, icons in 16×16 pixel style, corner treatments that hint at the
  board's aesthetic.

### Palette

Anchored to the wood-board tones so the board and shell share DNA.

| Token | Role | Approximate hex |
|---|---|---|
| `--board-wood` | Board background | `#d9a063` |
| `--stone-black` | Black stone | `#1a1a1a` |
| `--stone-white` | White stone | `#f5f0e6` |
| `--ink` | Body text | `#2b2b2b` |
| `--ink-muted` | Secondary text | `#6b6b6b` |
| `--paper` | Shell background | `#fafaf7` |
| `--paper-elevated` | Cards, modals | `#ffffff` |
| `--accent` | CTAs, active states | TBD (warm, not competing with board) |
| `--danger` | Resign, destructive | `#b94a3e` |
| `--success` | Wins, confirmations | `#4a7c59` |

Dark mode: defer. Single-theme until the light theme is tight.

### Typography

- One sans family, three sizes (display / body / caption).
- Numeric tabular for clocks and move numbers (so digits don't dance).
- Pixel font used *only* inside the Phaser scene (win banner, move number
  on the board) to reinforce the boundary.

### Motion

Restrained. Moves land with a short settle animation; the win line sweeps
once; route transitions fade. No parallax, no decorative loops. Motion
should direct attention, not decorate.

## Component families

To build incrementally without re-designing each screen from scratch:

### Layout primitives
- `<AppShell>` — top bar + outlet; applies global padding and max-width.
- `<Stack>` / `<Row>` — vertical / horizontal flex primitives with gap
  tokens.
- `<Card>` — padded container with subtle border and optional elevation.

### Identity and state
- `<Avatar>` — initials fallback, optional image, stone-color ring for
  turn indicator.
- `<PlayerCard>` — avatar + name + clock + role indicator (used on match
  and replay screens).
- `<StatusBadge>` — turn / thinking / won / draw / resigned pills.

### Game surfaces
- `<Board>` — the React wrapper around Phaser (see `architecture.md`).
- `<MoveHistory>` — scrollable list with hover-to-preview.
- `<Clock>` — tabular numerals, low-time warning state.
- `<Timeline>` — horizontal scrubber for replays and puzzle progress.

### Controls
- `<Button>` — primary / secondary / danger variants.
- `<IconButton>` — 16×16 pixel icons, for top-bar and menu actions.
- `<Modal>` — for confirmations (resign, leave match).
- `<DifficultyPicker>` — bot difficulty selector with brief descriptions.

### Feedback
- `<Toast>` — transient confirmations and non-blocking errors.
- `<EmptyState>` — illustrated (pixel-style) empty lists.

## What we're explicitly not designing

- Chat, DMs, or threaded discussions.
- A feed of public games or a "trending" surface.
- Rating ladders, tournament brackets, or seasonal ranks.
- Push notifications (the web is tab-present or it isn't).
- Micro-transactions, cosmetic stores, or ad surfaces.

These aren't forbidden forever — they're not on the table for this pivot.
Keeping the surface small keeps the design coherent.
