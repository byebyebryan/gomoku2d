# Backend

Scope: the services behind the web app — identity, persistence, live matches,
lab-powered features. The browser still owns the cheapest, lowest-trust flows;
backend services exist for durability, sharing, and trust boundaries.

## Goals

- Cloud-backed identity and game state so the game works across devices and
  between two humans.
- Reuse `gomoku-core` as the single rules engine — browser via wasm, server
  via native Rust.
- Stay within GCP Always Free tiers; this is a personal project, not a
  service with real traffic.
- Everything reproducible from the repo: infra changes are committed
  workflows or documented snippets, never console-only clicks.

## Non-goals

- Enterprise-grade anti-cheat. We'll verify replays; we won't fingerprint
  browsers or detect bot-assisted play.
- Real-time sub-100ms guarantees. Turn-based play tolerates 200-500ms
  latency; Firestore listeners are fine.
- Paid tiers, SLAs, or anything that risks billing surprises.

## Version Mapping

`docs/roadmap.md` owns sequencing. This backend doc describes the target
service model and the feature menu, but not every piece lands at once.

| Version | Backend intent | Included | Deferred |
|---|---|---|---|
| `P3 / v0.3` | Backend foundation and cloud continuity | Firebase Auth, cloud profile, local profile promotion, private cloud history, owner-scoped Firestore rules | live PvP, ranked/trusted matches, public replay sharing, replay analysis, puzzles |
| `P4 / v0.4` | Lab-powered product identity | replay analysis, critical moments, puzzles, save-this-game positions, bot personalities/customization; Cloud Run only if browser-side wasm is not enough | live PvP, ranked/trusted matches, broad public sharing |
| `P5 / v0.5` | Presentation systems and skins | theme/skin support and product polish; backend usually unchanged | live PvP, ranked/trusted matches, broad public sharing |
| `P6 / v0.6` | Online product expansion | Cloud Run match authority, direct challenge/PvP, trusted match history, matchmaking/ranked if useful, explicit public shareables | broad social features |

Cloud Run is part of the target backend, but `v0.3` does not need it. `v0.4`
may use it for heavier lab-powered analysis if browser-side wasm is not enough;
otherwise it can wait until the online/trusted-match phase.

## Implementation References

This file is the backend design contract. Operational state and setup commands
live elsewhere so the design doc does not become a deployment log:

- `backend_infra.md` — live Firebase/GCP setup, locations, app IDs, enabled
  APIs, Firestore rules deployment, and pending infra checklist.
- `backend_cost.md` — free-tier assumptions, rough usage estimates, and
  headroom checks.

## Architecture

Four components:

| Component | Role | Cost note |
|---|---|---|
| **Local profile** | Local identity, local settings, local match history | No backend cost |
| **Firebase Auth** | Sign-in when cloud-backed features are needed | Auth is initialized as Identity Platform; keep providers inside the no-cost social-sign-in tier tracked in `backend_cost.md` |
| **Firestore** | Document storage: cloud profiles, trusted matches, published replays, puzzles | Current database state lives in `backend_infra.md`; cost posture lives in `backend_cost.md` |
| **Cloud Run** | Rust service: trusted match authority, username reservation, verification, strong bot, puzzle generation | For request-based billing: 2M requests · 180k vCPU-s · 360k GiB-s per month free (us-central1-based) |

Everything scales to zero when idle. The browser is the fast path when
trust doesn't matter; Cloud Run is the path when it does.

### Where code lives

- `gomoku-bot-lab/gomoku-core` — rules engine, shared by browser (wasm) and
  server (native).
- `gomoku-bot-lab/gomoku-api/` — the Cloud Run service. Starts here as a
  new crate in the existing workspace so it can depend on core/bot via
  path deps. Graduates to top-level `gomoku-backend/` when deploy cadence
  diverges enough to justify the split.
- `firestore.rules` at the repo root — security rules, deployable and
  reviewed alongside code.
