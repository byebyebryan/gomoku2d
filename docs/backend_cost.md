# Backend Cost And Free-Tier Headroom

Scope: track the cost shape of backend pieces as they are added. The goal is
to keep Gomoku2D comfortably inside free tiers while the project is small, and
to notice early when a feature changes the cost profile.

Live setup state lives in `backend_infra.md`; backend architecture and trust
lanes live in `backend.md`.

This is an estimate document, not billing truth. Re-check official pricing
before each backend release and use the Firebase/GCP usage dashboards after
deploying.

Sources:

- [Cloud Firestore billing](https://firebase.google.com/docs/firestore/pricing)
- [Cloud Firestore quotas](https://docs.cloud.google.com/firestore/quotas)
- [Firebase pricing plans](https://firebase.google.com/docs/projects/billing/firebase-pricing-plans)
- [Firebase Authentication](https://firebase.google.com/docs/auth/)
- [Cloud Run pricing](https://cloud.google.com/run/pricing)
- [Google Cloud free tier](https://cloud.google.com/free/docs/gcp-free-tier)

## Current Backend Footprint

Project: `gomoku2d`

| Piece | Current state | Cost posture |
|---|---|---|
| Firebase project | Linked to GCP project `gomoku2d` | No direct cost |
| Firebase web app | `Gomoku2D Web` | No direct cost |
| Firebase Auth | Initialized as Identity Platform; Google provider pending OAuth config | Billing is enabled; stay inside no-cost MAU tier and avoid phone/SAML/OIDC |
| Firestore | Default Native-mode database in `us-central1`; `freeTier: true` | Uses the one free Firestore database for the project |
| Firestore rules | Repo rules deployed to `cloud.firestore` release | No runtime cost |
| Cloud Run | Not created in `v0.3` | Future `v0.4+`; billing is enabled, but no service exists yet |

## Firestore Free Tier

Firestore free quota resets daily around midnight Pacific time. A project gets
exactly one free database.

| Resource | Free quota |
|---|---:|
| Stored data | 1 GiB |
| Document reads | 50,000 / day |
| Document writes | 20,000 / day |
| Document deletes | 20,000 / day |
| Outbound data transfer | 10 GiB / month |

Features to avoid while we are intentionally staying free-tier-first:

- TTL deletes
- point-in-time recovery
- backups/restores/clones
- extra Firestore databases

## Expected `v0.3` Firestore Usage

The `v0.3` cloud path is private profile/history continuity, not live gameplay.
Local guest play has no backend cost.

Estimated operation budgets:

| Action | Expected Firestore reads | Expected Firestore writes | Notes |
|---|---:|---:|---|
| First sign-in / cloud profile create | 1 | 1-2 | Read-or-create profile, maybe update login timestamp/settings |
| Later sign-in | 1 | 0-1 | Load profile, maybe update `last_login_at` |
| Promote guest settings | 0-1 | 1 | Profile/settings merge |
| Promote one local match | 0-1 | 1 | Stable `local_origin_id` makes retry safe |
| Save one finished signed-in match | 0 | 1-2 | Match record, maybe profile summary update |
| Open Profile history | Up to current history page size | 0 | Reads one document per returned match |
| Open one saved replay directly | 1 | 0 | If not already loaded from history |

Initial implementation guardrails:

- Limit history queries, starting with the existing local cap of 24 records.
- Avoid always-on listeners for full match history unless the UI actually needs
  live updates.
- Prefer one write at match end over per-move cloud writes for casual play.
- Keep public replay publishing out of `v0.3`; private history and public
  shareables have different cost and trust profiles.

## Headroom Scenarios

These are deliberately rough. They are meant to catch order-of-magnitude
mistakes before implementation.

### Small Public Release

Assumption: 100 signed-in users/day, each opens Profile twice and saves 5
matches.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~5,000/day | ~10x below 50,000/day |
| Writes | ~1,200/day | ~16x below 20,000/day |
| Deletes | 0/day | No concern |

This is safely inside the free tier.

### Profile-Heavy Usage

Assumption: 500 signed-in users/day, each opens a 24-match history page 5 times.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~60,000/day | Exceeds 50,000/day |
| Writes | Depends on matches saved | Usually still fine |

If this becomes plausible, add pagination, caching, or a cheaper summary view
before adding more history-heavy surfaces.

### Replay Sharing / Online Future

Public replay pages, online match subscriptions, and trusted server-written
match state can change the read/write profile quickly. Before `v0.4` online
work, add a separate estimate for:

- active online match listeners
- per-move server writes
- public replay page reads
- profile/ranked leaderboard queries

## Auth Cost Notes

The project is currently initialized with an Identity Platform Auth config and
has Cloud Billing enabled. This is still safe for the expected scale, but cost
control now depends on staying inside the no-cost tier and monitoring usage,
not on Spark-plan shutoff behavior.

Current no-cost planning assumptions:

- Email, social, anonymous, and custom auth have a no-cost tier of 50,000 MAUs
  on Blaze / Identity Platform.
- SAML/OIDC has a much smaller no-cost tier and is out of scope.

Avoid:

- phone auth
- SAML/OIDC providers

Re-check Auth pricing before adding any provider beyond Google/GitHub social
sign-in.

## Cloud Run Future Budget

Cloud Run is intentionally out of `v0.3`. When it arrives, use
request-based billing in `us-central1` and keep minimum instances at zero.

Current free tier to plan against:

| Resource | Free quota |
|---|---:|
| Requests | 2,000,000 / month |
| CPU | 180,000 vCPU-seconds / month |
| Memory | 360,000 GiB-seconds / month |
| North America outbound transfer | 1 GiB / month |

Guardrails for future Cloud Run work:

- no always-on minimum instances unless we intentionally accept cost
- short request timeouts for bot/analysis endpoints
- no per-move Cloud Run calls for casual/local play
- use Cloud Run only for trust boundaries or work the browser cannot safely do

## Cost Review Checklist

Before each backend release:

- Check Firebase Console usage for Firestore reads/writes/storage.
- Check GCP billing dashboard if billing is enabled.
- Re-read official pricing pages if adding a new backend product.
- Add a row to this doc for the new backend feature.
- Decide whether the feature needs a read/write cap, pagination, cache, or
  manual rate limit.
