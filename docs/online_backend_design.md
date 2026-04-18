# Online / Backend Design

**Last updated:** 2026-04-18
**Status:** Draft. Phases 1–3 scheduled (auth/profile, replay sharing, match history); 4–5 stand up Cloud Run and the verifier; feature catalog beyond that is scoped but unscheduled.

---

## Goals

- Add cloud-backed identity and player data to `gomoku2d` without changing the current frontend-first game model.
- Keep the browser game deployable to GitHub Pages.
- Reuse the existing Rust core (`gomoku-core`, `gomoku-bot`) wherever possible — same rules engine in browser (via Wasm) and on the server (via Cloud Run).
- Stay within GCP Always Free tiers; this is primarily an experiment in setup and integration, not a product with real traffic.
- Add backend pieces only where they clearly improve UX, enable a new feature, or tighten a trust boundary.

## Non-goals (initial phase)

- Server-authoritative real-time gameplay
- Websocket infrastructure (polling or Firestore snapshot listeners are enough)
- Hardcore anti-cheat guarantees
- Full matchmaking / skill-based pairing
- Paid tiers or anything that risks billing

---

## Operating principles

This whole backend exists as an excuse to practice the workflows, not to ship
a product. That shapes a few non-negotiables:

- **Everything reproducible from the repo.** Any infra change — creating a
  service account, granting a role, wiring a pool, deploying code — is either
  a committed GH Actions workflow or a committed, runnable snippet (shell
  script or copy-pasteable block in a doc). If a fresh clone on a blank GCP
  project can't end up at "working deployed service" by following the repo,
  the setup isn't done.
- **One-time manual `gcloud` is fine, but documented.** Bootstrap steps that
  only need to run once (creating the WIF pool, the deploy service account,
  the Artifact Registry repo) can be imperative — but the exact commands
  live in this doc or a committed script, with placeholders called out
  explicitly. No "you just have to know to do X in the console."
- **Recurring work goes through CD.** Anything runnable more than once — image
  builds, Cloud Run deploys, Firestore rules publishes, secret rotations that
  require a version bump — goes through a GH Actions workflow, not a laptop
  `gcloud` invocation. Workflows can be `workflow_dispatch`-only for now;
  the point is the steps are committed, not that they run on every push.
- **No secrets live in the repo, and no long-lived secrets live in GitHub.**
  Deploy auth is via Workload Identity Federation (short-lived OIDC tokens).
  Runtime access to GCP services uses Application Default Credentials from the
  attached service account — no secret to store. If a non-GCP secret ever
  shows up (third-party API key, signing key), it goes in Secret Manager with
  a pinned version, not an env var or repo file. Anything that looks like a
  key should raise an eyebrow in review.
- **Write down the trade-offs, not just the choices.** If a decision could
  have gone another way (WIF vs. JSON key, Firestore rules vs. Cloud Run
  arbitration, anonymous-first vs. OAuth-first), both sides and the reason
  stay in the doc. This is meant to be a reference for the next similar
  project as much as for this one.

One-time bootstrap snippets in this doc should eventually move to a committed
`infra/` directory (e.g. `infra/bootstrap.sh`, `infra/README.md`) when there's
enough of them to justify the split. Until then they stay inline here.

---

## Architecture

Three first-class backend components:

| Component | Role | GCP Always Free |
|-----------|------|-----------------|
| **Firebase Auth** | Identity, session state, provider linking | Unlimited for Google / GitHub / Anonymous |
| **Firestore** | Document storage: profiles, replays, match history, leaderboards | 1 GiB storage, 50k reads/day, 20k writes/day, 20k deletes/day |
| **Cloud Run** | Server logic in Rust — anything that needs trust or the native core | 2M req/month, 360k vCPU-seconds, 180k GiB-seconds memory, 1 GiB egress; scales to zero when idle |

Optional later, only if a feature justifies it:

- **Cloud Storage (GCS)** — 5 GB free; replay archives or custom avatars
- **BigQuery** — 1 TB queries/month, 10 GB storage; telemetry analytics
- **Firebase Hosting** — could replace GitHub Pages for proper preview deploys; not required