- `gomoku-web/src/cloud/firebase.ts` — browser Firebase initialization. It
  stays inert until all required `VITE_FIREBASE_*` env vars are present.

## Auth

Default path is **not** "anonymous auth on page load."

Instead:

- first meaningful interaction creates a **local profile** in browser
  storage
- **Google** is the first cloud sign-in provider; GitHub can follow if it stays
  small enough to justify the extra provider setup
- sign-in happens when the user opts into a cloud-backed feature such as synced
  history, online play, or replay sharing

This avoids creating backend identities for drive-by visitors while still
letting players get a feel for the game immediately.

The web uses the Firebase JS SDK directly (see `architecture.md`). Cloud Run
verifies callers by validating the Firebase ID token JWT against Google's
public keys.

### Identity model

Three layers:

1. **Local profile** — who are you on this device before sign-in?
   Browser-managed only.
2. **Firebase user** — who are you in cloud auth? Managed by Auth after sign-in.
3. **Gomoku cloud profile** — who are you *in this game* once cloud-backed?
   Stored in Firestore at `profiles/{uid}`.

Provider `displayName` / `photoURL` seed the cloud profile but don't own it.
The profile doc is the canonical app identity once a user signs in:

```ts
type Profile = {
  uid: string;
  username: string | null;        // unique, app-owned, reserved via Cloud Run
  display_name: string;           // seeded from provider, editable, not unique
  auth: {
    providers: Array<{
      provider: "google.com" | "github.com";
      display_name: string | null;
      avatar_url: string | null;
    }>;
  };
  settings: {
    board_hints: {
      immediate: "off" | "win" | "win_threat";
      imminent: "off" | "threat" | "threat_counter";
    };
    bot_config: BotConfigDocumentV1;
    game_config: {
      ruleset: "freestyle" | "renju";
      opening: "standard";
    };
    touch_control: "pointer" | "touchpad";
  };
  reset_at: Timestamp | null;
  match_history: {
    replay_matches: SavedMatchV2[];
    summary_matches: CloudMatchSummaryV1[];
    archived_stats: CloudArchivedMatchStatsV1;
  };
  created_at: Timestamp;
  updated_at: Timestamp;
};
```

`username` starts `null` and is only required when entering a public
surface (leaderboard, shareable profile URL). Reservation goes through
Cloud Run so the `usernames/{handle} → uid` transaction is atomic.

### Promotion flow

When a local player decides to sign in:

1. The app signs them in with Google or GitHub.
2. Firebase returns a stable `uid`.
3. The app creates or loads `profiles/{uid}`.
4. Local settings and finished local match history are imported into
   the cloud profile/history.
5. Imported local records keep a stable local-origin ID so the import is
   idempotent if the flow is retried.

Local state is disposable. Cloud state is durable.

## Data model

Canonical Firestore collections, document schemas, schema versions, and
field-level invariants live in [data_model.md](data_model.md).

For `v0.3`, the critical path is:

- `profiles/{uid}` for owner-scoped cloud identity and settings
- `profiles/{uid}.match_history.replay_matches` for capped private casual
  replay payloads
- `profiles/{uid}.match_history.summary_matches` plus
  `match_history.archived_stats` for longer private history stats without
  retaining every move list

Important distinction:

- **Local history** lives locally only.
- **Signed-in casual history** is saved as a capped private profile snapshot.
- **Trusted online/ranked history** later uses server-written match records with
  trust metadata.
- **Public shared replays** are a separate publish step; they are not created for
  every finished match by default.

Starter rules for the backend-foundation phase live in `firestore.rules`. The
current rules intentionally keep the first public backend slice narrow:

- owners can read their own `profiles/{uid}` document
- owners can create/update that document only if it matches the expected profile
  schema
- owners can delete their own `profiles/{uid}` document through Delete Cloud;
  local browser history is not deleted by that cloud delete path
