# Roadmap

The order we're going to build in. Picks sequencing across FE rewrite,
backend bring-up, and features so each phase delivers something playable
— not a stretch of months that only pays off at the end.

Phases aren't time-boxed; this is a personal project. They're sized so
"phase done" means "a working app, just with less in it than the next
phase."

## Phase 0 — Snapshot (done)

Offline single-player web game with a Phaser-only frontend, backed by
the Rust core and bots compiled to wasm. Tagged `v0.1`.

What this proved:
- Rust core + wasm bridge works in a real browser bundle.
- `RandomBot` and `SearchBot` play legal games.
- The build/deploy pipeline (Vite + wasm-pack + GH Actions + Pages)
  is solid enough to iterate on top of.

Shipped state is in `docs/archive/progress_v0.1.md`.

## Phase 1 — FE rewrite (React shell, Phaser board)

The pivot proper. Goal: same offline game, new architecture.

- Bring up React + Vite + React Router + Zustand in `gomoku-web/`.
- Wrap the existing Phaser board in a `<Board>` React component that
  takes props and emits events. Delete the old scenes and the
  scene-driven game loop.
- Rebuild the match screen as DOM: player cards, turn indicator, move
  history, menu. Wire offline bot match end-to-end through the new
  React/Zustand state.
- Establish the palette and layout primitives (`<AppShell>`, `<Card>`,
  `<Button>`, etc.) so later phases have a kit to pull from.
- Rebuild home as a simple "Play bot" entry point.

**Done when:** the deployed Pages site looks and plays like v0.1, but
the code underneath matches `architecture.md`. No regressions in bot
play, win detection, or performance.

**Out of scope:** auth, Firestore, online play, replays, puzzles.
Offline bot matches only.

## Phase 2 — Guest identity + cloud profile

First backend component. Introduces Firebase but nothing else.

- Local guest profile created on first meaningful interaction and
  persisted in browser storage.
- Firebase project + Firestore database bootstrapped (one-time manual;
  commands logged in `infra/README.md`).
- Web SDK wired up for Google/GitHub sign-in when a cloud-backed feature
  is needed.
- Signing in creates or loads `profiles/{uid}` and promotes local guest
  state to a cloud-backed profile.
- Profile screen: guest vs signed-in state, display name, avatar,
  linked providers, sign out.
- Firestore rules for profile-only access, deployed via a committed
  GH Actions workflow (`deploy-rules.yml`).

**Done when:** a stranger can open the site, play a bot match with a
local guest profile, and optionally sign in with Google/GitHub without
feeling like they lost their state.

**Out of scope:** Cloud Run, usernames, match history, anything public.

## Phase 3 — Cloud Run bring-up + username reservation

Stand up the Rust service. Ship one endpoint to prove the path, not
a whole feature suite.

- `gomoku-bot-lab/gomoku-api/` crate created, depends on `gomoku-core`
  via path dep.
- Container builds via Cloud Build, pushed to Artifact Registry.
- WIF pool + `gh-cd` service account configured (one-time manual).
- `deploy-api.yml` workflow deploys the container to Cloud Run.
- `gomoku-api-runtime` SA created with scoped Firestore access.
- One endpoint: `POST /reserve_username` — JWT-authenticated, runs a
  Firestore transaction over `usernames/{handle} → uid`, updates the
  caller's profile.
- Web adds a "set username" flow, gated to just before public features.

**Done when:** a signed-in user can reserve a unique username, the
service is deployed by a workflow, and a reviewer could re-create the
whole setup from `infra/README.md`.

**Out of scope:** match authority, replays, any other endpoint.

## Phase 4 — Replay persistence + shareable links

First feature users will actually notice. Makes the offline bot match
into something worth coming back to.

- Guest matches are stored in local history only.
- Signed-in users get cloud-backed match history in
  `profiles/{uid}/matches/{id}`.
- Public `replays/{id}` are created only when the user explicitly hits
  Share / Publish from a cloud-saved match.
- Replay screen in the web app: timeline scrubber, playback controls,
  same board code in a read-only mode.