All three primary components scale to zero when idle, so an inactive project costs nothing. The browser is the fast path for anything that doesn't need trust; Cloud Run is the path for anything that does.

---

## Auth

### Decision

Use **Firebase Authentication** as the first backend component.

Providers at launch:

- **Anonymous** — default on first load. Preserves the current "open tab,
  play" UX; profile persists across sessions on the same device from move one.
- **Google** — primary linkable provider. Most users have an account; good
  cross-device continuity.
- **GitHub** — secondary linkable provider. Fits the likely audience.

Later, only if justified: Apple, email/password, etc.

Anonymous auth upgrades to a permanent account via Firebase's `linkWithCredential`
/ `linkWithPopup` flow — same `uid`, same `profiles/{uid}` doc, no data loss.

### Why Firebase Auth first

- Low-friction sign-in for a static web app
- Gives a stable per-user identity (`uid`) immediately, including for anonymous sessions
- Works cleanly with Firestore rules
- Supports provider linking, so one user can sign in with multiple methods
- Cloud Run can verify callers by validating the Firebase ID token JWT

### What Firebase Auth gives us

Firebase Auth gives identity and session state, not a full app profile.

Useful built-in properties:

- `uid`
- `email`
- `displayName`
- `photoURL`
- `providerData`

This is enough to answer:

- who is signed in
- which providers are linked
- what basic profile data the provider exposed

It is **not** enough for app-specific player data such as:

- unique username
- rating / leaderboard stats
- saved preferences
- public profile settings
- game history metadata

### Identity model

Split identity into two layers:

1. **Firebase user**
   - Canonical auth identity
   - Keyed by Firebase `uid`
   - Owns sign-in state and linked providers

2. **Gomoku profile**
   - App-owned player profile
   - Stored separately in Firestore
   - Also keyed by the same `uid`

In short:

- Firebase Auth answers: **who are you?**
- Firestore profile answers: **who are you in gomoku2d?**

### Provider data vs app profile

Provider data should be treated as a **seed/default**, not as the canonical app profile.

Recommended interpretation:

- Use provider `displayName` as the initial `display_name`
- Use provider `photoURL` as the initial `avatar_url`
- Record linked providers in profile metadata
- Do **not** treat provider `displayName` as the permanent in-app username

Reasons not to use provider `displayName` as username:

- not guaranteed unique
- may be missing
- may change upstream
- may not match the player's desired in-game handle
- may be a full real name rather than a handle

### Username decision

Use two separate fields:

- `display_name`: human-readable label, seeded from provider data, editable, not unique
- `username`: app-owned handle, unique, reserved internally

For the first auth phase:

- `display_name` should be created automatically from provider data
- `username` can start as `null`
- requiring a username can be deferred until a user enters a public feature such as leaderboard, search, or public profile

This keeps onboarding low-friction without mixing provider identity with app handles.

### Initial profile shape

Suggested first-pass profile document:

```ts
type Profile = {
  uid: string;
  username: string | null;
  display_name: string;
  avatar_url: string | null;
  auth_providers: string[];
  created_at: string;
  updated_at: string;
  last_login_at: string;
};
```

Notes:

- `uid` is the document key as well as a field for convenience
- `auth_providers` stores values such as `google.com` or `github.com`
- timestamps can be stored as Firestore timestamps rather than strings in implementation

### Sign-in flows

Two paths:

**First load (anonymous):**
1. App calls `signInAnonymously()` on startup if no cached user exists
2. Firebase Auth returns a signed-in user with a stable `uid`
3. App creates `profiles/{uid}` with seeded defaults (see below)
4. Player is immediately able to play, save settings, view history, etc.