- client updates preserve locked app-owned fields such as `created_at` and
  `username`; `reset_at` can only stay unchanged or advance to the
  current write time; `display_name` can be promoted from a user-chosen local
  profile
- `match_history.replay_matches` is capped at 128 full replay records;
  `match_history.summary_matches` is capped at 1024 lightweight summary records
  and rolls older stats into `match_history.archived_stats`
- normal profile snapshot writes are throttled by a 5-minute cooldown; reset
  writes can bypass that cooldown only when they clear history and advance the
  reset barrier inside the constrained reset shape
- private match subcollections are closed for the current casual path and kept
  reserved for future server-verified/shareable records

## Cloud Run service

One binary, one container, one service to start. `axum` for HTTP, one
endpoint per concern.

### What it does

- **Username reservation.** Transaction over `usernames/{handle} → uid`.
  First thing the client can't do safely on its own.
- **Trusted match authority.** Live cloud-backed match creation and move
  application. Server validates every move against `gomoku-core`, writes the
  authoritative result, and is the only writer for trusted match state.
- **Private history upgrade path.** Optionally verify or enrich client-uploaded
  casual matches after the fact, without making the live local match depend on
  backend validation.
- **Replay verification.** `POST /verify` re-runs a replay through
  `gomoku-core`, stamps `verified: true` if it's a legal game with the
  claimed result.
- **Puzzle generation.** Offline job that scans eligible trusted saved-match
  history for forced-win branches, verifies with search, publishes to
  `puzzles/`.
- **Replay analysis.** Post-match or on-demand, runs each move through the
  strong bot to produce an evaluation curve — feeds the replay viewer's
  critical-move tags.
- **Strong bot endpoint.** `POST /bot/move` — FEN + difficulty in, move
  out. Depth the browser can't afford.

### Trust lanes

There are two distinct lanes:

1. **Casual / free play**
   - browser-authoritative
   - local-first
   - no per-move backend validation in the hot path
   - local-only sessions stay local only
   - signed-in casual matches can still sync privately to the cloud profile's
     embedded `match_history` snapshot as `client_uploaded`

2. **Trusted / cloud-backed play**
   - backend validates every move
   - used for ranked matches, server-owned online history, and any replay we
     intend to trust or share publicly as `server_verified`

This split is deliberate. It keeps the cheapest path fast while making the
durable/public path trustworthy.

Analysis rule:

- `client_uploaded` saved matches can still receive **private analysis** for the
  owner.
- Only `server_verified` matches feed public/ranked surfaces such as leaderboards
  and puzzle mining.

### How it talks to Firestore

Rust gRPC client via ADC. The attached runtime service account has
`roles/datastore.user`; it bypasses security rules by virtue of its IAM
role, not because of an "admin SDK" (there is no first-party Firebase
Admin SDK for Rust, which is why this matters to write down).

### Shape of a live match

```
client A                      server                       client B
   │                             │                            │
   │ POST /match (vs B)          │                            │
   ├────────────────────────────►│                            │
   │                             │ write matches/{id}         │
   │                             │   (status=active, ...)     │
   │◄────────────────────────────┤                            │
   │ {match_id}                  │                            │
   │                             │◄───────────────────────────┤
   │                             │    onSnapshot subscribe    │
   │  onSnapshot subscribe       │                            │
   │◄────────────────────────────┤                            │
   │                             │                            │
   │ POST /match/{id}/move       │                            │
   ├────────────────────────────►│                            │
   │                             │ validate with gomoku-core  │
   │                             │ update matches/{id}        │
   │◄────────────────────────────┤───────────────────────────►│
   │  (both see new move via Firestore push)                  │
```

Clients do not directly write authoritative moves into `matches/{id}`.
Firestore is the fanout layer; Cloud Run is the writer for trusted match state.

## CI/CD

Two credential stories, kept separate:

- **Deploy credentials** — GitHub Actions → GCP. **Workload Identity
  Federation**, no long-lived JSON keys. Short-lived OIDC token exchanged
  per workflow run.