- "Share" button on match result → copies a public URL. Loading the URL
  while signed out still renders the replay (public read on
  `replays/{id}`).
- Home screen gains the "in progress" and "recently finished" sections.

**Done when:** a guest can finish a game and see it locally; a signed-in
player can see their cloud history; and a signed-in player can explicitly
publish one replay and share its URL.

**Out of scope:** critical-move tagging, analysis, online play.

## Phase 5 — Online match (human vs human)

The biggest product leap. Two real people playing on the same board.

- `matches/{id}` schema + rules (participants read, server writes
  authoritative state).
- Cloud Run gains match-authority endpoints: `POST /match` (create),
  `POST /match/{id}/move` (apply), with server validating every move
  via `gomoku-core`.
- Web adds the online lobby (`/online`) as a direct-challenge surface.
  Starts with challenge-by-link; matchmaking queue comes after.
- Live match screen subscribes to `matches/{id}` via `onSnapshot`.
  Firestore is the fanout layer; Cloud Run is the authoritative writer.
- Abandonment handling: inactivity timeout, resign flow.

**Done when:** two people can open two tabs (or two devices), find
each other via a challenge link, and play a full game with the server
recording moves and result.

**Out of scope:** matchmaking by skill, notifications, untrusted
client-authored online matches, spectating.

## Phase 6 — Lab-powered analysis (critical moves, save-this-game)

Deliver on the lab-as-feature-source pillar.

- Post-match job (Cloud Run): re-run each move through `SearchBot` at
  server depth, write an `analysis` subdoc with per-move evaluation
  deltas and suggested best moves.
- Replay viewer gains: evaluation curve on the timeline, "critical
  moment" markers, analysis panel showing top alternatives.
- "Try from here" button: branch into a live bot match from any
  position in a replay. Plays against a bot calibrated to the turning
  point's difficulty.

**Done when:** an average-skill player can open any of their replays
and learn something — "you had this move, you missed this threat" —
without needing to understand evaluation numbers.

**Out of scope:** puzzle generation.

## Phase 7 — Puzzles

Second lab-driven feature. Reuses the analysis pipeline.

- Puzzle generator (Cloud Run job) scans `replays/{id}` for positions
  with forced wins or forced blocks at depth 5+, verifies with deeper
  search, tags by theme (open four, double threat, VCF, etc.).
- Publishes to `puzzles/{id}`.
- Web: `/puzzles` list and solver. Per-user progress in
  `puzzle_attempts/{uid}/{id}`.
- Daily puzzle surface on home screen.

**Done when:** a player can open the site and solve today's puzzle,
with per-puzzle and streak stats persisting.

**Out of scope:** matchmaking improvements, rankings.

## Phase 8+ — Opportunistic

Everything after puzzles is "pick by what's most fun to build":

- **Matchmaking by rating** — queue-based pairing, basic Elo.
- **Leaderboards** — public, scoped to bot-tier results initially, then
  verified human-vs-human.
- **Stronger bot endpoint** — `POST /bot/move` at higher depth for a
  "champion" preset.
- **Matchmaking extras** — rated queues, seasonal resets.
- **Cloud-synced settings** — theme, sound, preferred bot preset.
- **Spectating** — watch live high-rated games.
- **Friends** — accept/block, direct challenge UX beyond link-sharing.

No commitment on order or on shipping all of these. The framing of
"product" says we're building what's fun and coherent, not maximizing
feature count.

## Non-goals along the way

Called out explicitly so they don't sneak in:

- **Native mobile apps.** Mobile web only.
- **Real-time voice/video/chat.** Not in scope.
- **SSR / server-rendered pages.** Vite SPA stays static on Pages until
  there's a reason to change.
- **Micro-transactions, ads, account paywalls.** None, ever.

## Tracking

Progress on the current phase lives inline in PR descriptions, not in
a separate progress doc. When a phase completes, write one paragraph
at the bottom of this file (or in a tagged release note) covering what
shipped and what drifted. Avoid the "status log" failure mode — the
repo tells you what's built; the roadmap tells you where it's going.
