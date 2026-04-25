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
| `P3 / v0.3` | Backend foundation and cloud continuity | Firebase Auth, cloud profile, guest promotion, private cloud history, owner-scoped Firestore rules | live PvP, ranked/trusted matches, public replay sharing, replay analysis, puzzles |
| `P4 / v0.4` | Online product expansion | Cloud Run match authority, direct challenge/PvP, trusted match history, matchmaking/ranked if useful, explicit public shareables | lab-powered analysis/puzzles unless they are needed to support online surfaces |
| `P5 / v0.5` | Lab-powered features | replay analysis, critical moments, puzzles, save-this-game positions, stronger bot endpoints if needed | broad social features |

Cloud Run is part of the target backend, but `v0.3` does not need it unless an
early feature, such as username reservation, proves it cannot be handled safely
with Auth + Firestore rules alone.

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
| **Local guest profile** | Local identity, guest settings, guest match history | No backend cost |
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

- first meaningful interaction creates a **local guest profile** in browser
  storage
- **Google** is the first cloud sign-in provider; GitHub can follow if it stays
  small enough to justify the extra provider setup
- sign-in happens when the user opts into a cloud-backed feature such as synced
  history, online play, or replay sharing

This avoids creating backend identities for drive-by visitors while still
letting guests get a feel for the game immediately.

The web uses the Firebase JS SDK directly (see `architecture.md`). Cloud Run
verifies callers by validating the Firebase ID token JWT against Google's
public keys.

### Identity model

Three layers:

1. **Local guest profile** — who are you on this device before sign-in?
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
  avatar_url: string | null;
  auth_providers: string[];       // ["google.com"], ["google.com", "github.com"]
  created_at: Timestamp;
  updated_at: Timestamp;
  last_login_at: Timestamp;
};
```

`username` starts `null` and is only required when entering a public
surface (leaderboard, shareable profile URL). Reservation goes through
Cloud Run so the `usernames/{handle} → uid` transaction is atomic.

### Promotion flow

When a guest decides to sign in:

1. The app signs them in with Google or GitHub.
2. Firebase returns a stable `uid`.
3. The app creates or loads `profiles/{uid}`.
4. Local guest settings and finished guest match history are imported once into
   the cloud profile/history.
5. Imported local records keep a stable local-origin ID so the import is
   idempotent if the flow is retried.

Local guest state is disposable. Cloud state is durable.

## Firestore data model

Starting shape — widens as features land. Each collection maps to a security
rule; new features touch both in the same commit. For `v0.3`, the critical path
is `profiles/{uid}` plus private `profiles/{uid}/matches/{id}` history.

| Collection | Ownership | Used by |
|---|---|---|
| `profiles/{uid}` | owner read/write, self-only | Cloud identity, settings |
| `profiles/{uid}/matches/{id}` | owner read, owner create for casual saves, server writes trusted records / upgrades trust metadata | Cloud match history |
| `matches/{id}` | participants read, server writes authoritative state | Trusted live matches |
| `replays/{id}` | public read, owner/server create on explicit publish | Shared replays only |
| `puzzles/{id}` | public read, server write | Puzzle library |
| `puzzle_attempts/{uid}/{id}` | owner read/write | Per-user puzzle progress |
| `usernames/{handle}` | public read, server-only write | Username uniqueness |

Important distinction:

- **Guest history** lives locally only.
- **Signed-in casual history** is auto-saved privately to `profiles/{uid}/matches/{id}`
  at match end.
- **Trusted online/ranked history** also lands in `profiles/{uid}/matches/{id}`,
  but with server-written trust metadata.
- **Public shared replays** are a separate publish step; they are not created
  for every finished match by default.

A private match-history record is the canonical saved-match object. Published
replays are projections of those records, not the primary source of truth.

Suggested trust field on saved matches:

```ts
type MatchTrust = "client_uploaded" | "server_verified";
```

Suggested private match shape:

```ts
type SavedMatch = {
  id: string;
  owner_uid: string;
  mode: "bot" | "local_pvp" | "online";
  rules: RuleConfig;
  players: { black: string; white: string };
  result: ReplayResult;
  replay: Replay;
  trust: MatchTrust;
  saved_at: Timestamp;
  local_origin_id?: string;      // for one-time guest import dedupe
  analysis_status?: "none" | "queued" | "ready" | "failed";
  published_replay_id?: string | null;
};
```

Suggested published replay shape:

```ts
type PublishedReplay = {
  id: string;
  owner_uid: string;
  source_match_id: string;
  replay: Replay;
  trust: MatchTrust;
  published_at: Timestamp;
};
```

Starter rules for profile-only phase live in `firestore.rules`:

```
rules_version = '2';

service cloud.firestore {
  match /databases/{database}/documents {
    function signedIn() {
      return request.auth != null;
    }

    function isOwner(uid) {
      return signedIn() && request.auth.uid == uid;
    }

    match /profiles/{uid} {
      allow read: if isOwner(uid);
      allow create, update: if isOwner(uid)
        && request.resource.data.uid == uid;
      allow delete: if false;

      match /matches/{matchId} {
        allow read: if isOwner(uid);
        allow create: if isOwner(uid)
          && request.resource.data.owner_uid == uid;
        allow update, delete: if false;
      }
    }
  }
}
```

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
   - guest sessions stay local only
   - signed-in casual matches can still auto-save privately to cloud at match
     end as `client_uploaded`

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

Two service accounts:

| SA | Used by | Can | Cannot |
|---|---|---|---|
| `gh-cd` | GH Actions during deploys | Build/push images, deploy Cloud Run, publish Firestore rules, impersonate runtime SA | Read user data, read runtime secrets |
| `gomoku-api-runtime` | Cloud Run at runtime | Firestore access (scoped), read pinned Secret Manager versions if needed | Deploy anything, modify infra |

Secrets from non-GCP sources (third-party API keys, signing keys) go in
Secret Manager with **pinned versions** (`:3`, not `:latest`). Rotations
are explicit, deploy-gated, reviewable in PRs. None of the initial phases
require one — the pattern is documented so it's ready when we do.

Workflows live at:

- `.github/workflows/deploy.yml` — web (GitHub Pages), already exists
- `.github/workflows/deploy-api.yml` — Cloud Run service
- `.github/workflows/deploy-rules.yml` — Firestore rules

The bootstrap commands (project creation, WIF pool, service accounts,
Artifact Registry) are a one-time manual run documented in `infra/README.md`
(to be added when the first of these bootstraps actually happens).

## Feature catalog

Each is a standalone increment. `roadmap.md` sequences them; this is the
menu.

| Feature | Surface | Trust gate |
|---|---|---|
| Local guest profile | Browser-only guest mode | None |
| Auth + cloud profile | Google/GitHub sign-in, profile sync | Firebase Auth |
| Username reservation | `/reserve_username` | Cloud Run transaction |
| Cloud match history | Save finished signed-in casual match to `profiles/{uid}/matches/{id}` at match end | Private; `client_uploaded` |
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