- **Runtime credentials** — Cloud Run → Firestore/other GCP services.
  **Application Default Credentials** via the attached service account.
  No secret to store.

Current and planned service accounts:

| SA | Used by | Can | Cannot |
|---|---|---|---|
| `github-firestore-deploy` | GH Actions Firestore rules deploy | Publish Firestore rules through Firebase Rules API | Read user data, read runtime secrets, deploy app/runtime infra |
| `gh-cloud-run-deploy` | Future GH Actions Cloud Run deploy | Build/push images, deploy Cloud Run, impersonate runtime SA | Read user data, read runtime secrets |
| `gomoku-api-runtime` | Future Cloud Run runtime | Firestore access (scoped), read pinned Secret Manager versions if needed | Deploy anything, modify infra |

Secrets from non-GCP sources (third-party API keys, signing keys) go in
Secret Manager with **pinned versions** (`:3`, not `:latest`). Rotations
are explicit, deploy-gated, reviewable in PRs. None of the initial phases
require one — the pattern is documented so it's ready when we do.

Workflows live at:

- `.github/workflows/deploy.yml` — web (GitHub Pages), already exists
- `.github/workflows/deploy-firestore.yml` — Firestore rules, tag/manual only
- `.github/workflows/deploy-api.yml` — future Cloud Run service

The live Firebase/GCP setup, Workload Identity Federation bootstrap, deploy
model, and break-glass rules deploy path are documented in
`docs/backend_infra.md`.

## Feature catalog

Each is a standalone increment. `roadmap.md` sequences them; this is the
menu.

| Feature | Surface | Trust gate |
|---|---|---|
| Local profile | Browser-only local mode | None |
| Auth + cloud profile | Google/GitHub sign-in, profile sync | Firebase Auth |
| Username reservation | `/reserve_username` | Cloud Run transaction |
| Cloud match history | Coalesce finished signed-in casual matches into `profiles/{uid}.match_history` | Private; `client_uploaded` |
| Trusted match history | Server-validated online/ranked history in `profiles/{uid}/matches/{id}` | `server_verified` |
| Replay sharing | Public URL via `replays/{id}` projected from a saved private match | Explicit publish step |
| Online match (human vs human) | `matches/{id}` + Cloud Run authority | Server validates every move |
| Strong bot endpoint | `/bot/move` at higher depth | Optional auth + rate limit |
| Replay analysis | Private replay evaluation curve, critical moves | Private for all saved matches; public trust only for `server_verified` |
| Puzzle generation | Offline job → `puzzles/` | Derived from `server_verified` history and curated seed positions |
| Puzzle play + progress | Per-user attempts, streaks | Owner-scoped |
| Leaderboard | Verified results only | Server aggregates verified replays |
| Cloud-synced settings | In the profile doc | Owner |

## Open questions

- **Trusted match state storage.** Server-in-memory with Firestore as the
  durability layer vs. Firestore-as-state-of-record with Cloud Run as the only
  writer. The first is cleaner for reconnection and clocks; the second is
  simpler to operate. The trust boundary is settled; the storage shape is not.
- **Leaderboard writes.** Must go through Cloud Run (direct client writes
  would let anyone post any score). Open: do we write on every match
  result, or aggregate daily in a batch job? Depends on traffic shape.
- **Analysis scheduling.** Eagerly analyze every saved match on write, or queue
  analysis on first replay open? Product wants fast feedback; cost may prefer
  demand-driven analysis.
- **Replay retention.** Keep all published replays forever, or prune old
  unviewed ones after N days? Defer until storage actually costs something.
- **Backend crate location.** `gomoku-bot-lab/gomoku-api/` vs.
  top-level `gomoku-backend/`. Starts in-lab for the workspace benefits
  (one `cargo build`, shared `target/`), splits out when the server
  grows deploy surface that bot-lab shouldn't carry.