**Link to a provider (later, at the user's initiative):**
1. User clicks "Sign in with Google" / "Sign in with GitHub" from settings
2. App calls `linkWithPopup(provider)` on the current anonymous user
3. `uid` is preserved; `providerData` and `auth_providers` expand
4. `display_name` / `avatar_url` refresh from provider data *only if* the user
   hasn't explicitly edited them (respect prior edits)

**Sign-in on a new device (provider-first):**
1. User clicks "Sign in with Google" / "Sign in with GitHub"
2. Firebase Auth resolves to an existing `uid` (via provider linking) or a new one
3. App checks for `profiles/{uid}` — create seeded if missing, else refresh `last_login_at`

Seed rules for a newly created profile:

- `display_name = user.displayName ?? provider fallback ?? "Player <random suffix>"`
- `avatar_url = user.photoURL ?? null`
- `username = null`
- `auth_providers = ["anonymous"]` for first-load, provider IDs otherwise

### Account linking

Use Firebase account linking so one person can attach multiple providers to one Firebase user.

Desired behavior:

- A player who signs in with Google first can later link GitHub
- Both providers resolve to the same Firebase `uid`
- Therefore both providers resolve to the same `profiles/{uid}` document

This avoids duplicate app profiles for the same player.

### Basic rules / ownership model

Initial ownership model should be simple:

- each authenticated user can read and write their own profile
- users cannot write other users' profiles
- username uniqueness should not be trusted to pure client logic

Implication:

- profile document ownership maps directly to `request.auth.uid`
- username reservation goes through Cloud Run (see [Cloud Run](#cloud-run)) —
  transaction over `usernames/{handle} → uid` is the one place a pure-client
  approach breaks down

### Deferred decisions

These belong to later slices, not the first auth phase:

- whether username is required at first sign-in, or only when entering a
  public feature (leaderboard, profile URL, invite link)
- whether public profiles ship in v1
- whether provider avatars are cached or copied into app-owned storage (GCS)
- whether anonymous sessions older than some threshold without a linked
  provider get garbage-collected

---

## Cloud Run

Cloud Run hosts a small HTTP service written in Rust that reuses `gomoku-core`
and `gomoku-bot` directly. Same rules engine as the browser, just compiled for
Linux instead of Wasm. One source of truth for move legality, win detection,
and search.

### What lives on Cloud Run

- **Username reservation** — transaction over `usernames/{handle} → uid`. The
  first place the pure-client approach breaks down and the anchor for a
  broader "server-owned writes" pattern.
- **Replay verification** — given a submitted replay, reconstruct the game with
  the native core; reject if any move is illegal or the claimed result is
  wrong. Required before any feature that cares about trust (ranked
  leaderboard, verified match history, etc.).
- **Strong bot endpoint** — `SearchBot` at depths the browser can't afford (7+,
  or time-budgeted 3–5s). Request/response: send FEN, get back a move.
- **Puzzle generation** — offline job that mines replays for forced-win
  positions and publishes a daily puzzle doc.
- **Leaderboard aggregation** — periodic recompute from verified match results.

### What does not live on Cloud Run

- Reading / writing own profile (direct Firestore with security rules)
- Reading shared replays (direct Firestore read with `public: true`)
- Local game state and live move input (stays in the browser)
- Account sign-in flow (Firebase Auth SDK in the browser)

### Shape

- One binary, one container, one Cloud Run service to start
- `axum` or `actix-web` for HTTP
- Authenticates callers via Firebase ID token (verify JWT against Google's public keys)
- Talks to Firestore via the gRPC client (`firestore` crate or `gcloud-sdk`), authenticated through ADC — the attached runtime service account bypasses security rules by virtue of its IAM role, not by using an "admin SDK" (no first-party Firebase Admin SDK exists for Rust)
- Candidate repo location: `gomoku-api/` as a new crate in the workspace

### Deploy

Cloud Build → Artifact Registry → Cloud Run, driven from a GitHub Actions
workflow that mirrors the existing `deploy.yml` for Pages. See **CI/CD** below
for the auth story.

---

## CI/CD from GitHub Actions

Two separate credential stories, don't conflate them:

1. **Deploy credentials** — how GH Actions proves to GCP that it's allowed to
   push an image and update a Cloud Run service
2. **Runtime credentials and secrets** — what the running Cloud Run service
   uses at startup (Application Default Credentials for GCP APIs; Secret
   Manager only if we ever need a non-GCP secret)

### Project bootstrap

**One-time, manual.** Creates the GCP project, Firebase project, Firestore
database, and Artifact Registry repo, and enables the APIs everything else
depends on. Run locally with `gcloud` authenticated as an owner of the
billing account:

```sh
PROJECT=gomoku2d-prod
REGION=us-central1

# 1. Create the GCP project (or skip if it exists)
gcloud projects create $PROJECT --name="Gomoku2D"

# 2. Attach billing (required even for free-tier-only usage)
BILLING_ACCOUNT=$(gcloud billing accounts list --format='value(name)' | head -1)
gcloud billing projects link $PROJECT --billing-account=$BILLING_ACCOUNT

# 3. Enable the APIs this design depends on
gcloud services enable \
  --project=$PROJECT \
  run.googleapis.com \
  cloudbuild.googleapis.com \
  artifactregistry.googleapis.com \
  iamcredentials.googleapis.com \
  sts.googleapis.com \
  secretmanager.googleapis.com \
  firestore.googleapis.com \
  firebase.googleapis.com \
  identitytoolkit.googleapis.com

# 4. Add Firebase to the GCP project (required for Firebase Auth + Firestore rules)
#    Easiest path: Firebase Console → "Add project" → pick existing GCP project.
#    CLI equivalent needs firebase-tools: `firebase projects:addfirebase $PROJECT`

# 5. Create the Firestore database (Native mode, single-region)
gcloud firestore databases create \
  --project=$PROJECT \
  --location=$REGION \
  --type=firestore-native

# 6. Create the Artifact Registry repo the Cloud Run workflow pushes to
gcloud artifacts repositories create gomoku \
  --project=$PROJECT \
  --repository-format=docker \
  --location=$REGION \
  --description="Gomoku2D container images"
```

After this, the rest of the bootstrap snippets (WIF, runtime SA, secrets)
can run. The sections below assume these steps are done.

### Deploy credentials: Workload Identity Federation (no long-lived keys)

GCP's recommended pattern since 2023 is **Workload Identity Federation (WIF)**.
GitHub Actions has a built-in OIDC token issuer. GCP is configured to trust
tokens that come from this specific repo, exchanging them for short-lived
access tokens tied to a specific service account. No JSON key stored as a
GitHub secret, no rotation burden, credentials expire per-run.

**One-time bootstrap** (run locally with `gcloud` authenticated as project
owner). Commit-and-follow snippet — when enough of these accumulate, lift to
`infra/bootstrap.sh`:

```sh
PROJECT=gomoku2d-prod
PROJECT_NUM=$(gcloud projects describe $PROJECT --format='value(projectNumber)')
REPO=byebyebryan/gomoku2d

# 1. Service account that GH Actions will impersonate for deploys
gcloud iam service-accounts create gh-cd \
  --project=$PROJECT --display-name="GitHub Actions CD"

# 2. Grant minimal deploy permissions
for role in \
  roles/run.admin \
  roles/artifactregistry.writer \
  roles/cloudbuild.builds.editor \
  roles/iam.serviceAccountUser \
  roles/firebaserules.admin
do
  gcloud projects add-iam-policy-binding $PROJECT \
    --member="serviceAccount:gh-cd@$PROJECT.iam.gserviceaccount.com" \
    --role=$role
done

# 3. Workload Identity Pool + OIDC provider for GitHub
gcloud iam workload-identity-pools create github \
  --project=$PROJECT --location=global

gcloud iam workload-identity-pools providers create-oidc github \
  --project=$PROJECT --location=global \
  --workload-identity-pool=github \
  --issuer-uri=https://token.actions.githubusercontent.com \
  --attribute-mapping="google.subject=assertion.sub,attribute.repository=assertion.repository" \
  --attribute-condition="assertion.repository_owner == 'byebyebryan'"

# 4. Allow only this specific repo to impersonate the SA
gcloud iam service-accounts add-iam-policy-binding \
  gh-cd@$PROJECT.iam.gserviceaccount.com \
  --project=$PROJECT \
  --role=roles/iam.workloadIdentityUser \
  --member="principalSet://iam.googleapis.com/projects/$PROJECT_NUM/locations/global/workloadIdentityPools/github/attribute.repository/$REPO"
```

Record two non-secret values as GitHub Actions **repository variables** (not
secrets — nothing sensitive, just identifiers):

- `WIF_PROVIDER = projects/${PROJECT_NUM}/locations/global/workloadIdentityPools/github/providers/github`
- `CD_SERVICE_ACCOUNT = gh-cd@${PROJECT}.iam.gserviceaccount.com`

### Workflow shape

`.github/workflows/deploy-api.yml`:

```yaml
name: Deploy gomoku-api
on: workflow_dispatch

permissions:
  contents: read
  id-token: write   # required for OIDC

env:
  PROJECT: gomoku2d-prod
  REGION: us-central1
  IMAGE: us-central1-docker.pkg.dev/gomoku2d-prod/gomoku/api

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Authenticate to GCP
        uses: google-github-actions/auth@v2
        with:
          workload_identity_provider: ${{ vars.WIF_PROVIDER }}
          service_account: ${{ vars.CD_SERVICE_ACCOUNT }}

      - uses: google-github-actions/setup-gcloud@v2

      - name: Build container
        run: |
          gcloud builds submit gomoku-api \
            --tag=${IMAGE}:${{ github.sha }}

      - name: Deploy
        uses: google-github-actions/deploy-cloudrun@v2
        with:
          service: gomoku-api
          image: ${{ env.IMAGE }}:${{ github.sha }}
          region: ${{ env.REGION }}
```

No `credentials_json`, no `secrets.GCP_SA_KEY`. The OIDC exchange happens
inside `google-github-actions/auth@v2`.

### Runtime credentials and secrets

What the service needs to talk to GCP is handled by **Application Default
Credentials (ADC)** via the runtime service account — not Secret Manager. A
Cloud Run service authenticates to Firestore, Cloud Storage, Pub/Sub, etc.
automatically using the SA attached at deploy time. No key file, no secret,
nothing to rotate. The one-time bootstrap just creates the SA and grants the
roles it needs:

```sh
# Runtime SA, distinct from gh-cd
gcloud iam service-accounts create gomoku-api-runtime --project=$PROJECT

# Grant it Firestore access (example — narrow to the specific roles the service needs)
gcloud projects add-iam-policy-binding $PROJECT \
  --member="serviceAccount:gomoku-api-runtime@$PROJECT.iam.gserviceaccount.com" \
  --role=roles/datastore.user

# Allow gh-cd to deploy services that run as this SA
gcloud iam service-accounts add-iam-policy-binding \
  gomoku-api-runtime@$PROJECT.iam.gserviceaccount.com \
  --member="serviceAccount:gh-cd@$PROJECT.iam.gserviceaccount.com" \
  --role=roles/iam.serviceAccountUser
```

**Secret Manager** is the right tool for *app-level* secrets that don't come
from GCP itself — third-party API keys, webhook signing secrets, a
self-generated JWT signing key, etc. None of phases 1–5 require one; this
block is aspirational, documented so the pattern is ready when we do need it:

```sh
# Create a secret (example placeholder — no real secret in phase 1–5)
printf '%s' "$SOME_THIRD_PARTY_KEY" | \
  gcloud secrets create third-party-key --data-file=-

# Grant the runtime SA access
gcloud secrets add-iam-policy-binding third-party-key \
  --member="serviceAccount:gomoku-api-runtime@$PROJECT.iam.gserviceaccount.com" \
  --role=roles/secretmanager.secretAccessor
```

Wire it in on deploy:

```yaml
- name: Deploy
  uses: google-github-actions/deploy-cloudrun@v2
  with:
    service: gomoku-api
    image: ${{ env.IMAGE }}:${{ github.sha }}
    region: ${{ env.REGION }}
    flags: |
      --service-account=gomoku-api-runtime@gomoku2d-prod.iam.gserviceaccount.com
      --set-secrets=THIRD_PARTY_KEY=third-party-key:3
```

**Pin a specific version (e.g. `:3`), not `:latest`.** `:latest` means any new
secret version auto-rolls out on the next cold start, which is convenient but
silently propagates bad versions. Pinned versions make rollout explicit and
deploy-gated. Secret rotations then look like: (1) add a new version via a
committed `rotate-secret.yml` workflow, (2) update the pin in `deploy-api.yml`,
(3) deploy. Three steps, all in PRs, all reviewable.

### Firestore security rules

Rules are deployable code and belong in the repo alongside everything else.
Committed location: `firestore.rules` at the repo root (paired with
`firestore.indexes.json` if indexes are needed).

**One-time bootstrap:** install `firebase-tools` and run `firebase init
firestore` once locally to generate the initial files, then commit them.

**CD:** a committed workflow publishes rules on change.
`.github/workflows/deploy-rules.yml`:

```yaml
name: Deploy Firestore rules
on: workflow_dispatch

permissions:
  contents: read
  id-token: write

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Authenticate to GCP
        uses: google-github-actions/auth@v2
        with:
          workload_identity_provider: ${{ vars.WIF_PROVIDER }}
          service_account: ${{ vars.CD_SERVICE_ACCOUNT }}

      - uses: actions/setup-node@v4
        with:
          node-version: 20

      - run: npm install -g firebase-tools
      - run: firebase deploy --only firestore:rules --project gomoku2d-prod
```

The `roles/firebaserules.admin` grant is already in the WIF bootstrap role
loop above, so `gh-cd` can deploy rules without further setup.

Starting rules for phase 1 (profile + anonymous/linked auth):

```
rules_version = '2';
service cloud.firestore {
  match /databases/{database}/documents {
    match /profiles/{uid} {
      allow read: if request.auth != null && request.auth.uid == uid;
      allow write: if request.auth != null
        && request.auth.uid == uid
        && request.resource.data.uid == uid;
    }
  }
}
```

Rules widen as features land — replay sharing adds public reads on
`replays/{id}`, match history adds `profiles/{uid}/matches/{id}`, and so on.
Each feature PR should touch both code and rules in the same commit.

### Two service accounts, clear separation

| SA | Used by | Permissions | Cannot |
|----|---------|-------------|--------|
| `gh-cd` | GH Actions (during deploys) | Deploy Cloud Run, push images, impersonate runtime SA | Read user data, read runtime secrets |
| `gomoku-api-runtime` | The running Cloud Run service | Read specific Secret Manager secrets, Firestore access as configured | Deploy anything, modify infrastructure |

Least privilege on both. Neither can do the other's job, so a leak of one
doesn't compromise the other.

### Why not a JSON key in a GH secret?

The older pattern — `credentials_json: ${{ secrets.GCP_SA_KEY }}` — still
works and is a one-line setup. Trade-offs:

- Long-lived credential sitting in GitHub's secret store
- Rotation is manual; revocation after a leak is manual
- No audit trail tying deploys to specific workflow runs

For an experimental project either works. WIF is the right default because
it's the pattern worth learning once, and it's also noticeably cleaner to
revoke — detach the IAM policy binding from the pool provider and every repo
loses access at once.

---

## Feature catalog

Every feature below is scoped as a standalone increment. The phase order in
the next section picks the sequence; this section is the menu.

### Auth + profile

Covered above. Produces: `profiles/{uid}` with display_name, avatar_url,
providers, timestamps. Opens the door for everything else.

### Replay sharing

Persist a finished replay and return a short URL. Paste to a friend, they see
the game step through.

- **Backend:** Firestore `replays/{id}` (public-read); client writes own replay
  with `owner_uid == request.auth.uid`
- **Frontend:** "Share" button on result screen → copy URL; landing page loads
  the replay and plays it back using the same board code
- **Open:** URL shape (random id vs. slug), expiration policy, private vs.
  public toggle

### Match history on profile

Every completed game writes a replay doc under the owning profile. Profile page
shows recent matches with outcome, bot preset, duration.

- **Backend:** Firestore `profiles/{uid}/matches/{id}` (owner-read/write)
- **Frontend:** new profile view in `gomoku-web`; pagination by `created_at`
- **Open:** what counts as a match worth saving (all? only vs. bot?), retention
  policy

### Server-side replay verifier

Cloud Run endpoint that revalidates a replay before it's marked as trusted.
Gates any ranked feature.

- **Backend:** `POST /verify` — takes replay JSON, runs it through `gomoku-core`,
  returns `{ valid: bool, result, hash }`
- **Frontend:** submitted alongside replay writes that need trust
- **Open:** whether *all* replay writes go through the verifier, or only ones
  destined for leaderboards

### Leaderboard

Aggregate verified human-vs-bot results into a public leaderboard.

- **Backend:** Firestore `leaderboard/{scope}` (public-read, Cloud Run
  recomputes); scoped by bot preset (RANDOM, EASY, MEDIUM, STRONG) or a
  combined Elo-like rating
- **Frontend:** leaderboard view, rank indicator on profile
- **Open:** scoring (win count? win rate? Elo?), scope (daily/weekly/all-time),
  handling untrusted replays

### Cloud-synced settings

Persist preset choice, default variant, name overrides, etc. across devices.

- **Backend:** inside the existing profile doc
- **Frontend:** write on change, read on load
- **Open:** conflict handling when two devices edit simultaneously (probably:
  last-write-wins is fine)

### Daily puzzle

Curated position distributed to every user; solve → stats persist.

- **Backend:** Cloud Run job mines `gomoku-eval` replays for forced-win or
  forced-block positions; writes `puzzles/{yyyy-mm-dd}` with FEN, solution,
  and hint metadata; `puzzle_attempts/{uid}/{date}` tracks each user's result
- **Frontend:** new puzzle mode, restricted to a single move (or short
  sequence)
- **Open:** puzzle difficulty tiers, streak mechanics, whether failed attempts
  should block retries

### Stronger bot

Cloud Run endpoint serves `SearchBot` at higher depth than the browser can
sustain.

- **Backend:** `POST /bot/move` — takes FEN + difficulty knob, returns move and
  search trace
- **Frontend:** new preset (e.g., CHAMPION) that routes to the API instead of
  the local worker; falls back gracefully if offline
- **Open:** auth requirement (signed-in only? rate-limited anonymous?), cost
  of vCPU-seconds per match

### Async / correspondence play

Turn-based human vs. human across time, Firestore-listener driven.

- **Backend:** Firestore `matches/{id}` with `black_uid`, `white_uid`, move
  list, turn pointer; security rules enforce that only the player whose turn
  it is can append
- **Frontend:** "Invite" flow (share a URL with a match id), live updates via
  snapshot listener
- **Open:** timeouts, abandonment policy, notifications (browser push? email?
  none?)

### Telemetry / BigQuery

Opt-in anonymous match stream for understanding real play patterns — feeds
bot calibration.

- **Backend:** Firestore → BigQuery export, opt-in flag on profile
- **Frontend:** opt-in toggle in settings
- **Open:** everything; probably premature until there's actual traffic

---

## Suggested phase order

1. **Auth + profile** — Anonymous auth on first load; optional Google/GitHub
   linking from settings; Firestore profile doc seeded on first sign-in. Done
   when every visitor has a persistent `profiles/{uid}` from their first
   interaction, and can optionally link a provider to keep it across devices.
2. **Replay sharing** — Firestore-backed, public reads, shareable URL.
   Highest-payoff-per-effort feature; no trust surface. Done when a replay
   URL plays back correctly for any visitor.
3. **Match history on profile** — reuse the same replay plumbing; profile
   view lists recent matches. Done when a signed-in user can see their last
   N games.
4. **Cloud Run bring-up + username reservation** — stand up the service,
   wire JWT verification, ship the `POST /reserve_username` endpoint. Done
   when a user can set a unique username via the API.
5. **Replay verifier** — `POST /verify`, gated writes for trusted replays.
   Done when a verified flag is set on a persisted replay.
6. **One of:** leaderboard, daily puzzle, or correspondence play — pick by
   what's more fun to build at the time.

Everything after that is opportunistic.

---

## Open questions for the whole design

- **Trust model for leaderboards.** Profile, match history, and public replay
  reads have no trust surface — a user writing lies to their own match history
  only fools themselves. But leaderboard entries do: a client that can write
  `leaderboard/*` directly can write any score. Two options: (a) leaderboard
  writes go through Cloud Run's `POST /submit_score` (server verifies the
  underlying replay, then writes); (b) client writes to a pending collection,
  a background Cloud Run worker verifies and promotes to `leaderboard/*`.
  (a) is simpler and matches the "Cloud Run as first-class" stance.
- **Project structure.** `gomoku-api/` as a new crate in the workspace, or a
  sibling repo? Crate keeps the Rust core as a single `path = "../gomoku-core"`
  dependency; separate repo is cleaner to deploy independently. Leaning
  toward crate-in-workspace given how small this is.
