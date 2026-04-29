# Backend Cost And Free-Tier Headroom

Scope: track the cost shape of backend pieces as they are added. The goal is
to keep Gomoku2D comfortably inside free tiers while the project is small, and
to notice early when a feature changes the cost profile.

Live setup state lives in `backend_infra.md`; backend architecture and trust
lanes live in `backend.md`.

This is an estimate document, not billing truth. Re-check official pricing
before each backend release and use the Firebase/GCP usage dashboards after
deploying. Pricing assumptions below were last re-checked against Firebase's
official pricing and Firestore billing docs on 2026-04-29.

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
| Firebase Auth | Initialized as Identity Platform; Google provider enabled | Billing is enabled; stay inside no-cost MAU tier and avoid phone/SAML/OIDC |
| Firestore | Default Native-mode database in `us-central1`; `freeTier: true` | Uses the one free Firestore database for the project |
| Firestore rules | Repo rules deployed to `cloud.firestore` release | No runtime cost |
| Cloud Run | Not created in `v0.3` | Future online/trusted-match phase, or earlier only if lab-powered analysis needs server compute; billing is enabled, but no service exists yet |

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

Counting assumptions:

- Firestore bills document reads, writes, and deletes separately.
- A query has a one-read minimum even when it returns no documents.
- Security rules can add dependent reads. Our private match-create rules read
  the owner profile through `exists()` / `get()` to enforce the
  `history_reset_at` reset barrier. Count accepted private match creates as one
  additional dependent profile read.
- Repeated rule references to the same dependent document are charged once per
  request, not once per helper call.
- Replay currently resolves from the active Profile/history cache; there is no
  dedicated cloud replay fetch route in `v0.3.2`.

Estimated operation budgets for the current implementation:

| Action | Expected reads | Expected writes | Expected deletes | Notes |
|---|---:|---:|---:|---|
| Guest local play / local match finish | 0 | 0 | 0 | LocalStorage only |
| No-config build / signed-out Profile | 0 | 0 | 0 | Firebase bootstrap stays disabled |
| First sign-in / cloud profile create | 1 | 1 | 0 | `getDoc(profiles/{uid})`, then create profile |
| Cloud profile load / refresh | 1 | 1 | 0 | `getDoc(profiles/{uid})`, then merge login/provider/settings metadata; currently happens on signed-in Profile/Replay surfaces |
| Signed-in Profile history load | `1..24` | 0 | 0 | `limit(24)` query; one-read minimum if empty |
| Promote guest profile/settings | 0 | 1 | 0 | Profile merge before match import loop |
| Promote one eligible local match | `2..3` | 1 | 0 | Client checks `guest_import` and `cloud_saved` IDs, then match create adds one dependent profile read through rules |
| Skip one already-promoted local match | `1..2` | 0 | 0 | Stops after first matching deterministic ID |
| Save one finished signed-in match | 2 | 1 | 0 | Client existence read + dependent profile read through rules + `cloud_saved` document write |
| Retry a pending signed-in save | 2 | 0-1 | 0 | Same as direct save; no write if the match already exists |
| Open Replay from loaded history | 0 | 0 | 0 | Uses active local/cloud history cache |
| Reset signed-in profile with `D` private matches | `2 + max(1, D)` | 1 | `D` | Profile get + profile refresh get + profile reset write + query/read/delete private `client_uploaded` matches |

Initial implementation guardrails:

- Limit history queries, starting with the existing local cap of 24 records.
- Avoid always-on listeners for full match history unless the UI actually needs
  live updates.
- Prefer one write at match end over per-move cloud writes for casual play.
- Keep public replay publishing out of `v0.3`; private history and public
  shareables have different cost and trust profiles.
- Budget match creates as also reading the owner profile in rules. Firestore
  rules use the profile's `history_reset_at` as a server-side reset barrier, so
  the dashboard should be checked after real promotion/save smoke tests.

## `v0.3.2` Cost Formulas

These formulas are intentionally conservative enough for release planning. They
count private match creates with the extra dependent rules read noted above.

Let:

- `H = 24`, the current signed-in history query limit.
- `P = profile opens per signed-in user per day`.
- `M = new signed-in matches saved per user per day`.
- `G = one-time guest matches imported on first promotion`.
- `D = private matches cleared by Reset Profile`.

Approximate daily operations per signed-in user:

| Flow | Reads | Writes | Deletes |
|---|---:|---:|---:|
| Signed-in Profile opens | `P * (H + 1)` | `P` | `0` |
| New signed-in matches | `M * 2` | `M` | `0` |
| First guest promotion | `G * 3` | `1 + G` | `0` |
| Reset Profile | `2 + max(1, D)` | `1` | `D` |

The first likely bottleneck is repeated history reads, not match writes. A user
who opens Profile five times with a full 24-match history costs about 125 reads
and 5 profile refresh writes even if they play no new games. By contrast,
saving five signed-in matches costs about 10 reads and 5 writes.

## Headroom Scenarios

These are deliberately rough. They are meant to catch order-of-magnitude
mistakes before implementation.

### Small Public Release

Assumption: 100 signed-in users/day, each opens Profile twice and saves 5
matches.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~6,000/day | ~8x below 50,000/day |
| Writes | ~700/day | ~28x below 20,000/day |
| Deletes | 0/day | No concern |

This is safely inside the free tier.

### Profile-Heavy Usage

Assumption: 500 signed-in users/day, each opens a 24-match history page 5 times.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~62,500/day | Exceeds 50,000/day |
| Writes | ~2,500/day before match saves | Still below 20,000/day |

If this becomes plausible, add pagination, caching, or a cheaper summary view
before adding more history-heavy surfaces.

### One-Time Promotion Spike

Assumption: 100 existing local players sign in for the first time with 24 local
matches each.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~7,300 one-time reads | ~6x below 50,000/day |
| Writes | ~2,600 one-time writes | ~7x below 20,000/day |
| Deletes | 0 | No concern |

This is fine for current scale, but promotion is intentionally a one-time import
path. If the app ever has thousands of existing local players, staggered rollout
or smaller import batches would be safer.

### Reset-Heavy Debug Day

Assumption: 20 signed-in test profiles each reset a full 24-match private
history during QA.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~520/day | No concern |
| Writes | ~20/day | No concern |
| Deletes | ~480/day | No concern |

Reset is not a cost risk at current scale. The bigger risk is correctness:
reset-barrier writes and stale-match rejection must stay covered by rules tests.

### Replay Sharing / Online Future

Public replay pages, online match subscriptions, and trusted server-written
match state can change the read/write profile quickly. Before the online
product-expansion phase, add a separate estimate for:

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

`v0.3.0` note: Firebase/Auth/Firestore dashboards were reviewed after the
public sign-in smoke test and looked normal for the first cloud-profile slice.

`v0.3.1` note: the first local-build guest promotion smoke imported 24 private
`guest_import` matches for one signed-in profile. This is well inside the free
tier; the cost-relevant next check is repeated promotion/sign-in behavior and
future `cloud_saved` writes from newly finished signed-in matches.

`v0.3.2` note: production and local-build smoke covered signed-in
`cloud_saved` writes, cloud-history reload, Reset Profile deletion/barrier
behavior, and post-reset saves. The operation profile still matches the
estimates above: one finished signed-in match writes one private match document
after a small number of dedupe/profile reads, and Reset Profile pays one profile
write plus one delete per private `client_uploaded` match cleared.
